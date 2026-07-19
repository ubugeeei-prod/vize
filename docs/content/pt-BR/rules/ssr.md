---
title: Regras da SSR
---

<!-- Generated translation; source: rules/ssr.md -->

# Regras da SSR

Essas regras cobrem padrões de código e modelos que podem quebrar a renderização do servidor ou a hidratação. Eles
são documentados separadamente das regras HTML e Vapor porque o modo de falha é o limite
servidor/cliente.

## `ssr/no-browser-globals-in-ssr`

Relatórios globais apenas do navegador em código que pode rodar durante SSR.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

Bom:

```vue
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

Verificações de guarda, como `typeof window === "undefined"`, são permitidas porque o formulário de identificador de `typeof`
direto é seguro durante a renderização do servidor. Strings, comentários e literais regex também
são ignorados quando contêm nomes como `window` ou `document`. Acessar um membro como
`typeof window.innerWidth` ainda reporta, porque avalia o navegador globalmente.

## `ssr/no-hydration-mismatch`

Reporta valores de template não determinísticos que podem variar entre renderização do servidor e
hidratação do cliente.

Gravidade padrão: `warning`
Presets: `happy-path`, `nuxt`, `opinionated`

Ruim:

```vue
<template>
  <p>{{ Math.random() }}</p>
</template>
```

Bom:

```vue
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```
