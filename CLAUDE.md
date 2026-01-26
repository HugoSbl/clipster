# Clipster

Cross-platform clipboard manager built with Tauri 2 + Vue 3 + TypeScript.

## Tech Stack

- **Frontend**: Vue 3 (Composition API), Pinia, Tailwind CSS, Radix Vue
- **Backend**: Tauri 2, Rust, SQLite (rusqlite)
- **Clipboard**: arboard (cross-platform), platform-specific APIs for advanced features
- **Build**: Vite, vue-tsc

## Structure

- `src/` - Vue frontend
  - `components/` - Vue components (ClipboardCard, Timeline, PinboardTabs, etc.)
  - `stores/` - Pinia stores (clipboard.ts, pinboards.ts, settings.ts)
  - `composables/` - Vue composables (useKeyboard)
  - `types/` - TypeScript types matching Rust structs
  - `App.vue` - Root component with event listeners
- `src-tauri/src/` - Rust backend
  - `clipboard/` - Clipboard monitoring and reading (platform-specific)
  - `commands/` - Tauri IPC commands (clipboard, pinboard, settings, window)
  - `models/` - Data models (ClipboardItem, Pinboard)
  - `storage/` - SQLite database and file storage for images
  - `main.rs` - App setup, tray, shortcuts, window config

## Commands

```bash
npm run tauri dev     # Development with hot-reload
npm run build         # Build frontend (vue-tsc + vite)
npm run tauri build   # Full app bundle
npx vue-tsc --noEmit  # Type check frontend
cargo check           # Check Rust compilation
```

## Tauri IPC Commands

Invoke from frontend with `invoke<ReturnType>('command_name', { args })`:

- **Clipboard**: get_clipboard, get_clipboard_history, copy_to_clipboard, delete_clipboard_item
- **Pinboards**: get_pinboards, create_pinboard, add_item_to_pinboard, remove_item_from_pinboard
- **Settings**: get_settings, update_setting, get_history_limit

## Event System

Backend emits `clipboard-changed` event when new clipboard content detected:
```typescript
listen<ClipboardChangedPayload>('clipboard-changed', (event) => {
  // event.payload.item: ClipboardItem
});
```

## Data Storage

- **Database**: `~/Library/Application Support/.clipster/clipster.db` (macOS)
- **Images**: `~/Library/Application Support/.clipster/images/`
- Content types: text, image, files, link, audio

## Key Patterns

- **Vue**: `<script setup>` with Composition API, NO Options API
- **State**: Pinia stores with typed actions and getters
- **Drag & Drop**: Hybrid approach (HTML5 + tauri-plugin-drag)
  - HTML5 dragstart: Visual feedback, dataTransfer payload
  - Plugin fallback: Actual file transfer to OS (triggered automatically)
  - Limitation: HTML5 text/uri-list doesn't create real file drops
  - macOS: Plugin required for actual files (HTML5 creates .webloc bookmarks)
  - Windows: Plugin required (WebView2 doesn't support HTML5 file drag-out)
- **Platform code**: Use `#[cfg(target_os = "...")]` for platform-specific Rust

## Drag & Drop Details

**Rust Commands for Drag**:
- `prepare_image_for_drag(source_path, filename)` → (image_path, icon_path)
- `create_temp_text_file(content, filename)` → temp_path
- `create_temp_link_file(url, filename)` → .webloc/.url path

**Key Pattern**: Event target filtering (NOT CSS pointer-events)

## ⚠️ Gotchas

- ALWAYS run `npx vue-tsc --noEmit` before committing TypeScript changes
- NEVER use `any` type - use proper types or `unknown` with type guards
- Clipboard monitoring uses polling on macOS (250ms), events on Windows
- Images use draggable="false" attribute (NOT -webkit-user-drag CSS which is WebKit-only)
- Types in `src/types/` MUST match Rust structs in `src-tauri/src/models/`
- Global shortcut: Ctrl+Shift+V toggles window visibility
