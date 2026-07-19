---
title: Anotações de Comentários
---

<!-- Generated translation; source: guide/comment-annotations.md -->

# Anotações de Comentários

O Vize fornece anotações baseadas em comentários para controlar o linting, diagnósticos e comportamento de codegen. Existem dois sistemas de anotação dependendo de onde são usados:

- **`<!-- @vize:xxx -->`** — comentários HTML em `<template>` (diretivas Patina linter)
- **`// @vize forget: reason`** — Comentários JS em `<script>` (supressão de análise entre arquivos)

Todas as diretivas `@vize:` template são **removidas da saída da compilação** — elas nunca aparecem no código de produção.

## Diretivas Modelo (`@vize:`)

Usado dentro `<template>` como comentários em HTML. Esses controlam o comportamento da Pátina (o linter embutido).

### `@vize:expected`

Espere um diagnóstico na próxima linha. Se não for feito diagnóstico, isso é uma operação proibida. Semelhante ao `@ts-expect-error`.

```vue
<template>
  <ul>
    <!-- @vize:expected -->
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
```

### `@vize:ignore-start` / `@vize:ignore-end`

Suprima todos os diagnósticos dentro de uma região.

```vue
<template>
  <!-- @vize:ignore-start -->
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
  <!-- @vize:ignore-end -->
</template>
```

### `@vize:level(warn|error|off)`

Anule a gravidade dos diagnósticos na próxima linha.

```vue
<template>
  <!-- @vize:level(warn) -->
  <img src="/photo.png" />

  <!-- @vize:level(off) -->
  <li v-for="item in items">{{ item }}</li>
</template>
```

| Valor   | Efeito                  |
| ------- | ----------------------- |
| `warn`  | Rebaixamento para aviso |
| `error` | Upgrade para erro       |
| `off`   | Suprimir completamente  |

### `@vize:todo`

Emita um aviso de TUDO.

```vue
<template>
  <!-- @vize:todo add loading state -->
  <div>{{ data }}</div>
</template>
```

### `@vize:fixme`

Emita um erro FIXME.

```vue
<template>
  <!-- @vize:fixme broken on mobile -->
  <div class="layout">...</div>
</template>
```

### `@vize:deprecated`

Emita um aviso de depreciação.

```vue
<template>
  <!-- @vize:deprecated use NewComponent instead -->
  <OldComponent />
</template>
```

### `@vize:docs`

Comentário sobre documentação. Sem efeito de fiapos.

```vue
<template>
  <!-- @vize:docs Primary action button for form submission -->
  <button type="submit">Submit</button>
</template>
```

### `@vize:dev-only`

Marque um nó para ser desmontado em builds de produção, mantido em desenvolvimento.

```vue
<template>
  <!-- @vize:dev-only -->
  <div class="debug-panel">{{ internalState }}</div>
</template>
```

### Resumo

| Diretiva                 | Efeito                                   | Gravidade |
| ------------------------ | ---------------------------------------- | --------- |
| `@vize:expected`         | Espere diagnóstico na próxima linha      | —         |
| `@vize:ignore-start/end` | Suprimir todos os diagnósticos na região | —         |
| `@vize:level(...)`       | Anular a severidade da próxima linha     | —         |
| `@vize:todo <msg>`       | Emitir TODO                              | Aviso     |
| `@vize:fixme <msg>`      | Emit FIXME                               | Erro      |
| `@vize:deprecated <msg>` | Emitir aviso de descontinuação           | Aviso     |
| `@vize:docs <text>`      | Documentação (sem efeito de fiapo)       | —         |
| `@vize:dev-only`         | Tira em produção                         | —         |

## Supressão de Script (`@vize forget`)

Usado dentro `<script>` como comentários do JS. Suprime avisos de análise cruzada (Croquis) na linha seguinte.

### Sintaxe

```vue
<script setup>
// @vize forget: <reason>
<suppressed line>
</script>
```

É necessário um **motivo** — você deve explicar por que a supressão é necessária.

### Exemplo

```vue
<script setup>
import { inject } from "vue";

// @vize forget: intentionally destructuring for one-time read
const { count } = inject("state");
</script>
```

Sem a anotação, o Vize alertaria que desestruturar um valor de retorno de `inject()` reativo quebra o rastreamento de reatividade.

### Regras

| Governo                | Descrição                                                           |
| ---------------------- | ------------------------------------------------------------------- |
| Motivo necessário      | `// @vize forget` sem motivo é um erro                              |
| Cólon necessário       | Deve usar `// @vize forget: <reason>` (dois pontos antes do motivo) |
| Apenas a próxima linha | Aplica-se à próxima linha sem comentário, não vazia                 |
| Sem órfãos             | Uma supressão no final de um arquivo sem código após ele é um erro  |

### Supressões Múltiplas

Cada `@vize forget` se aplica independentemente à próxima linha de código:

```vue
<script setup>
import { inject } from "vue";

// @vize forget: one-time read for display name
const { name } = inject("user");

// @vize forget: static config value
const { theme } = inject("config");
</script>
```

### Pulando Comentários

A supressão mira a próxima **linha de código** , pulando comentários e linhas em branco:

```vue
<script setup>
// @vize forget: read-only access
// This comment is skipped
const { count } = inject("state");
</script>
```

### Razões Comuns

| Motivo                       | Quando usar                                |
| ---------------------------- | ------------------------------------------ |
| `intentionally non-reactive` | O valor não precisa ser reativo            |
| `read-only access`           | Apenas lendo, não acompanhando as mudanças |
| `legacy code`                | Problema conhecido, vou refatorar depois   |
| `third-party integration`    | Exigido pela biblioteca externa            |

### Exemplos inválidos

```ts
// @vize forget
const { count } = inject("state");
// ^ Error: requires a reason

// @vize forget because I said so
const { count } = inject("state");
// ^ Error: requires a colon before the reason

// @vize forget:
const { count } = inject("state");
// ^ Error: reason cannot be empty
```
