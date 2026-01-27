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

// File extension for footer type badge
const fileExtension = computed(() => {
  const type = props.item.content_type;

  if (type === 'text') return 'TXT';
  if (type === 'link') return 'URL';

  if (type === 'image') {
    if (props.item.image_path) {
      const ext = props.item.image_path.split('.').pop()?.toUpperCase();
      if (ext) return ext;
    }
    return 'IMG';
  }

  if (type === 'files' || type === 'audio' || type === 'documents') {
    if (props.item.content_text) {
      try {
        const files = JSON.parse(props.item.content_text) as string[];
        if (files.length > 0) {
          const ext = files[0].split('.').pop()?.toUpperCase();
          if (ext) return ext;
        }
      } catch {
        // Fall through to defaults
      }
    }
    if (type === 'files') return 'FILE';
    if (type === 'audio') return 'MP3';
    return 'PDF';
  }

  return 'FILE';
});

// Raw RGB string per type for CSS custom properties
const typeColorRgb = computed(() => {
  const colors: Record<string, string> = {
    text: '59,130,246',
    image: '139,92,246',
    files: '34,197,94',
    link: '249,115,22',
    audio: '236,72,153',
    documents: '8,145,178',
  };
  return colors[props.item.content_type] || '107,114,128';
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

  // Files, audio, documents drag as files
  const paths = getFilePaths();
  return paths.length > 0;
});

