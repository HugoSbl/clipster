<script setup lang="ts">
import { ref, computed, nextTick } from 'vue';
import draggable from 'vuedraggable';
import { usePinboardStore } from '@/stores/pinboards';
import type { Pinboard } from '@/types';

const store = usePinboardStore();

// Local state
const showCreateInput = ref(false);
const newPinboardName = ref('');
const createInputRef = ref<HTMLInputElement | null>(null);

// Context menu state
const contextMenu = ref<{
  show: boolean;
  x: number;
  y: number;
  pinboard: Pinboard | null;
}>({
  show: false,
  x: 0,
  y: 0,
  pinboard: null,
});

// Edit state
const editingId = ref<string | null>(null);
const editingName = ref('');
const editInputRef = ref<HTMLInputElement | null>(null);

// Drop state
const dropTargetId = ref<string | null>(null);

// Computed
const sortedPinboards = computed(() => store.sortedPinboards);
const activePinboardId = computed(() => store.activePinboardId);

// Draggable model for vuedraggable
const draggablePinboards = computed({
  get: () => sortedPinboards.value,
  set: (value: Pinboard[]) => {
    const ids = value.map((p) => p.id);
    store.reorderPinboards(ids);
  },
});

// Handle tab click
const handleTabClick = (pinboardId: string | null) => {
  store.setActivePinboard(pinboardId);
};

// Show create input
const showCreate = async () => {
  showCreateInput.value = true;
  newPinboardName.value = '';
  await nextTick();
  createInputRef.value?.focus();
};

// Create pinboard
const createPinboard = async () => {
  const name = newPinboardName.value.trim();
  if (name) {
    await store.createPinboard(name);
  }
  showCreateInput.value = false;
  newPinboardName.value = '';
};

// Cancel create
const cancelCreate = () => {
  showCreateInput.value = false;
  newPinboardName.value = '';
};

// Handle create input keydown
const handleCreateKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    createPinboard();
  } else if (e.key === 'Escape') {
    cancelCreate();
  }
};

// Context menu handlers
const openContextMenu = (e: MouseEvent, pinboard: Pinboard) => {
  e.preventDefault();
  contextMenu.value = {
    show: true,
    x: e.clientX,
    y: e.clientY,
    pinboard,
  };
};

const closeContextMenu = () => {
  contextMenu.value.show = false;
  contextMenu.value.pinboard = null;
};

// Start editing
const startEdit = async () => {
  if (!contextMenu.value.pinboard) return;
  editingId.value = contextMenu.value.pinboard.id;
  editingName.value = contextMenu.value.pinboard.name;
  closeContextMenu();
  await nextTick();
  editInputRef.value?.focus();
  editInputRef.value?.select();
};

// Save edit
const saveEdit = async () => {
  if (editingId.value && editingName.value.trim()) {
    const pinboard = store.pinboards.find((p) => p.id === editingId.value);
    await store.updatePinboard(editingId.value, editingName.value.trim(), pinboard?.icon || undefined);
  }
  editingId.value = null;
  editingName.value = '';
};

// Cancel edit
const cancelEdit = () => {
  editingId.value = null;
  editingName.value = '';
};

// Handle edit input keydown
const handleEditKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter') {
    saveEdit();
  } else if (e.key === 'Escape') {
    cancelEdit();
  }
};

// Delete pinboard
const deletePinboard = async () => {
  if (!contextMenu.value.pinboard) return;
  if (confirm(`Delete "${contextMenu.value.pinboard.name}"? Items will be moved to history.`)) {
    await store.deletePinboard(contextMenu.value.pinboard.id);
  }
  closeContextMenu();
};

// Close context menu when clicking outside
const handleClickOutside = () => {
  if (contextMenu.value.show) {
    closeContextMenu();
  }
};

// Drop handlers
const handleDragOver = (e: DragEvent, pinboardId: string) => {
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'move';
  }
  dropTargetId.value = pinboardId;
};

const handleDragLeave = () => {
  dropTargetId.value = null;
};

const handleDrop = async (e: DragEvent, pinboardId: string) => {
  e.preventDefault();
  dropTargetId.value = null;

  const itemId = e.dataTransfer?.getData('application/x-clipboard-item');
  if (itemId && pinboardId) {
    await store.addItemToPinboard(itemId, pinboardId);
  }
};

// Handle drop on history tab (removes from pinboard)
const handleDropOnHistory = async (e: DragEvent) => {
  e.preventDefault();
  dropTargetId.value = null;

  const itemId = e.dataTransfer?.getData('application/x-clipboard-item');
  if (itemId) {
    await store.removeItemFromPinboard(itemId);
  }
};

const handleDragOverHistory = (e: DragEvent) => {
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'move';
  }
  dropTargetId.value = 'history';
};
</script>

