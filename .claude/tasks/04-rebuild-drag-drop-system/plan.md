# Implementation Plan: Rebuild Drag & Drop System from Scratch

**Created**: 2026-01-23
**Estimated Time**: 3-4 hours
**Complexity**: Medium-High

---

## Overview

Refonte complète du système de drag & drop pour éliminer la race condition entre HTML5 et `tauri-plugin-drag`. L'approche est ultra-simple: supprimer tout le code HTML5 drag (clone manuel, positioning GPU), et utiliser **UNIQUEMENT** `tauri-plugin-drag` avec sa preview native automatique.

**Architecture cible:**
```
mousedown → threshold (5px) → startDrag() UNIQUEMENT
                               ↓
                      Plugin gère preview native
                      OS gère drag & drop
```

**Avantages:**
- ✅ Pas de race condition (un seul système)
- ✅ Preview native gérée par l'OS
- ✅ Cleanup simple (plus de listeners multiples)
- ✅ Cross-platform automatique

---

## Dependencies

**Execution order:**
1. Configuration → Backend → Frontend (strict dependency order)
2. No parallel work possible (frontend depends on backend commands)

**Prerequisites:**
- `tauri-plugin-drag@2.1.0` already installed ✅
- Rust `image` crate already available ✅
- No blocking dependencies

---

## File Changes

### 1. `src-tauri/tauri.conf.json` (CONFIGURATION - 15 min)

**Priority:** 🔴 Critical first step

**Action:** Add `dragDropEnabled: false` to window configuration

**Details:**
- Locate the `app.windows` array in the config
- Add `"dragDropEnabled": false` property to the first window object
- This disables Tauri's internal drag/drop system
- Enables `tauri-plugin-drag` to work without conflicts

**Why this is confusing:**
- `dragDropEnabled: false` means "disable internal, enable plugin use"
- Counter-intuitive naming but official behavior

**Example location:**
```json
{
  "app": {
    "windows": [
      {
        "title": "Clipster",
        "dragDropEnabled": false  // ADD THIS LINE
      }
    ]
  }
}
```

**Verification:**
- Save file and restart Tauri dev server
- Plugin should now have exclusive drag control

---

### 2. `src-tauri/src/commands/clipboard_commands.rs` (BACKEND - 45 min)

**Priority:** 🟡 Must complete before frontend changes

**Action 1:** Create `create_temp_text_file` command

**Details:**
- Add new Tauri command function after `prepare_image_for_drag` (around line 234)
- Use `std::env::temp_dir()` for temp file location (same as images)
- Write content to file with provided filename
- Return absolute path as String
- Add error handling with descriptive messages

**Function signature:**
```rust
#[tauri::command]
pub fn create_temp_text_file(
    content: String,
    filename: String,
) -> Result<String, String>
```

**Implementation notes:**
- Join temp_dir with filename: `temp_dir.join(&filename)`
- Use `std::fs::write()` to write content
- Map error to String: `.map_err(|e| format!("Failed to write text file: {}", e))?`
- Return path as lossy string: `file_path.to_string_lossy().to_string()`

**Consider:**
- Empty content is valid (user may have copied empty string)
- Filename sanitization already done in frontend
- Files cleaned by OS temp directory cleanup

---

**Action 2:** Create `create_temp_link_file` command

**Details:**
- Add new Tauri command function after `create_temp_text_file`
- Platform-specific file formats:
  - macOS: `.webloc` (XML plist format)
  - Windows: `.url` (INI format)
  - Linux: Return error (not supported)
- Use `#[cfg(target_os = "...")]` for platform conditionals
- Return absolute path as String

**Function signature:**
```rust
#[tauri::command]
pub fn create_temp_link_file(
    url: String,
    filename: String,
) -> Result<String, String>
```

**Implementation notes:**
- macOS plist format:
  ```xml
  <?xml version="1.0" encoding="UTF-8"?>
  <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "...">
  <plist version="1.0">
  <dict>
      <key>URL</key>
      <string>{url}</string>
  </dict>
  </plist>
  ```
- Windows INI format:
  ```ini
  [InternetShortcut]
  URL={url}
  ```
