# Seed: Rebuild Drag & Drop System from Scratch

**Created**: 2026-01-23
**Type**: Complete Refactoring
**Complexity**: Medium-High (3-4 hours estimated)

---

## 🎯 Objectif

Reconstruire le système de drag & drop **from scratch** en utilisant **uniquement** `tauri-plugin-drag` avec l'approche la plus simple et logique recommandée par Tauri.

### Comportements attendus

1. **Drag dans l'app puis vers le système**
   - L'utilisateur clique sur une card et drag
   - Drag démarre après un seuil de 5px (évite drags accidentels)
   - Preview native du plugin suit le curseur (gérée par l'OS)
   - L'utilisateur peut drag vers Finder, Desktop, autre app

2. **Seule la card est draggable**
   - L'image/document sous-jacent **ne doit PAS** être sélectionnable
   - Le texte **ne doit PAS** être sélectionnable
   - Aucun élément interne de la card ne doit être draggable

3. **Transfert du fichier réel**
   - Image → fichier image copié (pas juste l'icône)
   - Texte → fichier texte créé et copié
   - Document → document original copié
   - Pas de transfert d'icône/thumbnail, le fichier complet

4. **Cross-platform (macOS + Windows)**
   - Utiliser les outils Tauri préfaits
   - Méthode recommandée officielle
   - Même code pour Mac et Windows

---

## 🚀 Point de départ

### Fichiers à modifier

**1. `src/components/ClipboardCard.vue`** (MAJEUR)
- Supprimer toute la logique HTML5 drag (lignes 437-603)
- Supprimer `createExactClone()` (lignes 210-260)
- Supprimer handlers `handleImageDragStart`, `handleImageDrag` (lignes 592-603)
- Remplacer par approche simple: mousedown → threshold → `startDrag()` uniquement
- Garder `prepareImageForDrag()` (lignes 287-327) et `getFilePathsForDrag()` (lignes 329-418)

**2. `src/App.vue`** (MINEUR)
- Vérifier/ajuster global drag prevention (lignes 49-58)
- Potentiellement simplifier si plugin gère tout

**3. `src-tauri/tauri.conf.json`** (CONFIG)
- Ajouter `"dragDropEnabled": false` dans window config
- Évite conflits entre Tauri internal drag et plugin

**4. `src-tauri/src/commands/clipboard_commands.rs`** (VALIDATION)
- Fonction `prepare_image_for_drag()` déjà existante (lignes 150-233)
- Vérifier qu'elle copie bien le fichier complet, pas juste l'icône

### Fichiers à lire (contexte)

- `.claude/tasks/01-fix-drag-visual-card/analyze.md` - Race condition identifiée
- `.claude/tasks/01-fix-drag-visual-card/plan.md` - Architecture actuelle
- Documentation Tauri officielle (via Context7)
- README de `@crabnebula/tauri-plugin-drag`

---

## ⚠️ Pièges à éviter

### 1. Race Condition (Problème actuel)
**Ne PAS faire:**
- Mélanger HTML5 drag (`@dragstart`) et `tauri-plugin-drag`
- Créer un clone manuel ET utiliser le plugin
- Appeler `startDrag()` sans avoir désactivé HTML5 drag

**Pourquoi:** HTML5 drag démarre avant que `startDrag()` ne soit appelé, causant une compétition entre les deux systèmes.

### 2. Preview Visuelle (Problème actuel)
**Ne PAS faire:**
- Gérer manuellement un clone HTML avec `transform: translate3d()`
- Utiliser `setDragImage()` (HTML5 API)

**Faire:** Passer le paramètre `icon` à `startDrag()`, le plugin gère automatiquement la preview native.

### 3. Configuration Tauri
**Ne PAS faire:**
- Laisser `dragDropEnabled: true` (default)

**Faire:** Mettre `dragDropEnabled: false` dans tauri.conf.json pour désactiver le système interne Tauri et éviter conflits.

### 4. Sélection de contenu
**Ne PAS faire:**
- Oublier `user-select: none` sur les éléments internes
- Oublier `draggable="false"` sur les images

**Note:** Déjà implémenté correctement dans le code actuel (lignes 841-853 CSS).

### 5. Transfert de fichiers
**Ne PAS faire:**
- Passer juste le thumbnail/icône au plugin
- Utiliser des chemins relatifs
- Oublier de créer le fichier temp pour le texte

**Faire:**
- Images: Copier le fichier image complet vers temp (déjà fait par `prepare_image_for_drag`)
- Texte: Créer un fichier .txt temporaire avec le contenu
- Documents: Passer le chemin du document original

### 6. Cleanup
**Ne PAS faire:**
- Oublier de nettoyer les listeners `mousemove` et `mouseup`
- Laisser des fichiers temp non nettoyés

**Faire:**
- Toujours appeler `removeEventListener` dans `handleNativeDragEnd`
- Les fichiers temp sont nettoyés par l'OS (dans `std::env::temp_dir()`)

---

## 📋 Spécifications

### Décisions prises (via clarification)

1. **Approche**: Refonte from scratch
   - Supprimer toute logique HTML5 drag
   - Utiliser **UNIQUEMENT** `tauri-plugin-drag`
   - Architecture simple: mousedown → threshold → `startDrag()`

2. **Preview**: Native du plugin
   - Pas de clone HTML manuel
   - Plugin affiche automatiquement l'`icon` passé en paramètre
   - Preview gérée par l'OS (shadow macOS, transparency Windows)

3. **Threshold**: Avec détection de seuil 5px
   - Drag démarre seulement après 5px de mouvement
   - Évite les drags accidentels sur clicks courts
   - Garde le système de détection actuel

### Exigences fonctionnelles

**RF1: Drag visuel cohérent**
- La card ne doit pas "sauter" ou "flasher" au début du drag
- Preview doit apparaître immédiatement après le seuil
- Preview doit suivre le curseur de manière fluide

**RF2: Transfert fichier correct**
- Image → Fichier PNG/JPG complet (pas thumbnail)
- Texte → Fichier .txt avec contenu complet
- Document → Document original (PDF, DOC, etc.)
- Nom de fichier lisible: `{sourceApp}_{timestamp}.{ext}`

**RF3: Prévention sélection**
- Aucun texte sélectionnable dans la card
- Aucune image draggable nativement par le browser
- Click simple = sélection de la card (pas de drag)
- Click + drag > 5px = drag de fichier

**RF4: Cross-platform**
- Même code pour macOS et Windows
- Plugin `tauri-plugin-drag` gère les différences platform
- Pas de `#[cfg(target_os)]` dans le code Vue

**RF5: États visuels**
- Card en état "dragging" pendant le drag (opacity, scale)
- Pas de card fantôme qui reste après le drag
- Feedback visuel clair sur la card source

### Exigences techniques

**RT1: Configuration Tauri**
```json
{
  "app": {
    "windows": [{
      "dragDropEnabled": false
    }]
  }
}
```

**RT2: API Plugin**
```typescript
await startDrag({
  item: ['/absolute/path/to/file.ext'],  // Fichier complet
  icon: '/absolute/path/to/icon.png'     // Icône 64x64
});
```

**RT3: CSS Selection Prevention**
```css
.clipboard-card,
.clipboard-card * {
  user-select: none;
  -webkit-user-select: none;
}

.clipboard-card img {
  pointer-events: none;
}
```

**RT4: Attributs HTML**
```vue
<img draggable="false" @dragstart.prevent />
```

---

## 🔍 Contexte technique

### Architecture actuelle (à remplacer)

**Système dual (buggy):**
```
mousedown → threshold (5px)
  ├─ HTML5: createExactClone() + position manual
  └─ Native: startDrag() after threshold
      ↓
  Race condition: HTML5 peut gagner avant startDrag()
```

**Problèmes identifiés:**
1. **Race condition** entre HTML5 `@dragstart` et `startDrag()`
2. **Clone manuel complexe** avec `transform: translate3d()`
3. **Listeners multiples** (mousemove, mouseup) difficiles à cleanup
4. **Dual-mode** confond les deux systèmes de drag

### Architecture cible (simple)

**Système unique:**
```
mousedown → threshold (5px) → startDrag() UNIQUEMENT
                               ↓
                      Plugin gère la preview native
                      OS gère le drag & drop
```

**Avantages:**
1. ✅ Pas de race condition (un seul système)
2. ✅ Preview native gérée par l'OS (pas de manuel positioning)
3. ✅ Cleanup simple (plugin gère tout)
4. ✅ Cross-platform automatique

### Stack technique

**Frontend:**
- Vue 3 Composition API (`<script setup lang="ts">`)
- Pinia stores (clipboard, pinboards)
- Tauri IPC (`invoke()`)

**Backend:**
- Rust Tauri commands
- `tauri-plugin-drag@2.1.0`
- Image crate pour thumbnails

**Plugin API:**
```typescript
import { startDrag } from '@crabnebula/tauri-plugin-drag';

interface StartDragOptions {
  item: string[];   // Array de chemins absolus
  icon: string;     // Chemin absolu vers icône preview
}
```

### Données à préparer

**Pour Images:**
```typescript
const { imagePath, iconPath } = await invoke<[string, string]>(
  'prepare_image_for_drag',
  { sourcePath: item.image_path, readableFilename }
);
// imagePath: Fichier image complet dans temp
// iconPath: Thumbnail 64x64 pour preview
```

**Pour Texte:**
```typescript
// Créer fichier temp .txt avec contenu
const textPath = await invoke('create_temp_text_file', {
  content: item.content_text,
  filename: `${sourceApp}_${timestamp}.txt`
});
```

**Pour Documents/Files:**
```typescript
// Parser JSON paths
const paths = JSON.parse(item.content_text);
// Passer directement au plugin
```

### Types de contenu (ContentType enum)

```typescript
type ContentType =
  | 'text'      // → Créer .txt temp
  | 'image'     // → Copier image vers temp
  | 'files'     // → Passer paths directement
  | 'link'      // → Créer .webloc (macOS) ou .url (Windows)
  | 'audio'     // → Passer paths audio directement
  | 'documents' // → Passer paths docs directement
```

### Rust Command existant

**`prepare_image_for_drag()` (clipboard_commands.rs:150-233)**

Fait déjà:
- ✅ Copie fichier image vers temp avec nom lisible
- ✅ Crée thumbnail 64x64 PNG
- ✅ Supprime quarantine xattr (macOS)
- ✅ Retourne tuple `(imagePath, iconPath)`

**À créer:**
- `create_temp_text_file(content, filename)` → string path
- `create_temp_link_file(url, filename)` → string path

### Détection de seuil (à garder)

```typescript
const DRAG_THRESHOLD = 5; // pixels

let dragStartPos: { x: number; y: number } | null = null;

const handleMouseDown = (e: MouseEvent) => {
  dragStartPos = { x: e.clientX, y: e.clientY };
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
};

const handleMouseMove = (e: MouseEvent) => {
  if (!dragStartPos) return;

  const dx = Math.abs(e.clientX - dragStartPos.x);
  const dy = Math.abs(e.clientY - dragStartPos.y);

  if (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD) {
    // Threshold dépassé → lancer startDrag()
    initiateDrag();
  }
};
```

### Configuration Cross-Platform

**Tauri Config (`tauri.conf.json`):**
```json
{
  "app": {
    "windows": [{
      "title": "Clipster",
      "dragDropEnabled": false,  // CRITIQUE: désactive internal Tauri drag
      "decorations": true,
      "resizable": true
    }]
  }
}
```

**Important:**
- `dragDropEnabled: false` signifie "désactive le système INTERNE Tauri"
- Cela **active** l'utilisation de `tauri-plugin-drag`
- Naming confusing mais c'est le comportement officiel

### Documentation officielle Tauri

**Sources consultées via Context7:**
- Tauri v2 Window Customization: https://v2.tauri.app/learn/window-customization/
- tauri-plugin-drag npm: https://www.npmjs.com/package/@crabnebula/tauri-plugin-drag
- CrabNebula drag-rs: https://github.com/crabnebula-dev/drag-rs
- Tauri Drag/Drop Issue #9830: https://github.com/tauri-apps/tauri/issues/9830
- Tauri WebviewWindow API: https://v2.tauri.app/reference/javascript/api/namespacewebviewwindow/

**Key findings:**
- ✅ Plugin officiel de CrabNebula (maintainers de Tauri)
- ✅ Cross-platform: macOS, Windows, Linux (GTK)
- ✅ Preview native automatique via `icon` parameter
- ⚠️ Ne PAS mélanger avec HTML5 drag API
- ⚠️ Configurer `dragDropEnabled: false` obligatoire

### Patterns Vue à suivre

**Composition API strict:**
```typescript
// ✅ CORRECT
const isDragging = ref(false);
const dragStartPos = ref<{ x: number; y: number } | null>(null);

// ❌ INCORRECT (Options API)
data() { return { isDragging: false } }
```

**Event handlers:**
```typescript
// ✅ CORRECT
const handleMouseDown = (e: MouseEvent) => {
  dragStartPos.value = { x: e.clientX, y: e.clientY };
};

// ❌ INCORRECT
handleMouseDown: function(e) { ... }
```

**Store usage:**
```typescript
// ✅ CORRECT
import { usePinboardStore } from '@/stores/pinboards';
const pinboardStore = usePinboardStore();
pinboardStore.setDragging(true, itemId);

// ❌ INCORRECT
this.$store.commit('setDragging', true);
```

---

## 📊 Strategy Scores

**Code**: 5/6 → Deep exploration (2 agents)
**Web**: 0/6 → Skip (doc via Context7 uniquement)
**Docs**: 5/6 → explore-docs (Tauri official docs)

**Agents lancés:**
- ✅ explore-codebase: Analyse implémentation actuelle
- ✅ explore-codebase: File preparation et sélection prevention
- ✅ explore-docs: Documentation Tauri drag & drop

---

## 🎯 Résumé pour la prochaine phase

### À faire dans l'ordre

1. **Configuration** (`tauri.conf.json`)
   - Ajouter `"dragDropEnabled": false`

2. **Rust Commands** (`clipboard_commands.rs`)
   - Créer `create_temp_text_file()`
   - Créer `create_temp_link_file()`

3. **ClipboardCard Refactoring** (`ClipboardCard.vue`)
   - Supprimer toute logique HTML5
   - Garder threshold detection (5px)
   - Simplifier: mousedown → threshold → `startDrag()` only
   - États visuels: `isDragging` ref + CSS `.dragging` class

4. **Testing**
   - Test macOS: drag vers Finder, Desktop
   - Test Windows: drag vers Explorer, Desktop
   - Vérifier: fichier complet copié (pas juste icône)
   - Vérifier: prévention sélection fonctionne

### Estimation

**Complexité**: Medium-High
**Temps estimé**: 3-4 heures

**Breakdown:**
- Configuration Tauri: 15 min
- Rust commands (texte, link): 45 min
- Vue refactoring: 1.5-2h
- Testing & debugging: 1h

### Success Criteria

- ✅ Drag fonctionne sur macOS ET Windows
- ✅ Fichier complet copié (vérifié en ouvrant le fichier)
- ✅ Preview native suit le curseur
- ✅ Pas de race condition (comportement stable)
- ✅ Aucun texte/image sélectionnable
- ✅ Threshold 5px évite drags accidentels
- ✅ Code simple et maintenable
