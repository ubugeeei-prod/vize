---
title: "Règles SSR"
---

# Règles SSR

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.


| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](https://vizejs.dev/fr/rules/ssr.html#ssr-no-browser-globals-in-ssr) | [Mauvais](https://vizejs.dev/fr/rules/ssr.html#ssr-no-browser-globals-in-ssr-bad) · [Bon](https://vizejs.dev/fr/rules/ssr.html#ssr-no-browser-globals-in-ssr-good) | Interdire les variables globales propres au navigateur dans un contexte SSR |
| [`ssr/no-hydration-mismatch`](https://vizejs.dev/fr/rules/ssr.html#ssr-no-hydration-mismatch) | [Mauvais](https://vizejs.dev/fr/rules/ssr.html#ssr-no-hydration-mismatch-bad) · [Bon](https://vizejs.dev/fr/rules/ssr.html#ssr-no-hydration-mismatch-good) | Interdire les valeurs non déterministes qui causent des divergences d’hydratation |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)
