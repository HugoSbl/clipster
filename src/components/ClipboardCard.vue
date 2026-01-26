<script setup lang="ts">
import { computed, ref } from 'vue';
import type { ClipboardItem } from '@/types';
import { usePinboardStore } from '@/stores/pinboards';
import { invoke } from '@tauri-apps/api/core';
import { startDrag } from '@crabnebula/tauri-plugin-drag';

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

  // For documents, show the filename
  if (type === 'documents' && props.item.content_text) {
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
    documents: 'Document',
    video: 'Video',
  };

  return typeLabels[type] || 'Unknown';
});

// Get URL preview (domain only)
const urlPreview = computed(() => {
  if (props.item.content_type !== 'link' || !props.item.content_text) return '';
  const url = props.item.content_text;
  try {
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

// Get file info (for files type)
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

// Get document info (for documents type - PDF, Word, Excel, etc.)
const documentInfo = computed(() => {
  if (props.item.content_type !== 'documents' || !props.item.content_text) {
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

// Get thumbnail data URL with correct MIME type (auto-detect PNG vs JPEG)
const thumbnailDataUrl = computed(() => {
  if (!props.item.thumbnail_base64) return '';
  const base64 = props.item.thumbnail_base64;
  if (base64.startsWith('/9j/')) {
    return `data:image/jpeg;base64,${base64}`;
  } else if (base64.startsWith('R0lGOD')) {
    return `data:image/gif;base64,${base64}`;
  }
  return `data:image/png;base64,${base64}`;
});

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

// Sanitize a string to be safe for filenames
const sanitizeFilename = (name: string): string => {
  return name
    .replace(/[<>:"/\\|?*]/g, '')
    .replace(/\s+/g, '_')
    .substring(0, 50);
};

/**
 * Sanitize a file path before sending to Rust
 * 1. Remove file:// prefix if present
 * 2. Decode URI-encoded characters (%20 -> space, etc.)
 * 3. Returns a clean OS path (e.g., /Users/name/file.png)
 */
const sanitizePath = (path: string): string => {
  let cleanPath = path;

  // Remove file:// or file:/// prefix
  if (cleanPath.startsWith('file:///')) {
    cleanPath = cleanPath.substring(7); // Remove 'file://' (keep leading /)
  } else if (cleanPath.startsWith('file://')) {
    cleanPath = cleanPath.substring(7);
  }

  // Decode URI-encoded characters (%20 -> space, etc.)
  try {
    cleanPath = decodeURIComponent(cleanPath);
  } catch {
    // If decoding fails, use the path as-is
    console.warn('[sanitizePath] Failed to decode:', path);
  }

  // On Windows, handle drive letter paths (e.g., /C:/Users -> C:/Users)
  if (/^\/[A-Za-z]:/.test(cleanPath)) {
    cleanPath = cleanPath.substring(1);
  }

  return cleanPath;
};

// Generate a readable filename from item metadata
const generateReadableFilename = (item: ClipboardItem): string => {
  const sourceApp = item.source_app ? sanitizeFilename(item.source_app) : 'Clipster';
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
const prepareImageForDrag = async (
  item: ClipboardItem
): Promise<{ imagePath: string; iconPath: string } | null> => {
  if (!item.image_path) return null;

  try {
    const readableFilename = generateReadableFilename(item);
    const [imagePath, iconPath] = await invoke<[string, string]>('prepare_image_for_drag', {
      sourcePath: item.image_path,
      readableFilename,
    });
    return { imagePath, iconPath };
  } catch (err) {
    console.error('[ClipboardCard] prepareImageForDrag error:', err);
    return { imagePath: item.image_path, iconPath: item.image_path };
  }
};

// Get file paths for native drag (files, audio, documents - NOT images)
const getFilePaths = (): string[] => {
  const item = props.item;

  if (
    (item.content_type === 'files' ||
      item.content_type === 'audio' ||
      item.content_type === 'documents') &&
    item.content_text
  ) {
    try {
      const files = JSON.parse(item.content_text) as string[];
      return files.filter((f) => f && typeof f === 'string');
    } catch (err) {
      console.warn('[ClipboardCard] getFilePaths: Invalid JSON', err);
      return [];
    }
  }

  return [];
};

// Get file paths for drag, with async handling for images/text/links
const getFilePathsForDrag = async (): Promise<{ items: string[]; icon: string }> => {
  const item = props.item;

  // For images, prepare with readable filename and separate icon
  if (item.content_type === 'image' && item.image_path) {
    const prepared = await prepareImageForDrag(item);
    if (prepared) {
      return { items: [prepared.imagePath], icon: prepared.iconPath };
    }
    return { items: [], icon: '' };
  }

  // For text, create a temporary text file
  if (item.content_type === 'text' && item.content_text) {
    try {
      const readableFilename = generateReadableFilename(item).replace(/\.(png|jpg|jpeg)$/, '.txt');
      const textPath = await invoke<string>('create_temp_text_file', {
        content: item.content_text,
        filename: readableFilename,
      });
      return { items: [textPath], icon: textPath };
    } catch (err) {
      console.error('[ClipboardCard] create_temp_text_file failed:', err);
      return { items: [], icon: '' };
    }
  }

  // For links, create a platform-specific link file
  if (item.content_type === 'link' && item.content_text) {
    try {
      const sourceApp = item.source_app ? sanitizeFilename(item.source_app) : 'Link';
      const date = new Date(item.created_at);
      const timestamp = [
        date.getFullYear(),
        String(date.getMonth() + 1).padStart(2, '0'),
        String(date.getDate()).padStart(2, '0'),
        '_',
        String(date.getHours()).padStart(2, '0'),
        String(date.getMinutes()).padStart(2, '0'),
      ].join('');
      const filename = `${sourceApp}_${timestamp}`;

      const linkPath = await invoke<string>('create_temp_link_file', {
        url: item.content_text,
        filename,
      });
      return { items: [linkPath], icon: linkPath };
    } catch (err) {
      console.error('[ClipboardCard] create_temp_link_file failed:', err);
      return { items: [], icon: '' };
    }
  }

  // For files, audio, documents - parse JSON paths directly
  const paths = getFilePaths();
  return { items: paths, icon: paths[0] || '' };
};

// Check if item can be dragged as native files (NOT text - text uses HTML5)
const canDragAsNativeFiles = computed(() => {
  const item = props.item;

  // Images drag as files
  if (item.content_type === 'image' && item.image_path) {
    return true;
  }

  // Links drag as .webloc files
  if (item.content_type === 'link' && item.content_text) {
    return true;
  }

  // Files, audio, documents drag as files
  const paths = getFilePaths();
  return paths.length > 0;
});

// Check if item is text (uses HTML5 drag for direct paste)
const isTextItem = computed(() => {
  return props.item.content_type === 'text' && props.item.content_text;
});

// ============================================================================
// DRAG & DROP IMPLEMENTATION
// ============================================================================
//
// TWO MODES:
// 1. TEXT ITEMS: HTML5 drag with text/plain (pastes directly into text fields)
// 2. FILE ITEMS: Custom mouse + native plugin (drops real files to Finder)
//
// ============================================================================

/**
 * HTML5 Drag for TEXT items - allows direct paste into text fields
 */
const handleTextDragStart = (e: DragEvent) => {
  // Only handle if this is a text item
  if (!isTextItem.value || !e.dataTransfer || !props.item.content_text) return;

  e.dataTransfer.effectAllowed = 'copyMove';
  e.dataTransfer.setData('text/plain', props.item.content_text);
  e.dataTransfer.setData('application/x-clipboard-item', props.item.id);

  isDragging.value = true;
  pinboardStore.setDragging(true, props.item.id);

  // Add listener to hide window when drag leaves the app
  document.addEventListener('dragleave', handleTextDragLeave);
  document.addEventListener('dragover', handleTextDragOver);
};

// Track if we're still inside the window during text drag
let textDragInsideWindow = true;

const handleTextDragOver = () => {
  // We're inside the window
  textDragInsideWindow = true;
};

const handleTextDragLeave = async (e: DragEvent) => {
  // Only hide when truly leaving the window, not when moving between elements
  // relatedTarget is null when leaving to outside the browser/app
  const relatedTarget = e.relatedTarget as Node | null;

  // If relatedTarget exists and is inside the document, we're just moving between elements
  if (relatedTarget && document.contains(relatedTarget)) {
    return;
  }

  // Double-check with coordinates - must be outside window bounds
  const x = e.clientX;
  const y = e.clientY;
  const isOutside = x <= 0 || x >= window.innerWidth || y <= 0 || y >= window.innerHeight;

  if (!isOutside) {
    return;
  }

  textDragInsideWindow = false;

  // Small delay to confirm we're really outside
  setTimeout(async () => {
    if (!textDragInsideWindow && isDragging.value) {
      console.log('[handleTextDragLeave] Left window, hiding app');
      await invoke('hide_window');
    }
  }, 50);
};

const handleTextDragEnd = () => {
  // Only handle if this is a text item
  if (!isTextItem.value) return;

  // Remove listeners
  document.removeEventListener('dragleave', handleTextDragLeave);
  document.removeEventListener('dragover', handleTextDragOver);

  isDragging.value = false;
  pinboardStore.setDragging(false, null);
  textDragInsideWindow = true;
};

const dragGhost = ref<HTMLElement | null>(null);
const dragStartPos = ref<{ x: number; y: number } | null>(null);
const pendingDragPaths = ref<string[]>([]);
const pendingDragIcon = ref<string>('');
const hasTriggeredNative = ref(false);
const lastClickTime = ref(0);

const DRAG_THRESHOLD = 5;
const EDGE_MARGIN = 50;
const DOUBLE_CLICK_DELAY = 300;

/**
 * Check if near window edge (for triggering native drag)
 * EXCLUDES top edge - pins are at the top, we want to allow drops there
 */
const isNearEdge = (x: number, y: number): boolean => {
  const w = window.innerWidth;
  const h = window.innerHeight;
  // Left, right, or bottom edge - NOT top (where pins are)
  return x <= EDGE_MARGIN || x >= w - EDGE_MARGIN || y >= h - EDGE_MARGIN;
};

/**
 * Create ghost element (clone of card)
 */
const createGhost = () => {
  if (!cardRef.value || dragGhost.value) return;

  const ghost = cardRef.value.cloneNode(true) as HTMLElement;
  ghost.style.position = 'fixed';
  ghost.style.pointerEvents = 'none';
  ghost.style.zIndex = '9999';
  ghost.style.opacity = '0.8';
  ghost.style.transform = 'scale(0.8)';
  ghost.style.transition = 'none';
  ghost.style.width = `${cardRef.value.offsetWidth}px`;
  ghost.style.height = `${cardRef.value.offsetHeight}px`;
  document.body.appendChild(ghost);
  dragGhost.value = ghost;
};

/**
 * Update ghost position
 */
const updateGhostPosition = (x: number, y: number) => {
  if (!dragGhost.value) return;
  dragGhost.value.style.left = `${x - dragGhost.value.offsetWidth / 2}px`;
  dragGhost.value.style.top = `${y - dragGhost.value.offsetHeight / 2}px`;
};

/**
 * Remove ghost element
 */
const removeGhost = () => {
  if (dragGhost.value) {
    dragGhost.value.remove();
    dragGhost.value = null;
  }
};

/**
 * Mousedown - start tracking (for native file drag only, NOT text)
 */
const handleMouseDown = async (e: MouseEvent) => {
  if (!canDragAsNativeFiles.value || e.button !== 0) return;

  e.preventDefault();
  e.stopPropagation();

  dragStartPos.value = { x: e.clientX, y: e.clientY };
  hasTriggeredNative.value = false;

  // Prepare paths
  try {
    const { items } = await getFilePathsForDrag();
    pendingDragPaths.value = items.map(sanitizePath);

    invoke<string>('create_drag_icon', { path: pendingDragPaths.value[0] })
      .then((icon) => { pendingDragIcon.value = icon; })
      .catch(() => { pendingDragIcon.value = pendingDragPaths.value[0]; });
  } catch (err) {
    console.error('[handleMouseDown] Error:', err);
  }

  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
};

/**
 * Mousemove - update ghost + detect edge
 */
const handleMouseMove = async (e: MouseEvent) => {
  if (!dragStartPos.value || hasTriggeredNative.value) return;

  const dx = Math.abs(e.clientX - dragStartPos.value.x);
  const dy = Math.abs(e.clientY - dragStartPos.value.y);

  // Start dragging after threshold
  if (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD) {
    if (!isDragging.value) {
      isDragging.value = true;
      pinboardStore.setDragging(true, props.item.id);
      createGhost();
    }

    updateGhostPosition(e.clientX, e.clientY);

    // Find drop zone under cursor for visual highlighting
    // Hide ghost temporarily to get element underneath
    if (dragGhost.value) {
      dragGhost.value.style.display = 'none';
    }
    const elementUnder = document.elementFromPoint(e.clientX, e.clientY);
    const dropZone = elementUnder?.closest('[data-drop-zone]');
    if (dragGhost.value) {
      dragGhost.value.style.display = '';
    }

    // Update store with hovered zone (for PinboardTabs highlighting)
    if (dropZone) {
      const zoneId = dropZone.getAttribute('data-drop-zone');
      pinboardStore.$patch({ hoveredDropZone: zoneId });
    } else {
      pinboardStore.$patch({ hoveredDropZone: null });
    }

    // Check if near edge - trigger native drag
    if (isNearEdge(e.clientX, e.clientY) && pendingDragPaths.value.length > 0) {
      console.log('[handleMouseMove] Near edge - starting native drag');
      hasTriggeredNative.value = true;

      // Cleanup before async
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      removeGhost();

      try {
        await invoke('hide_window');
        await new Promise((r) => setTimeout(r, 50));

        await startDrag({
          item: pendingDragPaths.value,
          icon: pendingDragIcon.value || pendingDragPaths.value[0],
        });
      } catch (err) {
        console.error('[handleMouseMove] Native drag error:', err);
        await invoke('show_window');
      }

      cleanupDrag();
    }
  }
};

/**
 * Mouseup - internal drop, click, or cancel
 */
const handleMouseUp = (e: MouseEvent) => {
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);

  // If we weren't dragging (mouse didn't move past threshold), treat as click
  if (!isDragging.value && dragStartPos.value) {
    const dx = Math.abs(e.clientX - dragStartPos.value.x);
    const dy = Math.abs(e.clientY - dragStartPos.value.y);

    if (dx <= DRAG_THRESHOLD && dy <= DRAG_THRESHOLD) {
      // This was a click, not a drag
      const now = Date.now();
      const timeSinceLastClick = now - lastClickTime.value;

      if (timeSinceLastClick < DOUBLE_CLICK_DELAY) {
        // Double-click - copy the item
        cleanupDrag();
        emit('copy', props.item);
        lastClickTime.value = 0; // Reset to prevent triple-click issues
        return;
      } else {
        // Single click - select the item
        lastClickTime.value = now;
        cleanupDrag();
        emit('select', props.item);
        return;
      }
    }
  }

  if (isDragging.value && !hasTriggeredNative.value) {
    // IMPORTANT: Hide ghost BEFORE elementFromPoint so it doesn't block detection
    const ghostWasVisible = !!dragGhost.value;
    if (dragGhost.value) {
      dragGhost.value.style.display = 'none';
    }

    // Check for internal drop target
    const dropTarget = document.elementFromPoint(e.clientX, e.clientY);
    const dropZone = dropTarget?.closest('[data-drop-zone]');

    if (dropZone) {
      const zoneId = dropZone.getAttribute('data-drop-zone');
      console.log('[handleMouseUp] Drop on zone:', zoneId);

      dropZone.dispatchEvent(new CustomEvent('clipster-internal-drop', {
        detail: { itemId: props.item.id, zoneId },
        bubbles: true,
      }));
    } else {
      console.log('[handleMouseUp] No drop zone found at', e.clientX, e.clientY, 'target:', dropTarget);
    }

    // Restore ghost visibility (will be removed in cleanup anyway)
    if (ghostWasVisible && dragGhost.value) {
      dragGhost.value.style.display = '';
    }
  }

  cleanupDrag();
};

/**
 * Cleanup all drag state
 */
const cleanupDrag = () => {
  removeGhost();
  isDragging.value = false;
  pinboardStore.setDragging(false, null);
  dragStartPos.value = null;
  pendingDragPaths.value = [];
  pendingDragIcon.value = '';
  hasTriggeredNative.value = false;
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
    @click="handleClick"
    @dblclick="handleDoubleClick"
  >
    <!-- TRANSPARENT SHIELD: Native drag for files (NOT text) -->
    <div
      v-if="canDragAsNativeFiles"
      class="drag-shield"
      @mousedown="handleMouseDown"
    ></div>
    <!-- Card Header -->
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
        :src="thumbnailDataUrl"
        :alt="item.content_type === 'image' ? 'Image' : 'File preview'"
        class="visual-preview"
        loading="lazy"
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
    :draggable="isTextItem ? true : false"
    @click="handleClick"
    @dblclick="handleDoubleClick"
    @dragstart="handleTextDragStart"
    @dragend="handleTextDragEnd"
  >
    <!-- TRANSPARENT SHIELD: Native drag for files (NOT text - text uses HTML5) -->
    <div
      v-if="canDragAsNativeFiles"
      class="drag-shield"
      @mousedown="handleMouseDown"
    ></div>
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
          <svg v-else-if="item.content_type === 'documents'" class="badge-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="8" y2="13" />
            <line x1="16" y1="17" x2="8" y2="17" />
            <line x1="10" y1="9" x2="8" y2="9" />
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

      <!-- Documents Content (PDF, Word, Excel, etc.) -->
      <div v-else-if="item.content_type === 'documents'" class="documents-content">
        <div class="documents-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="8" y2="13" />
            <line x1="16" y1="17" x2="8" y2="17" />
            <line x1="10" y1="9" x2="8" y2="9" />
          </svg>
        </div>
        <p class="documents-count">{{ documentInfo.count }} document{{ documentInfo.count !== 1 ? 's' : '' }}</p>
        <p v-if="documentInfo.names.length > 0" class="documents-name">{{ documentInfo.names[0] }}</p>
      </div>

      <!-- Delete button overlay -->
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
  /* Prevent text selection and image dragging */
  user-select: none;
  -webkit-user-select: none;
  -webkit-touch-callout: none;
}

/**
 * TRANSPARENT SHIELD PATTERN
 * This invisible overlay captures all drag events before they reach
 * internal text/image elements, preventing unwanted selection.
 * The shield sits on top of content but below interactive elements (delete button).
 */
.drag-shield {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1; /* Above content, below delete button (z-index: 2) */
  cursor: grab;
  /* Completely transparent - no visual change */
  background: transparent;
  /* Shield must NOT block click events for underlying card */
  /* But MUST capture drag events - this is the key trick */
}

.drag-shield:active {
  cursor: grabbing;
}

/* Prevent text selection on all child elements */
.clipboard-card * {
  user-select: none;
  -webkit-user-select: none;
  -webkit-touch-callout: none;
}

.clipboard-card:active {
  cursor: grabbing;
}

.clipboard-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
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

/* Visual Preview Card */
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
/* z-index: 2 to appear ABOVE the drag-shield (z-index: 1) */
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
  z-index: 2; /* Above drag-shield */
}

.clipboard-card:hover .visual-delete,
.visual-card:hover .visual-delete {
  opacity: 1;
}

.visual-delete:hover {
  background: rgba(220, 38, 38, 1);
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.4);
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

.header-documents {
  background: linear-gradient(135deg, #ecfeff 0%, #cffafe 100%);
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

.badge-documents {
  background: #0891b2;
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
  position: relative;
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

/* Documents Content */
.documents-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
}

.documents-icon {
  width: 32px;
  height: 32px;
  color: #0891b2;
}

.documents-icon svg {
  width: 100%;
  height: 100%;
}

.documents-count {
  margin: 0;
  font-size: 12px;
  font-weight: 500;
  color: #374151;
}

.documents-name {
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

  .header-documents {
    background: linear-gradient(135deg, #164e63 0%, #0891b220 100%);
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
  .audio-count,
  .documents-count {
    color: #e5e7eb;
  }

  .file-name,
  .audio-name,
  .documents-name {
    color: #9ca3af;
  }

  .documents-icon {
    color: #22d3ee;
  }

  .link-domain {
    color: #fb923c;
  }

  .link-url {
    color: #6b7280;
  }
}
</style>
