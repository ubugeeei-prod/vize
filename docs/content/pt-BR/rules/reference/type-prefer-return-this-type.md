---
title: "type/prefer-return-this-type"
description: "Exigir `this` quando o método devolve apenas o tipo `this`."
---

# `type/prefer-return-this-type`

Exigir `this` quando o método devolve apenas o tipo `this`.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

setValue devolve this, mas o tipo de retorno declarado é o nome da classe Builder.

```vue annotate="remove:3"
<script setup lang="ts">
class Builder {
  setValue(): Builder {
    return this;
  }
}
</script>
```

## Correto

O tipo de retorno this corresponde ao valor que o método devolve.

```vue annotate="add:3"
<script setup lang="ts">
class Builder {
  setValue(): this {
    return this;
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
      "type/prefer-return-this-type": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_return_this_type.rs#L5)
