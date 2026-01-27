# Task: Improve Drag Error Handling and Logging

## Problem

Currently, errors from `startDrag()` and `prepareImageForDrag()` are silently swallowed (lines 451-453), making it impossible to debug issues when file drag operations fail. Additionally, there's no handling for edge cases like:
- Empty file path arrays
- Invalid image paths
- Failed thumbnail generation
- File system errors

Silent failures lead to poor user experience and difficult troubleshooting when drag operations fail unexpectedly.

## Proposed Solution

Add comprehensive error handling and logging throughout the drag system:

1. Wrap `startDrag()` calls in try-catch with descriptive error logging
2. Improve error handling in `getFilePathsForDrag()` for edge cases
3. Add fallback behavior when `prepareImageForDrag()` fails
4. Validate file paths before attempting drag operations
5. Log all drag-related errors to console with context

Errors should be logged clearly but not break the drag operation - provide graceful degradation where possible.

## Dependencies

- None (independent task)
- Can be executed in parallel with Task 2
- Enhances robustness regardless of other changes

## Context

**Key files:**
- `src/components/ClipboardCard.vue:411-459` - `handleNativeDragMove()` (add try-catch around startDrag)
- `src/components/ClipboardCard.vue:351-378` - `getFilePathsForDrag()` (improve error handling)
- Current issue: Lines 451-453 silently catch errors without logging

**Patterns to follow:**
- Use descriptive console.error messages with context
- Include file paths and error details in logs
- Follow existing error handling pattern from `prepareImageForDrag()` at line 320-324
- Don't throw errors that break the UI - gracefully degrade

**Error scenarios to handle:**
1. `startDrag()` fails - log error, don't break UI
2. `getFilePathsForDrag()` returns empty array - prevent drag attempt
3. `prepareImageForDrag()` fails - fallback to original path
4. Invalid JSON in `content_text` - log warning, return empty
5. File paths don't exist - validate before drag

**Logging format:**
```typescript
console.error('[ClipboardCard] startDrag failed:', error);
console.warn('[ClipboardCard] Empty file paths, skipping drag');
console.error('[ClipboardCard] prepareImageForDrag error:', { path, error });
```

## Success Criteria

- [ ] All `startDrag()` calls wrapped in try-catch with error logging
- [ ] `getFilePathsForDrag()` validates paths before returning
- [ ] Empty file path arrays are handled gracefully (don't call startDrag)
- [ ] `prepareImageForDrag()` failures have proper fallback behavior
- [ ] All errors logged with descriptive messages and context
- [ ] No silent failures - developers can debug drag issues via console
- [ ] UI remains functional even when drag operations fail

**Verification steps:**
1. Trigger drag with corrupted file path - error logged, UI doesn't crash
2. Trigger drag with invalid image path - fallback works, operation completes
3. Check console for any uncaught promise rejections during drag
4. Manually break `startDrag()` - error is logged with context
5. Test with empty file array - operation aborts gracefully with warning
