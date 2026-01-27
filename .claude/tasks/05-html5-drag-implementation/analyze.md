# Analysis: HTML5 Drag-and-Drop Implementation for File Export

**Analyzed**: 2026-01-24
**Status**: Complete

## Quick Summary (TL;DR)

> **CRITICAL FINDING**: HTML5 Drag and Drop API **cannot** create real file drops from Tauri WebView to OS file managers. The `text/uri-list` MIME type only transfers text URLs, not file references that Finder/Explorer recognize. Your current plugin-based implementation is the correct solution.
>
> **User Choice**: Proceed with HTML5 implementation anyway, understanding it will only work for text/URL transfers, not actual file drops.

**Strategy used:**
- Code: 6/6 → Deep (2 agents - current implementation + Rust backend)
- Web:  4/6 → intelligent-search (HTML5 limitations research)
- Docs: 5/6 → explore-docs (Tauri drag APIs)

**Key files to modify:**
- `src/components/ClipboardCard.vue:350-411` - Replace plugin-only with HTML5 dragstart
- `src/components/ClipboardCard.vue:648-660` - Modify CSS pointer-events strategy
- `src-tauri/tauri.conf.json:25` - Set dragDropEnabled: false (enable HTML5)

**Current Architecture:**
- ✅ Plugin-only approach (no HTML5 drag API)
- ✅ Hit-test fix via `pointer-events: none` on children
- ✅ Cross-platform file preparation (text, links, images)
- ✅ Native OS drag via `tauri-plugin-drag`

**⚠️ Critical Constraint:**
HTML5 `dataTransfer.setData('text/uri-list', 'file://...')` creates TEXT transfers, not FILE transfers. When dropped in Finder/Explorer:
- **macOS**: Creates `.webloc` bookmark file (not the actual image/document)
- **Windows**: Often ignored entirely by Explorer
- **Why**: WebView sandbox prevents JavaScript from creating file system references

**Patterns to follow:**
- Event.target filtering pattern (user choice vs pointer-events CSS)
- File URI encoding: `file:///path` (macOS), `file:///C:/path` (Windows)
- Percent-encoding: spaces = `%20`, special chars per RFC 3986
- Line endings: `\r\n` for multiple URIs in text/uri-list

**⚠️ Gotchas discovered:**
- HTML5 Drag won't actually transfer files (only text/URLs)
- Must call `preventDefault()` on dragover for drop zone
- `setDragImage()` must use existing DOM element (not created on-the-fly)
- Windows WebView2 has known drag-out bugs (file icon disappears)
- `text/uri-list` format requires `\r\n` line endings (not just `\n`)

**Dependencies:**
- Remove dependency on `@crabnebula/tauri-plugin-drag` if going pure HTML5
- Keep Rust file preparation commands (still needed)
- Set `dragDropEnabled: false` in tauri.conf.json

**Estimation:** ~3 tasks, ~4-6h total

---

## Codebase Context

### Current Implementation Analysis

**Path**: `src/components/ClipboardCard.vue`

**Architecture**: Plugin-only approach (clean, no HTML5 Drag API)

**Flow**:
```
mousedown (line 409)
  → threshold detection (5px, line 421-431)
  → initiateDrag (line 441-467)
  → getFilePathsForDrag (line 315-384) [async file prep]
  → startDrag() from @crabnebula/tauri-plugin-drag (line 458-461)
```

**Key Handlers**:

1. **handleMouseDown** (line 409-419)
   - Guards: `canDragAsFiles.value` (line 410), left-click only (line 411)
   - Records position for threshold (line 416)
   - Attaches listeners: mousemove, mouseup (lines 417-418)

2. **handleMouseMove** (line 421-431)
   - Threshold: 5px horizontal or vertical (line 427)
   - Removes own listener after threshold (line 428)
   - Calls `initiateDrag()` async (line 429)

3. **handleMouseUp** (line 433-439)
   - Cleanup: removes listeners (lines 434-435)
   - Resets state (lines 436-438)

