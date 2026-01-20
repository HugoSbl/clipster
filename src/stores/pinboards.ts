import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { Pinboard } from '@/types';
import { useClipboardStore } from './clipboard';

interface PinboardState {
  pinboards: Pinboard[];
  activePinboardId: string | null; // null = History view
  loading: boolean;
  error: string | null;
}

export const usePinboardStore = defineStore('pinboards', {
  state: (): PinboardState => ({
    pinboards: [],
    activePinboardId: null,
    loading: false,
    error: null,
  }),

  getters: {
    /**
     * Get sorted pinboards by position
     */
    sortedPinboards(state): Pinboard[] {
      return [...state.pinboards].sort((a, b) => a.position - b.position);
    },

    /**
     * Check if viewing history (no pinboard selected)
     */
    isHistoryView(state): boolean {
      return state.activePinboardId === null;
    },

    /**
     * Get active pinboard object
     */
    activePinboard(state): Pinboard | null {
      if (!state.activePinboardId) return null;
      return state.pinboards.find((p) => p.id === state.activePinboardId) || null;
    },
  },

  actions: {
    /**
     * Fetch all pinboards from backend
     */
    async fetchPinboards(): Promise<void> {
      this.loading = true;
      this.error = null;

      try {
        const pinboards = await invoke<Pinboard[]>('get_pinboards');
        this.pinboards = pinboards;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to fetch pinboards:', e);
      } finally {
        this.loading = false;
      }
    },

    /**
     * Create a new pinboard
     */
    async createPinboard(name: string, icon?: string): Promise<Pinboard | null> {
      try {
        const pinboard = await invoke<Pinboard>('create_pinboard', {
          name,
          icon: icon || null,
        });
        this.pinboards.push(pinboard);
        return pinboard;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to create pinboard:', e);
        return null;
      }
    },

    /**
     * Update a pinboard's name and/or icon
     */
    async updatePinboard(id: string, name: string, icon?: string): Promise<boolean> {
      try {
        await invoke<boolean>('update_pinboard', {
          id,
          name,
          icon: icon || null,
        });
        // Update local state
        const pinboard = this.pinboards.find((p) => p.id === id);
        if (pinboard) {
          pinboard.name = name;
          pinboard.icon = icon || null;
        }
        return true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to update pinboard:', e);
        return false;
      }
    },

    /**
     * Delete a pinboard
     */
    async deletePinboard(id: string): Promise<boolean> {
      try {
        await invoke<boolean>('delete_pinboard', { id });
        this.pinboards = this.pinboards.filter((p) => p.id !== id);
        // If deleted pinboard was active, switch to history
        if (this.activePinboardId === id) {
          this.activePinboardId = null;
        }
        return true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to delete pinboard:', e);
        return false;
      }
    },

    /**
     * Reorder pinboards
     */
    async reorderPinboards(pinboardIds: string[]): Promise<boolean> {
      try {
        await invoke('reorder_pinboards', { pinboardIds });
        // Update local positions
        pinboardIds.forEach((id, index) => {
          const pinboard = this.pinboards.find((p) => p.id === id);
          if (pinboard) {
            pinboard.position = index;
          }
        });
        return true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to reorder pinboards:', e);
        return false;
      }
    },

    /**
     * Set active pinboard (null for history)
     */
    setActivePinboard(id: string | null): void {
      this.activePinboardId = id;
    },

    /**
     * Add item to a pinboard
     */
    async addItemToPinboard(itemId: string, pinboardId: string): Promise<boolean> {
      try {
        await invoke<boolean>('add_item_to_pinboard', { itemId, pinboardId });
        // Update local clipboard store
        const clipboardStore = useClipboardStore();
        clipboardStore.updateItemPinboard(itemId, pinboardId);
        // Refresh if viewing the target pinboard
        if (this.activePinboardId === pinboardId) {
          await clipboardStore.fetchPinboardItems(pinboardId);
        }
        return true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to add item to pinboard:', e);
        return false;
      }
    },

    /**
     * Remove item from its pinboard
     */
    async removeItemFromPinboard(itemId: string): Promise<boolean> {
      try {
        await invoke<boolean>('remove_item_from_pinboard', { itemId });
        // Update local clipboard store
        const clipboardStore = useClipboardStore();
        clipboardStore.updateItemPinboard(itemId, null);
        // Refresh if viewing a pinboard (item will disappear)
        if (this.activePinboardId) {
          await clipboardStore.fetchPinboardItems(this.activePinboardId);
        }
        return true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to remove item from pinboard:', e);
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
