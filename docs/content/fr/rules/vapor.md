---
title: "Règles Vapor"
---

# Règles Vapor

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.

<span id="règles-de-la-vapeur"></span>

| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`script/no-get-current-instance`](https://vizejs.dev/fr/rules/vapor.html#script-no-get-current-instance) | [Mauvais](https://vizejs.dev/fr/rules/vapor.html#script-no-get-current-instance-bad) · [Bon](https://vizejs.dev/fr/rules/vapor.html#script-no-get-current-instance-good) | Interdire getCurrentInstance() en mode Vapor (renvoie null) |
| [`script/no-next-tick`](https://vizejs.dev/fr/rules/vapor.html#script-no-next-tick) | [Mauvais](https://vizejs.dev/fr/rules/vapor.html#script-no-next-tick-bad) · [Bon](https://vizejs.dev/fr/rules/vapor.html#script-no-next-tick-good) | Interdire l’utilisation de nextTick() dans les composants destinés à Vapor |
| [`script/no-options-api`](https://vizejs.dev/fr/rules/vapor.html#script-no-options-api) | [Mauvais](https://vizejs.dev/fr/rules/vapor.html#script-no-options-api-bad) · [Bon](https://vizejs.dev/fr/rules/vapor.html#script-no-options-api-good) | Interdire les formes de l’Options API en mode Vapor |
| [`vapor/no-inline-template`](https://vizejs.dev/fr/rules/vapor.html#vapor-no-inline-template) | [Mauvais](https://vizejs.dev/fr/rules/vapor.html#vapor-no-inline-template-bad) · [Bon](https://vizejs.dev/fr/rules/vapor.html#vapor-no-inline-template-good) | Interdire l’attribut obsolète inline-template |
| [`vapor/no-vue-lifecycle-events`](https://vizejs.dev/fr/rules/vapor.html#vapor-no-vue-lifecycle-events) | [Mauvais](https://vizejs.dev/fr/rules/vapor.html#vapor-no-vue-lifecycle-events-bad) · [Bon](https://vizejs.dev/fr/rules/vapor.html#vapor-no-vue-lifecycle-events-good) | Interdire les événements de cycle de vie @vue:xxx par élément (non pris en charge dans Vapor) |
| [`vapor/prefer-static-class`](https://vizejs.dev/fr/rules/vapor.html#vapor-prefer-static-class) | [Mauvais](https://vizejs.dev/fr/rules/vapor.html#vapor-prefer-static-class-bad) · [Bon](https://vizejs.dev/fr/rules/vapor.html#vapor-prefer-static-class-good) | Préférer une classe statique à une liaison de classe dynamique pour les chaînes littérales |
| [`vapor/require-vapor-attribute`](https://vizejs.dev/fr/rules/vapor.html#vapor-require-vapor-attribute) | [Mauvais](https://vizejs.dev/fr/rules/vapor.html#vapor-require-vapor-attribute-bad) · [Bon](https://vizejs.dev/fr/rules/vapor.html#vapor-require-vapor-attribute-good) | Suggérer l’ajout de l’attribut vapor à script setup |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)
