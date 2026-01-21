# Task: Display Source App Icon in Card Header

## Problem

The source app icon field is populated in the backend, but the frontend doesn't display it. Users should see the source app's icon in the clipboard card to quickly identify where content came from.

## Proposed Solution

Update ClipboardCard.vue to display source app icon:
- Add source app icon in card header (before type icon)
- Conditional render: only show if item.source_app_icon exists
- Display as 16x16 or 20x20 image with rounded corners
- Keep type icon as secondary indicator
- Add tooltip showing source_app name on hover

## Dependencies

- Task 1: UI enlargement provides more space in header
- Task 2: source_app_icon field exists in TypeScript interface
- Task 6 or 7: Icons are actually being populated

## Context

- Card header: `ClipboardCard.vue:148-168`
- Type indicator location: `ClipboardCard.vue:149-166`
- source_app field already exists in interface
- Pattern: conditional rendering with v-if

## Success Criteria

- Source app icon displays when available
- Icon is properly sized (16-20px)
- Fallback to type icon only when no source icon
- Tooltip shows app name
- No layout shift when icon is missing vs present
