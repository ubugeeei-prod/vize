---
title: Regras
description: Ative as regras de lint do Vize com uma predefinição e ajuste regra por regra.
---

<!-- Generated translation; source: rules/index.md -->

# Regras

O linter do Vize verifica componentes Vue de arquivo único: templates, scripts, estilos e as ligações entre arquivos. Escolha uma predefinição e depois aumente, reduza ou desative regras individuais.

## Ativar o lint

Com o Vite+, importe `defineConfig` de `@vizejs/vite-plugin/vite-plus`, configure `vue.linter` e execute `vp run lint`. Esse comando executa as regras do Vize e o Oxlint juntos.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
    linter: {
        preset: "happy-path",
        rules: {
            "vue/no-v-html": "off",
            "a11y/alt-text": "error",
        },
    },
});
```

Cada regra aceita `off`, `warn` ou `error`. Regras com configurações as leem de `vue.linter.ruleOptions`; veja [Opções das regras](/rules/options.md).

Se o projeto já tiver um script chamado `lint`, use `vp run vize:lint`. O comando embutido `vp lint` é o verificador do próprio Vite+ e não executa as regras do Vize.

## Predefinições

Uma predefinição é um conjunto inicial de regras. `happy-path` é a padrão.

| Predefinição | Regras | O que ativa |
| --- | ---: | --- |
| `essential` | 85 | Validade de templates e diretivas, remoções do Vue 3 (`vue/no-deprecated-*`, `script/no-deprecated-*`), correção de macros de script, `nextTick` e Options API, hooks de ciclo de vida, watchers e `expose` registrados depois de `await`, imports de `vue` em vez de `@vue/*`, componentes não usados, `vue/no-v-html`, `vue/no-unsafe-url`, `html/id-duplication` |
| `happy-path` (padrão) | 147 | `essential`, mais convenções de formatação de template (incluindo a forma curta de `v-bind` e eventos de componente com hífen), acessibilidade, conformidade HTML, riscos de SSR, propriedades não usadas, emits declarados e props e emits tipados |
| `ecosystem` | 155 | `happy-path`, mais as verificações das bibliotecas do ecossistema |
| `nuxt` | 213 | `opinionated`, ajustada para os auto-imports do Nuxt |
| `opinionated` | 214 | `happy-path`, mais convenções mais rígidas, props tipadas, `defineProps`/`defineEmits` baseados em tipos, ordem das macros de `<script setup>`, nomes de eventos em kebab-case, aspas consistentes nas chaves de props e emits, `useTemplateRef`, `defineSlots`, padrões de props e emits, `defineOptions` e as regras de estilo `css/*` |
| `incremental` | 0 | Nada. Comece aqui para ativar só as regras que você nomear. |

As contagens não incluem as regras entre arquivos. Quando `compiler.compatibility.vueVersion` é `2` ou `2.7`, as verificações de remoções do Vue 3 ficam desativadas.

## Encontrar uma regra

| Página | O que cobre |
| --- | --- |
| [Todas as regras](./all.md) | Todas as regras em um índice por categoria, com a menor predefinição que ativa cada uma |
| [Vue](./vue.md) | Templates, diretivas, componentes e blocos de SFC, e migração para o Vue 3 |
| [Tipo e script](./type-and-script.md) | `<script setup>`, macros do compilador, reatividade e verificações com tipos |
| [HTML](./html.md) | Validade do HTML e marcação semântica |
| [Acessibilidade](./accessibility.md) | ARIA, interação por teclado, rótulos, landmarks e alternativas de mídia |
| [SSR](./ssr.md) | Riscos de renderização no servidor e hidratação |
| [Vapor](./vapor.md) | Restrições para componentes compilados no modo Vapor |
| [Ecossistema e Nuxt](./ecosystem.md) | Nuxt, Vue Router, Pinia, Vue I18n, Vue Test Utils e Void Vue |
| [CSS](./css.md) | Verificações de correção e manutenção de CSS puro |
| [Musea](./musea.md) | Histórias `.art.vue`, variantes e tokens de design do Musea |
| [petite-vue](./petite-vue.md) | Páginas HTML que usam petite-vue |
| [Entre arquivos](./cross-file.md) | Verificações entre componentes, providers e roteadores, ativadas com `crossFile: true` |
| [Opções das regras](/rules/options.md) | `ruleOptions` tipadas e seus valores padrão |
| [Migração do ESLint](/rules/migration.md) | Nomes de regras do `eslint-plugin-vue` mapeados para o Vize, com as diferenças |

Cada página de regra começa pelo que a regra verifica, sua severidade padrão e as predefinições. Depois mostra um exemplo incorreto, que a regra reporta, e um correto, que ela aceita, cada um com uma frase de motivo, e a configuração que ativa a regra. Os exemplos são os mesmos que a suíte de testes do linter executa.

## Regras com informações de tipo

As regras em `type/` leem informações de tipo do projeto TypeScript em `tsconfig.json`. Além de uma predefinição ou de uma entrada em `rules` que as ative, elas precisam de `typeAware: true` em `vue.linter`. Declare nomes de ambiente compartilhados em `compilerOptions.types` ou com project references, e não em uma lista `globals` separada na configuração do Vize.