4. **initiateDrag** (line 441-467)
   - Sets dragging state (lines 442-443)
   - Calls `getFilePathsForDrag()` async (line 446)
   - Validates paths (line 448-454)
   - **Single plugin call**: `startDrag({ item, icon })` (line 458-461)
   - Error handling in catch block (line 462-464)
   - Cleanup in finally (line 465-466)

**File Preparation System** (`getFilePathsForDrag` - lines 315-384):

- **Images**: Calls Rust `prepare_image_for_drag` → returns `(imagePath, iconPath)`
- **Text**: Calls Rust `create_temp_text_file` → returns `.txt` file path
- **Links**: Calls Rust `create_temp_link_file` → returns `.webloc` (macOS) or `.url` (Windows)
- **Files/Audio/Documents**: Parses JSON from `content_text` → returns absolute paths

**CSS Hit-Test Fix** (lines 648-660):
```css
.clipboard-card * {
  pointer-events: none;  /* Prevents children from intercepting clicks */
}

.clipboard-card .visual-delete,
.clipboard-card .card-header {
  pointer-events: auto;  /* Re-enable for interactive elements */
}
```

**Visual Feedback** (lines 676-680):
```css
.clipboard-card.dragging {
  opacity: 0.5;
  transform: scale(0.95);
}
```

### Rust Backend File Handling

**Path**: `src-tauri/src/commands/clipboard_commands.rs`

**Commands**:

1. **prepare_image_for_drag** (lines 150-233)
   - Copies image to temp dir with readable filename
   - Creates 64x64 thumbnail PNG for drag icon
   - Removes macOS quarantine xattr
   - Returns: `(String, String)` - full image path + icon path

2. **create_temp_text_file** (lines 235-274)
   - Validates filename (no path traversal)
   - Auto-appends `.txt` extension
   - Writes content to temp dir
   - Returns: `String` - absolute file path

3. **create_temp_link_file** (lines 276-357)
   - **macOS** (lines 292-326): Creates `.webloc` plist XML
   - **Windows** (lines 328-350): Creates `.url` INI format
   - Returns: `String` - absolute file path

**File Path Format**:
- Returns absolute paths: `/var/folders/...` (macOS), `C:\Users\...` (Windows)
- No `file://` prefix in returns
- Spaces/special chars preserved (not URL-encoded)

**Platform-Specific Code**:
```rust
#[cfg(target_os = "macos")]
{
    // macOS: .webloc file (plist XML)
    let plist_content = format!(r#"<?xml version="1.0"...>
    <key>URL</key>
    <string>{}</string>..."#, url);
}

#[cfg(target_os = "windows")]
{
    // Windows: .url file (INI)
    let ini_content = format!("[InternetShortcut]\r\nURL={}\r\n", url);
}
```

### Plugin Configuration

**Path**: `src-tauri/tauri.conf.json:25`
```json
{
  "dragDropEnabled": false  // ✅ Already configured correctly
}
```

**Path**: `src-tauri/Cargo.toml:39`
```toml
tauri-plugin-drag = "2.1.0"  // ✅ Plugin installed
```

**Path**: `src-tauri/src/main.rs:61`
```rust
.plugin(tauri_plugin_drag::init())  // ✅ Plugin registered
```

---

## Research Findings (Web + Docs)

### HTML5 Drag API Fundamental Limitation

**Discovery from intelligent-search agent**:

> HTML5 Drag and Drop API has **fundamental limitations** in Tauri WebView environments for outbound file transfers. The core issue is that **WebView sandboxing** prevents JavaScript from creating true file drops that OS file managers recognize.

**Key Technical Findings**:

1. **text/uri-list Does NOT Work for Files**:
   ```typescript
   // ❌ This does NOT create a file drop
   event.dataTransfer.setData('text/uri-list', 'file:///path/to/image.png');

   // Result when dropped in Finder:
   // - macOS: Creates a .webloc bookmark file (not the image!)
   // - Windows: Often just ignored by Explorer
   ```