// Check if item uses HTML5 drag for direct paste (text and links)
const isTextItem = computed(() => {
  return (props.item.content_type === 'text' || props.item.content_type === 'link') && props.item.content_text;
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
 * Check if cursor should trigger native drag (near edge or outside window).
 * EXCLUDES top edge - pins are at the top, we want to allow drops there.
 */
const shouldTriggerNativeDrag = (x: number, y: number): boolean => {
  const w = window.innerWidth;
  const h = window.innerHeight;
  // Outside window bounds (cursor already left the webview)
  if (x <= 0 || x >= w || y <= 0 || y >= h) return true;
  // Near left, right, or bottom edge - NOT top (where pins are)
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
 *
 * IMPORTANT: Mouse listeners are added SYNCHRONOUSLY before any async work.
 * Path preparation runs in the background. This prevents the race condition
 * where the user moves the mouse during async file preparation and the
 * mousemove events are missed.
 */
const handleMouseDown = (e: MouseEvent) => {
  if (!canDragAsNativeFiles.value || e.button !== 0) return;

  e.preventDefault();
  e.stopPropagation();

  dragStartPos.value = { x: e.clientX, y: e.clientY };
  hasTriggeredNative.value = false;

  // Add listeners IMMEDIATELY so we never miss mouse events
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);

  // Prepare paths in background (non-blocking)
  getFilePathsForDrag()
    .then(({ items }) => {
      pendingDragPaths.value = items.map(sanitizePath);
      if (pendingDragPaths.value[0]) {
        invoke<string>('create_drag_icon', { path: pendingDragPaths.value[0] })
          .then((icon) => { pendingDragIcon.value = icon; })
          .catch(() => { pendingDragIcon.value = pendingDragPaths.value[0]; });
      }
    })
    .catch((err: unknown) => {
      console.error('[handleMouseDown] Path preparation error:', err);
    });
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

    // Trigger native drag when near edge or cursor left the window
    if (shouldTriggerNativeDrag(e.clientX, e.clientY) && pendingDragPaths.value.length > 0) {
      console.log('[handleMouseMove] Triggering native drag at', e.clientX, e.clientY);
      hasTriggeredNative.value = true;

      // Cleanup before async
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      removeGhost();

      try {
        // CRITICAL: startDrag() MUST be called while the window is still visible.
        // macOS needs a visible window to create the NSDraggingSession.
        // We start the drag first, then hide the window after a brief delay
        // to let the OS capture the drag session.
        const dragPromise = startDrag(
          {
            item: pendingDragPaths.value,
            icon: pendingDragIcon.value || pendingDragPaths.value[0],
          },
          (payload) => {
            console.log('[startDrag] Event:', payload.result);
          }
        );

        // Hide window after OS has captured the drag session
        setTimeout(() => {
          invoke('hide_window').catch((err: unknown) => {
            console.error('[handleMouseMove] hide_window error:', err);
          });
        }, 100);

        await dragPromise;
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
  <div
    ref="cardRef"
    class="clipboard-card"
    :class="{
      'visual-card': hasVisualPreview,
      selected: selected,
      dragging: isDragging,
    }"
    :style="{ '--type-rgb': typeColorRgb }"
    :draggable="isTextItem ? true : false"
    @click="handleClick"
    @dblclick="handleDoubleClick"
    @dragstart="handleTextDragStart"
    @dragend="handleTextDragEnd"
  >
    <!-- TRANSPARENT SHIELD: Native drag for files (NOT text) -->
    <div
      v-if="canDragAsNativeFiles"
      class="drag-shield"
      @mousedown="handleMouseDown"
    ></div>

    <!-- Header row: glass title pill + glass delete pill -->
    <div class="card-header">
      <span class="glass-pill header-label">{{ headerLabel }}</span>
      <button class="glass-pill delete-btn" @click="handleDelete" title="Delete">&times;</button>
    </div>

    <!-- Content: Visual (image) or Standard -->
    <div v-if="hasVisualPreview" class="visual-content">
      <img
        :src="thumbnailDataUrl"
        :alt="item.content_type === 'image' ? 'Image' : 'File preview'"
        class="visual-preview"
        loading="lazy"
      />
      <div v-if="item.content_type === 'files' && fileInfo.count > 1" class="glass-pill visual-badge">
        +{{ fileInfo.count - 1 }}
      </div>
    </div>

    <div v-else class="card-content">
      <!-- Text -->
      <div v-if="item.content_type === 'text'" class="text-content">
        <p class="text-preview">{{ textPreview }}</p>
      </div>

      <!-- Image without preview -->
      <div v-else-if="item.content_type === 'image'" class="placeholder-content">
        <span class="placeholder-text">No preview</span>
      </div>

      <!-- Files without preview -->
      <div v-else-if="item.content_type === 'files'" class="icon-content">
        <div class="content-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
        </div>
        <p class="content-label">{{ fileInfo.count }} file{{ fileInfo.count !== 1 ? 's' : '' }}</p>
        <p v-if="fileInfo.names.length > 0" class="content-sublabel">{{ fileInfo.names[0] }}</p>
      </div>

      <!-- Link -->
      <div v-else-if="item.content_type === 'link'" class="icon-content">
        <div class="content-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <line x1="2" y1="12" x2="22" y2="12" />
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
          </svg>
        </div>
        <p class="content-label">{{ urlPreview }}</p>
        <p class="content-sublabel">{{ item.content_text }}</p>
      </div>

      <!-- Audio -->
      <div v-else-if="item.content_type === 'audio'" class="icon-content">
        <div class="content-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 18V5l12-2v13" />
            <circle cx="6" cy="18" r="3" />
            <circle cx="18" cy="16" r="3" />
          </svg>
        </div>
        <p class="content-label">{{ audioInfo.count }} audio{{ audioInfo.count !== 1 ? ' files' : '' }}</p>
        <p v-if="audioInfo.names.length > 0" class="content-sublabel">{{ audioInfo.names[0] }}</p>
      </div>

      <!-- Documents -->
      <div v-else-if="item.content_type === 'documents'" class="icon-content">
        <div class="content-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="16" y1="13" x2="8" y2="13" />
            <line x1="16" y1="17" x2="8" y2="17" />
            <line x1="10" y1="9" x2="8" y2="9" />
          </svg>
        </div>
        <p class="content-label">{{ documentInfo.count }} document{{ documentInfo.count !== 1 ? 's' : '' }}</p>
        <p v-if="documentInfo.names.length > 0" class="content-sublabel">{{ documentInfo.names[0] }}</p>
      </div>
    </div>

    <!-- Footer: glass type pill (left) · glass meta pill (right) -->
    <div class="card-footer">
      <span class="glass-pill type-pill">
        <span class="type-dot"></span>
        {{ fileExtension }}
      </span>
      <span class="glass-pill footer-meta">
        <span class="footer-time">{{ formattedTime }}</span>
        <img
          v-if="item.source_app_icon"
          :src="`data:image/png;base64,${item.source_app_icon}`"
          :alt="item.source_app || 'Source'"
          :title="item.source_app || 'Source app'"
          class="footer-app-icon"
        />
        <span v-else class="footer-app-placeholder" :title="item.source_app || 'Unknown'">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <line x1="3" y1="9" x2="21" y2="9" />
            <line x1="9" y1="21" x2="9" y2="9" />
          </svg>
        </span>
      </span>
    </div>
  </div>
</template>

<style scoped>
/* ============================================================================
   GLASSMORPHISM CARD
   ============================================================================ */

.clipboard-card {
  --type-rgb: 107, 114, 128;

  flex-shrink: 0;
  height: 100%;
  aspect-ratio: 1 / 1;
  min-width: 0;
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  cursor: grab;
  overflow: hidden;
  position: relative;
  user-select: none;
  -webkit-user-select: none;
  -webkit-touch-callout: none;
  transition: transform 0.15s, box-shadow 0.15s, border-color 0.15s;

  /* Glass surface */
  background: rgba(255, 255, 255, 0.7);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid rgba(255, 255, 255, 0.5);
  box-shadow: 0 4px 30px rgba(0, 0, 0, 0.1);
}

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
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
}

.clipboard-card.selected {
  border-color: rgba(var(--type-rgb), 0.5);
  box-shadow:
    0 4px 30px rgba(0, 0, 0, 0.1),
    0 0 0 1px rgba(var(--type-rgb), 0.3);
}

.clipboard-card.dragging {
  opacity: 0.5;
  transform: scale(0.95);
  cursor: grabbing;
}

/* ============================================================================
   GLASS PILL — shared frosted-glass style for all pills/bubbles
   ============================================================================ */

.glass-pill {
  background: rgba(255, 255, 255, 0.55);
  backdrop-filter: blur(12px) saturate(150%);
  -webkit-backdrop-filter: blur(12px) saturate(150%);
  border: 1px solid rgba(255, 255, 255, 0.45);
  border-radius: 10px;
  box-shadow:
    0 1px 3px rgba(0, 0, 0, 0.06),
    inset 0 1px 0 rgba(255, 255, 255, 0.4);
}

/* ============================================================================
   DRAG SHIELD
   ============================================================================ */

.drag-shield {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1;
  cursor: grab;
  background: transparent;
}

.drag-shield:active {
  cursor: grabbing;
}

/* ============================================================================
   HEADER — glass title pill (left) + glass delete pill (right, on hover)
   ============================================================================ */

.card-header {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 7px 8px 4px;
  flex-shrink: 0;
  z-index: 2;
  position: relative;
}

.header-label {
  flex: 1;
  min-width: 0;
  padding: 3px 8px;
  font-size: 11px;
  font-weight: 600;
  color: #1f2937;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ============================================================================
   DELETE BUTTON — inline in header, revealed on hover
   ============================================================================ */

.delete-btn {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  padding: 0;
  border: none;
  background: rgba(239, 68, 68, 0.75);
  backdrop-filter: blur(12px) saturate(150%);
  -webkit-backdrop-filter: blur(12px) saturate(150%);
  color: white;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow:
    0 1px 3px rgba(239, 68, 68, 0.25),
    inset 0 1px 0 rgba(255, 255, 255, 0.2);
  z-index: 3;

  /* Hidden by default: slide in from right */
  max-width: 0;
  opacity: 0;
  overflow: hidden;
  margin-left: -4px;
  transition: max-width 0.2s ease, opacity 0.15s ease, margin-left 0.2s ease, transform 0.1s ease;
}

.clipboard-card:hover .delete-btn {
  max-width: 22px;
  opacity: 1;
  margin-left: 0;
}

.delete-btn:hover {
  background: rgba(220, 38, 38, 0.9);
  transform: scale(1.1);
}

/* ============================================================================
   FOOTER — [type pill] ··· [time + app icon]
   ============================================================================ */

.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 8px 7px;
  flex-shrink: 0;
  gap: 4px;
  z-index: 2;
  position: relative;
}

.type-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 7px 2px 5px;
  font-size: 10px;
  font-weight: 600;
  white-space: nowrap;
  color: rgb(var(--type-rgb));
}

.type-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
  background: rgb(var(--type-rgb));
}

.footer-meta {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 2px 5px 2px 7px;
  flex-shrink: 0;
}

.footer-time {
  font-size: 10px;
  font-weight: 500;
  color: rgba(0, 0, 0, 0.4);
  white-space: nowrap;
}

.footer-app-icon {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  object-fit: contain;
}

.footer-app-placeholder {
  width: 16px;
  height: 16px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(0, 0, 0, 0.25);
}

.footer-app-placeholder svg {
  width: 11px;
  height: 11px;
}

/* ============================================================================
   VISUAL CARD — image fills the card, header/footer float with glass pills
   ============================================================================ */

.visual-card {
  position: relative;
}

.visual-card .card-header {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: 2;
  padding: 7px 8px 0;
}

.visual-card .card-footer {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 2;
  padding: 0 8px 7px;
}

/* Visual card pills get darker glass for contrast on images */
.visual-card .glass-pill {
  background: rgba(0, 0, 0, 0.45);
  border-color: rgba(255, 255, 255, 0.15);
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.1);
}

