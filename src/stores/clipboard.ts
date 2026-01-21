import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ClipboardItem, ClipboardChangedPayload } from '@/types';

interface ClipboardState {
  items: ClipboardItem[];
  loading: boolean;
  searchQuery: string;
  totalCount: number;
  error: string | null;
  activePinboardId: string | null; // null = show all history
}

export const useClipboardStore = defineStore('clipboard', {
  state: (): ClipboardState => ({
    items: [],
    loading: false,
    searchQuery: '',
    totalCount: 0,
    error: null,
    activePinboardId: null,
  }),

  getters: {
    /**
     * Filter items by search query (client-side for instant results)
     */
    filteredItems(state): ClipboardItem[] {
      if (!state.searchQuery.trim()) {
        return state.items;
      }
      const query = state.searchQuery.toLowerCase();
      return state.items.filter((item) => {
        // Search in text content
        if (item.content_type === 'text' && item.content_text) {
          return item.content_text.toLowerCase().includes(query);
        }
        // Search in file paths
        if (item.content_type === 'files' && item.content_text) {
          return item.content_text.toLowerCase().includes(query);
        }
        // Images don't have searchable text content
        return false;
      });
    },

    /**
     * Check if there are any items
     */
    hasItems(state): boolean {
      return state.items.length > 0;
    },

    /**
     * Get text items only
     */
    textItems(state): ClipboardItem[] {
      return state.items.filter((item) => item.content_type === 'text');
    },

    /**
     * Get image items only
     */
    imageItems(state): ClipboardItem[] {
      return state.items.filter((item) => item.content_type === 'image');
    },

    /**
     * Get file items only
     */
    fileItems(state): ClipboardItem[] {
      return state.items.filter((item) => item.content_type === 'files');
    },
  },

  actions: {
    /**
     * Fetch clipboard history from backend
     */
    async fetchHistory(limit = 100, offset = 0): Promise<void> {
      this.loading = true;
      this.error = null;

      try {
        const items = await invoke<ClipboardItem[]>('get_clipboard_history', {
          limit,
          offset,
        });
        this.items = items;
        this.totalCount = await invoke<number>('get_clipboard_count');
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to fetch clipboard history:', e);
      } finally {
        this.loading = false;
      }
    },

    /**
     * Search clipboard items (server-side search)
     */
    async search(query: string, limit = 50): Promise<void> {
      this.searchQuery = query;

      if (!query.trim()) {
        await this.fetchHistory();
        return;
      }

      this.loading = true;
      this.error = null;

      try {
        const items = await invoke<ClipboardItem[]>('search_clipboard', {
          query,
          limit,
        });
        this.items = items;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to search clipboard:', e);
      } finally {
        this.loading = false;
      }
    },

    /**
     * Delete a clipboard item
     */
    async deleteItem(id: string): Promise<boolean> {
      try {
        const success = await invoke<boolean>('delete_clipboard_item', { id });
        if (success) {
          this.items = this.items.filter((item) => item.id !== id);
          this.totalCount = Math.max(0, this.totalCount - 1);
        }
        return success;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to delete clipboard item:', e);
        return false;
      }
    },

    /**
     * Copy an item back to system clipboard
     */
    async copyToClipboard(id: string): Promise<boolean> {
      try {
        await invoke('copy_to_clipboard', { id });
        return true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to copy to clipboard:', e);
        return false;
      }
    },

    /**
     * Toggle favorite status
     */
    async toggleFavorite(id: string): Promise<boolean> {
      try {
        await invoke<boolean>('toggle_favorite', { id });
        // Update local state
        const item = this.items.find((item) => item.id === id);
        if (item) {
          item.is_favorite = !item.is_favorite;
        }
        return true;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to toggle favorite:', e);
        return false;
      }
    },

    /**
     * Clear all clipboard history (except favorites and pinned)
     */
    async clearHistory(): Promise<number> {
      try {
        const deletedCount = await invoke<number>('clear_clipboard_history');
        await this.fetchHistory();
        return deletedCount;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to clear clipboard history:', e);
        return 0;
      }
    },

    /**
     * Add a new item to the beginning of the list
     * Called when clipboard-changed event is received
     */
    addItem(item: ClipboardItem): void {
      // Remove any existing item with same ID (shouldn't happen, but be safe)
      this.items = this.items.filter((existing) => existing.id !== item.id);
      // Add to beginning
      this.items.unshift(item);
      this.totalCount += 1;
    },

    /**
     * Set up event listener for clipboard changes
     * Returns unlisten function for cleanup
     */
    async setupEventListener(): Promise<UnlistenFn> {
      return await listen<ClipboardChangedPayload>('clipboard-changed', (event) => {
        console.log('Clipboard changed:', event.payload.item);
        this.addItem(event.payload.item);
      });
    },

    /**
     * Clear search and show all items
     */
    clearSearch(): void {
      this.searchQuery = '';
    },

    /**
     * Clear error state
     */
    clearError(): void {
      this.error = null;
    },

    /**
     * Set active pinboard filter
     */
    setActivePinboard(pinboardId: string | null): void {
      this.activePinboardId = pinboardId;
    },

    /**
     * Fetch items for a specific pinboard
     */
    async fetchPinboardItems(pinboardId: string, limit = 100): Promise<void> {
      this.loading = true;
      this.error = null;

      try {
        const items = await invoke<ClipboardItem[]>('get_pinboard_items', {
          pinboardId,
          limit,
        });
        this.items = items;
        this.totalCount = items.length;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to fetch pinboard items:', e);
      } finally {
        this.loading = false;
      }
    },

    /**
     * Refresh items based on current active pinboard
     */
    async refreshItems(): Promise<void> {
      if (this.activePinboardId) {
        await this.fetchPinboardItems(this.activePinboardId);
      } else {
        await this.fetchHistory();
      }
    },

    /**
     * Get full image data as base64
     */
    async getImageData(id: string): Promise<string | null> {
      try {
        const base64 = await invoke<string>('get_image_data', { id });
        return base64;
      } catch (e) {
        this.error = e instanceof Error ? e.message : String(e);
        console.error('Failed to get image data:', e);
        return null;
      }
    },

    /**
     * Update item's pinboard assignment locally
     * Called after drag-and-drop operations
     */
    updateItemPinboard(itemId: string, pinboardId: string | null): void {
      const item = this.items.find((i) => i.id === itemId);
      if (item) {
        item.pinboard_id = pinboardId;
      }
    },

    /**
     * Remove item from the current list (used when item is moved to a pinboard)
     */
    removeItemFromList(itemId: string): void {
      const index = this.items.findIndex((i) => i.id === itemId);
      if (index !== -1) {
        this.items.splice(index, 1);
        this.totalCount = Math.max(0, this.totalCount - 1);
      }
    },
  },
});
