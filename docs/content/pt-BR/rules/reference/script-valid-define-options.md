---
title: "script/valid-define-options"
description: "Exigir uso válido de defineOptions() (um único argumento de objeto, sem props/emits/expose/slots)."
---

# `script/valid-define-options`

Exigir uso válido de defineOptions() (um único argumento de objeto, sem props/emits/expose/slots).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

A primeira chamada coloca a declaração específica de `props` dentro de `defineOptions`; as chamadas posteriores também repetem a macro e incluem um argumento que não é objeto. Elas ilustram as restrições de formato proibido e de chamadas repetidas.

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
defineOptions({ props: ['foo'] })   // use defineProps instead
defineOptions({ name: 'Foo' })
defineOptions({ name: 'Bar' })      // duplicate call
defineOptions('Foo')                // not an object literal
</script>
```

## Correto

Uma única chamada de `defineOptions` recebe um objeto contendo apenas as opções comuns suportadas `name` e `inheritAttrs`.

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: 'Foo', inheritAttrs: false })
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
        "script/valid-define-options": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41)
