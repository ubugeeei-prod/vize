---
title: "Regras do Ecossistema"
---

# Regras do Ecossistema

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.


| Regra | Exemplos | Finalidade |
| --- | --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](#ecosystem-nuxt-prefer-nuxt-link) | [Incorreto](#ecosystem-nuxt-prefer-nuxt-link-bad) · [Correto](#ecosystem-nuxt-prefer-nuxt-link-good) | Preferir NuxtLink para links internos da aplicação |
| [`ecosystem/pinia-prefer-store-to-refs`](#ecosystem-pinia-prefer-store-to-refs) | [Incorreto](#ecosystem-pinia-prefer-store-to-refs-bad) · [Correto](#ecosystem-pinia-prefer-store-to-refs-good) | Preferir storeToRefs() ao desestruturar stores do Pinia |
| [`ecosystem/router-link-require-to`](#ecosystem-router-link-require-to) | [Incorreto](#ecosystem-router-link-require-to-bad) · [Correto](#ecosystem-router-link-require-to-good) | Exigir um destino `to` nos componentes RouterLink e NuxtLink |
| [`ecosystem/void-link-require-href`](#ecosystem-void-link-require-href) | [Incorreto](#ecosystem-void-link-require-href-bad) · [Correto](#ecosystem-void-link-require-href-good) | Exigir `href` nos componentes Link do Void Vue |
| [`ecosystem/void-link-valid-method`](#ecosystem-void-link-valid-method) | [Incorreto](#ecosystem-void-link-valid-method-bad) · [Correto](#ecosystem-void-link-valid-method-good) | Validar props method estáticas do Link do Void Vue |
| [`ecosystem/vue-i18n-no-missing-key`](#ecosystem-vue-i18n-no-missing-key) | [Incorreto](#ecosystem-vue-i18n-no-missing-key-bad) · [Correto](#ecosystem-vue-i18n-no-missing-key-good) | Relatar chaves estáticas do vue-i18n ausentes nas mensagens locais do SFC |
| [`ecosystem/vue-router-prefer-named-link`](#ecosystem-vue-router-prefer-named-link) | [Incorreto](#ecosystem-vue-router-prefer-named-link-bad) · [Correto](#ecosystem-vue-router-prefer-named-link-good) | Preferir objetos de rotas nomeadas a strings de caminho estáticas no RouterLink |
| [`ecosystem/vue-router-prefer-named-push`](#ecosystem-vue-router-prefer-named-push) | [Incorreto](#ecosystem-vue-router-prefer-named-push-bad) · [Correto](#ecosystem-vue-router-prefer-named-push-good) | Preferir objetos de rotas nomeadas para navegação programática com Vue Router |
| [`ecosystem/vue-test-utils-no-html-snapshot`](#ecosystem-vue-test-utils-no-html-snapshot) | [Incorreto](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Correto](#ecosystem-vue-test-utils-no-html-snapshot-good) | Evitar snapshots de wrapper.html() nos testes com Vue Test Utils |
| [`nuxt/no-nuxt-config-test-key`](#nuxt-no-nuxt-config-test-key) | [Incorreto](#nuxt-no-nuxt-config-test-key-bad) · [Correto](#nuxt-no-nuxt-config-test-key-good) | Proibir a definição da chave `test` na configuração do Nuxt |
| [`nuxt/no-page-meta-runtime-values`](#nuxt-no-page-meta-runtime-values) | [Incorreto](#nuxt-no-page-meta-runtime-values-bad) · [Correto](#nuxt-no-page-meta-runtime-values-good) | Proibir valores do contexto de execução no nível de avaliação imediata de `definePageMeta`, que é extraído para um fragmento separado durante a compilação e executado antes do setup do componente |
| [`nuxt/nuxt-config-keys-order`](#nuxt-nuxt-config-keys-order) | [Incorreto](#nuxt-nuxt-config-keys-order-bad) · [Correto](#nuxt-nuxt-config-keys-order-good) | Preferir a ordem recomendada das propriedades de configuração do Nuxt |
| [`nuxt/prefer-import-meta`](#nuxt-prefer-import-meta) | [Incorreto](#nuxt-prefer-import-meta-bad) · [Correto](#nuxt-prefer-import-meta-good) | Preferir `import.meta.*` a `process.*` |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `ecosystem/nuxt-prefer-nuxt-link`

Preferir NuxtLink para links internos da aplicação

[Incorreto](#ecosystem-nuxt-prefer-nuxt-link-bad) · [Correto](#ecosystem-nuxt-prefer-nuxt-link-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`  
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
        "ecosystem/nuxt-prefer-nuxt-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

**Incorreto**

O destino interno de configurações usa uma âncora comum em uma aplicação Nuxt.

```vue annotate="remove:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

**Correto**

NuxtLink trata o mesmo destino interno por meio do roteador do Nuxt.

```vue annotate="add:2"
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) · [Todas as regras](all.md)

### `ecosystem/pinia-prefer-store-to-refs`

Preferir storeToRefs() ao desestruturar stores do Pinia

[Incorreto](#ecosystem-pinia-prefer-store-to-refs-bad) · [Correto](#ecosystem-pinia-prefer-store-to-refs-good)

Severidade padrão: `warning`  
Predefinições: `ecosystem`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/pinia-prefer-store-to-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

**Incorreto**

Desestruturar `name` diretamente do store separa o valor de seu acesso reativo ao store.

```vue annotate="remove:2"
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

**Correto**

O store permanece intacto e storeToRefs cria uma referência reativa para name.

```vue annotate="add:2,3"
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) · [Todas as regras](all.md)

### `ecosystem/router-link-require-to`

Exigir um destino `to` nos componentes RouterLink e NuxtLink

[Incorreto](#ecosystem-router-link-require-to-bad) · [Correto](#ecosystem-router-link-require-to-good)

Severidade padrão: `error`  
Predefinições: `ecosystem`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Um link que seja a única raiz de um SFC pode herdar seu target dos atributos do pai. Este exemplo usa um link aninhado, cujo target deve ser explícito.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/router-link-require-to": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-router-link-require-to-bad"></span>

**Incorreto**

O RouterLink aninhado não tem um destino `to`; ele não pode depender da propagação de atributos do elemento raiz.

```vue annotate="remove:2"
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

<span id="ecosystem-router-link-require-to-good"></span>

**Correto**

`to="/settings"` fornece explicitamente o destino do link aninhado.

```vue annotate="add:2"
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) · [Todas as regras](all.md)

### `ecosystem/void-link-require-href`

Exigir `href` nos componentes Link do Void Vue

[Incorreto](#ecosystem-void-link-require-href-bad) · [Correto](#ecosystem-void-link-require-href-good)

Severidade padrão: `error`  
Predefinições: `ecosystem`  
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
        "ecosystem/void-link-require-href": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-require-href-bad"></span>

**Incorreto**

O Link importado de @void/vue omite seu destino href.

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

<span id="ecosystem-void-link-require-href-good"></span>

**Correto**

O mesmo Link importado recebe o destino de configurações por meio de href.

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) · [Todas as regras](all.md)

### `ecosystem/void-link-valid-method`

Validar props method estáticas do Link do Void Vue

[Incorreto](#ecosystem-void-link-valid-method-bad) · [Correto](#ecosystem-void-link-valid-method-good)

Severidade padrão: `warning`  
Predefinições: `ecosystem`  
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
        "ecosystem/void-link-valid-method": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-valid-method-bad"></span>

**Incorreto**

A ação DELETE solicita pré-busca, embora a pré-busca seja destinada a solicitações de navegação.

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

<span id="ecosystem-void-link-valid-method-good"></span>

**Correto**

Remover prefetch mantém a ação DELETE sem fazer a pré-busca dessa solicitação que não é GET.

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14) · [Todas as regras](all.md)

### `ecosystem/vue-i18n-no-missing-key`

Relatar chaves estáticas do vue-i18n ausentes nas mensagens locais do SFC

[Incorreto](#ecosystem-vue-i18n-no-missing-key-bad) · [Correto](#ecosystem-vue-i18n-no-missing-key-good)

Severidade padrão: `warning`  
Predefinições: `ecosystem`  
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
        "ecosystem/vue-i18n-no-missing-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

**Incorreto**

O template solicita auth.missing, mas as mensagens locais em inglês declaram apenas auth.login.

```vue annotate="remove:1"
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

**Correto**

O template solicita a chave auth.login, que existe nas mensagens locais.

```vue annotate="add:1"
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17) · [Todas as regras](all.md)

### `ecosystem/vue-router-prefer-named-link`

Preferir objetos de rotas nomeadas a strings de caminho estáticas no RouterLink

[Incorreto](#ecosystem-vue-router-prefer-named-link-bad) · [Correto](#ecosystem-vue-router-prefer-named-link-good)

Severidade padrão: `warning`  
Predefinições: `ecosystem`  
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
        "ecosystem/vue-router-prefer-named-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

**Incorreto**

O destino do RouterLink é um caminho literal, em vez de uma rota nomeada.

```vue annotate="remove:2"
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

**Correto**

O objeto de rota vinculado identifica o destino pelo nome de rota settings.

```vue annotate="add:2"
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) · [Todas as regras](all.md)

### `ecosystem/vue-router-prefer-named-push`

Preferir objetos de rotas nomeadas para navegação programática com Vue Router

[Incorreto](#ecosystem-vue-router-prefer-named-push-bad) · [Correto](#ecosystem-vue-router-prefer-named-push-good)

Severidade padrão: `warning`  
Predefinições: `ecosystem`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-router-prefer-named-push": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

**Incorreto**

router.push recebe uma string de caminho vinculada à grafia atual da URL.

```vue annotate="remove:2"
<script setup lang="ts">
router.push("/settings");
</script>
```

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

**Correto**

router.push recebe um objeto de rota com o nome estável settings.

```vue annotate="add:2"
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) · [Todas as regras](all.md)

### `ecosystem/vue-test-utils-no-html-snapshot`

Evitar snapshots de wrapper.html() nos testes com Vue Test Utils

[Incorreto](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Correto](#ecosystem-vue-test-utils-no-html-snapshot-good)

Severidade padrão: `warning`  
Predefinições: `ecosystem`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-test-utils-no-html-snapshot": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

**Incorreto**

A asserção registra um snapshot de todo o HTML do wrapper, em vez de verificar o comportamento esperado.

```vue annotate="remove:2"
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

**Correto**

A asserção verifica se o texto renderizado contém Saved.

```vue annotate="add:2"
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) · [Todas as regras](all.md)

### `nuxt/no-nuxt-config-test-key`

Proibir a definição da chave `test` na configuração do Nuxt

[Incorreto](#nuxt-no-nuxt-config-test-key-bad) · [Correto](#nuxt-no-nuxt-config-test-key-good)

Severidade padrão: `error`  
Predefinições: `nuxt`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Arquivos de configuração do Nuxt (nuxt.config.ts)  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

**Incorreto**

A configuração exportada do Nuxt define a chave identificadora `test` como o booleano `true`, a estrutura de configuração obsoleta que esta regra rejeita.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ test: true });
```

<span id="nuxt-no-nuxt-config-test-key-good"></span>

**Correto**

A configuração vazia remove essa propriedade booleana `test`. Este exemplo não proíbe um objeto de configuração de testes.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({});
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) · [Todas as regras](all.md)

### `nuxt/no-page-meta-runtime-values`

Proibir valores do contexto de execução no nível de avaliação imediata de `definePageMeta`, que é extraído para um fragmento separado durante a compilação e executado antes do setup do componente

[Incorreto](#nuxt-no-page-meta-runtime-values-bad) · [Correto](#nuxt-no-page-meta-runtime-values-good)

Severidade padrão: `error`  
Predefinições: `nuxt`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-page-meta-runtime-values": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

**Incorreto**

`useRoute()` é avaliado imediatamente durante a construção do objeto `definePageMeta`, embora a macro eleve esses metadados para fora do contexto de execução de setup.

```vue annotate="remove:2"
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

<span id="nuxt-no-page-meta-runtime-values-good"></span>

**Correto**

`validate` recebe uma função de callback, então o acesso a `useRoute().params.id` é adiado até a execução dessa função. A regra distingue corpos de funções com execução adiada de valores de metadados avaliados imediatamente.

```vue annotate="add:2"
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) · [Todas as regras](all.md)

### `nuxt/nuxt-config-keys-order`

Preferir a ordem recomendada das propriedades de configuração do Nuxt

[Incorreto](#nuxt-nuxt-config-keys-order-bad) · [Correto](#nuxt-nuxt-config-keys-order-good)

Severidade padrão: `error`  
Predefinições: `nuxt`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Arquivos de configuração do Nuxt (nuxt.config.ts)  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/nuxt-config-keys-order": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-nuxt-config-keys-order-bad"></span>

**Incorreto**

A configuração coloca `ssr` antes de `modules`, invertendo sua ordem na sequência de chaves de configuração do Nuxt recomendada pela regra.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ ssr: true, modules: [] });
```

<span id="nuxt-nuxt-config-keys-order-good"></span>

**Correto**

Colocar `modules` antes de `ssr` preserva os dois valores e atende à ordem prescrita; a correção muda a disposição, sem alterar o significado de nenhuma das opções.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({ modules: [], ssr: true });
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) · [Todas as regras](all.md)

### `nuxt/prefer-import-meta`

Preferir `import.meta.*` a `process.*`

[Incorreto](#nuxt-prefer-import-meta-bad) · [Correto](#nuxt-prefer-import-meta-good)

Severidade padrão: `error`  
Predefinições: `nuxt`  
Correção automática: Disponível para os diagnósticos compatíveis  
Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-prefer-import-meta-bad"></span>

**Incorreto**

`process.client` usa um indicador de ambiente legado do Nuxt que a regra pede para migrar para `import.meta`.

```vue annotate="remove:2"
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

<span id="nuxt-prefer-import-meta-good"></span>

**Correto**

`import.meta.client` mantém explícito o ramo exclusivo do navegador usando o indicador de ambiente substituto.

```vue annotate="add:2"
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) · [Todas as regras](all.md)
