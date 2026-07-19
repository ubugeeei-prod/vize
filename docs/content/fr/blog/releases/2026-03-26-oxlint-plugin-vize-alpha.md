---
title: oxlint-plugin-vize Alpha
description: Un nouveau pont plugin Oxlint JS regroupe les diagnostics Vize Patina dans une seule exécution Oxlint pour les SFC Vue.
---

<!-- Generated translation; source: blog/releases/2026-03-26-oxlint-plugin-vize-alpha.md -->

# `oxlint-plugin-vize` Alpha

<div class="blog-post-meta">
<span class="blog-meta-chip">
<span>
<span class="blog-meta-label">Publié le</span>
<span class="blog-meta-value">26-03-2026</span>
</span>
</span>
<a class="blog-author-card" href="https://github.com/ubugeeei">
<img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
<span class="blog-author-text">
<span class="blog-meta-label">Auteur</span>
<span class="blog-meta-value">ubugeeei</span>
</span>
</a>
</div>

Aujourd’hui, j’ouvre la première alpha de `oxlint-plugin-vize`, un nouveau pont plugin Oxlint JS pour Vize Patina.

L’objectif est simple : garder [Oxlint](https://oxc.rs/docs/guide/usage/linter) principal runner pour les règles JavaScript et TypeScript, tout en permettant à Vize de contribuer avec des diagnostics spécifiques à Vue dans la même exécution. Au lieu de choisir entre Oxlint et Patina, cet alpha consiste à les faire travailler ensemble.

## Ce que c’est

`oxlint-plugin-vize` permet à Oxlint d’exécuter Patina via la liaison native de Vize tout en utilisant le modèle de plugin JS et la configuration des règles d’Oxlint.

Cela signifie qu’un seul `.oxlintrc.json` peut mélanger des règles comme :

- Règles de base d’Oxlint telles que `no-console`
- Le plugin de `vue` intégré à Oxlint
- Règles Vize telles que `vize/vue/require-v-for-key`
- Diagnostics Vue à base de patine comme `vize/vue/no-v-html` et `vize/vue/no-duplicate-attributes`

Le plugin utilise l’espace de noms `vize` et lit les paramètres de `settings.vize`.

## Pourquoi cet alpha est-il important

Patina comprend déjà bien les modèles Vue, mais de nombreuses équipes souhaitent qu’Oxlint reste au centre de leur flux de travail sur les peluches.

Cet alpha est la première étape vers cette forme :

- un ordre de peluches
- un fichier de configuration
- un flux de sortie
- Règles JavaScript et TypeScript natives Rust ainsi que des diagnostics compatibles avec les modèles Vue

Pour les projets Vue, cette combinaison compte. Les règles modèles comme l’absence de `v-for` ou l’utilisation dangereuse de `v-html` devraient pouvoir être placées à côté des règles générales d’Oxlint, au lieu d’exiger un laissez-passer de lint séparé et un format de rapport distinct.

## Exemple de configuration

```json
{
  "plugins": ["vue"],
  "jsPlugins": ["oxlint-plugin-vize"],
  "settings": {
    "vize": {
      "locale": "en",
      "helpLevel": "none"
    }
  },
  "rules": {
    "no-console": "warn",
    "vize/vue/require-v-for-key": "error",
    "vize/vue/no-v-html": "warn",
    "vize/vue/no-duplicate-attributes": "error"
  }
}
```

L’alpha soutient actuellement :

- `settings.vize.locale` pour le langage diagnostique
- `settings.vize.helpLevel` avec `"full"`, `"short"`ou `"none"`
- `showHelp` pour la rétrocompatibilité
- `settings.patina` comme alias de compatibilité tandis que `settings.vize` devient la clé canonique

## Comment ça fonctionne

Le pont est conçu selon le modèle d’exécution par règle d’Oxlint plutôt que de le contester.

- La première règle Vize activée sur un fichier exécute un pass Patina natif uniquement pour cette règle.
- Si une seconde règle Vize est activée pour le même fichier, le plugin passe à un seul passage Patina complet partagé et réutilise le résultat pour les règles Vize restantes.
- Le contenu des fichiers et les résultats des règles sont mis en cache par fichier et par réglage pendant toute la durée de vie du processus Oxlint.

Ce design permet de garder la première règle bon marché tout en évitant le travail natif redondant une fois que plusieurs règles Vize sont actives.

## Diagnostic et sortie

L’un des aspects difficiles de cette intégration est le rapport de localisation.

Le système de plugins JS d’Oxlint fonctionne actuellement à partir du script extrait du programme Vue, tandis que de nombreux diagnostics Patina proviennent de blocs `<template>` ou d’autres SFC. Dans cette alpha, `oxlint-plugin-vize` garde le vrai bloc Vue et `line:column` en ligne dans le message de diagnostic, donc la sortie vous renvoie toujours au bon endroit dans le SFC.

Le dépôt inclut également un petit exemple `examples/oxlint-vize` pour montrer des résultats mitigés à partir de :

- Diagnostic du cœur Oxlint
- Support intégré Vue d’Oxlint
- Diagnostic Vize à dos patine

## Limitations actuelles

C’est encore un alpha, et quelques limites sont importantes à souligner clairement :

- Les plugins JS Oxlint dépendent actuellement du script extrait de Vue, donc les fichiers sans `<script>` ou `<script setup>` n’invoquent pas encore le plugin.
- Les ancrages de diagnostic pointent toujours vers le script programme lorsque Oxlint ne peut pas accepter directement la plage de modèles originale.
- Le package alpha initial visait le Node 24+ ; les versions actuelles prennent en charge Node 22 et Node 24+.
- Le support des plugins JS d’Oxlint est lui-même encore en évolution, donc certains défauts ici sont des contraintes en amont plutôt que des comportements uniquement liés à Vize.

## Pourquoi Alpha Now

Je voulais que cette intégration soit entre les mains des gens dès le début, même avant que chaque cas particulier ne soit peaufiné.

La forme du noyau semble déjà utile :

- Vize apporte une intelligence spécifique à Vue sur les peluches
- Oxlint reste le coureur de premier niveau
- La surface de configuration reste petite
- Le modèle de performance reste le natif avant tout

Cela suffit à commencer à recevoir de vrais retours de la part des utilisateurs de Vue qui veulent une pile de peluches plus rapide sans abandonner les vérifications en fonction des modèles.

## Que suivra-t-il

Les étapes suivantes sont simples :

- améliorer la cartographie de la localisation des modèles à mesure qu’Oxlint expose davantage de hooks de plugins compatibles Vue
- Renforcez le flux d’installation et de publication autour des liaisons natives de la plateforme
- Développer la documentation et les exemples pour de vrais projets
- continue d’affiner la façon dont le texte d’aide Patina apparaît dans les formateurs d’Oxlint

Cet alpha n’est pas l’état final. C’est le premier pont utilisable entre Oxlint et la linting Vue de Vize, et je suis impatient de voir où il nous mènera ensuite.
