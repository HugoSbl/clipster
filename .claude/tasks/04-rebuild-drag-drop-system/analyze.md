# Analysis: Rebuild Drag & Drop System from Scratch

**Analyzed**: 2026-01-23
**Status**: Complete
**Source**: Brainstorm phase (seed.md)

## Quick Summary (TL;DR)

> Refonte complète du système de drag & drop pour éliminer la race condition entre HTML5 et `tauri-plugin-drag`. Approche ultra-simple: mousedown → threshold 5px → `startDrag()` uniquement, pas de clone HTML manuel.

**Strategy used:**
- Code: 5/6 → Deep (already explored in brainstorm)
- Web:  0/6 → Skip (Context7 docs only)
- Docs: 5/6 → Explored (Tauri official docs)

**Key files to modify:**
- `src/components/ClipboardCard.vue` (MAJEUR) - Supprimer lignes 210-260, 437-603, simplifier handlers
- `src-tauri/tauri.conf.json` (CONFIG) - Ajouter `dragDropEnabled: false`
- `src-tauri/src/commands/clipboard_commands.rs` (BACKEND) - Créer 2 nouvelles commandes
- `src/App.vue` (MINEUR) - Vérifier global drag prevention

**Patterns to follow:**
- Vue Composition API strict (`<script setup lang="ts">`)
- Threshold detection pattern (5px, keep existing logic)
- Tauri plugin API: `startDrag({ item: [path], icon: iconPath })`

**⚠️ Gotchas discovered:**
1. **Race condition**: HTML5 `@dragstart` vs `startDrag()` - Must eliminate HTML5 completely
2. **Config naming**: `dragDropEnabled: false` means "enable plugin use" (confusing but official)
3. **Icon parameter**: Preview image, not the actual file being transferred
4. **Temp files**: Text/Link types need new Rust commands to create temp files

**Dependencies:**
- `tauri-plugin-drag@2.1.0` already installed
- Rust `image` crate already available
- No blocking dependencies

**Estimation:** ~4 tasks, ~3-4h total
- Config: 15 min
- Rust commands: 45 min
- Vue refactoring: 1.5-2h
- Testing: 1h

---

## Context from Brainstorm

### Objectif principal

Reconstruire le système de drag & drop **from scratch** en utilisant **uniquement** `tauri-plugin-drag` avec l'approche la plus simple et logique recommandée par Tauri.

