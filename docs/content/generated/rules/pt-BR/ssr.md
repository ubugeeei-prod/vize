---
title: "Regras da SSR"
---

# Regras da SSR

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.


| Regra | Exemplos | Finalidade |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](#ssr-no-browser-globals-in-ssr) | [Incorreto](#ssr-no-browser-globals-in-ssr-bad) · [Correto](#ssr-no-browser-globals-in-ssr-good) | Proibir variáveis globais exclusivas do navegador no contexto de SSR |
| [`ssr/no-hydration-mismatch`](#ssr-no-hydration-mismatch) | [Incorreto](#ssr-no-hydration-mismatch-bad) · [Correto](#ssr-no-hydration-mismatch-good) | Proibir valores não determinísticos que causam divergências na hidratação |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `ssr/no-browser-globals-in-ssr`

Proibir variáveis globais exclusivas do navegador no contexto de SSR

[Incorreto](#ssr-no-browser-globals-in-ssr-bad) · [Correto](#ssr-no-browser-globals-in-ssr-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ssr/no-browser-globals-in-ssr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

**Incorreto**

Setup lê `window.innerWidth` imediatamente, embora `window` não exista quando o componente é executado no servidor.

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

<span id="ssr-no-browser-globals-in-ssr-good"></span>

**Correto**

A largura inicial é um valor de ref seguro no servidor, e o acesso ao navegador é movido para `onMounted`, que é executado no cliente em vez de durante o setup de SSR.

```vue annotate="add:2,3,4,5,6"
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L158) · [Todas as regras](all.md)

### `ssr/no-hydration-mismatch`

Proibir valores não determinísticos que causam divergências na hidratação

[Incorreto](#ssr-no-hydration-mismatch-bad) · [Correto](#ssr-no-hydration-mismatch-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-hydration-mismatch-bad"></span>

**Incorreto**

O template avalia `Math.random()` durante a renderização, então o servidor e o cliente podem produzir textos diferentes para o mesmo parágrafo.

```vue annotate="remove:2"
<template>
  <p>{{ Math.random() }}</p>
</template>
```

<span id="ssr-no-hydration-mismatch-good"></span>

**Correto**

O parágrafo renderiza o estado estável `seed` em vez de um novo resultado aleatório. Neste exemplo no estilo do Nuxt, `useState` fornece o estado compartilhado e o inicializador é a constante `"stable"`.

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) · [Todas as regras](all.md)
