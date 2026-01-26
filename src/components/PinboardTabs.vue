<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import { usePinboardStore } from '@/stores/pinboards';
import type { Pinboard } from '@/types';

const store = usePinboardStore();

// Handle custom internal drop events from ClipboardCard's mouse-based drag
const handleInternalDrop = async (e: Event) => {
  const customEvent = e as CustomEvent<{ itemId: string; zoneId: string }>;
  const { itemId, zoneId } = customEvent.detail;

  console.log('[PinboardTabs] Internal drop:', itemId, 'to', zoneId);

  if (zoneId === 'history') {
    await store.removeItemFromPinboard(itemId);
  } else if (zoneId.startsWith('pinboard-')) {
    const pinboardId = zoneId.replace('pinboard-', '');
    await store.addItemToPinboard(itemId, pinboardId);
  }
};

onMounted(() => {
  document.addEventListener('clipster-internal-drop', handleInternalDrop);
});

onUnmounted(() => {
  document.removeEventListener('clipster-internal-drop', handleInternalDrop);
});

// Available icons for pinboards
const availableIcons = [
  '📌', '⭐', '💼', '📁', '🏠', '💡', '🎯', '🔖',
  '📝', '✅', '🔥', '💎', '🎨', '🎵', '📷', '🎬',
  '💻', '📱', '🌐', '📧', '💬', '📊', '📈', '🛠️',
];

// Local state
const showCreatePopover = ref(false);
const newPinboardName = ref('');
const newPinboardIcon = ref('📌');
const createInputRef = ref<HTMLInputElement | null>(null);
const addBtnRef = ref<HTMLButtonElement | null>(null);
const createPopoverPosition = ref({ x: 0, y: 0 });

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

// Edit popover state
const showEditPopover = ref(false);
const editingPinboard = ref<Pinboard | null>(null);
const editingName = ref('');
const editingIcon = ref('📌');
const editInputRef = ref<HTMLInputElement | null>(null);
const editPopoverPosition = ref({ x: 0, y: 0 });

// Drop target state for native drag
const dropTargetId = ref<string | null>(null);

// Computed
const sortedPinboards = computed(() => store.sortedPinboards);
const activePinboardId = computed(() => store.activePinboardId);
const isDraggingItem = computed(() => store.isDraggingItem);

// Handle tab click
const handleTabClick = (pinboardId: string | null) => {
  store.setActivePinboard(pinboardId);
};

// Native drag and drop handlers
const isValidDragData = (e: DragEvent): boolean => {
  if (!e.dataTransfer) return false;
  return e.dataTransfer.types.includes('text/plain') ||
    e.dataTransfer.types.includes('application/x-clipboard-item');
};

const getItemIdFromDrag = (e: DragEvent): string | null => {
  if (!e.dataTransfer) return null;
  const id = e.dataTransfer.getData('application/x-clipboard-item') ||
    e.dataTransfer.getData('text/plain');
  // Validate it looks like a UUID
  if (id && /^[0-9a-f-]{36}$/i.test(id)) {
    return id;
  }
  return null;
};

const handleDragOver = (e: DragEvent, targetId: string) => {
  if (!isValidDragData(e)) return;
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = 'move';
  }
  dropTargetId.value = targetId;
};

const handleDragLeave = (e: DragEvent) => {
  // Only clear if leaving to outside
  const relatedTarget = e.relatedTarget as HTMLElement;
  if (!relatedTarget || !relatedTarget.closest('.tab-drop-zone, .history-tab')) {
    dropTargetId.value = null;
  }
};

const handleDrop = async (e: DragEvent, targetId: string, isHistory: boolean) => {
  e.preventDefault();
  dropTargetId.value = null;

  const itemId = getItemIdFromDrag(e);
  if (!itemId) return;

  if (isHistory) {
    await store.removeItemFromPinboard(itemId);
  } else {
    await store.addItemToPinboard(itemId, targetId);
  }
};

// Show create popover
const openCreatePopover = async () => {
  // Get button position for popover placement
  if (addBtnRef.value) {
    const rect = addBtnRef.value.getBoundingClientRect();
    createPopoverPosition.value = {
      x: rect.right - 240, // Align right edge with button
      y: rect.bottom + 8,
    };
  }
  showCreatePopover.value = true;
  newPinboardName.value = '';
  newPinboardIcon.value = '📌';
  await nextTick();
  createInputRef.value?.focus();
};

