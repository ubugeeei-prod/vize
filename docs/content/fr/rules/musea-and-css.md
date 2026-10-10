---
title: "Règles Musea et CSS"
---

# Règles Musea et CSS

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.

<span id="règles-de-la-musea-et-du-css"></span>
<span id="règles-css-supplémentaires"></span>

| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`css/no-display-none`](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-display-none) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-display-none-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-display-none-good) | Suggérer v-show plutôt que display: none |
| [`css/no-hardcoded-values`](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-hardcoded-values) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-hardcoded-values-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-hardcoded-values-good) | Suggérer des variables CSS plutôt que des valeurs codées en dur |
| [`css/no-id-selectors`](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-id-selectors) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-id-selectors-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-id-selectors-good) | Déconseiller les sélecteurs d’identifiant en CSS |
| [`css/no-important`](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-important) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-important-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-important-good) | Déconseiller !important en CSS |
| [`css/no-utility-classes`](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-utility-classes) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-utility-classes-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-utility-classes-good) | Déconseiller l’implémentation de classes utilitaires dans les styles des composants |
| [`css/no-v-bind-performance`](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-v-bind-performance) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-v-bind-performance-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#css-no-v-bind-performance-good) | Signaler le coût de performance de v-bind() en CSS |
| [`css/prefer-logical-properties`](https://vizejs.dev/fr/rules/musea-and-css.html#css-prefer-logical-properties) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#css-prefer-logical-properties-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#css-prefer-logical-properties-good) | Recommander les propriétés logiques CSS pour mieux prendre en charge l’internationalisation |
| [`css/prefer-nested-selectors`](https://vizejs.dev/fr/rules/musea-and-css.html#css-prefer-nested-selectors) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#css-prefer-nested-selectors-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#css-prefer-nested-selectors-good) | Recommander l’imbrication CSS pour les sélecteurs de descendants |
| [`css/prefer-slotted`](https://vizejs.dev/fr/rules/musea-and-css.html#css-prefer-slotted) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#css-prefer-slotted-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#css-prefer-slotted-good) | Recommander ::v-slotted() pour styliser le contenu des slots |
| [`css/require-font-display`](https://vizejs.dev/fr/rules/musea-and-css.html#css-require-font-display) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#css-require-font-display-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#css-require-font-display-good) | Exiger font-display dans les règles @font-face |
| [`musea/no-empty-variant`](https://vizejs.dev/fr/rules/musea-and-css.html#musea-no-empty-variant) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#musea-no-empty-variant-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#musea-no-empty-variant-good) | Interdire les blocs &lt;variant&gt; vides |
| [`musea/prefer-design-tokens`](https://vizejs.dev/fr/rules/musea-and-css.html#musea-prefer-design-tokens) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#musea-prefer-design-tokens-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#musea-prefer-design-tokens-good) | Préférer les variables CSS de design tokens aux valeurs primitives codées en dur |
| [`musea/require-component`](https://vizejs.dev/fr/rules/musea-and-css.html#musea-require-component) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#musea-require-component-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#musea-require-component-good) | Exiger l’attribut component dans le bloc &lt;art&gt; |
| [`musea/require-title`](https://vizejs.dev/fr/rules/musea-and-css.html#musea-require-title) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#musea-require-title-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#musea-require-title-good) | Exiger l’attribut title dans le bloc &lt;art&gt; |
| [`musea/unique-variant-names`](https://vizejs.dev/fr/rules/musea-and-css.html#musea-unique-variant-names) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#musea-unique-variant-names-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#musea-unique-variant-names-good) | Exiger des noms de variants uniques |
| [`musea/valid-variant`](https://vizejs.dev/fr/rules/musea-and-css.html#musea-valid-variant) | [Mauvais](https://vizejs.dev/fr/rules/musea-and-css.html#musea-valid-variant-bad) · [Bon](https://vizejs.dev/fr/rules/musea-and-css.html#musea-valid-variant-good) | Exiger un attribut name dans les blocs &lt;variant&gt; |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)
