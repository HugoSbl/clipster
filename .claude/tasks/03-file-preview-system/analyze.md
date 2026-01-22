# Analysis: File Preview System for ClipboardCards

**Analyzed**: 2026-01-22
**Status**: Complete

## Quick Summary (TL;DR)

> Système de prévisualisation de fichiers sur les ClipboardCards pour tous types de fichiers copiés.

**Strategy used:**
- Code: 5/6 → Deep (2 agents)
- Web:  5/6 → intelligent-search
- Docs: 3/6 → explore-docs

**Key files to modify:**
- `src-tauri/src/storage/file_storage.rs` - Ajouter générateurs de thumbnails vidéo/PDF/Office
- `src-tauri/src/clipboard/clipboard_monitor.rs:276-300` - Étendre `generate_file_thumbnail()`
- `src/components/ClipboardCard.vue:137-145` - Modifier `hasVisualPreview` computed
- `src-tauri/Cargo.toml` - Ajouter dépendances (pdfium-render, ffmpeg-next optionnel)

**Patterns to follow:**
- Thumbnail pattern existant dans `file_storage.rs:170-193` - Lanczos3 400px max
- Quick Look macOS déjà utilisé pour documents (`file_storage.rs:236-377`)
- Base64 stocké en DB, lazy loading frontend

**Gotchas discovered:**
- Quick Look peut hang sur certains fichiers → timeout 3s déjà implémenté
- Thumbnails > 50KB sont skip (`clipboard_monitor.rs:298-299`)
- macOS Quick Look supporte DÉJÀ Word/Excel/PowerPoint/PDF → juste étendre les extensions autorisées
- Pour les vidéos: FFmpeg est lourd, préférer `qlmanage` sur macOS (natif)

**Dependencies:**
- macOS: Quick Look natif (déjà utilisé) - supporte 100+ types
- Cross-platform: `pdfium-render` pour PDF si Quick Look pas dispo
- Vidéos: `qlmanage` sur macOS, `ffmpeg-next` optionnel pour Windows

**Estimation:** ~6-8 tasks, ~4-6h total

---

## Codebase Context

### Architecture Actuelle

Le système de preview existe déjà partiellement :

1. **Modèle de données** (`src-tauri/src/models/clipboard_item.rs`):
   - `thumbnail_base64: Option<String>` - PNG base64 stocké en DB
   - `image_path: Option<String>` - Chemin vers image complète sur disque
   - Types supportés: `text`, `image`, `files`, `link`, `audio`

2. **Génération thumbnails** (`src-tauri/src/storage/file_storage.rs`):
   - `THUMBNAIL_MAX_SIZE = 400px` (ligne 14-16)
   - `generate_thumbnail()` - Resize avec Lanczos3 (lignes 170-193)
   - `generate_file_thumbnail_macos()` - Quick Look via `qlmanage` (lignes 236-377)
   - `generate_file_thumbnail_windows()` - Shell API (lignes 383-601)
   - **Blacklist code files** pour éviter hang de qlmanage (lignes 276-295)

3. **Pipeline de capture** (`src-tauri/src/clipboard/clipboard_monitor.rs`):
   - `process_files()` appelle `generate_file_thumbnail()` (ligne 176)
   - Skip si thumbnail > 50KB (lignes 298-299)
   - Timeout qlmanage: 3 secondes (ligne 336)

4. **Affichage frontend** (`src/components/ClipboardCard.vue`):
   - `hasVisualPreview` computed (lignes 137-145) : true si `thumbnail_base64` ET (image OU files)
   - Lazy loading images (`loading="lazy"`)
   - Visual card avec aspect ratio 1:1, object-fit cover

### Types de Fichiers Actuellement Supportés

| Type | macOS Support | Windows Support |
|------|---------------|-----------------|
| Images (PNG, JPG, GIF, WebP) | ✅ image crate | ✅ image crate |
| PDF | ✅ Quick Look | ❌ Icons only |
| Word (.docx) | ✅ Quick Look | ❌ Icons only |
| Excel (.xlsx) | ✅ Quick Look | ❌ Icons only |
| PowerPoint (.pptx) | ✅ Quick Look | ❌ Icons only |
| Pages (.pages) | ✅ Quick Look | N/A |
| Vidéos | ❌ Non implémenté | ❌ Non implémenté |
| Code files | ❌ Blacklisté | ❌ Blacklisté |

