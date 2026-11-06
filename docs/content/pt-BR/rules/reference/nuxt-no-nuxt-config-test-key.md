---
title: "nuxt/no-nuxt-config-test-key"
description: "Proibir a definição da chave `test` na configuração do Nuxt."
---

# `nuxt/no-nuxt-config-test-key`

Proibir a definição da chave `test` na configuração do Nuxt.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `nuxt` | Não | Nenhuma |

Aplicável a: Arquivos de configuração do Nuxt (nuxt.config.ts).

## Incorreto

A configuração exportada do Nuxt define a chave identificadora `test` como o booleano `true`, a estrutura de configuração obsoleta que esta regra rejeita.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ test: true });
```

## Correto

A configuração vazia remove essa propriedade booleana `test`. Este exemplo não proíbe um objeto de configuração de testes.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({});
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "nuxt/no-nuxt-config-test-key": "error"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras do ecossistema e Nuxt](../ecosystem.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16)
