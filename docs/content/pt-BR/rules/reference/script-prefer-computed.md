---
title: "script/prefer-computed"
description: "Preferir computed() para estado reativo derivado."
---

# `script/prefer-computed`

Preferir computed() para estado reativo derivado.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

O watcher deve apenas derivar o destino. Cópias editáveis e callbacks com outros efeitos colaterais são permitidos.

## Incorreto

O observador apenas copia uma derivação de `count` para um segundo ref, `doubled`, então o estado derivado é mantido por sincronização manual.

```vue annotate="remove:2,4,5"
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
const doubled = ref(0);
watch(count, (value) => { doubled.value = value * 2; });
</script>
```

## Correto

`computed(() => count.value * 2)` expressa a derivação diretamente e remove tanto o ref gravável adicional quanto seu observador de sincronização.

```vue annotate="add:2,4"
<script setup lang="ts">
import { ref, computed } from "vue";
const count = ref(0);
const doubled = computed(() => count.value * 2);
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
        "script/prefer-computed": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras de tipo e script](../type-and-script.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41)
