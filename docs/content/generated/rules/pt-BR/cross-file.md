---
title: "Regras entre arquivos"
---

# Regras entre arquivos

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.

As verificações de projeto precisam do grafo completo de componentes analisados. Esta página reúne a finalidade, a configuração, os arquivos compartilhados e os exemplos incorreto e correto completos de cada entrada; use os arquivos compartilhados nos dois exemplos. Os 60 códigos publicados têm limites de suporte distintos: 19 pertencem à etapa da CLI, com 18 pares de código-fonte qualificados e um projeto Vue ilustrativo acompanhado de seu grafo de fluxo reativo; 16 têm produtores experimentais no analisador Rust, mas não são emitidos individualmente por essa etapa; 25 são contratos publicados sem produtor atual de diagnósticos. Os seis IDs de regras específicas de projeto são apresentados separadamente. Ativar um ID não ativa um produtor indisponível. As notas de cada exemplo preservam as limitações de fatos de código-fonte, a qualificação por grafo e as dependências específicas de versão.

A CLI pública expõe a mesma etapa por meio de `vize lint --cross-file`. Os códigos exibidos como `vize:croquis/cf/*` usam `croquis/cf/*` em `lint.vize.rules` (omita `vize:`). Diagnósticos de informação ou sugestão tornam-se avisos da CLI. As localizações relacionadas explicam a relação entre a origem e o consumidor.

<span id="regras-de-arquivo-cruzado"></span>
<span id="direção-de-implementação"></span>

