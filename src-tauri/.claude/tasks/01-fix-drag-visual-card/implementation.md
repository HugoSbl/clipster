# Implementation Log: Fix Drag Visual Card

## Task 2: Implement Manual Ghost Positioning System

**Date**: 2026-01-23
**Status**: COMPLETED

### Changes Made

**File: `src/components/ClipboardCard.vue`**

1. **Added `dragOffset` variable** (line ~501)
   - Stores offset from cursor to clone origin for smooth tracking
   - Cleaned up in `cleanup()` function

2. **Added `updateClonePosition()` helper function** (lines ~504-513)
   - Takes `clientX`, `clientY` parameters
   - Calculates position: `x = clientX - offsetX`, `y = clientY - offsetY`
   - Uses `transform: translate3d(x, y, 0)` for GPU-accelerated positioning
   - Updates `dragClonePosition.value` for state tracking

3. **Modified `handleNativeDragStart()`** (lines ~409-431)
   - Calculate and store `dragOffset` from cursor to card origin
   - Add `.drag-ghost` class to clone for optimized styles
   - Set clone's `left` and `top` to `0px` (positioning via transform)
   - Call `updateClonePosition()` for initial positioning

4. **Modified `handleNativeDragMove()`** (lines ~434-442)
   - Call `updateClonePosition()` on EVERY mousemove (before threshold check)
   - This ensures smooth tracking even before native drag threshold is reached
   - Reordered logic: update position first, then check if drag has started

5. **Updated `cleanup()` function**
   - Added `dragOffset = null` to prevent memory leaks

6. **Added `.drag-ghost` CSS class** (scoped styles, global selector)
   - `position: fixed` with high z-index
   - `pointer-events: none` to prevent blocking mouse events
   - `will-change: transform` for GPU optimization hint
   - `left: 0; top: 0` with positioning via transform
   - Enhanced `box-shadow` for lift effect
   - `transition: none` to prevent lag during drag

### Validation Results

- **Typecheck**: Pass (npx vue-tsc --noEmit)
- **Lint**: No lint script available in project

### Technical Details

**Why translate3d instead of top/left?**
- `translate3d()` triggers GPU compositing layer
- Transforms don't cause reflow/repaint
- Results in 60fps smooth movement

**Why update position on every mousemove (not just after threshold)?**
- Provides immediate visual feedback
- Clone appears to "stick" to cursor from first frame
- Better UX - user sees responsive feedback

**Why :global(.drag-ghost)?**
- Clone is appended to `document.body`, outside component scope
- Vue scoped styles wouldn't apply otherwise
- Using `:global()` ensures styles reach the clone

### Notes

- The clone now follows the cursor smoothly from the moment of mousedown
- Visual feedback is immediate (no waiting for threshold)
- Native drag (tauri-plugin-drag) only triggers after 5px threshold
- Clone is properly removed on mouseup or when drag completes

---

## Task 3: Improve Drag Error Handling and Logging

**Date**: 2026-01-23
**Status**: COMPLETED
**Executed in parallel with Task 2**

### Changes Made

**File: `src/components/ClipboardCard.vue`**

1. **Enhanced `prepareImageForDrag()` error logging** (lines 320-323)
   - Changed error message format to `[ClipboardCard] prepareImageForDrag error:`
   - Added context object with `path` and `error` details
   - Maintains fallback behavior using original path

2. **Improved `getFilePaths()` error handling** (lines 329-359)
   - Added warning when no valid file paths found after filtering
   - Added warning for malformed JSON in `content_text`
   - All warnings include `contentType` and error context
   - Prevents silent failures during path extraction

3. **Enhanced `getFilePathsForDrag()` validation** (lines 361-398)
   - Added validation for empty image paths with error logging
   - Added warning when `prepareImageForDrag()` returns null
   - Added validation for empty file path arrays
   - Added warning for empty icon paths
   - All edge cases logged with `itemId` and `contentType`

4. **Improved `handleNativeDragMove()` error handling** (lines 439-498)
   - Changed empty file path logging from `console.log` to `console.warn`
   - Added validation and warning for empty icon paths
   - Changed `startDrag()` errors from `console.debug` to `console.error`
   - Added comprehensive error context: `itemId`, `contentType`, `filePaths`, `iconPath`
   - Errors logged but UI remains functional (graceful degradation)

### Validation Results

- **Typecheck**: Pass (npx vue-tsc --noEmit)
- All error messages use `[ClipboardCard]` prefix for easy console filtering
- Error objects include relevant context for debugging
- UI remains functional even when drag operations fail

### Logging Format

All error messages follow consistent patterns:
- `console.error('[ClipboardCard] startDrag failed:', { error, itemId, ... })`
- `console.warn('[ClipboardCard] Empty file paths, skipping drag', { ... })`
- Context objects include: `itemId`, `contentType`, `filePaths`, `error`

### Notes

- No silent failures - all errors are now visible in console
- Graceful degradation ensures drag failures don't crash the UI
- Follows existing error handling pattern from codebase
- All edge cases (empty paths, invalid JSON, null returns) are handled
