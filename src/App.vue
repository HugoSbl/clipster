<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
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

// Apply theme to document
const applyTheme = (theme: string) => {
  const html = document.documentElement;

  if (theme === 'dark') {
    html.classList.add('dark');
    html.style.colorScheme = 'dark';
  } else if (theme === 'light') {
    html.classList.remove('dark');
    html.style.colorScheme = 'light';
  } else {
    // System: follow OS preference
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    html.classList.toggle('dark', prefersDark);
    html.style.colorScheme = prefersDark ? 'dark' : 'light';
  }
};

// Listen for OS theme changes (for "system" mode)
const systemMediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
const handleSystemThemeChange = () => {
  if (settingsStore.theme === 'system') {
    applyTheme('system');
  }
};
systemMediaQuery.addEventListener('change', handleSystemThemeChange);

// Watch theme changes
watch(
  () => settingsStore.theme,
  (theme) => {
    applyTheme(theme);
  }
);

// Component refs
const searchBarRef = ref<InstanceType<typeof SearchBar> | null>(null);
const timelineRef = ref<InstanceType<typeof Timeline> | null>(null);

// Animation state
const isHiding = ref(false);

let unlistenFn: UnlistenFn | null = null;
let unlistenSettings: UnlistenFn | null = null;
let unlistenBlur: UnlistenFn | null = null;

// Store preventDefault function for cleanup
let preventDefaults: ((e: Event) => void) | null = null;

// Hide window with slide-down animation
const hideWithAnimation = async () => {
  if (isHiding.value) return;
  isHiding.value = true;

  // Wait for animation to complete (300ms)
  setTimeout(async () => {
    await invoke('hide_window');
    isHiding.value = false;
  }, 280);
};

onMounted(async () => {
  // Prevent default browser behavior for drag and drop
  // This prevents Tauri from opening dropped files in the window
  preventDefaults = (e: Event) => {
    e.preventDefault();
    e.stopPropagation();
  };

  // Block all drop events globally
  document.addEventListener('dragover', preventDefaults, false);
  document.addEventListener('dragenter', preventDefaults, false);
  document.addEventListener('dragleave', preventDefaults, false);
  document.addEventListener('drop', preventDefaults, false);

  // Fetch pinboards, settings, and initial history
  await Promise.all([
    pinboardStore.fetchPinboards(),
    settingsStore.fetchSettings(),
    clipboardStore.fetchHistory(),
  ]);

  // Apply persisted theme
  applyTheme(settingsStore.theme);

  // Set up real-time event listener
  unlistenFn = await clipboardStore.setupEventListener();

  // Listen for tray settings event
  unlistenSettings = await listen('open-settings', () => {
    settingsStore.openSettings();
  });

  // Listen for window blur (focus lost) - hide with animation
  const appWindow = getCurrentWindow();
  unlistenBlur = await appWindow.onFocusChanged(({ payload: focused }) => {
    if (!focused && !settingsStore.showModal) {
      hideWithAnimation();
    }
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
  if (unlistenBlur) {
    unlistenBlur();
  }

  // Remove system theme listener
  systemMediaQuery.removeEventListener('change', handleSystemThemeChange);

  // Remove global drop prevention listeners
  if (preventDefaults) {
    document.removeEventListener('dragover', preventDefaults, false);
    document.removeEventListener('dragenter', preventDefaults, false);
    document.removeEventListener('dragleave', preventDefaults, false);
    document.removeEventListener('drop', preventDefaults, false);
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
});
</script>

<template>
  <main class="app-container" :class="{ 'slide-down': isHiding }">
    <div class="top-bar">
      <PinboardTabs />
      <SearchBar
        ref="searchBarRef"
        :model-value="clipboardStore.searchQuery"
        @search="handleSearch"
      />
    </div>
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
  background-color: transparent;

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
  background-color: transparent;
}

.app-container {
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
  /* Mesh gradient background for glassmorphism */
  background:
    radial-gradient(ellipse at 20% 50%, rgba(120, 119, 198, 0.15) 0%, transparent 50%),
    radial-gradient(ellipse at 80% 20%, rgba(255, 119, 168, 0.12) 0%, transparent 50%),
    radial-gradient(ellipse at 50% 80%, rgba(99, 179, 237, 0.12) 0%, transparent 50%),
    rgba(250, 248, 245, 0.92);
  backdrop-filter: blur(24px) saturate(180%);
  -webkit-backdrop-filter: blur(24px) saturate(180%);
  border-top: 1px solid rgba(0, 0, 0, 0.08);
  transform: translateY(0);
  transition: transform 0.28s cubic-bezier(0.4, 0, 0.6, 1);
  overflow: hidden;
}

html.dark {
  color: rgba(255, 255, 255, 0.87);
}

html.dark .app-container {
  background:
    radial-gradient(ellipse at 20% 50%, rgba(120, 119, 198, 0.12) 0%, transparent 50%),
    radial-gradient(ellipse at 80% 20%, rgba(255, 119, 168, 0.08) 0%, transparent 50%),
    radial-gradient(ellipse at 50% 80%, rgba(99, 179, 237, 0.08) 0%, transparent 50%),
    rgba(15, 15, 20, 0.94);
  border-top-color: rgba(255, 255, 255, 0.06);
}

html.dark .top-bar {
  border-bottom-color: rgba(255, 255, 255, 0.06);
}

.top-bar {
  padding: 8px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.06);
  flex-shrink: 0;
}

.app-container.slide-down {
  transform: translateY(100%);
}

/* Global button reset */
button {
  font-family: inherit;
  cursor: pointer;
}

/* Base text color */
:root {
  color: #374151;
}
</style>