| Regra | Exemplos | Finalidade | Suporte atual |
| --- | --- | --- | --- |
| [`ecosystem/vue-router-extra-param`](#ecosystem-vue-router-extra-param) | [Incorreto](#ecosystem-vue-router-extra-param-bad) · [Correto](#ecosystem-vue-router-extra-param-good) | A rota não declara tab; o Vue Router o descarta. | ID de lint específico do projeto |
| [`ecosystem/vue-router-missing-param`](#ecosystem-vue-router-missing-param) | [Incorreto](#ecosystem-vue-router-missing-param-bad) · [Correto](#ecosystem-vue-router-missing-param-good) | O parâmetro obrigatório postId está ausente; depender da rota atual é frágil. | ID de lint específico do projeto |
| [`ecosystem/vue-router-param-type`](#ecosystem-vue-router-param-type) | [Incorreto](#ecosystem-vue-router-param-type-bad) · [Correto](#ecosystem-vue-router-param-type-good) | postId não é repetível, portanto um array é inválido. | ID de lint específico do projeto |
| [`ecosystem/vue-router-unknown-route`](#ecosystem-vue-router-unknown-route) | [Incorreto](#ecosystem-vue-router-unknown-route-bad) · [Correto](#ecosystem-vue-router-unknown-route-good) | O nome está ausente do roteador completo instalado. | ID de lint específico do projeto |
| [`html/cross-component-nesting`](#html-cross-component-nesting) | [Incorreto](#html-cross-component-nesting-bad) · [Correto](#html-cross-component-nesting-good) | Verificar o aninhamento HTML real após compor os componentes importados. | ID de lint específico do projeto |
| [`vize:croquis/cf/array-mutation`](#vize-croquis-cf-array-mutation) | [Incorreto](#vize-croquis-cf-array-mutation-bad) · [Correto](#vize-croquis-cf-array-mutation-good) | Um array é alterado por índice, o que um array reativo não rastreia. | Contrato; sem produtor atual |
| [`vize:croquis/cf/async-boundary`](#vize-croquis-cf-async-boundary) | [Incorreto](#vize-croquis-cf-async-boundary-bad) · [Correto](#vize-croquis-cf-async-boundary-good) | O estado reativo atravessa um limite assíncrono e pode ser observado desatualizado. | CLI |
| [`vize:croquis/cf/async-no-suspense`](#vize-croquis-cf-async-no-suspense) | [Incorreto](#vize-croquis-cf-async-no-suspense-bad) · [Correto](#vize-croquis-cf-async-no-suspense-good) | Um componente assíncrono é renderizado sem um limite Suspense. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/browser-api-ssr`](#vize-croquis-cf-browser-api-ssr) | [Incorreto](#vize-croquis-cf-browser-api-ssr-bad) · [Correto](#vize-croquis-cf-browser-api-ssr-good) | Uma API exclusiva do navegador é usada onde o componente pode ser renderizado no servidor. | CLI |
| [`vize:croquis/cf/circular-dep`](#vize-croquis-cf-circular-dep) | [Incorreto](#vize-croquis-cf-circular-dep-bad) · [Correto](#vize-croquis-cf-circular-dep-good) | Os componentes importam uns aos outros em um ciclo. | Contrato; sem produtor atual |
| [`vize:croquis/cf/circular-reactive-dependency`](#vize-croquis-cf-circular-reactive-dependency) | [Incorreto](#vize-croquis-cf-circular-reactive-dependency-bad) · [Correto](#vize-croquis-cf-circular-reactive-dependency-good) | Os cálculos reativos dependem uns dos outros em um ciclo. | CLI |
| [`vize:croquis/cf/closure-captures-reactive`](#vize-croquis-cf-closure-captures-reactive) | [Incorreto](#vize-croquis-cf-closure-captures-reactive-bad) · [Correto](#vize-croquis-cf-closure-captures-reactive-good) | Um fechamento captura um valor reativo e não verá atualizações posteriores. | Contrato; sem produtor atual |
| [`vize:croquis/cf/composable-outside-setup`](#vize-croquis-cf-composable-outside-setup) | [Incorreto](#vize-croquis-cf-composable-outside-setup-bad) · [Correto](#vize-croquis-cf-composable-outside-setup-good) | Uma função de composição é chamada fora de `setup`. | Contrato; sem produtor atual |
| [`vize:croquis/cf/computed-side-effects`](#vize-croquis-cf-computed-side-effects) | [Incorreto](#vize-croquis-cf-computed-side-effects-bad) · [Correto](#vize-croquis-cf-computed-side-effects-good) | Uma função de leitura computada escreve no estado ou produz outro efeito colateral. | Contrato; sem produtor atual |
| [`vize:croquis/cf/deep-import`](#vize-croquis-cf-deep-import) | [Incorreto](#vize-croquis-cf-deep-import-bad) · [Correto](#vize-croquis-cf-deep-import-good) | Uma cadeia de importações é mais profunda do que o projeto permite. | Contrato; sem produtor atual |
| [`vize:croquis/cf/destructuring-breaks-reactivity`](#vize-croquis-cf-destructuring-breaks-reactivity) | [Incorreto](#vize-croquis-cf-destructuring-breaks-reactivity-bad) · [Correto](#vize-croquis-cf-destructuring-breaks-reactivity-good) | Desestruturar um objeto reativo copia os campos e perde o rastreamento. | CLI |
| [`vize:croquis/cf/di-outside-setup`](#vize-croquis-cf-di-outside-setup) | [Incorreto](#vize-croquis-cf-di-outside-setup-bad) · [Correto](#vize-croquis-cf-di-outside-setup-good) | `provide` ou `inject` é chamado fora de `setup`. | Contrato; sem produtor atual |
| [`vize:croquis/cf/dom-access-without-next-tick`](#vize-croquis-cf-dom-access-without-next-tick) | [Incorreto](#vize-croquis-cf-dom-access-without-next-tick-bad) · [Correto](#vize-croquis-cf-dom-access-without-next-tick-good) | O DOM é lido antes de o Vue aplicar a atualização. | Contrato; sem produtor atual |
| [`vize:croquis/cf/duplicate-id`](#vize-croquis-cf-duplicate-id) | [Incorreto](#vize-croquis-cf-duplicate-id-bad) · [Correto](#vize-croquis-cf-duplicate-id-good) | O mesmo id de elemento é usado em mais de um componente. | CLI |
| [`vize:croquis/cf/event-listener-leak`](#vize-croquis-cf-event-listener-leak) | [Incorreto](#vize-croquis-cf-event-listener-leak-bad) · [Correto](#vize-croquis-cf-event-listener-leak-good) | Um ouvinte de eventos é registrado e nunca removido. | Contrato; sem produtor atual |
| [`vize:croquis/cf/event-modifier`](#vize-croquis-cf-event-modifier) | [Incorreto](#vize-croquis-cf-event-modifier-bad) · [Correto](#vize-croquis-cf-event-modifier-good) | Um ouvinte de eventos usa um modificador que o evento emitido não suporta. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/hydration-risk`](#vize-croquis-cf-hydration-risk) | [Incorreto](#vize-croquis-cf-hydration-risk-bad) · [Correto](#vize-croquis-cf-hydration-risk-good) | Este código de diagnóstico agrupa várias ocorrências de reatividade, incluindo uma prop copiada para uma ref. Ele não implica que toda expressão Date.now() seja detectada pela passagem de análise entre arquivos. | CLI |
| [`vize:croquis/cf/inherit-attrs-unused`](#vize-croquis-cf-inherit-attrs-unused) | [Incorreto](#vize-croquis-cf-inherit-attrs-unused-bad) · [Correto](#vize-croquis-cf-inherit-attrs-unused-good) | `inheritAttrs: false` está definido e o componente nunca lê os atributos. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/inject-without-symbol`](#vize-croquis-cf-inject-without-symbol) | [Incorreto](#vize-croquis-cf-inject-without-symbol-bad) · [Correto](#vize-croquis-cf-inject-without-symbol-good) | `inject` usa uma chave comum em vez de um símbolo `InjectionKey`. | CLI |
| [`vize:croquis/cf/injected-async-mutation-race`](#vize-croquis-cf-injected-async-mutation-race) | [Incorreto](#vize-croquis-cf-injected-async-mutation-race-bad) · [Correto](#vize-croquis-cf-injected-async-mutation-race-good) | Um valor injetado é alterado por uma tarefa assíncrona sujeita a uma condição de corrida. | CLI |
| [`vize:croquis/cf/lifecycle-outside-setup`](#vize-croquis-cf-lifecycle-outside-setup) | [Incorreto](#vize-croquis-cf-lifecycle-outside-setup-bad) · [Correto](#vize-croquis-cf-lifecycle-outside-setup-good) | Um gancho de ciclo de vida é registrado fora de `setup`. | Contrato; sem produtor atual |
| [`vize:croquis/cf/lifecycle-without-cleanup`](#vize-croquis-cf-lifecycle-without-cleanup) | [Incorreto](#vize-croquis-cf-lifecycle-without-cleanup-bad) · [Correto](#vize-croquis-cf-lifecycle-without-cleanup-good) | Um gancho de ciclo de vida inicia um trabalho e nunca faz sua limpeza. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/missing-required-prop`](#vize-croquis-cf-missing-required-prop) | [Incorreto](#vize-croquis-cf-missing-required-prop-bad) · [Correto](#vize-croquis-cf-missing-required-prop-good) | Uma prop obrigatória não é passada. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/missing-suspense`](#vize-croquis-cf-missing-suspense) | [Incorreto](#vize-croquis-cf-missing-suspense-bad) · [Correto](#vize-croquis-cf-missing-suspense-good) | Uma dependência assíncrona é usada fora de um limite Suspense. | Contrato; sem produtor atual |
| [`vize:croquis/cf/module-scope-reactive`](#vize-croquis-cf-module-scope-reactive) | [Incorreto](#vize-croquis-cf-module-scope-reactive-bad) · [Correto](#vize-croquis-cf-module-scope-reactive-good) | O estado reativo é criado no escopo do módulo e compartilhado por todos os chamadores. | Contrato; sem produtor atual |
| [`vize:croquis/cf/multi-root-attrs`](#vize-croquis-cf-multi-root-attrs) | [Incorreto](#vize-croquis-cf-multi-root-attrs-bad) · [Correto](#vize-croquis-cf-multi-root-attrs-good) | Um componente com múltiplas raízes recebe atributos e não tem onde colocá-los. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/mutated-after-escape`](#vize-croquis-cf-mutated-after-escape) | [Incorreto](#vize-croquis-cf-mutated-after-escape-bad) · [Correto](#vize-croquis-cf-mutated-after-escape-good) | Um objeto reativo é alterado depois de escapar de seu proprietário. | Contrato; sem produtor atual |
| [`vize:croquis/cf/non-reactive-provide`](#vize-croquis-cf-non-reactive-provide) | [Incorreto](#vize-croquis-cf-non-reactive-provide-bad) · [Correto](#vize-croquis-cf-non-reactive-provide-good) | Um valor fornecido não é reativo, portanto os descendentes não verão atualizações. | CLI |
| [`vize:croquis/cf/non-unique-id`](#vize-croquis-cf-non-unique-id) | [Incorreto](#vize-croquis-cf-non-unique-id-bad) · [Correto](#vize-croquis-cf-non-unique-id-good) | Um id de elemento dentro de um laço não é único por item. | CLI |
| [`vize:croquis/cf/object-identity-comparison`](#vize-croquis-cf-object-identity-comparison) | [Incorreto](#vize-croquis-cf-object-identity-comparison-bad) · [Correto](#vize-croquis-cf-object-identity-comparison-good) | Um objeto reativo é comparado por identidade, que muda ao remover os invólucros. | Contrato; sem produtor atual |
| [`vize:croquis/cf/pinia-getter`](#vize-croquis-cf-pinia-getter) | [Incorreto](#vize-croquis-cf-pinia-getter-bad) · [Correto](#vize-croquis-cf-pinia-getter-good) | Uma função de leitura do Pinia é lida sem `storeToRefs`, portanto não permanecerá reativa. | Contrato; sem produtor atual |
| [`vize:croquis/cf/prop-type-mismatch`](#vize-croquis-cf-prop-type-mismatch) | [Incorreto](#vize-croquis-cf-prop-type-mismatch-bad) · [Correto](#vize-croquis-cf-prop-type-mismatch-good) | O valor de uma prop passada não corresponde ao tipo declarado. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/provide-inject-type`](#vize-croquis-cf-provide-inject-type) | [Incorreto](#vize-croquis-cf-provide-inject-type-bad) · [Correto](#vize-croquis-cf-provide-inject-type-good) | Um valor fornecido e sua injeção não têm o mesmo tipo. | CLI |
| [`vize:croquis/cf/provide-without-symbol`](#vize-croquis-cf-provide-without-symbol) | [Incorreto](#vize-croquis-cf-provide-without-symbol-bad) · [Correto](#vize-croquis-cf-provide-without-symbol-good) | `provide` usa uma chave comum em vez de um símbolo `InjectionKey`. | CLI |
| [`vize:croquis/cf/reactive-export`](#vize-croquis-cf-reactive-export) | [Incorreto](#vize-croquis-cf-reactive-export-bad) · [Correto](#vize-croquis-cf-reactive-export-good) | O estado reativo é exportado pelo módulo. | Contrato; sem produtor atual |
| [`vize:croquis/cf/reactivity-outside-setup`](#vize-croquis-cf-reactivity-outside-setup) | [Incorreto](#vize-croquis-cf-reactivity-outside-setup-bad) · [Correto](#vize-croquis-cf-reactivity-outside-setup-good) | Uma API reativa é chamada fora de `setup`. | Contrato; sem produtor atual |
| [`vize:croquis/cf/reassignment-breaks-reactivity`](#vize-croquis-cf-reassignment-breaks-reactivity) | [Incorreto](#vize-croquis-cf-reassignment-breaks-reactivity-bad) · [Correto](#vize-croquis-cf-reassignment-breaks-reactivity-good) | Reatribuir uma variável reativa a substitui por um valor simples. | CLI |
| [`vize:croquis/cf/reference-escapes-scope`](#vize-croquis-cf-reference-escapes-scope) | [Incorreto](#vize-croquis-cf-reference-escapes-scope-bad) · [Correto](#vize-croquis-cf-reference-escapes-scope-good) | Uma referência reativa escapa do escopo responsável por seu ciclo de vida. | Contrato; sem produtor atual |
| [`vize:croquis/cf/setup-context-violation`](#vize-croquis-cf-setup-context-violation) | [Incorreto](#vize-croquis-cf-setup-context-violation-bad) · [Correto](#vize-croquis-cf-setup-context-violation-good) | O contexto de setup é usado de uma forma que o Vue não permite. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/shallow-deep-access`](#vize-croquis-cf-shallow-deep-access) | [Incorreto](#vize-croquis-cf-shallow-deep-access-bad) · [Correto](#vize-croquis-cf-shallow-deep-access-good) | Uma propriedade profunda de um valor `shallowReactive` ou `shallowRef` é lida como se fosse rastreada. | Contrato; sem produtor atual |
| [`vize:croquis/cf/spread-breaks-reactivity`](#vize-croquis-cf-spread-breaks-reactivity) | [Incorreto](#vize-croquis-cf-spread-breaks-reactivity-bad) · [Correto](#vize-croquis-cf-spread-breaks-reactivity-good) | Espalhar um objeto reativo copia seus valores e perde o rastreamento. | CLI |
| [`vize:croquis/cf/suspense-no-fallback`](#vize-croquis-cf-suspense-no-fallback) | [Incorreto](#vize-croquis-cf-suspense-no-fallback-bad) · [Correto](#vize-croquis-cf-suspense-no-fallback-good) | `<Suspense>` não tem conteúdo alternativo. | Contrato; sem produtor atual |
| [`vize:croquis/cf/template-ref-timing`](#vize-croquis-cf-template-ref-timing) | [Incorreto](#vize-croquis-cf-template-ref-timing-bad) · [Correto](#vize-croquis-cf-template-ref-timing-good) | Uma ref de template é lida antes da montagem do componente. | Contrato; sem produtor atual |
| [`vize:croquis/cf/toraw-mutation`](#vize-croquis-cf-toraw-mutation) | [Incorreto](#vize-croquis-cf-toraw-mutation-bad) · [Correto](#vize-croquis-cf-toraw-mutation-good) | `toRaw` é usado e o objeto bruto é alterado em seguida. | Contrato; sem produtor atual |
| [`vize:croquis/cf/uncaught-error`](#vize-croquis-cf-uncaught-error) | [Incorreto](#vize-croquis-cf-uncaught-error-bad) · [Correto](#vize-croquis-cf-uncaught-error-good) | Um componente pode lançar um erro e nenhum limite de erro o captura. | CLI |
| [`vize:croquis/cf/undeclared-emit`](#vize-croquis-cf-undeclared-emit) | [Incorreto](#vize-croquis-cf-undeclared-emit-bad) · [Correto](#vize-croquis-cf-undeclared-emit-good) | O componente emite um evento que não foi declarado. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/undeclared-prop`](#vize-croquis-cf-undeclared-prop) | [Incorreto](#vize-croquis-cf-undeclared-prop-bad) · [Correto](#vize-croquis-cf-undeclared-prop-good) | Um pai passa uma prop que o filho não declara. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/undefined-slot`](#vize-croquis-cf-undefined-slot) | [Incorreto](#vize-croquis-cf-undefined-slot-bad) · [Correto](#vize-croquis-cf-undefined-slot-good) | Um pai preenche um slot que o filho não expõe. | Contrato; sem produtor atual |
| [`vize:croquis/cf/unhandled-event`](#vize-croquis-cf-unhandled-event) | [Incorreto](#vize-croquis-cf-unhandled-event-bad) · [Correto](#vize-croquis-cf-unhandled-event-good) | Um filho emite um evento que nenhum pai trata. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/unmatched-inject`](#vize-croquis-cf-unmatched-inject) | [Incorreto](#vize-croquis-cf-unmatched-inject-bad) · [Correto](#vize-croquis-cf-unmatched-inject-good) | `inject` nomeia uma chave que nenhum ancestral fornece. | CLI |
| [`vize:croquis/cf/unmatched-listener`](#vize-croquis-cf-unmatched-listener) | [Incorreto](#vize-croquis-cf-unmatched-listener-bad) · [Correto](#vize-croquis-cf-unmatched-listener-good) | Um pai escuta um evento que o filho não emite. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/unregistered-component`](#vize-croquis-cf-unregistered-component) | [Incorreto](#vize-croquis-cf-unregistered-component-bad) · [Correto](#vize-croquis-cf-unregistered-component-good) | Um template usa um componente que não foi registrado nem importado. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/unresolved-import`](#vize-croquis-cf-unresolved-import) | [Incorreto](#vize-croquis-cf-unresolved-import-bad) · [Correto](#vize-croquis-cf-unresolved-import-good) | Uma importação não resolve para um módulo. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/unused-attrs`](#vize-croquis-cf-unused-attrs) | [Incorreto](#vize-croquis-cf-unused-attrs-bad) · [Correto](#vize-croquis-cf-unused-attrs-good) | Atributos de herança automática são passados para um componente com múltiplas raízes que não os usa. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/unused-emit`](#vize-croquis-cf-unused-emit) | [Incorreto](#vize-croquis-cf-unused-emit-bad) · [Correto](#vize-croquis-cf-unused-emit-good) | Um evento declarado nunca é emitido. | Analisador Rust experimental; não é um código individual da CLI |
| [`vize:croquis/cf/unused-provide`](#vize-croquis-cf-unused-provide) | [Incorreto](#vize-croquis-cf-unused-provide-bad) · [Correto](#vize-croquis-cf-unused-provide-good) | Uma chave fornecida nunca é injetada. | CLI |
| [`vize:croquis/cf/value-extraction-breaks-reactivity`](#vize-croquis-cf-value-extraction-breaks-reactivity) | [Incorreto](#vize-croquis-cf-value-extraction-breaks-reactivity-bad) · [Correto](#vize-croquis-cf-value-extraction-breaks-reactivity-good) | Ler um valor reativo para uma variável local perde as atualizações posteriores. | CLI |
| [`vize:croquis/cf/watch-can-be-computed`](#vize-croquis-cf-watch-can-be-computed) | [Incorreto](#vize-croquis-cf-watch-can-be-computed-bad) · [Correto](#vize-croquis-cf-watch-can-be-computed-good) | Um observador apenas copia um valor para o estado e pode ser um valor computado. | Contrato; sem produtor atual |
| [`vize:croquis/cf/watcheffect-async`](#vize-croquis-cf-watcheffect-async) | [Incorreto](#vize-croquis-cf-watcheffect-async-bad) · [Correto](#vize-croquis-cf-watcheffect-async-good) | `watchEffect` inicia uma tarefa assíncrona e não consegue limpar a execução anterior. | CLI |
| [`vize:croquis/cf/watcher-outside-setup`](#vize-croquis-cf-watcher-outside-setup) | [Incorreto](#vize-croquis-cf-watcher-outside-setup-bad) · [Correto](#vize-croquis-cf-watcher-outside-setup-good) | `watch` ou `watchEffect` é chamado fora de `setup`. | Contrato; sem produtor atual |
| [`vue/cross-file-attrs-fallthrough`](#vue-cross-file-attrs-fallthrough) | [Incorreto](#vue-cross-file-attrs-fallthrough-bad) · [Correto](#vue-cross-file-attrs-fallthrough-good) | Um pai passa atributos para um filho resolvido cuja raiz não pode herdá-los e que não usa $attrs explicitamente. | ID de lint específico do projeto |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `ecosystem/vue-router-extra-param`

A rota não declara tab; o Vue Router o descarta.

Severidade padrão: error  
Aplicável a: Declarações alcançáveis do projeto e componentes importados  
Opções: crossFile; severidade da regra (off/warn/error)  
Correção automática: Nenhuma

O roteador completo instalado deve ser alcançável a partir de createApp(...).use(router) da aplicação. Tabelas de rotas desconhecidas ou dinâmicas não comprovam diagnósticos de nomes desconhecidos. Parâmetros ausentes geram avisos porque a navegação pode herdar um valor da rota atual.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-extra-param": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-extra-param-bad"></span>

**Incorreto**

O caminho `user-post` declara `userId` e `postId`, mas a navegação também fornece `tab`, que não foi declarado, como parâmetro do caminho.

`src/UserPost.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2", tab: "a" } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-extra-param-good"></span>

**Correto**

Remova `tab` de params e mantenha apenas as chaves presentes no caminho da rota. Use query separadamente se a aplicação precisar selecionar uma aba.

`src/UserPost.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Índice de verificações entre arquivos](cross-file.md)

### `ecosystem/vue-router-missing-param`

O parâmetro obrigatório postId está ausente; depender da rota atual é frágil.

Severidade padrão: warning  
Aplicável a: Declarações alcançáveis do projeto e componentes importados  
Opções: crossFile; severidade da regra (off/warn/error)  
Correção automática: Nenhuma

O roteador completo instalado deve ser alcançável a partir de createApp(...).use(router) da aplicação. Tabelas de rotas desconhecidas ou dinâmicas não comprovam diagnósticos de nomes desconhecidos. Parâmetros ausentes geram avisos porque a navegação pode herdar um valor da rota atual.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-missing-param": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-missing-param-bad"></span>

**Incorreto**

A navegação omite o parâmetro obrigatório `postId` do caminho `user-post`. Isso é um aviso porque o Vue Router pode herdar um valor da rota atual.

`src/UserPost.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1" } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-missing-param-good"></span>

**Correto**

Passe `userId` e `postId` explicitamente para que a navegação não dependa do estado dos parâmetros da rota atual.

`src/UserPost.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Índice de verificações entre arquivos](cross-file.md)

### `ecosystem/vue-router-param-type`

postId não é repetível, portanto um array é inválido.

Severidade padrão: error  
Aplicável a: Declarações alcançáveis do projeto e componentes importados  
Opções: crossFile; severidade da regra (off/warn/error)  
Correção automática: Nenhuma

O roteador completo instalado deve ser alcançável a partir de createApp(...).use(router) da aplicação. Tabelas de rotas desconhecidas ou dinâmicas não comprovam diagnósticos de nomes desconhecidos. Parâmetros ausentes geram avisos porque a navegação pode herdar um valor da rota atual.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-param-type": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-param-type-bad"></span>

**Incorreto**

`postId` é um parâmetro escalar do caminho, mas a navegação fornece a ele o array `["2"]`.

`src/UserPost.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: ["2"] } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-param-type-good"></span>

**Correto**

Passe o escalar `"2"` para o segmento `postId`, que não é repetível.

`src/UserPost.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Índice de verificações entre arquivos](cross-file.md)

### `ecosystem/vue-router-unknown-route`

O nome está ausente do roteador completo instalado.

Severidade padrão: error  
Aplicável a: Declarações alcançáveis do projeto e componentes importados  
Opções: crossFile; severidade da regra (off/warn/error)  
Correção automática: Nenhuma

O roteador completo instalado deve ser alcançável a partir de createApp(...).use(router) da aplicação. Tabelas de rotas desconhecidas ou dinâmicas não comprovam diagnósticos de nomes desconhecidos. Parâmetros ausentes geram avisos porque a navegação pode herdar um valor da rota atual.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-unknown-route": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-unknown-route-bad"></span>

**Incorreto**

O roteador instalado alcançável declara `user-post`, mas a navegação usa o nome incorreto `user-posts`.

`src/UserPost.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-posts", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-unknown-route-good"></span>

**Correto**

Use o nome registrado `user-post`, mantendo os dois parâmetros de caminho declarados.

`src/UserPost.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Índice de verificações entre arquivos](cross-file.md)

### `html/cross-component-nesting`

Verificar o aninhamento HTML real após compor os componentes importados.

Severidade padrão: warning  
Aplicável a: Declarações alcançáveis do projeto e componentes importados  
Opções: crossFile; severidade da regra (off/warn/error)  
Correção automática: Nenhuma

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "html/cross-component-nesting": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="html-cross-component-nesting-bad"></span>

**Incorreto**

O `<p>` do pai contém um filho resolvido cuja raiz é `<div>`, produzindo um aninhamento inválido de bloco em parágrafo após a composição.

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><p><Child /></p></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

<span id="html-cross-component-nesting-good"></span>

**Correto**

Use um contêiner `<section>` que possa conter o elemento de bloco do filho; o filho permanece inalterado.

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><section><Child /></section></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/array-mutation`

Um array é alterado por índice, o que um array reativo não rastreia.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Apenas para o Vue 2.7 histórico: use dependências compatíveis do Vue 2.7 e do compilador de SFCs neste cenário. Os proxies do Vue 3 rastreiam atribuições a índices de arrays, então `items[0] = next` é reativo no Vue 3 e não é um defeito do Vue 3. Este código publicado não tem produtor atual.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import Vue from 'vue';
import App from './App.vue';
new Vue({ render: h => h(App) }).$mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script lang="ts">
import Vue from 'vue';
import { replaceFirst } from './replace-first';
export default Vue.extend({
  data() { return { items: ['Before'] }; },
  methods: { replace() { replaceFirst(this.items, 'After'); } },
});
</script>
<template><section><p>{{ items[0] }}</p><button @click="replace">Replace</button></section></template>

```

<span id="vize-croquis-cf-array-mutation-bad"></span>

**Incorreto**

Neste projeto histórico com Vue 2.7, `items[0] = next` altera o array sem notificar o observador de arrays do Vue 2, de modo que o primeiro item exibido pode não ser atualizado.

`replace-first.ts`

```ts annotate="remove:2"
export function replaceFirst(items: string[], next: string): void {
  items[0] = next;
}

```

<span id="vize-croquis-cf-array-mutation-good"></span>

**Correto**

`splice(0, 1, next)` usa o método de mutação de arrays observado pelo Vue 2, permitindo que a mesma substituição atualize a visualização.

`replace-first.ts`

```ts annotate="add:2"
export function replaceFirst(items: string[], next: string): void {
  items.splice(0, 1, next);
}

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/async-boundary`

O estado reativo atravessa um limite assíncrono e pode ser observado desatualizado.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/async-boundary": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./SearchPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export interface Result { items: string[]; }
export async function load(query: string, options?: { signal?: AbortSignal }): Promise<Result> {
  const response = await fetch(`/search?q=${encodeURIComponent(query)}`, options);
  return response.json();
}
```

<span id="vize-croquis-cf-async-boundary-bad"></span>

**Incorreto**

Uma consulta antiga mais lenta pode terminar após uma consulta mais recente e sobrescrever `result`, porque o observador não tem limpeza na invalidação.

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue annotate="remove:10,11"
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value) => {
    result.value = await load(value);
  },
);
</script>
```

<span id="vize-croquis-cf-async-boundary-good"></span>

**Correto**

Registre a limpeza antes de aguardar: aborte a requisição antiga e invalide seu sinalizador `active`; depois, atribua apenas uma resposta que ainda esteja ativa.

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue annotate="add:10,11,12,13,14,15,16,17,18,19,20"
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/async-no-suspense`

Um componente assíncrono é renderizado sem um limite Suspense.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

Suporte atual: `no-source-async-fact`

O produtor de diagnósticos de limites lê macros.is_async(), mas a análise do código-fonte atualmente registra o await de nível superior no escopo de script-setup. Portanto, o par completo de código-fonte incorreto e correto abaixo não produz um diagnóstico async-no-suspense pela CLI atual. Ele explica a convenção de Suspense; fornecer o fato ausente da macro é um trabalho posterior de implementação.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-async-no-suspense-bad"></span>

**Incorreto**

O filho tem await no nível superior, mas o pai não fornece um limite `<Suspense>`. A análise atual do código-fonte não fornece o fato de macro necessário para emitir este código de diagnóstico.

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

<span id="vize-croquis-cf-async-no-suspense-good"></span>

**Correto**

O pai envolve o mesmo filho assíncrono em `<Suspense>` com conteúdo alternativo de carregamento. Isso demonstra a convenção; a passagem atual continua sem emitir o diagnóstico para nenhuma das duas alternativas de código-fonte.

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Suspense><Child /><template #fallback><p>Loading</p></template></Suspense></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/browser-api-ssr`

Uma API exclusiva do navegador é usada onde o componente pode ser renderizado no servidor.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/browser-api-ssr": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-browser-api-ssr-bad"></span>

**Incorreto**

`window.innerWidth` é executado durante setup, quando um ambiente SSR não tem o `window` do navegador.

`App.vue`

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-browser-api-ssr-good"></span>

**Correto**

Inicialize uma ref com um valor seguro para o servidor e leia `window` dentro de `onMounted`, que é executado após a montagem no cliente.

`App.vue`

```vue annotate="add:2,3,4"
<script setup lang="ts">
import { onMounted, ref } from "vue";
const width = ref(0);
onMounted(() => { width.value = window.innerWidth; });
</script>
<template><p>Content</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/circular-dep`

Os componentes importam uns aos outros em um ciclo.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Isto ilustra um ciclo concreto de inicialização imediata. Um componente Vue recursivo ou qualquer importação circular não é automaticamente incorreto. Nenhum produtor atual emite este código de contrato.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { aLabel } from './a';
</script>

<template>
<p>{{ aLabel }}</p>
</template>

```

`labels.ts`

```ts
export const aPrefix = 'A';
export const bPrefix = 'B';

```

<span id="vize-croquis-cf-circular-dep-bad"></span>

**Incorreto**

`a.ts` importa `b.ts`, que importa `a.ts` de volta. Ambos inicializam imediatamente uma constante a partir da constante ainda não inicializada do outro módulo, causando uma falha de zona morta temporal.

`a.ts`

```ts annotate="remove:1,2"
import { bLabel } from './b';
export const aLabel = 'A' + bLabel;

```

`b.ts`

```ts annotate="remove:1,2"
import { aLabel } from './a';
export const bLabel = 'B' + aLabel;

```

<span id="vize-croquis-cf-circular-dep-good"></span>

**Correto**

Os dois módulos leem prefixos inicializados do módulo independente `labels.ts`, removendo o ciclo e a leitura cruzada durante a inicialização imediata.

`a.ts`

```ts annotate="add:1,2"
import { bPrefix } from './labels';
export const aLabel = 'A' + bPrefix;

```

`b.ts`

```ts annotate="add:1,2"
import { aPrefix } from './labels';
export const bLabel = 'B' + aPrefix;

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/circular-reactive-dependency`

Os cálculos reativos dependem uns dos outros em um ciclo.

Severidade padrão: dependente do contexto  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/circular-reactive-dependency": "warn" },
    },
  },
});
```

```sh
vp run lint
```

Qualificação do exemplo: `illustrative-source-pair`

O projeto Vue completo abaixo ilustra a realimentação de atualizações e sua correção. Ele não é uma comprovação qualificada de diagnóstico da CLI: o produtor de diagnósticos exige identidades de referências e arestas de fluxo reativo retidas, como mostra o grafo que acompanha o exemplo. Esses arquivos-fonte não demonstram que o processamento atual do código-fonte emitirá este código exato. Os controles específicos de diagnóstico de grafos com IDs rastreados permanecem separados das verificações de gramática do código-fonte.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`count-key.ts`

```ts
import type { InjectionKey, Ref } from 'vue';
export const countKey: InjectionKey<Ref<number>> = Symbol('count');
```

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from 'vue';
import { countKey } from './count-key';
import CycleView from './CycleView.vue';
const count = ref(1); // A: the provider-owned source.
provide(countKey, count);
</script>
<template>
  <button @click="count++">Increment</button>
  <CycleView />
</template>
```

<span id="vize-croquis-cf-circular-reactive-dependency-bad"></span>

**Incorreto**

App possui e fornece count (A). CycleView deriva nextCount (B) e imediatamente escreve cada valor derivado de volta no mesmo count injetado. Cada escrita altera novamente a entrada do cálculo, criando um ciclo de realimentação de atualizações A → B → A. As identidades no grafo preservado abaixo representam essas duas referências, não variáveis sem relação que tenham nomes iguais.

`CycleView.vue`

```vue annotate="remove:2,6"
<script setup lang="ts">
import { computed, inject, watch } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
watch(nextCount, value => { count.value = value; }, { immediate: true });
</script>
<template><p>{{ nextCount }}</p></template>
```

```text annotate="remove:2"
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B; B -> A
```

<span id="vize-croquis-cf-circular-reactive-dependency-good"></span>

**Correto**

Remova o observador que escreve B de volta em A. App mantém a propriedade de count e o altera apenas por meio de sua ação explícita Increment; CycleView lê o nextCount derivado sem realimentar o resultado. As mesmas referências preservam apenas a dependência A → B.

`CycleView.vue`

```vue annotate="add:2"
<script setup lang="ts">
import { computed, inject } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
</script>
<template><p>{{ nextCount }}</p></template>
```

```text annotate="add:2"
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/closure-captures-reactive`

Um fechamento captura um valor reativo e não verá atualizações posteriores.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { computed, ref } from 'vue';
import { makeReader } from './reader';
const count = ref(0);
const read = makeReader(count);
const shown = computed(read);
</script>

<template>
<button @click="count++">Increment {{ count }}</button><p>{{ shown }}</p>
</template>

```

<span id="vize-croquis-cf-closure-captures-reactive-bad"></span>

**Incorreto**

`makeReader` copia `count.value` antes de criar o fechamento. O leitor computado retorna então esse número inicial sem ler uma dependência reativa.

`reader.ts`

```ts annotate="remove:3,4"
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  const captured = count.value;
  return () => captured;
}

```

<span id="vize-croquis-cf-closure-captures-reactive-good"></span>

**Correto**

O fechamento lê `count.value` quando é chamado, de modo que a função de leitura computada pode rastrear a ref e atualizar `shown` após os incrementos.

`reader.ts`

```ts annotate="add:3"
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  return () => count.value;
}

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/composable-outside-setup`

Uma função de composição é chamada fora de `setup`.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

A preocupação é este composable dependente do ciclo de vida, e não uma proibição geral de funções utilitárias comuns ou de todas as chamadas da Composition API fora de setup. Este contrato não tem produtor atual.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useTitle } from './use-title';
const title = useTitle();
</script>

<template>
<h1>{{ title }}</h1>
</template>

```

<span id="vize-croquis-cf-composable-outside-setup-bad"></span>

**Incorreto**

Importar `use-title.ts` registra `onMounted` antes que o setup de um componente esteja ativo. Chamar sua função exportada depois apenas retorna essa ref no escopo do módulo; isso não corrige a falta de vínculo com o ciclo de vida.

`use-title.ts`

```ts annotate="remove:2,3,4"
import { onMounted, ref } from 'vue';
const title = ref('Before mount');
onMounted(() => { title.value = 'Mounted'; });
export function useTitle() { return title; }

```

<span id="vize-croquis-cf-composable-outside-setup-good"></span>

**Correto**

Tanto a criação do estado quanto o registro do gancho passam para `useTitle`, que App chama de forma síncrona dentro de setup. O gancho de montagem agora pertence a essa instância de App.

`use-title.ts`

```ts annotate="add:2,3,4,5,6"
import { onMounted, ref } from 'vue';
export function useTitle() {
  const title = ref('Before mount');
  onMounted(() => { title.value = 'Mounted'; });
  return title;
}

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/computed-side-effects`

Uma função de leitura computada escreve no estado ou produz outro efeito colateral.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useDouble } from './use-double';
const { count, doubled, lastCalculated } = useDouble();
</script>

<template>
<button @click="count++">Increment {{ count }}</button><p>{{ doubled }} / {{ lastCalculated }}</p>
</template>

```

<span id="vize-croquis-cf-computed-side-effects-bad"></span>

**Incorreto**

Avaliar `doubled` escreve em `lastCalculated`, de modo que ler um valor computado também altera um estado separado. Isso vincula o efeito colateral ao momento em que a função de leitura de avaliação adiada é lida.

`use-double.ts`

```ts annotate="remove:1,5,6,7,8,9"
import { computed, ref } from 'vue';
export function useDouble() {
  const count = ref(0);
  const lastCalculated = ref(0);
  const doubled = computed(() => {
    const next = count.value * 2;
    lastCalculated.value = next;
    return next;
  });
  return { count, doubled, lastCalculated };
}

```

<span id="vize-croquis-cf-computed-side-effects-good"></span>

**Correto**

A função de leitura apenas retorna o número derivado. Um observador separado fica responsável pela escrita em `lastCalculated` quando `count` muda, incluindo seu valor inicial.

`use-double.ts`

```ts annotate="add:1,5,6"
import { computed, ref, watch } from 'vue';
export function useDouble() {
  const count = ref(0);
  const lastCalculated = ref(0);
  const doubled = computed(() => count.value * 2);
  watch(count, next => { lastCalculated.value = next * 2; }, { immediate: true });
  return { count, doubled, lastCalculated };
}

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/deep-import`

Uma cadeia de importações é mais profunda do que o projeto permite.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Esta é uma política de organização de projeto escolhida explicitamente; não cria um limite de profundidade ou uma opção suportados. Não há um produtor atual de diagnósticos para este contrato.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { label } from './entry';
</script>

<template>
<p>{{ label }}</p>
</template>

```

`value.ts`

```ts
export const label = 'Notice';

```

`level-one.ts`

```ts
export { label } from './level-two';

```

`level-two.ts`

```ts
export { label } from './level-three';

```

`level-three.ts`

```ts
export { label } from './value';

```

`public-api.ts`

```ts
export { label } from './value';

```

<span id="vize-croquis-cf-deep-import-bad"></span>

**Incorreto**

O ponto de entrada encaminha um valor simples por `level-one`, `level-two` e `level-three`, criando uma cadeia de importações desnecessariamente profunda para um projeto que deseja um limite público raso.

`entry.ts`

```ts annotate="remove:1"
export { label } from './level-one';

```

<span id="vize-croquis-cf-deep-import-good"></span>

**Correto**

O ponto de entrada usa `public-api.ts`, que reexporta o valor diretamente. O consumidor mantém o mesmo nome importado, enquanto a cadeia fica mais curta.

`entry.ts`

```ts annotate="add:1"
export { label } from './public-api';

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/destructuring-breaks-reactivity`

Desestruturar um objeto reativo copia os campos e perde o rastreamento.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/destructuring-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-destructuring-breaks-reactivity-bad"></span>

**Incorreto**

A desestruturação comum do objeto `props` copia seu valor atual de `item`; isso é diferente da desestruturação direta de `defineProps()` no Vue 3.5.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue annotate="remove:3"
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const { item } = props;
</script>
```

<span id="vize-croquis-cf-destructuring-breaks-reactivity-good"></span>

**Correto**

`toRef(props, "item")` preserva a conexão com a propriedade em `props`.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue annotate="add:2,3,5"
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/di-outside-setup`

`provide` ou `inject` é chamado fora de `setup`.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Este exemplo usa provide/inject de componente. A injeção por `app.provide` e pelo `app.runWithContext` suportado são outras formas válidas de associação a um responsável, não proibidas por este cenário. Nenhum produtor atual emite este código de contrato.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`theme.ts`

```ts
import { inject, provide } from 'vue';
import type { InjectionKey } from 'vue';
export const ThemeKey: InjectionKey<string> = Symbol('theme');
export function provideTheme() { provide(ThemeKey, 'dark'); }
export function useTheme() { return inject(ThemeKey, 'light'); }

```

`ThemedText.vue`

```vue
<script setup lang="ts">
import { useTheme } from './theme';
const theme = useTheme();
</script>

<template>
<p>{{ theme }}</p>
</template>

```

<span id="vize-croquis-cf-di-outside-setup-bad"></span>

**Incorreto**

`main.ts` chama o `provide` de componente sem uma instância de componente ativa. Por isso, o `inject` do filho não pode receber o valor pretendido do ancestral e usa `light`.

`main.ts`

```ts annotate="remove:1,2,3,4,5,6"
import { createApp } from 'vue';
import App from './App.vue';
import { provideTheme } from './theme';
provideTheme();
createApp(App).mount('#app');

```

`App.vue`

```vue
<script setup lang="ts">
import ThemedText from './ThemedText.vue';
</script>

<template>
<ThemedText />
</template>

```

<span id="vize-croquis-cf-di-outside-setup-good"></span>

**Correto**

App chama o provedor em seu setup antes de renderizar o filho. O filho agora herda o valor `dark` de seu componente ancestral.

`App.vue`

```vue annotate="add:3,4"
<script setup lang="ts">
import ThemedText from './ThemedText.vue';
import { provideTheme } from './theme';
provideTheme();
</script>

<template>
<ThemedText />
</template>

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/dom-access-without-next-tick`

O DOM é lido antes de o Vue aplicar a atualização.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`read-label.ts`

```ts
export function readLabel(node: HTMLElement | null): string {
  return node?.textContent ?? '';
}

```

<span id="vize-croquis-cf-dom-access-without-next-tick-bad"></span>

**Incorreto**

O manipulador de clique incrementa `count` e imediatamente lê o parágrafo renderizado, antes de o Vue aplicar a atualização agendada do DOM. `sampled` pode conter a contagem anterior.

`App.vue`

```vue annotate="remove:2,7"
<script setup lang="ts">
import { ref } from 'vue';
import { readLabel } from './read-label';
const count = ref(0);
const label = ref<HTMLElement | null>(null);
const sampled = ref('');
function increment() {
  count.value++;
  sampled.value = readLabel(label.value);
}
</script>

<template>
<button @click="increment">Increment</button><p ref="label">{{ count }}</p><p>DOM sample: {{ sampled }}</p>
</template>

```

<span id="vize-croquis-cf-dom-access-without-next-tick-good"></span>

**Correto**

Aguardar `nextTick()` após a escrita no estado permite que o Vue atualize o parágrafo antes de `readLabel` capturar seu texto.

`App.vue`

```vue annotate="add:2,7,9"
<script setup lang="ts">
import { nextTick, ref } from 'vue';
import { readLabel } from './read-label';
const count = ref(0);
const label = ref<HTMLElement | null>(null);
const sampled = ref('');
async function increment() {
  count.value++;
  await nextTick();
  sampled.value = readLabel(label.value);
}
</script>

<template>
<button @click="increment">Increment</button><p ref="label">{{ count }}</p><p>DOM sample: {{ sampled }}</p>
</template>

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/duplicate-id`

O mesmo id de elemento é usado em mais de um componente.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/duplicate-id": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./CheckoutForm.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-duplicate-id-bad"></span>

**Incorreto**

Os componentes alcançáveis de entrega e cobrança renderizam `id="postal-code"`, de modo que seus rótulos compartilham um destino ambíguo no documento.

`CheckoutForm.vue`

```vue
<script setup lang="ts">
import BillingAddress from "./BillingAddress.vue";
import ShippingAddress from "./ShippingAddress.vue";
</script>

<template>
  <ShippingAddress />
  <BillingAddress />
</template>
```

`ShippingAddress.vue`

```vue annotate="remove:2,3"
<template>
  <label for="postal-code">Shipping postal code</label>
  <input id="postal-code" />
</template>
```

`BillingAddress.vue`

```vue annotate="remove:2,3"
<template>
  <label for="postal-code">Billing postal code</label>
  <input id="postal-code" />
</template>
```

<span id="vize-croquis-cf-duplicate-id-good"></span>

**Correto**

Cada componente chama `useId()` e vincula seu próprio valor ao rótulo e ao campo de entrada, preservando a associação sem repetir um ID literal.

`CheckoutForm.vue`

```vue
<script setup lang="ts">
import BillingAddress from "./BillingAddress.vue";
import ShippingAddress from "./ShippingAddress.vue";
</script>

<template>
  <ShippingAddress />
  <BillingAddress />
</template>
```

`ShippingAddress.vue`

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Shipping postal code</label>
  <input :id="postalCodeId" />
</template>
```

`BillingAddress.vue`

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Billing postal code</label>
  <input :id="postalCodeId" />
</template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/element_id.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/event-listener-leak`

Um ouvinte de eventos é registrado e nunca removido.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useWidth } from './use-width';
const width = useWidth();
</script>

<template>
<p>{{ width }}</p>
</template>

```

<span id="vize-croquis-cf-event-listener-leak-bad"></span>

**Incorreto**

A montagem adiciona um ouvinte de redimensionamento de window que captura a ref de largura do componente, mas a desmontagem nunca o remove. Montagens repetidas podem reter ouvintes e estado sem uso.

`use-width.ts`

```ts annotate="remove:1"
import { onMounted, ref } from 'vue';
export function useWidth() {
  const width = ref(0);
  const resize = () => { width.value = window.innerWidth; };
  onMounted(() => { resize(); window.addEventListener('resize', resize); });
  return width;
}

```

<span id="vize-croquis-cf-event-listener-leak-good"></span>

**Correto**

`onUnmounted` remove exatamente a mesma função `resize` registrada na montagem, encerrando o ciclo de vida do ouvinte externo dessa instância.

`use-width.ts`

```ts annotate="add:1,6"
import { onMounted, onUnmounted, ref } from 'vue';
export function useWidth() {
  const width = ref(0);
  const resize = () => { width.value = window.innerWidth; };
  onMounted(() => { resize(); window.addEventListener('resize', resize); });
  onUnmounted(() => { window.removeEventListener('resize', resize); });
  return width;
}

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/event-modifier`

Um ouvinte de eventos usa um modificador que o evento emitido não suporta.

Severidade padrão: info  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-event-modifier-bad"></span>

**Incorreto**

`.stop` pressupõe o método de propagação de um evento nativo no evento personalizado `save` do filho, cujo conteúdo não precisa ser um Event do DOM.

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save.stop="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-event-modifier-good"></span>

**Correto**

Remova `.stop` do ouvinte do evento personalizado; trate a propagação nativa no ouvinte real do DOM quando necessário.

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/hydration-risk`

Este código de diagnóstico agrupa várias ocorrências de reatividade, incluindo uma prop copiada para uma ref. Ele não implica que toda expressão Date.now() seja detectada pela passagem de análise entre arquivos.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/hydration-risk": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-hydration-risk-bad"></span>

**Incorreto**

O filho inicializa `ref(props.count)` uma vez, de modo que sua contagem local deixa de acompanhar mudanças posteriores na prop do pai. Este é o produtor atual prop-to-ref, não um exemplo geral de SSR não determinístico.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :count="0" /></template>
```

`Child.vue`

```vue annotate="remove:2,4"
<script setup lang="ts">
import { ref } from "vue";
const props = defineProps<{ count: number }>();
const count = ref(props.count);
</script>
<template><p>{{ count }}</p></template>
```

<span id="vize-croquis-cf-hydration-risk-good"></span>

**Correto**

`toRef(props, "count")` aponta para a prop em vez de copiar seu valor inicial para um estado independente.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :count="0" /></template>
```

`Child.vue`

```vue annotate="add:2,4"
<script setup lang="ts">
import { toRef } from "vue";
const props = defineProps<{ count: number }>();
const count = toRef(props, "count");
</script>
<template><p>{{ count }}</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/inherit-attrs-unused`

`inheritAttrs: false` está definido e o componente nunca lê os atributos.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-inherit-attrs-unused-bad"></span>

**Incorreto**

O filho define `inheritAttrs: false`, mas nunca encaminha o atributo `class="notice"` do pai.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue annotate="remove:4"
<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>
<template><main>Content</main></template>
```

<span id="vize-croquis-cf-inherit-attrs-unused-good"></span>

**Correto**

Mantenha o controle explícito da herança e vincule `$attrs` ao destino `<main>` pretendido.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue annotate="add:4"
<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>
<template><main v-bind="$attrs">Content</main></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/inject-without-symbol`

`inject` usa uma chave comum em vez de um símbolo `InjectionKey`.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/inject-without-symbol": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";
export interface Theme { color: string; }
export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

<span id="vize-croquis-cf-inject-without-symbol-bad"></span>

**Incorreto**

O consumidor injeta a chave de string sem tipo `"theme"`, que não oferece uma identidade de símbolo compartilhada com o provedor.

`ThemeProvider.vue`

```vue annotate="remove:6"
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

<span id="vize-croquis-cf-inject-without-symbol-good"></span>

**Correto**

O consumidor e o provedor importam o mesmo `ThemeKey` em vez de duplicar nomes de string.

`ThemeProvider.vue`

```vue annotate="add:4,7"
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue annotate="add:3,5"
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/keys.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/injected-async-mutation-race`

Um valor injetado é alterado por uma tarefa assíncrona sujeita a uma condição de corrida.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/injected-async-mutation-race": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./StoreProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export async function loadCount(query: string, options?: { signal?: AbortSignal }): Promise<number> {
  const response = await fetch(`/count?q=${encodeURIComponent(query)}`, options);
  return Number(await response.text());
}
```

`CountSummary.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { StoreKey } from "./keys/store";
const store = inject(StoreKey)!;
</script>
<template><p>{{ store.count }}</p></template>
```

<span id="vize-croquis-cf-injected-async-mutation-race-bad"></span>

**Incorreto**

`CountLoader.vue` escreve um resultado aguardado diretamente no armazenamento injetado compartilhado com `CountSummary.vue`, permitindo que um trabalho desatualizado afete os dois consumidores.

`keys/store.ts`

```ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

`StoreProvider.vue`

```vue annotate="remove:12"
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);
</script>

<template>
  <CountLoader />
  <CountSummary />
</template>
```

`CountLoader.vue`

```vue annotate="remove:3,4,6,9,10"
<script setup lang="ts">
import { loadCount } from "./api";
import { inject, ref, watch } from "vue";
import { StoreKey } from "./keys/store";

const store = inject(StoreKey)!;
const query = ref("");

watch(query, async (value) => {
  store.count = await loadCount(value);
});
</script>
```

<span id="vize-croquis-cf-injected-async-mutation-race-good"></span>

**Correto**

O carregador cancela trabalhos invalidados e emite apenas um resultado ativo. O provedor fica responsável pela alteração do armazenamento por meio de `applyLoadedCount`.

`keys/store.ts`

```ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

`StoreProvider.vue`

```vue annotate="add:9,10,11,12,16"
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);

function applyLoadedCount(count: number) {
  store.count = count;
}
</script>

<template>
  <CountLoader @loaded="applyLoadedCount" />
  <CountSummary />
</template>
```

`CountLoader.vue`

```vue annotate="add:3,5,8,9,10,11,12,13,14,15,16,17,18"
<script setup lang="ts">
import { loadCount } from "./api";
import { ref, watch } from "vue";

const emit = defineEmits<{ loaded: [count: number] }>();
const query = ref("");

watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;

  onCleanup(() => {
    active = false;
    controller.abort();
  });

  const count = await loadCount(value, { signal: controller.signal });
  if (active) emit("loaded", count);
});
</script>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/lifecycle-outside-setup`

Um gancho de ciclo de vida é registrado fora de `setup`.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`install-title.ts`

```ts
import { onMounted } from 'vue';
export function installTitle() {
  onMounted(() => { document.title = 'Mounted application'; });
}

```

<span id="vize-croquis-cf-lifecycle-outside-setup-bad"></span>

**Incorreto**

O ponto de entrada chama `installTitle()` antes de montar uma aplicação, de modo que `onMounted` é registrado sem um contexto de setup de componente ativo.

`main.ts`

```ts annotate="remove:1,2,3,4,5,6"
import { createApp } from 'vue';
import App from './App.vue';
import { installTitle } from './install-title';
installTitle();
createApp(App).mount('#app');

```

`App.vue`

```vue annotate="remove:2"
<script setup lang="ts">

</script>

<template>
<p>Application</p>
</template>

```

<span id="vize-croquis-cf-lifecycle-outside-setup-good"></span>

**Correto**

Chamar o mesmo auxiliar de forma síncrona no setup de App vincula a função de retorno do ciclo de vida à montagem dessa instância.

`App.vue`

```vue annotate="add:2,3"
<script setup lang="ts">
import { installTitle } from './install-title';
installTitle();
</script>

<template>
<p>Application</p>
</template>

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/lifecycle-without-cleanup`

Um gancho de ciclo de vida inicia um trabalho e nunca faz sua limpeza.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-lifecycle-without-cleanup-bad"></span>

**Incorreto**

A montagem registra um ouvinte de redimensionamento de window, mas a desmontagem nunca remove a mesma função de retorno.

`App.vue`

```vue annotate="remove:2"
<script setup lang="ts">
import { onMounted } from "vue";
const resize = () => {};
onMounted(() => { window.addEventListener("resize", resize); });
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-lifecycle-without-cleanup-good"></span>

**Correto**

`onUnmounted` remove o ouvinte com o mesmo nome de evento e a mesma identidade de função usados por `addEventListener`.

`App.vue`

```vue annotate="add:2,5"
<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
const resize = () => {};
onMounted(() => { window.addEventListener("resize", resize); });
onUnmounted(() => { window.removeEventListener("resize", resize); });
</script>
<template><p>Content</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/missing-required-prop`

Uma prop obrigatória não é passada.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-missing-required-prop-bad"></span>

**Incorreto**

O pai renderiza `<Child />` sem a prop obrigatória `title: string` do filho.

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

<span id="vize-croquis-cf-missing-required-prop-good"></span>

**Correto**

`title="Hello"` fornece a prop obrigatória declarada pelo filho resolvido.

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/missing-suspense`

Uma dependência assíncrona é usada fora de um limite Suspense.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`AsyncCard.vue`

```vue
<script setup lang="ts">
const message = await Promise.resolve('Ready');
</script>

<template>
<p>{{ message }}</p>
</template>

```

<span id="vize-croquis-cf-missing-suspense-bad"></span>

**Incorreto**

`AsyncCard` tem await no nível superior, tornando seu setup assíncrono, mas App o renderiza sem um limite Suspense para coordenar essa dependência.

`App.vue`

```vue annotate="remove:6"
<script setup lang="ts">
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<AsyncCard />
</template>

```

<span id="vize-croquis-cf-missing-suspense-good"></span>

**Correto**

App envolve o filho assíncrono em `Suspense` e fornece conteúdo alternativo de carregamento até que o setup do filho seja resolvido.

`App.vue`

```vue annotate="add:2,7"
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /><template #fallback><p>Loading…</p></template></Suspense>
</template>

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/module-scope-reactive`

O estado reativo é criado no escopo do módulo e compartilhado por todos os chamadores.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Estado reativo no escopo de módulo é válido para stores intencionais da aplicação. Este exemplo pressupõe isolamento por componente ou requisição; o contrato publicado atualmente não tem produtor.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Counter from './Counter.vue';
</script>

<template>
<Counter /><Counter />
</template>

```

`Counter.vue`

```vue
<script setup lang="ts">
import { createCounter } from './counter';
const { count } = createCounter();
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

<span id="vize-croquis-cf-module-scope-reactive-bad"></span>

**Incorreto**

O módulo inicializa `count` uma vez, e as duas instâncias de Counter recebem a mesma ref. Clicar em uma altera os dois contadores, embora este exemplo pretenda manter um estado independente por instância.

`counter.ts`

```ts annotate="remove:2,3"
import { ref } from 'vue';
const count = ref(0);
export function createCounter() { return { count }; }

```

<span id="vize-croquis-cf-module-scope-reactive-good"></span>

**Correto**

Criar a ref dentro de `createCounter` dá a cada chamada síncrona de setup um objeto de estado separado, de modo que cada botão possui seu contador.

`counter.ts`

```ts annotate="add:2,3,4,5"
import { ref } from 'vue';
export function createCounter() {
  const count = ref(0);
  return { count };
}

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/multi-root-attrs`

Um componente com múltiplas raízes recebe atributos e não tem onde colocá-los.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-multi-root-attrs-bad"></span>

**Incorreto**

O filho tem raízes `<main>` e `<aside>`, de modo que o Vue não tem uma raiz única que possa receber automaticamente a classe do pai.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue annotate="remove:1"
<template><main>Content</main><aside>Help</aside></template>
```

<span id="vize-croquis-cf-multi-root-attrs-good"></span>

**Correto**

Encaminhe `$attrs` explicitamente para `<main>`, mantendo a segunda raiz.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue annotate="add:1"
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/mutated-after-escape`

Um objeto reativo é alterado depois de escapar de seu proprietário.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Esta é uma política explícita de propriedade de histórico imutável, e não uma proibição geral de passar objetos reativos ou de modificá-los depois. Nenhum produtor atual emite este contrato.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`archive.ts`

```ts
export interface Profile { name: string }
const records: Readonly<Profile>[] = [];
export function publish(profile: Readonly<Profile>): void { records.push(profile); }
export function latestName(): string { return records.at(-1)?.name ?? ''; }

```

`App.vue`

```vue
<script setup lang="ts">
import { publishProfile } from './profile';
import { latestName } from './archive';
publishProfile();
const archivedName = latestName();
</script>

<template>
<p>Archived name: {{ archivedName }}</p>
</template>

```

<span id="vize-croquis-cf-mutated-after-escape-bad"></span>

**Incorreto**

O arquivo retém o mesmo objeto passado a `publish`. Seu proprietário então altera o nome, mudando retroativamente o registro supostamente histórico para Grace. O parâmetro Readonly do TypeScript não copia o objeto.

`profile.ts`

```ts annotate="remove:5"
import { reactive } from 'vue';
import { publish } from './archive';
export function publishProfile(): void {
  const profile = reactive({ name: 'Ada' });
  publish(profile);
  profile.name = 'Grace';
}

```

<span id="vize-croquis-cf-mutated-after-escape-good"></span>

**Correto**

Publicar uma cópia simples separa o registro arquivado de Ada das edições posteriores do perfil reativo. A política de instantâneos do arquivo agora é mantida.

`profile.ts`

```ts annotate="add:5"
import { reactive } from 'vue';
import { publish } from './archive';
export function publishProfile(): void {
  const profile = reactive({ name: 'Ada' });
  publish({ ...profile });
  profile.name = 'Grace';
}

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/non-reactive-provide`

Um valor fornecido não é reativo, portanto os descendentes não verão atualizações.

Severidade padrão: dependente do contexto  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/non-reactive-provide": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-non-reactive-provide-bad"></span>

**Incorreto**

`ThemeProvider.vue` fornece um objeto simples. Alterar os campos desse objeto não dá ao consumidor que o injeta uma dependência reativa do Vue.

`keys/theme.ts`

```ts
export const ThemeKey = Symbol("theme");
```

`ThemeProvider.vue`

```vue annotate="remove:2,6"
<script setup lang="ts">
import { provide } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = { color: "blue" };
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

<span id="vize-croquis-cf-non-reactive-provide-good"></span>

**Correto**

O provedor envolve o tema em `ref`; a mesma referência injetada pode rastrear mudanças posteriores.

`keys/theme.ts`

```ts
export const ThemeKey = Symbol("theme");
```

`ThemeProvider.vue`

```vue annotate="add:2,6"
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/non-unique-id`

Um id de elemento dentro de um laço não é único por item.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/non-unique-id": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ResultsList.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-non-unique-id-bad"></span>

**Incorreto**

Cada iteração de `v-for` renderiza o mesmo ID literal `result-title`; a chave do laço não torna os IDs do DOM únicos.

`ResultsList.vue`

```vue annotate="remove:6"
<script setup lang="ts">
const results = [{ id: "first", title: "First result" }, { id: "second", title: "Second result" }];
</script>
<template>
  <article v-for="result in results" :key="result.id">
    <h2 id="result-title">{{ result.title }}</h2>
  </article>
</template>
```

<span id="vize-croquis-cf-non-unique-id-good"></span>

**Correto**

O ID do título inclui o ID estável do resultado, produzindo um identificador distinto no documento para cada item.

`ResultsList.vue`

```vue annotate="add:6"
<script setup lang="ts">
const results = [{ id: "first", title: "First result" }, { id: "second", title: "Second result" }];
</script>
<template>
  <article v-for="result in results" :key="result.id">
    <h2 :id="`result-${result.id}-title`">{{ result.title }}</h2>
  </article>
</template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/element_id.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/object-identity-comparison`

Um objeto reativo é comparado por identidade, que muda ao remover os invólucros.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

O exemplo pressupõe que os IDs identifiquem os registros de forma única. Comparar duas referências ao mesmo proxy reativo continua válido; este contrato não tem produtor atual.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`user.ts`

```ts
import { reactive } from 'vue';
export function makeUser() {
  const raw = { id: 7, name: 'Ada' };
  return { raw, proxy: reactive(raw) };
}

```

<span id="vize-croquis-cf-object-identity-comparison-bad"></span>

**Incorreto**

`proxy === raw` compara a identidade do invólucro, portanto é falso mesmo que ambos representem o mesmo registro de usuário. A aplicação pretendia comparar a identidade do registro, não a do invólucro do objeto.

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy === raw;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

<span id="vize-croquis-cf-object-identity-comparison-good"></span>

**Correto**

Comparar o `id` estável do registro responde à pergunta pretendida sem depender de o objeto ser bruto ou estar envolvido por um proxy.

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy.id === raw.id;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/pinia-getter`

Uma função de leitura do Pinia é lida sem `storeToRefs`, portanto não permanecerá reativa.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Pinia deve estar instalado, e main.ts instala seu plugin antes da montagem. Ler `store.doubled` diretamente dentro de um cálculo rastreado ou de um template é válido; o defeito aqui é capturar um valor instantâneo comum. Este contrato atualmente não tem produtor.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
createApp(App).use(createPinia()).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`counter-store.ts`

```ts
import { defineStore } from 'pinia';
export const useCounterStore = defineStore('counter', {
  state: () => ({ count: 0 }),
  getters: { doubled: state => state.count * 2 },
});

```

<span id="vize-croquis-cf-pinia-getter-bad"></span>

**Incorreto**

`const doubled = store.doubled` copia o número atual da função de leitura durante setup. O número copiado não acompanha atualizações posteriores de `store.count`.

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const doubled = store.doubled;
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

<span id="vize-croquis-cf-pinia-getter-good"></span>

**Correto**

`storeToRefs(store)` fornece uma ref reativa da função de leitura que pode ser desestruturada e desembrulhada pelo template, mantendo a conexão com o armazenamento.

`App.vue`

```vue annotate="add:2,5"
<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const { doubled } = storeToRefs(store);
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/prop-type-mismatch`

O valor de uma prop passada não corresponde ao tipo declarado.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-prop-type-mismatch-bad"></span>

**Incorreto**

O pai passa a expressão numérica `42` para a prop `title: string` do filho resolvido.

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :title="42" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

<span id="vize-croquis-cf-prop-type-mismatch-good"></span>

**Correto**

O literal `title="Hello"` fornece uma string que corresponde à declaração do filho.

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/provide-inject-type`

Um valor fornecido e sua injeção não têm o mesmo tipo.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/provide-inject-type": "warn" },
    },
  },
});
```

```sh
vp run lint
```

Esta verificação compara anotações explícitas de tipos do provedor e do consumidor, e não tipos inferidos de valores literais. Mantenha a anotação `as string` do provedor neste exemplo.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-provide-inject-type-bad"></span>

**Incorreto**

O provedor anota explicitamente `title` como `string`, enquanto o descendente solicita `inject<number>` para a mesma chave.

`App.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import Child from "./Child.vue";
provide("title", "Hello" as string);
</script>
<template><Child /></template>
```

`Child.vue`

```vue annotate="remove:3"
<script setup lang="ts">
import { inject } from "vue";
const title = inject<number>("title");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-provide-inject-type-good"></span>

**Correto**

O `inject<string>` explícito do consumidor concorda com a anotação do provedor. Mantenha `as string`: este produtor compara anotações explícitas, não tipos literais inferidos.

`App.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import Child from "./Child.vue";
provide("title", "Hello" as string);
</script>
<template><Child /></template>
```

`Child.vue`

```vue annotate="add:3"
<script setup lang="ts">
import { inject } from "vue";
const title = inject<string>("title");
</script>
<template><p>Content</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis/diagnostics.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/provide-without-symbol`

`provide` usa uma chave comum em vez de um símbolo `InjectionKey`.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/provide-without-symbol": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-provide-without-symbol-bad"></span>

**Incorreto**

Os dois componentes usam a string `"theme"`; funcionalidades sem relação podem reutilizar essa chave acidentalmente.

`ThemeProvider.vue`

```vue annotate="remove:5,6"
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

<span id="vize-croquis-cf-provide-without-symbol-good"></span>

**Correto**

Exporte um único símbolo tipado `ThemeKey` e importe esse mesmo valor nos pontos de provide e inject. Criar símbolos separados com a mesma descrição não os conectaria.

`ThemeProvider.vue`

```vue annotate="add:4,6,7"
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue annotate="add:3,5"
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

`keys/theme.ts`

```ts annotate="add:1,2,3,4,5,6,7"
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/keys.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/reactive-export`

O estado reativo é exportado pelo módulo.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Stores da aplicação compartilhados intencionalmente podem exportar estado reativo. Este cenário exige estado isolado e não afirma que toda exportação reativa seja inválida. Nenhum produtor atual emite este contrato.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

<span id="vize-croquis-cf-reactive-export-bad"></span>

**Incorreto**

O módulo exporta um único objeto reativo inicializado, de modo que todo importador recebe a mesma contagem. Em um módulo SSR compartilhado entre requisições, isso impede o isolamento de estado por instância ou requisição pretendido no exemplo.

`state.ts`

```ts annotate="remove:2"
import { reactive } from 'vue';
export const state = reactive({ count: 0 });

```

`App.vue`

```vue annotate="remove:2"
<script setup lang="ts">
import { state } from './state';
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

<span id="vize-croquis-cf-reactive-export-good"></span>

**Correto**

O módulo exporta uma fábrica, e App a chama dentro de setup. Cada instância obtém uma nova contagem reativa em vez da instância única exportada.

`state.ts`

```ts annotate="add:2"
import { reactive } from 'vue';
export function createState() { return reactive({ count: 0 }); }

```

`App.vue`

```vue annotate="add:2,3"
<script setup lang="ts">
import { createState } from './state';
const state = createState();
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/reactivity-outside-setup`

Uma API reativa é chamada fora de `setup`.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

O Vue permite ref/reactive/computed fora do setup de componente. O risco aqui é a propriedade ou o compartilhamento indesejados sob uma política explícita de isolamento de instâncias, e não um uso inválido da API. Este contrato não tem produtor atual.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Counter from './Counter.vue';
</script>

<template>
<Counter /><Counter />
</template>

```

`Counter.vue`

```vue
<script setup lang="ts">
import { useCounter } from './use-counter';
const { count, doubled } = useCounter();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>
</template>

```

<span id="vize-croquis-cf-reactivity-outside-setup-bad"></span>

**Incorreto**

As duas APIs reativas são executadas durante o carregamento do módulo. As duas instâncias de Counter, portanto, compartilham uma ref e um valor computado, apesar da intenção de manter contadores independentes.

`use-counter.ts`

```ts annotate="remove:2,3,4"
import { computed, ref } from 'vue';
const count = ref(0);
const doubled = computed(() => count.value * 2);
export function useCounter() { return { count, doubled }; }

```

<span id="vize-croquis-cf-reactivity-outside-setup-good"></span>

**Correto**

`useCounter` cria a ref e o valor computado de forma síncrona dentro de cada chamada de setup de componente, dando a cada componente visual seu próprio estado e sua própria derivação rastreada.

`use-counter.ts`

```ts annotate="add:2,3,4,5,6"
import { computed, ref } from 'vue';
export function useCounter() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/reassignment-breaks-reactivity`

Reatribuir uma variável reativa a substitui por um valor simples.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/reassignment-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-reassignment-breaks-reactivity-bad"></span>

**Incorreto**

O filho cria uma ref de prop e depois sobrescreve a variável com `props.user`, descartando a conexão dessa ref.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue annotate="remove:5,6,7"
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
let user = toRef(props, "user");

user = props.user;
</script>
```

<span id="vize-croquis-cf-reassignment-breaks-reactivity-good"></span>

**Correto**

Mantenha o `toRef` em uma variável `const` e remova a reatribuição que o substitui.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue annotate="add:5"
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
const user = toRef(props, "user");
</script>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/reference-escapes-scope`

Uma referência reativa escapa do escopo responsável por seu ciclo de vida.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Refs podem legitimamente ser retornadas por composables ou compartilhadas entre escopos. Este exemplo exige explicitamente um cache de valores instantâneos; não afirma que a desmontagem invalide uma ref. Nenhum produtor atual emite este contrato.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`saved.ts`

```ts
import type { Ref } from 'vue';
let saved: Ref<number> | number | undefined;
export function remember(value: Ref<number> | number): void { saved = value; }
export function remembered(): Ref<number> | number | undefined { return saved; }

```

<span id="vize-croquis-cf-reference-escapes-scope-bad"></span>

**Incorreto**

O cache no nível do processo retém a ref ativa de contagem do componente. Ele pode manter o estado dessa instância alcançável após a desmontagem e observar edições posteriores, embora esse cache deva armazenar um instantâneo.

`App.vue`

```vue annotate="remove:5"
<script setup lang="ts">
import { ref } from 'vue';
import { remember } from './saved';
const count = ref(0);
remember(count);
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

<span id="vize-croquis-cf-reference-escapes-scope-good"></span>

**Correto**

O cache recebe o número simples atual, de modo que mantém um instantâneo sem reter a ref pertencente ao componente.

`App.vue`

```vue annotate="add:5"
<script setup lang="ts">
import { ref } from 'vue';
import { remember } from './saved';
const count = ref(0);
remember(count.value);
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/setup-context-violation`

O contexto de setup é usado de uma forma que o Vue não permite.

Severidade padrão: dependente do contexto  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-setup-context-violation-bad"></span>

**Incorreto**

`ref(0)` é criada no escopo do módulo de um script comum, fora do contexto de setup por instância representado por este cenário do analisador.

`App.vue`

```vue annotate="remove:1,4,6"
<script lang="ts">
import { ref } from "vue";
const count = ref(0);
export default {};
</script>
<template><p>Count</p></template>
```

<span id="vize-croquis-cf-setup-context-violation-good"></span>

**Correto**

Mova a variável para script setup, onde cada instância de componente possui sua contagem e o template pode lê-la.

`App.vue`

```vue annotate="add:1,5"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
</script>
<template><p>{{ count }}</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/shallow-deep-access`

Uma propriedade profunda de um valor `shallowReactive` ou `shallowRef` é lida como se fosse rastreada.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { makeProfile } from './profile';
const profile = makeProfile();
</script>

<template>
<p>{{ profile.user.name }}</p><button @click="profile.user.name = 'Grace'">Rename</button>
</template>

```

<span id="vize-croquis-cf-shallow-deep-access-bad"></span>

**Incorreto**

`shallowReactive` rastreia a propriedade raiz `user`, mas deixa o objeto aninhado bruto. Alterar `profile.user.name` não notifica o template como uma mutação profunda rastreada.

`profile.ts`

```ts annotate="remove:1,2"
import { shallowReactive } from 'vue';
export function makeProfile() { return shallowReactive({ user: { name: 'Ada' } }); }

```

<span id="vize-croquis-cf-shallow-deep-access-good"></span>

**Correto**

O `reactive` profundo envolve o objeto de usuário aninhado, de modo que a mesma atribuição de nome pode acionar a atualização do nome exibido.

`profile.ts`

```ts annotate="add:1,2"
import { reactive } from 'vue';
export function makeProfile() { return reactive({ user: { name: 'Ada' } }); }

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/spread-breaks-reactivity`

Espalhar um objeto reativo copia seus valores e perde o rastreamento.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/spread-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-spread-breaks-reactivity-bad"></span>

**Incorreto**

`UserSummary.vue` espalha `props.user` em um novo objeto, criando um instantâneo dos dados reativos recebidos.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada", role: "admin" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue annotate="remove:3"
<script setup lang="ts">
const props = defineProps<{ user: { name: string; role: string } }>();
const copiedUser = { ...props.user };
</script>
```

<span id="vize-croquis-cf-spread-breaks-reactivity-good"></span>

**Correto**

`toRef(props, "user")` mantém uma referência à prop recebida em vez de copiar seus campos.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada", role: "admin" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue annotate="add:2,3,5"
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string; role: string } }>();
const user = toRef(props, "user");
</script>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/suspense-no-fallback`

`<Suspense>` não tem conteúdo alternativo.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Suspense sem conteúdo alternativo é uma sintaxe válida do Vue. Esta é uma convenção escolhida de interface de carregamento, e não um erro do compilador; o contrato publicado não tem produtor atual.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`AsyncCard.vue`

```vue
<script setup lang="ts">
const message = await Promise.resolve('Ready');
</script>

<template>
<p>{{ message }}</p>
</template>

```

<span id="vize-croquis-cf-suspense-no-fallback-bad"></span>

**Incorreto**

O limite Suspense tem um filho assíncrono, mas não tem conteúdo alternativo, deixando este exemplo sem conteúdo de carregamento durante o estado pendente.

`App.vue`

```vue annotate="remove:7"
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /></Suspense>
</template>

```

<span id="vize-croquis-cf-suspense-no-fallback-good"></span>

**Correto**

O slot `#fallback` fornece um parágrafo explícito de carregamento até que o filho assíncrono seja resolvido.

`App.vue`

```vue annotate="add:7"
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /><template #fallback><p>Loading…</p></template></Suspense>
</template>

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/template-ref-timing`

Uma ref de template é lida antes da montagem do componente.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`focus-input.ts`

```ts
export function focusInput(input: HTMLInputElement | null): void { input?.focus(); }

```

<span id="vize-croquis-cf-template-ref-timing-bad"></span>

**Incorreto**

Setup lê a ref de template antes da montagem, quando seu valor ainda é nulo. Por isso, a chamada opcional de foco não realiza nenhuma ação de foco.

`App.vue`

```vue annotate="remove:2,5"
<script setup lang="ts">
import { ref } from 'vue';
import { focusInput } from './focus-input';
const input = ref<HTMLInputElement | null>(null);
focusInput(input.value);
</script>

<template>
<input ref="input" aria-label="Name" />
</template>

```

<span id="vize-croquis-cf-template-ref-timing-good"></span>

**Correto**

`onMounted` adia a leitura até que o Vue tenha atribuído o elemento de entrada à ref de template, permitindo que o auxiliar de foco atue sobre ele.

`App.vue`

```vue annotate="add:2,5"
<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { focusInput } from './focus-input';
const input = ref<HTMLInputElement | null>(null);
onMounted(() => { focusInput(input.value); });
</script>

<template>
<input ref="input" aria-label="Name" />
</template>

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/toraw-mutation`

`toRaw` é usado e o objeto bruto é alterado em seguida.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { makeProfile, rename } from './profile';
const profile = makeProfile();
</script>

<template>
<p>{{ profile.name }}</p><button @click="rename(profile)">Rename</button>
</template>

```

<span id="vize-croquis-cf-toraw-mutation-bad"></span>

**Incorreto**

`rename` obtém o objeto-alvo bruto e escreve em `raw.name`, contornando a função de escrita do proxy que notificaria o nome reativo exibido.

`profile.ts`

```ts annotate="remove:1,4,5"
import { reactive, toRaw } from 'vue';
export function makeProfile() { return reactive({ name: 'Ada' }); }
export function rename(profile: { name: string }): void {
  const raw = toRaw(profile);
  raw.name = 'Grace';
}

```

<span id="vize-croquis-cf-toraw-mutation-good"></span>

**Correto**

Escrever em `profile.name` por meio do proxy reativo passado preserva a mesma renomeação e notifica seus dependentes.

`profile.ts`

```ts annotate="add:1,4"
import { reactive } from 'vue';
export function makeProfile() { return reactive({ name: 'Ada' }); }
export function rename(profile: { name: string }): void {
  profile.name = 'Grace';
}

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/uncaught-error`

Um componente pode lançar um erro e nenhum limite de erro o captura.

Severidade padrão: info  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/uncaught-error": "warn" },
    },
  },
});
```

```sh
vp run lint
```

O produtor atual examina expressões de template, como JSON.parse(input). Não reporta uma instrução throw que exista apenas no bloco script.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-uncaught-error-bad"></span>

**Incorreto**

O template do filho chama `JSON.parse` com uma entrada malformada, e o pai alcançável não tem um limite de captura de erros.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const input = "{";
</script>
<template><button @click="JSON.parse(input)">Parse</button></template>
```

<span id="vize-croquis-cf-uncaught-error-good"></span>

**Correto**

O pai registra `onErrorCaptured` ao redor desse filho. Retornar `false` interrompe a propagação; um limite de erro em produção também deve apresentar uma interface útil para recuperação.

`App.vue`

```vue annotate="add:2,4"
<script setup lang="ts">
import { onErrorCaptured } from "vue";
import Child from "./Child.vue";
onErrorCaptured(() => false);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const input = "{";
</script>
<template><button @click="JSON.parse(input)">Parse</button></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/undeclared-emit`

O componente emite um evento que não foi declarado.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-undeclared-emit-bad"></span>

**Incorreto**

O filho chama `emit("save")`, mas seu contrato `defineEmits` declara apenas `cancel`.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits<{ cancel: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-undeclared-emit-good"></span>

**Correto**

Declare `save` com sua tupla vazia de argumentos para que o evento emitido concorde com o contrato do componente.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/undeclared-prop`

Um pai passa uma prop que o filho não declara.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-undeclared-prop-bad"></span>

**Incorreto**

O pai passa `typo`, embora o filho resolvido declare apenas `title`. Essa convenção do analisador é distinta do comportamento geral de herança automática de atributos do Vue.

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" :typo="true" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

<span id="vize-croquis-cf-undeclared-prop-good"></span>

**Correto**

Remova a vinculação não intencional de `typo` e mantenha a prop declarada `title`.

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/undefined-slot`

Um pai preenche um slot que o filho não expõe.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`Card.vue`

```vue
<script setup lang="ts">
defineSlots<{ header(): unknown }>();
</script>

<template>
<article><header><slot name="header" /></header></article>
</template>

```

<span id="vize-croquis-cf-undefined-slot-bad"></span>

**Incorreto**

App fornece um slot `footer`, mas Card declara e renderiza apenas `header`. O conteúdo de Notice fornecido não tem um ponto de renderização de slot correspondente nesse filho.

`App.vue`

```vue annotate="remove:6"
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #footer>Notice</template></Card>
</template>

```

<span id="vize-croquis-cf-undefined-slot-good"></span>

**Correto**

App fornece `header`, correspondendo tanto à declaração tipada de slot do filho quanto ao seu ponto de renderização, de modo que Notice aparece ali.

`App.vue`

```vue annotate="add:6"
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #header>Notice</template></Card>
</template>

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/unhandled-event`

Um filho emite um evento que nenhum pai trata.

Severidade padrão: info  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unhandled-event-bad"></span>

**Incorreto**

`Child.vue` emite `save`, mas seu invólucro imediato não escuta esse evento; eventos de componentes não se propagam automaticamente por invólucros.

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-unhandled-event-good"></span>

**Correto**

`Wrapper.vue` associa um ouvinte de `save` ao seu filho direto. A função de retorno vazia demonstra o tratamento para esta regra, não uma implementação completa de salvamento.

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/unmatched-inject`

`inject` nomeia uma chave que nenhum ancestral fornece.

Severidade padrão: error / warning (com valor padrão)  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/unmatched-inject": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unmatched-inject-bad"></span>

**Incorreto**

`ThemeLabel.vue` injeta `ThemeKey`, mas seu ancestral alcançável `App.vue` nunca fornece essa chave.

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

`App.vue`

```vue
<script setup lang="ts">
import ThemeLabel from "./ThemeLabel.vue";
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

<span id="vize-croquis-cf-unmatched-inject-good"></span>

**Correto**

`App.vue` fornece um tema reativo usando o mesmo `ThemeKey` exportado, antes de renderizar o descendente que o injeta.

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

`App.vue`

```vue annotate="add:2,4,5,6,7"
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis/diagnostics.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/unmatched-listener`

Um pai escuta um evento que o filho não emite.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unmatched-listener-bad"></span>

**Incorreto**

O pai escuta `save`, enquanto o filho resolvido declara apenas `cancel`.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits<{ cancel: [] }>();
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-unmatched-listener-good"></span>

**Correto**

O filho declara e emite `save`, correspondendo ao nome do ouvinte do pai.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/unregistered-component`

Um template usa um componente que não foi registrado nem importado.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unregistered-component-bad"></span>

**Incorreto**

Existe um arquivo `Child.vue`, mas o pai não importa nem registra `Child` de outra forma para seu template.

`App.vue`

```vue
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

<span id="vize-croquis-cf-unregistered-component-good"></span>

**Correto**

Importe `Child` no script setup do pai para que o template resolva a variável do componente.

`App.vue`

```vue annotate="add:1,2,3"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/component_resolution.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/unresolved-import`

Uma importação não resolve para um módulo.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unresolved-import-bad"></span>

**Incorreto**

O pai importa `./Missing.vue`, mas o projeto contém `Child.vue` em vez desse caminho.

`App.vue`

```vue annotate="remove:2"
<script setup lang="ts">
import Child from "./Missing.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

<span id="vize-croquis-cf-unresolved-import-good"></span>

**Correto**

Aponte a importação para o arquivo existente `./Child.vue`, mantendo a mesma variável no template.

`App.vue`

```vue annotate="add:2"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/component_resolution.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/unused-attrs`

Atributos de herança automática são passados para um componente com múltiplas raízes que não os usa.

Severidade padrão: info  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unused-attrs-bad"></span>

**Incorreto**

O `tracking-code` do pai não é consumido como prop nem encaminhado pelo filho com múltiplas raízes.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child tracking-code="notice" /></template>
```

`Child.vue`

```vue annotate="remove:1"
<template><main>Content</main><aside>Help</aside></template>
```

<span id="vize-croquis-cf-unused-attrs-good"></span>

**Correto**

Vincular `$attrs` a `<main>` dá um destino explícito a esse atributo de herança automática.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child tracking-code="notice" /></template>
```

`Child.vue`

```vue annotate="add:1"
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/unused-emit`

Um evento declarado nunca é emitido.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

O CrossFileAnalyzer experimental em Rust tem um produtor para este código. A etapa da CLI não emite este código individualmente; configurar seu ID não ativa essa etapa Rust. Estes cenários descrevem o grafo e os fatos suportados pelo analisador, e não uma promessa do Vite+.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unused-emit-bad"></span>

**Incorreto**

O filho declara `save`, mas nunca chama a função de emissão de eventos com esse nome.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-unused-emit-good"></span>

**Correto**

O exemplo chama `emit("save")`, fazendo com que o evento declarado seja usado. Interações reais devem emiti-lo quando a ação correspondente ocorrer.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue annotate="add:3"
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/unused-provide`

Uma chave fornecida nunca é injetada.

Severidade padrão: warning  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/unused-provide": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";
export interface Theme { color: string; }
export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

<span id="vize-croquis-cf-unused-provide-bad"></span>

**Incorreto**

`App.vue` fornece `ThemeKey`, mas sua subárvore renderizada de `Dashboard.vue` não tem nenhum consumidor dessa chave.

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import Dashboard from "./Dashboard.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <Dashboard />
</template>
```

`Dashboard.vue`

```vue annotate="remove:2"
<template>
  <h1>Dashboard</h1>
</template>
```

<span id="vize-croquis-cf-unused-provide-good"></span>

**Correto**

O painel agora renderiza `ThemeLabel.vue`, que injeta a identidade exata de `ThemeKey` do ancestral.

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import Dashboard from "./Dashboard.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <Dashboard />
</template>
```

`Dashboard.vue`

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
import ThemeLabel from "./ThemeLabel.vue";
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue annotate="add:1,2,3,4,5,6"
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/value-extraction-breaks-reactivity`

Ler um valor reativo para uma variável local perde as atualizações posteriores.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/value-extraction-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-bad"></span>

**Incorreto**

O `item` reativo desestruturado do Vue 3.5 é lido para `itemSnapshot` uma vez; substituições posteriores da prop não atualizam esse instantâneo.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue annotate="remove:3"
<script setup lang="ts">
const { item } = defineProps<{ item: { name: string } }>();
const itemSnapshot = item;
</script>
```

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-good"></span>

**Correto**

Leia `item` dentro de `computed`, para que a transformação de desestruturação reativa de props do Vue possa rastrear cada avaliação.

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue annotate="add:2,3,5"
<script setup lang="ts">
import { computed } from "vue";

const { item } = defineProps<{ item: { name: string } }>();
const itemView = computed(() => item);
</script>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/watch-can-be-computed`

Um observador apenas copia um valor para o estado e pode ser um valor computado.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Isto ilustra a preferência publicada por estado puramente derivado. Watchers continuam adequados para efeitos externos ou estado gravável de forma independente; nenhum produtor atual emite este contrato.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useDouble } from './use-double';
const { count, doubled } = useDouble();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>
</template>

```

<span id="vize-croquis-cf-watch-can-be-computed-bad"></span>

**Incorreto**

O observador não produz nenhum efeito externo; ele apenas mantém uma segunda ref gravável sincronizada com o dobro de `count`. Este exemplo não tem escritas independentes nesse valor derivado.

`use-double.ts`

```ts annotate="remove:1,4,5"
import { ref, watch } from 'vue';
export function useDouble() {
  const count = ref(0);
  const doubled = ref(0);
  watch(count, next => { doubled.value = next * 2; }, { immediate: true });
  return { count, doubled };
}

```

<span id="vize-croquis-cf-watch-can-be-computed-good"></span>

**Correto**

Uma função de leitura computada expressa diretamente a mesma derivação e remove a sincronização manual e o estado gravável adicional.

`use-double.ts`

```ts annotate="add:1,4"
import { computed, ref } from 'vue';
export function useDouble() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/watcheffect-async`

`watchEffect` inicia uma tarefa assíncrona e não consegue limpar a execução anterior.

Severidade padrão: error  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/watcheffect-async": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./SearchPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export interface Result { items: string[]; }
export async function load(query: string, options?: { signal?: AbortSignal }): Promise<Result> {
  const response = await fetch(`/search?q=${encodeURIComponent(query)}`, options);
  return response.json();
}
```

<span id="vize-croquis-cf-watcheffect-async-bad"></span>

**Incorreto**

O `watchEffect` assíncrono mistura a coleta implícita de dependências com uma requisição aguardada e sem uma proteção contra invalidação.

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue annotate="remove:3,8,9,10"
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watchEffect } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watchEffect(async () => {
  result.value = await load(props.query);
});
</script>
```

<span id="vize-croquis-cf-watcheffect-async-good"></span>

**Correto**

Um `watch(() => props.query, ...)` explícito declara a origem, registra a limpeza da requisição e recusa uma resposta desatualizada após a invalidação.

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue annotate="add:3,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22"
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Produtor](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[Índice de verificações entre arquivos](cross-file.md)

### `vize:croquis/cf/watcher-outside-setup`

`watch` ou `watchEffect` é chamado fora de `setup`.

Severidade padrão: Não emitido  
Aplicável a: Grafo de componentes analisado e fatos suportados descritos abaixo  
Correção automática: Nenhuma; revise os arquivos relacionados e aplique a correção  
Opções: Sem opções por código; os diagnósticos suportados pela CLI aceitam substituições de severidade

Este é um contrato de diagnóstico publicado sem produtor atual. O cenário incorreto e correto abaixo explica o risco e a correção; nenhuma opção atualmente faz este código ser emitido.

Watchers no escopo de módulo são válidos quando seu responsável mantém e chama uma função de interrupção ou lhes atribui intencionalmente a duração da aplicação. Este exemplo exige ciclos de vida pertencentes ao componente; o contrato não tem produtor atual.

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Observer from './Observer.vue';
</script>

<template>
<Observer /><Observer />
</template>

```

`Observer.vue`

```vue
<script setup lang="ts">
import { useObserver } from './use-observer';
const { count, observed } = useObserver();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ observed }}</p>
</template>

```

<span id="vize-croquis-cf-watcher-outside-setup-bad"></span>

**Incorreto**

O observador é criado no carregamento do módulo, fora do setup de qualquer uma das instâncias de Observer, e as duas instâncias compartilham suas refs. Ele não é interrompido automaticamente quando uma instância específica de Observer é desmontada.

`use-observer.ts`

```ts annotate="remove:2,3,4,5"
import { ref, watch } from 'vue';
const count = ref(0);
const observed = ref(0);
watch(count, next => { observed.value = next; });
export function useObserver() { return { count, observed }; }

```

<span id="vize-croquis-cf-watcher-outside-setup-good"></span>

**Correto**

Cada chamada síncrona de setup cria suas próprias refs e seu próprio observador dentro de `useObserver`. O Vue associa esse observador ao ciclo de vida do componente que o chama.

`use-observer.ts`

```ts annotate="add:2,3,4,5,6,7"
import { ref, watch } from 'vue';
export function useObserver() {
  const count = ref(0);
  const observed = ref(0);
  watch(count, next => { observed.value = next; });
  return { count, observed };
}

```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Explicação pública](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Índice de verificações entre arquivos](cross-file.md)

### `vue/cross-file-attrs-fallthrough`

Um pai passa atributos para um filho resolvido cuja raiz não pode herdá-los e que não usa $attrs explicitamente.

Severidade padrão: warning  
Aplicável a: Declarações alcançáveis do projeto e componentes importados  
Opções: crossFile; severidade da regra (off/warn/error)  
Correção automática: Nenhuma

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "vue/cross-file-attrs-fallthrough": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**Arquivos compartilhados do projeto**

Use estes arquivos sem alterações nos exemplos incorreto e correto. Instale os pacotes importados no projeto: Vue, além de vue-router ou Pinia onde indicado. Siga as notas de suporte específicas de versão. A raiz de entrada torna explícita a relação entre os componentes.

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vue-cross-file-attrs-fallthrough-bad"></span>

**Incorreto**

O pai passa `class="notice"` para um filho resolvido com raiz em fragmento, que não tem um destino automático para atributos e nunca lê `$attrs`.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue annotate="remove:1"
<template><main>Content</main><aside>Help</aside></template>
```

<span id="vue-cross-file-attrs-fallthrough-good"></span>

**Correto**

O filho escolhe `<main>` como destino vinculando `$attrs` ali; seu irmão `<aside>` permanece separado.

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue annotate="add:1"
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

Os arquivos do exemplo correto demonstram a alteração descrita acima; outros diagnósticos ainda podem se aplicar ao projeto completo.

[Índice de verificações entre arquivos](cross-file.md)
