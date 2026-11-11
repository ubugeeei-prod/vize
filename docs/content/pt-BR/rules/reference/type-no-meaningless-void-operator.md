---
title: "type/no-meaningless-void-operator"
description: "Proibir o operador void em expressões que já são void."
---

# `type/no-meaningless-void-operator`

Proibir o operador void em expressões que já são void.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Não | Nenhuma |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

logVoid() já é void, então o operador void não muda o resultado.

```vue annotate="remove:2,3"
<script setup lang="ts">
function logVoid(): void {}
void logVoid();
</script>
```

## Correto

getNumber() devolve uma Promise, então void descarta esse thenable de propósito.

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
void getNumber();
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-meaningless-void-operator": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_meaningless_void_operator.rs#L5)
