---
title: "Regras de Vapor"
---

# Regras de Vapor

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.


| Regra | Exemplos | Finalidade |
| --- | --- | --- |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [Incorreto](#script-no-get-current-instance-bad) · [Correto](#script-no-get-current-instance-good) | Proibir getCurrentInstance() no modo Vapor (retorna null) |
| [`script/no-next-tick`](#script-no-next-tick) | [Incorreto](#script-no-next-tick-bad) · [Correto](#script-no-next-tick-good) | Proibir o uso de nextTick() em componentes orientados a Vapor |
| [`script/no-options-api`](#script-no-options-api) | [Incorreto](#script-no-options-api-bad) · [Correto](#script-no-options-api-good) | Proibir padrões da Options API no modo Vapor |
| [`vapor/no-inline-template`](#vapor-no-inline-template) | [Incorreto](#vapor-no-inline-template-bad) · [Correto](#vapor-no-inline-template-good) | Proibir o atributo obsoleto inline-template |
| [`vapor/no-vue-lifecycle-events`](#vapor-no-vue-lifecycle-events) | [Incorreto](#vapor-no-vue-lifecycle-events-bad) · [Correto](#vapor-no-vue-lifecycle-events-good) | Proibir eventos de ciclo de vida @vue:xxx por elemento (não suportados em Vapor) |
| [`vapor/prefer-static-class`](#vapor-prefer-static-class) | [Incorreto](#vapor-prefer-static-class-bad) · [Correto](#vapor-prefer-static-class-good) | Preferir class estática a uma vinculação dinâmica de class para literais de string |
| [`vapor/require-vapor-attribute`](#vapor-require-vapor-attribute) | [Incorreto](#vapor-require-vapor-attribute-bad) · [Correto](#vapor-require-vapor-attribute-good) | Sugerir a adição do atributo vapor a script setup |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `script/no-get-current-instance`

Proibir getCurrentInstance() no modo Vapor (retorna null)

[Incorreto](#script-no-get-current-instance-bad) · [Correto](#script-no-get-current-instance-good)

Severidade padrão: `error`  
Predefinições: `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Verificações de script voltadas ao Vapor; a ativação explícita também aplica a restrição a scripts comuns  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-get-current-instance-bad"></span>

**Incorreto**

O setup marcado como Vapor importa e chama `getCurrentInstance`, dependendo de uma API de instância que esta regra proíbe para componentes orientados a Vapor.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

<span id="script-no-get-current-instance-good"></span>

**Correto**

`inject("app-config")` obtém a configuração fornecida explicitamente sem importar nem chamar `getCurrentInstance`.

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [Todas as regras](all.md)

### `script/no-next-tick`

Proibir o uso de nextTick() em componentes orientados a Vapor

[Incorreto](#script-no-next-tick-bad) · [Correto](#script-no-next-tick-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Verificações de script voltadas ao Vapor; a ativação explícita também aplica a restrição a scripts comuns  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-next-tick-bad"></span>

**Incorreto**

O componente orientado a Vapor importa e aguarda `nextTick`, introduzindo a dependência do agendamento da atualização do DOM que esta regra de migração rejeita.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

<span id="script-no-next-tick-good"></span>

**Correto**

O input é obtido por `useTemplateRef` e recebe foco em `onMounted`. O momento explícito de montagem substitui a dependência de `nextTick` do exemplo.

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [Todas as regras](all.md)

### `script/no-options-api`

Proibir padrões da Options API no modo Vapor

[Incorreto](#script-no-options-api-bad) · [Correto](#script-no-options-api-good)

Severidade padrão: `error`  
Predefinições: `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Verificações de script voltadas ao Vapor; a ativação explícita também aplica a restrição a scripts comuns  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-options-api-bad"></span>

**Incorreto**

O objeto da exportação padrão declara `data()` da Options API, uma forma de opção de componente proibida por esta regra.

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

<span id="script-no-options-api-good"></span>

**Correto**

O estado do componente passa a ser um `ref` da Composition API em `<script setup>` com Vapor, removendo o objeto da Options API e sua opção `data`.

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [Todas as regras](all.md)

### `vapor/no-inline-template`

Proibir o atributo obsoleto inline-template

[Incorreto](#vapor-no-inline-template-bad) · [Correto](#vapor-no-inline-template-good)

Severidade padrão: `error`  
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
        "vapor/no-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-inline-template-bad"></span>

**Incorreto**

LegacyCard usa o atributo inline-template para a marcação de seu conteúdo filho.

```vue annotate="remove:2,3"
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

<span id="vapor-no-inline-template-good"></span>

**Correto**

A marcação é passada pelo slot padrão em vez de um template inline.

```vue annotate="add:2,3,4,5"
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) · [Todas as regras](all.md)

### `vapor/no-vue-lifecycle-events`

Proibir eventos de ciclo de vida @vue:xxx por elemento (não suportados em Vapor)

[Incorreto](#vapor-no-vue-lifecycle-events-bad) · [Correto](#vapor-no-vue-lifecycle-events-good)

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
        "vapor/no-vue-lifecycle-events": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-vue-lifecycle-events-bad"></span>

**Incorreto**

O input usa o evento de ciclo de vida de template @vue:mounted.

```vue annotate="remove:2"
<template>
  <input @vue:mounted="focusInput" />
</template>
```

<span id="vapor-no-vue-lifecycle-events-good"></span>

**Correto**

onMounted acessa a referência nomeada de template e coloca o foco no input por meio do hook de ciclo de vida de script suportado.

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) · [Todas as regras](all.md)

### `vapor/prefer-static-class`

Preferir class estática a uma vinculação dinâmica de class para literais de string

[Incorreto](#vapor-prefer-static-class-bad) · [Correto](#vapor-prefer-static-class-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
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
        "vapor/prefer-static-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-prefer-static-class-bad"></span>

**Incorreto**

A vinculação de class avalia uma string constante, embora a classe não mude.

```vue annotate="remove:2"
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

<span id="vapor-prefer-static-class-good"></span>

**Correto**

Um atributo class estático expressa as mesmas classes do painel sem uma vinculação.

```vue annotate="add:2"
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) · [Todas as regras](all.md)

### `vapor/require-vapor-attribute`

Sugerir a adição do atributo vapor a script setup

[Incorreto](#vapor-require-vapor-attribute-bad) · [Correto](#vapor-require-vapor-attribute-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Não implementada no lint de SFC  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Suporte atual: `no-sfc-finding`

Esta regra é um marcador provisório com callback vazio. Adicionar vapor seleciona a compilação Vapor; o linter atual não reporta este ID do catálogo pela ausência desse atributo.

**ID configurado (sem diagnóstico de SFC atualmente)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-require-vapor-attribute-bad"></span>

**Incorreto**

O bloco script setup não tem o atributo de compilação Vapor. Esta é uma convenção pretendida: a função de retorno atualmente vazia da regra não diagnostica sua ausência.

```vue annotate="remove:1"
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

<span id="vapor-require-vapor-attribute-good"></span>

**Correto**

Adicionar vapor seleciona a compilação Vapor. Isso demonstra a correção pretendida e não implica que o linter atual emita esta regra do catálogo.

```vue annotate="add:1"
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

O exemplo correto ilustra a convenção pretendida; o fluxo atual de SFC não emite o diagnóstico específico da regra para nenhum dos exemplos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) · [Todas as regras](all.md)
