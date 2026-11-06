---
title: "Règles Musea"
description: "Les stories `.art.vue`, les variantes et l’usage des jetons de design de Musea."
---

# Règles Musea

Les stories `.art.vue`, les variantes et l’usage des jetons de design de Musea.

Chaque page de règle montre un exemple qui échoue, un exemple qui passe et la configuration qui active la règle.

## À activer (6)

Dans aucun préréglage. Activez-les par leur nom dans `linter.rules`, passé à `withVue`.

| Règle | Ce qu’elle vérifie | Correction automatique |
| --- | --- | --- |
| <span id="musea-no-empty-variant"></span>[`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | Interdire les blocs &lt;variant&gt; vides. |  |
| <span id="musea-prefer-design-tokens"></span>[`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | Préférer les variables CSS de design tokens aux valeurs primitives codées en dur. |  |
| <span id="musea-require-component"></span>[`musea/require-component`](./reference/musea-require-component.md) | Exiger l’attribut component dans le bloc &lt;art&gt;. |  |
| <span id="musea-require-title"></span>[`musea/require-title`](./reference/musea-require-title.md) | Exiger l’attribut title dans le bloc &lt;art&gt;. |  |
| <span id="musea-unique-variant-names"></span>[`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | Exiger des noms de variants uniques. |  |
| <span id="musea-valid-variant"></span>[`musea/valid-variant`](./reference/musea-valid-variant.md) | Exiger un attribut name dans les blocs &lt;variant&gt;. |  |

[Vue d’ensemble des règles](../rules/index.md) · [Options des règles](/rules/options.md) · [Migrer depuis ESLint](/rules/migration.md)
