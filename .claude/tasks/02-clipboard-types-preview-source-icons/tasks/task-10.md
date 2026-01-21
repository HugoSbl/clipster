# Task: Implement Shell Thumbnail Cache (Windows)

## Problem

Windows also needs file thumbnails for copied files. The Shell Thumbnail Cache (IThumbnailCache) provides previews for many file types, similar to Quick Look on macOS.

## Proposed Solution

Implement thumbnail generation for Windows:
- Use IThumbnailCache COM interface or SHGetFileInfo
- Generate thumbnails for copied file paths
- Convert to PNG bytes
- Store in thumbnail_base64 field
- Fallback to file type icon (via SHGetFileInfo with SHGFI_ICON)

## Dependencies

- None (Windows file reading already works)

## Context

- Windows implementation section: `clipboard_reader.rs:31-213`
- Shell APIs: Win32_UI_Shell
- IThumbnailCache or simpler SHGetFileInfo approach
- Match thumbnail size with macOS (200px max)

## Success Criteria

- Image files show actual thumbnail
- Documents show preview when available
- Unknown types show file type icon
- Icons are properly converted to PNG
- No COM initialization issues
