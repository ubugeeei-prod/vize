---
title: "vue/no-ref-object-reactivity-loss"
description: "Proíbe copiar o valor de um ref no mesmo escopo e perder suas atualizações reativas."
---

# `vue/no-ref-object-reactivity-loss`

Proíbe copiar o valor de um ref no mesmo escopo e perder suas atualizações reativas.

| Severidade padrão | Predefinições | Correção automática | Opções |
| --- | --- | --- | --- |
| `warning` | Nenhuma | Não | Nenhuma |

Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup.

A verificação acompanha funções reconhecidas que criam refs e cópias de valores no mesmo escopo. Leituras dentro de outra função e escritas em `.value` não são verificadas. É uma verificação opcional baseada no código-fonte, sem necessidade de consultas nativas de tipos.

## Incorreto

Ler `count.value` em `snapshot` na raiz do script captura o número atual, sem acompanhar as atualizações posteriores do ref.

```vue annotate="remove:2,4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const snapshot = count.value;
</script>
```

## Correto

Um getter computado lê `count.value` quando sua dependência muda, preservando um resultado reativo em vez de copiar o valor no escopo raiz.

```vue annotate="add:2,4"
<script setup lang="ts">
import { computed, ref } from "vue";
const count = ref(0);
const snapshot = computed(() => count.value);
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
        "vue/no-ref-object-reactivity-loss": "warn"
      }
    },
  },
});
```

Depois, execute `vp run lint`.

## Páginas relacionadas

- [Regras Vue](../vue.md)
- [Todas as regras](../all.md)
- [Código-fonte](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_object_reactivity_loss.rs#L39)
