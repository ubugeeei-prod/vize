---
title: Styles d'UI
---

<!-- Generated translation; source: guide/ui-styles.md -->

# Styles d'UI

Les composants de `@vizejs/ui` fonctionnent sans feuille de style visuelle. Pour utiliser les styles optionnels de Vize, importez la base, une palette et les composants utilisés par votre page :

```ts
import "@vizejs/ui/base.css";
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/component-button.css";
import "@vizejs/ui/component-input.css";
import "@vizejs/ui/component-textarea.css";
import "@vizejs/ui/component-checkbox.css";
import "@vizejs/ui/component-switch.css";
import "@vizejs/ui/component-dialog.css";
import "@vizejs/ui/component-card.css";
import "@vizejs/ui/component-badge.css";
import "@vizejs/ui/component-alert.css";
import "@vizejs/ui/component-tooltip.css";
// Add these only when their low-level behavior is used without the JS entry:
// import "@vizejs/ui/component-progress-bar.css";
// import "@vizejs/ui/component-scroll-area.css";
// import "@vizejs/ui/motion.css";
```

```vue
<template>
  <main data-vize-theme="paper">
    <Button>Save changes</Button>
  </main>
</template>
```

`base.css` fournit les jetons sémantiques, la densité et la politique forced-colors ; c'est un alias de l'export existant `theme.css`. Il ne réinitialise pas les éléments de l'application et ne modifie pas les composants Vize sans style. Les fichiers de composant sont de simples ressources CSS et ne changent pas l'API JavaScript ou Vue. Importer un composant seul n'ajoute les règles visuelles d'aucun autre composant.

Input, Textarea, Checkbox et Switch ont chacun leur propre export CSS. Les champs de texte conservent le comportement natif d'édition et de redimensionnement ; Checkbox distingue l'état coché de l'état mixte, tandis que Switch déplace son curseur quand l'état change. Les petites transitions d'état s'arrêtent sous `prefers-reduced-motion`. Le focus reste visible pour les utilisateurs du clavier, et le mode forced-colors conserve les coches natives des cases à cocher. Les fichiers de composant réinitialisent leurs surcharges locales à chaque frontière de thème, de sorte qu'un formulaire Paper imbriqué dans une page Signal conserve la typographie et les proportions de Paper.

Card, Badge, Alert et Tooltip ont eux aussi chacun un fichier visuel distinct. Card utilise ses hooks `variant`, `density` et `tone` ; Badge distingue les libellés, les compteurs et le texte de statut ; Alert suit sa variante de live region sans ajouter de bouton de fermeture. Tooltip garde visible le focus clavier du déclencheur et ne démarre sa brève entrée qu'après la mesure de sa position flottante. Les surfaces statiques de Card ne sont pas animées. Les changements de ton de Badge et les entrées d'Alert/Tooltip s'arrêtent sous `prefers-reduced-motion` ; le mode forced-colors rétablit les bordures système. Ces styles sont réinitialisés aux frontières de thème imbriquées, y compris les hôtes Shadow DOM.

Les recettes existantes de structure et de mouvement de ProgressBar et ScrollArea sont également publiées sous forme de fichiers autonomes uniquement CSS. Leurs entrées JavaScript chargent déjà la feuille de style agrégée héritée pour le comportement requis. Importez le fichier autonome lorsque vous utilisez les hooks CSS sans l'entrée JavaScript, ou lorsque vous contrôlez explicitement les feuilles de style ; évitez d'importer les deux chemins sur une même page.

| Préréglage | Caractère                                                             |
| ---------- | --------------------------------------------------------------------- |
| `paper`    | Papier chaleureux, encre, bordures fines et contrôles à angles droits |
| `signal`   | Surfaces graphite denses aux arêtes nettes et à faible profondeur     |
| `atelier`  | Neutres d'atelier sobres avec un unique accent retenu                 |

Pour proposer un sélecteur de style, remplacez l'import du préréglage unique ci-dessus par les préréglages que vous souhaitez proposer. Chaque feuille de style ne s'active que dans sa propre portée :

```ts
import "@vizejs/ui/theme-preset-paper.css";
import "@vizejs/ui/theme-preset-signal.css";
import "@vizejs/ui/theme-preset-atelier.css";

document.documentElement.dataset.vizeTheme = "signal";
```

Les préréglages existants `midnight`, `play`, `high-contrast` et `headless` restent disponibles. Définissez `data-vize-theme` sur `<html>` lorsqu'une boîte de dialogue ou une infobulle est téléportée dans le body du document, afin que le contenu flottant hérite de la même palette que la page. Pour une préférence enregistrée, `@vizejs/ui/theme-scope` fournit un script d'amorçage exécuté avant le rendu. Les anciens imports `theme.css`, `theme-preset-*.css` et `style.css` continuent de fonctionner ; évitez d'importer `style.css` avec `base.css`, car il contient déjà la base et tous les préréglages hérités.

Button offre un bref retour visuel au survol, à l'appui et au focus. Avec `dialog.css`, Dialog anime son apparition et une sortie de 200ms. La fermeture libère immédiatement le confinement du focus, l'inert extérieur et le verrouillage du défilement ; le panneau sortant reste inert et masqué des technologies d'assistance jusqu'à la fin de son animation. Le Dialog headless est toujours démonté immédiatement, tout comme le Dialog stylé lorsque `prefers-reduced-motion: reduce` correspond. Les deux fichiers visuels conservent des bordures nettes en mode forced-colors. Le CSS de l'application situé hors des cascade layers de Vize peut surcharger n'importe quelle règle.
