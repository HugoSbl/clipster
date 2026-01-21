# Task: Display File Thumbnails in Card

## Problem

The file content section in ClipboardCard currently shows a generic file icon SVG. When Quick Look/Shell thumbnails are available, we should display them instead for better visual identification.

## Proposed Solution

Update ClipboardCard.vue files content section:
- Check if thumbnail_base64 exists for file items
- Display thumbnail image if available (similar to image content)
- Show file count overlay on thumbnail corner
- Fall back to generic file icon when no thumbnail
- Adjust layout to accommodate thumbnail display

## Dependencies

- Task 9 or 10: File thumbnails are generated
- Task 1: UI enlargement provides more space

## Context

- Files content section: `ClipboardCard.vue:192-201`
- Image content pattern to follow: `ClipboardCard.vue:178-189`
- thumbnail_base64 field already in interface
- Reuse existing thumbnail CSS classes

## Success Criteria

- Files with thumbnails show actual preview
- Count badge shows number of files
- Generic icon shown when no thumbnail
- Layout is consistent with image cards
- Works for single and multiple files