**Comportements attendus:**
1. Drag dans l'app puis vers le système (Finder, Desktop, Explorer)
2. Seule la card est draggable (pas les éléments internes)
3. Transfert du fichier réel (pas juste l'icône)
4. Cross-platform (macOS + Windows) avec même code

### Décisions utilisateur confirmées

1. **Approche**: Refonte from scratch (supprimer HTML5, UNIQUEMENT plugin)
2. **Preview**: Native du plugin (pas de clone HTML manuel)
3. **Threshold**: 5px detection (garder le système actuel)

### Architecture actuelle (problématique)

**Système dual buggy:**
```
mousedown → threshold (5px)
  ├─ HTML5: createExactClone() + manual positioning
  └─ Native: startDrag() after threshold
      ↓
  Race condition: HTML5 peut gagner avant startDrag()
```

**Problèmes identifiés:**
- Race condition entre HTML5 `@dragstart` et `startDrag()`
- Clone manuel complexe avec `transform: translate3d()`
- Listeners multiples difficiles à cleanup
- Dual-mode confond les deux systèmes

### Architecture cible (simple)

**Système unique:**
```
mousedown → threshold (5px) → startDrag() UNIQUEMENT
                               ↓
                      Plugin gère preview native
                      OS gère drag & drop
```

**Avantages:**
- ✅ Pas de race condition (un seul système)
- ✅ Preview native gérée par l'OS
- ✅ Cleanup simple
- ✅ Cross-platform automatique

---

## Codebase Context

### Files Analysis

**1. `src/components/ClipboardCard.vue`** (PRIMARY)

**Current implementation (to remove):**
- Lines 210-260: `createExactClone()` - Manual DOM cloning with GPU optimization
- Lines 437-478: `handleNativeDragStart()` - Records position, creates clone
- Lines 480-550: `handleNativeDragMove()` - Threshold detection + `startDrag()` call
- Lines 552-559: `handleNativeDragEnd()` - Cleanup
- Lines 579-589: `updateClonePosition()` - Manual positioning with `translate3d()`
- Lines 592-603: `handleImageDragStart()`, `handleImageDrag()` - Browser drag prevention

**To keep:**
- Lines 287-327: `prepareImageForDrag()` - Rust command call for images (GOOD)
- Lines 329-418: `getFilePathsForDrag()` - File path extraction (GOOD)
- Lines 420-430: `canDragAsFiles` computed - Drag eligibility check (GOOD)
- Lines 841-853: CSS `user-select: none` - Selection prevention (GOOD)

**Refactoring strategy:**
- Remove all HTML5 drag logic (lines 210-260, 437-603)
- Simplify to: mousedown → threshold → `startDrag()` only
- Keep threshold detection pattern (5px)
- Keep state management (`isDragging` ref)

**2. `src-tauri/tauri.conf.json`** (CONFIGURATION)

**Current state:**
- No `dragDropEnabled` config (defaults to `true`)
- This enables Tauri's internal drag/drop system
- Causes conflicts with `tauri-plugin-drag`

**Required change:**
```json
{
  "app": {
    "windows": [{
      "dragDropEnabled": false  // Disables internal, enables plugin use
    }]
  }
}
```

**3. `src-tauri/src/commands/clipboard_commands.rs`** (BACKEND)

**Existing command (lines 150-233):**
- `prepare_image_for_drag(source_path, readable_filename)` → `(imagePath, iconPath)`
- ✅ Copies image to temp with readable name
- ✅ Creates 64x64 thumbnail
- ✅ Removes macOS quarantine xattr
- ✅ Returns both paths

**Missing commands (to create):**
```rust
// For text content
fn create_temp_text_file(content: String, filename: String) -> Result<String, String>

// For link content
fn create_temp_link_file(url: String, filename: String) -> Result<String, String>
```

**4. `src/App.vue`** (GLOBAL)

**Current implementation (lines 49-58):**
```typescript
// Prevent default browser behavior for drag and drop
document.addEventListener('dragover', preventDefaults, false);
document.addEventListener('dragenter', preventDefaults, false);
document.addEventListener('dragleave', preventDefaults, false);
document.addEventListener('drop', preventDefaults, false);
```

**Status:** Already correct, may need adjustment after refactor.

---

## Documentation Insights

### Tauri Plugin API

**`@crabnebula/tauri-plugin-drag@2.1.0`**

Official community plugin by CrabNebula (Tauri maintainers).

**API:**
```typescript
import { startDrag } from '@crabnebula/tauri-plugin-drag';

interface StartDragOptions {
  item: string[];   // Array of absolute file paths
  icon: string;     // Absolute path to icon (32x32 or 64x64 PNG)
}

await startDrag(options);
```

**Key findings from docs:**
- ✅ Cross-platform: macOS, Windows, Linux (GTK)
- ✅ Preview native automatique via `icon` parameter
- ✅ OS handles ghost image rendering
- ⚠️ Do NOT mix with HTML5 drag API
- ⚠️ Must set `dragDropEnabled: false` in config

**Platform-specific preview:**
- macOS: Shows icon with shadow
- Windows: Shows icon with semi-transparency
- Linux: GTK handles rendering

### Configuration gotcha

**Confusing naming:**
- `dragDropEnabled: true` (default) → Tauri internal drag enabled, plugin BLOCKED
- `dragDropEnabled: false` → Tauri internal disabled, plugin WORKS

This is counter-intuitive but official behavior.

---

## Content Type Handling

### Types et préparation fichiers

**ContentType enum:**
```typescript
type ContentType =
  | 'text'      // → Create .txt temp file
  | 'image'     // → Copy image to temp (existing command)
  | 'files'     // → Pass paths directly
  | 'link'      // → Create .webloc (macOS) or .url (Windows)
  | 'audio'     // → Pass paths directly
  | 'documents' // → Pass paths directly
```

**Preparation workflow:**

1. **Images** (existing):
   ```typescript
   const [imagePath, iconPath] = await invoke<[string, string]>(
     'prepare_image_for_drag',
     { sourcePath: item.image_path, readableFilename }
   );
   ```

2. **Text** (to create):
   ```typescript
   const textPath = await invoke<string>(
     'create_temp_text_file',
     { content: item.content_text, filename: `${sourceApp}_${timestamp}.txt` }
   );
   const iconPath = '/default/text-icon.png'; // Static icon
   ```

3. **Link** (to create):
   ```typescript
   const linkPath = await invoke<string>(
     'create_temp_link_file',
     { url: item.content_text, filename: `${sourceApp}_${timestamp}.webloc` }
   );
   const iconPath = '/default/link-icon.png'; // Static icon
   ```

4. **Files/Audio/Documents** (no prep):
   ```typescript
   const paths = JSON.parse(item.content_text) as string[];
   const iconPath = '/default/file-icon.png'; // Static icon for preview
   // Pass paths directly to startDrag()
   ```

---

## Implementation Strategy

### Phase 1: Configuration (15 min)

**File:** `src-tauri/tauri.conf.json`

Add to window config:
```json
"dragDropEnabled": false
```

**Why:** Disables Tauri's internal drag/drop, enables plugin usage.

### Phase 2: Rust Commands (45 min)

**File:** `src-tauri/src/commands/clipboard_commands.rs`

**Command 1: Text file creation**
```rust
#[tauri::command]
pub fn create_temp_text_file(
    content: String,
    filename: String,
) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(&filename);

    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write text file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}
```

**Command 2: Link file creation**
```rust
#[tauri::command]
pub fn create_temp_link_file(
    url: String,
    filename: String,
) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();

    #[cfg(target_os = "macos")]
    {
        // .webloc format for macOS
        let file_path = temp_dir.join(&filename);
        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>URL</key>
    <string>{}</string>
</dict>
</plist>"#,
            url
        );
        std::fs::write(&file_path, plist_content)
            .map_err(|e| format!("Failed to write webloc: {}", e))?;
        Ok(file_path.to_string_lossy().to_string())
    }

    #[cfg(target_os = "windows")]
    {
        // .url format for Windows
        let file_path = temp_dir.join(&filename);
        let ini_content = format!("[InternetShortcut]\nURL={}\n", url);
        std::fs::write(&file_path, ini_content)
            .map_err(|e| format!("Failed to write url file: {}", e))?;
        Ok(file_path.to_string_lossy().to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("Link files not supported on this platform".to_string())
    }
}
```

**Register commands in `main.rs`:**
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    create_temp_text_file,
    create_temp_link_file,
])
```

### Phase 3: Vue Refactoring (1.5-2h)

**File:** `src/components/ClipboardCard.vue`

**Step 3.1: Remove old code**
- Delete `createExactClone()` function (lines 210-260)
- Delete `handleNativeDragStart()` (lines 437-478)
- Delete `handleNativeDragMove()` (lines 480-550)
- Delete `handleNativeDragEnd()` (lines 552-559)
- Delete `updateClonePosition()` (lines 579-589)
- Delete `handleImageDragStart()`, `handleImageDrag()` (lines 592-603)

**Step 3.2: Add new simplified handlers**

```typescript
// Constants
const DRAG_THRESHOLD = 5; // pixels

