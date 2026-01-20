<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useClipboardStore } from '@/stores/clipboard';
import ClipboardCard from './ClipboardCard.vue';
import type { ClipboardItem } from '@/types';

const store = useClipboardStore();

// State
const selectedId = ref<string | null>(null);
const previewImage = ref<string | null>(null);
const previewLoading = ref(false);
const timelineRef = ref<HTMLElement | null>(null);

// Computed
const items = computed(() => store.filteredItems);
const loading = computed(() => store.loading);
const hasItems = computed(() => store.hasItems);
const error = computed(() => store.error);
const isModalOpen = computed(() => previewImage.value !== null || previewLoading.value);

// Watch for new items and select the first one
watch(
  () => items.value.length,
  (newLen, oldLen) => {
    if (newLen > oldLen && items.value.length > 0) {
      // New item added, select it
      selectedId.value = items.value[0].id;
      // Scroll to start
      if (timelineRef.value) {
        timelineRef.value.scrollTo({ left: 0, behavior: 'smooth' });
      }
    }
  }
);

// Handlers
const handleSelect = (item: ClipboardItem) => {
  selectedId.value = item.id;
};

const handleCopy = async (item: ClipboardItem) => {
  const success = await store.copyToClipboard(item.id);
  if (success) {
    console.log('Copied to clipboard:', item.id);
  }
};

const handleDelete = async (id: string) => {
  await store.deleteItem(id);
  if (selectedId.value === id) {
    selectedId.value = items.value.length > 0 ? items.value[0].id : null;
  }
};

const handleToggleFavorite = async (id: string) => {
  await store.toggleFavorite(id);
};

// Image preview
const openImagePreview = async (item: ClipboardItem) => {
  if (item.content_type !== 'image') return;

  previewLoading.value = true;
  const imageData = await store.getImageData(item.id);
  previewLoading.value = false;

  if (imageData) {
    previewImage.value = `data:image/png;base64,${imageData}`;
  }
};

const closeImagePreview = () => {
  previewImage.value = null;
};

// Navigation methods (exposed for global keyboard handling)
const navigateLeft = () => {
  if (!items.value.length) return;
  const currentIndex = selectedId.value
    ? items.value.findIndex((item) => item.id === selectedId.value)
    : -1;
  if (currentIndex > 0) {
    selectedId.value = items.value[currentIndex - 1].id;
    scrollToSelected(currentIndex - 1);
  } else if (currentIndex === -1 && items.value.length > 0) {
    // No selection, select last item
    selectedId.value = items.value[items.value.length - 1].id;
    scrollToSelected(items.value.length - 1);
  }
};

const navigateRight = () => {
  if (!items.value.length) return;
  const currentIndex = selectedId.value
    ? items.value.findIndex((item) => item.id === selectedId.value)
    : -1;
  if (currentIndex < items.value.length - 1) {
    selectedId.value = items.value[currentIndex + 1].id;
    scrollToSelected(currentIndex + 1);
  } else if (currentIndex === -1 && items.value.length > 0) {
    // No selection, select first item
    selectedId.value = items.value[0].id;
    scrollToSelected(0);
  }
};

const selectCurrent = () => {
  if (!selectedId.value) return;
  const item = items.value.find((i) => i.id === selectedId.value);
  if (item) {
    if (item.content_type === 'image') {
      openImagePreview(item);
    } else {
      handleCopy(item);
    }
  }
};

const deleteCurrent = () => {
  if (selectedId.value) {
    handleDelete(selectedId.value);
  }
};

const toggleFavoriteCurrent = () => {
  if (selectedId.value) {
    handleToggleFavorite(selectedId.value);
  }
};

const clearSelection = () => {
  if (isModalOpen.value) {
    closeImagePreview();
  } else {
    selectedId.value = null;
  }
};

// Expose methods for parent component
defineExpose({
  navigateLeft,
  navigateRight,
  selectCurrent,
  deleteCurrent,
  toggleFavoriteCurrent,
  clearSelection,
  isModalOpen,
});

// Scroll to selected card
const scrollToSelected = (index: number) => {
  if (!timelineRef.value) return;
  const cards = timelineRef.value.querySelectorAll('.clipboard-card');
  if (cards[index]) {
    cards[index].scrollIntoView({ behavior: 'smooth', inline: 'center', block: 'nearest' });
  }
};

// Handle click on selected image card to open preview
const handleCardClick = (item: ClipboardItem) => {
  if (selectedId.value === item.id && item.content_type === 'image') {
    openImagePreview(item);
  } else {
    handleSelect(item);
  }
};
</script>

