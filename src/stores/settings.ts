import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

export interface AppSettings {
  shortcut: string;
  history_limit: number;
  start_hidden: boolean;
}

interface SettingsState {
  settings: AppSettings;
  loading: boolean;
  error: string | null;
  showModal: boolean;
}

const defaultSettings: AppSettings = {
  shortcut: 'Ctrl+Shift+V',
  history_limit: 500,
  start_hidden: false,
};

export const useSettingsStore = defineStore('settings', {
  state: (): SettingsState => ({
    settings: { ...defaultSettings },
    loading: false,
    error: null,
    showModal: false,
  }),

  getters: {
    shortcut: (state) => state.settings.shortcut,
    historyLimit: (state) => state.settings.history_limit,
    startHidden: (state) => state.settings.start_hidden,
  },

  actions: {
    /**
     * Fetch all settings from backend
     */
    async fetchSettings(): Promise<void> {
      this.loading = true;
      this.error = null;

      try {
        const settings = await invoke<AppSettings>('get_settings');
        this.settings = settings;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to fetch settings:', e);
      } finally {
        this.loading = false;
      }
    },

    /**
     * Update a single setting
     */
    async updateSetting(key: keyof AppSettings, value: string | number | boolean): Promise<boolean> {
      try {
        await invoke('update_setting', {
          key,
          value: String(value),
        });

        // Update local state
        if (key === 'shortcut') {
          this.settings.shortcut = value as string;
        } else if (key === 'history_limit') {
          this.settings.history_limit = value as number;
        } else if (key === 'start_hidden') {
          this.settings.start_hidden = value as boolean;
        }

        return true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to update setting:', e);
        return false;
      }
    },

    /**
     * Set history limit with pruning
     */
    async setHistoryLimit(limit: number): Promise<boolean> {
      try {
        await invoke('set_history_limit', { limit });
        this.settings.history_limit = limit;
        return true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to set history limit:', e);
        return false;
      }
    },

    /**
     * Show settings modal
     */
    openSettings(): void {
      this.showModal = true;
    },

    /**
     * Hide settings modal
     */
    closeSettings(): void {
      this.showModal = false;
    },

    /**
     * Reset to default settings
     */
    async resetToDefaults(): Promise<boolean> {
      try {
        await this.updateSetting('shortcut', defaultSettings.shortcut);
        await this.setHistoryLimit(defaultSettings.history_limit);
        await this.updateSetting('start_hidden', defaultSettings.start_hidden);
        return true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to reset settings:', e);
        return false;
      }
    },

    /**
     * Clear error state
     */
    clearError(): void {
      this.error = null;
    },
  },
});
