---
title: "script/consistent-type-imports"
description: "Uniformiser l’usage des imports de type, en comptant les usages du template comme des usages de valeur."
---

# `script/consistent-type-imports`

Uniformiser l’usage des imports de type, en comptant les usages du template comme des usages de valeur.

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | Aucune | Oui | [Options des règles](/rules/options.md) |

Champ d’application : Scripts des SFC Vue et modules TypeScript autonomes (`.ts`).

## Ce qui compte comme un usage de valeur

Chaque usage qu’un composant fait d’un import compte, d’après la même analyse des liaisons que lit le compilateur : une expression du template, une balise de composant dans toute casse que Vue résout, une directive personnalisée, une ref de template, un `v-bind()` dans `<style>` et l’autre bloc de script.

Un alias de `v-for` ou de slot du même nom masque l’import. Une position de type, comme `(item as Foo)` dans le template ou l’attribut `generic`, est un usage de type.

Lorsqu’un bloc ne peut pas être lu (un bloc externe `src`, une expression qui ne s’analyse pas, un `eval` direct, le balisage `<variant>` de Musea), la règle ne signale rien pour ce composant.

Un nom importé avec `import type` qu’une expression du template ou un `v-bind()` dans `<style>` évalue est également signalé, car cette liaison n’existe pas à l’exécution. La lecture dans le template est laissée à `vue/no-undefined-refs` lorsque cette règle est active. Les balises de composant et les directives ne sont pas signalées : un composant enregistré ou importé automatiquement peut les rendre.

## Options

`prefer`, `fixStyle` et `disallowTypeAnnotations` sont les options de typescript-eslint. `emitDecoratorMetadata` (`true` par défaut) indique si le projet émet les métadonnées des décorateurs, ce que le lint ne lit pas dans `tsconfig.json`. Avec `true`, un type qu’une signature décorée sérialise compte comme un usage de valeur.

## La correction

Si la déclaration scindée conserve un import de valeur, son ordre d’évaluation reste inchangé. Convertir toute la déclaration en `import type` peut supprimer l’initialisation du module avec `verbatimModuleSyntax`. Une déclaration à scinder qui contient un commentaire ou des attributs d’import est signalée sans correction.

## Mauvais

`Foo` est importé comme une valeur, alors que son seul usage est l’argument de type de `defineProps`. `Bar` n’est pas signalé : le template lit `Bar.label`, ce qui est un usage de valeur.

```vue annotate="remove:3"
<script setup lang="ts">
import { Bar } from "./Foo";
import { Foo } from "./Foo";

defineProps<{ item: Foo }>();
</script>

<template>
  <p>{{ Bar.label }} {{ item.label }}</p>
</template>
```

## Bon

`Foo` est importé avec `import type` ; l’import est donc effacé à la compilation. `Bar` reste un import de valeur, car le template l’évalue.

```vue annotate="add:3"
<script setup lang="ts">
import { Bar } from "./Foo";
import type { Foo } from "./Foo";

defineProps<{ item: Foo }>();
</script>

<template>
  <p>{{ Bar.label }} {{ item.label }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/consistent-type-imports": "warn"
    }
  },
});
```

Lancez ensuite `vp run lint`.

## Différences avec typescript-eslint

typescript-eslint a besoin de `vue-eslint-parser` pour voir un usage dans le template, et oxlint ignore les fichiers `.vue` pour cette règle. Une signature décorée compte comme un usage de valeur, sauf si `emitDecoratorMetadata` vaut `false` ; typescript-eslint lit `tsconfig.json` et ignore alors tout fichier contenant un décorateur. La correction ne fusionne jamais avec une autre déclaration. Un import de type qu’une expression du template ou un `v-bind()` dans `<style>` évalue est signalé.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/consistent_type_imports.rs#L55)
