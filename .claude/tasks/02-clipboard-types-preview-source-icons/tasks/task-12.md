# Task: Implement Adaptive Overlay Window

## Problem

The current window is fixed at 800x600px. It should behave like the Paste app: full width of the screen, approximately 1/3 height, positioned at the bottom, appearing as an overlay above other applications.

## Proposed Solution

Implement adaptive window sizing:
- On app startup, query primary monitor dimensions via Tauri's Monitor API
- Calculate dimensions: width = monitor.width, height = monitor.height * 0.33
- Position at screen bottom: y = monitor.height * 0.67
- Apply via window.set_size() and window.set_position()
- Update tauri.conf.json: remove fixed dimensions, keep alwaysOnTop, transparent
- Handle multi-monitor setups (use focused monitor or primary)

## Dependencies

- None (can be done independently, but best done last for visual polish)

## Context

- Current config: `tauri.conf.json:13-22`
- Tauri Monitor API for screen dimensions
- main.rs setup hook for window configuration
- Window should update if moved between monitors (stretch goal)

## Success Criteria

- Window spans full screen width
- Window is ~1/3 screen height
- Window anchored to bottom of screen
- Always on top of other windows
- Works on different screen resolutions
- Transparent background for overlay effect