- Use `format!()` macro with raw string literal `r#"..."#` for XML
- Use conditional compilation `#[cfg(target_os = "macos")]`

**Consider:**
- URL encoding not needed (OS handles it)
- Invalid URLs still create files (let OS validate)
- Linux support can be added later if needed

---

**Action 3:** Register commands in `main.rs`

**Details:**
- Locate `invoke_handler` call in `main.rs` (around line 70)
- Add `create_temp_text_file` and `create_temp_link_file` to handler array
- Ensure comma-separated list syntax is correct

**Pattern to follow:**
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    prepare_image_for_drag,
    create_temp_text_file,     // ADD THIS
    create_temp_link_file,     // ADD THIS
])
```

**Verification:**
- Rust compilation should pass: `cargo check`
- Commands available for frontend invocation

---

### 3. `src/components/ClipboardCard.vue` (FRONTEND - 1.5-2h)

**Priority:** 🟢 Final step after backend ready

**Action 1:** Remove obsolete HTML5 drag logic

**Details:**
- Delete `createExactClone()` function (lines 210-260)
  - This was GPU-optimized DOM cloning for manual ghost
  - No longer needed with plugin's native preview
- Delete `handleNativeDragStart()` (lines 437-478)
  - Created clone and set up initial state
  - Replaced by simpler `handleMouseDown()`
- Delete `handleNativeDragMove()` (lines 480-550)
  - Threshold detection + `startDrag()` call
  - Logic preserved but simplified
- Delete `handleNativeDragEnd()` (lines 552-559)
  - Cleanup function
  - Replaced by `handleMouseUp()`
- Delete `updateClonePosition()` (lines 579-589)
  - Manual positioning with `transform: translate3d()`
  - No longer needed
- Delete `handleImageDragStart()` (lines 592-603)
  - Browser drag prevention
  - No longer needed (no HTML5 drag)
- Delete `handleImageDrag()` (lines 592-603)
  - Browser drag prevention
  - No longer needed

**Consider:**
- These functions represent ~390 lines of code
- Save a backup or rely on git history
- Check for any remaining references after deletion

---

**Action 2:** Add simplified drag handlers

**Details:**
- Add three new handler functions in script section
- Keep same threshold pattern (5px detection)
- Remove all clone creation and positioning logic
- Call `startDrag()` directly after threshold

**Handler 1: `handleMouseDown`**
- Record initial mouse position
- Add `mousemove` and `mouseup` event listeners to document
- Store position in `dragStartPos` ref

**Handler 2: `handleMouseMove`**
- Calculate distance from start position
- Check against `DRAG_THRESHOLD` (5px)
- If threshold exceeded:
  - Remove `mousemove` listener (prevent re-trigger)
  - Set `isDragging` state to true
  - Call `pinboardStore.setDragging(true, props.item.id)`
  - Call `getFilePathsForDrag()` to prepare files
  - Call `startDrag({ item, icon })` with plugin
  - Handle errors with console.error

**Handler 3: `handleMouseUp`**
- Remove both `mousemove` and `mouseup` listeners
- Reset `dragStartPos` to null
- Reset `isDragging` to false
- Call `pinboardStore.setDragging(false, null)`

**Pattern to follow:**
- Use Composition API refs (`ref<T>()`)
- Arrow function syntax for handlers
- Async/await for `getFilePathsForDrag()`
- Try-catch for error handling

**Consider:**
- Threshold detection logic preserved from old code
- State management pattern unchanged
- No visual clone, plugin handles preview

---

**Action 3:** Update `getFilePathsForDrag()` function

**Details:**
- Expand function to handle text and link types
- Add new case branches to switch statement
- Call new Rust commands for text/link
- Use placeholder icons for non-image types

**Add case for 'text':**
- Generate readable filename: `${sourceApp}_${timestamp}.txt`
- Call `invoke<string>('create_temp_text_file', { content, filename })`
- Return with placeholder icon: `/default/text-icon.png`

**Add case for 'link':**
- Generate readable filename with platform-specific extension:
  - macOS: `.webloc`
  - Windows: `.url`
- Detect platform with `process.platform === 'darwin'`
- Call `invoke<string>('create_temp_link_file', { url, filename })`
- Return with placeholder icon: `/default/link-icon.png`

**Keep existing cases:**
- 'image': Already correct (uses `prepare_image_for_drag`)
- 'files', 'audio', 'documents': Already correct (pass paths directly)

**Pattern to follow:**
- Same async/await structure as image case
- Same timestamp format for consistency
- Same error handling pattern
- Return type: `Promise<{ items: string[]; icon: string }>`

**Consider:**
- Placeholder icons can be actual files later
- Empty content/url should still create files
- Frontend doesn't validate URLs (Rust does)

---

**Action 4:** Update template drag bindings

**Details:**
- Replace existing drag event handlers with simple `@mousedown`
- Remove all `@dragstart` handlers from images
- Remove `:draggable` attribute bindings
- Keep `dragging` class binding for visual feedback

**Change main card element:**
- FROM: Complex drag event handlers
- TO: Single `@mousedown="handleMouseDown"`
- Keep `:class="{ dragging: isDragging }"`

**Remove from all `<img>` elements:**
- Remove `@dragstart="handleImageDragStart"`
- Remove `@drag="handleImageDrag"`
- Remove `:draggable="false"` attribute

**Pattern to follow:**
```vue
<div
  class="clipboard-card"
  :class="{ dragging: isDragging }"
  @mousedown="handleMouseDown"
