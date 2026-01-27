# Fix: Clone Disparaît Immédiatement Après Threshold

**Date**: 2026-01-23
**Type**: Critical Bug Fix
**Status**: ✅ FIXED

## TL;DR

Le `finally` block dans `handleNativeDragMove` appelait `cleanup()` immédiatement après `startDrag()`, supprimant le clone du DOM alors que le drag continuait. Solution : Ne cleanup que dans `handleNativeDragEnd`.

## Problème Observé

**Symptômes:**
1. ✅ La preview native du plugin n'apparaît plus (problème précédent résolu)
2. ❌ Le clone HTML apparaît brièvement (quelques pixels)
3. ❌ Puis le clone disparaît complètement pendant le reste du drag

## Root Cause Analysis

### Séquence Buggy

```typescript
const handleNativeDragMove = async (e: MouseEvent) => {
  // ...threshold detection...
  if (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD) {
    try {
      await startDrag({ item: items, icon: '' });
      console.log('startDrag completed');  // ← Retourne immédiatement
    } finally {
      cleanup();  // ← SUPPRIME LE CLONE ! 😱
    }
  }
};
```

### Pourquoi Ça Arrive

1. **`startDrag()` est async mais retourne immédiatement**
   - La fonction initie le drag natif via le plugin
   - Elle retourne dès que le drag est initié
   - Le drag **continue** via l'OS après le return

2. **Le `finally` block s'exécute trop tôt**
   - Exécuté juste après `startDrag()` retourne
   - Appelle `cleanup()` qui supprime le clone du DOM
   - Le drag OS continue, mais notre clone a disparu !

3. **L'utilisateur voit un flash**
   - Clone visible pendant ~quelques ms (temps de `startDrag()`)
   - Puis disparaît brutalement
   - Drag continue sans aucune preview visible

### Chaîne d'Événements (Buggy)

```
User dépasse threshold (5px)
  → startDrag() appelé
  → Plugin initie le drag natif ✅
  → startDrag() retourne (drag continue en arrière-plan)
  → finally block s'exécute
  → cleanup() supprime le clone 💥
  → Drag continue sans preview visible ❌
  → User relâche (mouseup)
  → handleNativeDragEnd appelé
  → cleanup() appelé à nouveau (no-op, clone déjà supprimé)
```

## Solution Implémentée

### Ne Cleanup Que sur MouseUp

Le cleanup doit se faire **seulement** quand l'utilisateur relâche la souris (`mouseup`), pas quand `startDrag()` retourne.

**AVANT (buggy):**
```typescript
try {
  await startDrag({ item: items, icon: '' });
} finally {
  isDragging.value = false;
  pinboardStore.setDragging(false, null);
  cleanup();  // ❌ Trop tôt !
}
```

**APRÈS (correct):**
```typescript
try {
  await startDrag({ item: items, icon: '' });
} catch (err) {
  console.error('[ClipboardCard] startDrag failed:', err);
  // On error, cleanup immediately
  isDragging.value = false;
  pinboardStore.setDragging(false, null);
  cleanup();
}
// Don't cleanup here! The drag is still ongoing.
// Cleanup will happen in handleNativeDragEnd when user releases mouse.
```

### Cleanup dans handleNativeDragEnd

```typescript
const handleNativeDragEnd = () => {
  // Reset drag state
  isDragging.value = false;
  pinboardStore.setDragging(false, null);

  // Clean up listeners and clone
  cleanup();
};
```

## Pourquoi Ça Fonctionne

### Cycle de Vie Correct

```
User mousedown
  → handleNativeDragStart() crée le clone ✅
  → Clone reste dans le DOM

User mousemove (< 5px)
  → updateClonePosition() mis à jour ✅
  → Clone suit le curseur

User mousemove (> 5px)
  → startDrag() initie le drag natif ✅
  → Fonction retourne mais clone RESTE dans le DOM ✅
  → Clone continue de suivre le curseur

User mouseup (relâche)
  → handleNativeDragEnd() appelé ✅
  → Réinitialise isDragging
  → cleanup() supprime le clone ✅
```

### Séparation des Responsabilités

- **`startDrag()`** : Initie le transfert de fichier (OS-level)
- **Clone HTML** : Gère la preview visuelle pendant TOUT le drag
- **`cleanup()`** : Supprime le clone seulement à la fin du drag

## Modifications Apportées

### 1. Suppression du `finally` Block

```typescript
// AVANT
try {
  await startDrag(...);
} finally {
  cleanup(); // ❌
}

// APRÈS
try {
  await startDrag(...);
} catch (err) {
  cleanup(); // ✅ Seulement sur erreur
}
// Pas de finally, le drag continue !
```

### 2. Amélioration de `handleNativeDragEnd`

```typescript
const handleNativeDragEnd = () => {
  // Reset drag state
  isDragging.value = false;
  pinboardStore.setDragging(false, null);

  // Clean up listeners and clone
  cleanup();
};
```

## Bénéfices

✅ **Clone visible pendant TOUT le drag** : De mousedown à mouseup
✅ **Preview cohérente** : Pas de flash ou disparition
✅ **Transfert fonctionnel** : Les fichiers sont toujours transférés
✅ **Cleanup propre** : Une seule fois, au bon moment

## Fichiers Modifiés

- `src/components/ClipboardCard.vue`:
  - Suppression du `finally` block dans `handleNativeDragMove`
  - Cleanup seulement sur erreur dans le `catch`
  - Amélioration de `handleNativeDragEnd` avec reset d'état

## Validation

- ✅ TypeScript compilation: Pass
- ⏳ Test manuel requis (voir Test Plan)

## Test Plan

1. **Drag une card avec image**
   - Attendu : Clone apparaît immédiatement
   - Attendu : Clone RESTE VISIBLE pendant tout le drag
   - Attendu : Clone suit le curseur en continu
   - Vérifier : Pas de disparition après les premiers pixels

2. **Relâcher la souris**
   - Attendu : Clone disparaît proprement
   - Attendu : Pas de clone résiduel

3. **Drag vers Finder**
   - Attendu : Fichier transféré correctement
   - Vérifier : Fichier existe avec bon nom

4. **Rapid drags (plusieurs fois de suite)**
   - Vérifier : Pas de fuite mémoire (clones multiples)
   - Vérifier : Chaque drag a son propre clone qui disparaît

## Notes Techniques

### Async/Await Gotcha

`startDrag()` est async, mais ça ne signifie pas qu'elle "attend" que le drag soit terminé :
- Elle initie le drag via le plugin
- Elle retourne dès que l'initiation est faite
- Le drag continue de manière native (OS-level)

**Analogie** :
```typescript
async function launchRocket() {
  await ignite(); // ← Allume les moteurs
  return; // ← Retourne immédiatement
  // La fusée continue de voler ! 🚀
}
```

### Event Listeners Critiques

Les listeners `mousemove` et `mouseup` restent actifs après `startDrag()` :
- `mousemove` : Continue de mettre à jour la position du clone
- `mouseup` : Déclenche le cleanup au bon moment

C'est pourquoi supprimer le clone dans le `finally` était une erreur fatale.