<template>
  <div class="pinboard-tabs" @click="handleClickOutside">
    <!-- History Tab (always first, not draggable, but is drop target) -->
    <button
      class="tab history-tab"
      :class="{ active: activePinboardId === null, 'drop-target': dropTargetId === 'history' }"
      @click="handleTabClick(null)"
      @dragover="handleDragOverHistory"
      @dragleave="handleDragLeave"
      @drop="handleDropOnHistory"
    >
      <span class="tab-icon">📋</span>
      <span class="tab-name">History</span>
    </button>

    <!-- Draggable Pinboard Tabs -->
    <draggable
      v-model="draggablePinboards"
      item-key="id"
      class="draggable-tabs"
      ghost-class="tab-ghost"
      drag-class="tab-dragging"
      :animation="150"
    >
      <template #item="{ element: pinboard }">
        <button
          class="tab pinboard-tab"
          :class="{ active: activePinboardId === pinboard.id, 'drop-target': dropTargetId === pinboard.id }"
          @click="handleTabClick(pinboard.id)"
          @contextmenu="openContextMenu($event, pinboard)"
          @dragover="handleDragOver($event, pinboard.id)"
          @dragleave="handleDragLeave"
          @drop="handleDrop($event, pinboard.id)"
        >
          <!-- Editing state -->
          <template v-if="editingId === pinboard.id">
            <input
              ref="editInputRef"
              v-model="editingName"
              type="text"
              class="tab-edit-input"
              @blur="saveEdit"
              @keydown="handleEditKeydown"
              @click.stop
            />
          </template>
          <!-- Normal state -->
          <template v-else>
            <span class="tab-icon">{{ pinboard.icon || '📌' }}</span>
            <span class="tab-name">{{ pinboard.name }}</span>
          </template>
        </button>
      </template>
    </draggable>

    <!-- Create New Pinboard -->
    <div v-if="showCreateInput" class="create-input-wrapper">
      <input
        ref="createInputRef"
        v-model="newPinboardName"
        type="text"
        class="create-input"
        placeholder="Pinboard name..."
        @blur="cancelCreate"
        @keydown="handleCreateKeydown"
      />
    </div>
    <button v-else class="tab add-tab" @click="showCreate" title="Create pinboard">
      <span class="tab-icon">+</span>
    </button>

    <!-- Context Menu -->
    <Teleport to="body">
      <div
        v-if="contextMenu.show"
        class="context-menu"
        :style="{ top: `${contextMenu.y}px`, left: `${contextMenu.x}px` }"
        @click.stop
      >
        <button class="context-item" @click="startEdit">
          <span class="context-icon">✏️</span>
          Rename
        </button>
        <button class="context-item danger" @click="deletePinboard">
          <span class="context-icon">🗑️</span>
          Delete
        </button>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.pinboard-tabs {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
  overflow-x: auto;
  flex-shrink: 0;
}

.pinboard-tabs::-webkit-scrollbar {
  height: 4px;
}

.pinboard-tabs::-webkit-scrollbar-thumb {
  background: #d1d5db;
  border-radius: 2px;
}

.draggable-tabs {
  display: flex;
  gap: 4px;
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
  font-size: 13px;
  color: #6b7280;
  transition: background-color 0.15s, color 0.15s;
  white-space: nowrap;
}

.tab:hover {
  background: #e5e7eb;
  color: #374151;
}

.tab.active {
  background: white;
  color: #1f2937;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.tab-icon {
  font-size: 14px;
}

.tab-name {
  font-weight: 500;
}

.add-tab {
  padding: 6px 10px;
  color: #9ca3af;
}

.add-tab:hover {
  background: #dbeafe;
  color: #3b82f6;
}

/* Drop target state */
.tab.drop-target {
  background: #dbeafe;
  border: 2px dashed #3b82f6;
  color: #1d4ed8;
}

.add-tab .tab-icon {
  font-size: 16px;
  font-weight: 600;
}

/* Dragging states */
.tab-ghost {
  opacity: 0.5;
  background: #dbeafe;
}

.tab-dragging {
  background: white;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

/* Create input */
.create-input-wrapper {
  display: flex;
  align-items: center;
}

.create-input {
  width: 120px;
  padding: 6px 10px;
  border: 1px solid #3b82f6;
  border-radius: 6px;
  font-size: 13px;
  outline: none;
  background: white;
}

/* Edit input */
.tab-edit-input {
  width: 80px;
  padding: 2px 6px;
  border: 1px solid #3b82f6;
  border-radius: 4px;
  font-size: 13px;
  outline: none;
  background: white;
}

/* Context menu */
.context-menu {
  position: fixed;
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  padding: 4px;
  min-width: 120px;
  z-index: 10000;
}

.context-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  border-radius: 4px;
  background: transparent;
  cursor: pointer;
  font-size: 13px;
  color: #374151;
  text-align: left;
}

.context-item:hover {
  background: #f3f4f6;
}

.context-item.danger {
  color: #dc2626;
}

.context-item.danger:hover {
  background: #fef2f2;
}

.context-icon {
  font-size: 14px;
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
  .pinboard-tabs {
    background: #111827;
    border-bottom-color: #374151;
  }

  .pinboard-tabs::-webkit-scrollbar-thumb {
    background: #4b5563;
  }

  .tab {
    color: #9ca3af;
  }

  .tab:hover {
    background: #374151;
    color: #e5e7eb;
  }

  .tab.active {
    background: #1f2937;
    color: #f9fafb;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }

  .add-tab:hover {
    background: #1e3a5f;
    color: #60a5fa;
  }

  .tab-ghost {
    background: #1e3a5f;
  }

  .tab-dragging {
    background: #1f2937;
  }

  .create-input,
  .tab-edit-input {
    background: #1f2937;
    border-color: #60a5fa;
    color: #f3f4f6;
  }

  .context-menu {
    background: #1f2937;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  }

  .context-item {
    color: #e5e7eb;
  }

  .context-item:hover {
    background: #374151;
  }

  .context-item.danger {
    color: #f87171;
  }

  .context-item.danger:hover {
    background: #450a0a;
  }

  .tab.drop-target {
    background: #1e3a5f;
    border: 2px dashed #60a5fa;
    color: #93c5fd;
  }
}
</style>