// Create pinboard
const createPinboard = async () => {
  const name = newPinboardName.value.trim();
  if (name) {
    const pinboard = await store.createPinboard(name, newPinboardIcon.value);
    if (pinboard) {
      store.setActivePinboard(pinboard.id);
    }
  }
  closeCreatePopover();
};

// Close create popover
const closeCreatePopover = () => {
  showCreatePopover.value = false;
  newPinboardName.value = '';
  newPinboardIcon.value = '📌';
};

// Handle create input keydown
const handleCreateKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && newPinboardName.value.trim()) {
    createPinboard();
  } else if (e.key === 'Escape') {
    closeCreatePopover();
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

// Open edit popover
const openEditPopover = async () => {
  if (!contextMenu.value.pinboard) return;
  editingPinboard.value = contextMenu.value.pinboard;
  editingName.value = contextMenu.value.pinboard.name;
  editingIcon.value = contextMenu.value.pinboard.icon || '📌';
  editPopoverPosition.value = { x: contextMenu.value.x, y: contextMenu.value.y };
  closeContextMenu();
  showEditPopover.value = true;
  await nextTick();
  editInputRef.value?.focus();
  editInputRef.value?.select();
};

// Save edit
const saveEdit = async () => {
  if (editingPinboard.value && editingName.value.trim()) {
    await store.updatePinboard(editingPinboard.value.id, editingName.value.trim(), editingIcon.value);
  }
  closeEditPopover();
};

// Close edit popover
const closeEditPopover = () => {
  showEditPopover.value = false;
  editingPinboard.value = null;
  editingName.value = '';
  editingIcon.value = '📌';
};

// Handle edit input keydown
const handleEditKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && editingName.value.trim()) {
    saveEdit();
  } else if (e.key === 'Escape') {
    closeEditPopover();
  }
};

// Delete pinboard
const deletePinboard = async () => {
  if (!contextMenu.value.pinboard) return;
  await store.deletePinboard(contextMenu.value.pinboard.id);
  closeContextMenu();
};

// Close popups when clicking outside
const handleClickOutside = () => {
  if (contextMenu.value.show) {
    closeContextMenu();
  }
};

</script>

