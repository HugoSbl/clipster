# Analysis: Fix Drag Visual - Card Instead of File

**Analyzed**: 2026-01-23
**Status**: Complete

## Quick Summary (TL;DR)

> **Primary content for lazy loading**

**Strategy used:**
- Code: 5/6 → Deep (2 agents)
- Web:  0/6 → Skip
- Docs: 2/6 → explore-docs

**Problem Confirmed**: Race condition entre HTML5 drag (pour visuel) et native drag (pour file transfer). Comportement mixte/instable reporté par l'utilisateur.

**Root Cause**: Les deux systèmes de drag coexistent sur le même élément sans synchronisation. Le browser peut initier `dragstart` HTML5 avant que `handleNativeDragMove()` ait pu appeler `startDrag()` du plugin.

**Key files to modify:**
- `src/components/ClipboardCard.vue:392-513` - Drag handlers (HTML5 + Native)
- `src/components/ClipboardCard.vue:526-600` - Template bindings

**Patterns to follow:**
- `createExactClone()` existe déjà (ligne 210-260) - copie récursive des styles
- Système de thumbnail pour images : `prepare_image_for_drag` command (clipboard_commands.rs:152-233)

**⚠️ Gotchas discovered:**
- **Race condition critique** : `@dragstart` peut se déclencher avant `mousemove` threshold
- `canDragAsFiles` peut être contourné par le browser
- Pas de `e.preventDefault()` dans `handleNativeDragStart()` (ligne 398)
- Erreurs de `startDrag()` sont avalées silencieusement (ligne 451-453)

**Dependencies:** Aucune dépendance bloquante

**Estimation:** ~2 tasks, ~2-3h total

---

## User Clarifications

**Q: Symptômes observés pendant le drag ?**
**A:** Comportement mixte/instable - parfois le fichier brut, parfois la card. Confirme une race condition.

---

## Codebase Context

### Current Drag Architecture (Dual-Mode System)

Clipster utilise **deux systèmes de drag parallèles** :

1. **HTML5 Drag API** (pour texte/liens)
   - Événements: `@dragstart` / `@dragend`
   - Ghost image: `createExactClone()` + `setDragImage()`
   - Activé quand: `canDragAsFiles = false`
   - Fichiers: ClipboardCard.vue:473-513

2. **Native Drag via tauri-plugin-drag** (pour fichiers/images)
   - Événements: `@mousedown` → `mousemove` detection
   - Ghost image: Géré par le plugin (paramètre `icon`)
   - Activé quand: `canDragAsFiles = true`
   - Fichiers: ClipboardCard.vue:392-459

### Problem: Race Condition

**Séquence actuelle (buggy):**
```
1. User mousedown → handleNativeDragStart() enregistre position
2. User bouge souris légèrement
3. Browser auto-initie drag AVANT threshold detection
4. dragstart HTML5 se déclenche → handleDragStart()
5. mousemove continue → handleNativeDragMove() (trop tard!)
```

**Résultat:** HTML5 drag prend le contrôle, drag la card mais ne transfert pas le fichier.

### Key Code Locations

| File | Lines | Section | Issue |
|------|-------|---------|-------|
| ClipboardCard.vue | 398-409 | `handleNativeDragStart()` | Pas de `e.preventDefault()` |
| ClipboardCard.vue | 411-459 | `handleNativeDragMove()` | Erreurs avalées (451-453) |
| ClipboardCard.vue | 526, 595 | Template `:draggable` | Browser peut outrepasser |
| ClipboardCard.vue | 381-390 | `canDragAsFiles` computed | Condition peut échouer |

### Existing Solutions (Already Implemented)

✅ **Clone system exists**: `createExactClone()` (210-260)
- Copie récursive du DOM avec tous les styles inline
- Positionnement off-screen (`left: -9999px`)
- Shadow: `0 8px 24px rgba(0, 0, 0, 0.2)`

✅ **Thumbnail generation**: `prepare_image_for_drag` command
- Backend Rust: clipboard_commands.rs:152-233
- Crée thumbnail 64x64 pour preview
- Copie dans `/tmp` avec nom lisible

✅ **Threshold detection**: 5px movement avant drag
- Anti-faux-positif pour éviter drag accidentel
- Défini à ligne 395: `const DRAG_THRESHOLD = 5`

---

## Documentation Insights (tauri-plugin-drag)

### API Overview

```typescript
import { startDrag } from '@crabnebula/tauri-plugin-drag';

await startDrag({
  item: string[],        // File paths to drag
  icon: string,          // Preview image path (OS-level ghost)
  mode?: 'copy'|'move'   // v2.1.0+ feature
});
```

### Critical Understanding

**Le plugin NE gère PAS le visuel du drag dans l'app.**

- `icon` = preview utilisée par l'OS lors du drag **externe** (vers Finder, Desktop, etc.)
- Pour le visuel **interne**, on doit utiliser HTML5 `setDragImage()`

**Approche hybride recommandée:**
1. Empêcher HTML5 drag default behavior complètement
2. Sur mousedown, créer le clone immédiatement
3. Sur mousemove threshold, appeler `startDrag()` ET afficher le clone manuellement
4. Gérer le visuel nous-mêmes via position absolute qui suit la souris

### Version Details

- **Current**: v2.1.0 (7 months old)
- **Features**: `mode: 'copy'|'move'` (contrôle le comportement OS)
- **Platform**: macOS, Windows, Linux (GTK)
- **Repository**: https://github.com/crabnebula-dev/drag-rs

---

## Research Findings

### Web Research: N/A (Skip)

Problème spécifique au projet, pas de recherche web nécessaire.

---