>
  <!-- Card content -->
</div>
```

**Consider:**
- Template changes minimal (mostly removals)
- No conditional rendering needed
- Visual feedback via CSS class unchanged

---

**Action 5:** Update CSS styles

**Details:**
- Keep selection prevention CSS (already correct)
- Keep dragging state CSS (already correct)
- Remove `.drag-ghost` class (no longer used)

**Keep these CSS rules:**
```css
.clipboard-card,
.clipboard-card * {
  user-select: none;
  -webkit-user-select: none;
}

.clipboard-card img {
  pointer-events: none;
}

.clipboard-card.dragging {
  opacity: 0.5;
  transform: scale(0.95);
}
```

**Remove these CSS rules:**
```css
.drag-ghost {
  /* Delete entire class definition */
  /* Includes position, transform, pointer-events, etc. */
}
```

**Pattern to follow:**
- No new CSS needed
- Selection prevention multi-layered (CSS + HTML)
- Visual feedback unchanged for user

**Consider:**
- Might find orphaned `.drag-ghost` references
- Check if any other components reference it
- CSS is scoped to this component

---

**Action 6:** Verify state management integration

**Details:**
- Confirm `isDragging` ref exists and is used
- Confirm `dragStartPos` ref exists with correct type
- Confirm `pinboardStore.setDragging()` calls are correct
- Confirm `startDrag` import from plugin

**Check imports:**
- `import { startDrag } from '@crabnebula/tauri-plugin-drag'` (should exist)
- `import { invoke } from '@tauri-apps/api/core'` (should exist)
- `import { usePinboardStore } from '@/stores/pinboards'` (should exist)

**Check refs:**
- `const isDragging = ref(false)` (add if missing)
- `const dragStartPos = ref<{ x: number; y: number } | null>(null)` (add if missing)

**Check store usage:**
- Pattern: `pinboardStore.setDragging(true, props.item.id)`
- Pattern: `pinboardStore.setDragging(false, null)`

**Consider:**
- Store contract unchanged (same method signatures)
- Plugin import might need adjustment if not present
- Type annotations important for TypeScript

---

### 4. `src/App.vue` (GLOBAL - VERIFICATION ONLY)

**Priority:** 🔵 Optional verification step

**Action:** Verify global drag prevention is still appropriate

**Details:**
- Check lines 49-58 for global `preventDefault` on drag events
- These prevent browser from opening dropped files
- Should remain unchanged (protects against external drops)
- Plugin handles drag OUT, this prevents drag IN

**Current implementation (verify correct):**
```typescript
document.addEventListener('dragover', preventDefaults, false);
document.addEventListener('dragenter', preventDefaults, false);
document.addEventListener('dragleave', preventDefaults, false);
document.addEventListener('drop', preventDefaults, false);
```

**Why this is still needed:**
- Prevents Tauri WebView from acting like browser
- Blocks file opening when user drops files INTO app
- Plugin only handles drag OUT, not IN

**Consider:**
- No changes needed if already present
- Remove only if conflicts occur (unlikely)
- Test by dropping external file into app window

---

## Testing Strategy

### Manual Testing (Required)

**Test Environment Setup:**
- macOS: Test on Finder, Desktop
- Windows: Test on Explorer, Desktop (if available)
- Both: Test all content types

**Test Plan:**

**1. Threshold Detection**
- Test: Click card without moving mouse
- Expected: No drag initiated, card selects normally
- Test: Drag < 5px
- Expected: No drag initiated
- Test: Drag > 5px
- Expected: Drag starts, native preview appears

**2. Image Drag (macOS + Windows)**
- Test: Drag image card to Finder/Explorer
- Expected: Full image file copied with readable name
- Verify: Open dropped file, should be complete image
- Verify: Filename format: `SourceApp_YYYYMMDD_HHMMSS.png`

**3. Text Drag (macOS + Windows)**
- Test: Drag text card to Desktop
- Expected: `.txt` file created
- Verify: Open file, should contain full text content
- Verify: Filename format: `SourceApp_YYYYMMDD_HHMMSS.txt`

**4. Link Drag (macOS)**
- Test: Drag link card to Finder
- Expected: `.webloc` file created
- Verify: Double-click opens URL in browser

**5. Link Drag (Windows)**
- Test: Drag link card to Desktop
- Expected: `.url` file created
- Verify: Double-click opens URL in browser

**6. Files/Audio/Documents Drag**
- Test: Drag file card to Finder/Explorer
- Expected: Original file(s) copied
- Verify: Files are complete and openable

**7. Selection Prevention**
- Test: Try to select text in card
- Expected: No text selection possible
- Test: Try to drag image inside card
- Expected: No browser drag, only card drag

**8. Preview Visual**
- Test: Start drag, observe preview
- Expected: Native OS preview (shadow on macOS, transparency on Windows)
- Expected: Preview follows cursor smoothly

**9. Drop Prevention**
- Test: Drop external file into Clipster window
- Expected: File NOT opened in app
- Expected: No navigation away from app

**10. Multiple Drags**
- Test: Drag multiple cards in sequence
- Expected: Each drag completes cleanly
- Expected: No leftover listeners or state

### Automated Testing (Future)

**Not required for initial implementation:**
- Unit tests for Rust commands
- E2E tests with Playwright
- Visual regression tests

**If added later:**
- Test file creation in temp directory
- Test platform-specific link formats
- Test threshold detection logic

---

## Documentation

**No documentation updates required for this refactor:**
- Internal implementation change only
- Public API unchanged (user still drags cards)
- No new features from user perspective

**Optional future documentation:**
- Add code comments explaining threshold logic
- Document new Rust commands for future maintainers
- Add architecture diagram showing plugin usage

---

## Rollout Considerations

**No breaking changes:**
- User-facing behavior unchanged
- Drag & drop works same way
- Visual feedback identical

**Rollback plan:**
- Revert to previous commit if issues arise
- Git history preserves old implementation
- No database migrations or data changes

**Performance considerations:**
- Simpler code should be faster
- No manual DOM manipulation overhead
- Native preview handled by OS (efficient)

**Feature flags:**
- None needed
- Direct replacement of buggy implementation

**Migration steps:**
- None for users
- Automatic on app update

---

## Risk Assessment

**Risks:**

1. **Platform-specific bugs** (MEDIUM)
   - Risk: Link files might not work on all Windows versions
   - Mitigation: Test on Windows 10 and 11
   - Fallback: Disable link drag on unsupported platforms

2. **Temp file cleanup** (LOW)
   - Risk: Temp files accumulate over time
   - Mitigation: OS handles temp directory cleanup
   - Fallback: Could add manual cleanup on app close

3. **Icon placeholders** (LOW)
   - Risk: Placeholder icon paths might not exist
   - Mitigation: Use actual icon files or fallback
   - Fallback: Plugin accepts empty icon string

4. **Race condition eliminated** (RESOLVED)
   - Old risk: HTML5 vs plugin race
   - Resolution: Single system (plugin only)

**Testing focus:**
- Test all content types on both platforms
- Test threshold detection edge cases
- Test rapid sequential drags

---

## Implementation Checklist

**Phase 1: Configuration (15 min)**
- [ ] Add `dragDropEnabled: false` to `tauri.conf.json`
- [ ] Restart Tauri dev server
- [ ] Verify no config errors

**Phase 2: Backend Commands (45 min)**
- [ ] Create `create_temp_text_file` function
- [ ] Create `create_temp_link_file` function
- [ ] Register commands in `main.rs`
- [ ] Run `cargo check` to verify compilation
- [ ] Test commands via Tauri dev tools (optional)

**Phase 3: Frontend Refactoring (1.5-2h)**
- [ ] Delete obsolete functions (lines 210-260, 437-603)
- [ ] Add new `handleMouseDown` handler
- [ ] Add new `handleMouseMove` handler
- [ ] Add new `handleMouseUp` handler
- [ ] Update `getFilePathsForDrag` with text/link support
- [ ] Update template drag bindings
- [ ] Remove `.drag-ghost` CSS class
- [ ] Verify `isDragging` and `dragStartPos` refs exist
- [ ] Run `npm run type-check` to verify TypeScript
- [ ] Run `npm run build` to verify Vue compilation

**Phase 4: Testing (1h)**
- [ ] Test threshold detection (< 5px, > 5px)
- [ ] Test image drag on macOS
- [ ] Test text drag on macOS
- [ ] Test link drag on macOS
- [ ] Test files drag on macOS
- [ ] Test image drag on Windows (if available)
- [ ] Test text drag on Windows (if available)
- [ ] Test link drag on Windows (if available)
- [ ] Test selection prevention
- [ ] Test native preview appearance
- [ ] Test drop prevention (external files)
- [ ] Test multiple sequential drags

**Phase 5: Verification**
- [ ] All tests passing
- [ ] No console errors during drag
- [ ] No race condition observed
- [ ] Code simpler and more maintainable
- [ ] Git commit with clear message

---

## Success Criteria

**Functional Requirements:**
- ✅ Drag works on macOS AND Windows
- ✅ Full file copied (verified by opening file)
- ✅ Native preview follows cursor smoothly
- ✅ No race condition (stable behavior)
- ✅ No text/image selection possible
- ✅ Threshold 5px prevents accidental drags

**Technical Requirements:**
- ✅ No HTML5 drag logic remaining
- ✅ Single drag system (plugin only)
- ✅ Simple, maintainable code (~390 lines removed)
- ✅ Proper cleanup (no listener leaks)
- ✅ Cross-platform (same code)

**Testing Requirements:**
- ✅ All content types work (image, text, link, files, audio, documents)
- ✅ Files open correctly after drop
- ✅ No browser file opening on drop into app
- ✅ Visual feedback clear (dragging state)
- ✅ No errors in console during normal operation

---

## Estimated Timeline

**Total: 3-4 hours**

| Phase | Time | Description |
|-------|------|-------------|
| Configuration | 15 min | Add `dragDropEnabled: false` |
| Rust Commands | 45 min | Create text/link file commands |
| Vue Refactoring | 1.5-2h | Remove HTML5, add simplified handlers |
| Testing | 1h | Manual testing all content types |
| **Buffer** | 30 min | Debugging unexpected issues |

**Critical path:**
Config → Backend → Frontend → Testing (strict sequential order)

---

## Notes

**Key architectural insight:**
The old implementation tried to use TWO drag systems simultaneously (HTML5 for visuals + plugin for transfer), causing a race condition. The new implementation uses ONE system (plugin only) which handles both visuals (via native preview) and transfer. This is simpler, more reliable, and cross-platform.

**Code reduction:**
- **Before:** ~1000 lines (with HTML5 drag logic)
- **After:** ~610 lines (plugin only)
- **Removed:** ~390 lines of complex clone management

**Maintainability win:**
- Easier to debug (one system vs two)
- Easier to test (fewer code paths)
- Easier to extend (add new content types)
