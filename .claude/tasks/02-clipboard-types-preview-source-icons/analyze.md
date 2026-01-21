# Analysis: Clipboard Types, Preview & Source App Detection

**Analyzed**: 2026-01-21
**Status**: Complete

## Quick Summary (TL;DR)

> **Objectif**: Améliorer le clipboard manager avec prévisualisation d'images/fichiers, bandeaux colorés par type, détection de l'app source, et agrandissement de l'interface.

**Strategy used:**
- Code: 5/6 → Deep (2 explore-codebase agents)
- Web:  5/6 → intelligent-search (2 agents)
- Docs: 4/6 → explore-docs

**Key files to modify:**

| File | Purpose |
|------|---------|
| `src-tauri/src/clipboard/clipboard_reader.rs` | Add macOS file list support, source app detection |
| `src-tauri/src/clipboard/clipboard_monitor.rs:169-171` | Implement `get_source_app()` |
| `src-tauri/src/models/clipboard_item.rs` | Already has `source_app` field (unused) |
| `src/components/ClipboardCard.vue:246-291` | Resize cards, improve type banners |
| `src/components/Timeline.vue` | Adjust layout for larger cards |
| `src-tauri/tauri.conf.json:16-17` | Window size 800x600 → larger |
| `src/types/clipboard.ts` | Add source_app_icon field |

**Patterns to follow:**
- DIB→PNG conversion pattern: `clipboard_reader.rs:144-190`
- Type detection logic: `clipboard_item.rs:40-84`
- Color system: Blue(text), Purple(image), Green(files), Orange(link), Pink(audio)

**⚠️ Gotchas discovered:**
- macOS: `arboard` does NOT support file lists - need custom NSPasteboard code
- macOS source app detection: No native API - requires Accessibility API workaround
- Windows: GetClipboardOwner() returns HWND, need process API for app name/icon
- Image preview on macOS works (arboard), but files preview doesn't

**Dependencies:** None blocking - all capabilities can be added incrementally

**Estimation:** ~8-12 tasks, implementation complexity varies by feature

---

## 1. Current Implementation Status

### Content Types (Already Implemented)

| Type | Backend | Frontend Display | Copy Back |
|------|---------|------------------|-----------|
| Text | ✅ Full | ✅ 80-char preview | ✅ Works |
| Link | ✅ Auto-detect | ✅ Domain extraction | ✅ Works |
| Image | ✅ PNG/DIB | ✅ Thumbnail 70px | ❌ TODO |
| Files | ⚠️ Windows only | ✅ Count + first name | ❌ TODO |
| Audio | ✅ Extension detect | ✅ Count + filename | ❌ TODO |

### Key Code Locations

**Backend (Rust):**
- Content type enum: `clipboard_item.rs:9-15`
- Format detection: `clipboard_item.rs:40-84`
- Windows clipboard reader: `clipboard_reader.rs:31-213`
- macOS clipboard reader: `clipboard_reader.rs:218-314`
- Image processing: `clipboard_monitor.rs:91-130`
- Thumbnail generation: `file_storage.rs:158-186` (200px max)

**Frontend (Vue):**
- Card component: `ClipboardCard.vue:1-663`
- Card dimensions: `180px × 140px` (lines 246-261)
- Type color bands: `ClipboardCard.vue:264-291`
- Timeline container: `Timeline.vue:239-459`

---

## 2. Image Preview Issues on macOS

### Current Behavior
- Windows: CF_DIB/CF_DIBV5 → converts to PNG → thumbnail generated ✅
- macOS: arboard `get_image()` → RGBA bytes → PNG encoding ✅

### Root Cause of Issues
The macOS implementation works correctly in `clipboard_reader.rs:279-314`:
```rust
pub fn read_image() -> Option<ImageData> {
    let mut clipboard = Clipboard::new().ok()?;
    let img_data = clipboard.get_image().ok()?;
    // ... converts RGBA to PNG
}
```

**Potential issue**: If image preview doesn't work on Mac, check:
1. Thumbnail generation path (file_storage.rs)
2. Base64 encoding for frontend
3. Frontend image display logic

---

## 3. File Preview Implementation

### Windows (Implemented)
Uses `CF_HDROP` format via `clipboard-win` crate:
```rust
// clipboard_reader.rs - Windows reads file paths
pub fn read_files() -> Option<Vec<String>> {
    let files = clipboard_win::get_clipboard(clipboard_win::formats::FileList)?;
    Some(files)
}
```

### macOS (NOT Implemented)
`arboard` does NOT support file lists. Need custom NSPasteboard code:

```rust
// REQUIRED: New macOS file reading
use cocoa::appkit::NSPasteboard;
use cocoa::foundation::{NSArray, NSString};

pub fn read_files_macos() -> Option<Vec<String>> {
    unsafe {
        let pasteboard = NSPasteboard::generalPasteboard(nil);
        let file_type = NSString::alloc(nil).init_str("public.file-url");

        if let Some(urls) = pasteboard.propertyListForType_(file_type) {
            // Parse URL array to file paths
        }
    }
}
```

### Document Snapshots
For file thumbnails:
- **macOS**: Quick Look API (`QLThumbnailGenerator`)
- **Windows**: Shell Thumbnail Cache (`IThumbnailCache`)
- **Fallback**: File extension icons

---

## 4. Source App Detection

### Windows (Feasible ⭐⭐)

