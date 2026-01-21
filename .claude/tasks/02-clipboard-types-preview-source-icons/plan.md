# Implementation Plan: Clipboard Types, Preview & Source App Detection

## Overview

Transform Clipster into a Paste-like overlay application with:
1. **macOS file list support** via NSPasteboard (arboard doesn't support this)
2. **Source app detection** on both Windows and macOS (with Accessibility API)
3. **Quick Look thumbnails** for file previews
4. **Full-width overlay window** (~100% width, ~33% height, adaptive to screen)
5. **Enlarged UI** with bigger cards (280x220px) and improved spacing

**Architecture approach**: Platform-specific implementations wrapped in unified Rust API, with adaptive window sizing via Tauri's monitor APIs.

## Dependencies

Files must be modified in this order due to dependencies:
1. Cargo.toml (add dependencies first)
2. Backend Rust modules (clipboard reader → monitor → models)
3. Frontend types (clipboard.ts)
4. Frontend components (ClipboardCard.vue → Timeline.vue → App.vue)
5. Tauri config (window settings last)

---

## File Changes

### `src-tauri/Cargo.toml`

- Add macOS-specific dependencies for NSPasteboard file reading:
  ```
  [target.'cfg(target_os = "macos")'.dependencies]
  objc2 = "0.5"
  objc2-app-kit = { version = "0.2", features = ["NSPasteboard", "NSPasteboardItem"] }
  objc2-foundation = { version = "0.2", features = ["NSString", "NSArray", "NSURL"] }
  ```
- Add macOS Quick Look thumbnail generation:
  ```
  core-graphics = "0.24"
  ```
- Add macOS Accessibility API for frontmost app detection (via cocoa or accessibility crate)
- Add Windows features to existing `windows` crate for source app:
  ```
  "Win32_UI_Shell"           # SHGetFileInfo for icons
  "Win32_System_ProcessStatus" # GetModuleFileNameEx
  "Win32_System_Threading"    # OpenProcess
  ```

### `src-tauri/src/clipboard/clipboard_reader.rs`

**macOS file list support (lines 277-284 currently returns None):**
- Implement `read_files()` for macOS using NSPasteboard directly
- Access `NSPasteboard::generalPasteboard()`
- Read `public.file-url` UTI type from pasteboard
- Convert NSURL array to Vec<String> file paths
- Update `detect_format()` to check for files before text (follow Windows order at line 52-62)
- Pattern to follow: Windows implementation structure at lines 166-171

**Update `read_clipboard()` for macOS (lines 287-299):**
- Add file check after image check (currently only checks image and text)
- Call new `read_files()` and return `ClipboardContent::Files` if available

### `src-tauri/src/clipboard/clipboard_monitor.rs`

**Source app detection (implement `get_source_app()` at line 169-172):**

For Windows:
- Use `GetClipboardOwner()` to get HWND of clipboard owner
- Call `GetWindowThreadProcessId()` to get process ID from HWND
- Use `OpenProcess()` + `GetModuleFileNameExW()` to get executable path
- Extract app name from path (e.g., "chrome.exe" → "Chrome")
- Consider: Handle edge cases where clipboard owner window is destroyed

For macOS:
- Track frontmost application when clipboard changes (poll-based monitor)
- Use NSWorkspace's `frontmostApplication` property
- Get `localizedName` from NSRunningApplication
- Consider: Background copy scenarios may not have frontmost app as source
- Fallback: Return None if unable to determine

**Icon extraction (new function `get_source_app_icon()`):**

For Windows:
- Use `SHGetFileInfo()` with `SHGFI_ICON | SHGFI_LARGEICON` flags
- Convert HICON to PNG bytes using GDI+ or winapi
- Return as base64-encoded string

For macOS:
- Use `NSRunningApplication.icon` property
- Convert NSImage to PNG data
- Return as base64-encoded string

**Update process methods:**
- In `process_text()` (line 71-89): Pass source app icon to ClipboardItem
- In `process_image()` (line 92-131): Pass source app icon to ClipboardItem
- In `process_files()` (line 134-149): Pass source app icon to ClipboardItem

### `src-tauri/src/models/clipboard_item.rs`

**Add new field to ClipboardItem struct (after line 124):**
- Add `source_app_icon: Option<String>` field for base64-encoded icon

**Update constructors:**
- `new_text()` (line 140-153): Add `source_app_icon` parameter
- `new_link()` (line 156-168): Add `source_app_icon` parameter
- `new_image()` (line 171-187): Add `source_app_icon` parameter
- `new_files()` (line 190-204): Add `source_app_icon` parameter
- `new_audio()` (line 207-220): Add `source_app_icon` parameter

**Update `from_row()` (line 233-250):**
- Add `source_app_icon: row.get("source_app_icon")?` field

### `src-tauri/src/storage/database.rs`

**Update database schema:**
- Add `source_app_icon TEXT` column to clipboard_items table
- Run migration to add column to existing databases
- Consider: Default to NULL for existing items

**Update insert_item():**
- Include `source_app_icon` in INSERT statement

### `src-tauri/src/storage/file_storage.rs`

**Add Quick Look thumbnail generation for files:**

For macOS:
- Create `generate_file_thumbnail_macos(path: &Path) -> Option<Vec<u8>>`
- Use `QLThumbnailGenerator` to generate preview
- Convert CGImage to PNG bytes
- Fallback to file type icon if Quick Look fails

For Windows:
- Create `generate_file_thumbnail_windows(path: &Path) -> Option<Vec<u8>>`
- Use `IThumbnailCache` Shell interface
- Or use `SHGetFileInfo()` for file type icons
- Convert to PNG bytes

**Add file type icon fallback:**
- Create icon mapping for common extensions (pdf, doc, xls, etc.)
- Embed default file icon as fallback

### `src/types/clipboard.ts`

**Update ClipboardItem interface (line 11-21):**
- Add `source_app_icon: string | null;` field after `source_app`
- Update JSDoc comment to reflect new field

### `src/components/ClipboardCard.vue`

**Card dimensions (line 246-261):**
- Change width from `180px` to `280px`
- Change height from `140px` to `220px`
- Update border-radius proportionally (8px → 12px)

**Thumbnail size (line 414-419):**
- Change `max-height: 70px` to `max-height: 120px`
- Update placeholder dimensions accordingly

**Card header (line 148-168):**
- Add source app icon display before type icon
- Conditional render: show icon only if `item.source_app_icon` exists
- Display as 16x16 image with rounded corners
- Keep type icon as fallback/secondary indicator

**Type icons (line 327-373):**
- Increase icon size from 20px to 24px for better visibility

**Font sizes:**
- Increase `.text-preview` font-size from 12px to 14px
- Increase `.timestamp` font-size from 10px to 11px
- Increase `.file-count`, `.audio-count` from 12px to 14px

**File content section (line 192-201):**
- Add thumbnail display for files if `thumbnail_base64` exists
- Show file icon as fallback
- Display first file thumbnail with count overlay

**Padding and spacing:**
- Increase `.card-header` padding from `8px 10px 4px` to `10px 12px 6px`
- Increase `.card-content` padding from `4px 10px` to `6px 12px`
- Increase `.card-actions` padding from `4px 8px 8px` to `6px 10px 10px`

### `src/components/Timeline.vue`

**Timeline gap:**
- Increase gap between cards from 12px to 16px

**Scroll container:**
- Ensure horizontal scroll still works with larger cards
- Update any hardcoded dimensions

### `src-tauri/tauri.conf.json`

**Window configuration (line 13-22):**
- Remove fixed width/height values
- Set `resizable: false`
- Keep `alwaysOnTop: true`
- Keep `decorations: false` or minimal
- Add `transparent: true` for overlay effect

**Note**: Actual adaptive sizing will be handled via Tauri's monitor API at runtime (in main.rs or a setup hook).

### `src-tauri/src/main.rs` (or new `src/window.rs`)

**Add adaptive window sizing:**
- On app startup, query primary monitor dimensions via `tauri::Monitor`
- Calculate: width = monitor.width, height = monitor.height * 0.33
- Position window at bottom of screen (y = monitor.height * 0.67)
- Apply dimensions with `window.set_size()` and `window.set_position()`
- Consider: Handle multi-monitor setups

---

## Testing Strategy

**Tests to create:**
- `src-tauri/src/clipboard/clipboard_reader_tests.rs`: Test macOS file reading
- Test source app detection mocking

**Manual verification:**
1. Copy files from Finder on macOS → verify they appear in history
2. Copy from Chrome, VSCode, etc. → verify source app name appears
3. Verify source app icons display correctly
4. Test Quick Look thumbnails for PDF, images, documents
5. Test window appears as full-width overlay at screen bottom
6. Test card sizing and layout with new dimensions
7. Test on different screen sizes/resolutions

---

## Documentation

**Update `CLAUDE.md`:**
- Add new clipboard module structure
- Document Quick Look and Accessibility API requirements
- Note macOS permissions needed (Accessibility for source app)

---

## Rollout Considerations

**macOS permissions:**
- Accessibility permission required for source app detection
- Add `NSAppleEventsUsageDescription` to Info.plist
- Graceful fallback: If permission denied, source_app remains None

**Database migration:**
- New `source_app_icon` column must be nullable
- Existing items will have NULL for new field

**Incremental implementation order:**
1. Phase 1: UI enlargement (lowest risk, immediate visual improvement)
2. Phase 2: macOS file list support (core functionality)
3. Phase 3: Source app detection Windows (straightforward)
4. Phase 4: Quick Look thumbnails (file previews)
5. Phase 5: Source app detection macOS (requires permissions)
6. Phase 6: Adaptive window sizing (polish)

**Breaking changes:** None - all additions are backward compatible.
