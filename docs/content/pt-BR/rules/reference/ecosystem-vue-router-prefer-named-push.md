---
title: "ecosystem/vue-router-prefer-named-push"
description: "Preferir objetos de rotas nomeadas para navegação programática com Vue Router."
---

# `ecosystem/vue-router-prefer-named-push`

Preferir objetos de rotas nomeadas para navegação programática com Vue Router.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

## Incorreto

router.push recebe uma string de caminho vinculada à grafia atual da URL.

```vue annotate="remove:2"
<script setup lang="ts">
router.push("/settings");
</script>
```

## Correto

router.push recebe um objeto de rota com o nome estável settings.

```vue annotate="add:2"
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

## Configuração (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/vue-router-prefer-named-push": "warn"
    }
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras do ecossistema e Nuxt](../ecosystem.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19)
