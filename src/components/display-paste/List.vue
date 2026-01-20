<script setup lang="ts">
import { computed, ref } from 'vue';
import { useClipboardStore } from '@/stores/clipboard';
import type { ClipboardItem } from '@/types';

const store = useClipboardStore();

// Computed properties
const items = computed(() => store.filteredItems);
const loading = computed(() => store.loading);
const hasItems = computed(() => store.hasItems);
const error = computed(() => store.error);

// Image preview modal state
const previewImage = ref<string | null>(null);
const previewLoading = ref(false);

// Actions
const handleCopy = async (item: ClipboardItem) => {
  const success = await store.copyToClipboard(item.id);
  if (success) {
    console.log('Copied to clipboard:', item.id);
  }
};

const handleDelete = async (id: string, event: Event) => {
  event.stopPropagation();
  await store.deleteItem(id);
};

const handleToggleFavorite = async (id: string, event: Event) => {
  event.stopPropagation();
  await store.toggleFavorite(id);
};

// Open image preview modal
const openImagePreview = async (item: ClipboardItem, event: Event) => {
  event.stopPropagation();
  if (item.content_type !== 'image') return;

  previewLoading.value = true;
  const imageData = await store.getImageData(item.id);
  previewLoading.value = false;

  if (imageData) {
    previewImage.value = `data:image/png;base64,${imageData}`;
  }
};

// Close image preview modal
const closeImagePreview = () => {
  previewImage.value = null;
};

// Format timestamp for display
const formatTime = (timestamp: string): string => {
  const date = new Date(timestamp);
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
};

// Truncate text for preview
const truncateText = (text: string | null, maxLength = 100): string => {
  if (!text) return '';
  if (text.length <= maxLength) return text;
  return text.substring(0, maxLength) + '...';
};

// Get parsed file list
const getFilePaths = (item: ClipboardItem): string[] => {
  if (item.content_type !== 'files' || !item.content_text) return [];
  try {
    return JSON.parse(item.content_text) as string[];
  } catch {
    return [];
  }
};

// Get filename from path
const getFileName = (path: string): string => {
  return path.split(/[/\\]/).pop() || path;
};
</script>

<template>
  <div class="clipboard-list">
    <!-- Header -->
    <div class="list-header">
      <h2>Clipboard History</h2>
      <span class="item-count">{{ store.totalCount }} items</span>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="loading">
      <span>Loading...</span>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="error">
      <span>{{ error }}</span>
      <button @click="store.clearError()">Dismiss</button>
    </div>

    <!-- Empty state -->
    <div v-else-if="!hasItems" class="empty-state">
      <p>No clipboard items yet.</p>
      <p class="hint">Copy something to get started!</p>
    </div>

    <!-- Item list -->
    <div v-else class="items-container">
      <div
        v-for="item in items"
        :key="item.id"
        class="clipboard-item"
        :class="{ favorite: item.is_favorite }"
        @click="handleCopy(item)"
      >
        <!-- Content preview -->
        <div class="item-content">
          <div class="item-type">
            <span v-if="item.content_type === 'text'" class="type-badge text">T</span>
            <span v-else-if="item.content_type === 'image'" class="type-badge image">I</span>
            <span v-else class="type-badge files">F</span>
          </div>

          <!-- Text preview -->
          <div v-if="item.content_type === 'text'" class="item-text">
            <p class="preview">{{ truncateText(item.content_text) }}</p>
            <span class="timestamp">{{ formatTime(item.created_at) }}</span>
          </div>

          <!-- Image preview with thumbnail -->
          <div v-else-if="item.content_type === 'image'" class="item-image">
            <div class="thumbnail-container" @click="openImagePreview(item, $event)">
              <img
                v-if="item.thumbnail_base64"
                :src="`data:image/png;base64,${item.thumbnail_base64}`"
                alt="Thumbnail"
                class="thumbnail"
              />
              <div v-else class="thumbnail-placeholder">No preview</div>
            </div>
            <span class="timestamp">{{ formatTime(item.created_at) }}</span>
          </div>

          <!-- Files preview -->
          <div v-else-if="item.content_type === 'files'" class="item-files">
            <div class="files-list">
              <template v-if="getFilePaths(item).length <= 3">
                <p v-for="(path, idx) in getFilePaths(item)" :key="idx" class="file-path">
                  {{ getFileName(path) }}
                </p>
              </template>
              <template v-else>
                <p class="file-path">{{ getFileName(getFilePaths(item)[0]) }}</p>
                <p class="file-count">+{{ getFilePaths(item).length - 1 }} more files</p>
              </template>
            </div>
            <span class="timestamp">{{ formatTime(item.created_at) }}</span>
          </div>
        </div>

        <!-- Actions -->
        <div class="item-actions">
          <button
            class="action-btn favorite-btn"
            :class="{ active: item.is_favorite }"
            @click="handleToggleFavorite(item.id, $event)"
            title="Toggle favorite"
          >
            {{ item.is_favorite ? '★' : '☆' }}
          </button>
          <button
            class="action-btn delete-btn"
            @click="handleDelete(item.id, $event)"
            title="Delete"
          >
            ×
          </button>
        </div>
      </div>
    </div>

    <!-- Image Preview Modal -->
    <div v-if="previewImage || previewLoading" class="modal-overlay" @click="closeImagePreview">
      <div class="modal-content" @click.stop>
        <button class="modal-close" @click="closeImagePreview">×</button>
        <div v-if="previewLoading" class="modal-loading">Loading...</div>
        <img v-else :src="previewImage!" alt="Full image" class="preview-image" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.clipboard-list {
  width: 100%;
  max-width: 600px;
  margin: 0 auto;
  text-align: left;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 1rem;
  border-bottom: 1px solid #e0e0e0;
}

