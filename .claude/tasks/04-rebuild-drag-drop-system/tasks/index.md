# Tasks: Rebuild Drag & Drop System

## Overview

Complete refactoring of the drag & drop system to eliminate race conditions between HTML5 drag and `tauri-plugin-drag`. The new architecture uses a single, simplified approach: plugin-only drag with native OS preview.

**Goal**: Replace the complex dual-mode system (~390 lines) with a simple mousedown → threshold → startDrag() flow.

## Task List

| Task | Name | Dependencies | Est. Time |
|------|------|--------------|-----------|
| 1 | Configure Tauri for Plugin Use | None | 15 min |
| 2 | Add Backend Commands for Text/Link | Task 1 | 45 min |
| 3 | Refactor ClipboardCard for Plugin-Only | Task 2 | 1.5-2h |
| 4 | Manual Testing and Verification | Task 3 | 1h |

- [x] **Task 1**: Configure Tauri for Plugin Use - `task-01.md`
- [x] **Task 2**: Add Backend Commands for Text/Link - `task-02.md` (depends on Task 1)
- [x] **Task 3**: Refactor ClipboardCard for Plugin-Only - `task-03.md` (depends on Task 2)
- [x] **Task 4**: Manual Testing and Verification - `task-04.md` (depends on Task 3)

## Execution Strategy

**Task 1 → Task 2 → Task 3 → Task 4**

**No parallelization possible**: Each task depends on the previous one completing.

- Task 2 needs Task 1's config to be in place
- Task 3 needs Task 2's backend commands to exist
- Task 4 needs Task 3's implementation to be complete

**Total estimated time**: 3-4 hours

## Recommended Commands

```bash
# Execute tasks sequentially (recommended)
/apex:3-execute 04-rebuild-drag-drop-system 1
/apex:3-execute 04-rebuild-drag-drop-system 2
/apex:3-execute 04-rebuild-drag-drop-system 3
/apex:3-execute 04-rebuild-drag-drop-system 4

# Or use auto-continue (executes next task automatically)
/apex:3-execute 04-rebuild-drag-drop-system 1 --yolo
```

**Start with**: Task 1 - it has no dependencies and takes only 15 minutes.

## Task Breakdown Rationale

**Task 1 (Config)**: Separated because it's quick, standalone, and must be done first to enable plugin use.

**Task 2 (Backend)**: Grouped both commands together because they're related (temp file creation), in the same file, and frontend depends on both.

**Task 3 (Frontend)**: Kept as single large task because the refactoring is tightly coupled - removing HTML5 logic and adding plugin logic must happen together to avoid broken intermediate states.

**Task 4 (Testing)**: Separated because it's purely validation and can only happen after implementation is complete.

## Critical Success Factors

1. **Strict sequential order**: Do NOT skip ahead or parallelize
2. **Complete each task**: Verify success criteria before moving to next
3. **No mixing systems**: Remove HTML5 drag entirely before relying on plugin
4. **Type safety**: Run `npx vue-tsc --noEmit` after Task 3
5. **Comprehensive testing**: Task 4 must cover all content types and platforms

## Files Modified (Summary)

- `src-tauri/tauri.conf.json` - Task 1
- `src-tauri/src/commands/clipboard_commands.rs` - Task 2
- `src-tauri/src/main.rs` - Task 2
- `src/components/ClipboardCard.vue` - Task 3 (major refactoring)

**Net change**: ~390 lines removed, ~100 lines added (simpler codebase)
