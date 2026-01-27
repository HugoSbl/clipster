# Drag & Drop System - Manual Testing Checklist

**Date**: 2026-01-23
**Task**: Task 4 - Manual Testing and Verification
**Platform**: macOS (primary), Windows (secondary)

---

## Pre-Testing Setup

- [ ] Application builds successfully (npm run build)
- [ ] Backend compiles successfully (cargo build)
- [ ] Run application: `npm run tauri dev`
- [ ] Application window opens without errors
- [ ] Clipboard monitoring is active

---

## Test 1: Image Drag & Drop

### Setup
1. Copy an image to clipboard (screenshot, image file, etc.)
2. Verify image appears in Clipster timeline
3. Image card should show thumbnail preview

### Test Steps
- [ ] **Click test**: Click card briefly (no movement) → Should NOT start drag
- [ ] **Threshold test**: Click and move <5px → Should NOT start drag
- [ ] **Drag initiation**: Click and move >5px → Drag should start
- [ ] **Visual feedback**: Card shows `.dragging` state (opacity/scale change)
- [ ] **Native preview**: OS drag ghost appears (not manual HTML clone)
- [ ] **Drag to Finder**: Drag card to Finder/Desktop
- [ ] **File verification**:
  - File appears with name format: `{SourceApp}_{timestamp}.{ext}`
  - Open file → Verify it's the FULL image (not thumbnail)
  - Check file size matches original
- [ ] **Cleanup**: No phantom drag ghost remains after drop

### Expected Results
✅ Full image file transferred with readable filename
✅ Native OS drag preview (not HTML clone)
✅ 5px threshold prevents accidental drags

---

## Test 2: Text Drag & Drop

### Setup
1. Copy text to clipboard (paragraph, code snippet, etc.)
2. Verify text appears in Clipster timeline
3. Text card should show preview

### Test Steps
- [ ] **Drag initiation**: Click and move >5px → Drag should start
- [ ] **Visual feedback**: Card shows dragging state
- [ ] **Drag to Desktop**: Drag card to Desktop
- [ ] **File verification**:
  - `.txt` file created with name: `{SourceApp}_{timestamp}.txt`
  - Open file → Verify ALL text content is present
  - Check for long text (>10KB) - all content preserved
  - Test special characters (émojis, accents) - properly encoded

### Expected Results
✅ Complete `.txt` file with all content
✅ UTF-8 encoding preserves special characters
✅ Readable filename format

---

## Test 3: Link Drag & Drop

### Setup
1. Copy a URL to clipboard (from browser, etc.)
2. Verify link appears in Clipster timeline
3. Link card should show URL preview

### Test Steps
- [ ] **Drag to Desktop**: Drag link card to Desktop
- [ ] **File verification (macOS)**:
  - `.webloc` file created
  - Double-click file → Opens in Safari/default browser
  - Correct URL loads
- [ ] **File verification (Windows)**:
  - `.url` file created
  - Double-click file → Opens in default browser
  - Correct URL loads

### Expected Results
✅ Platform-specific link file created (`.webloc` or `.url`)
✅ Link opens correctly in browser
✅ No encoding issues with URL

---

## Test 4: Files Drag & Drop

### Setup
1. Copy file(s) to clipboard (from Finder/Explorer)
2. Verify files appear in Clipster timeline
3. Card should show file name/count

### Test Steps
- [ ] **Single file**: Drag to Desktop → Original file transferred
- [ ] **Multiple files**: Card shows count badge → All files transfer
- [ ] **File integrity**:
  - File size matches original
  - File opens correctly
  - No corruption

### Expected Results
✅ Original files transferred (not copies)
✅ Multiple files handled correctly
✅ File integrity maintained

---

## Test 5: Audio Drag & Drop

### Setup
1. Copy audio file(s) to clipboard
2. Verify audio appears in Clipster timeline
3. Card should show audio icon/count

### Test Steps
- [ ] **Drag to Desktop**: Audio file transferred
- [ ] **Playback test**: Double-click file → Plays correctly
- [ ] **Metadata preserved**: File info intact

### Expected Results
✅ Audio file plays correctly
✅ Metadata/tags preserved

---

## Test 6: Documents Drag & Drop

### Setup
1. Copy document(s) to clipboard (PDF, Word, etc.)
2. Verify document appears in Clipster timeline
3. Card should show document icon/name