2. **Why It Fails**:
   - WebView security model blocks JavaScript from file system operations
   - `text/uri-list` transfers text URLs, not file references
   - Windows Explorer expects `CF_HDROP` format (binary structure)
   - macOS Finder expects `NSFilenamesPboardType` on pasteboard
   - HTML5 API cannot create these native clipboard formats

3. **setDragImage() Works (Visual Only)**:
   - Function works reliably for visual feedback
   - But underlying file transfer still fails
   - You get a nice ghost image, but drop won't work

4. **WebView2 Known Issues** (Windows):
   - Documented limitations in Microsoft WebView2 Feedback
   - Chromium security model prevents web content file operations
   - File icon on cursor disappears when dragging into app window

### File URI Encoding Standards (RFC 8089)

**macOS/Linux**:
```
File path:  /Users/hugo/My Documents/file name.txt
File URI:   file:///Users/hugo/My%20Documents/file%20name.txt
```

**Windows**:
```
File path:  C:\Users\hugo\file name.txt
File URI:   file:///C:/Users/hugo/file%20name.txt
```

**Encoding Rules** (RFC 3986):
- Space = `%20`
- Reserved chars: `: / ? # [ ] @ ! $ & ' ( ) * + , ; =` → percent-encode
- Safe chars: `A-Z a-z 0-9 - _ . ~` → no encoding needed

**text/uri-list Format**:
```
file:///path/to/file1.txt\r\nfile:///path/to/file2.png\r\n
```
- Each URI on separate line
- **Must** use `\r\n` (carriage return + line feed), not just `\n`
- Trailing `\r\n` required

### Native Plugin vs HTML5 Comparison

**tauri-plugin-drag (Current)**:
- ✅ Real file drops recognized by OS
- ✅ Uses platform APIs: `NSDraggingSession` (macOS), `IDataObject` (Windows)
- ✅ Creates proper clipboard formats: `CF_HDROP`, `NSFilenamesPboardType`
- ✅ Bypasses WebView sandbox entirely
- ✅ Cross-platform abstraction handled internally

**HTML5 Drag API**:
- ❌ Cannot create file system references from JavaScript
- ❌ `text/uri-list` only transfers text, not files
- ❌ Windows Explorer doesn't recognize WebView drag-out
- ✅ Works for text/URL transfers between apps
- ✅ Fine-grained control over drag events

---

## User Clarifications

**Q: Do you want to improve the existing plugin implementation OR attempt HTML5 despite limitations?**
**A: Attempt HTML5 implementation anyway**

**Q: Keep current pointer-events CSS approach or try event.target filtering?**
**A: Try event.target filtering instead**

**Q: For Windows, stick with plugin (works now) or add Rust fallback command?**
**A: Stick with plugin (works now)**

---

## Key Files to Modify

### 1. `src/components/ClipboardCard.vue`

**Changes Required**:

**A. Replace Plugin Drag with HTML5 (lines 350-467)**:

Remove:
- `handleMouseDown`, `handleMouseMove`, `handleMouseUp`, `initiateDrag`
- `import { startDrag } from '@crabnebula/tauri-plugin-drag'`

Add:
- `handleDragStart` event handler
- OS detection logic (macOS vs Windows)
- File URI encoding function
- `setDragImage()` implementation

**B. Modify Hit-Test Prevention (lines 648-660)**:

Remove:
```css
.clipboard-card * {
  pointer-events: none;
}
```

Add:
- Event target filtering in `handleDragStart`
- Check `event.target === cardRef.value`

**C. Template Changes (lines 416-490)**:

Add:
```vue
<div
  draggable="true"
  @dragstart="handleDragStart"
  @dragend="handleDragEnd"
  ...
>
```

### 2. `src-tauri/tauri.conf.json`

**Verify** (line 25):
```json
{
  "dragDropEnabled": false  // ✅ Required for HTML5 drag
}
```

