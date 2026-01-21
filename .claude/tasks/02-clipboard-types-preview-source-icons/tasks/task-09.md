# Task: Implement Quick Look File Thumbnails (macOS)

## Problem

When files are copied to the clipboard, we only show a generic file icon. macOS Quick Look can generate actual preview thumbnails for many file types (PDFs, images, documents) providing much better visual identification.

## Proposed Solution

Implement Quick Look thumbnail generation for macOS:
- Use QLThumbnailGenerator (or Core Graphics Quick Look APIs)
- Generate thumbnails for copied file paths
- Convert CGImage to PNG bytes
- Store in thumbnail_base64 field (reuse existing field)
- Add core-graphics dependency to Cargo.toml
- Fallback to file type icon if Quick Look fails

## Dependencies

- Task 3: macOS file list reading provides file paths

## Context

- Thumbnail storage field already exists: `clipboard_item.rs:115-116`
- Current thumbnail generation: `file_storage.rs:158-186`
- macOS Quick Look: QLThumbnailGenerator or lower-level APIs
- Target size: 200px max (matches existing thumbnail logic)

## Success Criteria

- PDF files show actual page preview
- Image files show thumbnail
- Documents show preview when possible
- Unknown types fall back to file icon
- Thumbnails are reasonably sized (< 50KB)
- No crashes on protected/inaccessible files
