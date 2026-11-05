---
title: "vue/no-this-in-before-route-enter"
description: "Proíbe acessar com `this` uma instância ainda não criada em `beforeRouteEnter`."
---

# `vue/no-this-in-before-route-enter`

Proíbe acessar com `this` uma instância ainda não criada em `beforeRouteEnter`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`beforeRouteEnter` executa antes de a instância do componente estar disponível, então `this.load()` não pode acessá-la.

```vue annotate="remove:4,5"
<script lang="ts">
export default {
  beforeRouteEnter(to, from, next) {
    this.load();
    next();
  },
  methods: { load() {} },
};
</script>
```

## Correto

Use a instância recebida pelo callback de `next` e chame `vm.load()` depois que o componente for criado.

```vue annotate="add:4"
<script lang="ts">
export default {
  beforeRouteEnter(to, from, next) {
    next((vm) => vm.load());
  },
  methods: { load() {} },
};
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-this-in-before-route-enter": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_this_in_before_route_enter.rs#L24)
