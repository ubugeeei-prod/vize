---
title: "Todas as Regras de Patina"
---

# Todas as Regras de Patina

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.

As verificações de projeto precisam do grafo completo de componentes analisados. Esta página reúne a finalidade, a configuração, os arquivos compartilhados e os exemplos incorreto e correto completos de cada entrada; use os arquivos compartilhados nos dois exemplos. Os 60 códigos publicados têm limites de suporte distintos: 19 pertencem à etapa da CLI, com 18 pares de código-fonte qualificados e um projeto Vue ilustrativo acompanhado de seu grafo de fluxo reativo; 16 têm produtores experimentais no analisador Rust, mas não são emitidos individualmente por essa etapa; 25 são contratos publicados sem produtor atual de diagnósticos. Os seis IDs de regras específicas de projeto são apresentados separadamente. Ativar um ID não ativa um produtor indisponível. As notas de cada exemplo preservam as limitações de fatos de código-fonte, a qualificação por grafo e as dependências específicas de versão.

A CLI pública expõe a mesma etapa por meio de `vize lint --cross-file`. Os códigos exibidos como `vize:croquis/cf/*` usam `croquis/cf/*` em `lint.vize.rules` (omita `vize:`). Diagnósticos de informação ou sugestão tornam-se avisos da CLI. As localizações relacionadas explicam a relação entre a origem e o consumidor.