### 3. `src-tauri/Cargo.toml` (Optional)

**If removing plugin**:
```toml
# Remove or comment out:
# tauri-plugin-drag = "2.1.0"
```

### 4. `src-tauri/src/main.rs` (Optional)

**If removing plugin** (line 61):
```rust
// Remove or comment out:
// .plugin(tauri_plugin_drag::init())
```

**Keep Rust commands** (lines 9-14):
- `prepare_image_for_drag` - Still needed for file prep
- `create_temp_text_file` - Still needed
- `create_temp_link_file` - Still needed

---

## Implementation Patterns

### Pattern 1: HTML5 dragstart Handler

```typescript
const handleDragStart = async (event: DragEvent) => {
  // 1. Target filtering (user's choice)
  if (event.target !== cardRef.value) {
    event.preventDefault();
    return;
  }

  // 2. Prepare files
  const { items, icon } = await getFilePathsForDrag();
  if (items.length === 0) return;

  // 3. OS detection
  const isWindows = navigator.platform.startsWith('Win');

  // 4. Convert paths to URIs
  const uris = items.map(path => pathToFileUri(path, isWindows)).join('\r\n');

  // 5. Set data transfer
  event.dataTransfer!.effectAllowed = 'copy';
  event.dataTransfer!.setData('text/uri-list', uris + '\r\n');
  event.dataTransfer!.setData('text/plain', items[0]); // Fallback

  // 6. Set drag image
  const dragImage = createDragImage(icon);
  event.dataTransfer!.setDragImage(dragImage, 0, 0);

  // 7. Visual feedback
  isDragging.value = true;
  pinboardStore.setDragging(true, props.item.id);
};
```

### Pattern 2: File Path to URI Conversion

