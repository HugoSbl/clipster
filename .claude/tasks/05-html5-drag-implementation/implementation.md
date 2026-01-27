# Implementation: HTML5 Drag and Drop (Hybrid Approach)

## Status: ✅ Complete
**Progress**: All tasks completed

---

## Session Log

### Session 1 - 2026-01-24

**Task(s) Completed**: Full HTML5 drag-drop implementation with plugin fallback

**Files Changed:**
- `src/components/ClipboardCard.vue` - Added HTML5 drag handlers with hybrid plugin fallback
  - Added `convertFileSrc` import from @tauri-apps/api/core
  - Implemented `pathToFileUri()` helper for RFC 8089/3986 compliant file URI encoding
  - Implemented `createDragImage()` helper for custom 64x64 ghost images
  - Implemented `handleDragStart()` with event target filtering, dataTransfer setup, and plugin fallback
  - Implemented `handleDragEnd()` for cleanup
  - Added `draggable="true"`, `@dragstart`, `@dragend` to both card templates
  - Removed CSS `pointer-events: none` (replaced with event target filtering in JS)
  - Added `.html5-drag-ghost` CSS styles
  - Added comprehensive documentation comment block explaining hybrid approach

- `CLAUDE.md` - Updated drag & drop documentation
  - Expanded "Key Patterns" section with hybrid approach details
  - Added "Drag & Drop Details" section with Rust commands and key patterns

**Implementation Approach:**
The implementation uses a **hybrid strategy**:
1. **HTML5 Drag API**: Provides dragstart event, custom ghost image, and dataTransfer payload
2. **Plugin Fallback**: Ensures actual file transfer (triggered 50ms after HTML5 drag starts)

**Why Hybrid?**
- HTML5 `text/uri-list` cannot create real file drops from Tauri WebView to OS
- macOS: HTML5 creates .webloc bookmarks instead of actual files
- Windows: WebView2 ignores HTML5 file drag-out
- Plugin provides actual working file transfers via native OS APIs

**Key Technical Details:**
- Event target filtering (`e.target !== e.currentTarget`) replaces CSS pointer-events
- File paths encoded to `file://` URIs per RFC 8089 with percent-encoding per RFC 3986
- dataTransfer uses `\r\n` line endings for text/uri-list (RFC 2483 compliance)
- Ghost image created off-screen at 64x64px, loaded via Tauri's convertFileSrc()
- Plugin called with 50ms delay to let HTML5 drag initialize first

**TypeScript Validation:**
- ✅ `npx vue-tsc --noEmit` passed with no errors

**Notes:**
- User explicitly requested HTML5 implementation despite knowing it won't create real file drops
- Hybrid approach satisfies the HTML5 requirement while maintaining working functionality
- All console.log debugging statements kept per plan (helpful for testing)
- Implementation follows existing codebase patterns (Composition API, clear naming)

---

## Suggested Commit

```
feat: add HTML5 drag-drop with plugin fallback

- Implement HTML5 dragstart/dragend handlers with event target filtering
- Add pathToFileUri() for RFC-compliant file:// URI encoding
- Add createDragImage() for custom 64x64 ghost images
- Set dataTransfer with text/uri-list MIME type (\r\n line endings)
- Maintain plugin fallback for actual file transfers (hybrid approach)
- Remove CSS pointer-events in favor of JS event filtering
- Update CLAUDE.md with hybrid drag-drop documentation
```
