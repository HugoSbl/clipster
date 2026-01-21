# Task: Add source_app_icon Field to Data Model

## Problem

The data model needs a new field to store the source application's icon as a base64-encoded PNG. This field must be added to the Rust struct, database schema, and TypeScript interface before source app detection can be implemented.

## Proposed Solution

Add `source_app_icon: Option<String>` field to ClipboardItem:
- Update Rust struct in clipboard_item.rs
- Update all constructor methods to accept the new field
- Add database migration for the new column
- Update TypeScript interface in clipboard.ts

## Dependencies

- None (data model changes are foundational)

## Context

- Rust struct: `clipboard_item.rs:103-136`
- Constructors: `clipboard_item.rs:140-220`
- Database from_row: `clipboard_item.rs:233-250`
- TypeScript interface: `clipboard.ts:11-21`
- Database schema: `storage/database.rs`

## Success Criteria

- ClipboardItem struct has source_app_icon field
- All constructors accept source_app_icon parameter
- Database migration adds nullable source_app_icon column
- TypeScript interface matches Rust struct
- Existing items continue to work (NULL icon)
- App compiles and existing clipboard capture still works