// State
const isDragging = ref(false);
const dragStartPos = ref<{ x: number; y: number } | null>(null);

// Mousedown handler
const handleMouseDown = (e: MouseEvent) => {
  dragStartPos.value = { x: e.clientX, y: e.clientY };
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
};

// Mousemove handler - threshold detection
const handleMouseMove = async (e: MouseEvent) => {
  if (!dragStartPos.value) return;

  const dx = Math.abs(e.clientX - dragStartPos.value.x);
  const dy = Math.abs(e.clientY - dragStartPos.value.y);

  if (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD) {
    // Threshold exceeded - remove listeners
    document.removeEventListener('mousemove', handleMouseMove);
    dragStartPos.value = null;

    // Set dragging state
    isDragging.value = true;
    pinboardStore.setDragging(true, props.item.id);

    // Prepare file and start drag
    try {
      const { items, icon } = await getFilePathsForDrag();

      if (items.length === 0) {
        console.error('No files to drag');
        return;
      }

      await startDrag({ item: items, icon });
    } catch (error) {
      console.error('[ClipboardCard] Drag failed:', error);
    }
  }
};

// Mouseup handler - cleanup
const handleMouseUp = () => {
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);
  dragStartPos.value = null;
  isDragging.value = false;
  pinboardStore.setDragging(false, null);
};
```

**Step 3.3: Update `getFilePathsForDrag()`**

Add support for text and link types:

```typescript
async function getFilePathsForDrag(): Promise<{ items: string[]; icon: string }> {
  const { item } = props;

  try {
    switch (item.content_type) {
      case 'image': {
        const sourceApp = item.source_app || 'Clipster';
        const timestamp = new Date().toISOString().replace(/[:.]/g, '').slice(0, 15);
        const readableFilename = `${sourceApp}_${timestamp}.png`;

        const [imagePath, iconPath] = await invoke<[string, string]>(
          'prepare_image_for_drag',
          { sourcePath: item.image_path, readableFilename }
        );

        return { items: [imagePath], icon: iconPath };
      }

      case 'text': {
        const sourceApp = item.source_app || 'Clipster';
        const timestamp = new Date().toISOString().replace(/[:.]/g, '').slice(0, 15);
        const filename = `${sourceApp}_${timestamp}.txt`;

        const textPath = await invoke<string>(
          'create_temp_text_file',
          { content: item.content_text || '', filename }
        );

        return { items: [textPath], icon: '/default/text-icon.png' };
      }

      case 'link': {
        const sourceApp = item.source_app || 'Clipster';
        const timestamp = new Date().toISOString().replace(/[:.]/g, '').slice(0, 15);
        const ext = process.platform === 'darwin' ? 'webloc' : 'url';
        const filename = `${sourceApp}_${timestamp}.${ext}`;

        const linkPath = await invoke<string>(
          'create_temp_link_file',
          { url: item.content_text || '', filename }
        );

        return { items: [linkPath], icon: '/default/link-icon.png' };
      }

      case 'files':
      case 'audio':
      case 'documents': {
        const paths = getFilePaths();
        return { items: paths || [], icon: '/default/file-icon.png' };
      }

      default:
        return { items: [], icon: '' };
    }
  } catch (error) {
    console.error('[ClipboardCard] File preparation failed:', error);
    return { items: [], icon: '' };
  }
}
```

**Step 3.4: Update template**

Replace drag handlers:
```vue
<div
  class="clipboard-card"
  :class="{ dragging: isDragging }"
  @mousedown="handleMouseDown"
