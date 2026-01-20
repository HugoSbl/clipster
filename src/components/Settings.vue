<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import { useClipboardStore } from '@/stores/clipboard';

const settingsStore = useSettingsStore();
const clipboardStore = useClipboardStore();

// Local state for form inputs
const historyLimit = ref(500);
const startHidden = ref(false);

// Computed
const isOpen = computed(() => settingsStore.showModal);

// Initialize local state when modal opens
onMounted(async () => {
  await settingsStore.fetchSettings();
  historyLimit.value = settingsStore.historyLimit;
  startHidden.value = settingsStore.startHidden;
});

// Close modal
const close = () => {
  settingsStore.closeSettings();
};

// Save history limit
const saveHistoryLimit = async () => {
  await settingsStore.setHistoryLimit(historyLimit.value);
};

// Toggle start hidden
const toggleStartHidden = async () => {
  startHidden.value = !startHidden.value;
  await settingsStore.updateSetting('start_hidden', startHidden.value);
};

// Clear history
const clearHistory = async () => {
  if (confirm('Clear all clipboard history? This cannot be undone. Favorites and pinned items will be kept.')) {
    const deleted = await clipboardStore.clearHistory();
    alert(`Cleared ${deleted} items from history.`);
  }
};

// Reset to defaults
const resetDefaults = async () => {
  if (confirm('Reset all settings to defaults?')) {
    await settingsStore.resetToDefaults();
    historyLimit.value = settingsStore.historyLimit;
    startHidden.value = settingsStore.startHidden;
  }
};

// Handle click outside
const handleOverlayClick = (e: MouseEvent) => {
  if ((e.target as HTMLElement).classList.contains('settings-overlay')) {
    close();
  }
};

// Handle escape key
const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    close();
  }
};
</script>

<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="settings-overlay"
      @click="handleOverlayClick"
      @keydown="handleKeydown"
      tabindex="0"
    >
      <div class="settings-modal">
        <!-- Header -->
        <div class="settings-header">
          <h2>Settings</h2>
          <button class="close-btn" @click="close" title="Close">×</button>
        </div>

        <!-- Content -->
        <div class="settings-content">
          <!-- General Section -->
          <section class="settings-section">
            <h3>General</h3>

            <div class="setting-item">
              <label for="history-limit">History Limit</label>
              <div class="setting-control">
                <input
                  id="history-limit"
                  type="range"
                  min="100"
                  max="1000"
                  step="50"
                  v-model.number="historyLimit"
                  @change="saveHistoryLimit"
                />
                <span class="setting-value">{{ historyLimit }}</span>
              </div>
              <p class="setting-description">Maximum number of items to keep in history</p>
            </div>

            <div class="setting-item">
              <label>Start Hidden</label>
              <div class="setting-control">
                <button
                  class="toggle-btn"
                  :class="{ active: startHidden }"
                  @click="toggleStartHidden"
                >
                  {{ startHidden ? 'On' : 'Off' }}
                </button>
              </div>
              <p class="setting-description">Start the app minimized to system tray</p>
            </div>
          </section>

          <!-- Shortcut Section -->
          <section class="settings-section">
            <h3>Keyboard Shortcut</h3>

            <div class="setting-item">
              <label>Toggle Window</label>
              <div class="setting-control">
                <kbd class="shortcut-display">{{ settingsStore.shortcut }}</kbd>
              </div>
              <p class="setting-description">Global shortcut to show/hide the clipboard manager</p>
            </div>
          </section>

          <!-- Data Section -->
          <section class="settings-section">
            <h3>Data</h3>

            <div class="setting-item">
              <label>Clear History</label>
              <div class="setting-control">
                <button class="danger-btn" @click="clearHistory">
                  Clear All History
                </button>
              </div>
              <p class="setting-description">Remove all items except favorites and pinned</p>
            </div>
          </section>
        </div>

        <!-- Footer -->
        <div class="settings-footer">
          <button class="secondary-btn" @click="resetDefaults">Reset to Defaults</button>
          <button class="primary-btn" @click="close">Done</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

