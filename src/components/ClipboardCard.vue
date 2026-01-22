<script setup lang="ts">
import { computed, ref } from 'vue';
import type { ClipboardItem } from '@/types';
import { usePinboardStore } from '@/stores/pinboards';
import { startDrag } from '@crabnebula/tauri-plugin-drag';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  item: ClipboardItem;
  selected?: boolean;
}>();

const emit = defineEmits<{
  select: [item: ClipboardItem];
  copy: [item: ClipboardItem];
  delete: [id: string];
}>();

const pinboardStore = usePinboardStore();

// Refs
const cardRef = ref<HTMLElement | null>(null);

// Drag state
const isDragging = ref(false);

// Format timestamp for display
const formattedTime = computed(() => {
  const date = new Date(props.item.created_at);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
});

// Get truncated text preview
const textPreview = computed(() => {
  if (props.item.content_type !== 'text' || !props.item.content_text) return '';
  const text = props.item.content_text;
  if (text.length <= 80) return text;
  return text.substring(0, 80) + '...';
});

// Get header label (type or filename)
const headerLabel = computed(() => {
  const type = props.item.content_type;

  // For files, show the filename
  if (type === 'files' && props.item.content_text) {
    try {
      const files = JSON.parse(props.item.content_text) as string[];
      if (files.length > 0) {
        const name = files[0].split(/[/\\]/).pop() || files[0];
        return files.length > 1 ? `${name} +${files.length - 1}` : name;
      }
    } catch {
      // Fall through to default
    }
  }

  // For audio files, show the filename
  if (type === 'audio' && props.item.content_text) {
    try {
      const files = JSON.parse(props.item.content_text) as string[];
      if (files.length > 0) {
        const name = files[0].split(/[/\\]/).pop() || files[0];
        return files.length > 1 ? `${name} +${files.length - 1}` : name;
      }
    } catch {
      // Fall through to default
    }
  }

  // Default type labels
  const typeLabels: Record<string, string> = {
    text: 'Text',
    image: 'Image',
    files: 'File',
    link: 'Link',
    audio: 'Audio',
    video: 'Video',
  };

  return typeLabels[type] || 'Unknown';
});

// Get URL preview (domain only)
const urlPreview = computed(() => {
  if (props.item.content_type !== 'link' || !props.item.content_text) return '';
  const url = props.item.content_text;
  try {
    // Handle www. prefix
    const urlToParse = url.startsWith('www.') ? `https://${url}` : url;
    const parsed = new URL(urlToParse);
    return parsed.hostname.replace('www.', '');
  } catch {
    return url.length <= 40 ? url : url.substring(0, 40) + '...';
  }
});

// Get audio info
const audioInfo = computed(() => {
  if (props.item.content_type !== 'audio' || !props.item.content_text) {
    return { count: 0, names: [] as string[] };
  }
  try {
    const files = JSON.parse(props.item.content_text) as string[];
    const names = files.map((f) => f.split(/[/\\]/).pop() || f);
    return { count: files.length, names };
  } catch {
    return { count: 0, names: [] as string[] };
  }
});

// Get file info
const fileInfo = computed(() => {
  if (props.item.content_type !== 'files' || !props.item.content_text) {
    return { count: 0, names: [] as string[] };
  }
  try {
    const files = JSON.parse(props.item.content_text) as string[];
    const names = files.map((f) => f.split(/[/\\]/).pop() || f);
    return { count: files.length, names };
  } catch {
    return { count: 0, names: [] as string[] };
  }
});

// Check if item has a visual preview (image or file with thumbnail)
const hasVisualPreview = computed(() => {
  if (props.item.content_type === 'image' && props.item.thumbnail_base64) {
    return true;
  }
  if (props.item.content_type === 'files' && props.item.thumbnail_base64) {
    return true;
  }
  return false;
});

// Handle card click
// Handle double click to copy
const handleDoubleClick = () => {
  emit('copy', props.item);
};

// Handle delete
const handleDelete = (e: Event) => {
  e.stopPropagation();
  emit('delete', props.item.id);
};