### Problème Identifié

Le code **blackliste** certaines extensions qui causent des hangs avec qlmanage (lignes 276-295 de `file_storage.rs`):

```rust
// Unsupported file types that might cause qlmanage to hang
let unsupported_extensions = [
    "rs", "ts", "js", "jsx", "tsx", "py", "rb", "go", "java", "c", "cpp", "h",
    "swift", "kt", "sh", "bash", "zsh", "fish", "ps1", "bat", "cmd",
    "json", "yaml", "yml", "toml", "xml", "html", "css", "scss", "less",
    "md", "txt", "log", "gitignore", "dockerignore", "env",
];
```

Mais les **documents Office et PDF ne sont PAS blacklistés** → ils devraient déjà fonctionner !

**Root cause probable**: Les extensions Office/PDF ne sont pas dans la liste d'images supportées (ligne 241-244):
```rust
let image_extensions = ["png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "ico"];
```

Le code utilise `image crate` pour ces extensions, et **Quick Look uniquement si image crate échoue** → mais c'est inversé ! Quick Look devrait être la source primaire pour documents.

---

## Documentation Insights

### Approche Recommandée par Type

**1. Images (tous formats)**
- Déjà géré par `image` crate
- Supporte: PNG, JPEG, GIF, WebP, BMP, TIFF, ICO
- Format exotique: Quick Look fallback

**2. PDF**
- **macOS**: Quick Look (`qlmanage -t`) - DÉJÀ EN PLACE
- **Windows**: `pdfium-render` crate (Google PDFium, MIT license)
- Performance: ~100ms première page

**3. Documents Office (Word, Excel, PowerPoint)**
- **macOS**: Quick Look natif - génère thumbnails pixel-perfect
- **Windows**: Pas de solution élégante sans LibreOffice
- Recommandation: Afficher icône sur Windows, preview sur macOS

**4. Vidéos (MP4, MOV, AVI)**
- **macOS**: Quick Look peut extraire frame
- **Alternative légère**: Utiliser `qlmanage -t` (pas besoin de FFmpeg)
- **Windows**: `ffmpeg-next` mais lourd en dépendances
- Recommandation: Quick Look sur macOS, icône sur Windows (MVP)

**5. Pages, Keynote, Numbers**
- macOS Quick Look uniquement (format Apple propriétaire)

### Performance Patterns

1. **Async processing**: Générer thumbnails en background pendant capture
2. **Caching**: Déjà stocké en base64 dans DB (pas de régénération)
3. **Size limits**: Skip > 50KB (déjà implémenté)
4. **Timeout**: 3s pour Quick Look (déjà implémenté)
5. **Lazy loading**: Frontend utilise `loading="lazy"`

---

## Research Findings

### Libraries Rust Évaluées

| Library | Usage | Performance | Cross-platform |
|---------|-------|-------------|----------------|
| `image` | Resize/encode | Excellente | ✅ |
| `pdfium-render` | PDF → image | ~100ms/page | ✅ |
| `ffmpeg-next` | Vidéo → frame | Lourd mais complet | ✅ |
| Quick Look (qlmanage) | Documents | ~50-300ms | macOS only |

### Recommandation: Approche Hybride

```
┌──────────────────────────────────────────────────────────────┐
│                    STRATEGY RECOMMANDÉE                       │
├──────────────────────────────────────────────────────────────┤
│ Type            │ macOS              │ Windows              │
├─────────────────┼────────────────────┼──────────────────────┤
│ Images          │ image crate        │ image crate          │
│ PDF             │ Quick Look         │ pdfium-render        │
│ Office docs     │ Quick Look         │ Icône (Shell API)    │
│ Vidéos          │ Quick Look         │ Icône (MVP)          │
│ Apple formats   │ Quick Look         │ N/A                  │
└──────────────────────────────────────────────────────────────┘
```

