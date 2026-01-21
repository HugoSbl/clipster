# Task: Enlarge UI Components

## Problem

The current card size (180x140px) and font sizes are too small for comfortable use. The UI needs to be enlarged to match the planned Paste-like overlay design with cards at 280x220px.

## Proposed Solution

Update ClipboardCard.vue and Timeline.vue to use larger dimensions:
- Cards: 280x220px with proportionally larger border radius
- Thumbnails: max-height 120px (from 70px)
- Type icons: 24px (from 20px)
- Fonts: 14px text preview, 11px timestamp
- Spacing: larger padding throughout

Update Timeline.vue gap from 12px to 16px.

## Dependencies

- None (can start immediately - pure frontend changes)

## Context

- Card styles: `ClipboardCard.vue:246-261`
- Type icons: `ClipboardCard.vue:327-373`
- Thumbnail size: `ClipboardCard.vue:414-419`
- Timeline gap: `Timeline.vue` scroll container styles
- No backend changes required

## Success Criteria

- Cards display at 280x220px
- All text is readable and proportionally sized
- Thumbnail previews are larger and clearer
- Timeline scrolling works correctly with new card sizes
- No visual regressions or overflow issues
- App compiles and runs without errors
