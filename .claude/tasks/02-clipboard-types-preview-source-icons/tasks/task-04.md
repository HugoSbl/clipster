# Task: Implement Windows Source App Detection

## Problem

When content is copied to the clipboard, we don't know which application it came from. Windows provides APIs to detect the clipboard owner application, which would improve UX by showing where each clip originated.

## Proposed Solution

Implement source app detection for Windows in clipboard_monitor.rs:
- Use GetClipboardOwner() to get HWND of clipboard owner
- Use GetWindowThreadProcessId() to get process ID
- Use OpenProcess() + GetModuleFileNameExW() to get exe path
- Extract app name from path (e.g., "chrome.exe" → "Chrome")
- Add required Windows crate features to Cargo.toml
- Handle edge cases (destroyed windows, system clipboard)

## Dependencies

- Task 2: source_app_icon field must exist in data model

## Context

- TODO location: `clipboard_monitor.rs:169-172`
- Process methods that call get_source_app: lines 87, 124, 147
- Windows crate already in use: `clipboard_reader.rs:35-39`
- Need features: Win32_System_ProcessStatus, Win32_System_Threading

## Success Criteria

- Copying from Chrome shows "Chrome" as source_app
- Copying from VSCode shows "Visual Studio Code" or similar
- System clipboard operations return None gracefully
- No crashes when owner window is destroyed
- Performance impact is minimal (API calls are fast)
