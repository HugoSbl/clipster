# Task: Implement macOS Source App Icon Extraction

## Problem

Once we detect the source application on macOS, we need to extract its icon. macOS provides the icon through NSRunningApplication's icon property as an NSImage.

## Proposed Solution

Implement icon extraction for macOS:
- Get NSRunningApplication from frontmost app detection
- Access the icon property (NSImage)
- Convert NSImage to PNG data via NSBitmapImageRep
- Encode as base64 string
- Cache icons by bundle identifier to avoid repeated extraction
- Return None gracefully on failure

## Dependencies

- Task 5: macOS source app detection provides NSRunningApplication

## Context

- NSRunningApplication already accessed in Task 5
- NSImage → PNG conversion is common pattern
- objc2-app-kit dependencies already added in Task 3
- Target icon size: 32x32 or similar for consistency

## Success Criteria

- Safari icon is extracted and stored as base64 PNG
- Various app icons work (Terminal, Finder, etc.)
- System apps return valid icons
- Apps without icons return None gracefully
- Icons are properly sized and encoded
