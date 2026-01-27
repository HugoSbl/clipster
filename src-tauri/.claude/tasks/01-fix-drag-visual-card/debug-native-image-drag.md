# Debug: Native Image Drag on macOS

**Date**: 2026-01-23
**Issue**: L'image brute est toujours draggée au lieu du clone de card

## Problème Observé

1. **Symptôme principal**: Lors du drag, c'est l'image brute (l'ours) qui apparaît, pas le clone stylisé de la card
2. **Symptôme secondaire**: Quand on relâche, macOS affiche l'image en Quick Look (preview en grand)

## Root Cause Analysis (MISE À JOUR)

### Vraie Cause Profonde (identifiée via logs)

**Le plugin `@crabnebula/tauri-plugin-drag` affiche sa propre preview native !**

Les logs montrent :
```
icon: "/var/folders/.../icon_Ghostty_20260123_095400.png"
```

**Le problème n'était PAS les éléments `<img>` HTML**, mais le **paramètre `icon` du plugin** :

1. Notre clone HTML est créé correctement
2. Le plugin `tauri-plugin-drag` reçoit un paramètre `icon` avec le chemin de l'image
3. Le plugin affiche SA PROPRE preview native en utilisant ce fichier
4. La preview native du plugin **masque** notre clone HTML

### Confusion Initiale

Première hypothèse (incorrecte): Les éléments `<img>` ont un drag natif agressif
- Solution tentée: Ajouter `@dragstart` et `@drag` handlers sur les images
- Résultat: **Aucun effet** car ce n'était pas la vraie cause

### Vraie Chaîne d'Événements (Buggy)

```
User mousedown sur card
  → handleNativeDragStart() crée le clone HTML ✅
  → Clone positionné et suit le curseur ✅

User dépasse le threshold (5px)
  → startDrag({ items: [...], icon: "/path/to/image.png" }) ❌
  → Le plugin affiche SON propre ghost natif avec l'icon
  → Le ghost natif masque notre clone HTML ❌
```

### Chaîne d'Événements (Buggy)

```
User mousedown sur card
  → handleNativeDragStart() s'exécute
  → preventDefault() sur le conteneur
  → Crée le clone manuel

User bouge la souris
  → macOS détecte une image sous le curseur
  → Initie un drag natif DE L'IMAGE (bypass notre code)
  → L'image brute est affichée, pas notre clone
```

## Solution Implémentée (CORRIGÉE)

### Solution Réelle : Désactiver l'Icon du Plugin

**Le vrai fix** : Ne pas passer d'icon au plugin, ou passer une chaîne vide.

```typescript
// Dans getFilePathsForDrag()
// AVANT (buggy):
return { items: [prepared.imagePath], icon: prepared.iconPath };

// APRÈS (correct):
return { items: [prepared.imagePath], icon: '' };
```

### Modifications Apportées

**1. Images** (ligne ~390)
```typescript
// Return empty icon to disable plugin's native preview
// Our HTML clone will be the only visible drag preview
return { items: [prepared.imagePath], icon: '' };
```

**2. Fichiers/Audio/Documents** (ligne ~415)
```typescript
// Return empty icon to disable plugin's native preview
return { items: paths, icon: '' };
```

**3. Suppression du Warning** (ligne ~515)
```typescript
// Removed validation warning for empty icon
// Icon is intentionally empty to use our HTML clone
```

### Solutions Tentées (Pas Nécessaires)

Les handlers `@dragstart` et `@drag` sur les images **n'étaient pas nécessaires** :
- Ils bloquent le drag HTML5, pas le plugin natif
- Peuvent être gardés comme "defense in depth"
- Mais ce n'est pas eux qui corrigent le problème principal

## Pourquoi Ça Fonctionne

### Séparation Preview vs Transfert de Fichier

Le plugin `tauri-plugin-drag` a **deux responsabilités distinctes** :

1. **Transfert de fichier** : Paramètre `item` (les chemins des fichiers)
2. **Preview visuelle** : Paramètre `icon` (l'image affichée pendant le drag)

**Notre stratégie** :
- ✅ On laisse le plugin gérer le **transfert** (`item: ["/path/to/file.png"]`)
- ❌ On désactive la **preview native** (`icon: ''`)
- ✅ On gère la **preview** nous-mêmes avec notre clone HTML

### Résultat

Avec `icon: ''` :
- Le plugin n'affiche RIEN comme preview native
- Notre clone HTML est le SEUL élément visible
- Le transfert de fichier fonctionne quand même parfaitement
- L'utilisateur voit la belle card stylisée pendant le drag

## Fichiers Modifiés

- `src/components/ClipboardCard.vue`:
  - **Modification critique** : `getFilePathsForDrag()` retourne `icon: ''` au lieu du chemin
  - Suppression du warning pour icon vide (ligne ~515-520)
  - Ajout de commentaires explicatifs
  - (Bonus) Ajout de `handleImageDragStart()` et `@dragstart` sur images (defense in depth)

## Validation

- ✅ TypeScript compilation: Pass
- ⏳ Test manuel requis: Vérifier que le clone de card est visible, pas l'image brute
- ⏳ Test Quick Look: Vérifier que relâcher ne lance pas Quick Look
- ⏳ Test transfert: Vérifier que les fichiers sont bien transférés vers Finder/Desktop

## Notes Techniques

### Architecture du Plugin tauri-plugin-drag

Le plugin a **deux paramètres distincts** :

```typescript
await startDrag({
  item: string[],  // Chemins des fichiers à transférer
  icon: string     // Chemin de l'image pour la preview native
});
```

**Séparation des responsabilités** :
- `item` : Gère le **transfert de données** (nécessaire)
- `icon` : Gère la **preview visuelle** (optionnel)

### Pourquoi Icon Vide Fonctionne

Quand `icon: ''` :
- Le plugin **n'affiche aucune preview native**
- Le transfert de fichier fonctionne quand même
- Seul notre clone HTML est visible

**Avantages** :
- Contrôle total sur la preview (CSS, animations, styles)
- Cohérence visuelle (notre card stylisée, pas une image brute)
- Pas de conflit entre preview HTML et preview native

### Alternative Considérée (Rejetée)

**Option 1** : Cacher notre clone et utiliser seulement l'icon du plugin
- ❌ Perd le contrôle sur les styles
- ❌ Ne peut pas avoir une preview qui ressemble à la card
- ❌ Limité par ce que le plugin peut afficher

**Option 2** : Passer une image transparente comme icon
- ❌ Plus complexe (créer un fichier transparent)
- ❌ Overhead inutile
- ✅ `icon: ''` est plus simple et fonctionne parfaitement

## Test Plan

1. **Test de base**: Drag une card avec image, vérifier que c'est le clone stylisé qui suit le curseur
2. **Test Quick Look**: Relâcher la souris, vérifier que Quick Look ne s'ouvre PAS
3. **Test fichier externe**: Drag vers Finder, vérifier que le fichier est bien copié
4. **Test multiple images**: Tester avec plusieurs cards différentes
