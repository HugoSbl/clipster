# Task: Implement Manual Ghost Positioning System

## Problem

After preventing HTML5 drag (Task 1), we need a system to manually position the ghost clone so it follows the cursor during drag operations. Currently, `createExactClone()` creates a clone positioned off-screen, but we need to:
1. Position it at the cursor location
2. Update its position smoothly as the mouse moves
3. Clean it up properly when drag ends

Without this, the ghost clone will exist but won't be visible or follow the cursor, defeating the purpose of the visual feedback.

## Proposed Solution

Create a complete manual positioning system that:
1. Implements `updateClonePosition()` helper function to apply cursor-based transforms
2. Updates clone position on EVERY mousemove (not just after threshold)
3. Uses `transform: translate3d()` for GPU-accelerated smooth movement
4. Properly cleans up clone from DOM when drag ends
5. Adds CSS class `.drag-ghost` with optimized positioning styles

The system should provide butter-smooth visual feedback that feels natural, using GPU acceleration and proper cleanup to avoid memory leaks.

## Dependencies

- **Task 1**: Requires ghost clone to be created and state refs (`dragClone`, `dragClonePosition`) to be available
- Must run AFTER Task 1 completes

## Context

**Key files:**
- `src/components/ClipboardCard.vue:411-459` - `handleNativeDragMove()` (update position here)
- `src/components/ClipboardCard.vue:461-470` - `cleanup()` (remove clone here)
- New function after line 470 - `updateClonePosition()` helper
- Style section - Add `.drag-ghost` CSS class

**Patterns to follow:**
- Use `transform: translate3d(x, y, 0)` for GPU acceleration (better than top/left)
- Consider `requestAnimationFrame` if positioning feels janky
- Follow existing cleanup pattern in `cleanup()` function
- Apply `pointer-events: none` to prevent clone from blocking mouse events

**Technical approach:**
```typescript
// On every mousemove:
dragClonePosition.value = { x: e.clientX, y: e.clientY };
updateClonePosition(); // Apply transform to clone

// Helper calculates offset from mousedown point
// Applies translate3d for smooth GPU-accelerated movement
```

**Performance considerations:**
- Use `will-change: transform` in CSS for optimization hint
- Keep transforms on GPU with translate3d (not translate2d)
- Avoid layout thrashing by batching DOM reads/writes

## Success Criteria

- [ ] Ghost clone appears at cursor position immediately on mousedown
- [ ] Clone follows cursor smoothly during mouse movement
- [ ] Movement is GPU-accelerated (no jank or stuttering)
- [ ] Clone is properly removed from DOM when drag ends
- [ ] No memory leaks (clone doesn't persist between drags)
- [ ] Clone has `pointer-events: none` (doesn't block interactions)
- [ ] CSS `.drag-ghost` class provides optimal positioning styles

**Verification steps:**
1. Start drag - clone should appear at cursor
2. Move mouse rapidly - clone should follow smoothly without lag
3. Move to screen edges - clone should stay visible
4. End drag - clone should disappear immediately
5. Start another drag - no leftover clones from previous drag
6. Check DevTools performance - no layout thrashing warnings