.visual-card .header-label {
  color: rgba(255, 255, 255, 0.95);
}

.visual-card .delete-btn {
  background: rgba(239, 68, 68, 0.8);
  border-color: rgba(255, 255, 255, 0.15);
}

.visual-card .type-pill {
  color: rgba(255, 255, 255, 0.9);
}

.visual-card .type-dot {
  background: rgb(var(--type-rgb));
}

.visual-card .footer-time {
  color: rgba(255, 255, 255, 0.65);
}

.visual-card .footer-app-placeholder {
  color: rgba(255, 255, 255, 0.45);
}

.visual-content {
  position: absolute;
  inset: 0;
  overflow: hidden;
  border-radius: 16px;
}

.visual-preview {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.visual-badge {
  position: absolute;
  bottom: 32px;
  right: 8px;
  padding: 2px 7px;
  color: white;
  font-size: 10px;
  font-weight: 600;
  z-index: 2;
}

/* ============================================================================
   CARD CONTENT — standard (non-visual) cards
   ============================================================================ */

.card-content {
  flex: 1;
  padding: 4px 10px 10px;
  overflow: hidden;
  min-height: 0;
  position: relative;
  z-index: 1;
}

/* Text */
.text-content {
  height: 100%;
  overflow: hidden;
}

.text-preview {
  margin: 0;
  font-size: 11px;
  font-weight: 500;
  line-height: 1.4;
  color: #374151;
  word-break: break-word;
  display: -webkit-box;
  -webkit-line-clamp: 6;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* Placeholder */
.placeholder-content {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.placeholder-text {
  font-size: 10px;
  color: #9ca3af;
}

/* Icon-based content (files, link, audio, documents) */
.icon-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.content-icon {
  width: 40px;
  height: 40px;
  color: rgb(var(--type-rgb));
  opacity: 0.7;
}

.content-icon svg {
  width: 100%;
  height: 100%;
}

.content-label {
  margin: 0;
  font-size: 12px;
  font-weight: 500;
  color: #374151;
}

.content-sublabel {
  margin: 0;
  font-size: 10px;
  color: #6b7280;
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ============================================================================
   DARK MODE
   ============================================================================ */

html.dark .clipboard-card {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.12);
  box-shadow: 0 4px 30px rgba(0, 0, 0, 0.3);
}

html.dark .clipboard-card:hover {
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

html.dark .clipboard-card.selected {
  border-color: rgba(var(--type-rgb), 0.5);
  box-shadow:
    0 4px 30px rgba(0, 0, 0, 0.3),
    0 0 0 1px rgba(var(--type-rgb), 0.3);
}

/* Dark glass pills */
html.dark .glass-pill {
  background: rgba(0, 0, 0, 0.35);
  border-color: rgba(255, 255, 255, 0.12);
  box-shadow:
    0 1px 3px rgba(0, 0, 0, 0.15),
    inset 0 1px 0 rgba(255, 255, 255, 0.06);
}

html.dark .header-label {
  color: rgba(255, 255, 255, 0.9);
}

html.dark .type-pill {
  color: rgb(var(--type-rgb));
}

html.dark .footer-time {
  color: rgba(255, 255, 255, 0.45);
}

html.dark .footer-app-placeholder {
  color: rgba(255, 255, 255, 0.3);
}

html.dark .text-preview {
  color: rgba(255, 255, 255, 0.8);
}

html.dark .placeholder-text {
  color: rgba(255, 255, 255, 0.3);
}

html.dark .content-icon {
  color: rgb(var(--type-rgb));
  opacity: 0.8;
}

html.dark .content-label {
  color: rgba(255, 255, 255, 0.85);
}

html.dark .content-sublabel {
  color: rgba(255, 255, 255, 0.4);
}
</style>
