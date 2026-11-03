---
title: "Règles HTML"
description: "Validité du HTML et balisage sémantique dans les templates."
---

# Règles HTML

Validité du HTML et balisage sémantique dans les templates.

Chaque page de règle montre un exemple qui échoue, un exemple qui passe et la configuration qui active la règle.

## Essentielles (1)

Règles de justesse du préréglage `essential`.

| Règle | Ce qu’elle vérifie | Correction automatique |
| --- | --- | --- |
| <span id="html-id-duplication"></span>[`html/id-duplication`](./reference/html-id-duplication.md) | Interdire les identifiants d’éléments en double. |  |

## Par défaut (7)

Ajoutées par `happy-path`, le préréglage par défaut.

| Règle | Ce qu’elle vérifie | Correction automatique |
| --- | --- | --- |
| <span id="html-deprecated-attr"></span>[`html/deprecated-attr`](./reference/html-deprecated-attr.md) | Interdire les attributs HTML obsolètes. |  |
| <span id="html-deprecated-element"></span>[`html/deprecated-element`](./reference/html-deprecated-element.md) | Interdire les éléments HTML obsolètes. |  |
| <span id="html-no-consecutive-br"></span>[`html/no-consecutive-br`](./reference/html-no-consecutive-br.md) | Interdire les éléments &lt;br&gt; consécutifs. |  |
| <span id="html-no-duplicate-class"></span>[`html/no-duplicate-class`](./reference/html-no-duplicate-class.md) | Interdire les noms de classes en double dans un attribut class statique. |  |
| <span id="html-no-duplicate-dt"></span>[`html/no-duplicate-dt`](./reference/html-no-duplicate-dt.md) | Interdire les noms &lt;dt&gt; en double dans &lt;dl&gt;. |  |
| <span id="html-no-empty-palpable-content"></span>[`html/no-empty-palpable-content`](./reference/html-no-empty-palpable-content.md) | Interdire les éléments vides qui attendent un contenu visible. |  |
| <span id="html-require-datetime"></span>[`html/require-datetime`](./reference/html-require-datetime.md) | Exiger l’attribut datetime sur l’élément &lt;time&gt;. |  |

## Préréglages plus stricts (1)

Seulement dans `opinionated`, `nuxt` ou `ecosystem`. La colonne Préréglages indique lesquels.

| Règle | Ce qu’elle vérifie | Préréglages | Correction automatique |
| --- | --- | --- | --- |
| <span id="html-no-dupe-style-properties"></span>[`html/no-dupe-style-properties`](./reference/html-no-dupe-style-properties.md) | Interdire les propriétés en double dans les attributs de style en ligne. | `nuxt`, `opinionated` |  |

[Vue d’ensemble des règles](../rules/index.md) · [Options des règles](/rules/options.md) · [Migrer depuis ESLint](/rules/migration.md)
