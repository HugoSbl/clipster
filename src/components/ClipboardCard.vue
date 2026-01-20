<script setup lang="ts">
import { computed, ref } from 'vue';
import type { ClipboardItem } from '@/types';

const props = defineProps<{
  item: ClipboardItem;
  selected?: boolean;
}>();

const emit = defineEmits<{
  select: [item: ClipboardItem];
  copy: [item: ClipboardItem];
  delete: [id: string];
  toggleFavorite: [id: string];
}>();

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

// Handle card click
const handleClick = () => {
  emit('select', props.item);
};

// Handle double click to copy
const handleDoubleClick = () => {
  emit('copy', props.item);
};

// Handle delete
const handleDelete = (e: Event) => {
  e.stopPropagation();
  emit('delete', props.item.id);
};

// Handle favorite toggle
const handleToggleFavorite = (e: Event) => {
  e.stopPropagation();
  emit('toggleFavorite', props.item.id);
};

// Drag handlers
const handleDragStart = (e: DragEvent) => {
  isDragging.value = true;
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('application/x-clipboard-item', props.item.id);
    // Also set text data for external drops
    if (props.item.content_type === 'text' && props.item.content_text) {
      e.dataTransfer.setData('text/plain', props.item.content_text);
    }
  }
};

const handleDragEnd = () => {
  isDragging.value = false;
};
</script>

<template>
  <div
    class="clipboard-card"
    :class="{
      selected: selected,
      favorite: item.is_favorite,
      dragging: isDragging,
      'type-text': item.content_type === 'text',
      'type-image': item.content_type === 'image',
      'type-files': item.content_type === 'files',
    }"
    draggable="true"
    @click="handleClick"
    @dblclick="handleDoubleClick"
    @dragstart="handleDragStart"
    @dragend="handleDragEnd"
  >
    <!-- Card Header -->
    <div class="card-header">
      <span class="type-indicator">
        <span v-if="item.content_type === 'text'" class="type-icon text">T</span>
        <span v-else-if="item.content_type === 'image'" class="type-icon image">I</span>
        <span v-else class="type-icon files">F</span>
      </span>
      <span class="timestamp">{{ formattedTime }}</span>
    </div>

    <!-- Card Content -->
    <div class="card-content">
      <!-- Text Content -->
      <div v-if="item.content_type === 'text'" class="text-content">
        <p class="text-preview">{{ textPreview }}</p>
      </div>

      <!-- Image Content -->
      <div v-else-if="item.content_type === 'image'" class="image-content">
        <img
          v-if="item.thumbnail_base64"
          :src="`data:image/png;base64,${item.thumbnail_base64}`"
          alt="Image"
          class="thumbnail"
          loading="lazy"
        />
        <div v-else class="thumbnail-placeholder">
          <span>No preview</span>
        </div>
      </div>

      <!-- Files Content -->
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
    </div>

    <!-- Card Actions -->
    <div class="card-actions">
      <button
        class="action-btn favorite-btn"
        :class="{ active: item.is_favorite }"
        @click="handleToggleFavorite"
        title="Toggle favorite"
      >
        {{ item.is_favorite ? '★' : '☆' }}
      </button>
      <button class="action-btn delete-btn" @click="handleDelete" title="Delete">×</button>
    </div>
  </div>
</template>

<style scoped>
.clipboard-card {
  flex-shrink: 0;
  width: 180px;
  height: 140px;
  background: white;
  border-radius: 8px;
  border: 2px solid transparent;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
  cursor: pointer;
  transition: transform 0.15s, box-shadow 0.15s, border-color 0.15s;
  contain: content;
  overflow: hidden;
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

.clipboard-card.favorite {
  background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
}

/* Card Header */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 10px 4px;
  flex-shrink: 0;
}

.type-indicator {
  display: flex;
  align-items: center;
}

.type-icon {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
}

.type-icon.text {
  background: #dbeafe;
  color: #1d4ed8;
}

.type-icon.image {
  background: #f3e8ff;
  color: #7c3aed;
}

.type-icon.files {
  background: #dcfce7;
  color: #16a34a;
}

.timestamp {
  font-size: 10px;
  color: #9ca3af;
}

/* Card Content */
.card-content {
  flex: 1;
  padding: 4px 10px;
  overflow: hidden;
  min-height: 0;
}

/* Text Content */
.text-content {
  height: 100%;
  overflow: hidden;
}

.text-preview {
  margin: 0;
  font-size: 12px;
  line-height: 1.4;
  color: #374151;
  word-break: break-word;
  display: -webkit-box;
  -webkit-line-clamp: 4;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* Image Content */
.image-content {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.thumbnail {
  max-width: 100%;
  max-height: 70px;
  border-radius: 4px;
  object-fit: contain;
}

.thumbnail-placeholder {
  width: 80px;
  height: 50px;
  background: #f3f4f6;
  border-radius: 4px;
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

/* Card Actions */
.card-actions {
  display: flex;
  justify-content: flex-end;
  gap: 4px;
  padding: 4px 8px 8px;
  opacity: 0;
  transition: opacity 0.15s;
}

.clipboard-card:hover .card-actions {
  opacity: 1;
}

.action-btn {
  width: 24px;
  height: 24px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: rgba(0, 0, 0, 0.05);
  cursor: pointer;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s;
}

.action-btn:hover {
  background: rgba(0, 0, 0, 0.1);
}

.favorite-btn {
  color: #f59e0b;
}

.favorite-btn.active {
  background: #fef3c7;
}

.delete-btn {
  color: #ef4444;
}

.delete-btn:hover {
  background: #fee2e2;
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
  .clipboard-card {
    background: #1f2937;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .clipboard-card.favorite {
    background: linear-gradient(135deg, #422006 0%, #451a03 100%);
  }

  .clipboard-card.selected {
    border-color: #60a5fa;
  }

  .text-preview {
    color: #e5e7eb;
  }

  .thumbnail-placeholder {
    background: #374151;
  }

  .file-count {
    color: #e5e7eb;
  }

  .file-name {
    color: #9ca3af;
  }

  .action-btn {
    background: rgba(255, 255, 255, 0.1);
  }

  .action-btn:hover {
    background: rgba(255, 255, 255, 0.2);
  }

  .favorite-btn.active {
    background: #451a03;
  }

  .delete-btn:hover {
    background: #450a0a;
  }
}
</style>
