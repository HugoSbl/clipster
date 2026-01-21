# Tasks: Clipboard Types, Preview & Source App Detection

## Overview

Transform Clipster into a Paste-like overlay with enlarged UI, macOS file support, source app detection with icons, Quick Look thumbnails, and adaptive full-width window.

## Task List

| Task | Name | Dependencies | Platform |
|------|------|--------------|----------|
| 1 | Enlarge UI Components | None | Frontend |
| 2 | Add source_app_icon Field | None | Both |
| 3 | macOS File List Reading | None | macOS |
| 4 | Windows Source App Detection | Task 2 | Windows |
| 5 | macOS Source App Detection | Task 2, 3 | macOS |
| 6 | Windows Source App Icon | Task 4 | Windows |
| 7 | macOS Source App Icon | Task 5 | macOS |
| 8 | Display Source App Icon in Card | Task 1, 2, (6 or 7) | Frontend |
| 9 | Quick Look File Thumbnails | Task 3 | macOS |
| 10 | Shell Thumbnail Cache | None | Windows |
| 11 | Display File Thumbnails in Card | Task 1, (9 or 10) | Frontend |
| 12 | Adaptive Overlay Window | None | Both |

- [x] **Task 1**: Enlarge UI Components - `task-01.md`
- [x] **Task 2**: Add source_app_icon Field - `task-02.md`
- [x] **Task 3**: macOS File List Reading - `task-03.md`
- [x] **Task 4**: Windows Source App Detection - `task-04.md` (depends on Task 2)
- [x] **Task 5**: macOS Source App Detection - `task-05.md` (depends on Tasks 2, 3)
- [x] **Task 6**: Windows Source App Icon - `task-06.md` (depends on Task 4)
- [x] **Task 7**: macOS Source App Icon - `task-07.md` (depends on Task 5)
- [x] **Task 8**: Display Source App Icon in Card - `task-08.md` (depends on Tasks 1, 2, 6/7)
- [x] **Task 9**: Quick Look File Thumbnails (macOS) - `task-09.md` (depends on Task 3)
- [x] **Task 10**: Shell Thumbnail Cache (Windows) - `task-10.md`
- [x] **Task 11**: Display File Thumbnails in Card - `task-11.md` (depends on Tasks 1, 9/10)
- [x] **Task 12**: Adaptive Overlay Window - `task-12.md`

## Execution Strategy

```
[Task 1 ‖ Task 2 ‖ Task 3 ‖ Task 10 ‖ Task 12]
              ↓
        [Task 4 ‖ Task 5 ‖ Task 9]
              ↓
        [Task 6 ‖ Task 7]
              ↓
        [Task 8 ‖ Task 11]
```

### Wave 1 (Foundation - No Dependencies)
**Run in parallel:** Tasks 1, 2, 3, 10, 12

- Task 1: UI enlargement (frontend)
- Task 2: Data model changes (backend)
- Task 3: macOS file reading (macOS backend)
- Task 10: Windows thumbnails (Windows backend)
- Task 12: Overlay window (config/backend)

### Wave 2 (Source Detection + macOS Thumbnails)
**Run after Wave 1:** Tasks 4, 5, 9

- Task 4: Windows source app (needs Task 2)
- Task 5: macOS source app (needs Tasks 2, 3)
- Task 9: Quick Look thumbnails (needs Task 3)

### Wave 3 (Icon Extraction)
**Run after Wave 2:** Tasks 6, 7

- Task 6: Windows icon extraction (needs Task 4)
- Task 7: macOS icon extraction (needs Task 5)

### Wave 4 (Frontend Integration)
**Run after Wave 3:** Tasks 8, 11

- Task 8: Source app icon display (needs Tasks 1, 2, 6/7)
- Task 11: File thumbnail display (needs Tasks 1, 9/10)

## Platform-Specific Paths

### macOS Development Path
```
[Task 1 ‖ Task 2 ‖ Task 3 ‖ Task 12] → [Task 5 ‖ Task 9] → Task 7 → [Task 8 ‖ Task 11]
```

### Windows Development Path
```
[Task 1 ‖ Task 2 ‖ Task 10 ‖ Task 12] → Task 4 → Task 6 → [Task 8 ‖ Task 11]
```

## Recommended Commands

```bash
# Auto-detect parallel tasks (recommended)
/apex:3-execute 02-clipboard-types-preview-source-icons

# Wave 1: Foundation (parallel)
/apex:3-execute 02-clipboard-types-preview-source-icons 1,2,3,10,12

# Wave 2: Source detection (parallel, after Wave 1)
/apex:3-execute 02-clipboard-types-preview-source-icons 4,5,9

# Wave 3: Icon extraction (parallel, after Wave 2)
/apex:3-execute 02-clipboard-types-preview-source-icons 6,7

# Wave 4: Frontend integration (parallel, after Wave 3)
/apex:3-execute 02-clipboard-types-preview-source-icons 8,11

# Single task execution
/apex:3-execute 02-clipboard-types-preview-source-icons 1
```

## Notes

- **macOS focus**: If developing on macOS, skip Windows tasks (4, 6, 10) for now
- **Windows focus**: If developing on Windows, skip macOS tasks (3, 5, 7, 9) for now
- **Quick wins**: Start with Task 1 (UI) for immediate visual improvement
- **Core functionality**: Task 3 (macOS files) unlocks file support

**Start with**: Tasks 1, 2, 3 in parallel - they have no dependencies and cover different layers (frontend, data model, backend).
