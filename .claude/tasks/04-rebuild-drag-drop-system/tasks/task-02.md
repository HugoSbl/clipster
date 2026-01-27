# Task 2: Add Backend Commands for Text and Link File Creation

## Problem

Currently, only images can be dragged out of Clipster to external apps. When users try to drag text or links, there's no backend command to create temporary files that can be transferred via the drag & drop plugin.

The plugin requires file paths to transfer - for text and links, we need to create temporary `.txt` and `.webloc`/`.url` files on-demand.

## Proposed Solution

Create two new Tauri commands in the Rust backend:

1. **`create_temp_text_file`**: Creates a temporary `.txt` file with the provided content
2. **`create_temp_link_file`**: Creates platform-specific link files (`.webloc` for macOS, `.url` for Windows)

Both commands should:
- Accept content and filename as parameters
- Create files in `std::env::temp_dir()`
- Return the absolute path to the created file
- Remove macOS quarantine xattr if applicable

## Dependencies

- **Task 1**: Tauri configuration must be updated first
- External: Existing `prepare_image_for_drag` command as reference pattern (`clipboard_commands.rs:150-233`)

## Context

**Files to modify:**
- `src-tauri/src/commands/clipboard_commands.rs` - Add new commands
- `src-tauri/src/main.rs` - Register commands in `.invoke_handler()`

**Reference pattern:**
The existing `prepare_image_for_drag` command shows the pattern:
```rust
#[tauri::command]
pub fn prepare_image_for_drag(
    source_path: String,
    readable_filename: String,
) -> Result<(String, String), String>
```

**Platform-specific file formats:**
- macOS links: `.webloc` (plist XML format)
- Windows links: `.url` (INI format)
- Use `#[cfg(target_os = "...")]` for platform code

**Security considerations:**
- Validate filename doesn't contain path traversal (../)
- Remove quarantine xattr on macOS (com.apple.quarantine)
- Files in temp dir are auto-cleaned by OS

## Success Criteria

- `create_temp_text_file` command successfully creates `.txt` files
- `create_temp_link_file` command creates `.webloc` (macOS) or `.url` (Windows)
- Both commands return absolute file paths
- Commands are registered in `main.rs`
- Rust compilation succeeds (`cargo check`)
- Frontend can successfully invoke both commands via IPC