## Vision Analysis: N/A

Aucune image fournie avec la commande.

---

## Key Files with Line Numbers

### 1. **ClipboardCard.vue** (`src/components/ClipboardCard.vue`)

**Imports (5-6):**
```typescript
import { startDrag } from '@crabnebula/tauri-plugin-drag';
import { invoke } from '@tauri-apps/api/core';
```

**State (21-26):**
```typescript
const cardRef = ref<HTMLElement | null>(null);
const isDragging = ref(false);
```

**Clone Creation (210-260):**
```typescript
const createExactClone = (): HTMLElement => {
  // Clones DOM recursively with inline styles
  // Returns element for drag ghost image
}
```

**Native Drag Handlers (392-470):**
```typescript
// Line 392-409: handleNativeDragStart()
// Line 411-459: handleNativeDragMove() - RACE CONDITION HERE
// Line 461-470: cleanup()
```

**HTML5 Drag Handlers (473-513):**
```typescript
// Line 473-508: handleDragStart() - CONFLICT WITH NATIVE
// Line 510-513: handleDragEnd()
```

**Template Bindings (526-600):**
```vue
<div
  ref="cardRef"
  :draggable="!canDragAsFiles"  <!-- Line 526: CAN BE OVERRIDDEN -->
  @mousedown="handleNativeDragStart"
  @dragstart="handleDragStart"  <!-- Line 598: FIRES EARLY -->
  @dragend="handleDragEnd"
  :class="{ dragging: isDragging }"
>
```

### 2. **clipboard_commands.rs** (`src-tauri/src/commands/clipboard_commands.rs`)

**Image Preparation (152-233):**
```rust
#[tauri::command]
pub async fn prepare_image_for_drag(
    source_path: String,
    readable_filename: String,
) -> Result<(String, String), String> {
    // Copies image to temp with readable filename
    // Creates 64x64 thumbnail for drag icon
    // Returns (image_path, icon_path)
}
```

### 3. **pinboards.ts** (`src/stores/pinboards.ts`)

**Drag State (17-21, 227-234):**
```typescript
isDraggingItem: boolean;
draggingItemId: string | null;

setDragging(isDragging: boolean, itemId: string | null): void {
  // Called from ClipboardCard:437, 455, 483, 512
}
```

### 4. **@crabnebula/tauri-plugin-drag** (`node_modules/@crabnebula/tauri-plugin-drag/guest-js/index.ts`)

**Plugin API (57-73):**
```typescript
export async function startDrag(
  options: Options,
  onEvent?: (result: CallbackPayload) => void
): Promise<void> {
  await invoke("plugin:drag|start_drag", {
    item: options.item,
    image: options.icon,
    options: { mode: options.mode },
    onEvent: onEventChannel,
  });
}
```

---

## Patterns to Follow

### Vue Composition API
- **Script setup** : `<script setup lang="ts">` exclusivement
- **Refs pour state** : `isDragging`, `cardRef`
- **Computed pour dérivation** : `canDragAsFiles`
- **Pas d'Options API**

### Drag Detection Pattern
```typescript
// Threshold-based (anti-false-positive)
let dragStartPos: { x, y } | null = null;
const DRAG_THRESHOLD = 5; // pixels

const handleStart = (e: MouseEvent) => {
  dragStartPos = { x: e.clientX, y: e.clientY };
  document.addEventListener('mousemove', handleMove);
};

const handleMove = (e: MouseEvent) => {
  const dx = Math.abs(e.clientX - dragStartPos.x);
  if (dx > DRAG_THRESHOLD) {
    // NOW start drag
  }
};
```

### Error Handling Pattern
```typescript
try {
  const result = await invoke<ReturnType>('command', params);
} catch (error) {
  console.error('Command failed:', error);
  // Fallback behavior
}
```

### File Naming Convention
- Format: `{sourceApp}_{YYYYMMDD_HHMMSS}.{ext}`
- Sanitization: alphanumeric only, spaces → underscores
- Example: `Google_Chrome_20240123_143022.png`

---

## Dependencies

**Aucune dépendance bloquante identifiée.**

Packages déjà installés:
- `@crabnebula/tauri-plugin-drag: ^2.1.0` ✅
- Plugin initialisé dans `main.rs:60` ✅
- Command `prepare_image_for_drag` enregistrée ✅

---

## Implementation Strategy

### Solution Proposée: Désactiver HTML5 Drag Complètement

**Approche:**
1. Ajouter `e.preventDefault()` dans `handleNativeDragStart()`
2. Pour **tous** les types (pas seulement files):
   - Créer clone avec `createExactClone()`
   - Le positionner en `position: fixed` qui suit la souris
   - Pour files: appeler `startDrag()` en parallèle pour file transfer
   - Pour text/links: encoder dans DataTransfer custom event
3. Supprimer les handlers `@dragstart` / `@dragend` du template
4. Gérer le visuel manuellement via `mousemove` listeners

### Alternative: Forcer Native Drag Priority

**Approche:**
1. Dans `handleNativeDragStart()`, ajouter:
   ```typescript
   e.preventDefault(); // Empêche HTML5 dragstart
   e.stopPropagation();
   ```
2. Appeler `startDrag()` **immédiatement** sans attendre threshold
3. Passer un `icon` qui est le chemin vers une capture de la card
4. Créer une capture de card en canvas ou screenshot

**Avantage:** Utilise le plugin pour tout
**Inconvénient:** Nécessite générer une image de la card

### Recommandation

**Solution 1 (manuel hybrid)** est plus flexible et maintenable.

---

## Next Steps

Run `/apex:2-plan 01-fix-drag-visual-card` to create implementation plan.
