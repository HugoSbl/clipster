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
        :placeholder="placeholder || 'Search...'"
        class="search-input"
        @input="handleInput"
        @keydown="handleKeyDown"
      />

      <!-- Clear Button -->
      <button
        v-if="inputValue"
        class="clear-btn"
        @click="clearSearch"
        title="Clear (Esc)"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.search-bar {
  width: 280px;
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 10px;
  width: 14px;
  height: 14px;
  color: #9ca3af;
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 32px;
  padding: 0 32px 0 32px;
  border: none;
  border-radius: 8px;
  font-size: 13px;
  color: #374151;
  background: rgba(0, 0, 0, 0.06);
  outline: none;
  transition: background-color 0.15s, box-shadow 0.15s;
}

.search-input::placeholder {
  color: #9ca3af;
}

.search-input:hover {
  background: rgba(0, 0, 0, 0.08);
}

.search-input:focus {
  background: rgba(255, 255, 255, 0.8);
  box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.1);
}

.clear-btn {
  position: absolute;
  right: 6px;
  width: 20px;
  height: 20px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: #9ca3af;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s, color 0.15s;
}

.clear-btn:hover {
  background: rgba(0, 0, 0, 0.08);
  color: #6b7280;
}

.clear-btn svg {
  width: 12px;
  height: 12px;
}
</style>
