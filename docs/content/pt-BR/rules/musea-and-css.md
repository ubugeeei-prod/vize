---
title: Regras de Musea e CSS
---

<!-- Generated translation; source: rules/musea-and-css.md -->

# Regras de Musea e CSS

As regras de musea validam `<art>` e `<variant>` bloqueios. As regras CSS inspecionam o conteúdo do estilo e recomendam
padrões que mantenham os estilos de componentes tematicáveis, previsíveis e compatíveis com Vue e Vapor.

## `musea/require-title`

Exige que todo arquivo de arte forneça um título de exibição. O título pode vir de `<art title="...">`,
`defineArt("./Button.vue", { title: "..." })`, ou do recurso de `defineArt` fonte de componentes.

Gravidade padrão: `error`

Ruim:

```vue
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

Bom:

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

## `musea/require-component`

Exige que todo arquivo de arte nomeie o componente que ele documenta. Prefiro `defineArt("./Button.vue", ...)`;
`<art component="...">` continua suportado para compatibilidade.

Gravidade padrão: `warning`

Ruim:

```vue
<art title="Button">
  <variant name="primary" />
</art>
```

Bom:

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

## `musea/valid-variant`

Exige que `<variant>` blocos tenham um `name`válido.

Gravidade padrão: `error`

Ruim:

```vue
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

Bom:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## `musea/unique-variant-names`

Requer que nomes variantes sejam únicos dentro de um bloco de arte.

Gravidade padrão: `error`

Ruim:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

Bom:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

## `musea/no-empty-variant`

Reporta variantes vazias que não documentam props, slots ou estado visual.

Gravidade padrão: `warning`

Ruim:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

Bom:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

## `musea/prefer-design-tokens`

Prefiere variáveis CSS de token de design em vez de valores primitivos codificados fixamente em exemplos de Musea.

Gravidade padrão: `warning`

Ruim:

```vue
<art title="Button" component="./Button.vue">
  <variant name="danger">
    <Button style="color: #d00">Delete</Button>
  </variant>
</art>
```

Bom:

```vue
<art title="Button" component="./Button.vue">
  <variant name="danger">
    <Button class="danger">Delete</Button>
  </variant>
</art>

<style scoped>
.danger {
  color: var(--color-danger-text);
}
</style>
```

## `css/no-important`

Desencoraja `!important`.

Gravidade padrão: `warning`

Ruim:

```vue
<style scoped>
.button {
  color: red !important;
}
</style>
```

Bom:

```vue
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

## `css/no-hardcoded-values`

Sugere variáveis CSS em vez de valores codificados de cor, espaçamento ou tamanho.

Gravidade padrão: `warning`

Ruim:

```vue
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

Bom:

```vue
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

## `css/no-id-selectors`

Desestimula seletores de ID em estilos de componentes porque são difíceis de sobrescrever e reutilizar.

Gravidade padrão: `warning`

Ruim:

```vue
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

Bom:

```vue
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

## `css/no-display-none`

Sugere usar primitivas de visibilidade do Vue em vez de esconder os branch dos componentes com CSS.

Gravidade padrão: `warning`

Ruim:

```vue
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

Bom:

```vue
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

## `css/no-v-bind-performance`

Alerta sobre o custo de execução do CSS `v-bind()` em estilos quentes.

Gravidade padrão: `warning`

Ruim:

```vue
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

Bom:

```vue
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

## `css/prefer-logical-properties`

Recomenda propriedades lógicas para layouts internacionalizados.

Gravidade padrão: `warning`

Ruim:

```vue
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

Bom:

```vue
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

## `css/prefer-slotted`

Recomenda `::v-slotted()` ao estilizar o conteúdo dos slots.

Gravidade padrão: `warning`

Ruim:

```vue
<style scoped>
.content h2 {
  margin-block: 0;
}
</style>
```

Bom:

```vue
<style scoped>
::v-slotted(h2) {
  margin-block: 0;
}
</style>
```

## `css/require-font-display`

Exige `font-display` em declarações `@font-face` .

Gravidade padrão: `warning`

Ruim:

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

Bom:

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

## Regras Adicionais de CSS

`css/no-utility-classes` alerta contra a implementação de classes utilitárias dentro dos estilos de componentes. Padrão:
`warning`.

`css/prefer-nested-selectors` recomenda o aninhamento CSS para seletores descendentes. Padrão: `warning`.
