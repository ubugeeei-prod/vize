---
title: Diagnostic d’analyse
---

<!-- Generated translation; source: guide/analysis-diagnostics.md -->

# Diagnostic d’analyse

Cette page explique comment les diagnostics Vize sont organisés. La référence détaillée des règles se trouve désormais dans la section Règles
afin que chaque règle puisse garder ensemble son comportement, sa sévérité par défaut, sa couverture prédéfinie et les exemples de
Mauvais/Bon.

## Référence des règles

- [Rules overview](../rules/index.md)
- [Vue rules](../rules/vue.md)
- [Accessibility rules](../rules/accessibility.md)
- [Type and script rules](../rules/type-and-script.md)
- [HTML rules](../rules/html.md)
- [SSR rules](../rules/ssr.md)
- [Vapor rules](../rules/vapor.md)
- [Cross-file rules](../rules/cross-file.md)
- [Musea and CSS rules](../rules/musea-and-css.md)

## Familles diagnostiques

Les règles de patine sont des règles de peluches en file indienne. Ils utilisent des noms tels que `vue/require-v-for-key` et peuvent être configurés
depuis `vize.config.*`, la ligne de cli, l’API JavaScript et le pont Oxlint.

Les diagnostics croisés utilisent des codes `vize:croquis/cf/*`. Ils sont émis par
`vize lint --cross-file` après que Vize ait construit un graphe de projet, afin de comparer les fournisseurs avec des injecteurs de
, de suivre les identifiants dupliqués et de repérer les risques de réactivité à travers les frontières des composants.

Les diagnostics sensibles au type utilisent le vérificateur TypeScript. Ils ont besoin de la même configuration de projet que
TypeScript voit à travers `tsconfig.json`, incluant `compilerOptions.types`, `paths`et les références de projet
. Vize ne nécessite pas de liste de `globals` séparée pour ces noms.

Les diagnostics de Musea et CSS sont des règles soutenues par des bibliothèques. Ils s’exécutent lorsque les blocs d’art de Musea ou les
de contenu de style sont analysés et documentés séparément car ils ne font pas partie de la règle standard du modèle Vue
surface.
