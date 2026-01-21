# Task: Implement Windows Source App Icon Extraction

## Problem

Once we detect the source application, we need to extract its icon to display in the clipboard card UI. Windows provides Shell APIs to get application icons from executables.

## Proposed Solution

Implement icon extraction for Windows:
- Use SHGetFileInfo() with SHGFI_ICON | SHGFI_LARGEICON flags
- Convert HICON to PNG bytes (via GDI+ or manual bitmap extraction)
- Encode as base64 string
- Add Win32_UI_Shell feature to Cargo.toml
- Cache icons by app path to avoid repeated extraction
- Return None gracefully on failure

## Dependencies

- Task 4: Windows source app detection provides the exe path

## Context

- get_source_app_icon() should be called after get_source_app()
- Store result in source_app_icon field
- Icon size: 32x32 or 48x48 for quality
- Convert to 16x16 PNG for storage efficiency

## Success Criteria

- Chrome icon is extracted and stored as base64 PNG
- Various app icons work (VSCode, Notepad, etc.)
- System apps return valid icons
- Invalid paths return None without crashing
- Icons are reasonably sized (< 10KB base64)
