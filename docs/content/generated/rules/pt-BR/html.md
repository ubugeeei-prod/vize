---
title: "Regras HTML"
---

# Regras HTML

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.


| Regra | Exemplos | Finalidade |
| --- | --- | --- |
| [`html/deprecated-attr`](#html-deprecated-attr) | [Incorreto](#html-deprecated-attr-bad) · [Correto](#html-deprecated-attr-good) | Proibir atributos HTML obsoletos |
| [`html/deprecated-element`](#html-deprecated-element) | [Incorreto](#html-deprecated-element-bad) · [Correto](#html-deprecated-element-good) | Proibir elementos HTML obsoletos |
| [`html/id-duplication`](#html-id-duplication) | [Incorreto](#html-id-duplication-bad) · [Correto](#html-id-duplication-good) | Proibir IDs de elementos duplicados |
| [`html/no-consecutive-br`](#html-no-consecutive-br) | [Incorreto](#html-no-consecutive-br-bad) · [Correto](#html-no-consecutive-br-good) | Proibir elementos &lt;br&gt; consecutivos |
| [`html/no-dupe-style-properties`](#html-no-dupe-style-properties) | [Incorreto](#html-no-dupe-style-properties-bad) · [Correto](#html-no-dupe-style-properties-good) | Proibir propriedades duplicadas em atributos de estilo em linha |
| [`html/no-duplicate-class`](#html-no-duplicate-class) | [Incorreto](#html-no-duplicate-class-bad) · [Correto](#html-no-duplicate-class-good) | Proibir nomes de classe duplicados em um atributo class estático |
| [`html/no-duplicate-dt`](#html-no-duplicate-dt) | [Incorreto](#html-no-duplicate-dt-bad) · [Correto](#html-no-duplicate-dt-good) | Proibir nomes &lt;dt&gt; duplicados em &lt;dl&gt; |
| [`html/no-empty-palpable-content`](#html-no-empty-palpable-content) | [Incorreto](#html-no-empty-palpable-content-bad) · [Correto](#html-no-empty-palpable-content-good) | Proibir elementos vazios que esperam conteúdo visível |
| [`html/require-datetime`](#html-require-datetime) | [Incorreto](#html-require-datetime-bad) · [Correto](#html-require-datetime-good) | Exigir o atributo datetime no elemento &lt;time&gt; |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `html/deprecated-attr`

Proibir atributos HTML obsoletos

[Incorreto](#html-deprecated-attr-bad) · [Correto](#html-deprecated-attr-good)

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
        "html/deprecated-attr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-attr-bad"></span>

**Incorreto**

O parágrafo usa o atributo de apresentação obsoleto `align`.

```vue annotate="remove:1,2,3"
<template>
<p align="center">Notice</p>
</template>
```

<span id="html-deprecated-attr-good"></span>

**Correto**

A classe e a declaração `text-align: center` expressam o alinhamento por meio de CSS.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) · [Todas as regras](all.md)

### `html/deprecated-element`

Proibir elementos HTML obsoletos

[Incorreto](#html-deprecated-element-bad) · [Correto](#html-deprecated-element-good)

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
        "html/deprecated-element": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-element-bad"></span>

**Incorreto**

O elemento `center` usa um elemento de apresentação HTML obsoleto.

```vue annotate="remove:2"
<template>
  <center>Profile</center>
</template>
```

<span id="html-deprecated-element-good"></span>

**Correto**

Uma seção e uma classe de estilo substituem o elemento obsoleto, preservando o conteúdo.

```vue annotate="add:2"
<template>
  <section class="profile">Profile</section>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33) · [Todas as regras](all.md)

### `html/id-duplication`

Proibir IDs de elementos duplicados

[Incorreto](#html-id-duplication-bad) · [Correto](#html-id-duplication-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "html/id-duplication": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-id-duplication-bad"></span>

**Incorreto**

Tanto o campo quanto o parágrafo de ajuda declaram `id="email"`, tornando ambíguo o destino do rótulo.

```vue annotate="remove:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

<span id="html-id-duplication-good"></span>

**Correto**

O campo mantém `email`; o parágrafo de ajuda usa `email-help`, e aria-describedby faz referência a esse ID distinto.

```vue annotate="add:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) · [Todas as regras](all.md)

### `html/no-consecutive-br`

Proibir elementos &lt;br&gt; consecutivos

[Incorreto](#html-no-consecutive-br-bad) · [Correto](#html-no-consecutive-br-good)

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
        "html/no-consecutive-br": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-consecutive-br-bad"></span>

**Incorreto**

Dois elementos de quebra consecutivos criam espaçamento entre blocos dentro de um único parágrafo.

```vue annotate="remove:2"
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

<span id="html-no-consecutive-br-good"></span>

**Correto**

Parágrafos separados expressam os dois blocos de conteúdo sem repetir elementos de quebra.

```vue annotate="add:2,3"
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) · [Todas as regras](all.md)

### `html/no-dupe-style-properties`

Proibir propriedades duplicadas em atributos de estilo em linha

[Incorreto](#html-no-dupe-style-properties-bad) · [Correto](#html-no-dupe-style-properties-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
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
        "html/no-dupe-style-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-dupe-style-properties-bad"></span>

**Incorreto**

Cada estilo estático repete uma propriedade; `margin` e `MARGIN` também contam como a mesma propriedade.

```vue annotate="remove:2,3"
<template>
<div style="color: red; color: blue">text</div>
<div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

<span id="html-no-dupe-style-properties-good"></span>

**Correto**

O estilo estático usa propriedades distintas de cor e plano de fundo. Vinculações dinâmicas de estilo ficam fora desta verificação de atributos estáticos.

```vue annotate="add:2,3"
<template>
<div style="color: red; background: blue">text</div>
<div :style="{ color: a, color: b }">text</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) · [Todas as regras](all.md)

### `html/no-duplicate-class`

Proibir nomes de classe duplicados em um atributo class estático

[Incorreto](#html-no-duplicate-class-bad) · [Correto](#html-no-duplicate-class-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
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
        "html/no-duplicate-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-class-bad"></span>

**Incorreto**

A lista de classes estática repete o token `btn`.

```vue annotate="remove:2"
<template>
<div class="btn btn primary">click</div>
</template>
```

<span id="html-no-duplicate-class-good"></span>

**Correto**

A lista de classes mantém um token `btn` e o token distinto `primary`.

```vue annotate="add:2"
<template>
<div class="btn primary">click</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) · [Todas as regras](all.md)

### `html/no-duplicate-dt`

Proibir nomes &lt;dt&gt; duplicados em &lt;dl&gt;

[Incorreto](#html-no-duplicate-dt-bad) · [Correto](#html-no-duplicate-dt-good)

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
        "html/no-duplicate-dt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-dt-bad"></span>

**Incorreto**

A mesma lista de definições repete o termo `API` para duas descrições.

```vue annotate="remove:5"
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

<span id="html-no-duplicate-dt-good"></span>

**Correto**

Um único termo API é seguido pelas duas descrições, evitando a repetição do termo.

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_duplicate_dt.rs#L41) · [Todas as regras](all.md)

### `html/no-empty-palpable-content`

Proibir elementos vazios que esperam conteúdo visível

[Incorreto](#html-no-empty-palpable-content-bad) · [Correto](#html-no-empty-palpable-content-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "html/no-empty-palpable-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-empty-palpable-content-bad"></span>

**Incorreto**

O parágrafo, o item de lista e a célula da tabela têm conteúdo perceptível vazio.

```vue annotate="remove:2,3,4"
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

<span id="html-no-empty-palpable-content-good"></span>

**Correto**

O texto preenche o parágrafo, a interpolação fornece o conteúdo do item de lista e aria-label dá um nome explícito à célula que, de outra forma, estaria vazia.

```vue annotate="add:2,3,4"
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) · [Todas as regras](all.md)

### `html/require-datetime`

Exigir o atributo datetime no elemento &lt;time&gt;

[Incorreto](#html-require-datetime-bad) · [Correto](#html-require-datetime-good)

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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-require-datetime-bad"></span>

**Incorreto**

O elemento time contém uma data legível por pessoas, mas não tem um valor datetime legível por máquinas.

```vue annotate="remove:2"
<template>
  <time>May 13, 2026</time>
</template>
```

<span id="html-require-datetime-good"></span>

**Correto**

`datetime="2026-05-13"` fornece a data correspondente em um formato legível por máquinas.

```vue annotate="add:2"
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) · [Todas as regras](all.md)
