# Implementation Plan: Fix Drag Visual - Card Instead of File

**Created**: 2026-01-23
**Strategy**: Hybrid manual approach - prevent HTML5 drag, manage visual feedback manually

## Overview

The current implementation has a race condition between HTML5 drag (for visual) and native drag via tauri-plugin-drag (for file transfer). The browser can initiate `dragstart` before our threshold detection completes, causing HTML5 to take over and drag the card data instead of transferring files.

**Solution approach:**
1. Prevent browser's automatic HTML5 drag initiation completely
2. Create visual clone immediately on mousedown (before threshold)
3. Display clone manually via `position: fixed` that follows the cursor
4. For files/images: call `startDrag()` plugin in parallel after threshold
5. For text/links: manage drag data through custom implementation

This approach eliminates the race condition by taking full control of both visual feedback and data transfer.

## Dependencies

**None blocking** - All required dependencies already installed:
- `@crabnebula/tauri-plugin-drag: ^2.1.0` ✅
- Plugin initialized in `main.rs:60` ✅
- `prepare_image_for_drag` command registered ✅

**Execution order:**
1. Modify drag handlers (Step 1)
2. Update template bindings (Step 2)
3. Add manual ghost styles (Step 3)
4. Manual testing (Step 4)

## File Changes

### `src/components/ClipboardCard.vue` (Script Section)

**Lines 21-30: Add new state for manual ghost**
- Add `dragClone` ref to hold the manually created ghost element: `const dragClone = ref<HTMLElement | null>(null)`
- Add `dragClonePosition` ref to track cursor position: `const dragClonePosition = ref<{ x: number; y: number } | null>(null)`
- Keep existing `isDragging` ref for state management
- Keep existing `cardRef` for element reference