<template>
  <div class="pinboard-tabs" :class="{ 'drag-mode': isDraggingItem }" @click="handleClickOutside">
    <!-- History Tab -->
    <button
      class="tab history-tab"
      data-drop-zone="history"
      :class="{
        active: activePinboardId === null,
        'drop-target': dropTargetId === 'history',
        'drop-ready': isDraggingItem && dropTargetId !== 'history'
      }"
      @click="handleTabClick(null)"
      @dragover="handleDragOver($event, 'history')"
      @dragleave="handleDragLeave"
      @drop="handleDrop($event, 'history', true)"
    >
      <span class="tab-icon">📋</span>
      <span class="tab-name">History</span>
    </button>

    <!-- Pinboard Tabs -->
    <div class="pinboard-tabs-list">
      <div
        v-for="pinboard in sortedPinboards"
        :key="pinboard.id"
        class="tab-drop-zone"
        :data-drop-zone="`pinboard-${pinboard.id}`"
        :class="{
          'drop-target': dropTargetId === pinboard.id,
          'drop-ready': isDraggingItem && dropTargetId !== pinboard.id
        }"
        @dragover="handleDragOver($event, pinboard.id)"
        @dragleave="handleDragLeave"
        @drop="handleDrop($event, pinboard.id, false)"
      >
        <button
          class="tab pinboard-tab"
          :class="{ active: activePinboardId === pinboard.id }"
          @click="handleTabClick(pinboard.id)"
          @contextmenu="openContextMenu($event, pinboard)"
        >
          <span class="tab-icon">{{ pinboard.icon || '📌' }}</span>
          <span class="tab-name">{{ pinboard.name }}</span>
        </button>
      </div>
    </div>

    <!-- Add New Pinboard Button -->
    <button
      ref="addBtnRef"
      class="tab add-tab"
      :class="{ active: showCreatePopover }"
      @click.stop="showCreatePopover ? closeCreatePopover() : openCreatePopover()"
      title="Create pinboard"
    >
      <span class="tab-icon">+</span>
    </button>

    <!-- Create Popover (teleported to body) -->
    <Teleport to="body">
      <Transition name="popover">
        <div
          v-if="showCreatePopover"
          class="popover create-popover"
          :style="{ top: `${createPopoverPosition.y}px`, left: `${createPopoverPosition.x}px` }"
          @click.stop
        >
          <div class="popover-arrow"></div>

          <!-- Icon Grid -->
          <div class="icon-grid">
            <button
              v-for="icon in availableIcons"
              :key="icon"
              class="icon-btn"
              :class="{ selected: newPinboardIcon === icon }"
              @click="newPinboardIcon = icon"
            >
              {{ icon }}
            </button>
          </div>

          <!-- Name Input -->
          <div class="input-row">
            <span class="input-icon">{{ newPinboardIcon }}</span>
            <input
              ref="createInputRef"
              v-model="newPinboardName"
              type="text"
              class="name-input"
              placeholder="Name..."
              @keydown="handleCreateKeydown"
            />
            <button
              class="create-btn"
              :disabled="!newPinboardName.trim()"
              @click="createPinboard"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </button>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Context Menu -->
    <Teleport to="body">
      <div
        v-if="contextMenu.show"
        class="context-menu"
        :style="{ top: `${contextMenu.y}px`, left: `${contextMenu.x}px` }"
        @click.stop
      >
        <button class="context-item" @click="openEditPopover">
          <span class="context-icon">✏️</span>
          Edit
        </button>
        <button class="context-item danger" @click="deletePinboard">
          <span class="context-icon">🗑️</span>
          Delete
        </button>
      </div>
    </Teleport>

    <!-- Edit Popover -->
    <Teleport to="body">
      <Transition name="popover">
        <div
          v-if="showEditPopover"
          class="popover edit-popover"
          :style="{ top: `${editPopoverPosition.y}px`, left: `${editPopoverPosition.x}px` }"
          @click.stop
        >
          <!-- Icon Grid -->
          <div class="icon-grid">
            <button
              v-for="icon in availableIcons"
              :key="icon"
              class="icon-btn"
              :class="{ selected: editingIcon === icon }"
              @click="editingIcon = icon"
            >
              {{ icon }}
            </button>
          </div>

          <!-- Name Input -->
          <div class="input-row">
            <span class="input-icon">{{ editingIcon }}</span>
            <input
              ref="editInputRef"
              v-model="editingName"
              type="text"
              class="name-input"
              placeholder="Name..."
              @keydown="handleEditKeydown"
            />
            <button
              class="create-btn"
              :disabled="!editingName.trim()"
              @click="saveEdit"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </button>
          </div>

          <button class="cancel-link" @click="closeEditPopover">Cancel</button>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<style scoped>
.pinboard-tabs {
  display: flex;
  align-items: center;
  gap: 4px;
  background: transparent;
  overflow-x: auto;
  flex-shrink: 0;
  padding: 4px;
  margin: -4px;
  border-radius: 10px;
  transition: background-color 0.2s ease;
}

/* When dragging a card, subtly highlight the tab area */
.pinboard-tabs.drag-mode {
  background: rgba(59, 130, 246, 0.04);
}

.pinboard-tabs::-webkit-scrollbar {
  height: 4px;
}

.pinboard-tabs::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.2);
  border-radius: 2px;
}

.pinboard-tabs-list {
  display: flex;
  gap: 4px;
}

