---
title: "script/no-dupe-keys"
description: "Proibir chaves duplicadas entre props/data/computed/methods/setup/inject da Options API."
---

# `script/no-dupe-keys`

Proibir chaves duplicadas entre props/data/computed/methods/setup/inject da Options API.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

`foo` é declarado tanto em props quanto em data, e `bar` tanto em computed quanto em methods. Essas declarações disputam as mesmas chaves na instância do componente.

```vue annotate="remove:5,8,9,10,11"
<script lang="ts">
export default {
  props: ['foo'],
  data() {
    return { foo: 1 } // duplicate of prop `foo`
  },
  computed: {
    bar() { return 2 }
  },
  methods: {
    bar() {} // duplicate of computed `bar`
  }
}
</script>
```

## Correto

As declarações de prop, data e computed usam nomes distintos (`foo`, `bar` e `baz`), eliminando ambas as colisões entre opções.

```vue annotate="add:5,8"
<script lang="ts">
export default {
  props: ['foo'],
  data() {
    return { bar: 1 }
  },
  computed: {
    baz() { return 2 }
  }
}
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
        "script/no-dupe-keys": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L55)
