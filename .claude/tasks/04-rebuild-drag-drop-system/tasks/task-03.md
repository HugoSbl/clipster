# Task 3: Refactor ClipboardCard for Plugin-Only Drag System

## Problem

ClipboardCard currently uses a dual-mode drag system:
1. **HTML5 drag** (`@dragstart`, `createExactClone()`) for visual preview
2. **Plugin drag** (`startDrag()`) for native file transfer

These two systems race against each other - HTML5 drag starts before `startDrag()` completes, causing:
- Visual "flash" or "jump" when drag begins
- Inconsistent drag ghost appearance
- Complex ~390 lines of manual DOM cloning and positioning code

The complexity makes the code hard to maintain and debug.

## Proposed Solution

**Complete refactoring** to use **ONLY** `tauri-plugin-drag`:

1. **Remove ALL HTML5 drag logic** (~390 lines):
   - `createExactClone()` function
   - `handleNativeDragStart()`, `handleNativeDragMove()`, `handleNativeDragEnd()`
   - `updateClonePosition()`
   - `handleImageDragStart()`, `handleImageDrag()`
   - `.drag-ghost` CSS class

2. **Implement simplified flow**:
   ```
   mousedown → threshold (5px) → startDrag() ONLY
                                    ↓
                          Plugin handles native preview
   ```

3. **Expand `getFilePathsForDrag()`** to support all content types:
   - Images: existing `prepare_image_for_drag` command
   - Text: new `create_temp_text_file` command
   - Links: new `create_temp_link_file` command
   - Files/Audio/Documents: parse JSON paths directly

4. **Update template**:
   - Replace `@dragstart` with `@mousedown`
   - Remove all HTML5 drag attributes
   - Keep `draggable="false"` on images (prevents browser drag)

## Dependencies

- **Task 2**: Backend commands for text/link must exist before frontend can call them
- External: `@crabnebula/tauri-plugin-drag` must be installed (already is)

## Context

**File to modify:**
- `src/components/ClipboardCard.vue` (MAJOR refactoring)

**Lines to REMOVE** (~390 lines total):
- Lines 210-260: `createExactClone()` - Manual DOM cloning
- Lines 437-478: `handleNativeDragStart()` - Drag initialization
- Lines 480-550: `handleNativeDragMove()` - Clone positioning
- Lines 552-559: `handleNativeDragEnd()` - Cleanup
- Lines 579-589: `updateClonePosition()` - GPU optimization
- Lines 592-603: `handleImageDragStart()`, `handleImageDrag()` - HTML5 handlers

**Functions to KEEP and EXPAND:**
- `prepareImageForDrag()` (lines 287-327) - Already handles images
- `getFilePathsForDrag()` (lines 329-418) - Need to add text/link support

**New simplified pattern:**
```typescript
const DRAG_THRESHOLD = 5;
const isDragging = ref(false);
const dragStartPos = ref<{ x: number; y: number } | null>(null);

const handleMouseDown = (e: MouseEvent) => {
  dragStartPos.value = { x: e.clientX, y: e.clientY };
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
};

const handleMouseMove = async (e: MouseEvent) => {
  if (!dragStartPos.value) return;

  const dx = Math.abs(e.clientX - dragStartPos.value.x);
  const dy = Math.abs(e.clientY - dragStartPos.value.y);

  if (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD) {
    await initiateDrag();
  }
};

const initiateDrag = async () => {
  // Cleanup listeners
  // Get file paths for content type
  // Call startDrag() with item and icon
};
```

**Import needed:**
```typescript
import { startDrag } from '@crabnebula/tauri-plugin-drag';
```

**Content type mapping:**
- `text` → `create_temp_text_file()`
- `link` → `create_temp_link_file()`
- `image` → `prepare_image_for_drag()` (existing)
- `files`, `audio`, `documents` → Parse JSON paths

**Visual states to maintain:**
- Add `.dragging` CSS class during drag (opacity, scale)
- Remove class on drag end
- No drag ghost - plugin handles it

## Success Criteria

- All HTML5 drag code removed (~390 lines deleted)
- Simplified mousedown → threshold → startDrag() flow implemented
- All content types (text, image, link, files, audio, documents) supported
- No race condition (stable, predictable behavior)
- Drag initiates after 5px threshold, not on click
- Code is cleaner and more maintainable
- TypeScript compilation passes (`npx vue-tsc --noEmit`)
- No visual "flash" when drag starts
