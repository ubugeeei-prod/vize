---
title: "type/strict-boolean-expressions"
description: "Exigir expressões booleanas seguras nas condições de script e template."
---

# `type/strict-boolean-expressions`

Exigir expressões booleanas seguras nas condições de script e template.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo.

Ative typeAware e esta regra explicitamente. Por padrão, números que podem ser nulos são proibidos, enquanto números não nulos são permitidos.

Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.

## Incorreto

`if (count)` depende da conversão implícita para booleano de uma variável numérica que pode estar ausente em vez de um teste booleano explícito; também confunde zero com ausência.

```vue annotate="remove:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

## Correto

`count !== undefined && count > 0` testa separadamente a presença e a positividade, produzindo uma condição booleana explícita após restringir o tipo do valor opcional.

```vue annotate="add:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/strict-boolean-expressions": "warn"
    },
    "typeAware": true
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7)
