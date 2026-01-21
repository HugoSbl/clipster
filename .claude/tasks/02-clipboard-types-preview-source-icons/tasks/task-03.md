# Task: Implement macOS File List Reading via NSPasteboard

## Problem

The current macOS clipboard reader (`arboard` crate) doesn't support reading file lists from the clipboard. When users copy files in Finder, Clipster doesn't capture them. This is a core functionality gap compared to Windows.

## Proposed Solution

Implement native NSPasteboard access for macOS to read file URLs:
- Add objc2 dependencies to Cargo.toml for macOS
- Implement `read_files()` in clipboard_reader.rs for macOS
- Access NSPasteboard's generalPasteboard
- Read `public.file-url` UTI type
- Convert NSURL array to Vec<String> file paths
- Update `detect_format()` to check for files
- Update `read_clipboard()` to return files when available

## Dependencies

- None (independent backend feature)

## Context

- Current macOS read_files returns None: `clipboard_reader.rs:277-284`
- Windows implementation pattern to follow: `clipboard_reader.rs:166-171`
- macOS platform module: `clipboard_reader.rs:218-314`
- Detection order should match Windows: `clipboard_reader.rs:52-62`

## Success Criteria

- Copy files in Finder → files appear in Clipster history
- File paths are correctly extracted as strings
- Multiple files are captured as array
- Existing text and image capture still works
- No crashes when clipboard has other content types