```rust
use windows::Win32::System::DataExchange::{GetClipboardOwner, OpenClipboard, CloseClipboard};
use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;

pub fn get_source_app_windows() -> Option<String> {
    unsafe {
        OpenClipboard(None).ok()?;
        let owner_hwnd = GetClipboardOwner();
        CloseClipboard();

        if owner_hwnd.0 == 0 { return None; }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(owner_hwnd, Some(&mut pid));

        // Open process and get exe path
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).ok()?;
        let mut path = vec![0u16; 260];
        let len = GetModuleFileNameExW(handle, None, &mut path);

        path.truncate(len as usize);
        Some(String::from_utf16_lossy(&path).split('\\').last()?.to_string())
    }
}
```

**Icon extraction**: Use `ExtractIconExW()` or `SHGetFileInfo()` for app icon.

### macOS (Complex ⭐⭐⭐⭐)

NSPasteboard does **NOT** expose source app. Workarounds:

1. **Accessibility API** (requires permission):
   - Track frontmost app when clipboard changes
   - Not 100% reliable (background copy scenarios)

2. **Implementation approach**:
```rust
use accessibility::{AXUIElement, attribute};

fn get_frontmost_app() -> Option<String> {
    let system_wide = AXUIElement::system_wide();
    let app = system_wide.attribute(&attribute::FocusedApplication)?;
    app.attribute(&attribute::Title)
}
```

**Requirements**:
- `NSAppleEventsUsageDescription` in Info.plist
- User must grant Accessibility permissions

### Recommendation
1. Implement Windows source detection first (straightforward)
2. macOS as "nice-to-have" (complexity vs benefit)

---

## 5. UI Enlargement Plan

### Current Dimensions
- Window: 800×600px
- Cards: 180×140px
- Timeline gap: 12px
- Font sizes: 14px base, 10px timestamps

### Proposed Changes

| Element | Current | Proposed | Notes |
|---------|---------|----------|-------|
| Window | 800×600 | 1200×800 | ~50% screen coverage |
| Cards | 180×140 | 280×220 | More space for info |
| Thumbnail | 70px max | 120px max | Better image preview |
| Timeline gap | 12px | 16px | Breathing room |
| Font base | 14px | 16px | Better readability |

### Files to Modify

1. `tauri.conf.json`:
```json
{
  "width": 1200,
  "height": 800,
  "resizable": true  // Allow resizing?
}
```

2. `ClipboardCard.vue`:
```css
.card {
  width: 280px;
  height: 220px;
}
```

3. `Timeline.vue`:
```css
.timeline-track {
  gap: 16px;
}
```

---

## 6. Color Band System (Already Implemented)

Current implementation in `ClipboardCard.vue:264-291`:

| Type | Gradient | Hex Values |
|------|----------|------------|
| Text | Blue | `#3b82f6` → `#60a5fa` |
| Image | Purple | `#8b5cf6` → `#a78bfa` |
| Files | Green | `#22c55e` → `#4ade80` |
| Link | Orange | `#f97316` → `#fb923c` |
| Audio | Pink | `#ec4899` → `#f472b6` |

**No changes needed** - system is already well-designed.

---

## 7. Implementation Priorities

### Phase 1: Fix macOS File Support (High Priority)
1. Implement `read_files_macos()` using NSPasteboard
2. Test file path extraction
3. Verify existing thumbnail generation works

### Phase 2: Source App Detection
1. Windows: Implement `get_source_app_windows()`
2. Add icon extraction (Windows)
3. Store source_app in database (field exists)
4. Display icon in card header

### Phase 3: UI Enlargement
1. Update window size in tauri.conf.json
2. Resize card component
3. Adjust thumbnail display size
4. Update timeline spacing

### Phase 4: macOS Source App (Optional)
1. Add Accessibility API integration
2. Request permissions
3. Track frontmost app on clipboard change

---

## 8. Crates to Add

```toml
# For macOS file list support
[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
objc = "0.2"

# For Windows source app + icon
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
    "Win32_UI_Shell",           # SHGetFileInfo for icons
    "Win32_System_ProcessStatus", # GetModuleFileNameEx
] }
```

---

## 9. TypeScript Type Updates

```typescript
// src/types/clipboard.ts
export interface ClipboardItem {
  id: string;
  content_type: ContentType;
  content_text: string | null;
  thumbnail_base64: string | null;
  image_path: string | null;
  source_app: string | null;          // App name (e.g., "Chrome")
  source_app_icon: string | null;     // NEW: Base64 icon
  created_at: string;
  pinboard_id: string | null;
  is_favorite: boolean;
}
```

---

## 10. User Clarifications Needed

Before implementation, clarify:

1. **Window size**: Exact dimensions for "half screen" target?
2. **macOS priority**: Is source app detection essential or nice-to-have?
3. **Document preview**: Should we generate actual thumbnails or just show file icons?
4. **Resizable window**: Should window be resizable or fixed size?

---

## Key Files Reference

| File | Lines | Purpose |
|------|-------|---------|
| `clipboard_reader.rs` | 31-314 | Clipboard reading (Windows + macOS) |
| `clipboard_monitor.rs` | 169-171 | `get_source_app()` TODO |
| `clipboard_item.rs` | 9-15 | ContentType enum |
| `ClipboardCard.vue` | 246-291 | Card styling + type bands |
| `Timeline.vue` | 327-365 | Scroll container |
| `tauri.conf.json` | 16-21 | Window configuration |
| `clipboard.ts` | 11-21 | TypeScript interfaces |
