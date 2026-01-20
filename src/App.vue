<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useClipboardStore } from '@/stores/clipboard';
import { usePinboardStore } from '@/stores/pinboards';
import { useSettingsStore } from '@/stores/settings';
import { useKeyboard } from '@/composables/useKeyboard';
import PinboardTabs from './components/PinboardTabs.vue';
import SearchBar from './components/SearchBar.vue';
import Timeline from './components/Timeline.vue';
import Settings from './components/Settings.vue';
import type { UnlistenFn } from '@tauri-apps/api/event';

const clipboardStore = useClipboardStore();
const pinboardStore = usePinboardStore();
const settingsStore = useSettingsStore();

// Component refs
const searchBarRef = ref<InstanceType<typeof SearchBar> | null>(null);
const timelineRef = ref<InstanceType<typeof Timeline> | null>(null);

let unlistenFn: UnlistenFn | null = null;
let unlistenSettings: UnlistenFn | null = null;

onMounted(async () => {
  // Fetch pinboards, settings, and initial history
  await Promise.all([
    pinboardStore.fetchPinboards(),
    settingsStore.fetchSettings(),
    clipboardStore.fetchHistory(),
  ]);

  // Set up real-time event listener
  unlistenFn = await clipboardStore.setupEventListener();

  // Listen for tray settings event
  unlistenSettings = await listen('open-settings', () => {
    settingsStore.openSettings();
  });
});

// Watch for pinboard changes and refresh items
watch(
  () => pinboardStore.activePinboardId,
  async (pinboardId) => {
    clipboardStore.setActivePinboard(pinboardId);
    if (pinboardId) {
      await clipboardStore.fetchPinboardItems(pinboardId);
    } else {
      await clipboardStore.fetchHistory();
    }
  }
);

onUnmounted(() => {
  // Clean up event listeners
  if (unlistenFn) {
    unlistenFn();
  }
  if (unlistenSettings) {
    unlistenSettings();
  }
});

// Handle search
const handleSearch = async (query: string) => {
  await clipboardStore.search(query);
};

// Set up global keyboard handling
useKeyboard({
  onNavigateLeft: () => {
    timelineRef.value?.navigateLeft();
  },
  onNavigateRight: () => {
    timelineRef.value?.navigateRight();
  },
  onSelect: () => {
    timelineRef.value?.selectCurrent();
  },
  onDelete: () => {
    timelineRef.value?.deleteCurrent();
  },
  onFocusSearch: () => {
    searchBarRef.value?.focus();
  },
  onEscape: () => {
    timelineRef.value?.clearSelection();
  },
  onToggleFavorite: () => {
    timelineRef.value?.toggleFavoriteCurrent();
  },
});
</script>

<template>
  <main class="app-container">
    <PinboardTabs />
    <SearchBar
      ref="searchBarRef"
      :model-value="clipboardStore.searchQuery"
      @search="handleSearch"
    />
    <Timeline ref="timelineRef" />
    <Settings />
  </main>
</template>

<style>
:root {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial,
    sans-serif;
  font-size: 14px;
  line-height: 1.5;
  font-weight: 400;

  color: #1f2937;
  background-color: #f9fafb;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

html,
body,
#app {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

.app-container {
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
}

/* Global button reset */
button {
  font-family: inherit;
  cursor: pointer;
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
  :root {
    color: #f3f4f6;
    background-color: #111827;
  }
}
</style>
