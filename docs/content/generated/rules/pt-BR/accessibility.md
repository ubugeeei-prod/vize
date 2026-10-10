---
title: "Regras de Acessibilidade"
---

# Regras de Acessibilidade

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.

<span id="regras-adicionais-de-acessibilidade"></span>

| Regra | Exemplos | Finalidade |
| --- | --- | --- |
| [`a11y/alt-text`](#a11y-alt-text) | [Incorreto](#a11y-alt-text-bad) · [Correto](#a11y-alt-text-good) | Exigir texto alternativo para elementos de mídia |
| [`a11y/anchor-has-content`](#a11y-anchor-has-content) | [Incorreto](#a11y-anchor-has-content-bad) · [Correto](#a11y-anchor-has-content-good) | Exigir conteúdo acessível nos elementos de âncora |
| [`a11y/anchor-is-valid`](#a11y-anchor-is-valid) | [Incorreto](#a11y-anchor-is-valid-bad) · [Correto](#a11y-anchor-is-valid-good) | Exigir um href válido nos elementos de âncora |
| [`a11y/aria-props`](#a11y-aria-props) | [Incorreto](#a11y-aria-props-bad) · [Correto](#a11y-aria-props-good) | Proibir atributos ARIA inválidos |
| [`a11y/aria-role`](#a11y-aria-role) | [Incorreto](#a11y-aria-role-bad) · [Correto](#a11y-aria-role-good) | Exigir que elementos com papéis ARIA usem um papel ARIA válido e não abstrato |
| [`a11y/aria-unsupported-elements`](#a11y-aria-unsupported-elements) | [Incorreto](#a11y-aria-unsupported-elements-bad) · [Correto](#a11y-aria-unsupported-elements-good) | Proibir atributos ARIA em elementos que não os aceitam |
| [`a11y/click-events-have-key-events`](#a11y-click-events-have-key-events) | [Incorreto](#a11y-click-events-have-key-events-bad) · [Correto](#a11y-click-events-have-key-events-good) | Exigir manipuladores de eventos de teclado junto aos eventos de clique |
| [`a11y/form-control-has-label`](#a11y-form-control-has-label) | [Incorreto](#a11y-form-control-has-label-bad) · [Correto](#a11y-form-control-has-label-good) | Exigir rótulos associados aos controles de formulário |
| [`a11y/heading-has-content`](#a11y-heading-has-content) | [Incorreto](#a11y-heading-has-content-bad) · [Correto](#a11y-heading-has-content-good) | Exigir conteúdo acessível nos elementos de título |
| [`a11y/heading-levels`](#a11y-heading-levels) | [Incorreto](#a11y-heading-levels-bad) · [Correto](#a11y-heading-levels-good) | Proibir que níveis de título sejam pulados |
| [`a11y/iframe-has-title`](#a11y-iframe-has-title) | [Incorreto](#a11y-iframe-has-title-bad) · [Correto](#a11y-iframe-has-title-good) | Exigir um atributo title nos elementos iframe |
| [`a11y/img-alt`](#a11y-img-alt) | [Incorreto](#a11y-img-alt-bad) · [Correto](#a11y-img-alt-good) | Exigir o atributo alt nas imagens para garantir acessibilidade |
| [`a11y/interactive-supports-focus`](#a11y-interactive-supports-focus) | [Incorreto](#a11y-interactive-supports-focus-bad) · [Correto](#a11y-interactive-supports-focus-good) | Exigir que elementos com papéis interativos possam receber foco |
| [`a11y/label-has-for`](#a11y-label-has-for) | [Incorreto](#a11y-label-has-for-bad) · [Correto](#a11y-label-has-for-good) | Exigir controles de formulário associados aos rótulos |
| [`a11y/landmark-roles`](#a11y-landmark-roles) | [Incorreto](#a11y-landmark-roles-bad) · [Correto](#a11y-landmark-roles-good) | Validar a posição e a unicidade dos papéis de regiões de referência |
| [`a11y/media-has-caption`](#a11y-media-has-caption) | [Incorreto](#a11y-media-has-caption-bad) · [Correto](#a11y-media-has-caption-good) | Exigir legendas nos elementos de mídia |
| [`a11y/mouse-events-have-key-events`](#a11y-mouse-events-have-key-events) | [Incorreto](#a11y-mouse-events-have-key-events-bad) · [Correto](#a11y-mouse-events-have-key-events-good) | Exigir eventos de foco e perda de foco junto aos eventos de mouse |
| [`a11y/no-access-key`](#a11y-no-access-key) | [Incorreto](#a11y-no-access-key-bad) · [Correto](#a11y-no-access-key-good) | Proibir o uso do atributo accesskey |
| [`a11y/no-aria-hidden-on-focusable`](#a11y-no-aria-hidden-on-focusable) | [Incorreto](#a11y-no-aria-hidden-on-focusable-bad) · [Correto](#a11y-no-aria-hidden-on-focusable-good) | Proibir aria-hidden="true" em elementos que podem receber foco |
| [`a11y/no-autofocus`](#a11y-no-autofocus) | [Incorreto](#a11y-no-autofocus-bad) · [Correto](#a11y-no-autofocus-good) | Proibir o uso do atributo autofocus |
| [`a11y/no-distracting-elements`](#a11y-no-distracting-elements) | [Incorreto](#a11y-no-distracting-elements-bad) · [Correto](#a11y-no-distracting-elements-good) | Proibir elementos que causam distração, como &lt;marquee&gt; e &lt;blink&gt; |
| [`a11y/no-i-for-icon`](#a11y-no-i-for-icon) | [Incorreto](#a11y-no-i-for-icon-bad) · [Correto](#a11y-no-i-for-icon-good) | Proibir o uso do elemento &lt;i&gt; para ícones |
| [`a11y/no-redundant-roles`](#a11y-no-redundant-roles) | [Incorreto](#a11y-no-redundant-roles-bad) · [Correto](#a11y-no-redundant-roles-good) | Proibir papéis ARIA redundantes |
| [`a11y/no-refer-to-non-existent-id`](#a11y-no-refer-to-non-existent-id) | [Incorreto](#a11y-no-refer-to-non-existent-id-bad) · [Correto](#a11y-no-refer-to-non-existent-id-good) | Proibir referências a IDs inexistentes |
| [`a11y/no-role-presentation-on-focusable`](#a11y-no-role-presentation-on-focusable) | [Incorreto](#a11y-no-role-presentation-on-focusable-bad) · [Correto](#a11y-no-role-presentation-on-focusable-good) | Proibir role="presentation" ou role="none" em elementos que podem receber foco |
| [`a11y/no-static-element-interactions`](#a11y-no-static-element-interactions) | [Incorreto](#a11y-no-static-element-interactions-bad) · [Correto](#a11y-no-static-element-interactions-good) | Proibir manipuladores de eventos em elementos estáticos |
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [Incorreto](#a11y-placeholder-label-option-bad) · [Correto](#a11y-placeholder-label-option-good) | Exigir disabled ou hidden na opção de orientação de um select |
| [`a11y/role-has-required-aria-props`](#a11y-role-has-required-aria-props) | [Incorreto](#a11y-role-has-required-aria-props-bad) · [Correto](#a11y-role-has-required-aria-props-good) | Exigir as propriedades obrigatórias dos papéis ARIA |
| [`a11y/tabindex-no-positive`](#a11y-tabindex-no-positive) | [Incorreto](#a11y-tabindex-no-positive-bad) · [Correto](#a11y-tabindex-no-positive-good) | Proibir valores positivos de tabindex |
| [`a11y/use-list`](#a11y-use-list) | [Incorreto](#a11y-use-list-bad) · [Correto](#a11y-use-list-good) | Sugerir elementos de lista para textos que parecem itens de lista |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [Incorreto](#vue-use-unique-element-ids-bad) · [Correto](#vue-use-unique-element-ids-good) | Exigir IDs de elementos únicos por meio de useId(), em vez de literais estáticos |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `a11y/alt-text`

Exigir texto alternativo para elementos de mídia

[Incorreto](#a11y-alt-text-bad) · [Correto](#a11y-alt-text-good)

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
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-alt-text-bad"></span>

**Incorreto**

O controle de envio com imagem fornece apenas a URL da imagem; ele não tem texto `alt` que descreva a ação.

```vue annotate="remove:2"
<template>
  <input type="image" src="/submit.png" />
</template>
```

<span id="a11y-alt-text-good"></span>

**Correto**

`alt="Submit search"` dá ao controle com imagem um nome acessível que descreve o envio da pesquisa.

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) · [Todas as regras](all.md)

### `a11y/anchor-has-content`

Exigir conteúdo acessível nos elementos de âncora

[Incorreto](#a11y-anchor-has-content-bad) · [Correto](#a11y-anchor-has-content-good)

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
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-has-content-bad"></span>

**Incorreto**

O link `/settings` não tem texto nem outro conteúdo que lhe dê um nome, então seu destino não tem uma descrição acessível.

```vue annotate="remove:2"
<template>
  <a href="/settings"></a>
</template>
```

<span id="a11y-anchor-has-content-good"></span>

**Correto**

O texto visível `Settings` fornece conteúdo para o link com o mesmo destino.

```vue annotate="add:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) · [Todas as regras](all.md)

### `a11y/anchor-is-valid`

Exigir um href válido nos elementos de âncora

[Incorreto](#a11y-anchor-is-valid-bad) · [Correto](#a11y-anchor-is-valid-good)

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
        "a11y/anchor-is-valid": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-is-valid-bad"></span>

**Incorreto**

A primeira âncora usa `#` para uma ação; a segunda usa uma URL JavaScript. Nenhuma fornece um destino de navegação comum.

```vue annotate="remove:2,3"
<template>
  <a href="#" @click="openPanel">Open panel</a>
  <a href="JaVaScRiPt:void(0)">Run action</a>
</template>
```

<span id="a11y-anchor-is-valid-good"></span>

**Correto**

Um botão nativo executa `openPanel`, enquanto a âncora restante tem o destino real `/docs/javascript-urls`.

```vue annotate="add:2,3"
<template>
  <button type="button" @click="openPanel">Open panel</button>
  <a href="/docs/javascript-urls">JavaScript URL guide</a>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) · [Todas as regras](all.md)

### `a11y/aria-props`

Proibir atributos ARIA inválidos

[Incorreto](#a11y-aria-props-bad) · [Correto](#a11y-aria-props-good)

Severidade padrão: `error`  
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
        "a11y/aria-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-props-bad"></span>

**Incorreto**

`aria-lable` está escrito incorretamente e não é um atributo ARIA aceito.

```vue annotate="remove:2"
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

<span id="a11y-aria-props-good"></span>

**Correto**

O atributo aceito `aria-label` fornece o nome do botão.

```vue annotate="add:2"
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) · [Todas as regras](all.md)

### `a11y/aria-role`

Exigir que elementos com papéis ARIA usem um papel ARIA válido e não abstrato

[Incorreto](#a11y-aria-role-bad) · [Correto](#a11y-aria-role-good)

Severidade padrão: `error`  
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
        "a11y/aria-role": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-role-bad"></span>

**Incorreto**

`datepicker` não é um papel ARIA reconhecido para esta seção.

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

<span id="a11y-aria-role-good"></span>

**Correto**

A seção usa o papel reconhecido `dialog` e um rótulo que descreve a seleção da data.

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) · [Todas as regras](all.md)

### `a11y/aria-unsupported-elements`

Proibir atributos ARIA em elementos que não os aceitam

[Incorreto](#a11y-aria-unsupported-elements-bad) · [Correto](#a11y-aria-unsupported-elements-good)

Severidade padrão: `error`  
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
        "a11y/aria-unsupported-elements": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-unsupported-elements-bad"></span>

**Incorreto**

O elemento de metadados tem `aria-hidden`, embora `meta` não aceite atributos ARIA.

```vue annotate="remove:2"
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

<span id="a11y-aria-unsupported-elements-good"></span>

**Correto**

Remover o atributo ARIA mantém intacta a declaração do conjunto de caracteres.

```vue annotate="add:2"
<template>
  <meta charset="utf-8" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) · [Todas as regras](all.md)

### `a11y/click-events-have-key-events`

Exigir manipuladores de eventos de teclado junto aos eventos de clique

[Incorreto](#a11y-click-events-have-key-events-bad) · [Correto](#a11y-click-events-have-key-events-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Verifica elementos não interativos sem uma função interativa. Botões nativos e elementos com uma função ARIA interativa ficam fora dos diagnósticos desta regra.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/click-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-click-events-have-key-events-bad"></span>

**Incorreto**

A `div` não interativa tem um manipulador de clique, mas não trata eventos de teclado.

```vue annotate="remove:2"
<template>
<div @click="activate">Activate</div>
</template>
```

<span id="a11y-click-events-have-key-events-good"></span>

**Correto**

Um `button` nativo permite a ativação pelo teclado para o mesmo manipulador `activate`.

```vue annotate="add:2"
<template>
<button @click="activate">Activate</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) · [Todas as regras](all.md)

### `a11y/form-control-has-label`

Exigir rótulos associados aos controles de formulário

[Incorreto](#a11y-form-control-has-label-bad) · [Correto](#a11y-form-control-has-label-good)

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
        "a11y/form-control-has-label": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-form-control-has-label-bad"></span>

**Incorreto**

O campo de pesquisa não tem um rótulo que identifique o que o usuário deve inserir.

```vue annotate="remove:2"
<template>
  <input type="search" />
</template>
```

<span id="a11y-form-control-has-label-good"></span>

**Correto**

Envolver o campo em um rótulo associa o texto visível `Search` ao controle.

```vue annotate="add:2,3,4,5"
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) · [Todas as regras](all.md)

### `a11y/heading-has-content`

Exigir conteúdo acessível nos elementos de título

[Incorreto](#a11y-heading-has-content-bad) · [Correto](#a11y-heading-has-content-good)

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
        "a11y/heading-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-has-content-bad"></span>

**Incorreto**

O `h2` acrescenta um nível de título, mas não tem conteúdo de título.

```vue annotate="remove:2"
<template>
  <h2></h2>
</template>
```

<span id="a11y-heading-has-content-good"></span>

**Correto**

`Billing settings` fornece o conteúdo do título de nível dois existente.

```vue annotate="add:2"
<template>
  <h2>Billing settings</h2>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) · [Todas as regras](all.md)

### `a11y/heading-levels`

Proibir que níveis de título sejam pulados

[Incorreto](#a11y-heading-levels-bad) · [Correto](#a11y-heading-levels-good)

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
        "a11y/heading-levels": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-levels-bad"></span>

**Incorreto**

A sequência de títulos passa diretamente de `h1` para `h3`, pulando o nível dois.

```vue annotate="remove:3"
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

<span id="a11y-heading-levels-good"></span>

**Correto**

Alterar o título de cobrança para `h2` preserva uma hierarquia consecutiva de títulos.

```vue annotate="add:3"
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) · [Todas as regras](all.md)

### `a11y/iframe-has-title`

Exigir um atributo title nos elementos iframe

[Incorreto](#a11y-iframe-has-title-bad) · [Correto](#a11y-iframe-has-title-good)

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
        "a11y/iframe-has-title": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-iframe-has-title-bad"></span>

**Incorreto**

O quadro de finalização de compra tem uma URL de origem, mas não tem um `title` que descreva o conteúdo incorporado.

```vue annotate="remove:2"
<template>
  <iframe src="/checkout"></iframe>
</template>
```

<span id="a11y-iframe-has-title-good"></span>

**Correto**

`title="Checkout preview"` dá um nome ao conteúdo desse quadro.

```vue annotate="add:2"
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) · [Todas as regras](all.md)

### `a11y/img-alt`

Exigir o atributo alt nas imagens para garantir acessibilidade

[Incorreto](#a11y-img-alt-bad) · [Correto](#a11y-img-alt-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "a11y/img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-img-alt-bad"></span>

**Incorreto**

A imagem do avatar não tem o atributo `alt`.

```vue annotate="remove:2"
<template>
  <img src="/avatar.png" />
</template>
```

<span id="a11y-img-alt-good"></span>

**Correto**

`alt="User avatar"` fornece uma alternativa textual para o avatar.

```vue annotate="add:2"
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) · [Todas as regras](all.md)

### `a11y/interactive-supports-focus`

Exigir que elementos com papéis interativos possam receber foco

[Incorreto](#a11y-interactive-supports-focus-bad) · [Correto](#a11y-interactive-supports-focus-good)

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
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-interactive-supports-focus-bad"></span>

**Incorreto**

Atribuir a um `span` o papel de botão e um manipulador de clique não permite que o elemento receba foco pelo teclado.

```vue annotate="remove:2"
<template>
  <span role="button" @click="open">Open</span>
</template>
```

<span id="a11y-interactive-supports-focus-good"></span>

**Correto**

O botão nativo pode receber foco e mantém a mesma ação `open`.

```vue annotate="add:2"
<template>
  <button type="button" @click="open">Open</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) · [Todas as regras](all.md)

### `a11y/label-has-for`

Exigir controles de formulário associados aos rótulos

[Incorreto](#a11y-label-has-for-bad) · [Correto](#a11y-label-has-for-good)

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
        "a11y/label-has-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-label-has-for-bad"></span>

**Incorreto**

O rótulo separado não está associado por meio de `for` nem envolve o campo.

```vue annotate="remove:2"
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

<span id="a11y-label-has-for-good"></span>

**Correto**

`for="email"` corresponde ao ID do campo e associa explicitamente os dois elementos.

```vue annotate="add:2"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) · [Todas as regras](all.md)

### `a11y/landmark-roles`

Validar a posição e a unicidade dos papéis de regiões de referência

[Incorreto](#a11y-landmark-roles-bad) · [Correto](#a11y-landmark-roles-good)

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
        "a11y/landmark-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-landmark-roles-bad"></span>

**Incorreto**

Dois elementos `main` declaram regiões de referência principais duplicadas no mesmo template.

```vue annotate="remove:3"
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

<span id="a11y-landmark-roles-good"></span>

**Correto**

O painel permanece como a região de referência principal; a área de configurações se torna uma região de referência de navegação com nome.

```vue annotate="add:3"
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) · [Todas as regras](all.md)

### `a11y/media-has-caption`

Exigir legendas nos elementos de mídia

[Incorreto](#a11y-media-has-caption-bad) · [Correto](#a11y-media-has-caption-good)

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
        "a11y/media-has-caption": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-media-has-caption-bad"></span>

**Incorreto**

O vídeo tem controles de reprodução, mas não tem uma faixa de legendas.

```vue annotate="remove:2"
<template>
  <video src="/demo.mp4" controls />
</template>
```

<span id="a11y-media-has-caption-good"></span>

**Correto**

Um `track` com `kind="captions"` fornece as legendas em inglês para o mesmo vídeo.

```vue annotate="add:2,3,4"
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) · [Todas as regras](all.md)

### `a11y/mouse-events-have-key-events`

Exigir eventos de foco e perda de foco junto aos eventos de mouse

[Incorreto](#a11y-mouse-events-have-key-events-bad) · [Correto](#a11y-mouse-events-have-key-events-good)

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
        "a11y/mouse-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-mouse-events-have-key-events-bad"></span>

**Incorreto**

A visibilidade da prévia muda apenas por meio dos manipuladores de entrada e saída do mouse.

```vue annotate="remove:2"
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

<span id="a11y-mouse-events-have-key-events-good"></span>

**Correto**

As mesmas ações da prévia são executadas ao receber e perder o foco, e o botão pode receber foco pelo teclado.

```vue annotate="add:2,3,4,5,6,7,8,9,10"
<template>
  <button
    type="button"
    @focus="showPreview"
    @blur="hidePreview"
    @mouseenter="showPreview"
    @mouseleave="hidePreview"
  >
    Preview
  </button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) · [Todas as regras](all.md)

### `a11y/no-access-key`

Proibir o uso do atributo accesskey

[Incorreto](#a11y-no-access-key-bad) · [Correto](#a11y-no-access-key-good)

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
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-access-key-bad"></span>

**Incorreto**

O atalho `accesskey="s"` pode entrar em conflito com atalhos do navegador ou de tecnologias assistivas.

```vue annotate="remove:2"
<template>
  <button accesskey="s">Save</button>
</template>
```

<span id="a11y-no-access-key-good"></span>

**Correto**

Remover `accesskey` mantém disponível o botão Save comum.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) · [Todas as regras](all.md)

### `a11y/no-aria-hidden-on-focusable`

Proibir aria-hidden="true" em elementos que podem receber foco

[Incorreto](#a11y-no-aria-hidden-on-focusable-bad) · [Correto](#a11y-no-aria-hidden-on-focusable-good)

Severidade padrão: `error`  
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
        "a11y/no-aria-hidden-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

**Incorreto**

O botão Close, que pode receber foco, é ocultado da árvore de acessibilidade com `aria-hidden="true"`.

```vue annotate="remove:2"
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

**Correto**

O botão permanece exposto e recebe um rótulo `Close`, em vez de ser ocultado.

```vue annotate="add:2"
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) · [Todas as regras](all.md)

### `a11y/no-autofocus`

Proibir o uso do atributo autofocus

[Incorreto](#a11y-no-autofocus-bad) · [Correto](#a11y-no-autofocus-good)

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
        "a11y/no-autofocus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-autofocus-bad"></span>

**Incorreto**

O campo solicita foco automático quando aparece.

```vue annotate="remove:2"
<template>
  <input autofocus name="query" />
</template>
```

<span id="a11y-no-autofocus-good"></span>

**Correto**

Remover `autofocus` evita essa solicitação de foco automático e mantém o campo de consulta.

```vue annotate="add:2"
<template>
  <input name="query" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) · [Todas as regras](all.md)

### `a11y/no-distracting-elements`

Proibir elementos que causam distração, como &lt;marquee&gt; e &lt;blink&gt;

[Incorreto](#a11y-no-distracting-elements-bad) · [Correto](#a11y-no-distracting-elements-good)

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
        "a11y/no-distracting-elements": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-distracting-elements-bad"></span>

**Incorreto**

O elemento `marquee` introduz texto que se move automaticamente.

```vue annotate="remove:2"
<template>
  <marquee>Limited offer</marquee>
</template>
```

<span id="a11y-no-distracting-elements-good"></span>

**Correto**

Um parágrafo exibe a mesma oferta sem o elemento marquee que causa distração.

```vue annotate="add:2"
<template>
  <p>Limited offer</p>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) · [Todas as regras](all.md)

### `a11y/no-i-for-icon`

Proibir o uso do elemento &lt;i&gt; para ícones

[Incorreto](#a11y-no-i-for-icon-bad) · [Correto](#a11y-no-i-for-icon-good)

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
        "a11y/no-i-for-icon": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-i-for-icon-bad"></span>

**Incorreto**

O ícone é renderizado por meio de `i`, cuja semântica de texto não descreve uma ação representada apenas por um ícone.

```vue annotate="remove:3"
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

<span id="a11y-no-i-for-icon-good"></span>

**Correto**

Um span decorativo oculta o glifo do ícone, enquanto o texto separado `Delete item` dá um nome à ação do botão.

```vue annotate="add:3,4"
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) · [Todas as regras](all.md)

### `a11y/no-redundant-roles`

Proibir papéis ARIA redundantes

[Incorreto](#a11y-no-redundant-roles-bad) · [Correto](#a11y-no-redundant-roles-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
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
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-redundant-roles-bad"></span>

**Incorreto**

O botão nativo já tem o papel de botão, então `role="button"` repete sua semântica implícita.

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

<span id="a11y-no-redundant-roles-good"></span>

**Correto**

Remover o papel repetido mantém a semântica de botão fornecida pelo HTML.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) · [Todas as regras](all.md)

### `a11y/no-refer-to-non-existent-id`

Proibir referências a IDs inexistentes

[Incorreto](#a11y-no-refer-to-non-existent-id-bad) · [Correto](#a11y-no-refer-to-non-existent-id-good)

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
        "a11y/no-refer-to-non-existent-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

**Incorreto**

`aria-labelledby` aponta para `save-label`, mas nenhum elemento declara esse ID.

```vue
<template>
  <button aria-labelledby="save-label">Save</button>
</template>
```

<span id="a11y-no-refer-to-non-existent-id-good"></span>

**Correto**

Adicionar o span correspondente resolve a referência e fornece o rótulo do botão.

```vue annotate="add:2"
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) · [Todas as regras](all.md)

### `a11y/no-role-presentation-on-focusable`

Proibir role="presentation" ou role="none" em elementos que podem receber foco

[Incorreto](#a11y-no-role-presentation-on-focusable-bad) · [Correto](#a11y-no-role-presentation-on-focusable-good)

Severidade padrão: `error`  
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
        "a11y/no-role-presentation-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

**Incorreto**

O link de cobrança, que pode receber foco, solicita role=presentation, o que entra em conflito com seu papel interativo de link; os navegadores devem ignorar essa solicitação de apresentação.

```vue annotate="remove:2"
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

<span id="a11y-no-role-presentation-on-focusable-good"></span>

**Correto**

Remova a solicitação de apresentação conflitante e use o papel nativo de link e o destino de cobrança.

```vue annotate="add:2"
<template>
  <a href="/billing">Billing</a>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) · [Todas as regras](all.md)

### `a11y/no-static-element-interactions`

Proibir manipuladores de eventos em elementos estáticos

[Incorreto](#a11y-no-static-element-interactions-bad) · [Correto](#a11y-no-static-element-interactions-good)

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
        "a11y/no-static-element-interactions": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-static-element-interactions-bad"></span>

**Incorreto**

Uma seção estática recebe uma ação da tecla Enter sem ter um papel interativo.

```vue annotate="remove:2"
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

<span id="a11y-no-static-element-interactions-good"></span>

**Correto**

Um botão nativo executa a mesma ação usando um elemento interativo adequado.

```vue annotate="add:2"
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) · [Todas as regras](all.md)

### `a11y/placeholder-label-option`

Exigir disabled ou hidden na opção de orientação de um select

[Incorreto](#a11y-placeholder-label-option-bad) · [Correto](#a11y-placeholder-label-option-good)

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
        "a11y/placeholder-label-option": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-placeholder-label-option-bad"></span>

**Incorreto**

A opção de orientação com valor vazio permanece selecionável como se fosse um valor de país.

```vue annotate="remove:3"
<template>
  <select v-model="country">
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

<span id="a11y-placeholder-label-option-good"></span>

**Correto**

Adicionar `disabled` distingue a orientação da opção Japão, que pode ser selecionada.

```vue annotate="add:3"
<template>
  <select v-model="country">
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L36) · [Todas as regras](all.md)

### `a11y/role-has-required-aria-props`

Exigir as propriedades obrigatórias dos papéis ARIA

[Incorreto](#a11y-role-has-required-aria-props-bad) · [Correto](#a11y-role-has-required-aria-props-good)

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
        "a11y/role-has-required-aria-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-role-has-required-aria-props-bad"></span>

**Incorreto**

O papel de caixa de seleção omite `aria-checked`, que informa o estado da caixa de seleção.

```vue annotate="remove:2"
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

<span id="a11y-role-has-required-aria-props-good"></span>

**Correto**

`aria-checked="false"` fornece o estado exigido pelo papel de caixa de seleção.

```vue annotate="add:2"
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) · [Todas as regras](all.md)

### `a11y/tabindex-no-positive`

Proibir valores positivos de tabindex

[Incorreto](#a11y-tabindex-no-positive-bad) · [Correto](#a11y-tabindex-no-positive-good)

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
        "a11y/tabindex-no-positive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-tabindex-no-positive-bad"></span>

**Incorreto**

Um tabindex positivo de 3 cria uma ordem de foco personalizada antes dos controles comuns.

```vue annotate="remove:2"
<template>
  <button tabindex="3">Save</button>
</template>
```

<span id="a11y-tabindex-no-positive-good"></span>

**Correto**

O botão usa sua ordem de foco nativa sem um tabindex positivo.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) · [Todas as regras](all.md)

### `a11y/use-list`

Sugerir elementos de lista para textos que parecem itens de lista

[Incorreto](#a11y-use-list-bad) · [Correto](#a11y-use-list-good)

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
        "a11y/use-list": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-use-list-bad"></span>

**Incorreto**

Os itens das tarefas são parágrafos separados com marcadores de hífen digitados, em vez de elementos de lista.

```vue annotate="remove:2,3"
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

<span id="a11y-use-list-good"></span>

**Correto**

Uma lista não ordenada e seus itens expressam as mesmas tarefas com semântica de lista.

```vue annotate="add:2,3,4,5"
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) · [Todas as regras](all.md)

### `vue/use-unique-element-ids`

Exigir IDs de elementos únicos por meio de useId(), em vez de literais estáticos

[Incorreto](#vue-use-unique-element-ids-bad) · [Correto](#vue-use-unique-element-ids-good)

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
        "vue/use-unique-element-ids": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-unique-element-ids-bad"></span>

**Incorreto**

O ID literal `email` é reutilizado por todas as instâncias deste componente, o que pode direcionar seu rótulo ao elemento errado quando várias instâncias são renderizadas.

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**Correto**

`useId()` produz o `emailId` da instância; vincule o mesmo valor ao `for` do rótulo e ao `id` do input.

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [Todas as regras](all.md)