// Handle card click
const handleClick = () => {
  emit('select', props.item);
};

// Store for cleanup
let dragClone: HTMLElement | null = null;

// Copy computed styles from source to target element (inline)
const copyStyles = (source: Element, target: HTMLElement) => {
  const computed = window.getComputedStyle(source);
  for (const prop of computed) {
    try {
      target.style.setProperty(prop, computed.getPropertyValue(prop));
    } catch {
      // Some properties can't be set
    }
  }
};

// Recursively copy styles to all descendants
const deepCopyStyles = (source: Element, target: Element) => {
  copyStyles(source, target as HTMLElement);
  const sourceChildren = source.children;
  const targetChildren = target.children;
  for (let i = 0; i < sourceChildren.length && i < targetChildren.length; i++) {
    deepCopyStyles(sourceChildren[i], targetChildren[i]);
  }
};

// Create an exact clone of the card with inline styles
const createExactClone = (): HTMLElement | null => {
  if (!cardRef.value) return null;

  const rect = cardRef.value.getBoundingClientRect();

  // Clone the entire card
  const clone = cardRef.value.cloneNode(true) as HTMLElement;

  // Copy all computed styles inline (this isolates the clone from CSS)
  deepCopyStyles(cardRef.value, clone);

  // Set fixed dimensions and position
  clone.style.position = 'fixed';
  clone.style.left = '-9999px';
  clone.style.top = '-9999px';
  clone.style.width = `${rect.width}px`;
  clone.style.height = `${rect.height}px`;
  clone.style.margin = '0';
  clone.style.transform = 'none';
  clone.style.pointerEvents = 'none';
  clone.style.zIndex = '99999';
  clone.style.opacity = '1';
  clone.style.boxShadow = '0 8px 24px rgba(0, 0, 0, 0.2)';

  return clone;
};