.settings-modal {
  background: white;
  border-radius: 12px;
  width: 90%;
  max-width: 480px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #e5e7eb;
}

.settings-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #111827;
}

.close-btn {
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
  font-size: 20px;
  color: #6b7280;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: #f3f4f6;
  color: #111827;
}

.settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
}

.settings-section {
  margin-bottom: 24px;
}

.settings-section:last-child {
  margin-bottom: 0;
}

.settings-section h3 {
  margin: 0 0 12px;
  font-size: 14px;
  font-weight: 600;
  color: #374151;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.setting-item {
  margin-bottom: 16px;
}

.setting-item:last-child {
  margin-bottom: 0;
}

.setting-item label {
  display: block;
  font-size: 14px;
  font-weight: 500;
  color: #111827;
  margin-bottom: 6px;
}

.setting-control {
  display: flex;
  align-items: center;
  gap: 12px;
}

.setting-value {
  min-width: 40px;
  font-size: 14px;
  font-weight: 500;
  color: #3b82f6;
}

.setting-description {
  margin: 4px 0 0;
  font-size: 12px;
  color: #6b7280;
}

/* Range input */
input[type="range"] {
  flex: 1;
  height: 6px;
  -webkit-appearance: none;
  background: #e5e7eb;
  border-radius: 3px;
  outline: none;
}

input[type="range"]::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #3b82f6;
  cursor: pointer;
  border: 2px solid white;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

/* Toggle button */
.toggle-btn {
  padding: 6px 16px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  background: #f9fafb;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  color: #6b7280;
  transition: all 0.15s;
}

.toggle-btn.active {
  background: #3b82f6;
  border-color: #3b82f6;
  color: white;
}

/* Shortcut display */
.shortcut-display {
  display: inline-block;
  padding: 6px 12px;
  background: #f3f4f6;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-family: monospace;
  font-size: 13px;
  color: #374151;
}

/* Buttons */
.danger-btn {
  padding: 8px 16px;
  border: 1px solid #fecaca;
  border-radius: 6px;
  background: #fef2f2;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  color: #dc2626;
  transition: all 0.15s;
}

.danger-btn:hover {
  background: #fee2e2;
  border-color: #fca5a5;
}

.settings-footer {
  display: flex;
  justify-content: space-between;
  padding: 16px 20px;
  border-top: 1px solid #e5e7eb;
}

.secondary-btn {
  padding: 8px 16px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  background: white;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  color: #374151;
  transition: all 0.15s;
}

.secondary-btn:hover {
  background: #f9fafb;
}

.primary-btn {
  padding: 8px 20px;
  border: none;
  border-radius: 6px;
  background: #3b82f6;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  color: white;
  transition: all 0.15s;
}

.primary-btn:hover {
  background: #2563eb;
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
  .settings-modal {
    background: #1f2937;
  }

  .settings-header {
    border-bottom-color: #374151;
  }

  .settings-header h2 {
    color: #f9fafb;
  }

  .close-btn {
    color: #9ca3af;
  }

  .close-btn:hover {
    background: #374151;
    color: #f3f4f6;
  }

  .settings-section h3 {
    color: #9ca3af;
  }

  .setting-item label {
    color: #f3f4f6;
  }

  .setting-description {
    color: #9ca3af;
  }

  input[type="range"] {
    background: #374151;
  }

  .toggle-btn {
    background: #374151;
    border-color: #4b5563;
    color: #9ca3af;
  }

  .toggle-btn.active {
    background: #3b82f6;
    border-color: #3b82f6;
    color: white;
  }

  .shortcut-display {
    background: #374151;
    border-color: #4b5563;
    color: #e5e7eb;
  }

  .danger-btn {
    background: #450a0a;
    border-color: #991b1b;
    color: #fca5a5;
  }

  .danger-btn:hover {
    background: #7f1d1d;
  }

  .settings-footer {
    border-top-color: #374151;
  }

  .secondary-btn {
    background: #374151;
    border-color: #4b5563;
    color: #e5e7eb;
  }

  .secondary-btn:hover {
    background: #4b5563;
  }
}
</style>
