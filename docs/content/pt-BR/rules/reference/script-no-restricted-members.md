---
title: "script/no-restricted-members"
description: "Proibir acessos a membros object.property configurados pelo projeto."
---

# `script/no-restricted-members`

Proibir acessos a membros object.property configurados pelo projeto.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | Nenhuma | Não | [Opções das regras](/rules/options.md) |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

Este exemplo configura window.localStorage. A regra não tem uma lista padrão de bloqueio; ativá-la sozinha não reporta um membro.

## Incorreto

Com `{ object: "window", property: "localStorage" }` configurado em `ruleOptions`, `window.localStorage` acessa o par objeto/membro proibido. Esta regra não tem membros proibidos por padrão.

```vue annotate="remove:2"
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

## Correto

`authStorage.read("token")` delega a leitura à função auxiliar de armazenamento da aplicação e deixa de acessar o membro configurado `window.localStorage`.

```vue annotate="add:2"
<script setup lang="ts">
const token = authStorage.read("token");
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
        "script/no-restricted-members": "error"
      },
      "ruleOptions": {
        "script/no-restricted-members": {
          "members": [
            {
              "object": "window",
              "property": "localStorage"
            }
          ]
        }
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Opções das regras](/rules/options.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53)