// Sanitize a string to be safe for filenames
const sanitizeFilename = (name: string): string => {
  return name
    .replace(/[<>:"/\\|?*]/g, '')
    .replace(/\s+/g, '_')
    .substring(0, 50);
};

// Generate a readable filename from item metadata
const generateReadableFilename = (item: ClipboardItem): string => {
  const sourceApp = item.source_app ? sanitizeFilename(item.source_app) : 'Image';
  const date = new Date(item.created_at);
  const timestamp = [
    date.getFullYear(),
    String(date.getMonth() + 1).padStart(2, '0'),
    String(date.getDate()).padStart(2, '0'),
    '_',
    String(date.getHours()).padStart(2, '0'),
    String(date.getMinutes()).padStart(2, '0'),
    String(date.getSeconds()).padStart(2, '0'),
  ].join('');

  const extension = item.image_path?.split('.').pop() || 'png';
  return `${sourceApp}_${timestamp}.${extension}`;
};

// Prepare image for drag by copying to temp with readable name
// Returns { imagePath, iconPath } for separate drag item and icon
const prepareImageForDrag = async (
  item: ClipboardItem
): Promise<{ imagePath: string; iconPath: string } | null> => {
  console.log('═══════════════════════════════════════════════════════════');
  console.log('[DEBUG prepareImageForDrag] CALLED');
  console.log('[DEBUG]   item.id:', item.id);
  console.log('[DEBUG]   item.content_type:', item.content_type);
  console.log('[DEBUG]   item.image_path:', item.image_path);

  if (!item.image_path) {
    console.log('[DEBUG]   No image_path, returning null');
    return null;
  }

  try {
    const readableFilename = generateReadableFilename(item);
    console.log('[DEBUG]   readableFilename:', readableFilename);
    console.log('[DEBUG]   Calling Rust prepare_image_for_drag...');

    const [imagePath, iconPath] = await invoke<[string, string]>('prepare_image_for_drag', {
      sourcePath: item.image_path,
      readableFilename,
    });

    console.log('[DEBUG]   Rust returned:');
    console.log('[DEBUG]     imagePath:', imagePath);
    console.log('[DEBUG]     iconPath:', iconPath);
    console.log('═══════════════════════════════════════════════════════════');

    return { imagePath, iconPath };
  } catch (err) {
    console.error('[DEBUG]   ERROR from Rust:', err);
    // Fallback: use same path for both
    return { imagePath: item.image_path, iconPath: item.image_path };
  }
};

// Get file paths for native drag (files and audio - NOT images)
const getFilePaths = (): string[] => {
  const item = props.item;

  // For files, parse the JSON array from content_text
  if (item.content_type === 'files' && item.content_text) {
    try {
      const files = JSON.parse(item.content_text) as string[];
      return files.filter((f) => f && typeof f === 'string');
    } catch {
      return [];
    }
  }

  // For audio files
  if (item.content_type === 'audio' && item.content_text) {
    try {
      const files = JSON.parse(item.content_text) as string[];
      return files.filter((f) => f && typeof f === 'string');
    } catch {
      return [];
    }
  }

  return [];
};

// Get file paths for drag, with async handling for images
// Returns { items, icon } - items is array of file paths, icon is the preview image
const getFilePathsForDrag = async (): Promise<{ items: string[]; icon: string }> => {
  const item = props.item;

  console.log('───────────────────────────────────────────────────────────');
  console.log('[DEBUG getFilePathsForDrag] CALLED');
  console.log('[DEBUG]   content_type:', item.content_type);

  // For images, prepare with readable filename and separate icon
  if (item.content_type === 'image' && item.image_path) {
    console.log('[DEBUG]   Type=image, calling prepareImageForDrag...');
    const prepared = await prepareImageForDrag(item);
    if (prepared) {
      console.log('[DEBUG]   Returning for image drag:');
      console.log('[DEBUG]     items:', [prepared.imagePath]);
      console.log('[DEBUG]     icon:', prepared.iconPath);
      return { items: [prepared.imagePath], icon: prepared.iconPath };
    }
    console.log('[DEBUG]   prepareImageForDrag returned null!');
    return { items: [], icon: '' };
  }

  // For other types, use sync method (icon = first file)
  const paths = getFilePaths();
  console.log('[DEBUG]   Type=files/audio, using getFilePaths:');
  console.log('[DEBUG]     paths:', paths);
  console.log('[DEBUG]     icon will be:', paths[0] || '(empty)');
  return { items: paths, icon: paths[0] || '' };
};

// Check if item can be dragged as native files
const canDragAsFiles = computed(() => {
  const item = props.item;
  // Images can always be dragged (will be prepared async)
  if (item.content_type === 'image' && item.image_path) {
    return true;
  }
  // For other types, check sync paths
  const paths = getFilePaths();
  return paths.length > 0;
});

// Native drag state
let dragStartPos: { x: number; y: number } | null = null;
let dragStarted = false;
const DRAG_THRESHOLD = 5; // pixels before drag starts

// Handle native file drag (for files, images, audio)
const handleNativeDragStart = (e: MouseEvent) => {
  if (!canDragAsFiles.value) return;
  if (e.button !== 0) return;

  // Record start position for drag detection
  dragStartPos = { x: e.clientX, y: e.clientY };
  dragStarted = false;

  // Add listeners for drag detection
  document.addEventListener('mousemove', handleNativeDragMove);
  document.addEventListener('mouseup', handleNativeDragEnd);
};

const handleNativeDragMove = async (e: MouseEvent) => {
  if (!dragStartPos || dragStarted) return;

  const dx = Math.abs(e.clientX - dragStartPos.x);
  const dy = Math.abs(e.clientY - dragStartPos.y);

  // Only start drag if moved beyond threshold
  if (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD) {
    dragStarted = true;

    console.log('▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶');
    console.log('[DEBUG handleNativeDragMove] DRAG THRESHOLD EXCEEDED');

    // Use async method to get paths (handles image renaming)
    const { items, icon } = await getFilePathsForDrag();

    console.log('[DEBUG]   getFilePathsForDrag returned:');
    console.log('[DEBUG]     items:', items);
    console.log('[DEBUG]     icon:', icon);

    if (items.length === 0) {
      console.log('[DEBUG]   No items to drag, aborting');
      return;
    }

    isDragging.value = true;
    pinboardStore.setDragging(true, props.item.id);

    try {
      console.log('[DEBUG]   CALLING startDrag() with:');
      console.log('[DEBUG]     item:', items);
      console.log('[DEBUG]     icon:', icon);
      console.log('▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶▶');

      await startDrag({
        item: items,
        icon: icon,
      });

      console.log('[DEBUG]   startDrag() completed successfully');
    } catch (err) {
      console.debug('[DEBUG]   startDrag() error:', err);
    } finally {
      isDragging.value = false;
      pinboardStore.setDragging(false, null);
      cleanup();
    }
  }
};

const handleNativeDragEnd = () => {
  cleanup();
};

const cleanup = () => {
  dragStartPos = null;
  dragStarted = false;
  document.removeEventListener('mousemove', handleNativeDragMove);
  document.removeEventListener('mouseup', handleNativeDragEnd);
};

// Native HTML5 drag handlers (for text, links - internal drag)
const handleDragStart = (e: DragEvent) => {
  // Skip HTML5 drag for file types - they use native drag
  if (canDragAsFiles.value) {
    e.preventDefault();
    return;
  }

  if (!e.dataTransfer || !cardRef.value) return;

  isDragging.value = true;
  pinboardStore.setDragging(true, props.item.id);

  // Set drag data with multiple formats for compatibility
  e.dataTransfer.setData('text/plain', props.item.content_text || '');
  e.dataTransfer.setData('application/x-clipboard-item', props.item.id);
  e.dataTransfer.effectAllowed = 'move';

  // Create exact clone with inline styles to avoid WebView ghost bugs
  dragClone = createExactClone();
  if (dragClone) {
    document.body.appendChild(dragClone);

    const rect = cardRef.value.getBoundingClientRect();
    const offsetX = e.clientX - rect.left;
    const offsetY = e.clientY - rect.top;
    e.dataTransfer.setDragImage(dragClone, offsetX, offsetY);

    // Clean up clone after a short delay (drag image is captured synchronously)
    setTimeout(() => {
      if (dragClone) {
        dragClone.remove();
        dragClone = null;
      }
    }, 100);
  }
};

const handleDragEnd = () => {
  isDragging.value = false;
  pinboardStore.setDragging(false, null);
};
</script>

<template>
  <!-- Visual Preview Card (Image/File with thumbnail) -->
  <div
    v-if="hasVisualPreview"
    ref="cardRef"
    class="clipboard-card visual-card"
    :class="{
      selected: selected,
      dragging: isDragging,
    }"
    :draggable="!canDragAsFiles"
    @click="handleClick"
    @dblclick="handleDoubleClick"
    @mousedown="handleNativeDragStart"
    @dragstart="handleDragStart"
    @dragend="handleDragEnd"
  >
    <!-- Card Header (same as standard cards) -->
    <div class="card-header" :class="`header-${item.content_type}`">
      <div class="header-left">
        <span class="type-badge" :class="`badge-${item.content_type}`">
          <svg v-if="item.content_type === 'image'" class="badge-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <circle cx="8.5" cy="8.5" r="1.5" />
            <polyline points="21 15 16 10 5 21" />
          </svg>
          <svg v-else-if="item.content_type === 'files'" class="badge-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
        </span>
        <span class="header-label">{{ headerLabel }}</span>
      </div>
      <div class="header-right">
        <span class="timestamp">{{ formattedTime }}</span>
        <img
          v-if="item.source_app_icon"
          :src="`data:image/png;base64,${item.source_app_icon}`"
          :alt="item.source_app || 'Source'"
          :title="item.source_app || 'Source app'"
          class="source-icon"
          draggable="false"
        />
        <span v-else class="source-icon-placeholder" :title="item.source_app || 'Unknown'">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <line x1="3" y1="9" x2="21" y2="9" />
            <line x1="9" y1="21" x2="9" y2="9" />
          </svg>
        </span>
      </div>
    </div>

    <!-- Preview image container -->
    <div class="visual-content">
      <img
        :src="`data:image/png;base64,${item.thumbnail_base64}`"
        :alt="item.content_type === 'image' ? 'Image' : 'File preview'"
        class="visual-preview"
        loading="lazy"
        draggable="false"
      />
      <!-- File count badge (for multiple files) -->
      <div v-if="item.content_type === 'files' && fileInfo.count > 1" class="visual-badge">
        +{{ fileInfo.count - 1 }}
      </div>
      <!-- Delete button overlay -->
      <button class="visual-delete" @click="handleDelete" title="Delete">×</button>
    </div>
  </div>

  <!-- Standard Card (Text, Link, Audio, Files without preview) -->
  <div
    v-else
    ref="cardRef"
    class="clipboard-card"
    :class="{
      selected: selected,
      dragging: isDragging,
    }"
    :draggable="!canDragAsFiles"
    @click="handleClick"
    @dblclick="handleDoubleClick"
    @mousedown="handleNativeDragStart"
    @dragstart="handleDragStart"
    @dragend="handleDragEnd"
  >
    <!-- Card Header -->
    <div class="card-header" :class="`header-${item.content_type}`">
      <div class="header-left">
        <span class="type-badge" :class="`badge-${item.content_type}`">
          <svg v-if="item.content_type === 'text'" class="badge-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="8" y2="13" />
            <line x1="16" y1="17" x2="8" y2="17" />
          </svg>
          <svg v-else-if="item.content_type === 'image'" class="badge-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <circle cx="8.5" cy="8.5" r="1.5" />
            <polyline points="21 15 16 10 5 21" />
          </svg>
          <svg v-else-if="item.content_type === 'files'" class="badge-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
          <svg v-else-if="item.content_type === 'link'" class="badge-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
            <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
          </svg>
          <svg v-else-if="item.content_type === 'audio'" class="badge-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18V5l12-2v13" />
            <circle cx="6" cy="18" r="3" />
            <circle cx="18" cy="16" r="3" />
          </svg>
          <svg v-else class="badge-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" />
            <line x1="12" y1="17" x2="12.01" y2="17" />
          </svg>
        </span>
        <span class="header-label">{{ headerLabel }}</span>
      </div>
      <div class="header-right">
        <span class="timestamp">{{ formattedTime }}</span>
        <img
          v-if="item.source_app_icon"
          :src="`data:image/png;base64,${item.source_app_icon}`"
          :alt="item.source_app || 'Source'"
          :title="item.source_app || 'Source app'"
          class="source-icon"
          draggable="false"
        />
        <span v-else class="source-icon-placeholder" :title="item.source_app || 'Unknown'">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <line x1="3" y1="9" x2="21" y2="9" />
            <line x1="9" y1="21" x2="9" y2="9" />
          </svg>
        </span>
      </div>
    </div>

    <!-- Card Content -->
    <div class="card-content">
      <!-- Text Content -->
      <div v-if="item.content_type === 'text'" class="text-content">
        <p class="text-preview">{{ textPreview }}</p>
      </div>

      <!-- Image without preview -->
      <div v-else-if="item.content_type === 'image'" class="image-content">
        <div class="thumbnail-placeholder">
          <span>No preview</span>
        </div>
      </div>

      <!-- Files without preview -->
      <div v-else-if="item.content_type === 'files'" class="files-content">
        <div class="file-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
        </div>
        <p class="file-count">{{ fileInfo.count }} file{{ fileInfo.count !== 1 ? 's' : '' }}</p>
        <p v-if="fileInfo.names.length > 0" class="file-name">{{ fileInfo.names[0] }}</p>
      </div>

      <!-- Link Content -->
      <div v-else-if="item.content_type === 'link'" class="link-content">
        <div class="link-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <line x1="2" y1="12" x2="22" y2="12" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
          </svg>
        </div>
        <p class="link-domain">{{ urlPreview }}</p>
        <p class="link-url">{{ item.content_text }}</p>
      </div>

      <!-- Audio Content -->
      <div v-else-if="item.content_type === 'audio'" class="audio-content">
        <div class="audio-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18V5l12-2v13" />
            <circle cx="6" cy="18" r="3" />
            <circle cx="18" cy="16" r="3" />
          </svg>
        </div>
        <p class="audio-count">{{ audioInfo.count }} audio{{ audioInfo.count !== 1 ? ' files' : '' }}</p>
        <p v-if="audioInfo.names.length > 0" class="audio-name">{{ audioInfo.names[0] }}</p>
      </div>

      <!-- Delete button overlay (same position as visual cards) -->
      <button class="visual-delete" @click="handleDelete" title="Delete">×</button>
    </div>
  </div>
