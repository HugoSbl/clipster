# Clipster Windows

Clipboard manager for Windows built with Tauri 2 + Vue 3 + TypeScript.

## Tech Stack

| Layer | Technology |
|-------|------------|
| Frontend | Vue 3 (Composition API), Pinia, Tailwind CSS, Radix Vue |
| Backend | Tauri 2, Rust, Windows API |
| Build | Vite, vue-tsc |

## Project Structure

```
src/                    # Vue frontend
├── assets/             # CSS (tailwind.css), SVG
├── components/         # Vue components
│   └── display-paste/  # Clipboard display components
├── App.vue             # Root component
└── main.ts             # Vue entry point

src-tauri/              # Rust backend
├── src/
│   ├── main.rs         # Tauri entry point
│   ├── lib.rs          # Library exports
│   ├── commands/       # Tauri commands (IPC)
│   │   └── clipboard_commands.rs
│   └── windows_api/    # Windows API wrappers
│       └── windows_api.rs
└── Cargo.toml
```

## Commands

```bash
# Development
npm run tauri dev

# Build
npm run build              # Frontend only
npm run tauri build        # Full app bundle

# Type check
vue-tsc --noEmit
```

## Tauri Commands (IPC)

| Command | Description | File |
|---------|-------------|------|
| `get_clipboard` | Get clipboard text content | `clipboard_commands.rs:5` |

Frontend usage:
```typescript
import { invoke } from "@tauri-apps/api/core";
const text = await invoke("get_clipboard");
```

## Windows API

Clipboard access uses Windows crate with `CF_UNICODETEXT` format.

Key functions in `windows_api.rs`:
- `get_clipboard_datas()` - Read clipboard text (UTF-16 conversion)
- `list_clipboard_elements()` - Debug: list all clipboard formats

## Conventions

- Vue: `<script setup>` with Composition API
- State: Pinia stores (not yet implemented)
- Styling: Tailwind CSS utilities
- Rust: Module pattern with `mod.rs` exports
- Comments: French (legacy), prefer English for new code

## Window Config

- Size: 800x600, non-resizable
- Always on top, transparent background
- No decorations styling (custom titlebar possible)
