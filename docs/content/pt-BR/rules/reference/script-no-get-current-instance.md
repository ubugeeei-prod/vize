---
title: "script/no-get-current-instance"
description: "Proibir getCurrentInstance() no modo Vapor (retorna null)."
---

# `script/no-get-current-instance`

Proibir getCurrentInstance() no modo Vapor (retorna null).

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `error` | `opinionated` | Não | Nenhuma |

Aplicável a: Verificações de script voltadas ao Vapor; a ativação explícita também aplica a restrição a scripts comuns.

## Incorreto

O setup marcado como Vapor importa e chama `getCurrentInstance`, dependendo de uma API de instância que esta regra proíbe para componentes orientados a Vapor.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

## Correto

`inject("app-config")` obtém a configuração fornecida explicitamente sem importar nem chamar `getCurrentInstance`.

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
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
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38)
