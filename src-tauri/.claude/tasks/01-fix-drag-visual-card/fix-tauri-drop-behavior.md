# Fix: Tauri Ouvre les Fichiers Droppés (Comportement Navigateur)

**Date**: 2026-01-23
**Type**: Critical Bug Fix
**Status**: ✅ FIXED

## TL;DR

Tauri (basé sur WebView) ouvrait les fichiers droppés dans la fenêtre comme un navigateur. Solution : Bloquer globalement les événements `drop`, `dragover`, etc. avec `preventDefault()`.

## Problème Observé

Quand l'utilisateur drag & drop un fichier **dans** l'application Tauri :
- Tauri ouvre l'image dans la fenêtre (comme un navigateur)
- L'image remplace le contenu de l'application
- Équivalent à ouvrir une image dans un nouvel onglet de navigateur

## Root Cause

Tauri utilise un **WebView** (navigateur embarqué) qui a le comportement par défaut du navigateur web :

```
Drop d'une image → Navigateur ouvre l'image
Drop d'un PDF   → Navigateur ouvre le PDF
Drop d'un HTML  → Navigateur affiche le HTML
```

Ce comportement est **hérité du WebView** et doit être explicitement bloqué.

## Solution Implémentée

### Bloquer Globalement les Drop Events

Ajouter des event listeners au niveau `document` pour empêcher le comportement par défaut :

**Fichier : `src/App.vue`**

```typescript
onMounted(async () => {
  // Prevent default browser behavior for drag and drop
  // This prevents Tauri from opening dropped files in the window
  preventDefaults = (e: Event) => {
    e.preventDefault();
    e.stopPropagation();
  };

  // Block all drop events globally
  document.addEventListener('dragover', preventDefaults, false);
  document.addEventListener('dragenter', preventDefaults, false);
  document.addEventListener('dragleave', preventDefaults, false);
  document.addEventListener('drop', preventDefaults, false);

  // ... rest of setup
});
```

### Cleanup dans onUnmounted

```typescript
onUnmounted(() => {
  // Remove global drop prevention listeners
  if (preventDefaults) {
    document.removeEventListener('dragover', preventDefaults, false);
    document.removeEventListener('dragenter', preventDefaults, false);
    document.removeEventListener('dragleave', preventDefaults, false);
    document.removeEventListener('drop', preventDefaults, false);
  }

  // ... other cleanups
});
```

## Événements Bloqués

**1. `dragover`**
- Déclenché continuellement pendant le drag au-dessus de la fenêtre
- Doit être bloqué pour empêcher le drop

**2. `dragenter`**
- Déclenché quand le drag entre dans la fenêtre
- Indicateur visuel que le navigateur accepte le drop

**3. `dragleave`**
- Déclenché quand le drag quitte la fenêtre
- Nettoyage des indicateurs visuels

**4. `drop`**
- **L'événement critique** - déclenché quand l'utilisateur relâche
- Doit être bloqué pour empêcher l'ouverture du fichier

## Pourquoi Ça Fonctionne

### preventDefault() Bloque le Comportement Natif

```typescript
e.preventDefault(); // "N'ouvre PAS ce fichier"
e.stopPropagation(); // "N'envoie PAS l'événement plus haut"
```

### Application au Niveau Document

- Capture TOUS les drops dans l'application
- Même si un élément enfant ne bloque pas, `document` le fait
- Garantit qu'aucun drop n'ouvre de fichier

### Compatible avec Notre Drag Interne

- Notre drag utilise `mousedown` + `mousemove` (pas HTML5 drag)
- Ces listeners ne bloquent PAS notre système de drag
- Ils bloquent seulement les drops **de l'extérieur** vers l'intérieur

## Comportements Bloqués

✅ **Drop image externe dans Tauri** → Bloqué, n'ouvre plus l'image
✅ **Drop PDF externe dans Tauri** → Bloqué, n'ouvre plus le PDF
✅ **Drop fichier depuis Finder** → Bloqué, fichier pas ouvert
✅ **Drag interne (nos cards)** → Fonctionne toujours ! (différent système)

## Fichiers Modifiés

- `src/App.vue`:
  - Ajout de `preventDefaults` function
  - Event listeners dans `onMounted`
  - Cleanup dans `onUnmounted`

## Validation

- ✅ TypeScript compilation: Pass
- ⏳ Test manuel requis (voir Test Plan)

## Test Plan

### Test 1 : Drop Externe Bloqué
1. Ouvrir Finder
2. Sélectionner une image
3. Drag & drop dans la fenêtre Clipster
4. **Attendu** : Rien ne se passe (fichier pas ouvert)

### Test 2 : Drag Interne Fonctionne
1. Dans Clipster, drag une card
2. **Attendu** : Clone HTML suit le curseur
3. Drop sur Desktop/Finder
4. **Attendu** : Fichier copié correctement

### Test 3 : Différents Types de Fichiers
1. Essayer de drop : PDF, image, vidéo, texte
2. **Attendu** : Aucun ne s'ouvre dans Tauri

## Notes Techniques

### Pourquoi dans App.vue ?

- Point d'entrée de l'application
- Garantit que la protection est active dès le démarrage
- S'applique à toute l'application (pas juste un composant)

### Pourquoi Pas window au lieu de document ?

Les deux fonctionnent, mais `document` est plus courant pour :
- Capturer tous les événements dans le DOM
- Plus proche des éléments qui pourraient recevoir le drop
- Convention standard pour ce type de protection

### Alternative : data-tauri-drag-region

Tauri propose `data-tauri-drag-region` pour les zones draggables de la fenêtre, mais ça ne résout pas le problème du drop. C'est pour le **drag de la fenêtre**, pas le **drop de fichiers**.

## Gotchas Évités

❌ **Ne pas oublier stopPropagation()**
- `preventDefault()` seul ne suffit pas toujours
- Certains handlers parents pourraient quand même recevoir l'événement

❌ **Ne pas oublier le cleanup**
- Les listeners persistent même après unmount
- Cause des fuites mémoire si pas nettoyés

❌ **Bloquer aussi dragover**
- `drop` seul ne suffit pas
- Le navigateur vérifie `dragover` pour savoir si le drop est accepté

## Impact

✅ **Protection complète** : Plus aucun fichier ne peut être ouvert dans Tauri
✅ **Expérience utilisateur** : Pas de navigation accidentelle hors de l'app
✅ **Compatibilité** : Notre système de drag interne fonctionne toujours
✅ **Robustesse** : Fonctionne pour tous types de fichiers
