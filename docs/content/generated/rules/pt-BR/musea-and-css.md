---
title: "Regras de Musea e CSS"
---

# Regras de Musea e CSS

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.

<span id="regras-adicionais-de-css"></span>

| Regra | Exemplos | Finalidade |
| --- | --- | --- |
| [`css/no-display-none`](#css-no-display-none) | [Incorreto](#css-no-display-none-bad) · [Correto](#css-no-display-none-good) | Sugerir v-show em vez de display: none |
| [`css/no-hardcoded-values`](#css-no-hardcoded-values) | [Incorreto](#css-no-hardcoded-values-bad) · [Correto](#css-no-hardcoded-values-good) | Sugerir variáveis CSS em vez de valores fixos escritos diretamente |
| [`css/no-id-selectors`](#css-no-id-selectors) | [Incorreto](#css-no-id-selectors-bad) · [Correto](#css-no-id-selectors-good) | Desencorajar o uso de seletores de ID no CSS |
| [`css/no-important`](#css-no-important) | [Incorreto](#css-no-important-bad) · [Correto](#css-no-important-good) | Desencorajar o uso de !important no CSS |
| [`css/no-utility-classes`](#css-no-utility-classes) | [Incorreto](#css-no-utility-classes-bad) · [Correto](#css-no-utility-classes-good) | Alertar sobre a implementação de classes utilitárias nos estilos de componentes |
| [`css/no-v-bind-performance`](#css-no-v-bind-performance) | [Incorreto](#css-no-v-bind-performance-bad) · [Correto](#css-no-v-bind-performance-good) | Alertar sobre o custo de desempenho do v-bind() no CSS |
| [`css/prefer-logical-properties`](#css-prefer-logical-properties) | [Incorreto](#css-prefer-logical-properties-bad) · [Correto](#css-prefer-logical-properties-good) | Recomendar propriedades lógicas de CSS para melhorar o suporte à internacionalização |
| [`css/prefer-nested-selectors`](#css-prefer-nested-selectors) | [Incorreto](#css-prefer-nested-selectors-bad) · [Correto](#css-prefer-nested-selectors-good) | Recomendar o aninhamento de CSS para seletores de descendentes |
| [`css/prefer-slotted`](#css-prefer-slotted) | [Incorreto](#css-prefer-slotted-bad) · [Correto](#css-prefer-slotted-good) | Recomendar ::v-slotted() para estilizar o conteúdo de slots |
| [`css/require-font-display`](#css-require-font-display) | [Incorreto](#css-require-font-display-bad) · [Correto](#css-require-font-display-good) | Exigir font-display nas regras @font-face |
| [`musea/no-empty-variant`](#musea-no-empty-variant) | [Incorreto](#musea-no-empty-variant-bad) · [Correto](#musea-no-empty-variant-good) | Proibir blocos &lt;variant&gt; vazios |
| [`musea/prefer-design-tokens`](#musea-prefer-design-tokens) | [Incorreto](#musea-prefer-design-tokens-bad) · [Correto](#musea-prefer-design-tokens-good) | Preferir variáveis CSS de tokens de design a valores primitivos fixos escritos diretamente |
| [`musea/require-component`](#musea-require-component) | [Incorreto](#musea-require-component-bad) · [Correto](#musea-require-component-good) | Exigir o atributo component no bloco &lt;art&gt; |
| [`musea/require-title`](#musea-require-title) | [Incorreto](#musea-require-title-bad) · [Correto](#musea-require-title-good) | Exigir o atributo title no bloco &lt;art&gt; |
| [`musea/unique-variant-names`](#musea-unique-variant-names) | [Incorreto](#musea-unique-variant-names-bad) · [Correto](#musea-unique-variant-names-good) | Exigir nomes de variante únicos |
| [`musea/valid-variant`](#musea-valid-variant) | [Incorreto](#musea-valid-variant-bad) · [Correto](#musea-valid-variant-good) | Exigir o atributo name nos blocos &lt;variant&gt; |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `css/no-display-none`

Sugerir v-show em vez de display: none

[Incorreto](#css-no-display-none-bad) · [Correto](#css-no-display-none-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`, `nuxt`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: CSS dentro dos blocos style de SFCs  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-display-none": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-display-none-bad"></span>

**Incorreto**

A declaração `.message` oculta o parágrafo local por meio de CSS, em vez de uma condição de visibilidade no template.

```vue annotate="remove:2,4,5,6,7,8,9"
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

<span id="css-no-display-none-good"></span>

**Correto**

`v-show="isSaved"` torna explícita a condição de visibilidade no parágrafo local e remove `display: none`.

```vue annotate="add:2"
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) · [Todas as regras](all.md)

### `css/no-hardcoded-values`

Sugerir variáveis CSS em vez de valores fixos escritos diretamente

[Incorreto](#css-no-hardcoded-values-bad) · [Correto](#css-no-hardcoded-values-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`, `nuxt`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: CSS dentro dos blocos style de SFCs  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-hardcoded-values": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-hardcoded-values-bad"></span>

**Incorreto**

O botão incorpora números de espaçamento e uma cor hexadecimal diretamente nas declarações.

```vue annotate="remove:3,4"
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

<span id="css-no-hardcoded-values-good"></span>

**Correto**

As declarações fazem referência a propriedades personalizadas de espaçamento e cor com nomes, permitindo manter esses valores como tokens.

```vue annotate="add:3,4"
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30) · [Todas as regras](all.md)

### `css/no-id-selectors`

Desencorajar o uso de seletores de ID no CSS

[Incorreto](#css-no-id-selectors-bad) · [Correto](#css-no-id-selectors-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`, `nuxt`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: CSS dentro dos blocos style de SFCs  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-id-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-id-selectors-bad"></span>

**Incorreto**

`#submit` vincula a regra de estilo a um seletor de ID.

```vue annotate="remove:2"
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

<span id="css-no-id-selectors-good"></span>

**Correto**

A classe `.submit` fornece um ponto de aplicação de estilo reutilizável sem um seletor de ID.

```vue annotate="add:2"
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) · [Todas as regras](all.md)

### `css/no-important`

Desencorajar o uso de !important no CSS

[Incorreto](#css-no-important-bad) · [Correto](#css-no-important-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`, `nuxt`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: CSS dentro dos blocos style de SFCs  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-important": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-important-bad"></span>

**Incorreto**

A declaração de cor substitui a prioridade normal da cascata com `!important`.

```vue annotate="remove:3"
<style scoped>
.button {
  color: red !important;
}
</style>
```

<span id="css-no-important-good"></span>

**Correto**

A cor vem de uma propriedade personalizada sem uma declaração importante.

```vue annotate="add:3"
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) · [Todas as regras](all.md)

### `css/no-utility-classes`

Alertar sobre a implementação de classes utilitárias nos estilos de componentes

[Incorreto](#css-no-utility-classes-bad) · [Correto](#css-no-utility-classes-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`, `nuxt`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: CSS dentro dos blocos style de SFCs  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-utility-classes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-utility-classes-bad"></span>

**Incorreto**

Os seletores definidos usam nomes típicos de utilitários, como `.flex`, `.mt-4` e `.text-center`.

```vue annotate="remove:2,3,4"
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

<span id="css-no-utility-classes-good"></span>

**Correto**

Um seletor `.my-component` específico do componente agrupa seus estilos sob um único nome semântico.

```vue annotate="add:2"
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) · [Todas as regras](all.md)

### `css/no-v-bind-performance`

Alertar sobre o custo de desempenho do v-bind() no CSS

[Incorreto](#css-no-v-bind-performance-bad) · [Correto](#css-no-v-bind-performance-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`, `nuxt`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: CSS dentro dos blocos style de SFCs  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-v-bind-performance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-v-bind-performance-bad"></span>

**Incorreto**

A folha de estilos lê o valor variável de `offset` por meio do mecanismo `v-bind()` do CSS de SFC.

```vue annotate="remove:1,2,3,4,5"
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

<span id="css-no-v-bind-performance-good"></span>

**Correto**

O elemento recebe a transformação variável diretamente por meio da vinculação de estilo.

```vue annotate="add:1,2,3"
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) · [Todas as regras](all.md)

### `css/prefer-logical-properties`

Recomendar propriedades lógicas de CSS para melhorar o suporte à internacionalização

[Incorreto](#css-prefer-logical-properties-bad) · [Correto](#css-prefer-logical-properties-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`, `nuxt`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: CSS dentro dos blocos style de SFCs  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-logical-properties-bad"></span>

**Incorreto**

`margin-left` fixa a margem em um lado físico, independentemente da direção de escrita.

```vue annotate="remove:3"
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

<span id="css-prefer-logical-properties-good"></span>

**Correto**

`margin-inline-start` acompanha o início da direção em linha.

```vue annotate="add:3"
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) · [Todas as regras](all.md)

### `css/prefer-nested-selectors`

Recomendar o aninhamento de CSS para seletores de descendentes

[Incorreto](#css-prefer-nested-selectors-bad) · [Correto](#css-prefer-nested-selectors-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`, `nuxt`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: CSS dentro dos blocos style de SFCs  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-nested-selectors-bad"></span>

**Incorreto**

O seletor de descendentes `.card .title` repete o seletor pai em uma regra sem aninhamento.

```vue annotate="remove:2"
<style scoped>
.card .title { color: red; }
</style>
```

<span id="css-prefer-nested-selectors-good"></span>

**Correto**

A regra `.title` é aninhada dentro de `.card`, mantendo junta a relação de estilo entre pai e filho.

```vue annotate="add:2"
<style scoped>
.card { .title { color: red; } }
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) · [Todas as regras](all.md)

### `css/prefer-slotted`

Recomendar ::v-slotted() para estilizar o conteúdo de slots

[Incorreto](#css-prefer-slotted-bad) · [Correto](#css-prefer-slotted-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`, `nuxt`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: CSS dentro dos blocos style de SFCs  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-slotted": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-slotted-bad"></span>

**Incorreto**

A folha de estilos com escopo tem como alvo o ponto de inserção `slot`, em vez dos elementos fornecidos pelo slot.

```vue annotate="remove:2"
<style scoped>
slot { color: red; }
</style>
```

<span id="css-prefer-slotted-good"></span>

**Correto**

`:slotted(.label)` tem como alvo o elemento de rótulo fornecido, por meio do seletor de slot com escopo.

```vue annotate="add:2"
<style scoped>
:slotted(.label) { color: red; }
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) · [Todas as regras](all.md)

### `css/require-font-display`

Exigir font-display nas regras @font-face

[Incorreto](#css-require-font-display-bad) · [Correto](#css-require-font-display-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`, `nuxt`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: CSS dentro dos blocos style de SFCs  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/require-font-display": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-require-font-display-bad"></span>

**Incorreto**

A declaração font-face define a origem da fonte, mas omite sua política font-display.

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

<span id="css-require-font-display-good"></span>

**Correto**

`font-display: swap` seleciona explicitamente a política de exibir uma fonte substituta e depois a fonte carregada.

```vue annotate="add:5"
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13) · [Todas as regras](all.md)

### `musea/no-empty-variant`

Proibir blocos &lt;variant&gt; vazios

[Incorreto](#musea-no-empty-variant-bad) · [Correto](#musea-no-empty-variant-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Blocos art, variant e style de arquivos .art.vue do Musea  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/no-empty-variant": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-no-empty-variant-bad"></span>

**Incorreto**

A variante chamada primary está vazia, então não fornece conteúdo para a prévia.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-no-empty-variant-good"></span>

**Correto**

A variante renderiza um Button primary com seu conteúdo Save.

```vue annotate="add:2,3,4"
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8) · [Todas as regras](all.md)

### `musea/prefer-design-tokens`

Preferir variáveis CSS de tokens de design a valores primitivos fixos escritos diretamente

[Incorreto](#musea-prefer-design-tokens-bad) · [Correto](#musea-prefer-design-tokens-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Blocos art, variant e style de arquivos .art.vue do Musea  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

Exige um arquivo .art.vue e o inventário de tokens mostrado abaixo. Não infere um token a partir de uma cor arbitrária.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-prefer-design-tokens-bad"></span>

**Incorreto**

O exemplo art usa a cor azul literal, em vez do token de design primary configurado.

`Button.art.vue`

```vue annotate="remove:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

<span id="musea-prefer-design-tokens-good"></span>

**Correto**

O estilo faz referência a --color-primary, o token configurado para este exemplo.

`Button.art.vue`

```vue annotate="add:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) · [Todas as regras](all.md)

### `musea/require-component`

Exigir o atributo component no bloco &lt;art&gt;

[Incorreto](#musea-require-component-bad) · [Correto](#musea-require-component-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Blocos art, variant e style de arquivos .art.vue do Musea  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-component": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-component-bad"></span>

**Incorreto**

O bloco art fornece um título, mas não identifica o componente exibido na prévia.

```vue annotate="remove:1"
<art title="Button">
  <variant name="primary" />
</art>
```

<span id="musea-require-component-good"></span>

**Correto**

defineArt fornece ./Button.vue como o componente do bloco art.

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) · [Todas as regras](all.md)

### `musea/require-title`

Exigir o atributo title no bloco &lt;art&gt;

[Incorreto](#musea-require-title-bad) · [Correto](#musea-require-title-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Blocos art, variant e style de arquivos .art.vue do Musea  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-title": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-title-bad"></span>

**Incorreto**

O bloco art identifica Button.vue, mas não fornece um título.

```vue annotate="remove:1"
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-require-title-good"></span>

**Correto**

As opções de defineArt fornecem o título Button para o bloco art.

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) · [Todas as regras](all.md)

### `musea/unique-variant-names`

Exigir nomes de variante únicos

[Incorreto](#musea-unique-variant-names-bad) · [Correto](#musea-unique-variant-names-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Blocos art, variant e style de arquivos .art.vue do Musea  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/unique-variant-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-unique-variant-names-bad"></span>

**Incorreto**

Duas variantes no mesmo bloco art usam o nome primary.

```vue annotate="remove:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

<span id="musea-unique-variant-names-good"></span>

**Correto**

As variantes têm os nomes distintos primary e secondary.

```vue annotate="add:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10) · [Todas as regras](all.md)

### `musea/valid-variant`

Exigir o atributo name nos blocos &lt;variant&gt;

[Incorreto](#musea-valid-variant-bad) · [Correto](#musea-valid-variant-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Blocos art, variant e style de arquivos .art.vue do Musea  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/valid-variant": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-valid-variant-bad"></span>

**Incorreto**

A variante omite o nome necessário para identificar a prévia.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

<span id="musea-valid-variant-good"></span>

**Correto**

O nome primary identifica essa variante.

```vue annotate="add:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) · [Todas as regras](all.md)
