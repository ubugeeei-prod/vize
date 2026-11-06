---
title: "script/no-deprecated-events-api"
description: "Proibir a API de eventos do Vue 2 removida ($on / $off / $once)."
---

# `script/no-deprecated-events-api`

Proibir a API de eventos do Vue 2 removida ($on / $off / $once).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

As chamadas de `$on`, `$once` e `$off` usam os métodos de barramento de eventos da instância removidos no Vue 3.

```vue annotate="remove:4,5,6,7"
<script>
export default {
  mounted() {
    this.$on('event', handler)
    const vm = this
    vm.$once('event', handler)
    this.$off('event', handler)
  }
}
</script>
```

## Correto

`$emit` continua válido, enquanto a assinatura no barramento de eventos passa para o método `on` do emissor externo. A correção separa a emissão direcionada ao pai de um barramento de eventos externo.

```vue annotate="add:2,3,6,7"
<script>
import mitt from 'mitt'
const emitter = mitt()
export default {
  mounted() {
    this.$emit('event', payload)
    emitter.on('event', handler)
  }
}
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-events-api": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L51)
