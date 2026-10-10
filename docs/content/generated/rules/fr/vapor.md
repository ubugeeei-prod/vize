---
title: "Règles Vapor"
---

# Règles Vapor

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.

<span id="règles-de-la-vapeur"></span>

| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [Mauvais](#script-no-get-current-instance-bad) · [Bon](#script-no-get-current-instance-good) | Interdire getCurrentInstance() en mode Vapor (renvoie null) |
| [`script/no-next-tick`](#script-no-next-tick) | [Mauvais](#script-no-next-tick-bad) · [Bon](#script-no-next-tick-good) | Interdire l’utilisation de nextTick() dans les composants destinés à Vapor |
| [`script/no-options-api`](#script-no-options-api) | [Mauvais](#script-no-options-api-bad) · [Bon](#script-no-options-api-good) | Interdire les formes de l’Options API en mode Vapor |
| [`vapor/no-inline-template`](#vapor-no-inline-template) | [Mauvais](#vapor-no-inline-template-bad) · [Bon](#vapor-no-inline-template-good) | Interdire l’attribut obsolète inline-template |
| [`vapor/no-vue-lifecycle-events`](#vapor-no-vue-lifecycle-events) | [Mauvais](#vapor-no-vue-lifecycle-events-bad) · [Bon](#vapor-no-vue-lifecycle-events-good) | Interdire les événements de cycle de vie @vue:xxx par élément (non pris en charge dans Vapor) |
| [`vapor/prefer-static-class`](#vapor-prefer-static-class) | [Mauvais](#vapor-prefer-static-class-bad) · [Bon](#vapor-prefer-static-class-good) | Préférer une classe statique à une liaison de classe dynamique pour les chaînes littérales |
| [`vapor/require-vapor-attribute`](#vapor-require-vapor-attribute) | [Mauvais](#vapor-require-vapor-attribute-bad) · [Bon](#vapor-require-vapor-attribute-good) | Suggérer l’ajout de l’attribut vapor à script setup |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `script/no-get-current-instance`

Interdire getCurrentInstance() en mode Vapor (renvoie null)

[Mauvais](#script-no-get-current-instance-bad) · [Bon](#script-no-get-current-instance-good)

Gravité par défaut: `error`  
Préréglages: `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Contrôles de script destinés à Vapor ; une activation explicite applique aussi la restriction aux scripts ordinaires  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-get-current-instance-bad"></span>

**Mauvais**

Le setup marqué Vapor importe et appelle `getCurrentInstance`, s’appuyant sur une API d’instance que cette règle interdit pour les composants destinés à Vapor.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

<span id="script-no-get-current-instance-good"></span>

**Bon**

`inject("app-config")` obtient la configuration explicitement fournie sans importer ni appeler `getCurrentInstance`.

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [Toutes les règles](all.md)

### `script/no-next-tick`

Interdire l’utilisation de nextTick() dans les composants destinés à Vapor

[Mauvais](#script-no-next-tick-bad) · [Bon](#script-no-next-tick-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Contrôles de script destinés à Vapor ; une activation explicite applique aussi la restriction aux scripts ordinaires  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-next-tick-bad"></span>

**Mauvais**

Le composant destiné à Vapor importe `nextTick` et attend son résultat, introduisant la dépendance à la planification des mises à jour du DOM que cette règle de migration rejette.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

<span id="script-no-next-tick-good"></span>

**Bon**

L’input est obtenu par `useTemplateRef` et reçoit le focus dans `onMounted`. Ce point de montage explicite remplace la dépendance de l’exemple à `nextTick`.

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [Toutes les règles](all.md)

### `script/no-options-api`

Interdire les formes de l’Options API en mode Vapor

[Mauvais](#script-no-options-api-bad) · [Bon](#script-no-options-api-good)

Gravité par défaut: `error`  
Préréglages: `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Contrôles de script destinés à Vapor ; une activation explicite applique aussi la restriction aux scripts ordinaires  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-options-api-bad"></span>

**Mauvais**

L’objet exporté par défaut déclare `data()` de l’Options API, une forme d’option de composant interdite par cette règle.

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

<span id="script-no-options-api-good"></span>

**Bon**

L’état du composant devient une `ref` de la Composition API dans le `<script setup>` Vapor, supprimant l’objet de l’Options API et son option `data`.

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [Toutes les règles](all.md)

### `vapor/no-inline-template`

Interdire l’attribut obsolète inline-template

[Mauvais](#vapor-no-inline-template-bad) · [Bon](#vapor-no-inline-template-good)

Gravité par défaut: `error`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/no-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-inline-template-bad"></span>

**Mauvais**

LegacyCard utilise l’attribut inline-template pour le balisage de son enfant.

```vue annotate="remove:2,3"
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

<span id="vapor-no-inline-template-good"></span>

**Bon**

Le balisage est transmis par le slot par défaut au lieu d’un template en ligne.

```vue annotate="add:2,3,4,5"
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) · [Toutes les règles](all.md)

### `vapor/no-vue-lifecycle-events`

Interdire les événements de cycle de vie @vue:xxx par élément (non pris en charge dans Vapor)

[Mauvais](#vapor-no-vue-lifecycle-events-bad) · [Bon](#vapor-no-vue-lifecycle-events-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/no-vue-lifecycle-events": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-vue-lifecycle-events-bad"></span>

**Mauvais**

Le champ de saisie utilise l’événement de cycle de vie de template @vue:mounted.

```vue annotate="remove:2"
<template>
  <input @vue:mounted="focusInput" />
</template>
```

<span id="vapor-no-vue-lifecycle-events-good"></span>

**Bon**

onMounted accède à la référence de template nommée et donne le focus au champ de saisie au moyen du hook de cycle de vie de script pris en charge.

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) · [Toutes les règles](all.md)

### `vapor/prefer-static-class`

Préférer une classe statique à une liaison de classe dynamique pour les chaînes littérales

[Mauvais](#vapor-prefer-static-class-bad) · [Bon](#vapor-prefer-static-class-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/prefer-static-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-prefer-static-class-bad"></span>

**Mauvais**

La liaison de classe évalue une chaîne constante alors que la classe ne change pas.

```vue annotate="remove:2"
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

<span id="vapor-prefer-static-class-good"></span>

**Bon**

Un attribut class statique exprime les mêmes classes du panneau sans liaison.

```vue annotate="add:2"
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) · [Toutes les règles](all.md)

### `vapor/require-vapor-attribute`

Suggérer l’ajout de l’attribut vapor à script setup

[Mauvais](#vapor-require-vapor-attribute-bad) · [Bon](#vapor-require-vapor-attribute-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Non implémentée pour le lint des SFC  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Prise en charge actuelle: `no-sfc-finding`

Cette règle est un emplacement réservé doté d’un callback vide. Ajouter vapor sélectionne la compilation Vapor ; le linter actuel ne signale pas cet identifiant du catalogue lorsque vapor est absent.

**ID configuré (aucun diagnostic SFC actuellement)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-require-vapor-attribute-bad"></span>

**Mauvais**

Le bloc script setup ne possède pas l’attribut de compilation Vapor. Il s’agit d’une convention prévue : la fonction de rappel actuellement vide de la règle ne le signale pas.

```vue annotate="remove:1"
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

<span id="vapor-require-vapor-attribute-good"></span>

**Bon**

L’ajout de vapor sélectionne la compilation Vapor. Il illustre la correction prévue et ne signifie pas que le linter actuel émet cette règle du catalogue.

```vue annotate="add:1"
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

Le bon exemple illustre la convention visée ; le traitement actuel des SFC n’émet le diagnostic propre à cette règle pour aucun des deux exemples.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) · [Toutes les règles](all.md)