</template>

<style scoped>
.clipboard-card {
  flex-shrink: 0;
  height: 100%;
  aspect-ratio: 1 / 1;
  min-width: 0;
  background: white;
  border-radius: 10px;
  border: 2px solid transparent;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  display: flex;
  flex-direction: column;
  cursor: grab;
  transition: transform 0.15s, box-shadow 0.15s, border-color 0.15s;
  overflow: hidden;
  position: relative;
  /* Prevent text selection */
  user-select: none;
  -webkit-user-select: none;
  -webkit-touch-callout: none;
}

.clipboard-card * {
  user-select: none;
  -webkit-user-select: none;
  -webkit-touch-callout: none;
  /* Note: Images use draggable="false" attribute for cross-platform support */
  /* -webkit-user-drag is WebKit-only and doesn't work on Windows/Chromium */
}

.clipboard-card:active {
  cursor: grabbing;
}

.clipboard-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

/* ============================================
   Visual Preview Card (image/file with thumbnail)
   ============================================ */
.visual-card {
  /* Uses same structure as standard card */
}

.visual-content {
  flex: 1;
  position: relative;
  overflow: hidden;
  min-height: 0;
}

.visual-preview {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.visual-badge {
  position: absolute;
  bottom: 8px;
  right: 8px;
  padding: 4px 10px;
  background: rgba(0, 0, 0, 0.75);
  color: white;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 600;
  backdrop-filter: blur(4px);
}

/* Delete button - same style for all cards */
.visual-delete {
  position: absolute;
  bottom: 8px;
  right: 8px;
  width: 26px;
  height: 26px;
  padding: 0;
  border: none;
  border-radius: 8px;
  background: rgba(239, 68, 68, 0.85);
  color: white;
  cursor: pointer;
  font-size: 16px;
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.15s, background-color 0.15s, transform 0.15s;
  backdrop-filter: blur(4px);
  box-shadow: 0 2px 8px rgba(239, 68, 68, 0.3);
}

/* Show delete button on hover for ALL card types */
.clipboard-card:hover .visual-delete,
.visual-card:hover .visual-delete {
  opacity: 1;
}

.visual-delete:hover {
  background: rgba(220, 38, 38, 1);
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.4);
}