```typescript
function pathToFileUri(path: string, isWindows: boolean): string {
  // Normalize slashes
  const normalized = path.replace(/\\/g, '/');

  // Percent-encode special characters
  const parts = normalized.split('/');
  const encoded = parts.map(part => {
    return part.split('').map(char => {
      // Safe characters (unreserved)
      if (/[A-Za-z0-9\-_.~]/.test(char)) return char;
      // Space
      if (char === ' ') return '%20';
      // Everything else
      return encodeURIComponent(char);
    }).join('');
  }).join('/');

  // Format based on OS
  if (isWindows) {
    // Windows: file:///C:/path
    return `file:///${encoded}`;
  } else {
    // macOS/Linux: file:///path (already starts with /)
    return `file://${encoded}`;
  }
}
```

### Pattern 3: Drag Image Creation

```typescript
function createDragImage(iconPath: string): HTMLImageElement {
  const img = new Image();
  img.src = iconPath || '/default-icon.png';

  // Ensure image is loaded
  img.style.position = 'absolute';
  img.style.top = '-9999px';
  document.body.appendChild(img);

  // Cleanup after drag
  setTimeout(() => {
    document.body.removeChild(img);
  }, 1000);

  return img;
}
```

### Pattern 4: Event Target Filtering

```typescript
const handleDragStart = (event: DragEvent) => {
  // Only allow drag if user grabbed the card directly
  // Not if they grabbed an image, text, or button inside
  if (event.target !== cardRef.value) {
    event.preventDefault();
    return false;
  }

  // Continue with drag logic...
};
```

---

## Testing Strategy

### ⚠️ Expected Behavior with HTML5

**What WILL work**:
- ✅ Drag visual feedback (ghost image)
- ✅ Cursor changes appropriately
- ✅ Text/URL transfers between apps
- ✅ Custom drag image via `setDragImage()`

**What WON'T work**:
- ❌ File drops into Finder (creates .webloc instead)
- ❌ File drops into Windows Explorer (often ignored)
- ❌ Actual file copy/paste operations
- ❌ Native file promises

### Test Cases

**1. Text Transfer (Should Work)**:
- Drag text card to TextEdit/Notepad
- Expected: Text content pasted ✅
- Reason: Text transfer works in HTML5

**2. Image Drop to Finder (Won't Work as Expected)**:
- Drag image card to macOS Finder
- Expected HTML5 result: `.webloc` bookmark created ❌
- Expected Plugin result: Actual `.png` file copied ✅

**3. Image Drop to Windows Explorer (Won't Work)**:
- Drag image card to Windows Explorer
- Expected: Often ignored entirely ❌
- Reason: WebView2 doesn't support HTML5 drag-out for files

**4. Ghost Image Visual (Should Work)**:
- Start dragging any card
- Expected: Custom drag image appears and follows cursor ✅

**5. Hit-Test Prevention (Event Filtering)**:
- Click directly on `<img>` inside card
- Expected: Drag prevented (event.target !== cardRef) ✅
- Click on card background
- Expected: Drag starts ✅

---

## Alternative Hybrid Approach (Recommendation)

Since HTML5 cannot transfer files but you want to use it, consider:

**Hybrid Pattern**:
```typescript
const handleDragStart = async (event: DragEvent) => {
  // 1. Use HTML5 for visual feedback
  const dragImage = createDragImage(icon);
  event.dataTransfer!.setDragImage(dragImage, 0, 0);

  // 2. Set text/URI data (won't work for files, but try anyway)
  const uris = items.map(path => pathToFileUri(path)).join('\r\n');
  event.dataTransfer!.setData('text/uri-list', uris + '\r\n');

  // 3. For actual file transfer, fall back to plugin
  // This requires mixing both systems
  setTimeout(async () => {
    await startDrag({ item: items, icon });
  }, 100);
};
```

**Pros**:
- Custom drag image from HTML5
- Actual file transfer from plugin

**Cons**:
- Complex dual-system approach
- Timing issues between HTML5 and plugin
- Same race condition as before

---

## Conclusion

**Technical Reality**:
HTML5 Drag and Drop API cannot create file drops from Tauri WebView to OS file managers due to WebView sandbox security. The `text/uri-list` MIME type only works for text/URL transfers, not file references.

**User Choice**:
Proceed with HTML5 implementation understanding it will work for text/URLs but not actual file drops.

**Recommended Next Steps**:
1. Implement HTML5 dragstart handler with URI encoding
2. Test text/URL transfers (will work)
3. Document that file drops won't work as expected
4. Consider keeping plugin as fallback for file operations
5. Update user expectations in UI (e.g., "Copy URL" vs "Copy File")

**Success Criteria**:
- ✅ HTML5 dragstart handler implemented
- ✅ File URI encoding per platform (macOS/Windows)
- ✅ setDragImage() for custom ghost
- ✅ Event target filtering for hit-test
- ⚠️ File drops won't work (acknowledged limitation)
- ✅ Text/URL transfers work correctly

---

## Sources

**Codebase**:
- `src/components/ClipboardCard.vue:1-1169` - Current implementation
- `src-tauri/src/commands/clipboard_commands.rs:150-357` - File preparation commands
- `src-tauri/tauri.conf.json:25` - dragDropEnabled setting

**Research**:
- [HTML5 Drag API Limitations in Tauri](https://github.com/tauri-apps/tauri/discussions/9696)
- [WebView2 Drag-out Issues](https://github.com/MicrosoftEdge/WebView2Feedback/issues/1815)
- [RFC 8089 - File URI Scheme](https://www.rfc-editor.org/rfc/rfc8089.html)
- [MDN DataTransfer.setData()](https://developer.mozilla.org/en-US/docs/Web/API/DataTransfer/setData)
- [MDN File Drag and Drop](https://developer.mozilla.org/en-US/docs/Web/API/HTML_Drag_and_Drop_API/File_drag_and_drop)

**Documentation**:
- [@crabnebula/tauri-plugin-drag](https://www.npmjs.com/package/@crabnebula/tauri-plugin-drag)
- [Tauri v2 Window Customization](https://v2.tauri.app/learn/window-customization/)