<span id="todas-as-regras-de-pátina"></span>
<span id="categorias"></span>
<span id="essencial-48"></span>
<span id="altamente-recomendado-12"></span>
<span id="recomendado-42"></span>
<span id="acessibilidade-31"></span>
<span id="conformidade-com-html-9"></span>
<span id="tipo-consciente-5"></span>
<span id="vapor-7"></span>
<span id="ecossistema-9"></span>
<span id="css-10"></span>
<span id="musea-6"></span>
<span id="roteiro-60"></span>
<span id="essential-48"></span>
<span id="strongly-recommended-12"></span>
<span id="recommended-42"></span>
<span id="accessibility-31"></span>
<span id="html-conformance-9"></span>
<span id="type-aware-5"></span>
<span id="ecosystem-9"></span>
<span id="script-60"></span>

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
| [`ecosystem/nuxt-prefer-nuxt-link`](#ecosystem-nuxt-prefer-nuxt-link) | [Incorreto](#ecosystem-nuxt-prefer-nuxt-link-bad) · [Correto](#ecosystem-nuxt-prefer-nuxt-link-good) | Preferir NuxtLink para links internos da aplicação |
| [`ecosystem/pinia-prefer-store-to-refs`](#ecosystem-pinia-prefer-store-to-refs) | [Incorreto](#ecosystem-pinia-prefer-store-to-refs-bad) · [Correto](#ecosystem-pinia-prefer-store-to-refs-good) | Preferir storeToRefs() ao desestruturar stores do Pinia |
| [`ecosystem/router-link-require-to`](#ecosystem-router-link-require-to) | [Incorreto](#ecosystem-router-link-require-to-bad) · [Correto](#ecosystem-router-link-require-to-good) | Exigir um destino `to` nos componentes RouterLink e NuxtLink |
| [`ecosystem/void-link-require-href`](#ecosystem-void-link-require-href) | [Incorreto](#ecosystem-void-link-require-href-bad) · [Correto](#ecosystem-void-link-require-href-good) | Exigir `href` nos componentes Link do Void Vue |
| [`ecosystem/void-link-valid-method`](#ecosystem-void-link-valid-method) | [Incorreto](#ecosystem-void-link-valid-method-bad) · [Correto](#ecosystem-void-link-valid-method-good) | Validar props method estáticas do Link do Void Vue |
| [`ecosystem/vue-i18n-no-missing-key`](#ecosystem-vue-i18n-no-missing-key) | [Incorreto](#ecosystem-vue-i18n-no-missing-key-bad) · [Correto](#ecosystem-vue-i18n-no-missing-key-good) | Relatar chaves estáticas do vue-i18n ausentes nas mensagens locais do SFC |
| [`ecosystem/vue-router-extra-param`](#ecosystem-vue-router-extra-param) | [Incorreto](#ecosystem-vue-router-extra-param-bad) · [Correto](#ecosystem-vue-router-extra-param-good) | A rota não declara tab; o Vue Router o descarta. |
| [`ecosystem/vue-router-missing-param`](#ecosystem-vue-router-missing-param) | [Incorreto](#ecosystem-vue-router-missing-param-bad) · [Correto](#ecosystem-vue-router-missing-param-good) | O parâmetro obrigatório postId está ausente; depender da rota atual é frágil. |
| [`ecosystem/vue-router-param-type`](#ecosystem-vue-router-param-type) | [Incorreto](#ecosystem-vue-router-param-type-bad) · [Correto](#ecosystem-vue-router-param-type-good) | postId não é repetível, portanto um array é inválido. |
| [`ecosystem/vue-router-prefer-named-link`](#ecosystem-vue-router-prefer-named-link) | [Incorreto](#ecosystem-vue-router-prefer-named-link-bad) · [Correto](#ecosystem-vue-router-prefer-named-link-good) | Preferir objetos de rotas nomeadas a strings de caminho estáticas no RouterLink |
| [`ecosystem/vue-router-prefer-named-push`](#ecosystem-vue-router-prefer-named-push) | [Incorreto](#ecosystem-vue-router-prefer-named-push-bad) · [Correto](#ecosystem-vue-router-prefer-named-push-good) | Preferir objetos de rotas nomeadas para navegação programática com Vue Router |
| [`ecosystem/vue-router-unknown-route`](#ecosystem-vue-router-unknown-route) | [Incorreto](#ecosystem-vue-router-unknown-route-bad) · [Correto](#ecosystem-vue-router-unknown-route-good) | O nome está ausente do roteador completo instalado. |
| [`ecosystem/vue-test-utils-no-html-snapshot`](#ecosystem-vue-test-utils-no-html-snapshot) | [Incorreto](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Correto](#ecosystem-vue-test-utils-no-html-snapshot-good) | Evitar snapshots de wrapper.html() nos testes com Vue Test Utils |
| [`html/cross-component-nesting`](#html-cross-component-nesting) | [Incorreto](#html-cross-component-nesting-bad) · [Correto](#html-cross-component-nesting-good) | Verificar o aninhamento HTML real após compor os componentes importados. |
| [`html/deprecated-attr`](#html-deprecated-attr) | [Incorreto](#html-deprecated-attr-bad) · [Correto](#html-deprecated-attr-good) | Proibir atributos HTML obsoletos |
| [`html/deprecated-element`](#html-deprecated-element) | [Incorreto](#html-deprecated-element-bad) · [Correto](#html-deprecated-element-good) | Proibir elementos HTML obsoletos |
| [`html/id-duplication`](#html-id-duplication) | [Incorreto](#html-id-duplication-bad) · [Correto](#html-id-duplication-good) | Proibir IDs de elementos duplicados |
| [`html/no-consecutive-br`](#html-no-consecutive-br) | [Incorreto](#html-no-consecutive-br-bad) · [Correto](#html-no-consecutive-br-good) | Proibir elementos &lt;br&gt; consecutivos |
| [`html/no-dupe-style-properties`](#html-no-dupe-style-properties) | [Incorreto](#html-no-dupe-style-properties-bad) · [Correto](#html-no-dupe-style-properties-good) | Proibir propriedades duplicadas em atributos de estilo em linha |
| [`html/no-duplicate-class`](#html-no-duplicate-class) | [Incorreto](#html-no-duplicate-class-bad) · [Correto](#html-no-duplicate-class-good) | Proibir nomes de classe duplicados em um atributo class estático |
| [`html/no-duplicate-dt`](#html-no-duplicate-dt) | [Incorreto](#html-no-duplicate-dt-bad) · [Correto](#html-no-duplicate-dt-good) | Proibir nomes &lt;dt&gt; duplicados em &lt;dl&gt; |
| [`html/no-empty-palpable-content`](#html-no-empty-palpable-content) | [Incorreto](#html-no-empty-palpable-content-bad) · [Correto](#html-no-empty-palpable-content-good) | Proibir elementos vazios que esperam conteúdo visível |
| [`html/require-datetime`](#html-require-datetime) | [Incorreto](#html-require-datetime-bad) · [Correto](#html-require-datetime-good) | Exigir o atributo datetime no elemento &lt;time&gt; |
| [`musea/no-empty-variant`](#musea-no-empty-variant) | [Incorreto](#musea-no-empty-variant-bad) · [Correto](#musea-no-empty-variant-good) | Proibir blocos &lt;variant&gt; vazios |
| [`musea/prefer-design-tokens`](#musea-prefer-design-tokens) | [Incorreto](#musea-prefer-design-tokens-bad) · [Correto](#musea-prefer-design-tokens-good) | Preferir variáveis CSS de tokens de design a valores primitivos fixos escritos diretamente |
| [`musea/require-component`](#musea-require-component) | [Incorreto](#musea-require-component-bad) · [Correto](#musea-require-component-good) | Exigir o atributo component no bloco &lt;art&gt; |
| [`musea/require-title`](#musea-require-title) | [Incorreto](#musea-require-title-bad) · [Correto](#musea-require-title-good) | Exigir o atributo title no bloco &lt;art&gt; |
| [`musea/unique-variant-names`](#musea-unique-variant-names) | [Incorreto](#musea-unique-variant-names-bad) · [Correto](#musea-unique-variant-names-good) | Exigir nomes de variante únicos |
| [`musea/valid-variant`](#musea-valid-variant) | [Incorreto](#musea-valid-variant-bad) · [Correto](#musea-valid-variant-good) | Exigir o atributo name nos blocos &lt;variant&gt; |
| [`nuxt/no-nuxt-config-test-key`](#nuxt-no-nuxt-config-test-key) | [Incorreto](#nuxt-no-nuxt-config-test-key-bad) · [Correto](#nuxt-no-nuxt-config-test-key-good) | Proibir a definição da chave `test` na configuração do Nuxt |
| [`nuxt/no-page-meta-runtime-values`](#nuxt-no-page-meta-runtime-values) | [Incorreto](#nuxt-no-page-meta-runtime-values-bad) · [Correto](#nuxt-no-page-meta-runtime-values-good) | Proibir valores do contexto de execução no nível de avaliação imediata de `definePageMeta`, que é extraído para um fragmento separado durante a compilação e executado antes do setup do componente |
| [`nuxt/nuxt-config-keys-order`](#nuxt-nuxt-config-keys-order) | [Incorreto](#nuxt-nuxt-config-keys-order-bad) · [Correto](#nuxt-nuxt-config-keys-order-good) | Preferir a ordem recomendada das propriedades de configuração do Nuxt |
| [`nuxt/prefer-import-meta`](#nuxt-prefer-import-meta) | [Incorreto](#nuxt-prefer-import-meta-bad) · [Correto](#nuxt-prefer-import-meta-good) | Preferir `import.meta.*` a `process.*` |
| [`petite-vue/no-unsupported-directive`](#petite-vue-no-unsupported-directive) | [Incorreto](#petite-vue-no-unsupported-directive-bad) · [Correto](#petite-vue-no-unsupported-directive-good) | Proibir diretivas que o petite-vue não aceita |
| [`petite-vue/valid-v-effect`](#petite-vue-valid-v-effect) | [Incorreto](#petite-vue-valid-v-effect-bad) · [Correto](#petite-vue-valid-v-effect-good) | Exigir uma expressão não vazia em v-effect |
| [`petite-vue/valid-v-scope`](#petite-vue-valid-v-scope) | [Incorreto](#petite-vue-valid-v-scope-bad) · [Correto](#petite-vue-valid-v-scope-good) | Exigir que v-scope vincule um objeto literal |
| [`script/component-options-name-casing`](#script-component-options-name-casing) | [Incorreto](#script-component-options-name-casing-bad) · [Correto](#script-component-options-name-casing-good) | Exigir PascalCase na opção `name` do componente |
| [`script/custom-event-name-casing`](#script-custom-event-name-casing) | [Incorreto](#script-custom-event-name-casing-bad) · [Correto](#script-custom-event-name-casing-good) | Exigir camelCase nos nomes de eventos personalizados emitidos |
| [`script/define-emits-declaration`](#script-define-emits-declaration) | [Incorreto](#script-define-emits-declaration-bad) · [Correto](#script-define-emits-declaration-good) | Exigir a forma de defineEmits&lt;{}&gt;() baseada em tipos em vez da forma em tempo de execução ou de array |
| [`script/define-macros-order`](#script-define-macros-order) | [Incorreto](#script-define-macros-order-bad) · [Correto](#script-define-macros-order-good) | Exigir uma ordem consistente para as macros do compilador Vue em &lt;script setup&gt; |
| [`script/define-props-declaration`](#script-define-props-declaration) | [Incorreto](#script-define-props-declaration-bad) · [Correto](#script-define-props-declaration-good) | Exigir defineProps&lt;{ ... }&gt;() baseado em tipos em vez da forma com objeto em tempo de execução |
| [`script/define-props-destructuring`](#script-define-props-destructuring) | [Incorreto](#script-define-props-destructuring-bad) · [Correto](#script-define-props-destructuring-good) | Exigir um estilo consistente de desestruturação de defineProps em &lt;script setup&gt; |
| [`script/no-arrow-functions-in-watch`](#script-no-arrow-functions-in-watch) | [Incorreto](#script-no-arrow-functions-in-watch-bad) · [Correto](#script-no-arrow-functions-in-watch-good) | Proibir funções de seta como manipuladores de watch na Options API |
| [`script/no-async-in-computed`](#script-no-async-in-computed) | [Incorreto](#script-no-async-in-computed-bad) · [Correto](#script-no-async-in-computed-good) | Proibir funções assíncronas em propriedades computadas |
| [`script/no-boolean-default`](#script-no-boolean-default) | [Incorreto](#script-no-boolean-default-bad) · [Correto](#script-no-boolean-default-good) | Proibir um valor padrão em uma prop Boolean |
| [`script/no-deep-destructure-in-props`](#script-no-deep-destructure-in-props) | [Incorreto](#script-no-deep-destructure-in-props-bad) · [Correto](#script-no-deep-destructure-in-props-good) | Proibir desestruturação profundamente aninhada em defineProps |
| [`script/no-deprecated-data-object-declaration`](#script-no-deprecated-data-object-declaration) | [Incorreto](#script-no-deprecated-data-object-declaration-bad) · [Correto](#script-no-deprecated-data-object-declaration-good) | Proibir um literal de objeto como opção data do componente (o Vue 3 exige uma função) |
| [`script/no-deprecated-destroyed-lifecycle`](#script-no-deprecated-destroyed-lifecycle) | [Incorreto](#script-no-deprecated-destroyed-lifecycle-bad) · [Correto](#script-no-deprecated-destroyed-lifecycle-good) | Proibir os hooks de ciclo de vida obsoletos destroyed e beforeDestroy |
| [`script/no-deprecated-dollar-listeners-api`](#script-no-deprecated-dollar-listeners-api) | [Incorreto](#script-no-deprecated-dollar-listeners-api-bad) · [Correto](#script-no-deprecated-dollar-listeners-api-good) | Proibir a propriedade de instância $listeners removida no Vue 3 (incorporada a $attrs) |
| [`script/no-deprecated-dollar-scopedslots-api`](#script-no-deprecated-dollar-scopedslots-api) | [Incorreto](#script-no-deprecated-dollar-scopedslots-api-bad) · [Correto](#script-no-deprecated-dollar-scopedslots-api-good) | Proibir a propriedade de instância $scopedSlots removida no Vue 3 (usar $slots) |
| [`script/no-deprecated-events-api`](#script-no-deprecated-events-api) | [Incorreto](#script-no-deprecated-events-api-bad) · [Correto](#script-no-deprecated-events-api-good) | Proibir a API de eventos do Vue 2 removida ($on / $off / $once) |
| [`script/no-deprecated-props-default-this`](#script-no-deprecated-props-default-this) | [Incorreto](#script-no-deprecated-props-default-this-bad) · [Correto](#script-no-deprecated-props-default-this-good) | Proibir `this` dentro de uma função de valor padrão ou validação de prop (removido no Vue 3) |
| [`script/no-dupe-keys`](#script-no-dupe-keys) | [Incorreto](#script-no-dupe-keys-bad) · [Correto](#script-no-dupe-keys-good) | Proibir chaves duplicadas entre props/data/computed/methods/setup/inject da Options API |
| [`script/no-duplicate-attr-inheritance`](#script-no-duplicate-attr-inheritance) | [Incorreto](#script-no-duplicate-attr-inheritance-bad) · [Correto](#script-no-duplicate-attr-inheritance-good) | Sinalizar um componente que aplica duas vezes seus atributos repassados |
| [`script/no-export-in-script-setup`](#script-no-export-in-script-setup) | [Incorreto](#script-no-export-in-script-setup-bad) · [Correto](#script-no-export-in-script-setup-good) | Proibir instruções export dentro de &lt;script setup&gt; |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [Incorreto](#script-no-get-current-instance-bad) · [Correto](#script-no-get-current-instance-good) | Proibir getCurrentInstance() no modo Vapor (retorna null) |
| [`script/no-import-compiler-macros`](#script-no-import-compiler-macros) | [Incorreto](#script-no-import-compiler-macros-bad) · [Correto](#script-no-import-compiler-macros-good) | Proibir a importação de macros do compilador Vue que são importadas automaticamente |
| [`script/no-internal-imports`](#script-no-internal-imports) | [Incorreto](#script-no-internal-imports-bad) · [Correto](#script-no-internal-imports-good) | Proibir importações de módulos internos do Vue |
| [`script/no-multiple-slot-args`](#script-no-multiple-slot-args) | [Incorreto](#script-no-multiple-slot-args-bad) · [Correto](#script-no-multiple-slot-args-good) | Proibir passar mais de um argumento a uma chamada de função de slot com escopo |
| [`script/no-next-tick`](#script-no-next-tick) | [Incorreto](#script-no-next-tick-bad) · [Correto](#script-no-next-tick-good) | Proibir o uso de nextTick() em componentes orientados a Vapor |
| [`script/no-options-api`](#script-no-options-api) | [Incorreto](#script-no-options-api-bad) · [Correto](#script-no-options-api-good) | Proibir padrões da Options API no modo Vapor |
| [`script/no-potential-component-option-typo`](#script-no-potential-component-option-typo) | [Incorreto](#script-no-potential-component-option-typo-bad) · [Correto](#script-no-potential-component-option-typo-good) | Sinalizar prováveis erros de digitação nos nomes de opções de componentes da Options API |
| [`script/no-reactive-destructure`](#script-no-reactive-destructure) | [Incorreto](#script-no-reactive-destructure-bad) · [Correto](#script-no-reactive-destructure-good) | Proibir a desestruturação de objetos reativos que causa perda de reatividade |
| [`script/no-ref-as-operand`](#script-no-ref-as-operand) | [Incorreto](#script-no-ref-as-operand-bad) · [Correto](#script-no-ref-as-operand-good) | Exigir que variáveis vinculadas a refs sejam acessadas por `.value` quando usadas como operandos |
| [`script/no-required-prop-with-default`](#script-no-required-prop-with-default) | [Incorreto](#script-no-required-prop-with-default-bad) · [Correto](#script-no-required-prop-with-default-good) | Proibir uma prop que tenha required: true e também um valor padrão |
| [`script/no-reserved-identifiers`](#script-no-reserved-identifiers) | [Incorreto](#script-no-reserved-identifiers-bad) · [Correto](#script-no-reserved-identifiers-good) | Proibir o uso de identificadores reservados pelo compilador Vue |
| [`script/no-reserved-keys`](#script-no-reserved-keys) | [Incorreto](#script-no-reserved-keys-bad) · [Correto](#script-no-reserved-keys-good) | Proibir nomes reservados pelo Vue como chaves de props/data/computed/methods/setup/inject na Options API |
| [`script/no-reserved-props`](#script-no-reserved-props) | [Incorreto](#script-no-reserved-props-bad) · [Correto](#script-no-reserved-props-good) | Proibir nomes reservados na declaração de props de um componente |
| [`script/no-restricted-globals`](#script-no-restricted-globals) | [Incorreto](#script-no-restricted-globals-bad) · [Correto](#script-no-restricted-globals-good) | Proibir referências a variáveis globais do ambiente de execução que devem passar por um encapsulamento tipado |
| [`script/no-restricted-members`](#script-no-restricted-members) | [Incorreto](#script-no-restricted-members-bad) · [Correto](#script-no-restricted-members-good) | Proibir acessos a membros object.property configurados pelo projeto |
| [`script/no-side-effects-in-computed-properties`](#script-no-side-effects-in-computed-properties) | [Incorreto](#script-no-side-effects-in-computed-properties-bad) · [Correto](#script-no-side-effects-in-computed-properties-good) | Proibir efeitos colaterais em getters computados da Options API |
| [`script/no-top-level-ref-in-script`](#script-no-top-level-ref-in-script) | [Incorreto](#script-no-top-level-ref-in-script-bad) · [Correto](#script-no-top-level-ref-in-script-good) | Proibir ref/reactive no nível superior para evitar contaminação de estado entre requisições |
| [`script/no-unstable-nested-components`](#script-no-unstable-nested-components) | [Incorreto](#script-no-unstable-nested-components-bad) · [Correto](#script-no-unstable-nested-components-good) | Proibir definições de componentes dentro de funções de setup ou renderização |
| [`script/no-unused-emit-declarations`](#script-no-unused-emit-declarations) | [Incorreto](#script-no-unused-emit-declarations-bad) · [Correto](#script-no-unused-emit-declarations-good) | Sinalizar eventos declarados que nunca são emitidos |
| [`script/no-use-computed-property-like-method`](#script-no-use-computed-property-like-method) | [Incorreto](#script-no-use-computed-property-like-method-bad) · [Correto](#script-no-use-computed-property-like-method-good) | Proibir chamar uma propriedade computada da Options API como um método |
| [`script/no-with-defaults`](#script-no-with-defaults) | [Incorreto](#script-no-with-defaults-bad) · [Correto](#script-no-with-defaults-good) | Desencorajar withDefaults em favor de valores padrão na desestruturação (Vue 3.5+) |
| [`script/prefer-computed`](#script-prefer-computed) | [Incorreto](#script-prefer-computed-bad) · [Correto](#script-prefer-computed-good) | Preferir computed() para estado reativo derivado |
| [`script/prefer-define-options`](#script-prefer-define-options) | [Incorreto](#script-prefer-define-options-bad) · [Correto](#script-prefer-define-options-good) | Preferir defineOptions() a um &lt;script&gt; comum que só define name/inheritAttrs |
| [`script/prefer-import-from-vue`](#script-prefer-import-from-vue) | [Incorreto](#script-prefer-import-from-vue-bad) · [Correto](#script-prefer-import-from-vue-good) | Preferir importar de 'vue' em vez de pacotes internos |
| [`script/prefer-ref-over-reactive`](#script-prefer-ref-over-reactive) | [Incorreto](#script-prefer-ref-over-reactive-bad) · [Correto](#script-prefer-ref-over-reactive-good) | Recomendar o uso de ref() em vez de reactive() para gerenciar estado |
| [`script/prefer-use-attrs`](#script-prefer-use-attrs) | [Incorreto](#script-prefer-use-attrs-bad) · [Correto](#script-prefer-use-attrs-good) | Recomendar o uso de useAttrs() em vez de context.attrs |
| [`script/prefer-use-id`](#script-prefer-use-id) | [Incorreto](#script-prefer-use-id-bad) · [Correto](#script-prefer-use-id-good) | Recomendar o uso de useId() para gerar IDs únicos (Vue 3.5+) |
| [`script/prefer-use-slots`](#script-prefer-use-slots) | [Incorreto](#script-prefer-use-slots-bad) · [Correto](#script-prefer-use-slots-good) | Recomendar o uso de useSlots() em vez de context.slots |
| [`script/prefer-use-template-ref`](#script-prefer-use-template-ref) | [Incorreto](#script-prefer-use-template-ref-bad) · [Correto](#script-prefer-use-template-ref-good) | Recomendar useTemplateRef em vez de ref para referências de template (Vue 3.5+) |
| [`script/require-default-prop`](#script-require-default-prop) | [Incorreto](#script-require-default-prop-bad) · [Correto](#script-require-default-prop-good) | Exigir um valor padrão para toda prop opcional que não seja Boolean |
| [`script/require-explicit-emits`](#script-require-explicit-emits) | [Incorreto](#script-require-explicit-emits-bad) · [Correto](#script-require-explicit-emits-good) | Exigir que os eventos emitidos sejam declarados em defineEmits ou na opção emits |
| [`script/require-explicit-slots`](#script-require-explicit-slots) | [Incorreto](#script-require-explicit-slots-bad) · [Correto](#script-require-explicit-slots-good) | Exigir que os slots consumidos por useSlots() sejam tipados explicitamente com defineSlots&lt;...&gt;() |
| [`script/require-function-return-type`](#script-require-function-return-type) | [Incorreto](#script-require-function-return-type-bad) · [Correto](#script-require-function-return-type-good) | Exigir anotações de tipo de retorno nas funções |
| [`script/require-prop-type-constructor`](#script-require-prop-type-constructor) | [Incorreto](#script-require-prop-type-constructor-bad) · [Correto](#script-require-prop-type-constructor-good) | Exigir que os valores de `type` de props sejam construtores em vez de literais de string |
| [`script/require-prop-types`](#script-require-prop-types) | [Incorreto](#script-require-prop-types-bad) · [Correto](#script-require-prop-types-good) | Exigir que toda prop declare um tipo |
| [`script/require-symbol-provide`](#script-require-symbol-provide) | [Incorreto](#script-require-symbol-provide-bad) · [Correto](#script-require-symbol-provide-good) | Recomendar o uso de Symbol como chave de injeção para provide/inject |
| [`script/require-typed-object-prop`](#script-require-typed-object-prop) | [Incorreto](#script-require-typed-object-prop-bad) · [Correto](#script-require-typed-object-prop-good) | Exigir um tipo explícito em uma prop cujo tipo de tempo de execução seja `Object` ou `Array` |
| [`script/require-typed-ref`](#script-require-typed-ref) | [Incorreto](#script-require-typed-ref-bad) · [Correto](#script-require-typed-ref-good) | Exigir um argumento de tipo explícito em um ref() inicializado sem valor, com null ou com undefined |
| [`script/require-valid-default-prop`](#script-require-valid-default-prop) | [Incorreto](#script-require-valid-default-prop-bad) · [Correto](#script-require-valid-default-prop-good) | Exigir que o valor padrão de uma prop seja válido para seu tipo declarado |
| [`script/return-in-computed-property`](#script-return-in-computed-property) | [Incorreto](#script-return-in-computed-property-bad) · [Correto](#script-return-in-computed-property-good) | Exigir um valor de retorno em todo getter computado |
| [`script/return-in-emits-validator`](#script-return-in-emits-validator) | [Incorreto](#script-return-in-emits-validator-bad) · [Correto](#script-return-in-emits-validator-good) | Exigir um valor de retorno em todo validador de emits da Options API |
| [`script/valid-define-emits`](#script-valid-define-emits) | [Incorreto](#script-valid-define-emits-bad) · [Correto](#script-valid-define-emits-good) | Exigir uso válido de defineEmits() (sem argumentos de tipo e de tempo de execução juntos, sem referências locais, uma única chamada) |
| [`script/valid-define-options`](#script-valid-define-options) | [Incorreto](#script-valid-define-options-bad) · [Correto](#script-valid-define-options-good) | Exigir uso válido de defineOptions() (um único argumento de objeto, sem props/emits/expose/slots) |
| [`script/valid-define-props`](#script-valid-define-props) | [Incorreto](#script-valid-define-props-bad) · [Correto](#script-valid-define-props-good) | Exigir uso válido de defineProps() (uma única chamada, sem argumentos de tipo e de tempo de execução juntos, sem referências locais) |
| [`script/valid-next-tick`](#script-valid-next-tick) | [Incorreto](#script-valid-next-tick-bad) · [Correto](#script-valid-next-tick-good) | Exigir que o resultado de uma chamada de nextTick() seja aguardado, encadeado ou receba uma função de retorno |
| [`ssr/no-browser-globals-in-ssr`](#ssr-no-browser-globals-in-ssr) | [Incorreto](#ssr-no-browser-globals-in-ssr-bad) · [Correto](#ssr-no-browser-globals-in-ssr-good) | Proibir variáveis globais exclusivas do navegador no contexto de SSR |
| [`ssr/no-hydration-mismatch`](#ssr-no-hydration-mismatch) | [Incorreto](#ssr-no-hydration-mismatch-bad) · [Correto](#ssr-no-hydration-mismatch-good) | Proibir valores não determinísticos que causam divergências na hidratação |
| [`type/no-floating-promises`](#type-no-floating-promises) | [Incorreto](#type-no-floating-promises-bad) · [Correto](#type-no-floating-promises-good) | Proibir Promises soltas (não tratadas) |
| [`type/no-reactivity-loss`](#type-no-reactivity-loss) | [Incorreto](#type-no-reactivity-loss-bad) · [Correto](#type-no-reactivity-loss-good) | Proibir cópias estáticas simples de valores reativos em atribuições e chamadas |
| [`type/no-unsafe-template-binding`](#type-no-unsafe-template-binding) | [Incorreto](#type-no-unsafe-template-binding-bad) · [Correto](#type-no-unsafe-template-binding-good) | Proibir vinculações de template que resultam em tipos inseguros |
| [`type/require-typed-emits`](#type-require-typed-emits) | [Incorreto](#type-require-typed-emits-bad) · [Correto](#type-require-typed-emits-good) | Exigir uma definição de tipo para defineEmits |
| [`type/require-typed-props`](#type-require-typed-props) | [Incorreto](#type-require-typed-props-bad) · [Correto](#type-require-typed-props-good) | Exigir uma definição de tipo para defineProps |
| [`type/strict-boolean-expressions`](#type-strict-boolean-expressions) | [Incorreto](#type-strict-boolean-expressions-bad) · [Correto](#type-strict-boolean-expressions-good) | Exigir expressões booleanas seguras nas condições de script e template |
| [`vapor/no-inline-template`](#vapor-no-inline-template) | [Incorreto](#vapor-no-inline-template-bad) · [Correto](#vapor-no-inline-template-good) | Proibir o atributo obsoleto inline-template |
| [`vapor/no-vue-lifecycle-events`](#vapor-no-vue-lifecycle-events) | [Incorreto](#vapor-no-vue-lifecycle-events-bad) · [Correto](#vapor-no-vue-lifecycle-events-good) | Proibir eventos de ciclo de vida @vue:xxx por elemento (não suportados em Vapor) |
| [`vapor/prefer-static-class`](#vapor-prefer-static-class) | [Incorreto](#vapor-prefer-static-class-bad) · [Correto](#vapor-prefer-static-class-good) | Preferir class estática a uma vinculação dinâmica de class para literais de string |
| [`vapor/require-vapor-attribute`](#vapor-require-vapor-attribute) | [Incorreto](#vapor-require-vapor-attribute-bad) · [Correto](#vapor-require-vapor-attribute-good) | Sugerir a adição do atributo vapor a script setup |
| [`vize:croquis/cf/array-mutation`](#vize-croquis-cf-array-mutation) | [Incorreto](#vize-croquis-cf-array-mutation-bad) · [Correto](#vize-croquis-cf-array-mutation-good) | Um array é alterado por índice, o que um array reativo não rastreia. |
| [`vize:croquis/cf/async-boundary`](#vize-croquis-cf-async-boundary) | [Incorreto](#vize-croquis-cf-async-boundary-bad) · [Correto](#vize-croquis-cf-async-boundary-good) | O estado reativo atravessa um limite assíncrono e pode ser observado desatualizado. |
| [`vize:croquis/cf/async-no-suspense`](#vize-croquis-cf-async-no-suspense) | [Incorreto](#vize-croquis-cf-async-no-suspense-bad) · [Correto](#vize-croquis-cf-async-no-suspense-good) | Um componente assíncrono é renderizado sem um limite Suspense. |
| [`vize:croquis/cf/browser-api-ssr`](#vize-croquis-cf-browser-api-ssr) | [Incorreto](#vize-croquis-cf-browser-api-ssr-bad) · [Correto](#vize-croquis-cf-browser-api-ssr-good) | Uma API exclusiva do navegador é usada onde o componente pode ser renderizado no servidor. |
| [`vize:croquis/cf/circular-dep`](#vize-croquis-cf-circular-dep) | [Incorreto](#vize-croquis-cf-circular-dep-bad) · [Correto](#vize-croquis-cf-circular-dep-good) | Os componentes importam uns aos outros em um ciclo. |
| [`vize:croquis/cf/circular-reactive-dependency`](#vize-croquis-cf-circular-reactive-dependency) | [Incorreto](#vize-croquis-cf-circular-reactive-dependency-bad) · [Correto](#vize-croquis-cf-circular-reactive-dependency-good) | Os cálculos reativos dependem uns dos outros em um ciclo. |
| [`vize:croquis/cf/closure-captures-reactive`](#vize-croquis-cf-closure-captures-reactive) | [Incorreto](#vize-croquis-cf-closure-captures-reactive-bad) · [Correto](#vize-croquis-cf-closure-captures-reactive-good) | Um fechamento captura um valor reativo e não verá atualizações posteriores. |
| [`vize:croquis/cf/composable-outside-setup`](#vize-croquis-cf-composable-outside-setup) | [Incorreto](#vize-croquis-cf-composable-outside-setup-bad) · [Correto](#vize-croquis-cf-composable-outside-setup-good) | Uma função de composição é chamada fora de `setup`. |
| [`vize:croquis/cf/computed-side-effects`](#vize-croquis-cf-computed-side-effects) | [Incorreto](#vize-croquis-cf-computed-side-effects-bad) · [Correto](#vize-croquis-cf-computed-side-effects-good) | Uma função de leitura computada escreve no estado ou produz outro efeito colateral. |
| [`vize:croquis/cf/deep-import`](#vize-croquis-cf-deep-import) | [Incorreto](#vize-croquis-cf-deep-import-bad) · [Correto](#vize-croquis-cf-deep-import-good) | Uma cadeia de importações é mais profunda do que o projeto permite. |
| [`vize:croquis/cf/destructuring-breaks-reactivity`](#vize-croquis-cf-destructuring-breaks-reactivity) | [Incorreto](#vize-croquis-cf-destructuring-breaks-reactivity-bad) · [Correto](#vize-croquis-cf-destructuring-breaks-reactivity-good) | Desestruturar um objeto reativo copia os campos e perde o rastreamento. |
| [`vize:croquis/cf/di-outside-setup`](#vize-croquis-cf-di-outside-setup) | [Incorreto](#vize-croquis-cf-di-outside-setup-bad) · [Correto](#vize-croquis-cf-di-outside-setup-good) | `provide` ou `inject` é chamado fora de `setup`. |
| [`vize:croquis/cf/dom-access-without-next-tick`](#vize-croquis-cf-dom-access-without-next-tick) | [Incorreto](#vize-croquis-cf-dom-access-without-next-tick-bad) · [Correto](#vize-croquis-cf-dom-access-without-next-tick-good) | O DOM é lido antes de o Vue aplicar a atualização. |
| [`vize:croquis/cf/duplicate-id`](#vize-croquis-cf-duplicate-id) | [Incorreto](#vize-croquis-cf-duplicate-id-bad) · [Correto](#vize-croquis-cf-duplicate-id-good) | O mesmo id de elemento é usado em mais de um componente. |
| [`vize:croquis/cf/event-listener-leak`](#vize-croquis-cf-event-listener-leak) | [Incorreto](#vize-croquis-cf-event-listener-leak-bad) · [Correto](#vize-croquis-cf-event-listener-leak-good) | Um ouvinte de eventos é registrado e nunca removido. |
| [`vize:croquis/cf/event-modifier`](#vize-croquis-cf-event-modifier) | [Incorreto](#vize-croquis-cf-event-modifier-bad) · [Correto](#vize-croquis-cf-event-modifier-good) | Um ouvinte de eventos usa um modificador que o evento emitido não suporta. |
| [`vize:croquis/cf/hydration-risk`](#vize-croquis-cf-hydration-risk) | [Incorreto](#vize-croquis-cf-hydration-risk-bad) · [Correto](#vize-croquis-cf-hydration-risk-good) | Este código de diagnóstico agrupa várias ocorrências de reatividade, incluindo uma prop copiada para uma ref. Ele não implica que toda expressão Date.now() seja detectada pela passagem de análise entre arquivos. |
| [`vize:croquis/cf/inherit-attrs-unused`](#vize-croquis-cf-inherit-attrs-unused) | [Incorreto](#vize-croquis-cf-inherit-attrs-unused-bad) · [Correto](#vize-croquis-cf-inherit-attrs-unused-good) | `inheritAttrs: false` está definido e o componente nunca lê os atributos. |
| [`vize:croquis/cf/inject-without-symbol`](#vize-croquis-cf-inject-without-symbol) | [Incorreto](#vize-croquis-cf-inject-without-symbol-bad) · [Correto](#vize-croquis-cf-inject-without-symbol-good) | `inject` usa uma chave comum em vez de um símbolo `InjectionKey`. |
| [`vize:croquis/cf/injected-async-mutation-race`](#vize-croquis-cf-injected-async-mutation-race) | [Incorreto](#vize-croquis-cf-injected-async-mutation-race-bad) · [Correto](#vize-croquis-cf-injected-async-mutation-race-good) | Um valor injetado é alterado por uma tarefa assíncrona sujeita a uma condição de corrida. |
| [`vize:croquis/cf/lifecycle-outside-setup`](#vize-croquis-cf-lifecycle-outside-setup) | [Incorreto](#vize-croquis-cf-lifecycle-outside-setup-bad) · [Correto](#vize-croquis-cf-lifecycle-outside-setup-good) | Um gancho de ciclo de vida é registrado fora de `setup`. |
| [`vize:croquis/cf/lifecycle-without-cleanup`](#vize-croquis-cf-lifecycle-without-cleanup) | [Incorreto](#vize-croquis-cf-lifecycle-without-cleanup-bad) · [Correto](#vize-croquis-cf-lifecycle-without-cleanup-good) | Um gancho de ciclo de vida inicia um trabalho e nunca faz sua limpeza. |
| [`vize:croquis/cf/missing-required-prop`](#vize-croquis-cf-missing-required-prop) | [Incorreto](#vize-croquis-cf-missing-required-prop-bad) · [Correto](#vize-croquis-cf-missing-required-prop-good) | Uma prop obrigatória não é passada. |
| [`vize:croquis/cf/missing-suspense`](#vize-croquis-cf-missing-suspense) | [Incorreto](#vize-croquis-cf-missing-suspense-bad) · [Correto](#vize-croquis-cf-missing-suspense-good) | Uma dependência assíncrona é usada fora de um limite Suspense. |
| [`vize:croquis/cf/module-scope-reactive`](#vize-croquis-cf-module-scope-reactive) | [Incorreto](#vize-croquis-cf-module-scope-reactive-bad) · [Correto](#vize-croquis-cf-module-scope-reactive-good) | O estado reativo é criado no escopo do módulo e compartilhado por todos os chamadores. |
| [`vize:croquis/cf/multi-root-attrs`](#vize-croquis-cf-multi-root-attrs) | [Incorreto](#vize-croquis-cf-multi-root-attrs-bad) · [Correto](#vize-croquis-cf-multi-root-attrs-good) | Um componente com múltiplas raízes recebe atributos e não tem onde colocá-los. |
| [`vize:croquis/cf/mutated-after-escape`](#vize-croquis-cf-mutated-after-escape) | [Incorreto](#vize-croquis-cf-mutated-after-escape-bad) · [Correto](#vize-croquis-cf-mutated-after-escape-good) | Um objeto reativo é alterado depois de escapar de seu proprietário. |
| [`vize:croquis/cf/non-reactive-provide`](#vize-croquis-cf-non-reactive-provide) | [Incorreto](#vize-croquis-cf-non-reactive-provide-bad) · [Correto](#vize-croquis-cf-non-reactive-provide-good) | Um valor fornecido não é reativo, portanto os descendentes não verão atualizações. |
| [`vize:croquis/cf/non-unique-id`](#vize-croquis-cf-non-unique-id) | [Incorreto](#vize-croquis-cf-non-unique-id-bad) · [Correto](#vize-croquis-cf-non-unique-id-good) | Um id de elemento dentro de um laço não é único por item. |
| [`vize:croquis/cf/object-identity-comparison`](#vize-croquis-cf-object-identity-comparison) | [Incorreto](#vize-croquis-cf-object-identity-comparison-bad) · [Correto](#vize-croquis-cf-object-identity-comparison-good) | Um objeto reativo é comparado por identidade, que muda ao remover os invólucros. |
| [`vize:croquis/cf/pinia-getter`](#vize-croquis-cf-pinia-getter) | [Incorreto](#vize-croquis-cf-pinia-getter-bad) · [Correto](#vize-croquis-cf-pinia-getter-good) | Uma função de leitura do Pinia é lida sem `storeToRefs`, portanto não permanecerá reativa. |
| [`vize:croquis/cf/prop-type-mismatch`](#vize-croquis-cf-prop-type-mismatch) | [Incorreto](#vize-croquis-cf-prop-type-mismatch-bad) · [Correto](#vize-croquis-cf-prop-type-mismatch-good) | O valor de uma prop passada não corresponde ao tipo declarado. |
| [`vize:croquis/cf/provide-inject-type`](#vize-croquis-cf-provide-inject-type) | [Incorreto](#vize-croquis-cf-provide-inject-type-bad) · [Correto](#vize-croquis-cf-provide-inject-type-good) | Um valor fornecido e sua injeção não têm o mesmo tipo. |
| [`vize:croquis/cf/provide-without-symbol`](#vize-croquis-cf-provide-without-symbol) | [Incorreto](#vize-croquis-cf-provide-without-symbol-bad) · [Correto](#vize-croquis-cf-provide-without-symbol-good) | `provide` usa uma chave comum em vez de um símbolo `InjectionKey`. |
| [`vize:croquis/cf/reactive-export`](#vize-croquis-cf-reactive-export) | [Incorreto](#vize-croquis-cf-reactive-export-bad) · [Correto](#vize-croquis-cf-reactive-export-good) | O estado reativo é exportado pelo módulo. |
| [`vize:croquis/cf/reactivity-outside-setup`](#vize-croquis-cf-reactivity-outside-setup) | [Incorreto](#vize-croquis-cf-reactivity-outside-setup-bad) · [Correto](#vize-croquis-cf-reactivity-outside-setup-good) | Uma API reativa é chamada fora de `setup`. |
| [`vize:croquis/cf/reassignment-breaks-reactivity`](#vize-croquis-cf-reassignment-breaks-reactivity) | [Incorreto](#vize-croquis-cf-reassignment-breaks-reactivity-bad) · [Correto](#vize-croquis-cf-reassignment-breaks-reactivity-good) | Reatribuir uma variável reativa a substitui por um valor simples. |
| [`vize:croquis/cf/reference-escapes-scope`](#vize-croquis-cf-reference-escapes-scope) | [Incorreto](#vize-croquis-cf-reference-escapes-scope-bad) · [Correto](#vize-croquis-cf-reference-escapes-scope-good) | Uma referência reativa escapa do escopo responsável por seu ciclo de vida. |
| [`vize:croquis/cf/setup-context-violation`](#vize-croquis-cf-setup-context-violation) | [Incorreto](#vize-croquis-cf-setup-context-violation-bad) · [Correto](#vize-croquis-cf-setup-context-violation-good) | O contexto de setup é usado de uma forma que o Vue não permite. |
| [`vize:croquis/cf/shallow-deep-access`](#vize-croquis-cf-shallow-deep-access) | [Incorreto](#vize-croquis-cf-shallow-deep-access-bad) · [Correto](#vize-croquis-cf-shallow-deep-access-good) | Uma propriedade profunda de um valor `shallowReactive` ou `shallowRef` é lida como se fosse rastreada. |
| [`vize:croquis/cf/spread-breaks-reactivity`](#vize-croquis-cf-spread-breaks-reactivity) | [Incorreto](#vize-croquis-cf-spread-breaks-reactivity-bad) · [Correto](#vize-croquis-cf-spread-breaks-reactivity-good) | Espalhar um objeto reativo copia seus valores e perde o rastreamento. |
| [`vize:croquis/cf/suspense-no-fallback`](#vize-croquis-cf-suspense-no-fallback) | [Incorreto](#vize-croquis-cf-suspense-no-fallback-bad) · [Correto](#vize-croquis-cf-suspense-no-fallback-good) | `<Suspense>` não tem conteúdo alternativo. |
| [`vize:croquis/cf/template-ref-timing`](#vize-croquis-cf-template-ref-timing) | [Incorreto](#vize-croquis-cf-template-ref-timing-bad) · [Correto](#vize-croquis-cf-template-ref-timing-good) | Uma ref de template é lida antes da montagem do componente. |
| [`vize:croquis/cf/toraw-mutation`](#vize-croquis-cf-toraw-mutation) | [Incorreto](#vize-croquis-cf-toraw-mutation-bad) · [Correto](#vize-croquis-cf-toraw-mutation-good) | `toRaw` é usado e o objeto bruto é alterado em seguida. |
| [`vize:croquis/cf/uncaught-error`](#vize-croquis-cf-uncaught-error) | [Incorreto](#vize-croquis-cf-uncaught-error-bad) · [Correto](#vize-croquis-cf-uncaught-error-good) | Um componente pode lançar um erro e nenhum limite de erro o captura. |
| [`vize:croquis/cf/undeclared-emit`](#vize-croquis-cf-undeclared-emit) | [Incorreto](#vize-croquis-cf-undeclared-emit-bad) · [Correto](#vize-croquis-cf-undeclared-emit-good) | O componente emite um evento que não foi declarado. |
| [`vize:croquis/cf/undeclared-prop`](#vize-croquis-cf-undeclared-prop) | [Incorreto](#vize-croquis-cf-undeclared-prop-bad) · [Correto](#vize-croquis-cf-undeclared-prop-good) | Um pai passa uma prop que o filho não declara. |
| [`vize:croquis/cf/undefined-slot`](#vize-croquis-cf-undefined-slot) | [Incorreto](#vize-croquis-cf-undefined-slot-bad) · [Correto](#vize-croquis-cf-undefined-slot-good) | Um pai preenche um slot que o filho não expõe. |
| [`vize:croquis/cf/unhandled-event`](#vize-croquis-cf-unhandled-event) | [Incorreto](#vize-croquis-cf-unhandled-event-bad) · [Correto](#vize-croquis-cf-unhandled-event-good) | Um filho emite um evento que nenhum pai trata. |
| [`vize:croquis/cf/unmatched-inject`](#vize-croquis-cf-unmatched-inject) | [Incorreto](#vize-croquis-cf-unmatched-inject-bad) · [Correto](#vize-croquis-cf-unmatched-inject-good) | `inject` nomeia uma chave que nenhum ancestral fornece. |
| [`vize:croquis/cf/unmatched-listener`](#vize-croquis-cf-unmatched-listener) | [Incorreto](#vize-croquis-cf-unmatched-listener-bad) · [Correto](#vize-croquis-cf-unmatched-listener-good) | Um pai escuta um evento que o filho não emite. |
| [`vize:croquis/cf/unregistered-component`](#vize-croquis-cf-unregistered-component) | [Incorreto](#vize-croquis-cf-unregistered-component-bad) · [Correto](#vize-croquis-cf-unregistered-component-good) | Um template usa um componente que não foi registrado nem importado. |
| [`vize:croquis/cf/unresolved-import`](#vize-croquis-cf-unresolved-import) | [Incorreto](#vize-croquis-cf-unresolved-import-bad) · [Correto](#vize-croquis-cf-unresolved-import-good) | Uma importação não resolve para um módulo. |
| [`vize:croquis/cf/unused-attrs`](#vize-croquis-cf-unused-attrs) | [Incorreto](#vize-croquis-cf-unused-attrs-bad) · [Correto](#vize-croquis-cf-unused-attrs-good) | Atributos de herança automática são passados para um componente com múltiplas raízes que não os usa. |
| [`vize:croquis/cf/unused-emit`](#vize-croquis-cf-unused-emit) | [Incorreto](#vize-croquis-cf-unused-emit-bad) · [Correto](#vize-croquis-cf-unused-emit-good) | Um evento declarado nunca é emitido. |
| [`vize:croquis/cf/unused-provide`](#vize-croquis-cf-unused-provide) | [Incorreto](#vize-croquis-cf-unused-provide-bad) · [Correto](#vize-croquis-cf-unused-provide-good) | Uma chave fornecida nunca é injetada. |
| [`vize:croquis/cf/value-extraction-breaks-reactivity`](#vize-croquis-cf-value-extraction-breaks-reactivity) | [Incorreto](#vize-croquis-cf-value-extraction-breaks-reactivity-bad) · [Correto](#vize-croquis-cf-value-extraction-breaks-reactivity-good) | Ler um valor reativo para uma variável local perde as atualizações posteriores. |
| [`vize:croquis/cf/watch-can-be-computed`](#vize-croquis-cf-watch-can-be-computed) | [Incorreto](#vize-croquis-cf-watch-can-be-computed-bad) · [Correto](#vize-croquis-cf-watch-can-be-computed-good) | Um observador apenas copia um valor para o estado e pode ser um valor computado. |
| [`vize:croquis/cf/watcheffect-async`](#vize-croquis-cf-watcheffect-async) | [Incorreto](#vize-croquis-cf-watcheffect-async-bad) · [Correto](#vize-croquis-cf-watcheffect-async-good) | `watchEffect` inicia uma tarefa assíncrona e não consegue limpar a execução anterior. |
| [`vize:croquis/cf/watcher-outside-setup`](#vize-croquis-cf-watcher-outside-setup) | [Incorreto](#vize-croquis-cf-watcher-outside-setup-bad) · [Correto](#vize-croquis-cf-watcher-outside-setup-good) | `watch` ou `watchEffect` é chamado fora de `setup`. |
| [`vue/a11y-img-alt`](#vue-a11y-img-alt) | [Incorreto](#vue-a11y-img-alt-bad) · [Correto](#vue-a11y-img-alt-good) | Exigir o atributo alt nas imagens para garantir acessibilidade |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [Incorreto](#vue-attribute-hyphenation-bad) · [Correto](#vue-attribute-hyphenation-good) | Aplicar um padrão de nomes de atributos em componentes personalizados |
| [`vue/attribute-order`](#vue-attribute-order) | [Incorreto](#vue-attribute-order-bad) · [Correto](#vue-attribute-order-good) | Aplicar uma ordem consistente aos atributos |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [Incorreto](#vue-component-definition-name-casing-bad) · [Correto](#vue-component-definition-name-casing-good) | Exigir PascalCase ou kebab-case nos nomes de definição de componentes |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [Incorreto](#vue-component-name-in-template-casing-bad) · [Correto](#vue-component-name-in-template-casing-good) | Aplicar um padrão específico de maiúsculas e minúsculas aos nomes de componentes nos templates |
| [`vue/cross-file-attrs-fallthrough`](#vue-cross-file-attrs-fallthrough) | [Incorreto](#vue-cross-file-attrs-fallthrough-bad) · [Correto](#vue-cross-file-attrs-fallthrough-good) | Um pai passa atributos para um filho resolvido cuja raiz não pode herdá-los e que não usa $attrs explicitamente. |
| [`vue/html-button-has-type`](#vue-html-button-has-type) | [Incorreto](#vue-html-button-has-type-bad) · [Correto](#vue-html-button-has-type-good) | Exigir um type explícito e válido nos elementos button |
| [`vue/html-quotes`](#vue-html-quotes) | [Incorreto](#vue-html-quotes-bad) · [Correto](#vue-html-quotes-good) | Aplicar um padrão de aspas aos atributos HTML |
| [`vue/html-self-closing`](#vue-html-self-closing) | [Incorreto](#vue-html-self-closing-bad) · [Correto](#vue-html-self-closing-good) | Aplicar um padrão de tags com fechamento automático |
| [`vue/max-template-complexity`](#vue-max-template-complexity) | [Incorreto](#vue-max-template-complexity-bad) · [Correto](#vue-max-template-complexity-good) | Limitar a complexidade do próprio template de um componente, tanto ciclomática quanto cognitiva |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [Incorreto](#vue-multi-word-component-names-bad) · [Correto](#vue-multi-word-component-names-good) | Exigir nomes de componentes com mais de uma palavra |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [Incorreto](#vue-mustache-interpolation-spacing-bad) · [Correto](#vue-mustache-interpolation-spacing-good) | Aplicar espaçamento consistente dentro das interpolações com chaves duplas |
| [`vue/no-array-index-key`](#vue-no-array-index-key) | [Incorreto](#vue-no-array-index-key-bad) · [Correto](#vue-no-array-index-key-good) | Proibir o uso direto da variável de índice de v-for como :key |
| [`vue/no-bare-strings-in-template`](#vue-no-bare-strings-in-template) | [Incorreto](#vue-no-bare-strings-in-template-bad) · [Correto](#vue-no-bare-strings-in-template-good) | Proibir texto legível por pessoas diretamente no template quando ele deve ser internacionalizado |
| [`vue/no-boolean-attr-value`](#vue-no-boolean-attr-value) | [Incorreto](#vue-no-boolean-attr-value-bad) · [Correto](#vue-no-boolean-attr-value-good) | Proibir valores explícitos em atributos HTML booleanos |
| [`vue/no-child-content`](#vue-no-child-content) | [Incorreto](#vue-no-child-content-bad) · [Correto](#vue-no-child-content-good) | Proibir conteúdo filho ao usar v-html ou v-text |
| [`vue/no-deprecated-filter`](#vue-no-deprecated-filter) | [Incorreto](#vue-no-deprecated-filter-bad) · [Correto](#vue-no-deprecated-filter-good) | Proibir a sintaxe obsoleta de filtros do Vue 2 com o operador de barra vertical |
| [`vue/no-deprecated-functional-template`](#vue-no-deprecated-functional-template) | [Incorreto](#vue-no-deprecated-functional-template-bad) · [Correto](#vue-no-deprecated-functional-template-good) | Proibir o atributo `functional` no `<template>` de um SFC |
| [`vue/no-deprecated-html-element-is`](#vue-no-deprecated-html-element-is) | [Incorreto](#vue-no-deprecated-html-element-is-bad) · [Correto](#vue-no-deprecated-html-element-is-good) | Proibir o atributo `is` em elementos HTML nativos |
| [`vue/no-deprecated-inline-template`](#vue-no-deprecated-inline-template) | [Incorreto](#vue-no-deprecated-inline-template-bad) · [Correto](#vue-no-deprecated-inline-template-good) | Proibir o atributo obsoleto `inline-template` |
| [`vue/no-deprecated-router-link-tag-prop`](#vue-no-deprecated-router-link-tag-prop) | [Incorreto](#vue-no-deprecated-router-link-tag-prop-bad) · [Correto](#vue-no-deprecated-router-link-tag-prop-good) | Proibir a prop `tag` em &lt;router-link&gt; |
| [`vue/no-deprecated-scope-attribute`](#vue-no-deprecated-scope-attribute) | [Incorreto](#vue-no-deprecated-scope-attribute-bad) · [Correto](#vue-no-deprecated-scope-attribute-good) | Proibir o atributo obsoleto `scope` em &lt;template&gt; |
| [`vue/no-deprecated-slot-attribute`](#vue-no-deprecated-slot-attribute) | [Incorreto](#vue-no-deprecated-slot-attribute-bad) · [Correto](#vue-no-deprecated-slot-attribute-good) | Proibir o atributo obsoleto `slot` |
| [`vue/no-deprecated-slot-scope-attribute`](#vue-no-deprecated-slot-scope-attribute) | [Incorreto](#vue-no-deprecated-slot-scope-attribute-bad) · [Correto](#vue-no-deprecated-slot-scope-attribute-good) | Proibir o atributo obsoleto `slot-scope` |
| [`vue/no-deprecated-v-bind-sync`](#vue-no-deprecated-v-bind-sync) | [Incorreto](#vue-no-deprecated-v-bind-sync-bad) · [Correto](#vue-no-deprecated-v-bind-sync-good) | Proibir o modificador obsoleto `.sync` em `v-bind` |
| [`vue/no-deprecated-v-on-native-modifier`](#vue-no-deprecated-v-on-native-modifier) | [Incorreto](#vue-no-deprecated-v-on-native-modifier-bad) · [Correto](#vue-no-deprecated-v-on-native-modifier-good) | Proibir o modificador obsoleto `.native` em `v-on` |
| [`vue/no-deprecated-v-on-number-modifiers`](#vue-no-deprecated-v-on-number-modifiers) | [Incorreto](#vue-no-deprecated-v-on-number-modifiers-bad) · [Correto](#vue-no-deprecated-v-on-number-modifiers-good) | Proibir modificadores numéricos obsoletos de `keyCode` em `v-on` |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [Incorreto](#vue-no-dupe-v-else-if-bad) · [Correto](#vue-no-dupe-v-else-if-good) | Proibir condições duplicadas em cadeias de `v-if` / `v-else-if` |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [Incorreto](#vue-no-duplicate-attributes-bad) · [Correto](#vue-no-duplicate-attributes-good) | Proibir atributos duplicados no mesmo elemento |
| [`vue/no-empty-component-block`](#vue-no-empty-component-block) | [Incorreto](#vue-no-empty-component-block-bad) · [Correto](#vue-no-empty-component-block-good) | Proibir blocos vazios em SFCs |
| [`vue/no-inline-style`](#vue-no-inline-style) | [Incorreto](#vue-no-inline-style-bad) · [Correto](#vue-no-inline-style-good) | Desencorajar o uso de atributos de estilo inline |
| [`vue/no-invalid-html-attribute`](#vue-no-invalid-html-attribute) | [Incorreto](#vue-no-invalid-html-attribute-bad) · [Correto](#vue-no-invalid-html-attribute-good) | Proibir valores estáticos inválidos para atributos HTML |
| [`vue/no-lone-template`](#vue-no-lone-template) | [Incorreto](#vue-no-lone-template-bad) · [Correto](#vue-no-lone-template-good) | Proibir elementos `<template>` desnecessários |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [Incorreto](#vue-no-multi-spaces-bad) · [Correto](#vue-no-multi-spaces-good) | Proibir vários espaços consecutivos |
| [`vue/no-multiple-objects-in-class`](#vue-no-multiple-objects-in-class) | [Incorreto](#vue-no-multiple-objects-in-class-bad) · [Correto](#vue-no-multiple-objects-in-class-good) | Proibir vários objetos literais dentro de uma vinculação de array em :class |
| [`vue/no-multiple-template-root`](#vue-no-multiple-template-root) | [Incorreto](#vue-no-multiple-template-root-bad) · [Correto](#vue-no-multiple-template-root-good) | Proibir vários nós raiz em um template |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [Incorreto](#vue-no-mutating-props-bad) · [Correto](#vue-no-mutating-props-good) | Proibir a mutação de props de componentes |
| [`vue/no-negated-v-if-condition`](#vue-no-negated-v-if-condition) | [Incorreto](#vue-no-negated-v-if-condition-bad) · [Correto](#vue-no-negated-v-if-condition-good) | Proibir uma condição negada em v-if quando a cadeia tiver v-else |
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [Incorreto](#vue-no-non-component-keep-alive-child-bad) · [Correto](#vue-no-non-component-keep-alive-child-good) | Proibir invólucros de elementos comuns diretamente abaixo de `<KeepAlive>` |
| [`vue/no-preprocessor-lang`](#vue-no-preprocessor-lang) | [Incorreto](#vue-no-preprocessor-lang-bad) · [Correto](#vue-no-preprocessor-lang-good) | Desencorajar o uso de preprocessadores CSS em favor de CSS moderno |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [Incorreto](#vue-no-reserved-component-names-bad) · [Correto](#vue-no-reserved-component-names-good) | Proibir o uso de nomes reservados como nomes de componentes |
| [`vue/no-root-v-if`](#vue-no-root-v-if) | [Incorreto](#vue-no-root-v-if-bad) · [Correto](#vue-no-root-v-if-good) | Proibir v-if no único elemento raiz de um template |
| [`vue/no-script-non-standard-lang`](#vue-no-script-non-standard-lang) | [Incorreto](#vue-no-script-non-standard-lang-bad) · [Correto](#vue-no-script-non-standard-lang-good) | Desencorajar valores não padronizados de lang em scripts |
| [`vue/no-src-attribute`](#vue-no-src-attribute) | [Incorreto](#vue-no-src-attribute-bad) · [Correto](#vue-no-src-attribute-good) | Desencorajar o atributo src em blocos de SFCs |
| [`vue/no-static-inline-styles`](#vue-no-static-inline-styles) | [Incorreto](#vue-no-static-inline-styles-bad) · [Correto](#vue-no-static-inline-styles-good) | Proibir atributos estáticos de estilo inline |
| [`vue/no-template-key`](#vue-no-template-key) | [Incorreto](#vue-no-template-key-bad) · [Correto](#vue-no-template-key-good) | Proibir o atributo `key` em `<template>` |
| [`vue/no-template-lang`](#vue-no-template-lang) | [Incorreto](#vue-no-template-lang-bad) · [Correto](#vue-no-template-lang-good) | Desencorajar o atributo lang no bloco template |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [Incorreto](#vue-no-template-shadow-bad) · [Correto](#vue-no-template-shadow-good) | Proibir nomes de variáveis que ocultam variáveis de um escopo externo |
| [`vue/no-template-target-blank`](#vue-no-template-target-blank) | [Incorreto](#vue-no-template-target-blank-bad) · [Correto](#vue-no-template-target-blank-good) | Proibir target="_blank" sem rel="noopener noreferrer" |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [Incorreto](#vue-no-textarea-mustache-bad) · [Correto](#vue-no-textarea-mustache-good) | Proibir interpolação com chaves duplas em `<textarea>` |
| [`vue/no-undefined-refs`](#vue-no-undefined-refs) | [Incorreto](#vue-no-undefined-refs-bad) · [Correto](#vue-no-undefined-refs-good) | Proibir referências a variáveis não definidas nos templates |
| [`vue/no-unsafe-url`](#vue-no-unsafe-url) | [Incorreto](#vue-no-unsafe-url-bad) · [Correto](#vue-no-unsafe-url-good) | Alertar sobre vinculações de URL potencialmente inseguras |
| [`vue/no-unsandboxed-iframe`](#vue-no-unsandboxed-iframe) | [Incorreto](#vue-no-unsandboxed-iframe-bad) · [Correto](#vue-no-unsandboxed-iframe-good) | Exigir um atributo sandbox nos elementos iframe |
| [`vue/no-unused-components`](#vue-no-unused-components) | [Incorreto](#vue-no-unused-components-bad) · [Correto](#vue-no-unused-components-good) | Proibir o registro de componentes não usados nos templates |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [Incorreto](#vue-no-unused-properties-bad) · [Correto](#vue-no-unused-properties-good) | Proibir propriedades não usadas definidas em defineProps |
| [`vue/no-unused-refs`](#vue-no-unused-refs) | [Incorreto](#vue-no-unused-refs-bad) · [Correto](#vue-no-unused-refs-good) | Reportar refs de template (ref="x") nunca referenciadas em &lt;script&gt; |
| [`vue/no-unused-setup-bindings`](#vue-no-unused-setup-bindings) | [Incorreto](#vue-no-unused-setup-bindings-bad) · [Correto](#vue-no-unused-setup-bindings-good) | Proibir variáveis de script setup que nunca são lidas |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [Incorreto](#vue-no-unused-vars-bad) · [Correto](#vue-no-unused-vars-good) | Proibir definições de variáveis não usadas nas diretivas v-for e v-slot |
| [`vue/no-use-v-else-with-v-for`](#vue-no-use-v-else-with-v-for) | [Incorreto](#vue-no-use-v-else-with-v-for-bad) · [Correto](#vue-no-use-v-else-with-v-for-good) | Proibir `v-else-if` ou `v-else` no mesmo elemento que `v-for` |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [Incorreto](#vue-no-use-v-if-with-v-for-bad) · [Correto](#vue-no-use-v-if-with-v-for-good) | Proibir `v-if` no mesmo elemento que `v-for` |
| [`vue/no-useless-mustaches`](#vue-no-useless-mustaches) | [Incorreto](#vue-no-useless-mustaches-bad) · [Correto](#vue-no-useless-mustaches-good) | Proibir interpolação com chaves duplas cuja expressão seja uma string literal constante |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [Incorreto](#vue-no-useless-template-attributes-bad) · [Correto](#vue-no-useless-template-attributes-good) | Proibir atributos sem efeito em elementos `<template>` |
| [`vue/no-useless-v-bind`](#vue-no-useless-v-bind) | [Incorreto](#vue-no-useless-v-bind-bad) · [Correto](#vue-no-useless-v-bind-good) | Proibir um v-bind cujo valor seja uma string literal simples |
| [`vue/no-v-for-template-key-on-child`](#vue-no-v-for-template-key-on-child) | [Incorreto](#vue-no-v-for-template-key-on-child-bad) · [Correto](#vue-no-v-for-template-key-on-child-good) | Proibir `key` no filho de um `<template v-for>` |
| [`vue/no-v-html`](#vue-no-v-html) | [Incorreto](#vue-no-v-html-bad) · [Correto](#vue-no-v-html-good) | Alertar sobre v-html para prevenir vulnerabilidades XSS |
| [`vue/no-v-text`](#vue-no-v-text) | [Incorreto](#vue-no-v-text-bad) · [Correto](#vue-no-v-text-good) | Proibir a diretiva v-text; preferir interpolação com chaves duplas |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [Incorreto](#vue-no-v-text-v-html-on-component-bad) · [Correto](#vue-no-v-text-v-html-on-component-good) | Proibir v-text / v-html em elementos de componente |
| [`vue/permitted-contents`](#vue-permitted-contents) | [Incorreto](#vue-permitted-contents-bad) · [Correto](#vue-permitted-contents-good) | Aplicar as regras do modelo de conteúdo HTML |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [Incorreto](#vue-prefer-props-shorthand-bad) · [Correto](#vue-prefer-props-shorthand-good) | Recomendar sintaxe abreviada para props (Vue 3.4+) |
| [`vue/prefer-true-attribute-shorthand`](#vue-prefer-true-attribute-shorthand) | [Incorreto](#vue-prefer-true-attribute-shorthand-bad) · [Correto](#vue-prefer-true-attribute-shorthand-good) | Preferir a forma abreviada para um atributo booleano vinculado a `true` |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [Incorreto](#vue-prop-name-casing-bad) · [Correto](#vue-prop-name-casing-good) | Aplicar um padrão de maiúsculas e minúsculas aos nomes de props declaradas |
| [`vue/require-component-is`](#vue-require-component-is) | [Incorreto](#vue-require-component-is-bad) · [Correto](#vue-require-component-is-good) | Exigir `v-bind:is` em elementos `<component>` |
| [`vue/require-component-registration`](#vue-require-component-registration) | [Incorreto](#vue-require-component-registration-bad) · [Correto](#vue-require-component-registration-good) | Exigir importação ou registro explícito de componentes |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [Incorreto](#vue-require-scoped-style-bad) · [Correto](#vue-require-scoped-style-good) | Exigir o atributo scoped nas tags style |
| [`vue/require-toggle-inside-transition`](#vue-require-toggle-inside-transition) | [Incorreto](#vue-require-toggle-inside-transition-bad) · [Correto](#vue-require-toggle-inside-transition-good) | Exigir uma alternância no elemento envolvido por `<transition>` |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [Incorreto](#vue-require-v-for-key-bad) · [Correto](#vue-require-v-for-key-good) | Exigir `v-bind:key` nas diretivas `v-for` |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [Incorreto](#vue-scoped-event-names-bad) · [Correto](#vue-scoped-event-names-good) | Recomendar nomes de eventos com escopo no formato context:event |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [Incorreto](#vue-sfc-element-order-bad) · [Correto](#vue-sfc-element-order-good) | Aplicar uma ordem consistente aos elementos de nível superior dos SFCs |
| [`vue/single-style-block`](#vue-single-style-block) | [Incorreto](#vue-single-style-block-bad) · [Correto](#vue-single-style-block-good) | Recomendar um único bloco style |
| [`vue/slot-name-casing`](#vue-slot-name-casing) | [Incorreto](#vue-slot-name-casing-bad) · [Correto](#vue-slot-name-casing-good) | Exigir kebab-case nos slots nomeados usados por v-slot |
| [`vue/this-in-template`](#vue-this-in-template) | [Incorreto](#vue-this-in-template-bad) · [Correto](#vue-this-in-template-good) | Proibir `this.` nas expressões do template |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [Incorreto](#vue-use-unique-element-ids-bad) · [Correto](#vue-use-unique-element-ids-good) | Exigir IDs de elementos únicos por meio de useId(), em vez de literais estáticos |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [Incorreto](#vue-use-v-on-exact-bad) · [Correto](#vue-use-v-on-exact-good) | Exigir o modificador `.exact` em `v-on` quando houver manipuladores baseados em modificadores |
| [`vue/v-bind-style`](#vue-v-bind-style) | [Incorreto](#vue-v-bind-style-bad) · [Correto](#vue-v-bind-style-good) | Aplicar um padrão de sintaxe à diretiva `v-bind` |
| [`vue/v-on-event-hyphenation`](#vue-v-on-event-hyphenation) | [Incorreto](#vue-v-on-event-hyphenation-bad) · [Correto](#vue-v-on-event-hyphenation-good) | Exigir hífens nos nomes de eventos personalizados em v-on de componentes |
| [`vue/v-on-handler-style`](#vue-v-on-handler-style) | [Incorreto](#vue-v-on-handler-style-bad) · [Correto](#vue-v-on-handler-style-good) | Exigir manipuladores de v-on escritos como referência a método ou função inline |
| [`vue/v-on-style`](#vue-v-on-style) | [Incorreto](#vue-v-on-style-bad) · [Correto](#vue-v-on-style-good) | Aplicar um padrão de sintaxe à diretiva `v-on` |
| [`vue/v-slot-style`](#vue-v-slot-style) | [Incorreto](#vue-v-slot-style-bad) · [Correto](#vue-v-slot-style-good) | Aplicar um padrão de sintaxe à diretiva `v-slot` |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [Incorreto](#vue-valid-attribute-name-bad) · [Correto](#vue-valid-attribute-name-good) | Exigir nomes de atributos válidos |
| [`vue/valid-template-root`](#vue-valid-template-root) | [Incorreto](#vue-valid-template-root-bad) · [Correto](#vue-valid-template-root-good) | Exigir uma raiz `<template>` válida para a semântica de fragmentos do Vue 3 |
| [`vue/valid-v-bind`](#vue-valid-v-bind) | [Incorreto](#vue-valid-v-bind-bad) · [Correto](#vue-valid-v-bind-good) | Exigir diretivas `v-bind` válidas |
| [`vue/valid-v-cloak`](#vue-valid-v-cloak) | [Incorreto](#vue-valid-v-cloak-bad) · [Correto](#vue-valid-v-cloak-good) | Exigir diretivas `v-cloak` válidas |
| [`vue/valid-v-else`](#vue-valid-v-else) | [Incorreto](#vue-valid-v-else-bad) · [Correto](#vue-valid-v-else-good) | Exigir diretivas `v-else` válidas |
| [`vue/valid-v-for`](#vue-valid-v-for) | [Incorreto](#vue-valid-v-for-bad) · [Correto](#vue-valid-v-for-good) | Exigir diretivas `v-for` válidas |
| [`vue/valid-v-html`](#vue-valid-v-html) | [Incorreto](#vue-valid-v-html-bad) · [Correto](#vue-valid-v-html-good) | Exigir diretivas `v-html` válidas |
| [`vue/valid-v-if`](#vue-valid-v-if) | [Incorreto](#vue-valid-v-if-bad) · [Correto](#vue-valid-v-if-good) | Exigir diretivas `v-if` válidas |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [Incorreto](#vue-valid-v-memo-bad) · [Correto](#vue-valid-v-memo-good) | Exigir diretivas `v-memo` válidas |
| [`vue/valid-v-model`](#vue-valid-v-model) | [Incorreto](#vue-valid-v-model-bad) · [Correto](#vue-valid-v-model-good) | Exigir diretivas `v-model` válidas |
| [`vue/valid-v-on`](#vue-valid-v-on) | [Incorreto](#vue-valid-v-on-bad) · [Correto](#vue-valid-v-on-good) | Exigir diretivas `v-on` válidas |
| [`vue/valid-v-once`](#vue-valid-v-once) | [Incorreto](#vue-valid-v-once-bad) · [Correto](#vue-valid-v-once-good) | Exigir diretivas `v-once` válidas |
| [`vue/valid-v-show`](#vue-valid-v-show) | [Incorreto](#vue-valid-v-show-bad) · [Correto](#vue-valid-v-show-good) | Exigir diretivas `v-show` válidas |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [Incorreto](#vue-valid-v-slot-bad) · [Correto](#vue-valid-v-slot-good) | Exigir diretivas `v-slot` válidas |
| [`vue/valid-v-text`](#vue-valid-v-text) | [Incorreto](#vue-valid-v-text-bad) · [Correto](#vue-valid-v-text-good) | Exigir diretivas `v-text` válidas |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [Incorreto](#vue-warn-custom-block-bad) · [Correto](#vue-warn-custom-block-good) | Alertar sobre blocos personalizados em arquivos SFC |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [Incorreto](#vue-warn-custom-directive-bad) · [Correto](#vue-warn-custom-directive-good) | Alertar sobre diretivas personalizadas que precisam de registro |

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

### `petite-vue/no-unsupported-directive`

Proibir diretivas que o petite-vue não aceita

[Incorreto](#petite-vue-no-unsupported-directive-bad) · [Correto](#petite-vue-no-unsupported-directive-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Documentos HTML detectados como petite-vue; SFCs Vue comuns ficam fora do escopo desta regra.  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-no-unsupported-directive-bad"></span>

**Incorreto**

`v-memo`, `v-slot:header` e a diretiva personalizada `v-my-directive` não constam na lista de diretivas aceitas pelo petite-vue. O script do petite-vue identifica este HTML como o dialeto pertinente.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-no-unsupported-directive-good"></span>

**Correto**

A substituição usa a sintaxe aceita de `v-scope`, `v-effect`, `v-if`, `v-bind` e `v-on`, em vez de depender de diretivas não aceitas.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) · [Todas as regras](all.md)

### `petite-vue/valid-v-effect`

Exigir uma expressão não vazia em v-effect

[Incorreto](#petite-vue-valid-v-effect-bad) · [Correto](#petite-vue-valid-v-effect-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Documentos HTML detectados como petite-vue; SFCs Vue comuns ficam fora do escopo desta regra.  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-effect": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-effect-bad"></span>

**Incorreto**

Cada `v-effect` não tem uma expressão executável: seu valor está ausente, vazio ou contém apenas espaços em branco.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-effect-good"></span>

**Correto**

Ambos os valores de `v-effect` contêm uma expressão: um atualiza `el.textContent` e o outro incrementa `count`. Esta regra verifica se há uma expressão não vazia, e não a lógica de negócio do efeito.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) · [Todas as regras](all.md)

### `petite-vue/valid-v-scope`

Exigir que v-scope vincule um objeto literal

[Incorreto](#petite-vue-valid-v-scope-bad) · [Correto](#petite-vue-valid-v-scope-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Documentos HTML detectados como petite-vue; SFCs Vue comuns ficam fora do escopo desta regra.  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-scope": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-scope-bad"></span>

**Incorreto**

Os quatro valores não vazios de `v-scope` são um identificador, uma chamada, uma operação aritmética e um número; nenhum é analisado como um objeto literal.

```html annotate="remove:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-scope-good"></span>

**Correto**

Um `v-scope` sem valor usa o escopo raiz. Os outros valores são objetos literais, incluindo o objeto entre parênteses, que a regra aceita.

```html annotate="add:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) · [Todas as regras](all.md)

### `script/component-options-name-casing`

Exigir PascalCase na opção `name` do componente

[Incorreto](#script-component-options-name-casing-bad) · [Correto](#script-component-options-name-casing-good)

Severidade padrão: `error`  
Predefinições: _none_  
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
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-component-options-name-casing-bad"></span>

**Incorreto**

A opção de componente `name: 'my-component'` está em kebab-case, enquanto esta regra exige um nome de componente literal em PascalCase.

```vue annotate="remove:3"
<script lang="ts">
export default {
name: 'my-component' // kebab-case
}
</script>
```

<span id="script-component-options-name-casing-good"></span>

**Correto**

`MyComponent` começa com uma letra maiúscula e contém apenas caracteres alfanuméricos, atendendo à verificação de nome.

```vue annotate="add:3"
<script lang="ts">
export default {
name: 'MyComponent'
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) · [Todas as regras](all.md)

### `script/custom-event-name-casing`

Exigir camelCase nos nomes de eventos personalizados emitidos

[Incorreto](#script-custom-event-name-casing-bad) · [Correto](#script-custom-event-name-casing-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/custom-event-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-custom-event-name-casing-bad"></span>

**Incorreto**

A string emitida `my-event` contém um hífen e viola a política padrão de nomes de eventos em camelCase.

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits(['my-event'])
emit('my-event')         // kebab-case → report
</script>
```

<span id="script-custom-event-name-casing-good"></span>

**Correto**

Tanto a declaração quanto a chamada usam `myEvent`, preservando a correspondência entre o nome do evento e sua emissão e atendendo à política padrão de maiúsculas e minúsculas. Uma política configurada de kebab-case tem uma expectativa diferente.

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits(['myEvent'])
emit('myEvent')
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61) · [Todas as regras](all.md)

### `script/define-emits-declaration`

Exigir a forma de defineEmits&lt;{}&gt;() baseada em tipos em vez da forma em tempo de execução ou de array

[Incorreto](#script-define-emits-declaration-bad) · [Correto](#script-define-emits-declaration-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/define-emits-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-emits-declaration-bad"></span>

**Incorreto**

`defineEmits(["change"])` usa uma declaração de array em tempo de execução; esta regra de estilo prefere uma declaração baseada em tipos.

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

<span id="script-define-emits-declaration-good"></span>

**Correto**

`defineEmits<{ change: [id: number] }>()` move a declaração de evento para um argumento de tipo e descreve explicitamente o dado numérico usado por `emit("change", 1)`.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) · [Todas as regras](all.md)

### `script/define-macros-order`

Exigir uma ordem consistente para as macros do compilador Vue em &lt;script setup&gt;

[Incorreto](#script-define-macros-order-bad) · [Correto](#script-define-macros-order-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/define-macros-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-macros-order-bad"></span>

**Incorreto**

`defineProps` aparece antes de `defineModel`, embora `defineModel` venha primeiro na ordem canônica das macros.

```vue annotate="remove:2,3"
<script setup lang="ts">
// defineProps before defineModel (out of canonical order)
const props = defineProps<{ count: number }>()
const model = defineModel<string>()
</script>
```

<span id="script-define-macros-order-good"></span>

**Correto**

As declarações seguem a sequência exata `defineOptions`, `defineModel`, `defineProps`, `defineEmits`, `defineSlots`, antes de instruções não relacionadas em tempo de execução.

```vue annotate="add:2,4,5,6"
<script setup lang="ts">
defineOptions({ name: 'MyComponent' })
const model = defineModel<string>()
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
defineSlots<{ default(props: {}): any }>()
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L46) · [Todas as regras](all.md)

### `script/define-props-declaration`

Exigir defineProps&lt;{ ... }&gt;() baseado em tipos em vez da forma com objeto em tempo de execução

[Incorreto](#script-define-props-declaration-bad) · [Correto](#script-define-props-declaration-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/define-props-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-declaration-bad"></span>

**Incorreto**

`defineProps({ title: String })` fornece um objeto em tempo de execução, contrariando a preferência desta regra por props baseadas em tipos.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

<span id="script-define-props-declaration-good"></span>

**Correto**

`defineProps<{ title: string }>()` declara `title` no argumento de tipo e mantém o acesso a `props.title` sem um argumento de declaração em tempo de execução.

```vue annotate="add:2"
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) · [Todas as regras](all.md)

### `script/define-props-destructuring`

Exigir um estilo consistente de desestruturação de defineProps em &lt;script setup&gt;

[Incorreto](#script-define-props-destructuring-bad) · [Correto](#script-define-props-destructuring-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-destructuring": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-destructuring-bad"></span>

**Incorreto**

`defineProps` é atribuído à variável única `props` em vez de ser desestruturado, contrariando a preferência padrão pela desestruturação.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

<span id="script-define-props-destructuring-good"></span>

**Correto**

O padrão de objeto vincula `foo` e `bar` diretamente e fornece um valor padrão para o `bar` opcional. Isso depende da desestruturação reativa de props do Vue 3.5+; o modo configurável `never` prefere a forma oposta.

```vue annotate="add:2"
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) · [Todas as regras](all.md)

### `script/no-arrow-functions-in-watch`

Proibir funções de seta como manipuladores de watch na Options API

[Incorreto](#script-no-arrow-functions-in-watch-bad) · [Correto](#script-no-arrow-functions-in-watch-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-arrow-functions-in-watch": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-arrow-functions-in-watch-bad"></span>

**Incorreto**

O observador `value` da Options API e o `other.handler` aninhado são funções de seta. Uma função de seta captura o `this` do contexto ao redor em vez de receber a instância do componente.

```vue annotate="remove:4,5,9"
<script lang="ts">
export default {
watch: {
// `this` is not the component instance inside an arrow function.
value: () => {
this.doSomething()
},
other: {
handler: () => {}
}
}
}
</script>
```

<span id="script-no-arrow-functions-in-watch-good"></span>

**Correto**

Ambos os manipuladores passam a ser métodos comuns, permitindo que o Vue vincule `this` ao componente. A opção `deep: true` do observador continua compatível com a forma de objeto.

```vue annotate="add:4,8,9"
<script lang="ts">
export default {
watch: {
value(newValue, oldValue) {
this.doSomething()
},
other: {
handler(newValue) {},
deep: true
}
}
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59) · [Todas as regras](all.md)

### `script/no-async-in-computed`

Proibir funções assíncronas em propriedades computadas

[Incorreto](#script-no-async-in-computed-bad) · [Correto](#script-no-async-in-computed-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-async-in-computed": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-async-in-computed-bad"></span>

**Incorreto**

O getter de `computed` é `async`, então a busca produz uma Promise em vez de um valor computado derivado de forma síncrona.

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
import { computed } from "vue";
const data = computed(async () => {
  const response = await fetch("/api/data");
  return response.json();
});
</script>
```

<span id="script-no-async-in-computed-good"></span>

**Correto**

A busca assíncrona é movida para `watch` e armazena seu resultado em `data.value`. A limpeza cancela a requisição anterior e impede que uma função de retorno inativa grave um resultado desatualizado; não resta nenhum getter computado assíncrono.

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
import { ref, watch } from "vue";
const query = ref("");
const data = ref<unknown>(null);
watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;
  onCleanup(() => { active = false; controller.abort(); });
  const response = await fetch(`/api/data?q=${encodeURIComponent(value)}`, { signal: controller.signal });
  const next: unknown = await response.json();
  if (active) data.value = next;
});
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L46) · [Todas as regras](all.md)

### `script/no-boolean-default`

Proibir um valor padrão em uma prop Boolean

[Incorreto](#script-no-boolean-default-bad) · [Correto](#script-no-boolean-default-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/no-boolean-default": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-boolean-default-bad"></span>

**Incorreto**

Tanto `disabled` quanto `checked` declaram um `default` em uma prop cujo único construtor é `Boolean`; a regra rejeita até mesmo um valor padrão `false` explícito.

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
props: {
// Boolean props already default to false; an explicit default is confusing.
disabled: { type: Boolean, default: true },
checked: { type: Boolean, default: false }
}
}
</script>
```

<span id="script-no-boolean-default-good"></span>

**Correto**

As props exclusivamente Boolean omitem `default`, usando o valor false implícito do Vue. A união `[Boolean, String]` e a prop Number ilustram que esta verificação se limita ao construtor único `Boolean`.

```vue annotate="add:4,5,6,7,8,9,10"
<script lang="ts">
export default {
props: {
// No explicit default: defaults to false.
disabled: { type: Boolean },
disabled2: Boolean,
// Union type may legitimately need a default.
value: { type: [Boolean, String], default: '' },
// Non-Boolean prop.
count: { type: Number, default: 0 }
}
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54) · [Todas as regras](all.md)

### `script/no-deep-destructure-in-props`

Proibir desestruturação profundamente aninhada em defineProps

[Incorreto](#script-no-deep-destructure-in-props-bad) · [Correto](#script-no-deep-destructure-in-props-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/no-deep-destructure-in-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deep-destructure-in-props-bad"></span>

**Incorreto**

O padrão de vinculação percorre `user` para desestruturar `name`, ultrapassando a profundidade rasa padrão da desestruturação de props.

```vue annotate="remove:2"
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

<span id="script-no-deep-destructure-in-props-good"></span>

**Correto**

O objeto de props permanece intacto, e um getter computado lê `props.user.name`. O acesso aninhado continua explícito sem um padrão de vinculação profundamente aninhado.

```vue annotate="add:2,3,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) · [Todas as regras](all.md)

### `script/no-deprecated-data-object-declaration`

Proibir um literal de objeto como opção data do componente (o Vue 3 exige uma função)

[Incorreto](#script-no-deprecated-data-object-declaration-bad) · [Correto](#script-no-deprecated-data-object-declaration-good)

Severidade padrão: `error`  
Predefinições: _none_  
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
        "script/no-deprecated-data-object-declaration": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-data-object-declaration-bad"></span>

**Incorreto**

A opção `data` da Options API é um literal de objeto, uma forma do Vue 2 que o Vue 3 não aceita mais.

```vue annotate="remove:3,4,5"
<script lang="ts">
export default {
// `data` must be a function in Vue 3, not an object literal.
data: {
count: 0
}
}
</script>
```

<span id="script-no-deprecated-data-object-declaration-good"></span>

**Correto**

`data()` retorna um novo objeto `{ count: 0 }`, fornecendo a declaração de data baseada em função exigida pelo Vue 3.

```vue annotate="add:3,4"
<script lang="ts">
export default {
data() {
return { count: 0 }
}
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) · [Todas as regras](all.md)

### `script/no-deprecated-destroyed-lifecycle`

Proibir os hooks de ciclo de vida obsoletos destroyed e beforeDestroy

[Incorreto](#script-no-deprecated-destroyed-lifecycle-bad) · [Correto](#script-no-deprecated-destroyed-lifecycle-good)

Severidade padrão: `error`  
Predefinições: _none_  
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
        "script/no-deprecated-destroyed-lifecycle": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-destroyed-lifecycle-bad"></span>

**Incorreto**

`beforeDestroy` é a opção de ciclo de vida removida do Vue 2 usada para limpar o temporizador.

```vue annotate="remove:2"
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

**Correto**

Renomear o hook para `beforeUnmount` preserva o corpo da limpeza com o nome de ciclo de vida do Vue 3.

```vue annotate="add:2"
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) · [Todas as regras](all.md)

### `script/no-deprecated-dollar-listeners-api`

Proibir a propriedade de instância $listeners removida no Vue 3 (incorporada a $attrs)

[Incorreto](#script-no-deprecated-dollar-listeners-api-bad) · [Correto](#script-no-deprecated-dollar-listeners-api-good)

Severidade padrão: `error`  
Predefinições: _none_  
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
        "script/no-deprecated-dollar-listeners-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-listeners-api-bad"></span>

**Incorreto**

As leituras de membros e a referência direta no argumento usam `$listeners`, que o Vue 3 removeu após incorporar os ouvintes aos atributos.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const handlers = this.$listeners
const forwarded = ctx.$listeners
emit('input', $listeners)
</script>
```

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

**Correto**

As leituras passam a usar `this.$attrs` e `ctx.attrs` do contexto de setup. Essas formas substituem a API de ouvintes removida; os objetos receptores ilustrados precisam existir no contexto do componente ao redor.

```vue annotate="add:2,3"
<script setup lang="ts">
const handlers = this.$attrs
const forwarded = ctx.attrs
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L40) · [Todas as regras](all.md)

### `script/no-deprecated-dollar-scopedslots-api`

Proibir a propriedade de instância $scopedSlots removida no Vue 3 (usar $slots)

[Incorreto](#script-no-deprecated-dollar-scopedslots-api-bad) · [Correto](#script-no-deprecated-dollar-scopedslots-api-good)

Severidade padrão: `error`  
Predefinições: _none_  
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
        "script/no-deprecated-dollar-scopedslots-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-scopedslots-api-bad"></span>

**Incorreto**

`this.$scopedSlots`, `ctx.$scopedSlots` e a referência direta a `$scopedSlots` usam a API de slots com escopo do Vue 2 removida no Vue 3.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

**Correto**

Substituir `$scopedSlots` por `$slots` usa a API unificada de slots. O exemplo remove a grafia obsoleta em vez de estabelecer um contexto de setup para os objetos receptores.

```vue annotate="add:2,3,4"
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L44) · [Todas as regras](all.md)

### `script/no-deprecated-events-api`

Proibir a API de eventos do Vue 2 removida ($on / $off / $once)

[Incorreto](#script-no-deprecated-events-api-bad) · [Correto](#script-no-deprecated-events-api-good)

Severidade padrão: `error`  
Predefinições: _none_  
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
        "script/no-deprecated-events-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-events-api-bad"></span>

**Incorreto**

As chamadas de `$on`, `$once` e `$off` usam os métodos de barramento de eventos da instância removidos no Vue 3.

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
this.$on('event', handler)
this.$once('event', handler)
this.$off('event', handler)
emitter.$off('event')
</script>
```

<span id="script-no-deprecated-events-api-good"></span>

**Correto**

`$emit` continua válido, enquanto a assinatura no barramento de eventos passa para o método `on` do emissor externo. A correção separa a emissão direcionada ao pai de um barramento de eventos externo.

```vue annotate="add:2,3,4,5,6,7,8"
<script setup lang="ts">
// $emit is still valid in Vue 3
this.$emit('event', payload)

// Use an external emitter instead
import mitt from 'mitt'
const emitter = mitt()
emitter.on('event', handler)
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L42) · [Todas as regras](all.md)

### `script/no-deprecated-props-default-this`

Proibir `this` dentro de uma função de valor padrão ou validação de prop (removido no Vue 3)

[Incorreto](#script-no-deprecated-props-default-this-bad) · [Correto](#script-no-deprecated-props-default-this-good)

Severidade padrão: `error`  
Predefinições: _none_  
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
        "script/no-deprecated-props-default-this": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-props-default-this-bad"></span>

**Incorreto**

O valor padrão e o validador da prop leem `this`, mas essas funções não podem depender da instância do componente no Vue 3.

```vue annotate="remove:6,7,8,13,14"
<script lang="ts">
export default {
props: {
size: {
type: Number,
// `this` is not the component instance in Vue 3.
default() {
return this.defaultSize
}
},
value: {
type: Number,
validator() {
return this.value > 0
}
}
}
}
</script>
```

<span id="script-no-deprecated-props-default-this-good"></span>

**Correto**

A função de valor padrão lê `props.baseSize` de seu argumento, e o validador testa seu argumento `value`. Ambos deixam de depender de um objeto receptor de instância indisponível.

```vue annotate="add:6,7,8,13,14"
<script lang="ts">
export default {
props: {
size: {
type: Number,
// Vue 3 passes the raw props as the first argument instead.
default(props) {
return props.baseSize
}
},
value: {
type: Number,
validator(value) {
return value > 0
}
}
}
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) · [Todas as regras](all.md)

### `script/no-dupe-keys`

Proibir chaves duplicadas entre props/data/computed/methods/setup/inject da Options API

[Incorreto](#script-no-dupe-keys-bad) · [Correto](#script-no-dupe-keys-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-dupe-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-dupe-keys-bad"></span>

**Incorreto**

`foo` é declarado tanto em props quanto em data, e `bar` tanto em computed quanto em methods. Essas declarações disputam as mesmas chaves na instância do componente.

```vue annotate="remove:5,8,9,10,11"
<script lang="ts">
export default {
props: ['foo'],
data() {
return { foo: 1 } // duplicate of prop `foo`
},
computed: {
bar() { return 2 }
},
methods: {
bar() {} // duplicate of computed `bar`
}
}
</script>
```

<span id="script-no-dupe-keys-good"></span>

**Correto**

As declarações de prop, data e computed usam nomes distintos (`foo`, `bar` e `baz`), eliminando ambas as colisões entre opções.

```vue annotate="add:5,8"
<script lang="ts">
export default {
props: ['foo'],
data() {
return { bar: 1 }
},
computed: {
baz() { return 2 }
}
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L52) · [Todas as regras](all.md)

### `script/no-duplicate-attr-inheritance`

Sinalizar um componente que aplica duas vezes seus atributos repassados

[Incorreto](#script-no-duplicate-attr-inheritance-bad) · [Correto](#script-no-duplicate-attr-inheritance-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-duplicate-attr-inheritance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-duplicate-attr-inheritance-bad"></span>

**Incorreto**

Os valores explícitos `inheritAttrs: true` repetem o padrão do Vue. Esta regra sinaliza esse literal redundante mesmo quando não há expansão de `$attrs` na raiz.

```vue annotate="remove:2,3"
<script lang="ts">
defineOptions({ inheritAttrs: true })
export default { inheritAttrs: true }
</script>
```

<span id="script-no-duplicate-attr-inheritance-good"></span>

**Correto**

`inheritAttrs: false` expressa uma desativação efetiva, enquanto o objeto vazio de opções deixa implícita a herança padrão. Nenhuma das formas repete o valor redundante `true`.

```vue annotate="add:2,3"
<script lang="ts">
defineOptions({ inheritAttrs: false }) // intentional opt-out
export default {}                      // default inheritance, unstated
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L73) · [Todas as regras](all.md)

### `script/no-export-in-script-setup`

Proibir instruções export dentro de &lt;script setup&gt;

[Incorreto](#script-no-export-in-script-setup-bad) · [Correto](#script-no-export-in-script-setup-good)

Severidade padrão: `error`  
Predefinições: _none_  
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
        "script/no-export-in-script-setup": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-export-in-script-setup-bad"></span>

**Incorreto**

`export const count` tenta expor uma exportação de módulo a partir de `<script setup>`, onde exportações em tempo de execução são proibidas.

```vue annotate="remove:2"
<script setup lang="ts">
export const count = 1;
</script>
```

<span id="script-no-export-in-script-setup-good"></span>

**Correto**

Remover `export` mantém `count` como uma variável de setup em vez de uma exportação de módulo.

```vue annotate="add:2"
<script setup lang="ts">
const count = 1;
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) · [Todas as regras](all.md)

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

### `script/no-import-compiler-macros`

Proibir a importação de macros do compilador Vue que são importadas automaticamente

[Incorreto](#script-no-import-compiler-macros-bad) · [Correto](#script-no-import-compiler-macros-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-import-compiler-macros": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-import-compiler-macros-bad"></span>

**Incorreto**

A importação de `vue` inclui `defineProps` e `defineEmits`, embora sejam macros do compilador disponíveis diretamente em `<script setup>`.

```vue annotate="remove:2"
<script setup lang="ts">
import { defineProps, defineEmits } from "vue";
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

<span id="script-no-import-compiler-macros-good"></span>

**Correto**

Remover as importações das macros mantém ambas as chamadas tipadas intactas; nenhuma das declarações precisa de uma importação em tempo de execução.

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) · [Todas as regras](all.md)

### `script/no-internal-imports`

Proibir importações de módulos internos do Vue

[Incorreto](#script-no-internal-imports-bad) · [Correto](#script-no-internal-imports-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-internal-imports": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-internal-imports-bad"></span>

**Incorreto**

Ambas as importações apontam para arquivos internos de `dist` em vez do ponto de entrada público do pacote Vue, vinculando o componente aos caminhos dos arquivos de compilação.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { foo } from '@vue/runtime-core/dist/runtime-core.esm-bundler'
import { bar } from 'vue/dist/vue.esm-bundler'
</script>
```

<span id="script-no-internal-imports-good"></span>

**Correto**

Importar as funções auxiliares necessárias de `vue` remove a dependência dos locais dos arquivos internos de distribuição.

```vue annotate="add:2"
<script setup lang="ts">
import { ref, computed } from 'vue'
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28) · [Todas as regras](all.md)

### `script/no-multiple-slot-args`

Proibir passar mais de um argumento a uma chamada de função de slot com escopo

[Incorreto](#script-no-multiple-slot-args-bad) · [Correto](#script-no-multiple-slot-args-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-multiple-slot-args": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-multiple-slot-args-bad"></span>

**Incorreto**

As chamadas de slot passam vários argumentos posicionais ou expandem uma lista desconhecida de argumentos. Os slots do Vue recebem um único objeto de props, não uma lista de parâmetros posicionais.

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
slots.default(foo, bar)
$slots.header(a, b)
this.$scopedSlots.item(x, y)
useSlots().default(a, b)
slots.default(...args)
</script>
```

<span id="script-no-multiple-slot-args-good"></span>

**Correto**

`{ foo, bar }` combina os dados em um único argumento; `slotProps` e a chamada sem argumentos também respeitam a forma de chamada de slot suportada.

```vue annotate="add:2,3,4"
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) · [Todas as regras](all.md)

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

### `script/no-potential-component-option-typo`

Sinalizar prováveis erros de digitação nos nomes de opções de componentes da Options API

[Incorreto](#script-no-potential-component-option-typo-bad) · [Correto](#script-no-potential-component-option-typo-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-potential-component-option-typo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-potential-component-option-typo-bad"></span>

**Incorreto**

A opção está escrita como `method`, a uma edição de distância da opção reconhecida `methods`; o Vue não a trataria como a declaração de métodos pretendida.

```vue annotate="remove:2"
<script lang="ts">
export default { method: { save() {} } };
</script>
```

<span id="script-no-potential-component-option-typo-good"></span>

**Correto**

Alterar a chave para `methods` coloca `save()` dentro da opção de componente reconhecida.

```vue annotate="add:2"
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) · [Todas as regras](all.md)

### `script/no-reactive-destructure`

Proibir a desestruturação de objetos reativos que causa perda de reatividade

[Incorreto](#script-no-reactive-destructure-bad) · [Correto](#script-no-reactive-destructure-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/no-reactive-destructure": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reactive-destructure-bad"></span>

**Incorreto**

`const { count, name } = state` copia propriedades primitivas do objeto `reactive`, perdendo sua conexão com alterações posteriores nas propriedades.

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = state;
</script>
```

<span id="script-no-reactive-destructure-good"></span>

**Correto**

Desestruturar `toRefs(state)` cria refs para `count` e `name`, mantendo cada variável vinculada à propriedade reativa original.

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRefs } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = toRefs(state);
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) · [Todas as regras](all.md)

### `script/no-ref-as-operand`

Exigir que variáveis vinculadas a refs sejam acessadas por `.value` quando usadas como operandos

[Incorreto](#script-no-ref-as-operand-bad) · [Correto](#script-no-ref-as-operand-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-ref-as-operand": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-ref-as-operand-bad"></span>

**Incorreto**

`count + 1` usa o próprio objeto ref como operando aritmético em vez do número que ele encapsula.

```vue annotate="remove:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count + 1;
</script>
```

<span id="script-no-ref-as-operand-good"></span>

**Correto**

`count.value + 1` lê o número encapsulado antes de somar um; a aritmética no script exige esse acesso explícito ao ref.

```vue annotate="add:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count.value + 1;
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) · [Todas as regras](all.md)

### `script/no-required-prop-with-default`

Proibir uma prop que tenha required: true e também um valor padrão

[Incorreto](#script-no-required-prop-with-default-bad) · [Correto](#script-no-required-prop-with-default-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-required-prop-with-default": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-required-prop-with-default-bad"></span>

**Incorreto**

`title` é obrigatório e recebe o valor alternativo `"Untitled"`, combinando um contrato de entrada obrigatória com um valor padrão destinado à ausência de entrada.

```vue annotate="remove:2"
<script lang="ts">
export default { props: { title: { type: String, required: true, default: "Untitled" } } };
</script>
```

<span id="script-no-required-prop-with-default-good"></span>

**Correto**

Remover `required: true` torna `title` opcional e mantém `"Untitled"` como seu valor alternativo coerente.

```vue annotate="add:2"
<script lang="ts">
export default { props: { title: { type: String, default: "Untitled" } } };
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L29) · [Todas as regras](all.md)

### `script/no-reserved-identifiers`

Proibir o uso de identificadores reservados pelo compilador Vue

[Incorreto](#script-no-reserved-identifiers-bad) · [Correto](#script-no-reserved-identifiers-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-reserved-identifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-identifiers-bad"></span>

**Incorreto**

As variáveis `__props`, `__emit` e `__sfc__` usam identificadores reservados para código gerado pelo compilador Vue.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const __props = { name: "Ada" };
const __emit = () => {};
const __sfc__ = {};
</script>
```

<span id="script-no-reserved-identifiers-good"></span>

**Correto**

Os nomes comuns `props`, `emit` e `componentData` evitam esses identificadores gerados enquanto mantêm as declarações de props e emits.

```vue annotate="add:2,3,4"
<script setup lang="ts">
const props = defineProps<{ name: string }>();
const emit = defineEmits<{ save: [] }>();
const componentData = {};
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) · [Todas as regras](all.md)

### `script/no-reserved-keys`

Proibir nomes reservados pelo Vue como chaves de props/data/computed/methods/setup/inject na Options API

[Incorreto](#script-no-reserved-keys-bad) · [Correto](#script-no-reserved-keys-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-reserved-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-keys-bad"></span>

**Incorreto**

A chave de data retornada, `$el`, colide com a propriedade nativa da instância do componente Vue e também usa um prefixo `$` reservado.

```vue annotate="remove:2"
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

<span id="script-no-reserved-keys-good"></span>

**Correto**

Renomear o dado da aplicação para `elementLabel` evita a API nativa da instância e o prefixo reservado.

```vue annotate="add:2"
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L32) · [Todas as regras](all.md)

### `script/no-reserved-props`

Proibir nomes reservados na declaração de props de um componente

[Incorreto](#script-no-reserved-props-bad) · [Correto](#script-no-reserved-props-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-reserved-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-props-bad"></span>

**Incorreto**

`ref` e `$foo` na forma de objeto, além de `key` na forma de array, são nomes reservados de props. `ref` e `key` são controles do framework, e nomes com prefixo `$` são rejeitados.

```vue annotate="remove:4,5,6,8,9,10"
<script lang="ts">
export default {
props: {
ref: String,   // reserved
$foo: Number    // `$`-prefixed names are reserved
}
}

export default {
props: ['key']    // reserved (array form)
}
</script>
```

<span id="script-no-reserved-props-good"></span>

**Correto**

Os nomes comuns de props `name` e `refValue` evitam os nomes reservados tanto na grafia quanto no prefixo.

```vue annotate="add:4,5"
<script lang="ts">
export default {
props: {
name: String,
refValue: Number
}
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) · [Todas as regras](all.md)

### `script/no-restricted-globals`

Proibir referências a variáveis globais do ambiente de execução que devem passar por um encapsulamento tipado

[Incorreto](#script-no-restricted-globals-bad) · [Correto](#script-no-restricted-globals-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-globals": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-restricted-globals-bad"></span>

**Incorreto**

O exemplo lê diretamente as variáveis globais restritas por padrão `process`, `localStorage` e `sessionStorage`, ignorando as funções auxiliares explícitas de configuração e armazenamento do projeto.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const flag = process.env.FEATURE_FLAG
const token = localStorage.getItem('auth.token')
sessionStorage.setItem('view.scroll', String(window.scrollY))
</script>
```

<span id="script-no-restricted-globals-good"></span>

**Correto**

`useFeatureFlag`, `authStorage.read` e `viewStorage.write` removem essas referências diretas às variáveis globais restritas. O `window.scrollY` restante não é uma restrição padrão desta regra; a segurança em SSR é uma questão separada.

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
// Use a typed config helper that distinguishes server vs. client.
const flag = useFeatureFlag('FEATURE_FLAG')

// Use a typed wrapper that scopes keys and handles SSR / disabled storage.
const token = authStorage.read('auth.token')
viewStorage.write('view.scroll', String(window.scrollY))
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) · [Todas as regras](all.md)

### `script/no-restricted-members`

Proibir acessos a membros object.property configurados pelo projeto

[Incorreto](#script-no-restricted-members-bad) · [Correto](#script-no-restricted-members-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

Este exemplo configura window.localStorage. A regra não tem uma lista padrão de bloqueio; ativá-la sozinha não reporta um membro.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-members": "error"
      },
      "ruleOptions": {
        "script/no-restricted-members": {
          "members": [
            {
              "object": "window",
              "property": "localStorage"
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

<span id="script-no-restricted-members-bad"></span>

**Incorreto**

Com `{ object: "window", property: "localStorage" }` configurado em `ruleOptions`, `window.localStorage` acessa o par objeto/membro proibido. Esta regra não tem membros proibidos por padrão.

```vue annotate="remove:2"
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

<span id="script-no-restricted-members-good"></span>

**Correto**

`authStorage.read("token")` delega a leitura à função auxiliar de armazenamento da aplicação e deixa de acessar o membro configurado `window.localStorage`.

```vue annotate="add:2"
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) · [Todas as regras](all.md)

### `script/no-side-effects-in-computed-properties`

Proibir efeitos colaterais em getters computados da Options API

[Incorreto](#script-no-side-effects-in-computed-properties-bad) · [Correto](#script-no-side-effects-in-computed-properties-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-side-effects-in-computed-properties": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-side-effects-in-computed-properties-bad"></span>

**Incorreto**

`doubled` atribui um valor a `this.count`, e `reversed` modifica `this.items` por meio de `reverse()`. Ambos os getters alteram o estado do qual deveriam derivar seus valores.

```vue annotate="remove:8,9,12"
<script lang="ts">
export default {
data() {
return { count: 0, items: [] }
},
computed: {
doubled() {
this.count = this.count * 2 // side effect: assigns to data
return this.count
},
reversed() {
return this.items.reverse() // side effect: mutates the array
}
}
}
</script>
```

<span id="script-no-side-effects-in-computed-properties-good"></span>

**Correto**

`doubled` retorna a multiplicação sem atribuição. `reversed` copia o array antes de invertê-lo, então o getter não altera o estado original do componente.

```vue annotate="add:8,11"
<script lang="ts">
export default {
data() {
return { count: 0, items: [] }
},
computed: {
doubled() {
return this.count * 2
},
reversed() {
return [...this.items].reverse() // operate on a copy
}
}
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L70) · [Todas as regras](all.md)

### `script/no-top-level-ref-in-script`

Proibir ref/reactive no nível superior para evitar contaminação de estado entre requisições

[Incorreto](#script-no-top-level-ref-in-script-bad) · [Correto](#script-no-top-level-ref-in-script-good)

Severidade padrão: `error`  
Predefinições: _none_  
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
        "script/no-top-level-ref-in-script": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-top-level-ref-in-script-bad"></span>

**Incorreto**

O `<script>` comum inicializa `count` e `user` no escopo do módulo. Durante SSR, esses objetos de estado podem ser compartilhados entre instâncias de componentes e requisições.

```vue annotate="remove:1,2,4,8"
<script>
// This state is shared across all requests in SSR!
const count = ref(0)
const user = reactive({ name: '' })

export default {
setup() {
return { count, user }
}
}
</script>
```

<span id="script-no-top-level-ref-in-script-good"></span>

**Correto**

O ref de setup é inicializado por instância de componente; o script comum mantém apenas uma constante, uma função que produz estado e um ref criado dentro de `setup()`. Nenhuma dessas formas cria estado reativo no escopo do módulo comum.

```vue annotate="add:1,2,4,6,7,8,9,10,11,12,13,14,17,18,19"
<script setup>
// Script setup creates fresh state per request
const count = ref(0)
</script>

<script>
// Constants are fine
const API_URL = 'https://api.example.com'

// Functions that create state are fine
function createState() {
return reactive({ count: 0 })
}

export default {
setup() {
// Create state inside setup
const count = ref(0)
return { count }
}
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) · [Todas as regras](all.md)

### `script/no-unstable-nested-components`

Proibir definições de componentes dentro de funções de setup ou renderização

[Incorreto](#script-no-unstable-nested-components-bad) · [Correto](#script-no-unstable-nested-components-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-unstable-nested-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unstable-nested-components-bad"></span>

**Incorreto**

`defineComponent` é executado dentro do `setup()` do pai, criando uma nova definição do componente `Child` sempre que esse setup é executado.

```vue annotate="remove:3"
<script lang="ts">
import { defineComponent } from "vue";
export default { setup() { const Child = defineComponent({ render() { return null; } }); return { Child }; } };
</script>
```

<span id="script-no-unstable-nested-components-good"></span>

**Correto**

A definição de `Child` é movida para o escopo do módulo, e `setup()` retorna essa definição existente em vez de recriá-la.

```vue annotate="add:3,4"
<script lang="ts">
import { defineComponent } from "vue";
const Child = defineComponent({ render() { return null; } });
export default { setup() { return { Child }; } };
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) · [Todas as regras](all.md)

### `script/no-unused-emit-declarations`

Sinalizar eventos declarados que nunca são emitidos

[Incorreto](#script-no-unused-emit-declarations-bad) · [Correto](#script-no-unused-emit-declarations-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/no-unused-emit-declarations": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unused-emit-declarations-bad"></span>

**Incorreto**

`defineEmits` declara tanto `change` quanto `unused`, mas a função `emit` capturada só emite o evento literal `change`.

```vue annotate="remove:2,4"
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

<span id="script-no-unused-emit-declarations-good"></span>

**Correto**

Remover `unused` faz a lista de eventos declarados corresponder à emissão observada. O exemplo usa uma variável emit capturada que não escapa do escopo, permitindo essa conclusão sobre o uso local.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) · [Todas as regras](all.md)

### `script/no-use-computed-property-like-method`

Proibir chamar uma propriedade computada da Options API como um método

[Incorreto](#script-no-use-computed-property-like-method-bad) · [Correto](#script-no-use-computed-property-like-method-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-use-computed-property-like-method": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-use-computed-property-like-method-bad"></span>

**Incorreto**

`this.total()` chama o valor exposto pelo getter computado; o getter retorna `3`, que não pode ser chamado.

```vue annotate="remove:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total()); } } };
</script>
```

<span id="script-no-use-computed-property-like-method-good"></span>

**Correto**

`this.total` lê o valor computado sem parênteses de chamada, então `log` imprime o número derivado.

```vue annotate="add:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total); } } };
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44) · [Todas as regras](all.md)

### `script/no-with-defaults`

Desencorajar withDefaults em favor de valores padrão na desestruturação (Vue 3.5+)

[Incorreto](#script-no-with-defaults-bad) · [Correto](#script-no-with-defaults-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`  
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
        "script/no-with-defaults": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-with-defaults-bad"></span>

**Incorreto**

`withDefaults` envolve a declaração tipada de props apenas para fornecer valores padrão a `count` e `name`, em vez do estilo de valores padrão na desestruturação do Vue 3.5+ preferido aqui.

```vue annotate="remove:2"
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

<span id="script-no-with-defaults-good"></span>

**Correto**

O padrão de desestruturação coloca `count = 0` e `name = "Ada"` junto às suas variáveis e remove o encapsulamento de `withDefaults`.

```vue annotate="add:2"
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) · [Todas as regras](all.md)

### `script/prefer-computed`

Preferir computed() para estado reativo derivado

[Incorreto](#script-prefer-computed-bad) · [Correto](#script-prefer-computed-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

O watcher deve apenas derivar o destino. Cópias editáveis e callbacks com outros efeitos colaterais são permitidos.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-computed": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-computed-bad"></span>

**Incorreto**

O observador apenas copia uma derivação de `count` para um segundo ref, `doubled`, então o estado derivado é mantido por sincronização manual.

```vue annotate="remove:2,4,5"
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
const doubled = ref(0);
watch(count, (value) => { doubled.value = value * 2; });
</script>
```

<span id="script-prefer-computed-good"></span>

**Correto**

`computed(() => count.value * 2)` expressa a derivação diretamente e remove tanto o ref gravável adicional quanto seu observador de sincronização.

```vue annotate="add:2,4"
<script setup lang="ts">
import { ref, computed } from "vue";
const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) · [Todas as regras](all.md)

### `script/prefer-define-options`

Preferir defineOptions() a um &lt;script&gt; comum que só define name/inheritAttrs

[Incorreto](#script-prefer-define-options-bad) · [Correto](#script-prefer-define-options-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/prefer-define-options": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-define-options-bad"></span>

**Incorreto**

A única instrução significativa do script comum exporta um objeto contendo apenas `name` e `inheritAttrs`; essas opções podem ser expressas por `defineOptions`.

```vue annotate="remove:2"
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

<span id="script-prefer-define-options-good"></span>

**Correto**

O método `data()` mostrado faz o script conter lógica efetiva da Options API, portanto ele fica fora da sugestão conservadora desta regra para scripts que contêm apenas opções. Este exemplo correto demonstra uma exceção permitida; a migração direta colocaria `defineOptions({ name: 'MyComponent', inheritAttrs: false })` em `<script setup>`.

```vue annotate="add:2,3,4,5,6"
<script lang="ts">
// Real options logic — keep the plain script.
export default {
name: 'MyComponent',
data() { return { count: 0 } },
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) · [Todas as regras](all.md)

### `script/prefer-import-from-vue`

Preferir importar de 'vue' em vez de pacotes internos

[Incorreto](#script-prefer-import-from-vue-bad) · [Correto](#script-prefer-import-from-vue-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/prefer-import-from-vue": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-import-from-vue-bad"></span>

**Incorreto**

`ref` e `h` são importados dos pacotes internos `@vue/runtime-core` e `@vue/runtime-dom` em vez do pacote público `vue`.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

<span id="script-prefer-import-from-vue-good"></span>

**Correto**

Ambas as funções auxiliares são importadas juntas de `vue`, usando o ponto de entrada público do pacote em vez de qualquer um dos pacotes internos.

```vue annotate="add:2"
<script setup lang="ts">
import { ref, h } from 'vue'
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L32) · [Todas as regras](all.md)

### `script/prefer-ref-over-reactive`

Recomendar o uso de ref() em vez de reactive() para gerenciar estado

[Incorreto](#script-prefer-ref-over-reactive-bad) · [Correto](#script-prefer-ref-over-reactive-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/prefer-ref-over-reactive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-ref-over-reactive-bad"></span>

**Incorreto**

O estado é criado com `reactive`, contrariando a preferência desta regra de convenção por refs. O exemplo ilustra uma preferência de estilo, não um objeto reativo inerentemente inválido.

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
// reactive requires careful handling to avoid losing reactivity
const state = reactive({
count: 0,
name: 'foo'
})
</script>
```

<span id="script-prefer-ref-over-reactive-good"></span>

**Correto**

Os exemplos criam tanto estado escalar quanto de objeto com `ref`; campos relacionados também podem ser separados em refs distintos. Isso atende à forma preferida de criação de estado.

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
// ref is more explicit and safer
const count = ref(0)
const name = ref('foo')

// For objects, ref still works
const user = ref({ name: 'foo', age: 20 })

// Or use multiple refs for related data
const userName = ref('foo')
const userAge = ref(20)
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) · [Todas as regras](all.md)

### `script/prefer-use-attrs`

Recomendar o uso de useAttrs() em vez de context.attrs

[Incorreto](#script-prefer-use-attrs-bad) · [Correto](#script-prefer-use-attrs-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/prefer-use-attrs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-attrs-bad"></span>

**Incorreto**

`setup` obtém `attrs` desestruturando seu parâmetro de contexto, forma que esta regra pede para substituir pela função auxiliar da Composition API.

```vue annotate="remove:2"
<script lang="ts">
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

<span id="script-prefer-use-attrs-good"></span>

**Correto**

`useAttrs()` fornece `attrs` dentro de setup, mantendo a leitura de `attrs.class` sem depender do segundo parâmetro de setup.

```vue annotate="add:2,3"
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) · [Todas as regras](all.md)

### `script/prefer-use-id`

Recomendar o uso de useId() para gerar IDs únicos (Vue 3.5+)

[Incorreto](#script-prefer-use-id-bad) · [Correto](#script-prefer-use-id-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/prefer-use-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-id-bad"></span>

**Incorreto**

`id` contém `Math.random()`, então o identificador gerado para input/label pode diferir entre a renderização no servidor e no cliente. A variável com nome de ID é o contexto de geração reconhecido pela regra.

```vue annotate="remove:2"
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

<span id="script-prefer-use-id-good"></span>

**Correto**

`useId()` do Vue 3.5+ gera o identificador, e tanto `:for` quanto `:id` continuam lendo a mesma variável em vez de gerar valores aleatórios de forma independente.

```vue annotate="add:2,3"
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) · [Todas as regras](all.md)

### `script/prefer-use-slots`

Recomendar o uso de useSlots() em vez de context.slots

[Incorreto](#script-prefer-use-slots-bad) · [Correto](#script-prefer-use-slots-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/prefer-use-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-slots-bad"></span>

**Incorreto**

`setup` desestrutura `slots` de seu argumento de contexto, a forma de acesso que esta regra prefere substituir.

```vue annotate="remove:2,4"
<script lang="ts">
import { defineComponent, h } from "vue";
export default defineComponent({
  setup(_props, { slots }) { return () => h("div", slots.default?.()); },
});
</script>
```

<span id="script-prefer-use-slots-good"></span>

**Correto**

`useSlots()` obtém os slots dentro de setup, preservando a função de renderização e sua chamada opcional ao slot padrão sem um parâmetro de contexto.

```vue annotate="add:2,4,5,6,7"
<script lang="ts">
import { defineComponent, h, useSlots } from "vue";
export default defineComponent({
  setup() {
    const slots = useSlots();
    return () => h("div", slots.default?.());
  },
});
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) · [Todas as regras](all.md)

### `script/prefer-use-template-ref`

Recomendar useTemplateRef em vez de ref para referências de template (Vue 3.5+)

[Incorreto](#script-prefer-use-template-ref-bad) · [Correto](#script-prefer-use-template-ref-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/prefer-use-template-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-template-ref-bad"></span>

**Incorreto**

O ref `input`, que aceita null, está associado ao literal `ref="input"` do template, identificando-o como uma referência de elemento em vez de um dado comum que aceita null.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from 'vue'
const input = ref<HTMLInputElement | null>(null)
</script>
<template>
<input ref="input" />
</template>
```

<span id="script-prefer-use-template-ref-good"></span>

**Correto**

`useTemplateRef<HTMLInputElement>('input')` do Vue 3.5+ torna essa referência de template explícita. O `error = ref(null)` sem associação permanece um dado comum e fica intencionalmente fora desta regra.

```vue annotate="add:2,3,4,5,6,10"
<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'
// Paired with the template ref below.
const input = useTemplateRef<HTMLInputElement>('input')
// A nullable data ref the template never binds as a ref.
const error = ref(null)
</script>
<template>
<input ref="input" />
<p>{{ error }}</p>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75) · [Todas as regras](all.md)

### `script/require-default-prop`

Exigir um valor padrão para toda prop opcional que não seja Boolean

[Incorreto](#script-require-default-prop-bad) · [Correto](#script-require-default-prop-good)

Severidade padrão: `error`  
Predefinições: _none_  
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
        "script/require-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-default-prop-bad"></span>

**Incorreto**

`name` e `age` são props opcionais de tempo de execução que não são Boolean e não têm valores padrão, deixando indefinidos seus valores quando a entrada é omitida.

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
props: {
// optional, non-Boolean, no default
name: String,
age: { type: Number },
}
}
</script>
```

<span id="script-require-default-prop-good"></span>

**Correto**

`name` recebe `default: ''`. `enabled` usa o valor padrão false implícito de Boolean, e o `id` obrigatório não precisa de um valor alternativo, ilustrando ambas as exceções.

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
props: {
name: { type: String, default: '' },
enabled: Boolean,                 // Boolean defaults to false
id: { type: Number, required: true },
}
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) · [Todas as regras](all.md)

### `script/require-explicit-emits`

Exigir que os eventos emitidos sejam declarados em defineEmits ou na opção emits

[Incorreto](#script-require-explicit-emits-bad) · [Correto](#script-require-explicit-emits-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/require-explicit-emits": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-emits-bad"></span>

**Incorreto**

A função de emissão capturada emite `save`, mas `defineEmits([])` não declara esse evento.

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits([]);
emit("save");
</script>
```

<span id="script-require-explicit-emits-good"></span>

**Correto**

Adicionar `"save"` à declaração torna o evento literal emitido parte do contrato explícito de eventos do componente.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(["save"]);
emit("save");
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L61) · [Todas as regras](all.md)

### `script/require-explicit-slots`

Exigir que os slots consumidos por useSlots() sejam tipados explicitamente com defineSlots&lt;...&gt;()

[Incorreto](#script-require-explicit-slots-bad) · [Correto](#script-require-explicit-slots-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/require-explicit-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-slots-bad"></span>

**Incorreto**

O `defineProps<{ id: number }>()` tipado estabelece a sintaxe TypeScript, mas setup usa `useSlots()` sem uma declaração de `defineSlots`. Assim, a regra encontra slots consumidos sem um contrato explícito de slots.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ id: number }>()
const slots = useSlots()
</script>
```

<span id="script-require-explicit-slots-good"></span>

**Correto**

`defineSlots` declara um slot `default` cujas props incluem `msg: string`; `useSlots()` agora aparece junto de um contrato de slots explicitamente tipado.

```vue annotate="add:2"
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) · [Todas as regras](all.md)

### `script/require-function-return-type`

Exigir anotações de tipo de retorno nas funções

[Incorreto](#script-require-function-return-type-bad) · [Correto](#script-require-function-return-type-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/require-function-return-type": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-function-return-type-bad"></span>

**Incorreto**

Tanto `add` quanto `greet` anotam seus parâmetros, mas omitem uma anotação de tipo de retorno; retornos inferidos não atendem a esta política de anotação explícita.

```vue annotate="remove:2,6"
<script setup lang="ts">
const add = (a: number, b: number) => {
return a + b
}

function greet(name: string) {
return `Hello, ${name}`
}
</script>
```

<span id="script-require-function-return-type-good"></span>

**Correto**

`add` declara `: number`, e `greet` declara `: string`, tornando os contratos de retorno explícitos sem alterar nenhum dos corpos.

```vue annotate="add:2,6"
<script setup lang="ts">
const add = (a: number, b: number): number => {
return a + b
}

function greet(name: string): string {
return `Hello, ${name}`
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L49) · [Todas as regras](all.md)

### `script/require-prop-type-constructor`

Exigir que os valores de `type` de props sejam construtores em vez de literais de string

[Incorreto](#script-require-prop-type-constructor-bad) · [Correto](#script-require-prop-type-constructor-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/require-prop-type-constructor": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-type-constructor-bad"></span>

**Incorreto**

As declarações de props usam as strings `"String"` e `"Number"` como tipos de tempo de execução, inclusive dentro do array de construtores. Essas strings não são funções construtoras.

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
props: {
// The type should be the `String` constructor, not the string "String".
name: "String",
age: { type: "Number" },
id: { type: ["String", "Number"] }
}
}
</script>
```

<span id="script-require-prop-type-constructor-good"></span>

**Correto**

As declarações usam os identificadores reais `String` e `Number`, inclusive no array de união `[String, Number]`.

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
props: {
name: String,
age: { type: Number },
id: { type: [String, Number] }
}
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L59) · [Todas as regras](all.md)

### `script/require-prop-types`

Exigir que toda prop declare um tipo

[Incorreto](#script-require-prop-types-bad) · [Correto](#script-require-prop-types-good)

Severidade padrão: `error`  
Predefinições: _none_  
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
        "script/require-prop-types": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-types-bad"></span>

**Incorreto**

A entrada do array declara apenas o nome `status`; o valor `null` e o descritor vazio também não declaram um tipo de prop em tempo de execução.

```vue annotate="remove:3,4,5,6,8,9"
<script lang="ts">
export default {
props: ['status']            // array form: no types
}

export default {
props: {
status: null,              // no type
other: {}                  // empty descriptor: no type
}
}
</script>
```

<span id="script-require-prop-types-good"></span>

**Correto**

`status: String` fornece um construtor na forma abreviada, e `other` fornece `type: Number` dentro de seu descritor. Ambas as props agora têm declarações de tipo.

```vue annotate="add:4,5"
<script lang="ts">
export default {
props: {
status: String,
other: { type: Number, default: 0 }
}
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58) · [Todas as regras](all.md)

### `script/require-symbol-provide`

Recomendar o uso de Symbol como chave de injeção para provide/inject

[Incorreto](#script-require-symbol-provide-bad) · [Correto](#script-require-symbol-provide-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/require-symbol-provide": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-symbol-provide-bad"></span>

**Incorreto**

`provide` e `inject` usam chaves de string literal como `'user'` e `'theme'`, que podem colidir com outro provedor que use a mesma grafia.

```vue annotate="remove:1,2,3,4,6,7"
<script setup lang="ts">
// String keys can collide
provide('user', user)
const user = inject('user')

// Magic strings are error-prone
provide('theme', { dark: true })
</script>
```

<span id="script-require-symbol-provide-good"></span>

**Correto**

O `UserKey` compartilhado é criado com `Symbol` e anotado como `InjectionKey<User>`; ambas as chamadas passam essa chave em vez de uma string literal.

```vue annotate="add:1,2,3,5,6,7,8,9"
<script lang="ts">
// Define injection key with Symbol
export const UserKey: InjectionKey<User> = Symbol('user')

// Provide with Symbol
provide(UserKey, user)

// Inject with Symbol
const user = inject(UserKey)
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) · [Todas as regras](all.md)

### `script/require-typed-object-prop`

Exigir um tipo explícito em uma prop cujo tipo de tempo de execução seja `Object` ou `Array`

[Incorreto](#script-require-typed-object-prop-bad) · [Correto](#script-require-typed-object-prop-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/require-typed-object-prop": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-object-prop-bad"></span>

**Incorreto**

Os construtores simples `Object` e `Array` descrevem apenas categorias amplas de tempo de execução, então nem `user` nem a estrutura dos elementos de `items` têm um tipo estático explícito.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

<span id="script-require-typed-object-prop-good"></span>

**Correto**

`PropType<User>` e `PropType<User[]>` adicionam os tipos de objeto e de elemento enquanto mantêm os mesmos construtores de tempo de execução.

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) · [Todas as regras](all.md)

### `script/require-typed-ref`

Exigir um argumento de tipo explícito em um ref() inicializado sem valor, com null ou com undefined

[Incorreto](#script-require-typed-ref-bad) · [Correto](#script-require-typed-ref-good)

Severidade padrão: `warning`  
Predefinições: _none_  
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
        "script/require-typed-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-ref-bad"></span>

**Incorreto**

As chamadas do `ref` importado não têm um argumento de tipo nem um valor inicial útil: a ausência de argumento, `null` e `undefined` não permitem inferir o tipo pretendido para o valor futuro.

```vue annotate="remove:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref()           // Ref<undefined>
const b = ref(null)       // Ref<null>
const c = ref(undefined)  // Ref<undefined>
</script>
```

<span id="script-require-typed-ref-good"></span>

**Correto**

Os argumentos de tipo explícitos descrevem os refs de string e de User que aceita null. `ref(0)` já tem um inicializador numérico concreto e pode depender da inferência.

```vue annotate="add:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref<string>()
const b = ref<User | null>(null)
const c = ref(0)          // inferred Ref<number>
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55) · [Todas as regras](all.md)

### `script/require-valid-default-prop`

Exigir que o valor padrão de uma prop seja válido para seu tipo declarado

[Incorreto](#script-require-valid-default-prop-bad) · [Correto](#script-require-valid-default-prop-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/require-valid-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-valid-default-prop-bad"></span>

**Incorreto**

As props Number e Boolean recebem valores padrão escalares incompatíveis, e as props Array e Object usam valores literais compartilhados em vez de funções de criação.

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
props: {
count: { type: Number, default: '0' },     // string default for Number
enabled: { type: Boolean, default: 1 },     // non-boolean default for Boolean
items: { type: Array, default: [] },        // literal must be a factory
config: { type: Object, default: {} }       // literal must be a factory
}
}
</script>
```

<span id="script-require-valid-default-prop-good"></span>

**Correto**

Os valores padrão escalares passam a ser `0` e `false`; os valores padrão de array e objeto passam a ser funções que retornam novos valores. O exemplo `[String, Number]` aceita seu valor padrão de string porque ele corresponde a um dos tipos declarados.

```vue annotate="add:4,5,6,7,8"
<script lang="ts">
export default {
props: {
count: { type: Number, default: 0 },
enabled: { type: Boolean, default: false },
items: { type: Array, default: () => [] },
config: { type: Object, default: () => ({}) },
label: { type: [String, Number], default: '' }
}
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) · [Todas as regras](all.md)

### `script/return-in-computed-property`

Exigir um valor de retorno em todo getter computado

[Incorreto](#script-return-in-computed-property-bad) · [Correto](#script-return-in-computed-property-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/return-in-computed-property": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-computed-property-bad"></span>

**Incorreto**

O getter computado com corpo de bloco avalia `1 + 2`, mas nunca retorna o resultado, deixando o valor computado undefined.

```vue annotate="remove:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { 1 + 2; });
</script>
```

<span id="script-return-in-computed-property-good"></span>

**Correto**

`return 1 + 2` transforma a expressão no valor retornado pelo getter. A regra procura um return que retorne um valor no próprio getter, não apenas uma instrução de expressão.

```vue annotate="add:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { return 1 + 2; });
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) · [Todas as regras](all.md)

### `script/return-in-emits-validator`

Exigir um valor de retorno em todo validador de emits da Options API

[Incorreto](#script-return-in-emits-validator-bad) · [Correto](#script-return-in-emits-validator-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Scripts JS/TS em SFCs Vue; os exemplos mostram a forma relevante de Options API ou script setup  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Use uma função de seta com corpo em bloco para o filtro de SFC atualmente suportado. O validador subjacente também trata a sintaxe abreviada de métodos, mas o pré-filtro atual de SFC não encaminha essa forma de modo confiável.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-emits-validator": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-emits-validator-bad"></span>

**Incorreto**

O validador `submit` registra a carga útil, mas não retorna um resultado de validação, então seu corpo de bloco produz undefined.

```vue annotate="remove:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

<span id="script-return-in-emits-validator-good"></span>

**Correto**

`return payload != null` fornece um resultado booleano de validação para a carga útil enviada em vez de encerrar sem um valor retornado.

```vue annotate="add:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L59) · [Todas as regras](all.md)

### `script/valid-define-emits`

Exigir uso válido de defineEmits() (sem argumentos de tipo e de tempo de execução juntos, sem referências locais, uma única chamada)

[Incorreto](#script-valid-define-emits-bad) · [Correto](#script-valid-define-emits-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/valid-define-emits": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-emits-bad"></span>

**Incorreto**

A mesma chamada de `defineEmits` fornece tanto um argumento de tipo quanto o array de tempo de execução `["save"]`, misturando duas declarações mutuamente exclusivas.

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

<span id="script-valid-define-emits-good"></span>

**Correto**

Remover o argumento de tempo de execução deixa uma única declaração de evento baseada em tipos para `save`.

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) · [Todas as regras](all.md)

### `script/valid-define-options`

Exigir uso válido de defineOptions() (um único argumento de objeto, sem props/emits/expose/slots)

[Incorreto](#script-valid-define-options-bad) · [Correto](#script-valid-define-options-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/valid-define-options": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-options-bad"></span>

**Incorreto**

A primeira chamada coloca a declaração específica de `props` dentro de `defineOptions`; as chamadas posteriores também repetem a macro e incluem um argumento que não é objeto. Elas ilustram as restrições de formato proibido e de chamadas repetidas.

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
defineOptions({ props: ['foo'] })   // use defineProps instead
defineOptions({ name: 'Foo' })
defineOptions({ name: 'Bar' })      // duplicate call
defineOptions('Foo')                // not an object literal
</script>
```

<span id="script-valid-define-options-good"></span>

**Correto**

Uma única chamada de `defineOptions` recebe um objeto contendo apenas as opções comuns suportadas `name` e `inheritAttrs`.

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: 'Foo', inheritAttrs: false })
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) · [Todas as regras](all.md)

### `script/valid-define-props`

Exigir uso válido de defineProps() (uma única chamada, sem argumentos de tipo e de tempo de execução juntos, sem referências locais)

[Incorreto](#script-valid-define-props-bad) · [Correto](#script-valid-define-props-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/valid-define-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-props-bad"></span>

**Incorreto**

A mesma chamada de `defineProps` fornece tanto `{ title: string }` como argumento de tipo quanto `{ title: String }` como argumento de tempo de execução, combinação que o compilador não permite.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

<span id="script-valid-define-props-good"></span>

**Correto**

Remover o objeto de tempo de execução deixa uma única declaração baseada em tipos para `title` em vez de combinar ambas as formas de declaração.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) · [Todas as regras](all.md)

### `script/valid-next-tick`

Exigir que o resultado de uma chamada de nextTick() seja aguardado, encadeado ou receba uma função de retorno

[Incorreto](#script-valid-next-tick-bad) · [Correto](#script-valid-next-tick-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/valid-next-tick": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-next-tick-bad"></span>

**Incorreto**

O `nextTick()` importado é uma expressão isolada sem função de retorno, então a Promise retornada é ignorada e nenhum trabalho aguarda a atualização do DOM.

```vue annotate="remove:3"
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

<span id="script-valid-next-tick-good"></span>

**Correto**

`await nextTick()` consome a Promise e aguarda explicitamente a próxima atualização do DOM antes que o código subsequente de setup continue.

```vue annotate="add:3"
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) · [Todas as regras](all.md)

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

### `type/no-floating-promises`

Proibir Promises soltas (não tratadas)

[Incorreto](#type-no-floating-promises-bad) · [Correto](#type-no-floating-promises-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

As verificações com informações de tipos usam o runtime nativo do Corsa e o projeto TypeScript. `typeAware` sozinho não ativa uma regra opcional.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-floating-promises": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-floating-promises-bad"></span>

**Incorreto**

A função assíncrona `save` retorna uma Promise, mas a chamada isolada de `save()` não a aguarda nem a retorna, e não sinaliza explicitamente um descarte intencional.

```vue annotate="remove:3"
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

<span id="type-no-floating-promises-good"></span>

**Correto**

`void save()` sinaliza explicitamente a intenção de executar sem aguardar, aceita por esta regra. É um marcador explícito de descarte, não um manipulador de rejeição.

```vue annotate="add:3"
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) · [Todas as regras](all.md)

### `type/no-reactivity-loss`

Proibir cópias estáticas simples de valores reativos em atribuições e chamadas

[Incorreto](#type-no-reactivity-loss-bad) · [Correto](#type-no-reactivity-loss-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

As verificações com informações de tipos usam o runtime nativo do Corsa e o projeto TypeScript. `typeAware` sozinho não ativa uma regra opcional.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-reactivity-loss": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-reactivity-loss-bad"></span>

**Incorreto**

`const count = state.count` obtém uma cópia numérica simples da propriedade reativa, então atualizações posteriores de `state.count` não se refletem nessa variável.

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0 });
const count = state.count;
</script>
```

<span id="type-no-reactivity-loss-good"></span>

**Correto**

`toRef(state, "count")` mantém `count` vinculado à propriedade reativa original em vez de copiar seu valor primitivo atual.

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRef } from "vue";
const state = reactive({ count: 0 });
const count = toRef(state, "count");
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) · [Todas as regras](all.md)

### `type/no-unsafe-template-binding`

Proibir vinculações de template que resultam em tipos inseguros

[Incorreto](#type-no-unsafe-template-binding-bad) · [Correto](#type-no-unsafe-template-binding-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

As verificações com informações de tipos usam o runtime nativo do Corsa e o projeto TypeScript. `typeAware` sozinho não ativa uma regra opcional.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-unsafe-template-binding": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-unsafe-template-binding-bad"></span>

**Incorreto**

O `value` interpolado é explicitamente tipado como `any`, então o verificador não consegue atribuir um tipo concreto seguro à vinculação do template.

```vue annotate="remove:2"
<script setup lang="ts">
const value: any = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

<span id="type-no-unsafe-template-binding-good"></span>

**Correto**

Alterar a anotação para `string` fornece à mesma interpolação um tipo concreto que pode ser verificado, sem alterar o valor renderizado.

```vue annotate="add:2"
<script setup lang="ts">
const value: string = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) · [Todas as regras](all.md)

### `type/require-typed-emits`

Exigir uma definição de tipo para defineEmits

[Incorreto](#type-require-typed-emits-bad) · [Correto](#type-require-typed-emits-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

As verificações com informações de tipos usam o runtime nativo do Corsa e o projeto TypeScript. `typeAware` sozinho não ativa uma regra opcional.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-emits": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-emits-bad"></span>

**Incorreto**

O `defineEmits(["save"])` que usa apenas um array declara o nome do evento sem um contrato tipado de carga útil.

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

<span id="type-require-typed-emits-good"></span>

**Correto**

`defineEmits<{ save: [] }>()` declara o evento tipado `save` com uma tupla vazia de carga útil, indicando explicitamente que ele não recebe argumentos de carga útil.

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) · [Todas as regras](all.md)

### `type/require-typed-props`

Exigir uma definição de tipo para defineProps

[Incorreto](#type-require-typed-props-bad) · [Correto](#type-require-typed-props-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

As verificações com informações de tipos usam o runtime nativo do Corsa e o projeto TypeScript. `typeAware` sozinho não ativa uma regra opcional.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-props": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-props-bad"></span>

**Incorreto**

O `defineProps(["title"])` que usa apenas um array declara `title` pelo nome sem lhe atribuir um tipo.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps(["title"]);
</script>
```

<span id="type-require-typed-props-good"></span>

**Correto**

`defineProps<{ title: string }>()` atribui a `title` um tipo string explícito em vez de uma declaração de tempo de execução que contém apenas o nome.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57) · [Todas as regras](all.md)

### `type/strict-boolean-expressions`

Exigir expressões booleanas seguras nas condições de script e template

[Incorreto](#type-strict-boolean-expressions-bad) · [Correto](#type-strict-boolean-expressions-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Informações de tipos em scripts e templates de SFCs Vue, para as construções mostradas abaixo  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

Ative typeAware e esta regra explicitamente. Por padrão, números que podem ser nulos são proibidos, enquanto números não nulos são permitidos.

As verificações com informações de tipos usam o runtime nativo do Corsa e o projeto TypeScript. `typeAware` sozinho não ativa uma regra opcional.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/strict-boolean-expressions": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-strict-boolean-expressions-bad"></span>

**Incorreto**

`if (count)` depende da conversão implícita para booleano de uma variável numérica que pode estar ausente em vez de um teste booleano explícito; também confunde zero com ausência.

```vue annotate="remove:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

<span id="type-strict-boolean-expressions-good"></span>

**Correto**

`count !== undefined && count > 0` testa separadamente a presença e a positividade, produzindo uma condição booleana explícita após restringir o tipo do valor opcional.

```vue annotate="add:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) · [Todas as regras](all.md)

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

### `vue/a11y-img-alt`

Exigir o atributo alt nas imagens para garantir acessibilidade

[Incorreto](#vue-a11y-img-alt-bad) · [Correto](#vue-a11y-img-alt-good)

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
        "vue/a11y-img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-a11y-img-alt-bad"></span>

**Incorreto**

Nem a imagem estática nem a imagem de origem dinâmica fornecem um atributo alt.

```vue annotate="remove:2,3"
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

<span id="vue-a11y-img-alt-good"></span>

**Correto**

Imagens informativas recebem um texto alt descritivo; imagens decorativas recebem um alt vazio; a imagem dinâmica vincula sua descrição.

```vue annotate="add:2,3,4,5,6,7,8,9"
<template>
<!-- Informative image -->
<img src="/photo.jpg" alt="Team photo from company retreat" />

<!-- Decorative image (empty alt) -->
<img src="/decoration.svg" alt="" />

<!-- Dynamic alt -->
<img :src="photo" :alt="photoDescription" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33) · [Todas as regras](all.md)

### `vue/attribute-hyphenation`

Aplicar um padrão de nomes de atributos em componentes personalizados

[Incorreto](#vue-attribute-hyphenation-bad) · [Correto](#vue-attribute-hyphenation-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
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
        "vue/attribute-hyphenation": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-attribute-hyphenation-bad"></span>

**Incorreto**

O atributo do componente usa a grafia camelCase firstName.

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

<span id="vue-attribute-hyphenation-good"></span>

**Correto**

A grafia first-name segue a convenção configurada de atributos de componente separados por hífen.

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) · [Todas as regras](all.md)

### `vue/attribute-order`

Aplicar uma ordem consistente aos atributos

[Incorreto](#vue-attribute-order-bad) · [Correto](#vue-attribute-order-good)

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
        "vue/attribute-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-attribute-order-bad"></span>

**Incorreto**

O manipulador de evento aparece antes da diretiva estrutural v-if e do atributo comum id.

```vue annotate="remove:2"
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

<span id="vue-attribute-order-good"></span>

**Correto**

v-if vem primeiro, seguido de id e do manipulador de evento, conforme a ordem da regra.

```vue annotate="add:2"
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [Todas as regras](all.md)

### `vue/component-definition-name-casing`

Exigir PascalCase ou kebab-case nos nomes de definição de componentes

[Incorreto](#vue-component-definition-name-casing-bad) · [Correto](#vue-component-definition-name-casing-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

O nome de arquivo do componente é verificado. PascalCase e kebab-case são aceitos; o uso misturado de maiúsculas e minúsculas é reportado.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/component-definition-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-component-definition-name-casing-bad"></span>

**Incorreto**

O nome de arquivo myComponent.vue mistura uma inicial minúscula com uma letra maiúscula interna, em vez de usar PascalCase ou kebab-case.

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

<span id="vue-component-definition-name-casing-good"></span>

**Correto**

Renomear o arquivo para MyComponent.vue aplica PascalCase; o conteúdo do template permanece igual.

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) · [Todas as regras](all.md)

### `vue/component-name-in-template-casing`

Aplicar um padrão específico de maiúsculas e minúsculas aos nomes de componentes nos templates

[Incorreto](#vue-component-name-in-template-casing-bad) · [Correto](#vue-component-name-in-template-casing-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
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
        "vue/component-name-in-template-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-component-name-in-template-casing-bad"></span>

**Incorreto**

O componente é escrito em kebab-case e camelCase, embora a convenção seja PascalCase.

```vue annotate="remove:5,6"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <my-component />
  <myComponent />
</template>
```

<span id="vue-component-name-in-template-casing-good"></span>

**Correto**

MyComponent usa PascalCase; a sintaxe nativa de slot permanece em minúsculas.

```vue annotate="add:5,6,7"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <MyComponent />
  <RouterView />
  <slot />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31) · [Todas as regras](all.md)

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

### `vue/html-button-has-type`

Exigir um type explícito e válido nos elementos button

[Incorreto](#vue-html-button-has-type-bad) · [Correto](#vue-html-button-has-type-good)

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
        "vue/html-button-has-type": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-button-has-type-bad"></span>

**Incorreto**

Um botão omite type e outro fornece o tipo foo, que não é aceito.

```vue annotate="remove:2,3"
<template>
<button>Click</button>
<button type="foo">Click</button>
</template>
```

<span id="vue-html-button-has-type-good"></span>

**Correto**

Os botões especificam button, submit ou reset; um type vinculado é tratado como dinâmico.

```vue annotate="add:2,3,4,5"
<template>
<button type="button">Click</button>
<button type="submit">Save</button>
<button type="reset">Reset</button>
<button :type="dynamicType">Click</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39) · [Todas as regras](all.md)

### `vue/html-quotes`

Aplicar um padrão de aspas aos atributos HTML

[Incorreto](#vue-html-quotes-bad) · [Correto](#vue-html-quotes-good)

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
        "vue/html-quotes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-quotes-bad"></span>

**Incorreto**

Os atributos usam aspas simples ou nenhuma aspa, em vez da convenção de aspas duplas.

```vue annotate="remove:2,3,4"
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

<span id="vue-html-quotes-good"></span>

**Correto**

Tanto os atributos comuns quanto as expressões de diretivas usam aspas duplas.

```vue annotate="add:2,3"
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) · [Todas as regras](all.md)

### `vue/html-self-closing`

Aplicar um padrão de tags com fechamento automático

[Incorreto](#vue-html-self-closing-bad) · [Correto](#vue-html-self-closing-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Disponível para os diagnósticos compatíveis  
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
        "vue/html-self-closing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-self-closing-bad"></span>

**Incorreto**

O componente vazio usa uma tag de fechamento separada, enquanto os elementos vazios img e br omitem a grafia de fechamento automático configurada.

```vue annotate="remove:2,3,4"
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

<span id="vue-html-self-closing-good"></span>

**Correto**

O componente e os elementos vazios usam a sintaxe de fechamento automático; uma div com conteúdo mantém sua tag de fechamento.

```vue annotate="add:2,3,4,5,6,7"
<template>
  <MyComponent />
  <div></div>
  <div />
  <img />
  <br />
  <div>content</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) · [Todas as regras](all.md)

### `vue/max-template-complexity`

Limitar a complexidade do próprio template de um componente, tanto ciclomática quanto cognitiva

[Incorreto](#vue-max-template-complexity-bad) · [Correto](#vue-max-template-complexity-good)

Severidade padrão: `warning`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Bad tem complexidade ciclomática 13 e complexidade cognitiva 25 (limites: 11 e 16). Cada componente é medido separadamente; apenas templates HTML inline são suportados.

Veja [o cálculo da complexidade e os limites dos componentes](../guide/cross-file-complexity.md) para conhecer as contribuições às duas pontuações do exemplo.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/max-template-complexity": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-max-template-complexity-bad"></span>

**Incorreto**

As ramificações, o laço, o conteúdo de slot e as decisões em expressões escritos pelo componente pai produzem pontuações de 13 e 25, acima dos limites padrão de 11 e 16.

```vue annotate="remove:1,2,3,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19"
<script setup lang="ts">
defineProps<{ rows: Row[] }>();
</script>
<template>
  <section>
    <h1>{{ user ? user.name : 'Guest' }}</h1>
    <DataTable :rows="rows">
      <template #cell="{ row, column }">
        <span v-if="column.key === 'status'" :class="row.active ? 'on' : 'off'">{{ row.status ?? 'unknown' }}</span>
        <a v-else-if="column.key === 'link' && row.url" :href="row.url">{{ row.label }}</a>
        <template v-else>
          <em v-for="tag in row.tags" :key="tag">
            <b v-if="tag.pinned || tag.starred">{{ tag.hot ? '!' : '' }}</b>
          </em>
        </template>
      </template>
    </DataTable>
    <p v-if="!rows.length && !loading">No data</p>
  </section>
</template>
```

<span id="vue-max-template-complexity-good"></span>

**Correto**

O template pai delega a renderização a RowList e mantém um v-if; suas próprias pontuações são 2 e 1.

```vue annotate="add:2"
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) · [Todas as regras](all.md)

### `vue/multi-word-component-names`

Exigir nomes de componentes com mais de uma palavra

[Incorreto](#vue-multi-word-component-names-bad) · [Correto](#vue-multi-word-component-names-good)

Severidade padrão: `error`  
Predefinições: `essential`, `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

A ocorrência diagnosticada é o nome de arquivo. Renomeie o mesmo componente; alterar uma tag filha não corrige o problema.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/multi-word-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-multi-word-component-names-bad"></span>

**Incorreto**

Item.vue dá ao componente um nome de uma única palavra.

`Item.vue`

```vue
<template><p>Item</p></template>
```

<span id="vue-multi-word-component-names-good"></span>

**Correto**

TodoItem.vue dá ao mesmo template um nome de componente com mais de uma palavra.

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) · [Todas as regras](all.md)

### `vue/mustache-interpolation-spacing`

Aplicar espaçamento consistente dentro das interpolações com chaves duplas

[Incorreto](#vue-mustache-interpolation-spacing-bad) · [Correto](#vue-mustache-interpolation-spacing-good)

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
        "vue/mustache-interpolation-spacing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-mustache-interpolation-spacing-bad"></span>

**Incorreto**

A interpolação de texto não tem um espaço junto a um ou a ambos os delimitadores.

```vue annotate="remove:2,3,4"
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

<span id="vue-mustache-interpolation-spacing-good"></span>

**Correto**

Espaços separam a expressão dos delimitadores de abertura e fechamento das chaves duplas.

```vue annotate="add:2,3,4"
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) · [Todas as regras](all.md)

### `vue/no-array-index-key`

Proibir o uso direto da variável de índice de v-for como :key

[Incorreto](#vue-no-array-index-key-bad) · [Correto](#vue-no-array-index-key-good)

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
        "vue/no-array-index-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-array-index-key-bad"></span>

**Incorreto**

A chave da lista é seu índice atual, então a identidade do item muda quando a lista é reordenada.

```vue annotate="remove:2"
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

<span id="vue-no-array-index-key-good"></span>

**Correto**

A chave vem de item.id, preservando a identidade de cada item quando sua posição muda.

```vue annotate="add:2"
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) · [Todas as regras](all.md)

### `vue/no-bare-strings-in-template`

Proibir texto legível por pessoas diretamente no template quando ele deve ser internacionalizado

[Incorreto](#vue-no-bare-strings-in-template-bad) · [Correto](#vue-no-bare-strings-in-template-good)

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
        "vue/no-bare-strings-in-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-bare-strings-in-template-bad"></span>

**Incorreto**

O texto visível e os atributos de identificação incorporam strings sem tradução diretamente no template.

```vue annotate="remove:2,3,4,5"
<template>
<div>hello</div>
<img alt="a cat" />
<input placeholder="Search" />
<button title="Close">x</button>
</template>
```

<span id="vue-no-bare-strings-in-template-good"></span>

**Correto**

O conteúdo traduzível chama $t; os exemplos com pontuação e apenas números são exceções permitidas.

```vue annotate="add:2,3,4,5,6"
<template>
<div>{{ $t('hello') }}</div>
<img :alt="$t('cat')" />
<div>-</div>
<div>123</div>
<button :title="$t('close')">{{ $t('x') }}</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L47) · [Todas as regras](all.md)

### `vue/no-boolean-attr-value`

Proibir valores explícitos em atributos HTML booleanos

[Incorreto](#vue-no-boolean-attr-value-bad) · [Correto](#vue-no-boolean-attr-value-good)

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
        "vue/no-boolean-attr-value": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-boolean-attr-value-bad"></span>

**Incorreto**

Os atributos booleanos disabled e checked contêm valores de string redundantes.

```vue annotate="remove:2,3,4"
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

<span id="vue-no-boolean-attr-value-good"></span>

**Correto**

A presença de cada atributo booleano expressa o mesmo estado ativado, sem um valor.

```vue annotate="add:2,3,4"
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) · [Todas as regras](all.md)

### `vue/no-child-content`

Proibir conteúdo filho ao usar v-html ou v-text

[Incorreto](#vue-no-child-content-bad) · [Correto](#vue-no-child-content-good)

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
        "vue/no-child-content": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-child-content-bad"></span>

**Incorreto**

v-text substitui o conteúdo do parágrafo, então o texto alternativo escrito no template não pode ser preservado por essa diretiva.

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

<span id="vue-no-child-content-good"></span>

**Correto**

Remover o texto filho deixa v-text como a única fonte de conteúdo do parágrafo.

```vue annotate="add:2"
<template>
  <p v-text="message" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [Todas as regras](all.md)

### `vue/no-deprecated-filter`

Proibir a sintaxe obsoleta de filtros do Vue 2 com o operador de barra vertical

[Incorreto](#vue-no-deprecated-filter-bad) · [Correto](#vue-no-deprecated-filter-good)

Severidade padrão: `error`  
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
        "vue/no-deprecated-filter": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-filter-bad"></span>

**Incorreto**

A barra vertical usa a sintaxe de filtros removida do Vue para aplicar capitalize.

```vue annotate="remove:2"
<template>
{{ message | capitalize }}
</template>
```

<span id="vue-no-deprecated-filter-good"></span>

**Correto**

Chamar capitalize(message) aplica a transformação como uma expressão comum.

```vue annotate="add:2"
<template>
{{ capitalize(message) }}
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L53) · [Todas as regras](all.md)

### `vue/no-deprecated-functional-template`

Proibir o atributo `functional` no `<template>` de um SFC

[Incorreto](#vue-no-deprecated-functional-template-bad) · [Correto](#vue-no-deprecated-functional-template-good)

Severidade padrão: `error`  
Predefinições: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
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
        "vue/no-deprecated-functional-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-functional-template-bad"></span>

**Incorreto**

O template do SFC tem o atributo functional removido e lê o antigo contexto props.

```vue annotate="remove:1,2"
<template functional>
<div>{{ props.msg }}</div>
</template>
```

<span id="vue-no-deprecated-functional-template-good"></span>

**Correto**

O template comum omite functional e lê diretamente a variável msg exposta pelo componente.

```vue annotate="add:1,2"
<template>
<div>{{ msg }}</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) · [Todas as regras](all.md)

### `vue/no-deprecated-html-element-is`

Proibir o atributo `is` em elementos HTML nativos

[Incorreto](#vue-no-deprecated-html-element-is-bad) · [Correto](#vue-no-deprecated-html-element-is-good)

Severidade padrão: `error`  
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
        "vue/no-deprecated-html-element-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-html-element-is-bad"></span>

**Incorreto**

Uma div nativa usa o antigo atributo is sem prefixo para solicitar um componente Vue.

```vue annotate="remove:2"
<template>
<div is="MyComponent" />
</template>
```

<span id="vue-no-deprecated-html-element-is-good"></span>

**Correto**

Um componente dinâmico usa :is; a forma no elemento nativo usa explicitamente o prefixo vue:.

```vue annotate="add:2,3"
<template>
<component :is="MyComponent" />
<div is="vue:MyComponent" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) · [Todas as regras](all.md)

### `vue/no-deprecated-inline-template`

Proibir o atributo obsoleto `inline-template`

[Incorreto](#vue-no-deprecated-inline-template-bad) · [Correto](#vue-no-deprecated-inline-template-good)

Severidade padrão: `error`  
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
        "vue/no-deprecated-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-inline-template-bad"></span>

**Incorreto**

Card usa o atributo obsoleto inline-template para o conteúdo fornecido.

```vue annotate="remove:2"
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

<span id="vue-no-deprecated-inline-template-good"></span>

**Correto**

O mesmo conteúdo é passado normalmente, sem o atributo inline-template.

```vue annotate="add:2"
<template>
<Card><p>Details</p></Card>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) · [Todas as regras](all.md)

### `vue/no-deprecated-router-link-tag-prop`

Proibir a prop `tag` em &lt;router-link&gt;

[Incorreto](#vue-no-deprecated-router-link-tag-prop-bad) · [Correto](#vue-no-deprecated-router-link-tag-prop-good)

Severidade padrão: `error`  
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
        "vue/no-deprecated-router-link-tag-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-router-link-tag-prop-bad"></span>

**Incorreto**

RouterLink usa a prop tag removida para solicitar um elemento button.

```vue annotate="remove:2"
<template>
<router-link to="/home" tag="button">Home</router-link>
</template>
```

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

**Correto**

O slot fornece navigate a um botão escrito explicitamente no template.

```vue annotate="add:2,3,4"
<template>
<router-link to="/home" v-slot="{ navigate }">
<button @click="navigate">Home</button>
</router-link>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) · [Todas as regras](all.md)

### `vue/no-deprecated-scope-attribute`

Proibir o atributo obsoleto `scope` em &lt;template&gt;

[Incorreto](#vue-no-deprecated-scope-attribute-bad) · [Correto](#vue-no-deprecated-scope-attribute-good)

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
        "vue/no-deprecated-scope-attribute": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-scope-attribute-bad"></span>

**Incorreto**

O template do slot declara props pelo atributo obsoleto scope.

```vue annotate="remove:2"
<template>
<Card><template scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-scope-attribute-good"></span>

**Correto**

A diretiva do slot padrão declara a mesma variável props pela sintaxe atual de slots.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) · [Todas as regras](all.md)

### `vue/no-deprecated-slot-attribute`

Proibir o atributo obsoleto `slot`

[Incorreto](#vue-no-deprecated-slot-attribute-bad) · [Correto](#vue-no-deprecated-slot-attribute-good)

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
        "vue/no-deprecated-slot-attribute": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-slot-attribute-bad"></span>

**Incorreto**

O slot header é selecionado pelo antigo atributo slot.

```vue annotate="remove:3,4"
<template>
<Foo>
<template slot="header"><h1>Title</h1></template>
<div :slot="name">Title</div>
</Foo>
</template>
```

<span id="vue-no-deprecated-slot-attribute-good"></span>

**Correto**

v-slot:header seleciona explicitamente o slot header com a diretiva atual.

```vue annotate="add:3"
<template>
<Foo>
<template v-slot:header><h1>Title</h1></template>
</Foo>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) · [Todas as regras](all.md)

### `vue/no-deprecated-slot-scope-attribute`

Proibir o atributo obsoleto `slot-scope`

[Incorreto](#vue-no-deprecated-slot-scope-attribute-bad) · [Correto](#vue-no-deprecated-slot-scope-attribute-good)

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
        "vue/no-deprecated-slot-scope-attribute": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-slot-scope-attribute-bad"></span>

**Incorreto**

O template recebe as props do slot pelo atributo obsoleto slot-scope.

```vue annotate="remove:2"
<template>
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

**Correto**

A diretiva #default recebe essas props sem slot-scope.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) · [Todas as regras](all.md)

### `vue/no-deprecated-v-bind-sync`

Proibir o modificador obsoleto `.sync` em `v-bind`

[Incorreto](#vue-no-deprecated-v-bind-sync-bad) · [Correto](#vue-no-deprecated-v-bind-sync-good)

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
        "vue/no-deprecated-v-bind-sync": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-v-bind-sync-bad"></span>

**Incorreto**

As vinculações usam o modificador .sync removido, inclusive em combinação com .camel.

```vue annotate="remove:2,3,4"
<template>
<MyComponent :title.sync="title" />
<MyComponent v-bind:title.sync="title" />
<MyComponent :title.sync.camel="title" />
</template>
```

<span id="vue-no-deprecated-v-bind-sync-good"></span>

**Correto**

Use uma vinculação comum unidirecional de title ou v-model:title quando for necessário um canal de atualização.

```vue annotate="add:2,3"
<template>
<MyComponent :title="title" />
<MyComponent v-model:title="title" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) · [Todas as regras](all.md)

### `vue/no-deprecated-v-on-native-modifier`

Proibir o modificador obsoleto `.native` em `v-on`

[Incorreto](#vue-no-deprecated-v-on-native-modifier-bad) · [Correto](#vue-no-deprecated-v-on-native-modifier-good)

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
        "vue/no-deprecated-v-on-native-modifier": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-v-on-native-modifier-bad"></span>

**Incorreto**

Os manipuladores do componente usam o modificador de evento .native removido.

```vue annotate="remove:2,3,4"
<template>
<MyComponent @click.native="handler" />
<MyComponent v-on:click.native="handler" />
<MyComponent @click.native.stop="handler" />
</template>
```

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

**Correto**

Os manipuladores omitem .native e preservam outros modificadores de evento, como .stop.

```vue annotate="add:2,3"
<template>
<MyComponent @click="handler" />
<MyComponent @click.stop="handler" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) · [Todas as regras](all.md)

### `vue/no-deprecated-v-on-number-modifiers`

Proibir modificadores numéricos obsoletos de `keyCode` em `v-on`

[Incorreto](#vue-no-deprecated-v-on-number-modifiers-bad) · [Correto](#vue-no-deprecated-v-on-number-modifiers-good)

Severidade padrão: `error`  
Predefinições: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
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
        "vue/no-deprecated-v-on-number-modifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-v-on-number-modifiers-bad"></span>

**Incorreto**

Os manipuladores de teclado identificam as teclas pelos códigos numéricos removidos 13 e 27.

```vue annotate="remove:2,3,4"
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

**Correto**

Os manipuladores usam os modificadores de tecla nomeados enter e esc.

```vue annotate="add:2,3"
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) · [Todas as regras](all.md)

### `vue/no-dupe-v-else-if`

Proibir condições duplicadas em cadeias de `v-if` / `v-else-if`

[Incorreto](#vue-no-dupe-v-else-if-bad) · [Correto](#vue-no-dupe-v-else-if-good)

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
        "vue/no-dupe-v-else-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-dupe-v-else-if-bad"></span>

**Incorreto**

O else-if repete a condição ready já testada pelo primeiro ramo, tornando esse ramo posterior inacessível.

```vue annotate="remove:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

<span id="vue-no-dupe-v-else-if-good"></span>

**Correto**

O segundo ramo testa loading, um estado distinto que pode alcançar o else-if.

```vue annotate="add:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) · [Todas as regras](all.md)

### `vue/no-duplicate-attributes`

Proibir atributos duplicados no mesmo elemento

[Incorreto](#vue-no-duplicate-attributes-bad) · [Correto](#vue-no-duplicate-attributes-good)

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
        "vue/no-duplicate-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-duplicate-attributes-bad"></span>

**Incorreto**

O mesmo botão declara class duas vezes, em vez de usar um único valor combinado de class.

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

<span id="vue-no-duplicate-attributes-good"></span>

**Correto**

Os dois nomes de classe aparecem em um único atributo class.

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) · [Todas as regras](all.md)

### `vue/no-empty-component-block`

Proibir blocos vazios em SFCs

[Incorreto](#vue-no-empty-component-block-bad) · [Correto](#vue-no-empty-component-block-good)

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
        "vue/no-empty-component-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-empty-component-block-bad"></span>

**Incorreto**

Os blocos template, script e style não contêm conteúdo significativo.

```vue annotate="remove:1,3,5"
<template></template>

<script></script>

<style>
</style>
```

<span id="vue-no-empty-component-block-good"></span>

**Correto**

Cada bloco mantido contém marcação, declarações de script ou declarações de estilo de fato.

```vue annotate="add:1,2,3,5,6,7,9,10"
<template>
<div>Hello</div>
</template>

<script setup>
const message = "Hello";
</script>

<style scoped>
.button { color: red; }
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42) · [Todas as regras](all.md)

### `vue/no-inline-style`

Desencorajar o uso de atributos de estilo inline

[Incorreto](#vue-no-inline-style-bad) · [Correto](#vue-no-inline-style-good)

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
        "vue/no-inline-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-inline-style-bad"></span>

**Incorreto**

O atributo style estático incorpora a declaração de cor no elemento.

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

<span id="vue-no-inline-style-good"></span>

**Correto**

Classes expressam a cor fixa; a largura dependente de ratio permanece como uma vinculação dinâmica de estilo, fora da verificação de atributos estáticos.

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) · [Todas as regras](all.md)

### `vue/no-invalid-html-attribute`

Proibir valores estáticos inválidos para atributos HTML

[Incorreto](#vue-no-invalid-html-attribute-bad) · [Correto](#vue-no-invalid-html-attribute-good)

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
        "vue/no-invalid-html-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-invalid-html-attribute-bad"></span>

**Incorreto**

A âncora usa stylesheet como valor de rel, embora esse valor pertença a elementos link de folhas de estilo.

```vue annotate="remove:2"
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

<span id="vue-no-invalid-html-attribute-good"></span>

**Correto**

A âncora usa help, um valor de rel apropriado para um recurso de ajuda vinculado.

```vue annotate="add:2"
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) · [Todas as regras](all.md)

### `vue/no-lone-template`

Proibir elementos `<template>` desnecessários

[Incorreto](#vue-no-lone-template-bad) · [Correto](#vue-no-lone-template-good)

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
        "vue/no-lone-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-lone-template-bad"></span>

**Incorreto**

O template interno não tem uma diretiva nem uma função de slot que lhe dê uma finalidade estrutural.

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

<span id="vue-no-lone-template-good"></span>

**Correto**

Remover o invólucro desnecessário deixa o parágrafo diretamente dentro da div.

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) · [Todas as regras](all.md)

### `vue/no-multi-spaces`

Proibir vários espaços consecutivos

[Incorreto](#vue-no-multi-spaces-bad) · [Correto](#vue-no-multi-spaces-good)

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
        "vue/no-multi-spaces": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multi-spaces-bad"></span>

**Incorreto**

Dois espaços separam os atributos ou o nome do elemento e o primeiro atributo.

```vue annotate="remove:2,3"
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

<span id="vue-no-multi-spaces-good"></span>

**Correto**

Espaços únicos separam os mesmos atributos.

```vue annotate="add:2,3"
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) · [Todas as regras](all.md)

### `vue/no-multiple-objects-in-class`

Proibir vários objetos literais dentro de uma vinculação de array em :class

[Incorreto](#vue-no-multiple-objects-in-class-bad) · [Correto](#vue-no-multiple-objects-in-class-good)

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
        "vue/no-multiple-objects-in-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multiple-objects-in-class-bad"></span>

**Incorreto**

Um array de classes contém dois objetos literais no nível superior que podem ser combinados.

```vue annotate="remove:2,3"
<template>
<div :class="[{ a }, { b }]"></div>
<div :class="[{ active: isActive }, { error: hasError }]"></div>
</template>
```

<span id="vue-no-multiple-objects-in-class-good"></span>

**Correto**

Um único objeto contém as condições das classes; arrays com um objeto e uma string ou com entradas não literais continuam permitidos.

```vue annotate="add:2,3,4"
<template>
<div :class="{ a, b }"></div>
<div :class="[{ active: isActive }, 'static']"></div>
<div :class="[foo, bar]"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) · [Todas as regras](all.md)

### `vue/no-multiple-template-root`

Proibir vários nós raiz em um template

[Incorreto](#vue-no-multiple-template-root-bad) · [Correto](#vue-no-multiple-template-root-good)

Severidade padrão: `error`  
Predefinições: _none_  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Ative apenas quando houver um contrato de raiz única. O Vue 3 normalmente suporta fragmentos.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-multiple-template-root": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multiple-template-root-bad"></span>

**Incorreto**

A convenção opcional de raiz única encontra dois parágrafos irmãos na raiz do template.

```vue annotate="remove:2,3"
<template>
<p>First</p>
<p>Second</p>
</template>
```

<span id="vue-no-multiple-template-root-good"></span>

**Correto**

Uma section envolve os parágrafos em uma única raiz; ative essa convenção apenas quando houver um contrato de raiz única.

```vue annotate="add:2"
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) · [Todas as regras](all.md)

### `vue/no-mutating-props`

Proibir a mutação de props de componentes

[Incorreto](#vue-no-mutating-props-bad) · [Correto](#vue-no-mutating-props-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-mutating-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-mutating-props-bad"></span>

**Incorreto**

Incrementar props.count escreve diretamente em um valor fornecido pelo componente pai.

```vue annotate="remove:4"
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

<span id="vue-no-mutating-props-good"></span>

**Correto**

O componente emite update:count com o próximo valor, deixando o pai responsável por atualizar a prop.

```vue annotate="add:3,5,6,7"
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L42) · [Todas as regras](all.md)

### `vue/no-negated-v-if-condition`

Proibir uma condição negada em v-if quando a cadeia tiver v-else

[Incorreto](#vue-no-negated-v-if-condition-bad) · [Correto](#vue-no-negated-v-if-condition-good)

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
        "vue/no-negated-v-if-condition": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-negated-v-if-condition-bad"></span>

**Incorreto**

Os ramos emparelhados v-if e v-else começam com uma condição negada.

```vue
<template>
<div v-if="!ok">A</div>
<div v-else>B</div>
</template>
```

<span id="vue-no-negated-v-if-condition-good"></span>

**Correto**

Uma condição positiva ok vem primeiro; ao inverter uma condição, coloque primeiro o ramo originalmente oposto. Um v-if negado isolado e comparações !== continuam permitidos.

```vue annotate="add:2,3,4,6,7"
<template>
<div v-if="ok">B</div>
<div v-else>A</div>

<div v-if="!ok">A</div>

<div v-if="a !== b">A</div>
<div v-else>B</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L37) · [Todas as regras](all.md)

### `vue/no-non-component-keep-alive-child`

Proibir invólucros de elementos comuns diretamente abaixo de `<KeepAlive>`

[Incorreto](#vue-no-non-component-keep-alive-child-bad) · [Correto](#vue-no-non-component-keep-alive-child-good)

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
        "vue/no-non-component-keep-alive-child": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-non-component-keep-alive-child-bad"></span>

**Incorreto**

KeepAlive envolve condicionalmente uma div nativa, em vez de armazenar UserCard diretamente em cache.

```vue annotate="remove:3"
<template>
  <KeepAlive>
    <div v-if="ready">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

<span id="vue-no-non-component-keep-alive-child-good"></span>

**Correto**

O primeiro exemplo torna UserCard o filho condicional. O invólucro com v-show ilustra uma estrutura fora desta verificação de filhos condicionais, sem prometer que o invólucro nativo seja armazenado em cache.

```vue annotate="add:3,4,5,6"
<template>
  <KeepAlive>
    <UserCard v-if="ready" />
  </KeepAlive>
  <KeepAlive>
    <div v-show="opened">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L14) · [Todas as regras](all.md)

### `vue/no-preprocessor-lang`

Desencorajar o uso de preprocessadores CSS em favor de CSS moderno

[Incorreto](#vue-no-preprocessor-lang-bad) · [Correto](#vue-no-preprocessor-lang-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Não implementada no lint de SFC  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Suporte atual: `no-sfc-finding`

Esta entrada do catálogo atualmente não emite uma ocorrência específica da regra pelo lint de SFCs. O par Bad/Good descreve a convenção pretendida, e não uma ocorrência executável. Ativar o ID não fornece a verificação de SFC ausente.

**ID configurado (sem diagnóstico de SFC atualmente)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-preprocessor-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-preprocessor-lang-bad"></span>

**Incorreto**

O bloco style seleciona SCSS com lang. Isso descreve a convenção pretendida de não usar preprocessadores; o processamento atual de SFCs não emite esta regra.

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

<span id="vue-no-preprocessor-lang-good"></span>

**Correto**

As mesmas declarações CSS omitem o lang do preprocessador. Essa é a correção da convenção, e não uma diferença executável de diagnósticos entre Bad/Good hoje.

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

O exemplo correto ilustra a convenção pretendida; o fluxo atual de SFC não emite o diagnóstico específico da regra para nenhum dos exemplos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) · [Todas as regras](all.md)

### `vue/no-reserved-component-names`

Proibir o uso de nomes reservados como nomes de componentes

[Incorreto](#vue-no-reserved-component-names-bad) · [Correto](#vue-no-reserved-component-names-good)

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
        "vue/no-reserved-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-reserved-component-names-bad"></span>

**Incorreto**

O nome de componente button entra em conflito com o nome de um elemento HTML nativo.

```vue annotate="remove:1,2,3,4"
<script>
export default {
  name: "button",
};
</script>
```

<span id="vue-no-reserved-component-names-good"></span>

**Correto**

AppButton é um nome de componente da aplicação e não reutiliza o nome nativo button.

```vue annotate="add:1,2,4,5,6,7,8,9"
<script setup lang="ts">
defineOptions({ name: "AppButton" });
</script>

<template>
  <Transition>
    <AppButton />
  </Transition>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) · [Todas as regras](all.md)

### `vue/no-root-v-if`

Proibir v-if no único elemento raiz de um template

[Incorreto](#vue-no-root-v-if-bad) · [Correto](#vue-no-root-v-if-good)

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
        "vue/no-root-v-if": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-root-v-if-bad"></span>

**Incorreto**

A própria raiz do componente aparece e desaparece sob v-if.

```vue annotate="remove:2"
<template>
<div v-if="show">content</div>
</template>
```

<span id="vue-no-root-v-if-good"></span>

**Correto**

Uma div externa estável permanece como raiz, enquanto o parágrafo aninhado recebe a condição de visibilidade.

```vue annotate="add:2,3,4"
<template>
<div>
<p v-if="show">content</p>
</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) · [Todas as regras](all.md)

### `vue/no-script-non-standard-lang`

Desencorajar valores não padronizados de lang em scripts

[Incorreto](#vue-no-script-non-standard-lang-bad) · [Correto](#vue-no-script-non-standard-lang-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Não implementada no lint de SFC  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Suporte atual: `no-sfc-finding`

Esta entrada do catálogo atualmente não emite uma ocorrência específica da regra pelo lint de SFCs. O par Bad/Good descreve a convenção pretendida, e não uma ocorrência executável. Ativar o ID não fornece a verificação de SFC ausente.

**ID configurado (sem diagnóstico de SFC atualmente)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-script-non-standard-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-script-non-standard-lang-bad"></span>

**Incorreto**

O script usa sintaxe CoffeeScript com lang=coffee. O processamento atual de SFCs não emite esta regra do catálogo para essa linguagem.

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

<span id="vue-no-script-non-standard-lang-good"></span>

**Correto**

O script usa uma declaração TypeScript comum com lang=ts, ilustrando a convenção de linguagem pretendida.

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

O exemplo correto ilustra a convenção pretendida; o fluxo atual de SFC não emite o diagnóstico específico da regra para nenhum dos exemplos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [Todas as regras](all.md)

### `vue/no-src-attribute`

Desencorajar o atributo src em blocos de SFCs

[Incorreto](#vue-no-src-attribute-bad) · [Correto](#vue-no-src-attribute-good)

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
        "vue/no-src-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-src-attribute-bad"></span>

**Incorreto**

Os blocos do SFC delegam o conteúdo de template, script e style a arquivos src.

```vue annotate="remove:1,2,3"
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

<span id="vue-no-src-attribute-good"></span>

**Correto**

Cada bloco do SFC contém seu próprio conteúdo, sem um atributo src externo.

```vue annotate="add:1,2,3,4,5,6,7,8,9,10,11,12,13"
<template>
  <p>Hello</p>
</template>

<script setup lang="ts">
const label = "Hello";
</script>

<style scoped>
p {
  color: red;
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16) · [Todas as regras](all.md)

### `vue/no-static-inline-styles`

Proibir atributos estáticos de estilo inline

[Incorreto](#vue-no-static-inline-styles-bad) · [Correto](#vue-no-static-inline-styles-good)

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
        "vue/no-static-inline-styles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-static-inline-styles-bad"></span>

**Incorreto**

O parágrafo contém a declaração de cor constante no atributo style.

```vue annotate="remove:1,2,3"
<template>
<p style="color: red">Notice</p>
</template>
```

<span id="vue-no-static-inline-styles-good"></span>

**Correto**

Uma classe notice e uma folha de estilo com escopo mantêm a cor constante fora do atributo no template.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) · [Todas as regras](all.md)

### `vue/no-template-key`

Proibir o atributo `key` em `<template>`

[Incorreto](#vue-no-template-key-bad) · [Correto](#vue-no-template-key-good)

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
        "vue/no-template-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-key-bad"></span>

**Incorreto**

Um invólucro template sem laço tem uma key, embora não seja o limite da iteração com chave.

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

<span id="vue-no-template-key-good"></span>

**Correto**

A key pertence a uma iteração template v-for, na qual identifica cada fragmento repetido.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) · [Todas as regras](all.md)

### `vue/no-template-lang`

Desencorajar o atributo lang no bloco template

[Incorreto](#vue-no-template-lang-bad) · [Correto](#vue-no-template-lang-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Não implementada no lint de SFC  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Suporte atual: `no-sfc-finding`

Esta entrada do catálogo atualmente não emite uma ocorrência específica da regra pelo lint de SFCs. O par Bad/Good descreve a convenção pretendida, e não uma ocorrência executável. Ativar o ID não fornece a verificação de SFC ausente.

**ID configurado (sem diagnóstico de SFC atualmente)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-lang-bad"></span>

**Incorreto**

O template seleciona Pug por meio de lang. Essa é uma convenção pretendida de usar apenas HTML; o processamento atual de SFCs não gera diagnósticos para este ID do catálogo.

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

<span id="vue-no-template-lang-good"></span>

**Correto**

Um template HTML comum omite lang e usa o parágrafo diretamente. Isso ilustra a convenção sem afirmar que há uma ocorrência diagnosticada atualmente em SFCs.

```vue annotate="add:1,2"
<template>
<p>Notice</p>
</template>
```

O exemplo correto ilustra a convenção pretendida; o fluxo atual de SFC não emite o diagnóstico específico da regra para nenhum dos exemplos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [Todas as regras](all.md)

### `vue/no-template-shadow`

Proibir nomes de variáveis que ocultam variáveis de um escopo externo

[Incorreto](#vue-no-template-shadow-bad) · [Correto](#vue-no-template-shadow-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

A verificação atual compara variáveis de v-for aninhados. Ela não reporta uma variável de um único v-for apenas porque compartilha o nome de uma variável do script.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-shadow": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-shadow-bad"></span>

**Incorreto**

O v-for interno declara item novamente e oculta a variável item externa dentro do laço aninhado.

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

<span id="vue-no-template-shadow-good"></span>

**Correto**

O laço interno declara child, deixando item disponível para a linha externa e child para a linha aninhada.

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [Todas as regras](all.md)

### `vue/no-template-target-blank`

Proibir target="_blank" sem rel="noopener noreferrer"

[Incorreto](#vue-no-template-target-blank-bad) · [Correto](#vue-no-template-target-blank-good)

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
        "vue/no-template-target-blank": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-target-blank-bad"></span>

**Incorreto**

O link externo abre um novo contexto de navegação sem a proteção rel esperada.

```vue annotate="remove:2"
<template>
<a href="https://example.com" target="_blank">x</a>
</template>
```

<span id="vue-no-template-target-blank-good"></span>

**Correto**

O mesmo link inclui noopener noreferrer junto de target=_blank.

```vue annotate="add:2"
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) · [Todas as regras](all.md)

### `vue/no-textarea-mustache`

Proibir interpolação com chaves duplas em `<textarea>`

[Incorreto](#vue-no-textarea-mustache-bad) · [Correto](#vue-no-textarea-mustache-good)

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
        "vue/no-textarea-mustache": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-textarea-mustache-bad"></span>

**Incorreto**

O textarea coloca message em uma interpolação filha, em vez de vincular seu valor.

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

<span id="vue-no-textarea-mustache-good"></span>

**Correto**

v-model vincula o valor editável do textarea a message.

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [Todas as regras](all.md)

### `vue/no-undefined-refs`

Proibir referências a variáveis não definidas nos templates

[Incorreto](#vue-no-undefined-refs-bad) · [Correto](#vue-no-undefined-refs-good)

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
        "vue/no-undefined-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-undefined-refs-bad"></span>

**Incorreto**

O template lê missing, embora o script declare apenas message.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

<span id="vue-no-undefined-refs-good"></span>

**Correto**

A interpolação lê a variável message existente.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) · [Todas as regras](all.md)

### `vue/no-unsafe-url`

Alertar sobre vinculações de URL potencialmente inseguras

[Incorreto](#vue-no-unsafe-url-bad) · [Correto](#vue-no-unsafe-url-good)

Severidade padrão: `warning`  
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
        "vue/no-unsafe-url": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unsafe-url-bad"></span>

**Incorreto**

O destino da âncora começa com o esquema executável javascript:.

```vue annotate="remove:2"
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

<span id="vue-no-unsafe-url-good"></span>

**Correto**

A âncora usa o destino local comum de navegação /next.

```vue annotate="add:2"
<template>
<a href="/next">Continue</a>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [Todas as regras](all.md)

### `vue/no-unsandboxed-iframe`

Exigir um atributo sandbox nos elementos iframe

[Incorreto](#vue-no-unsandboxed-iframe-bad) · [Correto](#vue-no-unsandboxed-iframe-good)

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
        "vue/no-unsandboxed-iframe": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unsandboxed-iframe-bad"></span>

**Incorreto**

O frame incorporado não tem um atributo sandbox que limite suas capacidades.

```vue annotate="remove:2"
<template>
<iframe src="/embed"></iframe>
</template>
```

<span id="vue-no-unsandboxed-iframe-good"></span>

**Correto**

sandbox aplica restrições; allow-scripts habilita explicitamente essa única capacidade quando necessário.

```vue annotate="add:2,3"
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) · [Todas as regras](all.md)

### `vue/no-unused-components`

Proibir o registro de componentes não usados nos templates

[Incorreto](#vue-no-unused-components-bad) · [Correto](#vue-no-unused-components-good)

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
        "vue/no-unused-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-components-bad"></span>

**Incorreto**

UserAvatar é importado como componente, mas o template nunca o renderiza.

```vue annotate="remove:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <p>{{ user.name }}</p>
</template>
```

<span id="vue-no-unused-components-good"></span>

**Correto**

O template renderiza o UserAvatar importado e passa a variável user.

```vue annotate="add:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L46) · [Todas as regras](all.md)

### `vue/no-unused-properties`

Proibir propriedades não usadas definidas em defineProps

[Incorreto](#vue-no-unused-properties-bad) · [Correto](#vue-no-unused-properties-good)

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
        "vue/no-unused-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-properties-bad"></span>

**Incorreto**

O componente declara description como prop, mas renderiza apenas title.

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
```

<span id="vue-no-unused-properties-good"></span>

**Correto**

As duas props declaradas são referenciadas pelo template.

```vue annotate="add:7"
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ description }}</p>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) · [Todas as regras](all.md)

### `vue/no-unused-refs`

Reportar refs de template (ref="x") nunca referenciadas em &lt;script&gt;

[Incorreto](#vue-no-unused-refs-bad) · [Correto](#vue-no-unused-refs-good)

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
        "vue/no-unused-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-refs-bad"></span>

**Incorreto**

O template declara o nome de ref unused sem uma variável de referência correspondente no script.

```vue annotate="remove:1,3"
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

<span id="vue-no-unused-refs-good"></span>

**Correto**

A ref de template inputEl tem uma variável ref de mesmo nome em script setup.

```vue annotate="add:1,3,4"
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) · [Todas as regras](all.md)

### `vue/no-unused-setup-bindings`

Proibir variáveis de script setup que nunca são lidas

[Incorreto](#vue-no-unused-setup-bindings-bad) · [Correto](#vue-no-unused-setup-bindings-good)

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
        "vue/no-unused-setup-bindings": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-setup-bindings-bad"></span>

**Incorreto**

A variável message de script setup nunca é lida pelo template.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

<span id="vue-no-unused-setup-bindings-good"></span>

**Correto**

O parágrafo interpola message, usando a variável declarada.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) · [Todas as regras](all.md)

### `vue/no-unused-vars`

Proibir definições de variáveis não usadas nas diretivas v-for e v-slot

[Incorreto](#vue-no-unused-vars-bad) · [Correto](#vue-no-unused-vars-good)

Severidade padrão: `warning`  
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
        "vue/no-unused-vars": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-vars-bad"></span>

**Incorreto**

O laço declara um index não usado e o slot declara foo sem referenciá-lo.

```vue annotate="remove:2,3,4"
<template>
  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ foo }">
    <span>Hello</span>
  </template>
</template>
```

<span id="vue-no-unused-vars-good"></span>

**Correto**

Os exemplos usam index ou o marcam como intencionalmente não usado por meio de _index, e o slot renderiza data. Chaves de índice são apenas um exemplo de uso aqui, e não uma recomendação para manter a identidade estável dos itens.

```vue annotate="add:2,3,4,5"
<template>
  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
  <li v-for="(item, _index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ data }">
    <span>{{ data }}</span>
  </template>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) · [Todas as regras](all.md)

### `vue/no-use-v-else-with-v-for`

Proibir `v-else-if` ou `v-else` no mesmo elemento que `v-for`

[Incorreto](#vue-no-use-v-else-with-v-for-bad) · [Correto](#vue-no-use-v-else-with-v-for-good)

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
        "vue/no-use-v-else-with-v-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-use-v-else-with-v-for-bad"></span>

**Incorreto**

O ramo else e a iteração v-for estão associados ao mesmo parágrafo.

```vue annotate="remove:3"
<template>
<p v-if="ready">Ready</p>
<p v-else v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

<span id="vue-no-use-v-else-with-v-for-good"></span>

**Correto**

Um template separado contém v-else, e seu parágrafo filho contém v-for.

```vue annotate="add:3"
<template>
<p v-if="ready">Ready</p>
<template v-else><p v-for="item in items" :key="item.id">{{ item.name }}</p></template>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) · [Todas as regras](all.md)

### `vue/no-use-v-if-with-v-for`

Proibir `v-if` no mesmo elemento que `v-for`

[Incorreto](#vue-no-use-v-if-with-v-for-bad) · [Correto](#vue-no-use-v-if-with-v-for-good)

Severidade padrão: `warning`  
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
        "vue/no-use-v-if-with-v-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-use-v-if-with-v-for-bad"></span>

**Incorreto**

O mesmo elemento de lista combina v-if e v-for e testa a visibilidade por meio da variável do laço.

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

<span id="vue-no-use-v-if-with-v-for-good"></span>

**Correto**

Uma coleção computada filtra os itens visíveis antes que o template itere sobre eles.

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const visibleItems = computed(() => items.filter((item) => item.visible));
</script>

<template>
  <li v-for="item in visibleItems" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) · [Todas as regras](all.md)

### `vue/no-useless-mustaches`

Proibir interpolação com chaves duplas cuja expressão seja uma string literal constante

[Incorreto](#vue-no-useless-mustaches-bad) · [Correto](#vue-no-useless-mustaches-good)

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
        "vue/no-useless-mustaches": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-mustaches-bad"></span>

**Incorreto**

A interpolação contém apenas uma string constante e não precisa avaliar uma expressão.

```vue annotate="remove:2,3,4"
<template>
<div>{{ 'x' }}</div>
<div>{{ "x" }}</div>
<div>{{ `x` }}</div>
</template>
```

<span id="vue-no-useless-mustaches-good"></span>

**Correto**

O texto literal é escrito diretamente; expressões com variáveis, strings de template interpoladas e espaços separadores intencionais continuam sendo casos de interpolação.

```vue annotate="add:2,3,4,5"
<template>
<div>x</div>
<div>{{ x }}</div>
<div>{{ `pre-${x}` }}</div>
<span>A</span> {{ " " }} <span>B</span>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) · [Todas as regras](all.md)

### `vue/no-useless-template-attributes`

Proibir atributos sem efeito em elementos `<template>`

[Incorreto](#vue-no-useless-template-attributes-bad) · [Correto](#vue-no-useless-template-attributes-good)

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
        "vue/no-useless-template-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-template-attributes-bad"></span>

**Incorreto**

O template condicional tem uma class, mas esse invólucro estrutural não renderiza um elemento DOM para recebê-la.

```vue annotate="remove:2"
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

<span id="vue-no-useless-template-attributes-good"></span>

**Correto**

A class passa para o parágrafo que é realmente renderizado, enquanto v-if permanece no template estrutural.

```vue annotate="add:2"
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [Todas as regras](all.md)

### `vue/no-useless-v-bind`

Proibir um v-bind cujo valor seja uma string literal simples

[Incorreto](#vue-no-useless-v-bind-bad) · [Correto](#vue-no-useless-v-bind-good)

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
        "vue/no-useless-v-bind": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-v-bind-bad"></span>

**Incorreto**

A vinculação foo avalia uma string constante entre aspas ou uma string de template sem interpolação.

```vue annotate="remove:2,3"
<template>
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

<span id="vue-no-useless-v-bind-good"></span>

**Correto**

O valor constante vira um atributo estático; valores com variáveis e interpolações mantêm sua vinculação.

```vue annotate="add:2,3,4"
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) · [Todas as regras](all.md)

### `vue/no-v-for-template-key-on-child`

Proibir `key` no filho de um `<template v-for>`

[Incorreto](#vue-no-v-for-template-key-on-child-bad) · [Correto](#vue-no-v-for-template-key-on-child-good)

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
        "vue/no-v-for-template-key-on-child": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-for-template-key-on-child-bad"></span>

**Incorreto**

O parágrafo filho tem a key, enquanto a própria iteração template não tem chave.

```vue annotate="remove:2"
<template>
<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>
</template>
```

<span id="vue-no-v-for-template-key-on-child-good"></span>

**Correto**

A key passa para template v-for, identificando o fragmento repetido completo.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) · [Todas as regras](all.md)

### `vue/no-v-html`

Alertar sobre v-html para prevenir vulnerabilidades XSS

[Incorreto](#vue-no-v-html-bad) · [Correto](#vue-no-v-html-good)

Severidade padrão: `warning`  
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
        "vue/no-v-html": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-html-bad"></span>

**Incorreto**

v-html interpreta content como HTML, em vez de texto comum.

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

<span id="vue-no-v-html-good"></span>

**Correto**

A interpolação com chaves duplas exibe content como texto escapado, em vez de injetar HTML.

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) · [Todas as regras](all.md)

### `vue/no-v-text`

Proibir a diretiva v-text; preferir interpolação com chaves duplas

[Incorreto](#vue-no-v-text-bad) · [Correto](#vue-no-v-text-good)

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
        "vue/no-v-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-text-bad"></span>

**Incorreto**

O conteúdo da div é fornecido pela diretiva v-text.

```vue annotate="remove:2"
<template>
<div v-text="message"></div>
</template>
```

<span id="vue-no-v-text-good"></span>

**Correto**

A interpolação com chaves duplas expressa a mesma vinculação de texto diretamente no conteúdo do elemento.

```vue annotate="add:2"
<template>
<div>{{ message }}</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) · [Todas as regras](all.md)

### `vue/no-v-text-v-html-on-component`

Proibir v-text / v-html em elementos de componente

[Incorreto](#vue-no-v-text-v-html-on-component-bad) · [Correto](#vue-no-v-text-v-html-on-component-good)

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
        "vue/no-v-text-v-html-on-component": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-text-v-html-on-component-bad"></span>

**Incorreto**

A tag do componente recebe v-html ou v-text, que substitui o conteúdo do elemento em vez de fornecer slots ao componente.

```vue annotate="remove:2,3"
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

<span id="vue-no-v-text-v-html-on-component-good"></span>

**Correto**

Elementos HTML nativos podem receber as diretivas; MyComponent recebe seu conteúdo pelo slot padrão.

```vue annotate="add:2,3,4"
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) · [Todas as regras](all.md)

### `vue/permitted-contents`

Aplicar as regras do modelo de conteúdo HTML

[Incorreto](#vue-permitted-contents-bad) · [Correto](#vue-permitted-contents-good)

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
        "vue/permitted-contents": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-permitted-contents-bad"></span>

**Incorreto**

Os exemplos colocam conteúdo em bloco dentro de p, omitem o corpo da tabela, aninham controles interativos ou colocam uma div diretamente dentro de ul.

```vue annotate="remove:2,3,4,5"
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

<span id="vue-permitted-contents-good"></span>

**Correto**

Os exemplos usam conteúdo inline no parágrafo, um tbody explícito e filhos li. O componente personalizado MyItem não é tratado como um filho nativo conhecido de ul.

```vue annotate="add:2,3,4"
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) · [Todas as regras](all.md)

### `vue/prefer-props-shorthand`

Recomendar sintaxe abreviada para props (Vue 3.4+)

[Incorreto](#vue-prefer-props-shorthand-bad) · [Correto](#vue-prefer-props-shorthand-good)

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
        "vue/prefer-props-shorthand": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prefer-props-shorthand-bad"></span>

**Incorreto**

Cada vinculação repete o nome da variável correspondente, inclusive o equivalente camelCase de um argumento separado por hífen.

```vue annotate="remove:2,3,4,5"
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
  <span :style="style" />
  <div :aria-label="ariaLabel" />
</template>
```

<span id="vue-prefer-props-shorthand-good"></span>

**Correto**

A forma abreviada de vinculação de mesmo nome do Vue 3.4+ remove as expressões repetidas; uma variável de origem diferente, como bar, permanece explícita.

```vue annotate="add:2,3,4,5,6"
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <span :style />
  <div :aria-label />
  <MyComponent :foo="bar" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) · [Todas as regras](all.md)

### `vue/prefer-true-attribute-shorthand`

Preferir a forma abreviada para um atributo booleano vinculado a `true`

[Incorreto](#vue-prefer-true-attribute-shorthand-bad) · [Correto](#vue-prefer-true-attribute-shorthand-good)

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
        "vue/prefer-true-attribute-shorthand": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prefer-true-attribute-shorthand-bad"></span>

**Incorreto**

Um atributo booleano nativo disabled vincula o valor constante true.

```vue annotate="remove:2"
<template>
<input :disabled="true" />
</template>
```

<span id="vue-prefer-true-attribute-shorthand-good"></span>

**Correto**

O atributo nativo usa sua forma booleana abreviada. Vinculações false e props de componentes mantêm seus valores explícitos.

```vue annotate="add:2,3,4,5"
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) · [Todas as regras](all.md)

### `vue/prop-name-casing`

Aplicar um padrão de maiúsculas e minúsculas aos nomes de props declaradas

[Incorreto](#vue-prop-name-casing-bad) · [Correto](#vue-prop-name-casing-good)

Severidade padrão: `warning`  
Predefinições: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Verifica os nomes de props declaradas, e não o padrão de maiúsculas e minúsculas dos atributos passados a um filho.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/prop-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prop-name-casing-bad"></span>

**Incorreto**

O nome de prop declarado user_name usa uma grafia separada por sublinhado.

```vue annotate="remove:2,4"
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

<span id="vue-prop-name-casing-good"></span>

**Correto**

A declaração e sua referência no template usam o nome camelCase userName.

```vue annotate="add:2,4"
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) · [Todas as regras](all.md)

### `vue/require-component-is`

Exigir `v-bind:is` em elementos `<component>`

[Incorreto](#vue-require-component-is-bad) · [Correto](#vue-require-component-is-good)

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
        "vue/require-component-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-component-is-bad"></span>

**Incorreto**

O `<component>` dinâmico não tem um destino `is`, então o Vue não pode escolher um componente para renderizar.

```vue annotate="remove:2"
<template>
  <component />
</template>
```

<span id="vue-require-component-is-good"></span>

**Correto**

`:is="currentComponent"` fornece a seleção do componente; a vinculação pode mudar durante a execução.

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) · [Todas as regras](all.md)

### `vue/require-component-registration`

Exigir importação ou registro explícito de componentes

[Incorreto](#vue-require-component-registration-bad) · [Correto](#vue-require-component-registration-good)

Severidade padrão: `warning`  
Predefinições: `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Consulte as [opções tipadas e os valores padrão](/rules/options.md).

Liste nomes explícitos de componentes fornecidos por plugins da aplicação ou pelo previewSetup do Musea. Grafias PascalCase e kebab-case são aceitas; expressões regulares não são interpretadas. As opções não ativam a regra. Camadas posteriores substituem a lista; uma lista vazia remove os nomes herdados.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-component-registration": "warn"
      },
      "ruleOptions": {
        "vue/require-component-registration": {
          "globals": [
            "MyButton",
            "MyIcon"
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

<span id="vue-require-component-registration-bad"></span>

**Incorreto**

`MissingWidget` não está registrado nem incluído na lista configurada de componentes globais permitidos.

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

<span id="vue-require-component-registration-good"></span>

**Correto**

`MyButton` consta na opção `globals` do exemplo. Essa opção isenta um componente global conhecido; ela não o registra nem o importa.

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [Todas as regras](all.md)

### `vue/require-scoped-style`

Exigir o atributo scoped nas tags style

[Incorreto](#vue-require-scoped-style-bad) · [Correto](#vue-require-scoped-style-good)

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
        "vue/require-scoped-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-scoped-style-bad"></span>

**Incorreto**

O estilo `.button` não tem escopo e pode afetar elementos correspondentes fora deste componente.

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

<span id="vue-require-scoped-style-good"></span>

**Correto**

Adicionar `scoped` aplica o escopo de componente do Vue ao mesmo seletor e às mesmas declarações.

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [Todas as regras](all.md)

### `vue/require-toggle-inside-transition`

Exigir uma alternância no elemento envolvido por `<transition>`

[Incorreto](#vue-require-toggle-inside-transition-bad) · [Correto](#vue-require-toggle-inside-transition-good)

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
        "vue/require-toggle-inside-transition": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-toggle-inside-transition-bad"></span>

**Incorreto**

O filho estático dentro de `<Transition>` não tem visibilidade condicional nem seleção dinâmica para provocar uma mudança de entrada ou saída.

```vue annotate="remove:3"
<template>
<transition>
<div>content</div>
</transition>
</template>
```

<span id="vue-require-toggle-inside-transition-good"></span>

**Correto**

`v-if="show"` altera a existência do filho, fornecendo um limite de entrada ou saída à transição.

```vue annotate="add:3"
<template>
<transition>
<div v-if="show">content</div>
</transition>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L48) · [Todas as regras](all.md)

### `vue/require-v-for-key`

Exigir `v-bind:key` nas diretivas `v-for`

[Incorreto](#vue-require-v-for-key-bad) · [Correto](#vue-require-v-for-key-good)

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
        "vue/require-v-for-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-v-for-key-bad"></span>

**Incorreto**

Cada `<li>` repetido não tem uma chave que identifique seu item correspondente durante atualizações da lista.

```vue annotate="remove:2"
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

<span id="vue-require-v-for-key-good"></span>

**Correto**

`:key="item.id"` dá a cada nó repetido a identidade do item, em vez de sua posição atual.

```vue annotate="add:2"
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L35) · [Todas as regras](all.md)

### `vue/scoped-event-names`

Recomendar nomes de eventos com escopo no formato context:event

[Incorreto](#vue-scoped-event-names-bad) · [Correto](#vue-scoped-event-names-good)

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
        "vue/scoped-event-names": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-scoped-event-names-bad"></span>

**Incorreto**

`playAudio`, `pauseAudio` e `reloadAudio` codificam seu escopo como sufixos camelCase, em vez de seguir a convenção de eventos separados por dois-pontos da regra.

```vue annotate="remove:3,4,5"
<template>
  <AudioPlayer
    @playAudio="play"
    @pauseAudio="pause"
    @reloadAudio="reload"
  />
</template>
```

<span id="vue-scoped-event-names-good"></span>

**Correto**

`audio:play`, `audio:pause` e `audio:reload` compartilham um escopo explícito `audio:`. O componente emissor deve usar os mesmos nomes.

```vue annotate="add:3,4,5"
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) · [Todas as regras](all.md)

### `vue/sfc-element-order`

Aplicar uma ordem consistente aos elementos de nível superior dos SFCs

[Incorreto](#vue-sfc-element-order-bad) · [Correto](#vue-sfc-element-order-good)

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
        "vue/sfc-element-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-sfc-element-order-bad"></span>

**Incorreto**

O bloco style precede o bloco script, contrariando a ordem configurada dos blocos do SFC.

```vue annotate="remove:2,6,7,8"
<style scoped>
.panel {
  color: red;
}
</style>
<script setup lang="ts">
const label = "Save";
</script>
```

<span id="vue-sfc-element-order-good"></span>

**Correto**

Os blocos seguem script → template → style. Projetos podem escolher outra ordem pela opção tipada desta regra.

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts">
const label = "Save";
</script>

<template>
  <p>{{ label }}</p>
</template>

<style scoped>
p {
  color: red;
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) · [Todas as regras](all.md)

### `vue/single-style-block`

Recomendar um único bloco style

[Incorreto](#vue-single-style-block-bad) · [Correto](#vue-single-style-block-good)

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
        "vue/single-style-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-single-style-block-bad"></span>

**Incorreto**

O componente divide seus estilos com escopo de panel e title entre dois blocos style.

```vue annotate="remove:5,6,7"
<style scoped>
.panel {
  color: red;
}
</style>

<style scoped>
.title {
  color: blue;
}
</style>
```

<span id="vue-single-style-block-good"></span>

**Correto**

Os dois seletores mantêm o escopo em um único bloco style, atendendo à convenção de bloco único sem remover nenhum estilo.

```vue
<style scoped>
.panel {
  color: red;
}
.title {
  color: blue;
}
</style>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) · [Todas as regras](all.md)

### `vue/slot-name-casing`

Exigir kebab-case nos slots nomeados usados por v-slot

[Incorreto](#vue-slot-name-casing-bad) · [Correto](#vue-slot-name-casing-good)

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
        "vue/slot-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-slot-name-casing-bad"></span>

**Incorreto**

O slot nomeado `mySlot` usa camelCase onde a regra exige um nome separado por hífen.

```vue annotate="remove:2"
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

<span id="vue-slot-name-casing-good"></span>

**Correto**

`#my-slot` usa kebab-case. Renomeie o ponto de inserção do slot correspondente para o mesmo nome.

```vue annotate="add:2"
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [Todas as regras](all.md)

### `vue/this-in-template`

Proibir `this.` nas expressões do template

[Incorreto](#vue-this-in-template-bad) · [Correto](#vue-this-in-template-good)

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
        "vue/this-in-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-this-in-template-bad"></span>

**Incorreto**

As expressões do template acessam explicitamente `this.message`, `this.className` e `this.handleClick`, embora o Vue exponha essas variáveis diretamente.

```vue annotate="remove:2,3,4"
<template>
<div>{{ this.message }}</div>
<div :class="this.className"></div>
<button @click="this.handleClick()"></button>
</template>
```

<span id="vue-this-in-template-good"></span>

**Correto**

Use `message`, `className` e `handleClick` diretamente. A string literal `'this.is.a.string'` permanece igual porque não é um acesso a membro.

```vue annotate="add:2,3,4,5"
<template>
<div>{{ message }}</div>
<div :class="className"></div>
<button @click="handleClick()"></button>
<div>{{ 'this.is.a.string' }}</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L33) · [Todas as regras](all.md)

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

### `vue/use-v-on-exact`

Exigir o modificador `.exact` em `v-on` quando houver manipuladores baseados em modificadores

[Incorreto](#vue-use-v-on-exact-bad) · [Correto](#vue-use-v-on-exact-good)

Severidade padrão: `warning`  
Predefinições: `essential`, `nuxt`, `opinionated`  
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
        "vue/use-v-on-exact": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-v-on-exact-bad"></span>

**Incorreto**

O manipulador de clique comum também pode executar em Ctrl-clique, sobrepondo-se ao manipulador separado `.ctrl`.

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

<span id="vue-use-v-on-exact-good"></span>

**Correto**

`.exact` limita o manipulador de clique comum a cliques sem teclas modificadoras; o manipulador específico de Ctrl permanece separado.

```vue annotate="add:2,3,4,5,6"
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L28) · [Todas as regras](all.md)

### `vue/v-bind-style`

Aplicar um padrão de sintaxe à diretiva `v-bind`

[Incorreto](#vue-v-bind-style-bad) · [Correto](#vue-v-bind-style-good)

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
        "vue/v-bind-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-bind-style-bad"></span>

**Incorreto**

`v-bind:class` usa a forma longa onde o padrão de vinculação configurado exige a forma abreviada com dois-pontos.

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

<span id="vue-v-bind-style-good"></span>

**Correto**

`:class` mantém a mesma expressão com a forma abreviada exigida; esta regra trata da grafia, e não do tipo do valor.

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) · [Todas as regras](all.md)

### `vue/v-on-event-hyphenation`

Exigir hífens nos nomes de eventos personalizados em v-on de componentes

[Incorreto](#vue-v-on-event-hyphenation-bad) · [Correto](#vue-v-on-event-hyphenation-good)

Severidade padrão: `warning`  
Predefinições: `nuxt`, `opinionated`  
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
        "vue/v-on-event-hyphenation": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-event-hyphenation-bad"></span>

**Incorreto**

O listener do componente personalizado usa `@myEvent` em vez de um nome de evento separado por hífen.

```vue annotate="remove:2,3"
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
</template>
```

<span id="vue-v-on-event-hyphenation-good"></span>

**Correto**

`@my-event` usa a grafia exigida para eventos personalizados. Os listeners em elementos nativos e os argumentos de evento dinâmicos mostrados abaixo ficam fora desta verificação.

```vue annotate="add:2,3,4"
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L35) · [Todas as regras](all.md)

### `vue/v-on-handler-style`

Exigir manipuladores de v-on escritos como referência a método ou função inline

[Incorreto](#vue-v-on-handler-style-bad) · [Correto](#vue-v-on-handler-style-good)

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
        "vue/v-on-handler-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-handler-style-bad"></span>

**Incorreto**

Os manipuladores colocam mutações e várias instruções diretamente no atributo de evento.

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

<span id="vue-v-on-handler-style-good"></span>

**Correto**

Use uma referência a manipulador ou uma expressão de função, comum ou de seta, quando for necessária lógica inline. O limite da função torna explícita a forma do manipulador.

```vue annotate="add:2,3,4,5"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
<button @click="() => count++"></button>
<button @click="function () { count++ }"></button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33) · [Todas as regras](all.md)

### `vue/v-on-style`

Aplicar um padrão de sintaxe à diretiva `v-on`

[Incorreto](#vue-v-on-style-bad) · [Correto](#vue-v-on-style-good)

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
        "vue/v-on-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-style-bad"></span>

**Incorreto**

`v-on:click` usa a forma longa de listener de evento onde a regra exige a forma abreviada.

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

<span id="vue-v-on-style-good"></span>

**Correto**

`@click` mantém o mesmo manipulador e usa a forma abreviada configurada.

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) · [Todas as regras](all.md)

### `vue/v-slot-style`

Aplicar um padrão de sintaxe à diretiva `v-slot`

[Incorreto](#vue-v-slot-style-bad) · [Correto](#vue-v-slot-style-good)

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
        "vue/v-slot-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-slot-style-bad"></span>

**Incorreto**

O componente usa `#default` e o template usa `v-slot:header`, contrariando os padrões da regra para cada contexto.

```vue annotate="remove:2,4"
<template>
  <MyComponent #default="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template v-slot:header>Header</template>
  </MyComponent>
</template>
```

<span id="vue-v-slot-style-good"></span>

**Correto**

Use `v-slot` para o slot padrão do componente e `#header` para o slot nomeado do template.

```vue annotate="add:2,4"
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [Todas as regras](all.md)

### `vue/valid-attribute-name`

Exigir nomes de atributos válidos

[Incorreto](#vue-valid-attribute-name-bad) · [Correto](#vue-valid-attribute-name-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correção automática: Nenhuma; revise a alteração sugerida  
Aplicável a: Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra  
Opções: Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.

Diagnóstico do exemplo incorreto: `parser/template`

A grafia malformada de um atributo é diagnosticada por parser/template antes que esta regra defensiva veja um atributo. Portanto, Bad reporta parser/template; não promete uma ocorrência separada de vue/valid-attribute-name.

**Configuração (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-attribute-name": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-attribute-name-bad"></span>

**Incorreto**

A aspa dentro de `my"attr` torna o nome do atributo malformado. Este exemplo produz o diagnóstico `parser/template` do parser, sem prometer um diagnóstico separado da regra.

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

<span id="vue-valid-attribute-name-good"></span>

**Correto**

`my-attr` é um nome de atributo bem formado, então o parser do template pode ler o atributo e seu valor.

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [Todas as regras](all.md)

### `vue/valid-template-root`

Exigir uma raiz `<template>` válida para a semântica de fragmentos do Vue 3

[Incorreto](#vue-valid-template-root-bad) · [Correto](#vue-valid-template-root-good)

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
        "vue/valid-template-root": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-template-root-bad"></span>

**Incorreto**

Um `<template>` aninhado comum ocupa a raiz do template sem uma diretiva que lhe dê uma função de renderização.

```vue annotate="remove:2"
<template>
<template>content</template>
</template>
```

<span id="vue-valid-template-root-good"></span>

**Correto**

A `<div>` é um elemento raiz renderizável. Este exemplo não impõe uma restrição universal de raiz única aos fragmentos do Vue 3.

```vue annotate="add:2"
<template>
<div>content</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) · [Todas as regras](all.md)

### `vue/valid-v-bind`

Exigir diretivas `v-bind` válidas

[Incorreto](#vue-valid-v-bind-bad) · [Correto](#vue-valid-v-bind-good)

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
        "vue/valid-v-bind": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-bind-bad"></span>

**Incorreto**

O `v-bind` sem argumento não tem uma expressão de objeto, e a forma com argumento vazio não tem um nome de atributo.

```vue annotate="remove:2,3"
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

<span id="vue-valid-v-bind-good"></span>

**Correto**

Forneça um atributo e uma expressão, vincule um objeto ou use a forma abreviada de mesmo nome do Vue 3.4+, como `:loading`.

```vue annotate="add:2,3,4"
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) · [Todas as regras](all.md)

### `vue/valid-v-cloak`

Exigir diretivas `v-cloak` válidas

[Incorreto](#vue-valid-v-cloak-bad) · [Correto](#vue-valid-v-cloak-good)

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
        "vue/valid-v-cloak": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-cloak-bad"></span>

**Incorreto**

`v-cloak` recebe um valor, argumento ou modificador, embora não aceite nenhum deles.

```vue annotate="remove:2,3,4"
<template>
<div v-cloak="foo"></div>
<div v-cloak:arg></div>
<div v-cloak.mod></div>
</template>
```

<span id="vue-valid-v-cloak-good"></span>

**Correto**

Use `v-cloak` sozinho; CSS pode ocultar o elemento até que o Vue remova esse atributo após a montagem.

```vue annotate="add:2"
<template>
<div v-cloak></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) · [Todas as regras](all.md)

### `vue/valid-v-else`

Exigir diretivas `v-else` válidas

[Incorreto](#vue-valid-v-else-bad) · [Correto](#vue-valid-v-else-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/valid-v-else": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-else-bad"></span>

**Incorreto**

Os exemplos atribuem uma expressão a `v-else`, combinam-no com `v-if` ou omitem o ramo condicional adjacente que deve precedê-lo.

```vue annotate="remove:2,3"
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

<span id="vue-valid-v-else-good"></span>

**Correto**

Coloque `v-else` sozinho imediatamente após o ramo `v-if` correspondente.

```vue annotate="add:2"
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L32) · [Todas as regras](all.md)

### `vue/valid-v-for`

Exigir diretivas `v-for` válidas

[Incorreto](#vue-valid-v-for-bad) · [Correto](#vue-valid-v-for-good)

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
        "vue/valid-v-for": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-for-bad"></span>

**Incorreto**

Os laços omitem a expressão de iteração ou adicionam o modificador `.stop`, que não é aceito.

```vue annotate="remove:2,3,4"
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

<span id="vue-valid-v-for-good"></span>

**Correto**

Use `item in items` ou `(item, index) of items` com uma expressão de iteração completa e as chaves mostradas.

```vue annotate="add:2,3"
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L31) · [Todas as regras](all.md)

### `vue/valid-v-html`

Exigir diretivas `v-html` válidas

[Incorreto](#vue-valid-v-html-bad) · [Correto](#vue-valid-v-html-good)

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
        "vue/valid-v-html": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-html-bad"></span>

**Incorreto**

`v-html` não tem sua expressão ou usa um argumento ou modificador que esta diretiva não aceita.

```vue annotate="remove:2,3,4"
<template>
<div v-html></div>
<div v-html:arg="foo"></div>
<div v-html.mod="foo"></div>
</template>
```

<span id="vue-valid-v-html-good"></span>

**Correto**

`v-html="html"` fornece uma expressão válida. A validade sintática não sanitiza HTML nem torna seguro conteúdo não confiável.

```vue annotate="add:2"
<template>
<div v-html="html"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) · [Todas as regras](all.md)

### `vue/valid-v-if`

Exigir diretivas `v-if` válidas

[Incorreto](#vue-valid-v-if-bad) · [Correto](#vue-valid-v-if-good)

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
        "vue/valid-v-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-if-bad"></span>

**Incorreto**

As condições omitem uma expressão ou combinam `v-if` com uma diretiva else no mesmo nó.

```vue annotate="remove:2,3,4"
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

<span id="vue-valid-v-if-good"></span>

**Correto**

Cada `v-if` tem uma condição não vazia, como `ready` ou `count > 0`, sem uma diretiva else incompatível.

```vue annotate="add:2,3"
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) · [Todas as regras](all.md)

### `vue/valid-v-memo`

Exigir diretivas `v-memo` válidas

[Incorreto](#vue-valid-v-memo-bad) · [Correto](#vue-valid-v-memo-good)

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
        "vue/valid-v-memo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-memo-bad"></span>

**Incorreto**

`v-memo` sozinho não fornece ao Vue uma expressão de dependências para decidir quando reutilizar a subárvore.

```vue annotate="remove:2"
<template>
  <div v-memo></div>
</template>
```

<span id="vue-valid-v-memo-good"></span>

**Correto**

`v-memo="[valueA, valueB]"` fornece o array de dependências usado para memorização.

```vue annotate="add:2"
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) · [Todas as regras](all.md)

### `vue/valid-v-model`

Exigir diretivas `v-model` válidas

[Incorreto](#vue-valid-v-model-bad) · [Correto](#vue-valid-v-model-good)

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
        "vue/valid-v-model": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-model-bad"></span>

**Incorreto**

Uma `<div>` nativa não pode usar `v-model` como controle de formulário, e uma diretiva de input sem valor não tem uma expressão de destino gravável.

```vue annotate="remove:2,3"
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

<span id="vue-valid-v-model-good"></span>

**Correto**

Vincule o input, select, textarea ou componente personalizado às variáveis graváveis mostradas.

```vue annotate="add:2,3,4,5"
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) · [Todas as regras](all.md)

### `vue/valid-v-on`

Exigir diretivas `v-on` válidas

[Incorreto](#vue-valid-v-on-bad) · [Correto](#vue-valid-v-on-good)

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
        "vue/valid-v-on": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-on-bad"></span>

**Incorreto**

As formas de listener omitem um argumento de evento ou a expressão obrigatória de manipulador ou objeto.

```vue annotate="remove:2,3,4"
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
</template>
```

<span id="vue-valid-v-on-good"></span>

**Correto**

Use um evento com seu manipulador ou passe um objeto de listeners para `v-on` sem argumento.

```vue annotate="add:2,3"
<template>
  <div @click="handleClick"></div>
  <div v-on="{ click: handleClick }"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) · [Todas as regras](all.md)

### `vue/valid-v-once`

Exigir diretivas `v-once` válidas

[Incorreto](#vue-valid-v-once-bad) · [Correto](#vue-valid-v-once-good)

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
        "vue/valid-v-once": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-once-bad"></span>

**Incorreto**

`v-once` tem um valor, argumento ou modificador, embora esta diretiva seja um marcador sem valor para renderização única.

```vue annotate="remove:2,3,4"
<template>
<div v-once="foo"></div>
<div v-once:arg></div>
<div v-once.mod></div>
</template>
```

<span id="vue-valid-v-once-good"></span>

**Correto**

`v-once` sozinho marca a subárvore para renderização única, sem sintaxe não suportada.

```vue annotate="add:2"
<template>
<div v-once></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) · [Todas as regras](all.md)

### `vue/valid-v-show`

Exigir diretivas `v-show` válidas

[Incorreto](#vue-valid-v-show-bad) · [Correto](#vue-valid-v-show-good)

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
        "vue/valid-v-show": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-show-bad"></span>

**Incorreto**

`v-show` não tem sua expressão de visibilidade ou é colocado em um `<template>` que não tem um elemento DOM cujo display possa ser alterado.

```vue annotate="remove:2,3"
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

<span id="vue-valid-v-show-good"></span>

**Correto**

Aplique a expressão de visibilidade a um elemento renderizado, como `<div>`.

```vue annotate="add:2,3"
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) · [Todas as regras](all.md)

### `vue/valid-v-slot`

Exigir diretivas `v-slot` válidas

[Incorreto](#vue-valid-v-slot-bad) · [Correto](#vue-valid-v-slot-good)

Severidade padrão: `error`  
Predefinições: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/valid-v-slot": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-slot-bad"></span>

**Incorreto**

A diretiva de slot está em uma `<div>` nativa ou entra em conflito com outras declarações de slots padrão ou nomeados.

```vue annotate="remove:2,3,4"
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

<span id="vue-valid-v-slot-good"></span>

**Correto**

Declare o slot padrão de um componente nele próprio, ou seu slot nomeado em um filho `<template #header>`.

```vue annotate="add:2,3,4,5"
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) · [Todas as regras](all.md)

### `vue/valid-v-text`

Exigir diretivas `v-text` válidas

[Incorreto](#vue-valid-v-text-bad) · [Correto](#vue-valid-v-text-good)

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
        "vue/valid-v-text": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-text-bad"></span>

**Incorreto**

`v-text` não tem sua expressão de texto ou usa um argumento ou modificador que não é aceito.

```vue annotate="remove:2,3,4"
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

<span id="vue-valid-v-text-good"></span>

**Correto**

`v-text="msg"` é sintaticamente válido. A regra de estilo separada `vue/no-v-text` ainda pode preferir interpolação.

```vue annotate="add:2"
<template>
<div v-text="msg"></div>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) · [Todas as regras](all.md)

### `vue/warn-custom-block`

Alertar sobre blocos personalizados em arquivos SFC

[Incorreto](#vue-warn-custom-block-bad) · [Correto](#vue-warn-custom-block-good)

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
        "vue/warn-custom-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-warn-custom-block-bad"></span>

**Incorreto**

O SFC contém um bloco personalizado `<i18n>`, que precisa de uma integração externa além do processamento comum de template, script e style.

```vue annotate="remove:1,2,3,4"
<i18n>
{ "en": { "hello": "Hello" } }
</i18n>

<template>
  <p>{{ hello }}</p>
</template>
```

<span id="vue-warn-custom-block-good"></span>

**Correto**

O exemplo usa blocos padrão de template e script setup. Este aviso opcional de portabilidade não significa que todo bloco personalizado seja inválido no Vue.

```vue annotate="add:4,5,6,7"
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) · [Todas as regras](all.md)

### `vue/warn-custom-directive`

Alertar sobre diretivas personalizadas que precisam de registro

[Incorreto](#vue-warn-custom-directive-bad) · [Correto](#vue-warn-custom-directive-good)

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
        "vue/warn-custom-directive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-warn-custom-directive-bad"></span>

**Incorreto**

`v-focus`, `v-mask` e `v-click-outside` exigem implementações de diretivas específicas do projeto, que esta convenção opcional sinaliza.

```vue annotate="remove:2,3,4"
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

<span id="vue-warn-custom-directive-good"></span>

**Correto**

O exemplo usa as diretivas nativas `v-if`, `v-model` e `v-on`. Uma diretiva personalizada registrada corretamente ainda pode ser válida no Vue quando esta política estiver desativada.

```vue annotate="add:2,3,4"
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```

O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.

[Implementação](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) · [Todas as regras](all.md)