.tab-drop-zone {
  border-radius: 8px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

/* Ready to receive drop - subtle pulsing glow */
.tab-drop-zone.drop-ready {
  animation: pulse-ready 1.5s ease-in-out infinite;
}

.tab-drop-zone.drop-ready .tab {
  background: rgba(59, 130, 246, 0.08);
  border: 1px dashed rgba(59, 130, 246, 0.4);
}

/* Active drop target - prominent glow */
.tab-drop-zone.drop-target {
  background: rgba(59, 130, 246, 0.15);
  box-shadow:
    0 0 0 2px #3b82f6,
    0 0 20px rgba(59, 130, 246, 0.4),
    0 4px 12px rgba(59, 130, 246, 0.3);
  transform: scale(1.08);
  animation: none;
}

.tab-drop-zone.drop-target .tab {
  background: rgba(59, 130, 246, 0.2);
  color: #1d4ed8;
  border: 1px solid transparent;
}

@keyframes pulse-ready {
  0%, 100% {
    box-shadow: 0 0 0 0 rgba(59, 130, 246, 0);
  }
  50% {
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
  }
}

.tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid transparent;
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
  font-size: 13px;
  color: #6b7280;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  white-space: nowrap;
}

.tab:hover {
  background: rgba(0, 0, 0, 0.06);
  color: #374151;
}

.tab.active {
  background: rgba(255, 255, 255, 0.7);
  color: #1f2937;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
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

.add-tab:hover,
.add-tab.active {
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
}

.add-tab .tab-icon {
  font-size: 16px;
  font-weight: 600;
}

/* Drop ready state for history tab - subtle pulsing */
.tab.drop-ready {
  animation: pulse-ready 1.5s ease-in-out infinite;
  background: rgba(59, 130, 246, 0.08);
  border: 1px dashed rgba(59, 130, 246, 0.4);
}

/* Drop target state for history tab */
.tab.drop-target {
  background: rgba(59, 130, 246, 0.2);
  box-shadow:
    0 0 0 2px #3b82f6,
    0 0 20px rgba(59, 130, 246, 0.4),
    0 4px 12px rgba(59, 130, 246, 0.3);
  color: #1d4ed8;
  transform: scale(1.08);
  animation: none;
  border: 1px solid transparent;
}

/* Popover styles */
.popover {
  position: fixed;
  background: white;
  border-radius: 12px;
  padding: 12px;
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.2);
  z-index: 10000;
  width: 240px;
}

.edit-popover {
  transform: translateX(-50%);
}

.popover-arrow {
  position: absolute;
  top: -6px;
  right: 14px;
  width: 12px;
  height: 12px;
  background: white;
  transform: rotate(45deg);
  box-shadow: -2px -2px 4px rgba(0, 0, 0, 0.05);
}

.icon-grid {
  display: grid;
  grid-template-columns: repeat(8, 1fr);
  gap: 2px;
  margin-bottom: 10px;
}

.icon-btn {
  width: 26px;
  height: 26px;
  border: 2px solid transparent;
  border-radius: 6px;
  background: #f3f4f6;
  cursor: pointer;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.1s;
  padding: 0;
}

.icon-btn:hover {
  background: #e5e7eb;
  transform: scale(1.15);
}

.icon-btn.selected {
  border-color: #3b82f6;
  background: #dbeafe;
}

.input-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  background: #f3f4f6;
  border-radius: 8px;
}

.input-icon {
  font-size: 16px;
  flex-shrink: 0;
}

.name-input {
  flex: 1;
  border: none;
  background: transparent;
  font-size: 13px;
  color: #1f2937;
  outline: none;
  min-width: 0;
}

.name-input::placeholder {
  color: #9ca3af;
}

.create-btn {
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 6px;
  background: #3b82f6;
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
  flex-shrink: 0;
  padding: 0;
}

.create-btn:hover:not(:disabled) {
  background: #2563eb;
}

.create-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.create-btn svg {
  width: 14px;
  height: 14px;
}

.cancel-link {
  display: block;
  width: 100%;
  margin-top: 8px;
  padding: 6px;
  border: none;
  background: transparent;
  color: #9ca3af;
  font-size: 12px;
  cursor: pointer;
  text-align: center;
}

.cancel-link:hover {
  color: #6b7280;
}

/* Context menu */
.context-menu {
  position: fixed;
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
  padding: 4px;
  min-width: 100px;
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

/* Popover animation */
.popover-enter-active,
.popover-leave-active {
  transition: opacity 0.15s, transform 0.15s;
}

.popover-enter-from,
.popover-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

.edit-popover.popover-enter-from,
.edit-popover.popover-leave-to {
  transform: translateX(-50%) translateY(-4px);
}
</style>