>
  <!-- Card content -->
</div>
```

Remove `@dragstart` handlers from images (no longer needed).

**Step 3.5: Update CSS**

Keep existing:
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

Remove:
```css
.drag-ghost {
  /* Delete entire class - no longer used */
}
```

### Phase 4: Testing (1h)

**macOS tests:**
1. Drag image to Finder → Verify full file copied
2. Drag text to Desktop → Verify .txt file created
3. Drag link to Finder → Verify .webloc file created
4. Drag document to Desktop → Verify original file copied

**Windows tests:**
1. Drag image to Explorer → Verify full file copied
2. Drag text to Desktop → Verify .txt file created
3. Drag link to Desktop → Verify .url file created
4. Drag document to Explorer → Verify original file copied

**Functional tests:**
1. Click card without drag → No drag initiated
2. Drag < 5px → No drag initiated
3. Drag > 5px → Drag starts, preview follows cursor
4. Text/image not selectable → Verify with mouse
5. Preview native → Verify OS-specific styling

---

## Patterns to Follow

### Vue Composition API

**Correct pattern:**
```typescript
const isDragging = ref(false);
const dragStartPos = ref<{ x: number; y: number } | null>(null);

const handleMouseDown = (e: MouseEvent) => {
  dragStartPos.value = { x: e.clientX, y: e.clientY };
};
```

**Incorrect pattern (avoid):**
```typescript
data() {
  return { isDragging: false };
}
```

### Threshold Detection

**Keep existing pattern:**
```typescript
const DRAG_THRESHOLD = 5;

if (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD) {
  // Start drag
}
```

### Error Handling

**Always wrap Tauri commands:**
```typescript
try {
  const result = await invoke('command_name', { args });
} catch (error) {
  console.error('[Component] Operation failed:', error);
}
```

### Store Usage

**Correct:**
```typescript
import { usePinboardStore } from '@/stores/pinboards';
const pinboardStore = usePinboardStore();
pinboardStore.setDragging(true, itemId);
```

---

## Dependencies

**Existing (no changes needed):**
- `@crabnebula/tauri-plugin-drag@2.1.0` - Already installed
- `tauri-plugin-drag = "2.1.0"` - Already in Cargo.toml
- `image = "0.24"` - Already available
- Vue 3 + TypeScript - Already configured
- Pinia stores - Already implemented

**No blocking dependencies.**

---

## Success Criteria

**Functional:**
- ✅ Drag works on macOS AND Windows
- ✅ Full file copied (not just icon/thumbnail)
- ✅ Native preview follows cursor smoothly
- ✅ No race condition (stable behavior)
- ✅ No text/image selection possible
- ✅ Threshold 5px prevents accidental drags

**Technical:**
- ✅ No HTML5 drag logic remaining
- ✅ Single drag system (plugin only)
- ✅ Simple, maintainable code
- ✅ Proper cleanup (no listener leaks)
- ✅ Cross-platform (same code)

**Testing:**
- ✅ All content types work (image, text, link, files)
- ✅ File opens correctly after drop
- ✅ No browser file opening on drop into app
- ✅ Visual feedback clear (dragging state)