<template>
  <div class="timeline-container">
    <!-- Header -->
    <div class="timeline-header">
      <h2>Clipboard History</h2>
      <div class="header-info">
        <span class="item-count">{{ store.totalCount }} items</span>
        <span class="hint">← → navigate, Enter copy, / search</span>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="loading-state">
      <span>Loading...</span>
    </div>

    <!-- Error state -->
    <div v-else-if="error" class="error-state">
      <span>{{ error }}</span>
      <button @click="store.clearError()">Dismiss</button>
    </div>

    <!-- Empty state -->
    <div v-else-if="!hasItems" class="empty-state">
      <div class="empty-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="8" y="2" width="8" height="4" rx="1" ry="1" />
          <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
          <line x1="12" y1="11" x2="12" y2="17" />
          <line x1="9" y1="14" x2="15" y2="14" />
        </svg>
      </div>
      <p>No clipboard items yet</p>
      <p class="empty-hint">Copy something to get started!</p>
    </div>

    <!-- Timeline -->
    <div v-else ref="timelineRef" class="timeline-scroll">
      <div class="timeline-track">
        <ClipboardCard
          v-for="item in items"
          :key="item.id"
          :item="item"
          :selected="selectedId === item.id"
          @select="handleCardClick"
          @copy="handleCopy"
          @delete="handleDelete"
          @toggle-favorite="handleToggleFavorite"
        />
      </div>
    </div>

    <!-- Image Preview Modal -->
    <Teleport to="body">
      <div
        v-if="previewImage || previewLoading"
        class="modal-overlay"
        @click="closeImagePreview"
      >
        <div class="modal-content" @click.stop>
          <button class="modal-close" @click="closeImagePreview">×</button>
          <div v-if="previewLoading" class="modal-loading">Loading...</div>
          <img v-else :src="previewImage!" alt="Full image" class="preview-image" />
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.timeline-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  outline: none;
}

/* Header */
.timeline-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #e5e7eb;
  flex-shrink: 0;
}

.timeline-header h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #111827;
}

.header-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.item-count {
  font-size: 13px;
  color: #6b7280;
}

.hint {
  font-size: 11px;
  color: #9ca3af;
}

/* Loading, Error, Empty states */
.loading-state,
.error-state,
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  color: #6b7280;
}

.error-state {
  color: #dc2626;
}

.error-state button {
  margin-top: 8px;
  padding: 6px 12px;
  font-size: 13px;
}

.empty-icon {
  width: 64px;
  height: 64px;
  color: #d1d5db;
  margin-bottom: 16px;
}

.empty-icon svg {
  width: 100%;
  height: 100%;
}

.empty-state p {
  margin: 0;
  font-size: 14px;
}

.empty-hint {
  margin-top: 4px !important;
  font-size: 12px !important;
  color: #9ca3af;
}

/* Timeline Scroll */
.timeline-scroll {
  flex: 1;
  overflow-x: auto;
  overflow-y: hidden;
  padding: 16px;
  scroll-behavior: smooth;
  scroll-snap-type: x mandatory;
}

/* Custom scrollbar */
.timeline-scroll::-webkit-scrollbar {
  height: 8px;
}

.timeline-scroll::-webkit-scrollbar-track {
  background: #f3f4f6;
  border-radius: 4px;
}

.timeline-scroll::-webkit-scrollbar-thumb {
  background: #d1d5db;
  border-radius: 4px;
}

.timeline-scroll::-webkit-scrollbar-thumb:hover {
  background: #9ca3af;
}

/* Timeline Track */
.timeline-track {
  display: flex;
  gap: 12px;
  padding-bottom: 8px;
  min-width: max-content;
}

.timeline-track > * {
  scroll-snap-align: start;
}

/* Modal styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.modal-content {
  position: relative;
  max-width: 90vw;
  max-height: 90vh;
  background: white;
  border-radius: 8px;
  padding: 16px;
}

.modal-close {
  position: absolute;
  top: -12px;
  right: -12px;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 50%;
  background: #1f2937;
  color: white;
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

.modal-close:hover {
  background: #374151;
}

.modal-loading {
  padding: 32px 48px;
  color: #6b7280;
  font-size: 14px;
}

.preview-image {
  max-width: 85vw;
  max-height: 85vh;
  object-fit: contain;
  border-radius: 4px;
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
  .timeline-header {
    border-bottom-color: #374151;
  }

  .timeline-header h2 {
    color: #f9fafb;
  }

  .item-count {
    color: #9ca3af;
  }

  .timeline-scroll::-webkit-scrollbar-track {
    background: #1f2937;
  }

  .timeline-scroll::-webkit-scrollbar-thumb {
    background: #4b5563;
  }

  .timeline-scroll::-webkit-scrollbar-thumb:hover {
    background: #6b7280;
  }

  .empty-icon {
    color: #4b5563;
  }

  .modal-content {
    background: #1f2937;
  }
}
</style>
