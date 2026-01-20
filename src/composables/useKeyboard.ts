import { onMounted, onUnmounted, ref, type Ref } from 'vue';

export interface KeyboardOptions {
  /** Callback for navigating left */
  onNavigateLeft?: () => void;
  /** Callback for navigating right */
  onNavigateRight?: () => void;
  /** Callback for selecting/copying current item */
  onSelect?: () => void;
  /** Callback for deleting current item */
  onDelete?: () => void;
  /** Callback for focusing search */
  onFocusSearch?: () => void;
  /** Callback for escape key */
  onEscape?: () => void;
  /** Callback for toggling favorite */
  onToggleFavorite?: () => void;
  /** Ref to check if an input is focused */
  isInputFocused?: Ref<boolean>;
}

/**
 * Composable for global keyboard navigation
 * Handles arrow keys, Enter, Delete, /, Escape, and F keys
 */
export function useKeyboard(options: KeyboardOptions) {
  const isEnabled = ref(true);

  const handleKeyDown = (e: KeyboardEvent) => {
    // Skip if disabled
    if (!isEnabled.value) return;

    // Skip if typing in an input field (except for Escape)
    const target = e.target as HTMLElement;
    const isInInput =
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.isContentEditable;

    // Allow Escape even in input fields
    if (e.key === 'Escape') {
      options.onEscape?.();
      // If in input, blur it
      if (isInInput) {
        target.blur();
      }
      return;
    }

    // Skip other shortcuts if typing
    if (isInInput) return;

    switch (e.key) {
      case 'ArrowLeft':
        e.preventDefault();
        options.onNavigateLeft?.();
        break;

      case 'ArrowRight':
        e.preventDefault();
        options.onNavigateRight?.();
        break;

      case 'Enter':
        e.preventDefault();
        options.onSelect?.();
        break;

      case 'Delete':
      case 'Backspace':
        e.preventDefault();
        options.onDelete?.();
        break;

      case '/':
        e.preventDefault();
        options.onFocusSearch?.();
        break;

      case 'f':
        // F key to toggle favorite (not Ctrl+F which is browser find)
        if (!e.ctrlKey && !e.metaKey) {
          e.preventDefault();
          options.onToggleFavorite?.();
        }
        break;
    }
  };

  onMounted(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });

  return {
    /** Enable/disable keyboard handling */
    isEnabled,
    /** Manually disable temporarily (e.g., when modal is open) */
    disable: () => {
      isEnabled.value = false;
    },
    /** Re-enable keyboard handling */
    enable: () => {
      isEnabled.value = true;
    },
  };
}

/**
 * Keyboard shortcuts reference:
 *
 * | Key        | Action                          |
 * |------------|--------------------------------|
 * | ←          | Select previous item           |
 * | →          | Select next item               |
 * | Enter      | Copy selected (or open image)  |
 * | Delete     | Delete selected item           |
 * | Backspace  | Delete selected item           |
 * | /          | Focus search bar               |
 * | Escape     | Clear selection / close modal  |
 * | F          | Toggle favorite                |
 */
