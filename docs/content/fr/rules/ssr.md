---
title: "Règles SSR"
description: "Le code qui casse ou se comporte autrement pendant le rendu serveur et l’hydratation."
---

# Règles SSR

Le code qui casse ou se comporte autrement pendant le rendu serveur et l’hydratation.

Chaque page de règle montre un exemple qui échoue, un exemple qui passe et la configuration qui active la règle.

## Par défaut (2)

Ajoutées par `happy-path`, le préréglage par défaut.

| Règle | Ce qu’elle vérifie | Correction automatique |
| --- | --- | --- |
| <span id="ssr-no-browser-globals-in-ssr"></span>[`ssr/no-browser-globals-in-ssr`](./reference/ssr-no-browser-globals-in-ssr.md) | Interdire les variables globales propres au navigateur dans un contexte SSR. |  |
| <span id="ssr-no-hydration-mismatch"></span>[`ssr/no-hydration-mismatch`](./reference/ssr-no-hydration-mismatch.md) | Interdire les valeurs non déterministes qui causent des divergences d’hydratation. |  |

[Vue d’ensemble des règles](../rules/index.md) · [Options des règles](/rules/options.md) · [Migrer depuis ESLint](/rules/migration.md)
