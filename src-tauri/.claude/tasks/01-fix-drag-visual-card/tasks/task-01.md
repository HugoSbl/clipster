# Task: Prevent HTML5 Drag Race Condition

## Problem

Currently, there's a race condition where the browser can initiate HTML5 `dragstart` event before our manual threshold detection (5px) completes. This causes HTML5 drag to take control, dragging card data instead of transferring files via the tauri-plugin-drag.

The browser's automatic drag behavior conflicts with our custom drag system, resulting in:
- Sometimes the file icon is dragged (HTML5 wins)
- Sometimes the card is dragged properly (native wins)
- Unstable/unpredictable behavior reported by user

## Proposed Solution

Eliminate the race condition by **completely preventing HTML5 drag initiation** and creating the visual ghost clone immediately on mousedown (before threshold detection).

Key changes:
1. Add `e.preventDefault()` and `e.stopPropagation()` as first actions in `handleNativeDragStart()` to block browser's automatic drag
2. Create ghost clone IMMEDIATELY using existing `createExactClone()` function (don't wait for threshold)
3. Add new state refs to hold the clone and track its position
4. Remove HTML5 drag handlers (`handleDragStart`, `handleDragEnd`) - no longer needed
5. Remove HTML5 drag attributes from template (`:draggable`, `@dragstart`, `@dragend`)

This ensures we have full control over both visual feedback and file transfer timing, with no browser interference.

## Dependencies

- None (can start immediately)
- This is the **foundational task** - all other drag improvements depend on this fix

## Context

**Key files:**
- `src/components/ClipboardCard.vue:392-409` - `handleNativeDragStart()` (needs e.preventDefault())
- `src/components/ClipboardCard.vue:473-513` - HTML5 handlers to remove
- `src/components/ClipboardCard.vue:526-600` - Template bindings to clean up
- `src/components/ClipboardCard.vue:210-260` - Existing `createExactClone()` function (production-ready)

**Patterns to follow:**
- Use existing `createExactClone()` - already handles recursive style copying
- Maintain existing threshold pattern at line 395 (5px movement)
- Follow Vue Composition API with `<script setup>` and refs

**Root cause analysis:**
Current sequence (buggy):
```
1. mousedown → handleNativeDragStart() records position
2. slight mouse movement
3. Browser auto-initiates drag BEFORE threshold
4. dragstart HTML5 fires → handleDragStart()
5. mousemove continues → handleNativeDragMove() (too late!)
```

Fixed sequence:
```
1. mousedown → preventDefault() blocks browser
2. Create clone immediately
3. Mouse moves → threshold detection
4. After 5px → call startDrag() plugin
5. Visual handled by us, file transfer by plugin
```

## Success Criteria

- [ ] No HTML5 `dragstart` event can fire (blocked by preventDefault)
- [ ] Ghost clone is created immediately on mousedown (before threshold)
- [ ] Template has no `:draggable`, `@dragstart`, or `@dragend` attributes
- [ ] HTML5 drag handler functions removed from script
- [ ] No race condition occurs - drag behavior is consistent
- [ ] Existing threshold detection (5px) still works
- [ ] Code compiles without TypeScript errors

**Verification steps:**
1. Click and hold card - clone should appear immediately
2. Move mouse slightly (<5px) - clone should exist but no drag initiated yet
3. Move mouse >5px - drag should proceed smoothly
4. No console warnings about HTML5 drag conflicts
