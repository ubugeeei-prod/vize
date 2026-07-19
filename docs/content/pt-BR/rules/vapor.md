---
title: Regras de Vapor
---

<!-- Generated translation; source: rules/vapor.md -->

# Regras de Vapor

Essas regras cobrem restrições de templates para componentes e aplicativos orientados ao Vapor. API de composição e
orientação Vapor em nível de script vivem em [Type and script rules](./type-and-script.md).

## `vapor/no-vue-lifecycle-events`

Relata eventos do ciclo de vida por elemento, como `@vue:mounted`.

Gravidade padrão: `error`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <input @vue:mounted="focusInput" />
</template>
```

Bom:

```vue
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

## `vapor/require-vapor-attribute`

Sugere adicionar `vapor` ao `<script setup>` quando o preset espera componentes compatíveis com Vapor.

Gravidade padrão: `warning`
Presets: `nuxt`, `opinionated`

Ruim:

```vue
<script setup lang="ts">
const count = ref(0);
</script>
```

Bom:

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## `vapor/no-inline-template`

Relata o atributo `inline-template` obsoleto.

Gravidade padrão: `error`
Presets: `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

Bom:

```vue
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

## `vapor/prefer-static-class`

Relata ligações dinâmicas `:class` cujo valor é um literal estático de string.

Gravidade padrão: `warning`
Presets: `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

Bom:

```vue
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```