### Test Steps
- [ ] **PDF test**: Drag PDF to Desktop → Opens correctly
- [ ] **Word doc test**: Drag .docx to Desktop → Opens in Word/Pages
- [ ] **Content verification**: Document content is complete

### Expected Results
✅ Documents open in correct application
✅ Content is complete and formatted

---

## UX & Interaction Tests

### Selection Prevention
- [ ] **Text not selectable**: Try to select text in card → Should fail
- [ ] **Image not draggable**: Native browser drag disabled on images
- [ ] **No accidental selections**: Click around card → No text highlights

### Drag Threshold
- [ ] **Click**: Simple click (no movement) → No drag starts
- [ ] **Small move**: Click + move 2px → No drag starts
- [ ] **Threshold**: Click + move 6px → Drag starts immediately

### Visual States
- [ ] **Hover**: Card lifts slightly on hover
- [ ] **Dragging**: Card shows opacity/scale change during drag
- [ ] **After drag**: Card returns to normal state
- [ ] **No ghost**: No HTML clone remains after drag

### Native OS Preview
- [ ] **macOS**: Drag ghost has drop shadow (native macOS style)
- [ ] **Windows**: Drag ghost has transparency (native Windows style)
- [ ] **Icon**: Correct file icon shows in drag preview

---

## Edge Cases & Stress Tests

### Long Content
- [ ] **Large text** (>10KB): All content transfers
- [ ] **Large image** (>10MB): Full image transfers (not thumbnail)

### Special Characters
- [ ] **Filename with spaces**: `My Document.txt` → Sanitized correctly
- [ ] **Unicode in content**: émojis, 中文, العربية → Preserved correctly
- [ ] **URL encoding**: Special chars in URLs → Handled correctly

### Rapid Interactions
- [ ] **Quick drags**: Drag multiple items rapidly → No errors
- [ ] **Cancelled drag**: Drag then return to app → No temp files left
- [ ] **Interrupted drag**: ESC during drag → Cleanup occurs

### Error Handling
- [ ] **No content**: Empty text → Handled gracefully
- [ ] **Missing file**: Deleted file paths → Error handled
- [ ] **Permission denied**: Read-only locations → Error shown

---

## Regression Tests

### Existing Functionality
- [ ] **Clipboard monitoring**: Still captures new clipboard items
- [ ] **Pinboard system**: Items still pin/unpin correctly
- [ ] **Search**: Search still filters items
- [ ] **Delete**: Delete button still works
- [ ] **Keyboard nav**: Arrow keys still navigate timeline
- [ ] **Double-click**: Still copies item to clipboard
- [ ] **Selection**: Still selects/deselects cards

### Performance
- [ ] **Smooth drag**: No lag when moving cursor during drag
- [ ] **Quick response**: Drag starts <100ms after threshold
- [ ] **No memory leaks**: Multiple drags don't slow down app

---

## Cross-Platform Verification

### macOS Specific
- [ ] Drag to **Finder** → File appears with correct icon
- [ ] Drag to **Desktop** → File appears
- [ ] Drag to **Notes** → Content pastes correctly
- [ ] Drag to **TextEdit** → Text pastes correctly
- [ ] **Quick Look**: Press space on dragged file → Preview works
- [ ] **Quarantine**: No "downloaded from internet" warning

### Windows Specific
(If testing on Windows)
- [ ] Drag to **Explorer** → File appears with correct icon
- [ ] Drag to **Desktop** → File appears
- [ ] Drag to **Notepad** → Text pastes correctly
- [ ] **File properties**: Right-click → Properties shows correct info

---

## Final Checklist

- [ ] All 6 content types drag successfully
- [ ] Complete files transferred (verified by opening)
- [ ] Threshold prevents accidental drags (5px minimum)
- [ ] No selection issues (text/images not selectable)
- [ ] Native OS drag preview (no manual HTML clone)
- [ ] Visual states work correctly (dragging class)
- [ ] No regressions in other features
- [ ] Performance is smooth and responsive
- [ ] Error handling is graceful
- [ ] Cross-platform consistency confirmed (if testing both)

---

## Issues Found

**Document issues here during testing:**

| Issue # | Description | Severity | Content Type | Platform |
|---------|-------------|----------|--------------|----------|
| | | | | |

---

## Sign-Off

- [ ] **All critical tests passed**
- [ ] **No blocking issues found**
- [ ] **Ready for production use**

**Tester**: ___________
**Date**: ___________
**Platform(s) tested**: ___________

---

## Notes

Use this section for additional observations, performance notes, or suggestions for future improvements.