**Lines 392-409: Modify `handleNativeDragStart()`**
- Add `e.preventDefault()` as FIRST line to prevent browser's automatic drag initiation
- Add `e.stopPropagation()` to prevent event bubbling
- Create ghost clone IMMEDIATELY using existing `createExactClone()` function (don't wait for threshold)
- Store clone in `dragClone.value`
- Append clone to `document.body`
- Set initial clone position to cursor coordinates minus card offset
- Keep existing threshold detection setup (5px)
- Pattern: Follow existing threshold pattern at line 395

**Lines 411-459: Modify `handleNativeDragMove()`**
- Update `dragClonePosition.value` with current cursor coordinates on EVERY mousemove (not just after threshold)
- After threshold exceeded (existing logic):
  - Keep `isDragging.value = true`
  - Keep `pinboardStore.setDragging(true, props.item.id)`
  - For `canDragAsFiles = true`: call `await startDrag()` with proper error handling (wrap in try-catch)
  - For `canDragAsFiles = false`: encode drag data in a custom format (store in window/global for retrieval)
- Add proper error logging for `startDrag()` failures (currently silently swallowed at lines 451-453)
- Consider: Handle case where `getFilePathsForDrag()` returns empty array

**Lines 461-470: Modify `cleanup()`**
- Remove clone from DOM: `if (dragClone.value) { dragClone.value.remove(); }`
- Reset `dragClone.value = null`
- Reset `dragClonePosition.value = null`
- Keep existing listener removal
- Keep existing state resets

**Lines 473-513: Remove or repurpose HTML5 drag handlers**
- Delete `handleDragStart()` function entirely (no longer needed)
- Delete `handleDragEnd()` function entirely (no longer needed)
- Alternative: Keep as fallback with console.warn if accidentally triggered

**Lines 210-260: Verify `createExactClone()` works for manual positioning**
- Review that clone has `position: fixed` (should already be there)
- Ensure `pointer-events: none` is set (should already be there)
- Ensure clone is removed from flow with off-screen positioning initially
- No changes needed if current implementation works (already production-ready per analysis)

**New function after line 470: `updateClonePosition()`**
- Create helper function to update clone's transform based on `dragClonePosition`
- Apply `transform: translate3d(x, y, 0)` for GPU acceleration
- Call this in `handleNativeDragMove()` on every cursor update
- Pattern: Use `requestAnimationFrame` for smooth updates if needed

**Lines 351-378: Add error handling to `getFilePathsForDrag()`**
- Wrap `prepareImageForDrag()` call in try-catch
- If fails, fallback to original image path (already done at line 323, verify it works)
- Log errors to console for debugging
- Consider: Return empty array if all paths fail to prevent `startDrag()` from erroring

### `src/components/ClipboardCard.vue` (Template Section)

**Lines 526-600: Remove HTML5 drag bindings**
- Remove `:draggable="!canDragAsFiles"` attribute completely (line 526, 595)
- Remove `@dragstart="handleDragStart"` listener (line 598)
- Remove `@dragend="handleDragEnd"` listener (line 600)
- Keep `@mousedown="handleNativeDragStart"` (critical for our manual approach)
- Keep `@mouseup` listener if present (for cleanup)
- Keep `:class="{ dragging: isDragging }"` for visual feedback

**Verification:**
- Ensure no HTML5 drag events remain in template
- Ensure mousedown/mousemove handlers are preserved
- Check that `cardRef` is still bound to root element

### `src/components/ClipboardCard.vue` (Style Section)

**New style block: Manual ghost positioning**
- Add CSS class `.drag-ghost` for manually positioned clone
- Style properties:
  - `position: fixed !important` (force override)
  - `pointer-events: none` (prevent interaction)
  - `z-index: 999999` (ensure above everything)
  - `transition: none` (instant positioning)
  - `will-change: transform` (GPU optimization)
- Apply this class to `dragClone` element when created
- Pattern: Follow existing `.dragging` class style at line 1100+

**Existing style `.clipboard-card.dragging`:**
- Verify `opacity: 0.5` and `transform: scale(0.95)` still apply to source card
- No changes needed (provides good visual feedback that drag is active)

### `src/stores/pinboards.ts` (Optional Enhancement)

**Lines 255-268: Consider using `updateDragPosition()`**
- Currently defined but not called from ClipboardCard
- Could be used to track ghost position for drop zone highlighting
- Call from `handleNativeDragMove()` with cursor coordinates
- Pattern: `pinboardStore.updateDragPosition(e.clientX, e.clientY)`
- Consider: Only if drop zone highlighting is desired feature

**Lines 273-288: Verify `completeDrag()` integration**
- Currently not called from native drag handlers
- May need integration if drag-to-pinboard should work
- Consider: Call from `handleNativeDragEnd()` if cursor is over drop zone
- Pattern: Check `hoveredDropZone` and execute action

## Testing Strategy

**Manual testing required** (no automated tests for drag & drop):

### Test Case 1: File drag to external app
1. Open app with files/images in timeline
2. Click and drag a file card
3. Verify: Card clone appears immediately on mousedown
4. Verify: Clone follows cursor smoothly
5. Drag to Finder/Desktop
6. Verify: File is transferred correctly
7. Verify: Source card shows faded/scaled state during drag

### Test Case 2: Image drag to external app
1. Select image card
2. Drag to external app
3. Verify: Image file is transferred with readable filename
4. Verify: No errors in console
5. Check `/tmp` for generated thumbnail (cleanup verification)

### Test Case 3: Text/link drag (internal)
1. Select text or link card
2. Drag within app
3. Verify: Clone appears and follows cursor
4. Verify: Data is accessible (if internal drop implemented)
5. Verify: No file transfer attempted

### Test Case 4: Short click (no drag)
1. Click card without moving mouse
2. Verify: No drag initiated (threshold not exceeded)
3. Verify: Card selection still works
4. Verify: No ghost clone created

### Test Case 5: Edge cases
1. Drag card to edge of window - verify clone doesn't disappear
2. Rapid drag start/stop - verify cleanup happens correctly
3. Drag with slow movement - verify threshold works (5px)
4. Multiple cards in sequence - verify no clone leakage between drags

### Test Case 6: Error scenarios
1. Drag file with corrupted path - verify fallback behavior
2. Drag image when `prepare_image_for_drag` fails - verify error handling
3. Check console for any uncaught errors during drag operations

### Cross-platform verification
- Test on macOS (primary platform)
- Test on Windows if available
- Test on Linux if available
- Verify `startDrag()` plugin works on all platforms

## Documentation

**No documentation updates required** - This is a bug fix, not a feature change. Internal drag behavior remains the same from user perspective.

**Optional: Add code comments**
- Add comment above `handleNativeDragStart()` explaining race condition fix
- Add comment in `createExactClone()` noting it's used for manual ghost rendering
- Add comment explaining why HTML5 drag is completely disabled

## Rollout Considerations

**No breaking changes** - This is an internal implementation fix.

**Rollback plan:**
- If issues arise, revert to previous commit
- Original HTML5 + native dual-mode system can be restored

**Performance considerations:**
- Manual clone positioning uses `transform` for GPU acceleration
- `requestAnimationFrame` may be needed if jank occurs on mousemove
- Monitor for memory leaks if clone isn't properly cleaned up

**Feature flags:**
- None needed - direct replacement of buggy behavior

**Migration steps:**
- None - users will automatically get the fix

## Implementation Notes

**Critical path:**
1. Fix `handleNativeDragStart()` to prevent HTML5 drag - THIS IS THE KEY FIX
2. Create clone immediately and position manually
3. Update clone position on every mousemove
4. Call `startDrag()` after threshold for file transfer
5. Remove HTML5 drag event handlers from template

**Common pitfalls to avoid:**
- Forgetting to remove clone from DOM (memory leak)
- Not using `pointer-events: none` on clone (blocks mouse events)
- Calling `startDrag()` before threshold (defeats anti-false-positive)
- Not handling errors from `startDrag()` (silent failures)
- Leaving HTML5 drag attributes in template (race condition persists)

**Success criteria:**
- ✅ Clone appears immediately on mousedown
- ✅ Clone follows cursor smoothly
- ✅ File transfer works after threshold exceeded
- ✅ No race condition between HTML5 and native drag
- ✅ Source card shows visual feedback (faded/scaled)
- ✅ No console errors during drag operations
- ✅ Clone is properly cleaned up on drag end

## Estimated Complexity

**Time estimate:** 2-3 hours (as per analysis)

**Breakdown:**
- Modify drag handlers: 1-1.5h
- Update template and styles: 0.5h
- Manual testing: 0.5-1h

**Risk level:** Medium
- Risk: Manual ghost positioning may have edge cases on different platforms
- Risk: File transfer might fail if paths are invalid
- Mitigation: Comprehensive error handling and fallback behavior
