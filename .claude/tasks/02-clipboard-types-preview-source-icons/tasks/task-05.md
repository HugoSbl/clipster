# Task: Implement macOS Source App Detection

## Problem

macOS doesn't expose clipboard owner through NSPasteboard. We need an alternative approach to detect which application content was copied from, using the Accessibility API to track the frontmost application.

## Proposed Solution

Implement source app detection for macOS:
- Track frontmost application when clipboard changes (in polling loop)
- Use NSWorkspace's frontmostApplication property
- Get localizedName from NSRunningApplication
- Add necessary objc2 dependencies to Cargo.toml
- Handle edge cases (background copies, permission denied)
- Implement graceful fallback (return None if unable to determine)

## Dependencies

- Task 2: source_app_icon field must exist in data model
- Task 3: macOS NSPasteboard dependencies already added

## Context

- macOS monitoring loop: `clipboard_monitor.rs:226-272`
- Can track frontmost app at each poll iteration
- Accessibility permission may be required
- Add NSAppleEventsUsageDescription to Info.plist

## Success Criteria

- Copying from Safari shows "Safari" as source_app
- Copying from Terminal shows "Terminal"
- Works without Accessibility permission for basic detection
- Falls back gracefully if NSWorkspace fails
- No crashes or hangs when checking frontmost app
