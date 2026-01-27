# Tasks: Fix Drag Visual - Card Instead of File

## Overview

Fix the race condition between HTML5 drag and native drag in ClipboardCard component. Currently, the browser can initiate HTML5 dragstart before our threshold detection completes, causing unstable behavior where sometimes the file is dragged, sometimes the card is dragged.

**Solution strategy**: Prevent HTML5 drag completely, manage visual feedback manually with a ghost clone, and use tauri-plugin-drag for file transfer after threshold.

**Total estimated time**: 2-3 hours across 3 tasks

## Task List

| Task | Name | Dependencies | Est. Time |
|------|------|--------------|-----------|
| 1 | Prevent HTML5 Drag Race Condition | None | ~1h |
| 2 | Implement Manual Ghost Positioning System | Task 1 | ~45min |
| 3 | Improve Drag Error Handling and Logging | None | ~30min |

### Detailed Task Breakdown

- [x] **Task 1**: Prevent HTML5 Drag Race Condition - `task-01.md` ✅ **COMPLETED**
  - **Critical foundation task** - eliminates the root cause
  - Add preventDefault() to block browser's automatic drag
  - Create ghost clone immediately on mousedown
  - Remove HTML5 drag handlers and template bindings
  - Must be completed first
  - **Implementation report**: `task-01-implementation.md`

- [x] **Task 2**: Implement Manual Ghost Positioning System - `task-02.md` (depends on Task 1) ✅ **COMPLETED**
  - Create updateClonePosition() helper function
  - Position clone at cursor with GPU-accelerated transforms
  - Update position smoothly on every mousemove
  - Clean up clone properly when drag ends
  - Add CSS styles for .drag-ghost class
  - **Implementation report**: `task-02-implementation.md`

- [x] **Task 3**: Improve Drag Error Handling and Logging - `task-03.md` (independent) ✅ **COMPLETED**
  - Wrap startDrag() in try-catch with logging
  - Handle edge cases (empty paths, invalid files)
  - Add fallback behavior for failures
  - Validate file paths before drag operations
  - Can run in parallel with Task 2

## Execution Strategy

```
Task 1 → [Task 2 ‖ Task 3]
```

**Parallelization opportunity**: Tasks 2 and 3 can be executed simultaneously after Task 1 completes.

- **Task 1** is the critical foundation (prevents race condition)
- **Task 2** depends on Task 1 (needs ghost clone to position)
- **Task 3** is independent (error handling improvements)

## Recommended Commands

```bash
# Auto-detect parallel tasks (recommended)
/apex:3-execute 01-fix-drag-visual-card

# Execute tasks explicitly
/apex:3-execute 01-fix-drag-visual-card 1         # Start with foundation
/apex:3-execute 01-fix-drag-visual-card 2,3       # Then run 2 and 3 in parallel

# Or sequential execution
/apex:3-execute 01-fix-drag-visual-card 1
/apex:3-execute 01-fix-drag-visual-card 2
/apex:3-execute 01-fix-drag-visual-card 3
```

**Start with**: Task 1 - it's the critical foundation that prevents the race condition.

## Testing Strategy

After completing all tasks, perform comprehensive manual testing:

### Core Functionality Tests
1. **File drag to external app** - verify file transfer works
2. **Image drag to external app** - verify with readable filename
3. **Text/link drag** - verify internal behavior
4. **Short click** - verify threshold prevents accidental drag
5. **Rapid drags** - verify cleanup prevents clone leakage

### Visual Verification
- Clone appears immediately on mousedown
- Clone follows cursor smoothly (no lag)
- Source card fades/scales during drag
- Clone disappears cleanly on drag end

### Error Handling
- Check console for error logs when things fail
- Verify graceful degradation on errors
- Test with corrupted paths/invalid data

### Cross-Platform
- Test on macOS (primary)
- Test on Windows if available
- Test on Linux if available

## Success Criteria (Overall)

When all tasks are complete:
- ✅ No race condition between HTML5 and native drag
- ✅ Clone appears immediately and follows cursor smoothly
- ✅ File transfer works correctly after threshold (5px)
- ✅ Errors are logged with helpful context
- ✅ No memory leaks or leftover clones
- ✅ Consistent behavior (no more "sometimes works, sometimes doesn't")
- ✅ Source card shows proper visual feedback during drag

## Implementation Notes

**Critical to avoid:**
- Forgetting preventDefault() in Task 1 (race condition persists)
- Not removing clone from DOM (memory leak)
- Calling startDrag() before threshold (defeats anti-false-positive)
- Leaving HTML5 drag attributes in template (conflict remains)

**Performance tips:**
- Use transform: translate3d() for GPU acceleration
- Apply pointer-events: none to clone
- Consider requestAnimationFrame if positioning feels janky
- Monitor DevTools for layout thrashing

**Files modified:**
- `src/components/ClipboardCard.vue` (script, template, styles)
- No backend or other component changes needed