.list-header h2 {
  margin: 0;
  font-size: 1.2rem;
}

.item-count {
  font-size: 0.85rem;
  color: #666;
}

.loading,
.error,
.empty-state {
  padding: 2rem;
  text-align: center;
  color: #666;
}

.error {
  color: #e53935;
}

.empty-state .hint {
  font-size: 0.9rem;
  color: #999;
}

.items-container {
  max-height: 500px;
  overflow-y: auto;
}

.clipboard-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #e0e0e0;
  cursor: pointer;
  transition: background-color 0.15s;
}

.clipboard-item:hover {
  background-color: #f5f5f5;
}

.clipboard-item.favorite {
  background-color: #fff8e1;
}

.clipboard-item.favorite:hover {
  background-color: #ffecb3;
}

.item-content {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  flex: 1;
  min-width: 0;
}

.type-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: bold;
  flex-shrink: 0;
}

.type-badge.text {
  background-color: #e3f2fd;
  color: #1976d2;
}

.type-badge.image {
  background-color: #f3e5f5;
  color: #7b1fa2;
}

.type-badge.files {
  background-color: #e8f5e9;
  color: #388e3c;
}

.item-text,
.item-image,
.item-files {
  flex: 1;
  min-width: 0;
}

.preview {
  margin: 0;
  font-size: 0.9rem;
  line-height: 1.4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.timestamp {
  font-size: 0.75rem;
  color: #999;
}

/* Thumbnail styles */
.thumbnail-container {
  display: inline-block;
  cursor: zoom-in;
}

.thumbnail {
  max-width: 120px;
  max-height: 60px;
  border-radius: 4px;
  object-fit: cover;
  border: 1px solid #e0e0e0;
}

.thumbnail-placeholder {
  width: 80px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #f5f5f5;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  font-size: 0.7rem;
  color: #999;
}

/* Files list styles */
.files-list {
  margin: 0;
}

.file-path {
  margin: 0;
  font-size: 0.85rem;
  color: #333;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-count {
  margin: 0;
  font-size: 0.8rem;
  color: #666;
}

/* Action buttons */
.item-actions {
  display: flex;
  gap: 0.25rem;
  flex-shrink: 0;
}

.action-btn {
  width: 28px;
  height: 28px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  font-size: 1rem;
  opacity: 0.5;
  transition: opacity 0.15s, background-color 0.15s;
}

.action-btn:hover {
  opacity: 1;
  background-color: #e0e0e0;
}

.favorite-btn {
  color: #ffc107;
}

.favorite-btn.active {
  opacity: 1;
}

.delete-btn {
  color: #e53935;
}

.delete-btn:hover {
  background-color: #ffebee;
}

/* Modal styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  position: relative;
  max-width: 90vw;
  max-height: 90vh;
  background: white;
  border-radius: 8px;
  padding: 1rem;
}

.modal-close {
  position: absolute;
  top: -10px;
  right: -10px;
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 50%;
  background: #333;
  color: white;
  font-size: 1.2rem;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-close:hover {
  background: #555;
}

.modal-loading {
  padding: 2rem;
  color: #666;
}

.preview-image {
  max-width: 80vw;
  max-height: 80vh;
  object-fit: contain;
}

@media (prefers-color-scheme: dark) {
  .list-header {
    border-bottom-color: #444;
  }

  .clipboard-item {
    border-bottom-color: #444;
  }

  .clipboard-item:hover {
    background-color: #3a3a3a;
  }

  .clipboard-item.favorite {
    background-color: #4a4000;
  }

  .clipboard-item.favorite:hover {
    background-color: #5a5000;
  }

  .action-btn:hover {
    background-color: #444;
  }

  .delete-btn:hover {
    background-color: #4a2020;
  }

  .thumbnail {
    border-color: #444;
  }

  .thumbnail-placeholder {
    background-color: #333;
    border-color: #444;
  }

  .file-path {
    color: #ccc;
  }

  .modal-content {
    background: #2a2a2a;
  }
}
</style>