.clipboard-card.selected {
  border-color: #3b82f6;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
}

.clipboard-card.dragging {
  opacity: 0.5;
  transform: scale(0.95);
  cursor: grabbing;
}


/* Card Header */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 10px;
  flex-shrink: 0;
  border-radius: 10px 10px 0 0;
  overflow: hidden;
  max-width: 100%;
}

/* Header color themes */
.header-text {
  background: linear-gradient(135deg, #eff6ff 0%, #dbeafe 100%);
}

.header-image {
  background: linear-gradient(135deg, #faf5ff 0%, #f3e8ff 100%);
}

.header-files {
  background: linear-gradient(135deg, #f0fdf4 0%, #dcfce7 100%);
}

.header-link {
  background: linear-gradient(135deg, #fff7ed 0%, #ffedd5 100%);
}

.header-audio {
  background: linear-gradient(135deg, #fdf2f8 0%, #fce7f3 100%);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  flex: 1;
  overflow: hidden;
}

.type-badge {
  width: 22px;
  height: 22px;
  border-radius: 5px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.badge-icon {
  width: 12px;
  height: 12px;
}

.badge-text {
  background: #3b82f6;
  color: white;
}

.badge-image {
  background: #8b5cf6;
  color: white;
}

.badge-files {
  background: #22c55e;
  color: white;
}

.badge-link {
  background: #f97316;
  color: white;
}

.badge-audio {
  background: #ec4899;
  color: white;
}

.header-label {
  font-size: 11px;
  font-weight: 600;
  color: #374151;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  overflow: hidden;
}

.timestamp {
  font-size: 10px;
  color: #9ca3af;
}

.source-icon {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  object-fit: contain;
  background: white;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.source-icon-placeholder {
  width: 18px;
  height: 18px;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.05);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #9ca3af;
}

.source-icon-placeholder svg {
  width: 10px;
  height: 14px;
}

/* Card Content */
.card-content {
  flex: 1;
  padding: 6px 10px;
  overflow: hidden;
  min-height: 0;
  position: relative; /* For delete button positioning */
}

/* Text Content */
.text-content {
  height: 100%;
  overflow: hidden;
}

.text-preview {
  margin: 0;
  font-size: 11px;
  line-height: 1.4;
  color: #374151;
  word-break: break-word;
  display: -webkit-box;
  -webkit-line-clamp: 6;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* Image Content */
.image-content {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.thumbnail {
  max-width: 100%;
  max-height: 120px;
  border-radius: 6px;
  object-fit: contain;
}

.thumbnail-placeholder {
  width: 80px;
  height: 60px;
  background: #f3f4f6;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  color: #9ca3af;
}

/* Files Content */
.files-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  position: relative;
}

.file-icon {
  width: 32px;
  height: 32px;
  color: #6b7280;
}

.file-icon svg {
  width: 100%;
  height: 100%;
}

.file-count {
  margin: 0;
  font-size: 12px;
  font-weight: 500;
  color: #374151;
}

.file-name {
  margin: 0;
  font-size: 10px;
  color: #6b7280;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Link Content */
.link-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.link-icon {
  width: 28px;
  height: 28px;
  color: #ea580c;
}

.link-icon svg {
  width: 100%;
  height: 100%;
}

.link-domain {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  color: #ea580c;
}

.link-url {
  margin: 0;
  font-size: 10px;
  color: #9ca3af;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  padding: 0 8px;
}

/* Audio Content */
.audio-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.audio-icon {
  width: 32px;
  height: 32px;
  color: #db2777;
}

.audio-icon svg {
  width: 100%;
  height: 100%;
}

.audio-count {
  margin: 0;
  font-size: 12px;
  font-weight: 500;
  color: #374151;
}

.audio-name {
  margin: 0;
  font-size: 10px;
  color: #6b7280;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
  .clipboard-card {
    background: #1f2937;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .clipboard-card.selected {
    border-color: #60a5fa;
  }

  /* Dark mode header colors */
  .header-text {
    background: linear-gradient(135deg, #1e3a5f 0%, #1e40af20 100%);
  }

  .header-image {
    background: linear-gradient(135deg, #3b1f5f 0%, #6d28d920 100%);
  }

  .header-files {
    background: linear-gradient(135deg, #14532d 0%, #16a34a20 100%);
  }

  .header-link {
    background: linear-gradient(135deg, #7c2d12 0%, #ea580c20 100%);
  }

  .header-audio {
    background: linear-gradient(135deg, #831843 0%, #db277720 100%);
  }

  .header-label {
    color: #e5e7eb;
  }

  .source-icon {
    background: #374151;
  }

  .source-icon-placeholder {
    background: rgba(255, 255, 255, 0.1);
    color: #6b7280;
  }

  .text-preview {
    color: #e5e7eb;
  }

  .thumbnail-placeholder {
    background: #374151;
  }

  .file-count,
  .audio-count {
    color: #e5e7eb;
  }

  .file-name,
  .audio-name {
    color: #9ca3af;
  }

  .link-domain {
    color: #fb923c;
  }

  .link-url {
    color: #6b7280;
  }
}
</style>
