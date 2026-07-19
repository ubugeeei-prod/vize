---
title: Règles de type et de script
---

<!-- Generated translation; source: rules/type-and-script.md -->

# Règles de type et de script

Les règles de type utilisent le vérificateur TypeScript lorsque des informations sémantiques sont nécessaires. Vize lit la même forme de projet
que TypeScript lit depuis `tsconfig.json`, donc les noms d’ambiance partagés doivent provenir de
`compilerOptions.types`, de références de projet ou de fichiers de déclaration.

Les règles de script sont des règles Patina pour l’API de composition et le code orienté Vapor. Ils se concentrent sur des motifs
difficiles à compiler efficacement ou difficiles à raisonner en mode Vapor.

Le linting conscient du type est volontaire. Activez-le avec `linter.typeAware: true`, `vize lint --type-aware`ou
en activant explicitement une règle `type/*`. `type/no-reactivity-loss` peut être activé directement avec
`vize lint --strict-reactivity`. Si Corsa ne peut pas être lancé, Patina signale `type/corsa-runtime` et
saute la passe de règles à damier au lieu de supprimer silencieusement les règles configurées.

`--type-aware` utilise la même résolution exécutable Corsa que `vize check`; Configurez
`typeChecker.corsaPath` lorsque le projet a besoin d’un binaire explicite `tsgo` ou Corsa. Les valeurs par défaut restent
coût zéro : Patina ne sélectionne pas les SFC pour le linting à damier ni ne commence Corsa à moins que le drapeau,
`linter.typeAware`, ou une règle de `type/*` explicitement activée ne l’accepte.

```ts
export default defineConfig({
  linter: { typeAware: true },
});
```

## `type/require-typed-props`

Il faut `defineProps` typer au lieu d’utiliser une déclaration de tableau à l’exécution.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<script setup lang="ts">
const props = defineProps(["label", "count"]);
</script>
```

Bon :

```vue
<script setup lang="ts">
const props = defineProps<{
  label: string;
  count: number;
}>();
</script>
```

## `type/require-typed-emits`

Il nécessite `defineEmits` de décrire les charges utiles d’événements émises.

Sévérité par défaut : `warning`
Presets : `happy-path`, `nuxt`, `opinionated`

Mauvais :

```vue
<script setup lang="ts">
const emit = defineEmits(["save"]);

emit("save", form.value);
</script>
```

Bon :

```vue
<script setup lang="ts">
const emit = defineEmits<{
  save: [payload: FormValue];
}>();

emit("save", form.value);
</script>
```

## `type/no-unsafe-template-binding`

Des liaisons de modèles de rapports qui résolvent vers des valeurs non sûres telles que `any`. La règle est basée sur des damiers,
elle suit donc les types importés et la configuration du projet.

Sévérité par défaut : `warning`
Préréglages : `nuxt`, `opinionated`

Mauvais :

```vue
<script setup lang="ts">
const payload: any = await loadPayload();
</script>

<template>
  <p>{{ payload.title }}</p>
</template>
```

Bon :

```vue
<script setup lang="ts">
type Payload = { title: string };

const payload = await loadPayload<Payload>();
</script>

<template>
  <p>{{ payload.title }}</p>
</template>
```

## `type/no-floating-promises`

Signale des promesses créées mais non attendues, retournées ou gérées intentionnellement.
La vérification couvre à la fois les expressions `<script>` et les expressions modèles.

Sévérité par défaut : `warning`
Préréglages : `nuxt`, `opinionated`

Mauvais :

```vue
<script setup lang="ts">
function submit() {
  saveForm(form.value);
}
</script>

<template>
  <button @click="saveForm(form)">Save</button>
  <p>{{ loadPreview() }}</p>
</template>
```

Bon :

```vue
<script setup lang="ts">
type Preview = { title: string };

async function submit() {
  await saveForm(form.value);
}

const preview = ref<Preview | null>(null);

async function loadPreviewIntoState() {
  preview.value = await loadPreview();
}
</script>

<template>
  <button @click="void submit()">Save</button>
  <button @click="void loadPreviewIntoState()">Preview</button>
  <PreviewPanel v-if="preview" :preview="preview" />
</template>
```

## `type/no-reactivity-loss`

Rapporte des instantanés simples des valeurs réactives utilisées entre les flux. La règle s’exécute aussi lorsque
`vize lint --strict-reactivity` est activé.

Sévérité par défaut : `warning`
Préréglages : `nuxt`, `opinionated`

Mauvais :

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const item = props.item;
</script>
```

Bon :

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

## Checker Configuration

Les règles sensibles au type n’ont pas besoin d’un champ Vize `globals` séparé pour les noms TypeScript. Privilégiez
configuration native TypeScript :

Mauvais :

```ts
export default {
  globals: ["definePageMeta", "process"],
};
```

Bon :

```json
{
  "compilerOptions": {
    "types": ["node", "nuxt/app"]
  }
}
```

## `script/no-options-api`

Rapports Options Définitions de composants API dans des préréglages orientés Vapor.

Sévérité par défaut : `error`
Préréglages : `nuxt`, `opinionated`

Mauvais :

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

Bon :

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## `script/no-next-tick`

Les rapports `nextTick()` dans des composants orientés Vapor. Privilégiez les références directes, les hooks du cycle de vie ou les flux d’état
qui ne dépendent pas du prochain flush DOM.

Sévérité par défaut : `error`
Préréglages : `nuxt`, `opinionated`

Mauvais :

```vue
<script setup lang="ts" vapor>
await nextTick();
input.value?.focus();
</script>
```

Bon :

```vue
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>
```

## `script/no-get-current-instance`

Les rapports `getCurrentInstance()` dans des composants orientés Vapor. Elle atteint des composants internes d’exécution que
Vapor ne peut pas optimiser en toute sécurité.

Sévérité par défaut : `error`
Préréglages : `nuxt`, `opinionated`

Mauvais :

```vue
<script setup lang="ts" vapor>
const instance = getCurrentInstance();
const app = instance?.appContext.app;
</script>
```

Bon :

```vue
<script setup lang="ts" vapor>
const appConfig = useAppConfig();
</script>
```
