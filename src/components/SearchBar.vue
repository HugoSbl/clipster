<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue';

const props = defineProps<{
  modelValue?: string;
  placeholder?: string;
  debounceMs?: number;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  search: [query: string];
}>();

// Input element ref
const inputRef = ref<HTMLInputElement | null>(null);

// Local input value for immediate UI feedback
const inputValue = ref(props.modelValue || '');

// Expose focus method
const focus = () => {
  inputRef.value?.focus();
};

defineExpose({ focus });

// Debounce timer
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

// Watch for external value changes
watch(
  () => props.modelValue,
  (newValue) => {
    if (newValue !== undefined && newValue !== inputValue.value) {
      inputValue.value = newValue;
    }
  }
);

// Handle input changes with debounce
const handleInput = (event: Event) => {
  const target = event.target as HTMLInputElement;
  inputValue.value = target.value;

  // Clear existing timer
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }

  // Set new debounce timer
  const delay = props.debounceMs ?? 300;
  debounceTimer = setTimeout(() => {
    emit('update:modelValue', inputValue.value);
    emit('search', inputValue.value);
  }, delay);
};

// Clear search
const clearSearch = () => {
  inputValue.value = '';
  emit('update:modelValue', '');
  emit('search', '');

  // Clear any pending debounce
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
};

// Handle keyboard shortcuts
const handleKeyDown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    clearSearch();
    (event.target as HTMLInputElement).blur();
  }
};

// Cleanup
onUnmounted(() => {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
});
</script>

<template>
  <div class="search-bar">
    <div class="search-input-wrapper">
      <!-- Search Icon -->
      <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8" />
        <path d="M21 21l-4.35-4.35" />
      </svg>

      <!-- Input -->
      <input
        ref="inputRef"
        type="text"
        :value="inputValue"
        :placeholder="placeholder || 'Search clipboard history... (press / to focus)'"
        class="search-input"
        @input="handleInput"
        @keydown="handleKeyDown"
      />

      <!-- Clear Button -->
      <button
        v-if="inputValue"
        class="clear-btn"
        @click="clearSearch"
        title="Clear search (Esc)"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <!-- Search hint -->
    <div v-if="inputValue" class="search-hint">
      Press <kbd>Esc</kbd> to clear
    </div>
  </div>
</template>

<style scoped>
.search-bar {
  padding: 12px 16px;
  border-bottom: 1px solid #e5e7eb;
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  width: 18px;
  height: 18px;
  color: #9ca3af;
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 40px;
  padding: 0 40px 0 40px;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  font-size: 14px;
  color: #1f2937;
  background: #f9fafb;
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s, background-color 0.15s;
}

.search-input::placeholder {
  color: #9ca3af;
}

.search-input:hover {
  border-color: #d1d5db;
}

.search-input:focus {
  border-color: #3b82f6;
  background: white;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.clear-btn {
  position: absolute;
  right: 8px;
  width: 28px;
  height: 28px;
  padding: 0;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: #9ca3af;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s, color 0.15s;
}

.clear-btn:hover {
  background: #f3f4f6;
  color: #6b7280;
}

.clear-btn svg {
  width: 16px;
  height: 16px;
}

.search-hint {
  margin-top: 6px;
  font-size: 11px;
  color: #9ca3af;
  text-align: right;
}

.search-hint kbd {
  display: inline-block;
  padding: 2px 6px;
  font-size: 10px;
  font-family: inherit;
  background: #f3f4f6;
  border: 1px solid #e5e7eb;
  border-radius: 4px;
  box-shadow: 0 1px 1px rgba(0, 0, 0, 0.05);
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
  .search-bar {
    border-bottom-color: #374151;
  }

  .search-input {
    border-color: #374151;
    background: #1f2937;
    color: #f3f4f6;
  }

  .search-input:hover {
    border-color: #4b5563;
  }

  .search-input:focus {
    border-color: #60a5fa;
    background: #111827;
    box-shadow: 0 0 0 3px rgba(96, 165, 250, 0.1);
  }

  .clear-btn:hover {
    background: #374151;
    color: #d1d5db;
  }

  .search-hint kbd {
    background: #374151;
    border-color: #4b5563;
  }
}
</style>
