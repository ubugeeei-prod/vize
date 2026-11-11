---
title: "type/related-getter-setter-pairs"
description: "Exigir que getters e setters emparelhados usem tipos compatíveis."
---

# `type/related-getter-setter-pairs`

Exigir que getters e setters emparelhados usem tipos compatíveis.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

O getter devolve string enquanto o setter aceita number, então o par é incompatível.

```vue annotate="remove:4"
<script setup lang="ts">
interface Box {
  get size(): string;
  set size(value: number);
}
</script>
```

## Correto

O getter e o setter usam ambos string.

```vue annotate="add:4"
<script setup lang="ts">
interface Box {
  get size(): string;
  set size(value: string);
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
      "type/related-getter-setter-pairs": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/related_getter_setter_pairs.rs#L5)
