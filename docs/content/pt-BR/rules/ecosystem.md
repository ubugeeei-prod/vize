---
title: Regras do Ecossistema
---

<!-- Generated translation; source: rules/ecosystem.md -->

# Regras do Ecossistema

Essas regras abrangem convenções em torno de Nuxt, Vue Router, Pinia, vue-i18n, Vue Test Utils e Void Vue.

As regras do ecossistema são habilitadas pelo predefinido `ecosystem`. Os hosts também podem habilitá-los pelo nome ao usar
`incremental`; Eles não fazem parte de `happy-path`, `nuxt`ou `opinionated`.

Quando ajudantes do ecossistema do editor são ativados no LSP, o Vize também adiciona o nome de rota do Vue Router
completação, completação e diagnóstico de params de rota de arquivos para `useRoute().params`, conclusão de
chave Vue I18n, validação de chave JSON no workspace e prévias de inlay para chamadas estáticas de `t()` / `$t()` .

## `ecosystem/router-link-require-to`

Requer `to` ou `:to` em `<RouterLink>`, `<router-link>`, `<NuxtLink>`e `<nuxt-link>`.

Gravidade padrão: `error`
Presets: `ecosystem`

Ruim:

```vue
<template>
  <RouterLink>Settings</RouterLink>
</template>
```

Bom:

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

## `ecosystem/vue-router-prefer-named-link`

Avisos sobre cadeias internas estáticas de caminho em componentes semelhantes ao RouterLink. Objetos de rotas nomeadas mantêm rotas digitadas no Vue
Roteadores e completações do editor centradas em nomes de rotas e parámetros.

Gravidade padrão: `warning`
Presets: `ecosystem`

Ruim:

```vue
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

Bom:

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

## `ecosystem/vue-router-prefer-named-push`

Avisos em `router.push("/path")`, `router.replace("/path")`, e roteiam objetos com `path`estática.

Gravidade padrão: `warning`
Presets: `ecosystem`

Ruim:

```ts
router.push("/settings");
```

Bom:

```ts
router.push({ name: "settings" });
```

## `ecosystem/nuxt-prefer-nuxt-link`

Avisos sobre links internos de `<a href="/...">` em código orientado a Nuxt. Links externos, downloads e
`target="_blank"` continuam sendo âncoras simples.

Gravidade padrão: `warning`
Presets: `ecosystem`

Ruim:

```vue
<template>
  <a href="/settings">Settings</a>
</template>
```

Bom:

```vue
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

## `ecosystem/pinia-prefer-store-to-refs`

Avisa quando uma loja Pinia é desestruturada diretamente. Use `storeToRefs()` para estado e getters, e
manter as ações na instância da loja.

Gravidade padrão: `warning`
Presets: `ecosystem`

Ruim:

```ts
const { name } = useUserStore();
```

Bom:

```ts
const store = useUserStore();
const { name } = storeToRefs(store);
```

## `ecosystem/vue-i18n-no-missing-key`

Avisa quando uma chave estática `$t()` `$te()`, `$tm()`, `t()`, `te()`ou `tm()` está ausente no
mesmo bloco de `<i18n lang="json">` local do SFC.

Gravidade padrão: `warning`
Presets: `ecosystem`

Ruim:

```vue
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

Bom:

```vue
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

## `ecosystem/void-link-require-href`

Requer `href` ou `:href` no Void Vue `<Link>` componentes importados de `@void/vue`.

Gravidade padrão: `error`
Presets: `ecosystem`

Ruim:

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

Bom:

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

## `ecosystem/void-link-valid-method`

Avisa sobre valores estáticos desconhecidos de `<Link method>` do Void Vue e sobre props apenas GET, como `prefetch`
ou `reloadDocument` quando o link usa um método de mutação.

Gravidade padrão: `warning`
Presets: `ecosystem`

Ruim:

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

Bom:

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

## `ecosystem/vue-test-utils-no-html-snapshot`

Avisa sobre `expect(wrapper.html()).toMatchSnapshot()`. Prefira afirmações focadas em texto visível, atributos
, eventos emitidos ou estado dos componentes.

Gravidade padrão: `warning`
Presets: `ecosystem`

Ruim:

```ts
expect(wrapper.html()).toMatchSnapshot();
```

Bom:

```ts
expect(wrapper.text()).toContain("Saved");
```
