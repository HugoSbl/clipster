# Task 4: Manual Testing and Verification

## Problem

After refactoring the drag & drop system, we need comprehensive testing to verify:
1. All content types transfer correctly to external apps
2. Complete files are copied (not just icons/thumbnails)
3. Drag threshold prevents accidental drags
4. No selection issues (text/images not selectable)
5. Cross-platform behavior is consistent (macOS and Windows)

## Proposed Solution

Execute a comprehensive manual test plan covering all content types and edge cases on both macOS and Windows platforms.

## Dependencies

- **Task 3**: Frontend refactoring must be complete
- All code changes must be implemented and compiled
- Application must run without errors

## Context

**Testing environments:**
- **macOS**: Test drag to Finder, Desktop, Notes, TextEdit
- **Windows**: Test drag to Explorer, Desktop, Notepad

**Content types to test:**
1. **Images** - Should transfer full PNG/JPG, not thumbnail
2. **Text** - Should create `.txt` file with full content
3. **Links** - Should create `.webloc` (macOS) or `.url` (Windows)
4. **Files** - Should transfer original files
5. **Audio** - Should transfer audio files
6. **Documents** - Should transfer document files (PDF, DOC, etc.)

**Critical verifications:**
- Open dragged file to confirm it's complete (not just icon)
- Check filename format: `{sourceApp}_{timestamp}.{ext}`
- Verify file size matches original (for images/docs)
- Test threshold: click without drag should NOT trigger drag
- Test selection prevention: text should NOT be selectable

## Success Criteria

**Functional tests:**
- ✅ Image drag: Full image file copied (verify by opening)
- ✅ Text drag: `.txt` file created with complete content
- ✅ Link drag: `.webloc` or `.url` opens correct URL
- ✅ Files drag: Original files transferred
- ✅ Audio drag: Audio files play correctly
- ✅ Documents drag: Documents open correctly

**UX tests:**
- ✅ Threshold: Click without moving does NOT start drag
- ✅ Threshold: Moving >5px starts drag smoothly
- ✅ Preview: Native OS drag ghost appears (no flash)
- ✅ Selection: No text or images can be selected in card
- ✅ Visual: Card shows `.dragging` state during drag
- ✅ Cleanup: No phantom drag ghosts after drag ends

**Cross-platform tests:**
- ✅ macOS: Drag to Finder shows correct file icon
- ✅ macOS: Link creates `.webloc` that opens in Safari
- ✅ Windows: Drag to Explorer shows correct file icon
- ✅ Windows: Link creates `.url` that opens in default browser

**Edge cases:**
- ✅ Long text content (>10KB) transfers completely
- ✅ Special characters in filename are handled
- ✅ Drag cancelled (moved back to app) doesn't leave files
- ✅ Multiple rapid drags don't cause issues

## Test Execution

For each content type:
1. Copy item to clipboard (triggers Clipster to capture it)
2. Find item in Clipster timeline
3. Click and hold on card
4. Move mouse >5px (should see native drag ghost)
5. Drag to Finder/Explorer/Desktop
6. Release mouse
7. Open the transferred file
8. Verify file is complete and correct

**Regression checks:**
- Existing clipboard capture still works
- Pinboard functionality unaffected
- Search still works
- Delete still works
- Keyboard navigation still works

## Success Criteria Summary

- All 6 content types drag successfully
- Complete files transferred (verified by opening)
- Threshold prevents accidental drags
- No selection issues
- Cross-platform consistency confirmed
- No regressions in other features
- User experience is smooth and predictable
