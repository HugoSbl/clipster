# Task 1: Configure Tauri for Plugin-Only Drag

## Problem

Tauri has a built-in drag & drop system that conflicts with `tauri-plugin-drag`. When both are active, they compete for drag events, causing race conditions and unpredictable behavior. The internal Tauri system must be explicitly disabled to allow the plugin to work correctly.

## Proposed Solution

Modify `tauri.conf.json` to set `dragDropEnabled: false` in the window configuration. This counter-intuitively **enables** the use of `tauri-plugin-drag` by disabling Tauri's internal drag system.

## Dependencies

- None (can start immediately)
- This is the **first task** and must be completed before backend/frontend work

## Context

**File to modify:**
- `src-tauri/tauri.conf.json` - Window configuration

**Key insight from documentation:**
- `dragDropEnabled: false` means "disable Tauri's INTERNAL drag system"
- This allows `tauri-plugin-drag` to take over drag operations
- The naming is confusing but this is the official behavior

**Configuration location:**
```json
{
  "app": {
    "windows": [{
      // Add dragDropEnabled: false here
    }]
  }
}
```

## Success Criteria

- `dragDropEnabled: false` is added to the window configuration
- JSON file is valid (no syntax errors)
- Configuration change is committed
- Tauri's internal drag system is disabled (verified by successful plugin use in later tasks)
