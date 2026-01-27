# HTML5 Drag and Drop Implementation Plan

## ⚠️ Critical Limitation

**HTML5 Drag and Drop API CANNOT create real file transfers from Tauri WebView to OS file managers.**

The `text/uri-list` MIME type only transfers text URLs, not file references that Finder/Explorer recognize. This means:
- **macOS**: Drops will create `.webloc` bookmark files (web links), not the actual image/file
- **Windows**: Drops will often be ignored by Explorer (WebView2 doesn't support file drag-out)
- **Expected outcome**: Visual drag will work, but files won't actually transfer

**User has chosen to proceed with this implementation anyway.**

---

## Implementation Strategy

### Hybrid Approach
- **HTML5 API**: Visual drag feedback (ghost image, cursor states)
- **Plugin fallback**: Actual file transfer (keep existing `tauri-plugin-drag`)
- **Hit-test**: Event target filtering (NOT CSS pointer-events)

This gives us the HTML5 implementation requested while maintaining working file transfers via the plugin.

---

## Phase 1: HTML5 Drag Handlers

### 1.1 Add Template Bindings

**File**: `src/components/ClipboardCard.vue` (template section)

**Changes**:
```vue
<!-- Replace current div with draggable attributes -->
<div
  class="clipboard-card"
  draggable="true"
  @dragstart="handleDragStart"
  @dragend="handleDragEnd"
  @mousedown="handleMouseDown"
>
  <!-- existing content -->
</div>
```

**Rationale**:
- `draggable="true"`: Enables HTML5 drag on the card element
- `@dragstart`: Fired when user starts dragging (HTML5 event)
- `@dragend`: Cleanup when drag completes or cancels
- Keep `@mousedown`: Still needed for plugin fallback threshold detection

---

### 1.2 Implement `handleDragStart`

**File**: `src/components/ClipboardCard.vue` (script section)

**New function** (add after line 467):
```typescript
/**
 * HTML5 dragstart handler
 * Sets dataTransfer with file URIs and custom ghost image
 */
const handleDragStart = async (e: DragEvent) => {
  console.log('[handleDragStart] HTML5 dragstart fired');
  console.log('[handleDragStart] event.target:', e.target);
  console.log('[handleDragStart] currentTarget:', e.currentTarget);

  // Hit-test filtering: Only allow drag from card container
  if (e.target !== e.currentTarget) {
    console.log('[handleDragStart] BLOCKED: Drag initiated from child element');
    e.preventDefault();
    return;
  }

  // Check if item can be dragged as files
  if (!canDragAsFiles.value) {
    console.log('[handleDragStart] Item cannot be dragged as files');
    e.preventDefault();
    return;
  }

  // Prevent if clicking on interactive elements
  const target = e.target as HTMLElement;
  if (target.closest('.visual-delete') || target.closest('.card-header')) {
    console.log('[handleDragStart] BLOCKED: Interactive element clicked');
    e.preventDefault();
    return;
  }

  isDragging.value = true;

  try {
    // Get file paths from Rust commands
    const { items, icon } = await getFilePathsForDrag();

    if (!items || items.length === 0) {
      console.error('[handleDragStart] No file paths returned');
      e.preventDefault();
      return;
    }

    console.log('[handleDragStart] File paths:', items);

    // Convert file paths to file:// URIs
    const fileUris = items.map(pathToFileUri);
    console.log('[handleDragStart] Encoded URIs:', fileUris);

    // Set dataTransfer with text/uri-list MIME type
    if (e.dataTransfer) {
      // text/uri-list requires \r\n line endings (RFC 2483)
      const uriList = fileUris.join('\r\n');
      e.dataTransfer.effectAllowed = 'copy';
      e.dataTransfer.setData('text/uri-list', uriList);
      e.dataTransfer.setData('text/plain', items[0]); // Fallback

      console.log('[handleDragStart] dataTransfer set:', uriList);

      // Create custom ghost image
      const ghostImage = await createDragImage(icon || items[0]);
      if (ghostImage) {
        e.dataTransfer.setDragImage(ghostImage, 32, 32);
        console.log('[handleDragStart] Custom ghost image set');
      }
    }

    // PLUGIN FALLBACK: Also trigger plugin for actual file transfer
    // This ensures files actually drop correctly on macOS/Windows
    setTimeout(async () => {
      try {
        await startDrag({ item: items, icon: icon || items[0] });
        console.log('[handleDragStart] Plugin fallback triggered');
      } catch (err) {
        console.error('[handleDragStart] Plugin fallback failed:', err);
      }
    }, 50); // Small delay to let HTML5 drag start first

  } catch (error) {
    console.error('[handleDragStart] Error preparing drag:', error);
    e.preventDefault();
    isDragging.value = false;
  }
};
```

**Key details**:
- Event target filtering replaces CSS pointer-events
- Checks `e.target !== e.currentTarget` to prevent child element hijacking
- Calls existing `getFilePathsForDrag()` to prepare files via Rust
- Converts paths to `file://` URIs with proper encoding
- Sets both `text/uri-list` (standard) and `text/plain` (fallback)
- Uses `\r\n` line endings per RFC 2483 spec
- Creates custom ghost image via `createDragImage()` helper
- **Hybrid strategy**: Also triggers plugin fallback for actual file transfer

---

### 1.3 Implement `handleDragEnd`

**File**: `src/components/ClipboardCard.vue` (script section)

**New function** (add after `handleDragStart`):
```typescript
/**
 * HTML5 dragend handler
 * Cleanup drag state
 */
const handleDragEnd = (e: DragEvent) => {
  console.log('[handleDragEnd] Drag ended');
  isDragging.value = false;

  // Clean up any temporary ghost images
  const existingGhost = document.querySelector('.html5-drag-ghost');
  if (existingGhost) {
    existingGhost.remove();
  }
};
```

**Rationale**: Simple cleanup handler to reset state.

---

## Phase 2: Helper Functions

### 2.1 Implement `pathToFileUri`

**File**: `src/components/ClipboardCard.vue` (script section)

**New function** (add in utilities section, around line 300):
```typescript
/**
 * Convert absolute file path to file:// URI with proper encoding
 * RFC 8089: file URI scheme
 * RFC 3986: URI percent-encoding
 */
const pathToFileUri = (absolutePath: string): string => {
  console.log('[pathToFileUri] Input path:', absolutePath);

  // Platform detection
  const isWindows = absolutePath.match(/^[A-Z]:\\/i);
  const isMac = absolutePath.startsWith('/');

  if (isWindows) {
    // Windows: file:///C:/Users/name/file.txt
    // Replace backslashes with forward slashes
    const normalized = absolutePath.replace(/\\/g, '/');

    // Encode special characters (spaces, #, ?, etc.)
    const encoded = normalized
      .split('/')
      .map(segment => encodeURIComponent(segment))
      .join('/');

    const uri = `file:///${encoded}`;
    console.log('[pathToFileUri] Windows URI:', uri);
    return uri;
  } else if (isMac) {
    // macOS: file:///Users/name/file.txt
    // Encode each path segment
    const encoded = absolutePath
      .split('/')
      .map(segment => segment ? encodeURIComponent(segment) : '')
      .join('/');

    const uri = `file://${encoded}`;
    console.log('[pathToFileUri] macOS URI:', uri);
    return uri;
  } else {
    // Fallback: assume Unix-like path
    const encoded = absolutePath
      .split('/')
      .map(segment => segment ? encodeURIComponent(segment) : '')
      .join('/');

    return `file://${encoded}`;
  }
};
```

**Key encoding rules**:
- **Windows**: `C:\Users\hugo\file.txt` → `file:///C:/Users/hugo/file.txt`
- **macOS**: `/Users/hugo/My Documents/file.txt` → `file:///Users/hugo/My%20Documents/file.txt`
- Percent-encode spaces, `#`, `?`, and other special chars per RFC 3986
- Preserve path structure (don't encode `/` separators)

---

### 2.2 Implement `createDragImage`

**File**: `src/components/ClipboardCard.vue` (script section)

**New function** (add after `pathToFileUri`):
```typescript
/**
 * Create a custom ghost image for drag operation
 * Uses the thumbnail/icon path from Rust commands
 */
const createDragImage = async (iconPath: string): Promise<HTMLElement | null> => {
  try {
    // Create a temporary div for the ghost
    const ghost = document.createElement('div');
    ghost.className = 'html5-drag-ghost';
    ghost.style.position = 'absolute';
    ghost.style.top = '-1000px'; // Off-screen
    ghost.style.left = '-1000px';
    ghost.style.width = '64px';
    ghost.style.height = '64px';
    ghost.style.borderRadius = '8px';
    ghost.style.overflow = 'hidden';
    ghost.style.backgroundColor = 'white';
    ghost.style.boxShadow = '0 4px 12px rgba(0,0,0,0.15)';

    // Create image element
    const img = document.createElement('img');
    img.src = convertFileSrc(iconPath); // Tauri asset protocol
    img.style.width = '100%';
    img.style.height = '100%';
    img.style.objectFit = 'cover';

    ghost.appendChild(img);
    document.body.appendChild(ghost);

    // Wait for image to load
    await new Promise((resolve) => {
      img.onload = resolve;
      img.onerror = resolve; // Continue even if image fails
      setTimeout(resolve, 100); // Timeout fallback
    });

    return ghost;
  } catch (error) {
    console.error('[createDragImage] Failed to create ghost:', error);
    return null;
  }
};
```

**Rationale**:
- Creates 64x64 thumbnail preview
- Uses off-screen positioning to avoid visual glitches
- Loads icon via Tauri's `convertFileSrc()` for security
- Graceful fallback if image fails to load

---

## Phase 3: CSS Changes

### 3.1 Remove Pointer-Events Rules

**File**: `src/components/ClipboardCard.vue` (style section)

**Remove** (lines 648-660):
```css
/* DELETE THIS BLOCK */
.clipboard-card * {
  pointer-events: none;
  user-select: none;
}

.clipboard-card .visual-delete,
.clipboard-card .card-header {
  pointer-events: auto;
}
```

**Rationale**: Event target filtering in JavaScript replaces CSS pointer-events approach.

---

### 3.2 Add Ghost Image Styles

**File**: `src/components/ClipboardCard.vue` (style section)

**Add** (in style block):
```css
/* HTML5 drag ghost image */
.html5-drag-ghost {
  pointer-events: none;
  z-index: 9999;
}
```

**Rationale**: Ensures ghost image doesn't interfere with drag detection.

---

## Phase 4: Import Statements

### 4.1 Add Tauri Imports

**File**: `src/components/ClipboardCard.vue` (script imports)

**Add** (if not already present):
```typescript
import { convertFileSrc } from '@tauri-apps/api/core';
```

**Rationale**: Needed for `createDragImage()` to load icon paths safely.

---

## Phase 5: Keep Plugin Handlers (Fallback)

### 5.1 Preserve Existing Functions

**Do NOT delete**:
- `handleMouseDown` (lines 409-419)
- `handleMouseMove` (lines 421-431)
- `handleMouseUp` (lines 433-439)
- `initiateDrag` (lines 441-467)
- `getFilePathsForDrag` (lines 315-384)

**Rationale**:
- `getFilePathsForDrag()` is reused by `handleDragStart()`
- Plugin handlers serve as fallback for actual file transfer
- Threshold detection prevents accidental drags

---

### 5.2 Update `initiateDrag`

**File**: `src/components/ClipboardCard.vue`

**Modify** `initiateDrag` (line 441):
```typescript
const initiateDrag = async () => {
  // This function is now called as a fallback from handleDragStart
  // It handles the actual file transfer via native plugin
  isDragging.value = true;

  try {
    const { items, icon } = await getFilePathsForDrag();

    if (!items || items.length === 0) {
      console.error('[initiateDrag] Empty file paths, skipping plugin drag');
      isDragging.value = false;
      return;
    }

    console.log('[initiateDrag] Calling startDrag plugin...');
    await startDrag({
      item: items,
      icon: icon || items[0],
    });

    console.log('[initiateDrag] Plugin drag completed');
  } catch (error) {
    console.error('[initiateDrag] startDrag failed:', error);
    isDragging.value = false;
  }
};
```

**Rationale**: Clarify that this is the plugin fallback path.

---

## Phase 6: Testing Plan

### 6.1 Manual Testing Steps

**Test 1: HTML5 dragstart fires**
```
1. Open app in dev mode: npm run tauri dev
2. Click and drag a clipboard card
3. Check browser console for: "[handleDragStart] HTML5 dragstart fired"
4. Verify: Event target filtering works (can't grab child elements)
```

**Test 2: Ghost image appears**
```
1. Drag an image card
2. Expected: 64x64 thumbnail follows cursor
3. Check: Ghost persists during entire drag
4. Verify: Ghost disappears on drop (handleDragEnd cleanup)
```

**Test 3: File URI encoding**
```
1. Drag card with spaces in filename (e.g., "My Image.png")
2. Check console: "[pathToFileUri] macOS URI: file:///path/My%20Image.png"
3. Expected: Spaces encoded as %20, slashes preserved
```

**Test 4: dataTransfer payload**
```
1. Drag image card to Desktop
2. Drop it
3. macOS Expected: Creates .webloc file (bookmark, not actual image)
4. Windows Expected: Drop ignored or creates text file
5. LIMITATION CONFIRMED: HTML5 doesn't transfer actual files
```

**Test 5: Plugin fallback works**
```
1. Drag image card to Desktop
2. Drop it
3. Expected: Actual PNG/JPG file appears (plugin fallback succeeded)
4. Verify: File is complete and openable
```

**Test 6: Hit-test filtering**
```
1. Try to drag by clicking directly on image element
2. Expected: Drag blocked (console: "BLOCKED: Drag initiated from child element")
3. Try dragging from card background
4. Expected: Drag succeeds
```

---

### 6.2 Browser DevTools Inspection

**Check dataTransfer contents**:
```javascript
// Add to handleDragStart for debugging
console.log('dataTransfer types:', Array.from(e.dataTransfer.types));
console.log('text/uri-list:', e.dataTransfer.getData('text/uri-list'));
console.log('text/plain:', e.dataTransfer.getData('text/plain'));
```

**Expected output**:
```
dataTransfer types: ["text/uri-list", "text/plain"]
text/uri-list: file:///Users/hugo/Library/Application%20Support/.clipster/Image_20260123.png
text/plain: /Users/hugo/Library/Application Support/.clipster/Image_20260123.png
```

---

### 6.3 Cross-Platform Testing

**macOS**:
- ✅ HTML5 dragstart fires
- ✅ Ghost image appears
- ⚠️ Drop creates .webloc (bookmark file, not actual image)
- ✅ Plugin fallback copies actual file

**Windows** (if available):
- ✅ HTML5 dragstart fires
- ⚠️ Ghost image may be browser default (WebView2 limitation)
- ⚠️ Drop ignored by Explorer (WebView2 doesn't support file drag-out)
- ✅ Plugin fallback copies actual file

---

## Phase 7: Documentation

### 7.1 Add Code Comments

**File**: `src/components/ClipboardCard.vue`

**Add comment block** (at top of drag section):
```typescript
// ============================================================================
// DRAG & DROP IMPLEMENTATION (HYBRID APPROACH)
// ============================================================================
//
// This component uses a HYBRID drag-drop strategy:
//
// 1. HTML5 Drag API (handleDragStart/handleDragEnd):
//    - Provides visual feedback (ghost image, cursor states)
//    - Sets dataTransfer with text/uri-list MIME type
//    - LIMITATION: Cannot create real file drops from WebView to OS
//    - Result: macOS creates .webloc bookmarks, Windows often ignores
//
// 2. Native Plugin Fallback (startDrag):
//    - Uses tauri-plugin-drag for actual file transfer
//    - Triggered automatically 50ms after HTML5 drag starts
//    - Result: Real files copied to drop location
//
// Why both?
// - HTML5 gives us the dragstart event and custom ghost image
// - Plugin gives us actual working file transfers
// - Together they provide the complete experience
//
// See: .claude/tasks/05-html5-drag-implementation/ for detailed analysis
// ============================================================================
```

---

### 7.2 Update CLAUDE.md

**File**: `CLAUDE.md`

**Add section**:
```markdown
## Drag & Drop

- **Internal**: HTML5 Drag API for reordering items within app
- **External**: Hybrid approach (HTML5 + tauri-plugin-drag)
  - HTML5 dragstart: Visual feedback, dataTransfer payload
  - Plugin fallback: Actual file transfer to OS (triggered automatically)
  - Limitation: HTML5 text/uri-list doesn't create real file drops
  - macOS: Plugin required for actual files (HTML5 creates .webloc bookmarks)
  - Windows: Plugin required (WebView2 doesn't support HTML5 file drag-out)

**Rust Commands for Drag**:
- `prepare_image_for_drag(source_path, filename)` → (image_path, icon_path)
- `create_temp_text_file(content, filename)` → temp_path
- `create_temp_link_file(url, filename)` → .webloc/.url path

**Key Pattern**: Event target filtering (NOT CSS pointer-events)
```

---

## Phase 8: Error Handling

### 8.1 Add Try-Catch Blocks

Already included in `handleDragStart` implementation (Phase 1.2).

---

### 8.2 User Feedback (Optional Enhancement)

**Future consideration** (not in initial implementation):
```typescript
// Add visual feedback if drag fails
if (error) {
  // Show toast notification: "Drag failed, please try again"
}
```

---

## Success Criteria

✅ **HTML5 dragstart fires** when user drags a card
✅ **Event target filtering** prevents child elements from hijacking drag
✅ **Custom ghost image** appears and persists during drag
✅ **File URIs** properly encoded with `file://` prefix and percent-encoding
✅ **dataTransfer** contains text/uri-list with `\r\n` line endings
⚠️ **HTML5 drop behavior**: Creates .webloc on macOS, ignored on Windows (expected)
✅ **Plugin fallback**: Actual files transfer correctly via native drag
✅ **No bugs**: Hit-test issues resolved, ghost doesn't disappear

---

## Implementation Order

1. **Add imports** (Phase 4): `convertFileSrc`
2. **Add helper functions** (Phase 2): `pathToFileUri`, `createDragImage`
3. **Add event handlers** (Phase 1): `handleDragStart`, `handleDragEnd`
4. **Update template** (Phase 1.1): Add `draggable="true"` and `@dragstart/@dragend`
5. **Modify CSS** (Phase 3): Remove pointer-events, add ghost styles
6. **Update existing handlers** (Phase 5): Clarify plugin fallback role
7. **Add documentation** (Phase 7): Comments and CLAUDE.md updates
8. **Test** (Phase 6): Manual testing on macOS, Windows if available

---

## Estimated Complexity

- **Lines added**: ~200 (handlers + helpers + comments)
- **Lines removed**: ~15 (pointer-events CSS)
- **Files modified**: 2 (ClipboardCard.vue, CLAUDE.md)
- **Risk level**: LOW (existing plugin fallback prevents breakage)
- **Reversibility**: HIGH (git revert restores plugin-only approach)

---

## Notes for Implementation

1. **Keep console.log statements**: Helpful for debugging during testing
2. **Test on macOS first**: Primary development platform
3. **Plugin must stay**: Remove it and file drops will stop working
4. **Hybrid is necessary**: HTML5 alone cannot transfer files from WebView
5. **User chose this approach**: Despite knowing HTML5 limitations

---

## Alternative Approaches Rejected

❌ **Pure HTML5 only**: Would break file transfers entirely
❌ **Pure plugin only**: Already works, but user requested HTML5
❌ **CSS pointer-events**: User requested event.target filtering instead

**Chosen**: Hybrid (HTML5 visuals + plugin fallback) per user requirements.