---

## Key Files

### Backend (Rust)

| File | Lines | Purpose |
|------|-------|---------|
| `src-tauri/src/storage/file_storage.rs` | 236-377 | `generate_file_thumbnail_macos()` - À MODIFIER |
| `src-tauri/src/clipboard/clipboard_monitor.rs` | 276-300 | `generate_file_thumbnail()` wrapper |
| `src-tauri/src/models/clipboard_item.rs` | 69-84 | `detect_from_files()` - Extension detection |
| `src-tauri/Cargo.toml` | - | Ajouter `pdfium-render` pour Windows |

### Frontend (Vue)

| File | Lines | Purpose |
|------|-------|---------|
| `src/components/ClipboardCard.vue` | 137-145 | `hasVisualPreview` computed |
| `src/components/ClipboardCard.vue` | 477-544 | Visual card template |
| `src/types/clipboard.ts` | 1-22 | Types TypeScript |

---

## Patterns to Follow

### Thumbnail Generation Pattern
```rust
// Existing pattern in file_storage.rs:170-193
pub fn generate_thumbnail(image: &DynamicImage, max_size: u32) -> Result<Vec<u8>> {
    // Preserve aspect ratio
    let (width, height) = image.dimensions();
    let scale = (max_size as f32 / width.max(height) as f32).min(1.0);
    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;

    // Lanczos3 for quality
    let resized = image.resize(new_width, new_height, FilterType::Lanczos3);

    // Encode to PNG
    let mut bytes = Vec::new();
    resized.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)?;
    Ok(bytes)
}
```

### Quick Look Pattern (macOS)
```rust
// Existing pattern in file_storage.rs:300-377
let output = Command::new("qlmanage")
    .args(&["-t", "-s", &max_size.to_string(), "-o", temp_dir, path_str])
    .timeout(Duration::from_secs(3))  // IMPORTANT: timeout
    .output();
```

### Base64 Storage Pattern
```rust
// Existing pattern in file_storage.rs:200-203
pub fn thumbnail_to_base64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
```

---

## Dependencies

### Current Dependencies (from Cargo.toml)
- `image` - Image processing
- `arboard` - Clipboard access
- `base64` - Encoding
- `objc2`, `objc2_app_kit` - macOS APIs

### Dependencies to Add
```toml
# For Windows PDF support (optional, only if Quick Look unavailable)
[target.'cfg(target_os = "windows")'.dependencies]
pdfium-render = { version = "0.8", features = ["image"], optional = true }
```

### No FFmpeg Required (MVP)
Sur macOS, Quick Look gère les vidéos nativement. Pas besoin d'ajouter FFmpeg pour le MVP.

---

## Implementation Notes

### Phase 1: Fix Document Previews (Quick Win)

Le problème actuel: les documents passent par `image crate` d'abord, qui échoue, puis le code return None au lieu d'essayer Quick Look.

**Fix**: Réorganiser la logique dans `generate_file_thumbnail_macos()`:
1. Si extension = image connue → image crate
2. Si extension = document (pdf, docx, xlsx, pptx, pages, etc.) → Quick Look DIRECT
3. Si extension = code/text → skip (pas de preview)
4. Sinon → essayer Quick Look avec fallback

### Phase 2: Add Video Support (macOS)

Ajouter les extensions vidéo à la liste Quick Look:
```rust
let video_extensions = ["mp4", "mov", "avi", "mkv", "webm", "m4v"];
```

### Phase 3: Windows PDF (Optional)

Ajouter `pdfium-render` uniquement pour Windows si Quick Look n'est pas disponible.

---

## Constraints

1. **Ne pas casser le système existant** - Le copié-collé fonctionne enfin
2. **Performance** - Thumbnails générés async pendant capture
3. **Taille** - Skip si > 50KB (déjà implémenté)
4. **Timeout** - 3s max pour Quick Look (déjà implémenté)
5. **macOS priority** - Windows peut se contenter d'icônes pour MVP
