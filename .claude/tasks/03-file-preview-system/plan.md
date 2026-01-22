# Implementation Plan: File Preview System

## Overview

Le système de preview existe déjà et fonctionne bien pour les images et documents Office/PDF via Quick Look sur macOS. L'objectif est d'étendre le support aux **vidéos** et autres formats supportés par Quick Look, tout en gardant le système stable.

**Approche** : Modifications minimales, pas de nouvelles dépendances lourdes. On exploite Quick Look au maximum sur macOS.

## Dependencies

Ordre d'implémentation :
1. Backend Rust (file_storage.rs) - Ajouter support vidéos
2. Aucune modification frontend nécessaire (déjà prêt)
3. Tests manuels

## File Changes

### `src-tauri/src/storage/file_storage.rs`

**Objectif** : Ajouter le support des fichiers vidéo à la génération de thumbnails via Quick Look.

- **Action 1** : Modifier `is_image_file_macos()` (lignes 252-264)
  - Renommer en `is_direct_image_file()` pour clarifier son rôle
  - Cette fonction reste inchangée (images traitées par `image` crate)

- **Action 2** : Créer une nouvelle fonction `is_video_file()`
  - Liste des extensions vidéo : `mp4`, `mov`, `avi`, `mkv`, `webm`, `m4v`, `wmv`, `flv`, `3gp`
  - Ces fichiers seront traités par Quick Look (pas besoin de FFmpeg)

- **Action 3** : Modifier `generate_file_thumbnail_macos()` (lignes 236-249)
  - Ajouter une branche explicite pour les vidéos → Quick Look direct
  - Structure proposée :
    1. Si image → `generate_thumbnail_from_image_file()` (existant)
    2. Si vidéo → `generate_quicklook_thumbnail()` (existant, juste router)
    3. Sinon → `generate_quicklook_thumbnail()` (comportement actuel)
  - Pas besoin de modifier la logique Quick Look elle-même

- **Action 4** : Ajouter logging pour debug
  - Log quand un fichier vidéo est détecté
  - Log le résultat (succès/échec) pour faciliter le debug

**Pattern à suivre** : `is_image_file_macos()` lignes 252-264

### `src-tauri/src/clipboard/clipboard_monitor.rs`

**Objectif** : Aucune modification nécessaire

- Le code existant (`generate_file_thumbnail()` lignes 276-304) appelle déjà `file_storage::generate_file_thumbnail_macos()` correctement
- La limite de 50KB est déjà en place
- Les vidéos seront automatiquement supportées une fois le backend modifié

**Note** : Vérifier que le timeout de 3s est suffisant pour les vidéos longues (Quick Look est généralement rapide car il extrait juste une frame)

### `src/components/ClipboardCard.vue`

**Objectif** : Aucune modification nécessaire

- `hasVisualPreview` (lignes 137-145) vérifie déjà `content_type === 'files' && thumbnail_base64`
- Le template visual card (lignes 477-544) affiche déjà correctement les thumbnails
- Lazy loading est déjà en place

### `src-tauri/src/models/clipboard_item.rs`

**Objectif** : Aucune modification nécessaire

- Le type `files` est déjà utilisé pour tous les fichiers copiés
- `thumbnail_base64` stocke déjà le thumbnail
- Pas besoin d'ajouter un type `video` spécifique

### `src-tauri/Cargo.toml`

**Objectif** : Aucune modification pour le MVP macOS

- Quick Look gère les vidéos nativement
- `pdfium-render` pour Windows PDF peut être ajouté plus tard (scope out pour MVP)
- Pas besoin de `ffmpeg-next` sur macOS

## Testing Strategy

### Tests Manuels Requis

1. **Fichiers vidéo** (priorité haute)
   - Copier un fichier `.mp4` → vérifier thumbnail affiché
   - Copier un fichier `.mov` → vérifier thumbnail affiché
   - Copier une vidéo longue (>1h) → vérifier timeout ne bloque pas

2. **Fichiers documents** (régression)
   - Copier un `.pdf` → vérifier thumbnail toujours OK
   - Copier un `.docx` → vérifier thumbnail toujours OK
   - Copier un `.xlsx` → vérifier thumbnail toujours OK
   - Copier un `.pptx` → vérifier thumbnail toujours OK

3. **Fichiers images** (régression)
   - Copier un `.png` → vérifier thumbnail OK
   - Copier un `.jpg` → vérifier thumbnail OK
   - Copier une image > 50KB → vérifier skip OK

4. **Edge cases**
   - Copier un fichier vidéo corrompu → vérifier pas de crash
   - Copier un fichier avec extension vidéo mais pas une vidéo → vérifier graceful fail
   - Copier plusieurs fichiers dont une vidéo → vérifier thumbnail du premier fichier

### Validation

```bash
# Build et run
npm run tauri dev

# Vérifier les logs Rust pour :
# - "[generate_file_thumbnail_macos] Processing video file: ..."
# - "[generate_quicklook_thumbnail] ..."
```

## Documentation

Aucune documentation externe à mettre à jour.

Le fichier `CLAUDE.md` peut être mis à jour pour noter le support vidéo si souhaité.

## Rollout Considerations

### Risques

1. **Timeout vidéos longues** : Quick Look devrait être rapide mais surveiller
2. **Taille thumbnail** : Les thumbnails vidéo peuvent être > 50KB → seront skip (comportement acceptable)
3. **Formats exotiques** : Certains codecs peuvent ne pas être supportés par Quick Look

### Mitigation

- Le timeout 3s protège contre les hangs
- La limite 50KB évite le bloat DB
- Les fichiers non supportés retournent gracefully `None`

### Pas de Breaking Changes

- Le modèle de données ne change pas
- Le frontend n'a pas besoin de modifications
- Les fichiers existants en DB ne sont pas affectés

## Summary

| Fichier | Action | Complexité |
|---------|--------|------------|
| `file_storage.rs` | Ajouter support vidéos | Faible |
| `clipboard_monitor.rs` | Aucune | - |
| `ClipboardCard.vue` | Aucune | - |
| `clipboard_item.rs` | Aucune | - |
| `Cargo.toml` | Aucune | - |

**Estimation** : 1-2 heures max, principalement tests manuels.

**Scope out** :
- Support PDF/Office sur Windows (peut utiliser Shell API icons, déjà en place)
- FFmpeg pour vidéos Windows
- Support formats audio avec preview (mp3, wav via Quick Look - peut être ajouté plus tard)
