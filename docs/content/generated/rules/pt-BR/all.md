---
title: "Todas as Regras de Patina"
---

# Todas as Regras de Patina

Use este índice para encontrar o objetivo de uma regra, as configurações padrão e seu grupo. As páginas dos grupos apresentam as explicações, a configuração, os exemplos completos incorretos e corretos e os limites atuais. Os links existentes para regras e exemplos continuam disponíveis aqui.

As 66 entradas de projeto levam aos arquivos compartilhados completos e aos exemplos incorretos e corretos na página de regras entre arquivos. Os 60 códigos do analisador mantêm seus limites reais: 19 códigos da CLI (18 pares de fontes validados e um projeto ilustrativo com seu grafo reativo preservado), 16 códigos experimentais do analisador Rust que essa etapa da CLI não emite individualmente e 25 contratos sem produtor atual. Configurar um ID não ativa um produtor indisponível.

A CLI pública expõe a mesma etapa por meio de `vize lint --cross-file`. Os códigos exibidos como `vize:croquis/cf/*` usam `croquis/cf/*` em `vue.linter.rules` (omita `vize:`). Diagnósticos de informação ou sugestão tornam-se avisos da CLI. As localizações relacionadas explicam a relação entre a origem e o consumidor.

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
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [Incorreto](#a11y-placeholder-label-option-bad) · [Correto](#a11y-placeholder-label-option-good) | Exigir uma opção de orientação em um select required |
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
| [`script/no-computed-properties-in-data`](#script-no-computed-properties-in-data) | [Incorreto](#script-no-computed-properties-in-data-bad) · [Correto](#script-no-computed-properties-in-data-good) | Proibir o uso de propriedades computadas dentro da função data do componente |
| [`script/no-deep-destructure-in-props`](#script-no-deep-destructure-in-props) | [Incorreto](#script-no-deep-destructure-in-props-bad) · [Correto](#script-no-deep-destructure-in-props-good) | Proibir desestruturação profundamente aninhada em defineProps |
| [`script/no-deprecated-data-object-declaration`](#script-no-deprecated-data-object-declaration) | [Incorreto](#script-no-deprecated-data-object-declaration-bad) · [Correto](#script-no-deprecated-data-object-declaration-good) | Proibir um literal de objeto como opção data do componente (o Vue 3 exige uma função) |
| [`script/no-deprecated-delete-set`](#script-no-deprecated-delete-set) | [Incorreto](#script-no-deprecated-delete-set-bad) · [Correto](#script-no-deprecated-delete-set-good) | Proibir $set e $delete do Vue 2, removidos no Vue 3 |
| [`script/no-deprecated-destroyed-lifecycle`](#script-no-deprecated-destroyed-lifecycle) | [Incorreto](#script-no-deprecated-destroyed-lifecycle-bad) · [Correto](#script-no-deprecated-destroyed-lifecycle-good) | Proibir os hooks de ciclo de vida obsoletos destroyed e beforeDestroy |
| [`script/no-deprecated-dollar-listeners-api`](#script-no-deprecated-dollar-listeners-api) | [Incorreto](#script-no-deprecated-dollar-listeners-api-bad) · [Correto](#script-no-deprecated-dollar-listeners-api-good) | Proibir a propriedade de instância $listeners removida no Vue 3 (incorporada a $attrs) |
| [`script/no-deprecated-dollar-scopedslots-api`](#script-no-deprecated-dollar-scopedslots-api) | [Incorreto](#script-no-deprecated-dollar-scopedslots-api-bad) · [Correto](#script-no-deprecated-dollar-scopedslots-api-good) | Proibir a propriedade de instância $scopedSlots removida no Vue 3 (usar $slots) |
| [`script/no-deprecated-events-api`](#script-no-deprecated-events-api) | [Incorreto](#script-no-deprecated-events-api-bad) · [Correto](#script-no-deprecated-events-api-good) | Proibir a API de eventos do Vue 2 removida ($on / $off / $once) |
| [`script/no-deprecated-model-definition`](#script-no-deprecated-model-definition) | [Incorreto](#script-no-deprecated-model-definition-bad) · [Correto](#script-no-deprecated-model-definition-good) | Proibir a opção model do Vue 2, removida no Vue 3 |
| [`script/no-deprecated-props-default-this`](#script-no-deprecated-props-default-this) | [Incorreto](#script-no-deprecated-props-default-this-bad) · [Correto](#script-no-deprecated-props-default-this-good) | Proibir `this` dentro de uma função de valor padrão ou validação de prop (removido no Vue 3) |
| [`script/no-deprecated-vue-config-keycodes`](#script-no-deprecated-vue-config-keycodes) | [Incorreto](#script-no-deprecated-vue-config-keycodes-bad) · [Correto](#script-no-deprecated-vue-config-keycodes-good) | Proibir Vue.config.keyCodes, removido no Vue 3 |
| [`script/no-dupe-keys`](#script-no-dupe-keys) | [Incorreto](#script-no-dupe-keys-bad) · [Correto](#script-no-dupe-keys-good) | Proibir chaves duplicadas entre props/data/computed/methods/setup/inject da Options API |
| [`script/no-duplicate-attr-inheritance`](#script-no-duplicate-attr-inheritance) | [Incorreto](#script-no-duplicate-attr-inheritance-bad) · [Correto](#script-no-duplicate-attr-inheritance-good) | Sinalizar um componente que aplica duas vezes seus atributos repassados |
| [`script/no-export-in-script-setup`](#script-no-export-in-script-setup) | [Incorreto](#script-no-export-in-script-setup-bad) · [Correto](#script-no-export-in-script-setup-good) | Proibir instruções export dentro de &lt;script setup&gt; |
| [`script/no-expose-after-await`](#script-no-expose-after-await) | [Incorreto](#script-no-expose-after-await-bad) · [Correto](#script-no-expose-after-await-good) | Proibir expose() e defineExpose() após um await |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [Incorreto](#script-no-get-current-instance-bad) · [Correto](#script-no-get-current-instance-good) | Proibir getCurrentInstance() no modo Vapor (retorna null) |
| [`script/no-import-compiler-macros`](#script-no-import-compiler-macros) | [Incorreto](#script-no-import-compiler-macros-bad) · [Correto](#script-no-import-compiler-macros-good) | Proibir a importação de macros do compilador Vue que são importadas automaticamente |
| [`script/no-internal-imports`](#script-no-internal-imports) | [Incorreto](#script-no-internal-imports-bad) · [Correto](#script-no-internal-imports-good) | Proibir importações de módulos internos do Vue |
| [`script/no-lifecycle-after-await`](#script-no-lifecycle-after-await) | [Incorreto](#script-no-lifecycle-after-await-bad) · [Correto](#script-no-lifecycle-after-await-good) | Proibir hooks de ciclo de vida registrados após um await em setup() |
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
| [`script/no-watch-after-await`](#script-no-watch-after-await) | [Incorreto](#script-no-watch-after-await-bad) · [Correto](#script-no-watch-after-await-good) | Proibir watchers registrados após um await em setup() sem guardar a função de parada |
| [`script/no-with-defaults`](#script-no-with-defaults) | [Incorreto](#script-no-with-defaults-bad) · [Correto](#script-no-with-defaults-good) | Desencorajar withDefaults em favor de valores padrão na desestruturação (Vue 3.5+) |
| [`script/one-component-per-file`](#script-one-component-per-file) | [Incorreto](#script-one-component-per-file-bad) · [Correto](#script-one-component-per-file-good) | Proibir mais de uma definição de componente em um arquivo |
| [`script/order-in-components`](#script-order-in-components) | [Incorreto](#script-order-in-components-bad) · [Correto](#script-order-in-components-good) | Exigir a ordem padrão das propriedades nas opções do componente |
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
| [`script/require-render-return`](#script-require-render-return) | [Incorreto](#script-require-render-return-bad) · [Correto](#script-require-render-return-good) | Exigir que a função render do componente devolva um valor |
| [`script/require-slots-as-functions`](#script-require-slots-as-functions) | [Incorreto](#script-require-slots-as-functions-bad) · [Correto](#script-require-slots-as-functions-good) | Exigir que as propriedades de $slots sejam chamadas como funções |
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
| [`vue/no-deprecated-v-is`](#vue-no-deprecated-v-is) | [Incorreto](#vue-no-deprecated-v-is-bad) · [Correto](#vue-no-deprecated-v-is-good) | Proibir a diretiva obsoleta `v-is` |
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
| [`vue/valid-v-else-if`](#vue-valid-v-else-if) | [Incorreto](#vue-valid-v-else-if-bad) · [Correto](#vue-valid-v-else-if-good) | Exigir diretivas `v-else-if` válidas |
| [`vue/valid-v-for`](#vue-valid-v-for) | [Incorreto](#vue-valid-v-for-bad) · [Correto](#vue-valid-v-for-good) | Exigir diretivas `v-for` válidas |
| [`vue/valid-v-html`](#vue-valid-v-html) | [Incorreto](#vue-valid-v-html-bad) · [Correto](#vue-valid-v-html-good) | Exigir diretivas `v-html` válidas |
| [`vue/valid-v-if`](#vue-valid-v-if) | [Incorreto](#vue-valid-v-if-bad) · [Correto](#vue-valid-v-if-good) | Exigir diretivas `v-if` válidas |
| [`vue/valid-v-is`](#vue-valid-v-is) | [Incorreto](#vue-valid-v-is-bad) · [Correto](#vue-valid-v-is-good) | Exigir diretivas `v-is` válidas |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [Incorreto](#vue-valid-v-memo-bad) · [Correto](#vue-valid-v-memo-good) | Exigir diretivas `v-memo` válidas |
| [`vue/valid-v-model`](#vue-valid-v-model) | [Incorreto](#vue-valid-v-model-bad) · [Correto](#vue-valid-v-model-good) | Exigir diretivas `v-model` válidas |
| [`vue/valid-v-on`](#vue-valid-v-on) | [Incorreto](#vue-valid-v-on-bad) · [Correto](#vue-valid-v-on-good) | Exigir diretivas `v-on` válidas |
| [`vue/valid-v-once`](#vue-valid-v-once) | [Incorreto](#vue-valid-v-once-bad) · [Correto](#vue-valid-v-once-good) | Exigir diretivas `v-once` válidas |
| [`vue/valid-v-pre`](#vue-valid-v-pre) | [Incorreto](#vue-valid-v-pre-bad) · [Correto](#vue-valid-v-pre-good) | Exigir diretivas `v-pre` válidas |
| [`vue/valid-v-show`](#vue-valid-v-show) | [Incorreto](#vue-valid-v-show-bad) · [Correto](#vue-valid-v-show-good) | Exigir diretivas `v-show` válidas |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [Incorreto](#vue-valid-v-slot-bad) · [Correto](#vue-valid-v-slot-good) | Exigir diretivas `v-slot` válidas |
| [`vue/valid-v-text`](#vue-valid-v-text) | [Incorreto](#vue-valid-v-text-bad) · [Correto](#vue-valid-v-text-good) | Exigir diretivas `v-text` válidas |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [Incorreto](#vue-warn-custom-block-bad) · [Correto](#vue-warn-custom-block-good) | Alertar sobre blocos personalizados em arquivos SFC |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [Incorreto](#vue-warn-custom-directive-bad) · [Correto](#vue-warn-custom-directive-good) | Alertar sobre diretivas personalizadas que precisam de registro |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `a11y/alt-text`

Exigir texto alternativo para elementos de mídia

<span id="a11y-alt-text-bad"></span>

[Incorreto](./accessibility.md#a11y-alt-text-bad)

<span id="a11y-alt-text-good"></span>

[Correto](./accessibility.md#a11y-alt-text-good)

[Exemplos do grupo](./accessibility.md#a11y-alt-text) · [Referência da regra](/rules/reference/a11y-alt-text.md)

### `a11y/anchor-has-content`

Exigir conteúdo acessível nos elementos de âncora

<span id="a11y-anchor-has-content-bad"></span>

[Incorreto](./accessibility.md#a11y-anchor-has-content-bad)

<span id="a11y-anchor-has-content-good"></span>

[Correto](./accessibility.md#a11y-anchor-has-content-good)

[Exemplos do grupo](./accessibility.md#a11y-anchor-has-content) · [Referência da regra](/rules/reference/a11y-anchor-has-content.md)

### `a11y/anchor-is-valid`

Exigir um href válido nos elementos de âncora

<span id="a11y-anchor-is-valid-bad"></span>

[Incorreto](./accessibility.md#a11y-anchor-is-valid-bad)

<span id="a11y-anchor-is-valid-good"></span>

[Correto](./accessibility.md#a11y-anchor-is-valid-good)

[Exemplos do grupo](./accessibility.md#a11y-anchor-is-valid) · [Referência da regra](/rules/reference/a11y-anchor-is-valid.md)

### `a11y/aria-props`

Proibir atributos ARIA inválidos

<span id="a11y-aria-props-bad"></span>

[Incorreto](./accessibility.md#a11y-aria-props-bad)

<span id="a11y-aria-props-good"></span>

[Correto](./accessibility.md#a11y-aria-props-good)

[Exemplos do grupo](./accessibility.md#a11y-aria-props) · [Referência da regra](/rules/reference/a11y-aria-props.md)

### `a11y/aria-role`

Exigir que elementos com papéis ARIA usem um papel ARIA válido e não abstrato

<span id="a11y-aria-role-bad"></span>

[Incorreto](./accessibility.md#a11y-aria-role-bad)

<span id="a11y-aria-role-good"></span>

[Correto](./accessibility.md#a11y-aria-role-good)

[Exemplos do grupo](./accessibility.md#a11y-aria-role) · [Referência da regra](/rules/reference/a11y-aria-role.md)

### `a11y/aria-unsupported-elements`

Proibir atributos ARIA em elementos que não os aceitam

<span id="a11y-aria-unsupported-elements-bad"></span>

[Incorreto](./accessibility.md#a11y-aria-unsupported-elements-bad)

<span id="a11y-aria-unsupported-elements-good"></span>

[Correto](./accessibility.md#a11y-aria-unsupported-elements-good)

[Exemplos do grupo](./accessibility.md#a11y-aria-unsupported-elements) · [Referência da regra](/rules/reference/a11y-aria-unsupported-elements.md)

### `a11y/click-events-have-key-events`

Exigir manipuladores de eventos de teclado junto aos eventos de clique

<span id="a11y-click-events-have-key-events-bad"></span>

[Incorreto](./accessibility.md#a11y-click-events-have-key-events-bad)

<span id="a11y-click-events-have-key-events-good"></span>

[Correto](./accessibility.md#a11y-click-events-have-key-events-good)

[Exemplos do grupo](./accessibility.md#a11y-click-events-have-key-events) · [Referência da regra](/rules/reference/a11y-click-events-have-key-events.md)

### `a11y/form-control-has-label`

Exigir rótulos associados aos controles de formulário

<span id="a11y-form-control-has-label-bad"></span>

[Incorreto](./accessibility.md#a11y-form-control-has-label-bad)

<span id="a11y-form-control-has-label-good"></span>

[Correto](./accessibility.md#a11y-form-control-has-label-good)

[Exemplos do grupo](./accessibility.md#a11y-form-control-has-label) · [Referência da regra](/rules/reference/a11y-form-control-has-label.md)

### `a11y/heading-has-content`

Exigir conteúdo acessível nos elementos de título

<span id="a11y-heading-has-content-bad"></span>

[Incorreto](./accessibility.md#a11y-heading-has-content-bad)

<span id="a11y-heading-has-content-good"></span>

[Correto](./accessibility.md#a11y-heading-has-content-good)

[Exemplos do grupo](./accessibility.md#a11y-heading-has-content) · [Referência da regra](/rules/reference/a11y-heading-has-content.md)

### `a11y/heading-levels`

Proibir que níveis de título sejam pulados

<span id="a11y-heading-levels-bad"></span>

[Incorreto](./accessibility.md#a11y-heading-levels-bad)

<span id="a11y-heading-levels-good"></span>

[Correto](./accessibility.md#a11y-heading-levels-good)

[Exemplos do grupo](./accessibility.md#a11y-heading-levels) · [Referência da regra](/rules/reference/a11y-heading-levels.md)

### `a11y/iframe-has-title`

Exigir um atributo title nos elementos iframe

<span id="a11y-iframe-has-title-bad"></span>

[Incorreto](./accessibility.md#a11y-iframe-has-title-bad)

<span id="a11y-iframe-has-title-good"></span>

[Correto](./accessibility.md#a11y-iframe-has-title-good)

[Exemplos do grupo](./accessibility.md#a11y-iframe-has-title) · [Referência da regra](/rules/reference/a11y-iframe-has-title.md)

### `a11y/img-alt`

Exigir o atributo alt nas imagens para garantir acessibilidade

<span id="a11y-img-alt-bad"></span>

[Incorreto](./accessibility.md#a11y-img-alt-bad)

<span id="a11y-img-alt-good"></span>

[Correto](./accessibility.md#a11y-img-alt-good)

[Exemplos do grupo](./accessibility.md#a11y-img-alt) · [Referência da regra](/rules/reference/a11y-img-alt.md)

### `a11y/interactive-supports-focus`

Exigir que elementos com papéis interativos possam receber foco

<span id="a11y-interactive-supports-focus-bad"></span>

[Incorreto](./accessibility.md#a11y-interactive-supports-focus-bad)

<span id="a11y-interactive-supports-focus-good"></span>

[Correto](./accessibility.md#a11y-interactive-supports-focus-good)

[Exemplos do grupo](./accessibility.md#a11y-interactive-supports-focus) · [Referência da regra](/rules/reference/a11y-interactive-supports-focus.md)

### `a11y/label-has-for`

Exigir controles de formulário associados aos rótulos

<span id="a11y-label-has-for-bad"></span>

[Incorreto](./accessibility.md#a11y-label-has-for-bad)

<span id="a11y-label-has-for-good"></span>

[Correto](./accessibility.md#a11y-label-has-for-good)

[Exemplos do grupo](./accessibility.md#a11y-label-has-for) · [Referência da regra](/rules/reference/a11y-label-has-for.md)

### `a11y/landmark-roles`

Validar a posição e a unicidade dos papéis de regiões de referência

<span id="a11y-landmark-roles-bad"></span>

[Incorreto](./accessibility.md#a11y-landmark-roles-bad)

<span id="a11y-landmark-roles-good"></span>

[Correto](./accessibility.md#a11y-landmark-roles-good)

[Exemplos do grupo](./accessibility.md#a11y-landmark-roles) · [Referência da regra](/rules/reference/a11y-landmark-roles.md)

### `a11y/media-has-caption`

Exigir legendas nos elementos de mídia

<span id="a11y-media-has-caption-bad"></span>

[Incorreto](./accessibility.md#a11y-media-has-caption-bad)

<span id="a11y-media-has-caption-good"></span>

[Correto](./accessibility.md#a11y-media-has-caption-good)

[Exemplos do grupo](./accessibility.md#a11y-media-has-caption) · [Referência da regra](/rules/reference/a11y-media-has-caption.md)

### `a11y/mouse-events-have-key-events`

Exigir eventos de foco e perda de foco junto aos eventos de mouse

<span id="a11y-mouse-events-have-key-events-bad"></span>

[Incorreto](./accessibility.md#a11y-mouse-events-have-key-events-bad)

<span id="a11y-mouse-events-have-key-events-good"></span>

[Correto](./accessibility.md#a11y-mouse-events-have-key-events-good)

[Exemplos do grupo](./accessibility.md#a11y-mouse-events-have-key-events) · [Referência da regra](/rules/reference/a11y-mouse-events-have-key-events.md)

### `a11y/no-access-key`

Proibir o uso do atributo accesskey

<span id="a11y-no-access-key-bad"></span>

[Incorreto](./accessibility.md#a11y-no-access-key-bad)

<span id="a11y-no-access-key-good"></span>

[Correto](./accessibility.md#a11y-no-access-key-good)

[Exemplos do grupo](./accessibility.md#a11y-no-access-key) · [Referência da regra](/rules/reference/a11y-no-access-key.md)

### `a11y/no-aria-hidden-on-focusable`

Proibir aria-hidden="true" em elementos que podem receber foco

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

[Incorreto](./accessibility.md#a11y-no-aria-hidden-on-focusable-bad)

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

[Correto](./accessibility.md#a11y-no-aria-hidden-on-focusable-good)

[Exemplos do grupo](./accessibility.md#a11y-no-aria-hidden-on-focusable) · [Referência da regra](/rules/reference/a11y-no-aria-hidden-on-focusable.md)

### `a11y/no-autofocus`

Proibir o uso do atributo autofocus

<span id="a11y-no-autofocus-bad"></span>

[Incorreto](./accessibility.md#a11y-no-autofocus-bad)

<span id="a11y-no-autofocus-good"></span>

[Correto](./accessibility.md#a11y-no-autofocus-good)

[Exemplos do grupo](./accessibility.md#a11y-no-autofocus) · [Referência da regra](/rules/reference/a11y-no-autofocus.md)

### `a11y/no-distracting-elements`

Proibir elementos que causam distração, como &lt;marquee&gt; e &lt;blink&gt;

<span id="a11y-no-distracting-elements-bad"></span>

[Incorreto](./accessibility.md#a11y-no-distracting-elements-bad)

<span id="a11y-no-distracting-elements-good"></span>

[Correto](./accessibility.md#a11y-no-distracting-elements-good)

[Exemplos do grupo](./accessibility.md#a11y-no-distracting-elements) · [Referência da regra](/rules/reference/a11y-no-distracting-elements.md)

### `a11y/no-i-for-icon`

Proibir o uso do elemento &lt;i&gt; para ícones

<span id="a11y-no-i-for-icon-bad"></span>

[Incorreto](./accessibility.md#a11y-no-i-for-icon-bad)

<span id="a11y-no-i-for-icon-good"></span>

[Correto](./accessibility.md#a11y-no-i-for-icon-good)

[Exemplos do grupo](./accessibility.md#a11y-no-i-for-icon) · [Referência da regra](/rules/reference/a11y-no-i-for-icon.md)

### `a11y/no-redundant-roles`

Proibir papéis ARIA redundantes

<span id="a11y-no-redundant-roles-bad"></span>

[Incorreto](./accessibility.md#a11y-no-redundant-roles-bad)

<span id="a11y-no-redundant-roles-good"></span>

[Correto](./accessibility.md#a11y-no-redundant-roles-good)

[Exemplos do grupo](./accessibility.md#a11y-no-redundant-roles) · [Referência da regra](/rules/reference/a11y-no-redundant-roles.md)

### `a11y/no-refer-to-non-existent-id`

Proibir referências a IDs inexistentes

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

[Incorreto](./accessibility.md#a11y-no-refer-to-non-existent-id-bad)

<span id="a11y-no-refer-to-non-existent-id-good"></span>

[Correto](./accessibility.md#a11y-no-refer-to-non-existent-id-good)

[Exemplos do grupo](./accessibility.md#a11y-no-refer-to-non-existent-id) · [Referência da regra](/rules/reference/a11y-no-refer-to-non-existent-id.md)

### `a11y/no-role-presentation-on-focusable`

Proibir role="presentation" ou role="none" em elementos que podem receber foco

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

[Incorreto](./accessibility.md#a11y-no-role-presentation-on-focusable-bad)

<span id="a11y-no-role-presentation-on-focusable-good"></span>

[Correto](./accessibility.md#a11y-no-role-presentation-on-focusable-good)

[Exemplos do grupo](./accessibility.md#a11y-no-role-presentation-on-focusable) · [Referência da regra](/rules/reference/a11y-no-role-presentation-on-focusable.md)

### `a11y/no-static-element-interactions`

Proibir manipuladores de eventos em elementos estáticos

<span id="a11y-no-static-element-interactions-bad"></span>

[Incorreto](./accessibility.md#a11y-no-static-element-interactions-bad)

<span id="a11y-no-static-element-interactions-good"></span>

[Correto](./accessibility.md#a11y-no-static-element-interactions-good)

[Exemplos do grupo](./accessibility.md#a11y-no-static-element-interactions) · [Referência da regra](/rules/reference/a11y-no-static-element-interactions.md)

### `a11y/placeholder-label-option`

Exigir uma opção de orientação em um select required

<span id="a11y-placeholder-label-option-bad"></span>

[Incorreto](./accessibility.md#a11y-placeholder-label-option-bad)

<span id="a11y-placeholder-label-option-good"></span>

[Correto](./accessibility.md#a11y-placeholder-label-option-good)

[Exemplos do grupo](./accessibility.md#a11y-placeholder-label-option) · [Referência da regra](/rules/reference/a11y-placeholder-label-option.md)

### `a11y/role-has-required-aria-props`

Exigir as propriedades obrigatórias dos papéis ARIA

<span id="a11y-role-has-required-aria-props-bad"></span>

[Incorreto](./accessibility.md#a11y-role-has-required-aria-props-bad)

<span id="a11y-role-has-required-aria-props-good"></span>

[Correto](./accessibility.md#a11y-role-has-required-aria-props-good)

[Exemplos do grupo](./accessibility.md#a11y-role-has-required-aria-props) · [Referência da regra](/rules/reference/a11y-role-has-required-aria-props.md)

### `a11y/tabindex-no-positive`

Proibir valores positivos de tabindex

<span id="a11y-tabindex-no-positive-bad"></span>

[Incorreto](./accessibility.md#a11y-tabindex-no-positive-bad)

<span id="a11y-tabindex-no-positive-good"></span>

[Correto](./accessibility.md#a11y-tabindex-no-positive-good)

[Exemplos do grupo](./accessibility.md#a11y-tabindex-no-positive) · [Referência da regra](/rules/reference/a11y-tabindex-no-positive.md)

### `a11y/use-list`

Sugerir elementos de lista para textos que parecem itens de lista

<span id="a11y-use-list-bad"></span>

[Incorreto](./accessibility.md#a11y-use-list-bad)

<span id="a11y-use-list-good"></span>

[Correto](./accessibility.md#a11y-use-list-good)

[Exemplos do grupo](./accessibility.md#a11y-use-list) · [Referência da regra](/rules/reference/a11y-use-list.md)

### `css/no-display-none`

Sugerir v-show em vez de display: none

<span id="css-no-display-none-bad"></span>

[Incorreto](./musea-and-css.md#css-no-display-none-bad)

<span id="css-no-display-none-good"></span>

[Correto](./musea-and-css.md#css-no-display-none-good)

[Exemplos do grupo](./musea-and-css.md#css-no-display-none) · [Referência da regra](/rules/reference/css-no-display-none.md)

### `css/no-hardcoded-values`

Sugerir variáveis CSS em vez de valores fixos escritos diretamente

<span id="css-no-hardcoded-values-bad"></span>

[Incorreto](./musea-and-css.md#css-no-hardcoded-values-bad)

<span id="css-no-hardcoded-values-good"></span>

[Correto](./musea-and-css.md#css-no-hardcoded-values-good)

[Exemplos do grupo](./musea-and-css.md#css-no-hardcoded-values) · [Referência da regra](/rules/reference/css-no-hardcoded-values.md)

### `css/no-id-selectors`

Desencorajar o uso de seletores de ID no CSS

<span id="css-no-id-selectors-bad"></span>

[Incorreto](./musea-and-css.md#css-no-id-selectors-bad)

<span id="css-no-id-selectors-good"></span>

[Correto](./musea-and-css.md#css-no-id-selectors-good)

[Exemplos do grupo](./musea-and-css.md#css-no-id-selectors) · [Referência da regra](/rules/reference/css-no-id-selectors.md)

### `css/no-important`

Desencorajar o uso de !important no CSS

<span id="css-no-important-bad"></span>

[Incorreto](./musea-and-css.md#css-no-important-bad)

<span id="css-no-important-good"></span>

[Correto](./musea-and-css.md#css-no-important-good)

[Exemplos do grupo](./musea-and-css.md#css-no-important) · [Referência da regra](/rules/reference/css-no-important.md)

### `css/no-utility-classes`

Alertar sobre a implementação de classes utilitárias nos estilos de componentes

<span id="css-no-utility-classes-bad"></span>

[Incorreto](./musea-and-css.md#css-no-utility-classes-bad)

<span id="css-no-utility-classes-good"></span>

[Correto](./musea-and-css.md#css-no-utility-classes-good)

[Exemplos do grupo](./musea-and-css.md#css-no-utility-classes) · [Referência da regra](/rules/reference/css-no-utility-classes.md)

### `css/no-v-bind-performance`

Alertar sobre o custo de desempenho do v-bind() no CSS

<span id="css-no-v-bind-performance-bad"></span>

[Incorreto](./musea-and-css.md#css-no-v-bind-performance-bad)

<span id="css-no-v-bind-performance-good"></span>

[Correto](./musea-and-css.md#css-no-v-bind-performance-good)

[Exemplos do grupo](./musea-and-css.md#css-no-v-bind-performance) · [Referência da regra](/rules/reference/css-no-v-bind-performance.md)

### `css/prefer-logical-properties`

Recomendar propriedades lógicas de CSS para melhorar o suporte à internacionalização

<span id="css-prefer-logical-properties-bad"></span>

[Incorreto](./musea-and-css.md#css-prefer-logical-properties-bad)

<span id="css-prefer-logical-properties-good"></span>

[Correto](./musea-and-css.md#css-prefer-logical-properties-good)

[Exemplos do grupo](./musea-and-css.md#css-prefer-logical-properties) · [Referência da regra](/rules/reference/css-prefer-logical-properties.md)

### `css/prefer-nested-selectors`

Recomendar o aninhamento de CSS para seletores de descendentes

<span id="css-prefer-nested-selectors-bad"></span>

[Incorreto](./musea-and-css.md#css-prefer-nested-selectors-bad)

<span id="css-prefer-nested-selectors-good"></span>

[Correto](./musea-and-css.md#css-prefer-nested-selectors-good)

[Exemplos do grupo](./musea-and-css.md#css-prefer-nested-selectors) · [Referência da regra](/rules/reference/css-prefer-nested-selectors.md)

### `css/prefer-slotted`

Recomendar ::v-slotted() para estilizar o conteúdo de slots

<span id="css-prefer-slotted-bad"></span>

[Incorreto](./musea-and-css.md#css-prefer-slotted-bad)

<span id="css-prefer-slotted-good"></span>

[Correto](./musea-and-css.md#css-prefer-slotted-good)

[Exemplos do grupo](./musea-and-css.md#css-prefer-slotted) · [Referência da regra](/rules/reference/css-prefer-slotted.md)

### `css/require-font-display`

Exigir font-display nas regras @font-face

<span id="css-require-font-display-bad"></span>

[Incorreto](./musea-and-css.md#css-require-font-display-bad)

<span id="css-require-font-display-good"></span>

[Correto](./musea-and-css.md#css-require-font-display-good)

[Exemplos do grupo](./musea-and-css.md#css-require-font-display) · [Referência da regra](/rules/reference/css-require-font-display.md)

### `ecosystem/nuxt-prefer-nuxt-link`

Preferir NuxtLink para links internos da aplicação

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

[Incorreto](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link-bad)

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

[Correto](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link-good)

[Exemplos do grupo](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link) · [Referência da regra](/rules/reference/ecosystem-nuxt-prefer-nuxt-link.md)

### `ecosystem/pinia-prefer-store-to-refs`

Preferir storeToRefs() ao desestruturar stores do Pinia

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

[Incorreto](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs-bad)

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

[Correto](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs-good)

[Exemplos do grupo](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs) · [Referência da regra](/rules/reference/ecosystem-pinia-prefer-store-to-refs.md)

### `ecosystem/router-link-require-to`

Exigir um destino `to` nos componentes RouterLink e NuxtLink

<span id="ecosystem-router-link-require-to-bad"></span>

[Incorreto](./ecosystem.md#ecosystem-router-link-require-to-bad)

<span id="ecosystem-router-link-require-to-good"></span>

[Correto](./ecosystem.md#ecosystem-router-link-require-to-good)

[Exemplos do grupo](./ecosystem.md#ecosystem-router-link-require-to) · [Referência da regra](/rules/reference/ecosystem-router-link-require-to.md)

### `ecosystem/void-link-require-href`

Exigir `href` nos componentes Link do Void Vue

<span id="ecosystem-void-link-require-href-bad"></span>

[Incorreto](./ecosystem.md#ecosystem-void-link-require-href-bad)

<span id="ecosystem-void-link-require-href-good"></span>

[Correto](./ecosystem.md#ecosystem-void-link-require-href-good)

[Exemplos do grupo](./ecosystem.md#ecosystem-void-link-require-href) · [Referência da regra](/rules/reference/ecosystem-void-link-require-href.md)

### `ecosystem/void-link-valid-method`

Validar props method estáticas do Link do Void Vue

<span id="ecosystem-void-link-valid-method-bad"></span>

[Incorreto](./ecosystem.md#ecosystem-void-link-valid-method-bad)

<span id="ecosystem-void-link-valid-method-good"></span>

[Correto](./ecosystem.md#ecosystem-void-link-valid-method-good)

[Exemplos do grupo](./ecosystem.md#ecosystem-void-link-valid-method) · [Referência da regra](/rules/reference/ecosystem-void-link-valid-method.md)

### `ecosystem/vue-i18n-no-missing-key`

Relatar chaves estáticas do vue-i18n ausentes nas mensagens locais do SFC

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

[Incorreto](./ecosystem.md#ecosystem-vue-i18n-no-missing-key-bad)

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

[Correto](./ecosystem.md#ecosystem-vue-i18n-no-missing-key-good)

[Exemplos do grupo](./ecosystem.md#ecosystem-vue-i18n-no-missing-key) · [Referência da regra](/rules/reference/ecosystem-vue-i18n-no-missing-key.md)

### `ecosystem/vue-router-extra-param`

A rota não declara tab; o Vue Router o descarta.

<span id="ecosystem-vue-router-extra-param-bad"></span>

[Incorreto](./cross-file.md#ecosystem-vue-router-extra-param-bad)

<span id="ecosystem-vue-router-extra-param-good"></span>

[Correto](./cross-file.md#ecosystem-vue-router-extra-param-good)

[Exemplos do grupo](./cross-file.md#ecosystem-vue-router-extra-param) · [Referência da regra](/rules/project/ecosystem-vue-router-extra-param.md)

### `ecosystem/vue-router-missing-param`

O parâmetro obrigatório postId está ausente; depender da rota atual é frágil.

<span id="ecosystem-vue-router-missing-param-bad"></span>

[Incorreto](./cross-file.md#ecosystem-vue-router-missing-param-bad)

<span id="ecosystem-vue-router-missing-param-good"></span>

[Correto](./cross-file.md#ecosystem-vue-router-missing-param-good)

[Exemplos do grupo](./cross-file.md#ecosystem-vue-router-missing-param) · [Referência da regra](/rules/project/ecosystem-vue-router-missing-param.md)

### `ecosystem/vue-router-param-type`

postId não é repetível, portanto um array é inválido.

<span id="ecosystem-vue-router-param-type-bad"></span>

[Incorreto](./cross-file.md#ecosystem-vue-router-param-type-bad)

<span id="ecosystem-vue-router-param-type-good"></span>

[Correto](./cross-file.md#ecosystem-vue-router-param-type-good)

[Exemplos do grupo](./cross-file.md#ecosystem-vue-router-param-type) · [Referência da regra](/rules/project/ecosystem-vue-router-param-type.md)

### `ecosystem/vue-router-prefer-named-link`

Preferir objetos de rotas nomeadas a strings de caminho estáticas no RouterLink

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

[Incorreto](./ecosystem.md#ecosystem-vue-router-prefer-named-link-bad)

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

[Correto](./ecosystem.md#ecosystem-vue-router-prefer-named-link-good)

[Exemplos do grupo](./ecosystem.md#ecosystem-vue-router-prefer-named-link) · [Referência da regra](/rules/reference/ecosystem-vue-router-prefer-named-link.md)

### `ecosystem/vue-router-prefer-named-push`

Preferir objetos de rotas nomeadas para navegação programática com Vue Router

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

[Incorreto](./ecosystem.md#ecosystem-vue-router-prefer-named-push-bad)

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

[Correto](./ecosystem.md#ecosystem-vue-router-prefer-named-push-good)

[Exemplos do grupo](./ecosystem.md#ecosystem-vue-router-prefer-named-push) · [Referência da regra](/rules/reference/ecosystem-vue-router-prefer-named-push.md)

### `ecosystem/vue-router-unknown-route`

O nome está ausente do roteador completo instalado.

<span id="ecosystem-vue-router-unknown-route-bad"></span>

[Incorreto](./cross-file.md#ecosystem-vue-router-unknown-route-bad)

<span id="ecosystem-vue-router-unknown-route-good"></span>

[Correto](./cross-file.md#ecosystem-vue-router-unknown-route-good)

[Exemplos do grupo](./cross-file.md#ecosystem-vue-router-unknown-route) · [Referência da regra](/rules/project/ecosystem-vue-router-unknown-route.md)

### `ecosystem/vue-test-utils-no-html-snapshot`

Evitar snapshots de wrapper.html() nos testes com Vue Test Utils

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

[Incorreto](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot-bad)

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

[Correto](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot-good)

[Exemplos do grupo](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot) · [Referência da regra](/rules/reference/ecosystem-vue-test-utils-no-html-snapshot.md)

### `html/cross-component-nesting`

Verificar o aninhamento HTML real após compor os componentes importados.

<span id="html-cross-component-nesting-bad"></span>

[Incorreto](./cross-file.md#html-cross-component-nesting-bad)

<span id="html-cross-component-nesting-good"></span>

[Correto](./cross-file.md#html-cross-component-nesting-good)

[Exemplos do grupo](./cross-file.md#html-cross-component-nesting) · [Referência da regra](/rules/project/html-cross-component-nesting.md)

### `html/deprecated-attr`

Proibir atributos HTML obsoletos

<span id="html-deprecated-attr-bad"></span>

[Incorreto](./html.md#html-deprecated-attr-bad)

<span id="html-deprecated-attr-good"></span>

[Correto](./html.md#html-deprecated-attr-good)

[Exemplos do grupo](./html.md#html-deprecated-attr) · [Referência da regra](/rules/reference/html-deprecated-attr.md)

### `html/deprecated-element`

Proibir elementos HTML obsoletos

<span id="html-deprecated-element-bad"></span>

[Incorreto](./html.md#html-deprecated-element-bad)

<span id="html-deprecated-element-good"></span>

[Correto](./html.md#html-deprecated-element-good)

[Exemplos do grupo](./html.md#html-deprecated-element) · [Referência da regra](/rules/reference/html-deprecated-element.md)

### `html/id-duplication`

Proibir IDs de elementos duplicados

<span id="html-id-duplication-bad"></span>

[Incorreto](./html.md#html-id-duplication-bad)

<span id="html-id-duplication-good"></span>

[Correto](./html.md#html-id-duplication-good)

[Exemplos do grupo](./html.md#html-id-duplication) · [Referência da regra](/rules/reference/html-id-duplication.md)

### `html/no-consecutive-br`

Proibir elementos &lt;br&gt; consecutivos

<span id="html-no-consecutive-br-bad"></span>

[Incorreto](./html.md#html-no-consecutive-br-bad)

<span id="html-no-consecutive-br-good"></span>

[Correto](./html.md#html-no-consecutive-br-good)

[Exemplos do grupo](./html.md#html-no-consecutive-br) · [Referência da regra](/rules/reference/html-no-consecutive-br.md)

### `html/no-dupe-style-properties`

Proibir propriedades duplicadas em atributos de estilo em linha

<span id="html-no-dupe-style-properties-bad"></span>

[Incorreto](./html.md#html-no-dupe-style-properties-bad)

<span id="html-no-dupe-style-properties-good"></span>

[Correto](./html.md#html-no-dupe-style-properties-good)

[Exemplos do grupo](./html.md#html-no-dupe-style-properties) · [Referência da regra](/rules/reference/html-no-dupe-style-properties.md)

### `html/no-duplicate-class`

Proibir nomes de classe duplicados em um atributo class estático

<span id="html-no-duplicate-class-bad"></span>

[Incorreto](./html.md#html-no-duplicate-class-bad)

<span id="html-no-duplicate-class-good"></span>

[Correto](./html.md#html-no-duplicate-class-good)

[Exemplos do grupo](./html.md#html-no-duplicate-class) · [Referência da regra](/rules/reference/html-no-duplicate-class.md)

### `html/no-duplicate-dt`

Proibir nomes &lt;dt&gt; duplicados em &lt;dl&gt;

<span id="html-no-duplicate-dt-bad"></span>

[Incorreto](./html.md#html-no-duplicate-dt-bad)

<span id="html-no-duplicate-dt-good"></span>

[Correto](./html.md#html-no-duplicate-dt-good)

[Exemplos do grupo](./html.md#html-no-duplicate-dt) · [Referência da regra](/rules/reference/html-no-duplicate-dt.md)

### `html/no-empty-palpable-content`

Proibir elementos vazios que esperam conteúdo visível

<span id="html-no-empty-palpable-content-bad"></span>

[Incorreto](./html.md#html-no-empty-palpable-content-bad)

<span id="html-no-empty-palpable-content-good"></span>

[Correto](./html.md#html-no-empty-palpable-content-good)

[Exemplos do grupo](./html.md#html-no-empty-palpable-content) · [Referência da regra](/rules/reference/html-no-empty-palpable-content.md)

### `html/require-datetime`

Exigir o atributo datetime no elemento &lt;time&gt;

<span id="html-require-datetime-bad"></span>

[Incorreto](./html.md#html-require-datetime-bad)

<span id="html-require-datetime-good"></span>

[Correto](./html.md#html-require-datetime-good)

[Exemplos do grupo](./html.md#html-require-datetime) · [Referência da regra](/rules/reference/html-require-datetime.md)

### `musea/no-empty-variant`

Proibir blocos &lt;variant&gt; vazios

<span id="musea-no-empty-variant-bad"></span>

[Incorreto](./musea-and-css.md#musea-no-empty-variant-bad)

<span id="musea-no-empty-variant-good"></span>

[Correto](./musea-and-css.md#musea-no-empty-variant-good)

[Exemplos do grupo](./musea-and-css.md#musea-no-empty-variant) · [Referência da regra](/rules/reference/musea-no-empty-variant.md)

### `musea/prefer-design-tokens`

Preferir variáveis CSS de tokens de design a valores primitivos fixos escritos diretamente

<span id="musea-prefer-design-tokens-bad"></span>

[Incorreto](./musea-and-css.md#musea-prefer-design-tokens-bad)

<span id="musea-prefer-design-tokens-good"></span>

[Correto](./musea-and-css.md#musea-prefer-design-tokens-good)

[Exemplos do grupo](./musea-and-css.md#musea-prefer-design-tokens) · [Referência da regra](/rules/reference/musea-prefer-design-tokens.md)

### `musea/require-component`

Exigir o atributo component no bloco &lt;art&gt;

<span id="musea-require-component-bad"></span>

[Incorreto](./musea-and-css.md#musea-require-component-bad)

<span id="musea-require-component-good"></span>

[Correto](./musea-and-css.md#musea-require-component-good)

[Exemplos do grupo](./musea-and-css.md#musea-require-component) · [Referência da regra](/rules/reference/musea-require-component.md)

### `musea/require-title`

Exigir o atributo title no bloco &lt;art&gt;

<span id="musea-require-title-bad"></span>

[Incorreto](./musea-and-css.md#musea-require-title-bad)

<span id="musea-require-title-good"></span>

[Correto](./musea-and-css.md#musea-require-title-good)

[Exemplos do grupo](./musea-and-css.md#musea-require-title) · [Referência da regra](/rules/reference/musea-require-title.md)

### `musea/unique-variant-names`

Exigir nomes de variante únicos

<span id="musea-unique-variant-names-bad"></span>

[Incorreto](./musea-and-css.md#musea-unique-variant-names-bad)

<span id="musea-unique-variant-names-good"></span>

[Correto](./musea-and-css.md#musea-unique-variant-names-good)

[Exemplos do grupo](./musea-and-css.md#musea-unique-variant-names) · [Referência da regra](/rules/reference/musea-unique-variant-names.md)

### `musea/valid-variant`

Exigir o atributo name nos blocos &lt;variant&gt;

<span id="musea-valid-variant-bad"></span>

[Incorreto](./musea-and-css.md#musea-valid-variant-bad)

<span id="musea-valid-variant-good"></span>

[Correto](./musea-and-css.md#musea-valid-variant-good)

[Exemplos do grupo](./musea-and-css.md#musea-valid-variant) · [Referência da regra](/rules/reference/musea-valid-variant.md)

### `nuxt/no-nuxt-config-test-key`

Proibir a definição da chave `test` na configuração do Nuxt

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

[Incorreto](./ecosystem.md#nuxt-no-nuxt-config-test-key-bad)

<span id="nuxt-no-nuxt-config-test-key-good"></span>

[Correto](./ecosystem.md#nuxt-no-nuxt-config-test-key-good)

[Exemplos do grupo](./ecosystem.md#nuxt-no-nuxt-config-test-key) · [Referência da regra](/rules/reference/nuxt-no-nuxt-config-test-key.md)

### `nuxt/no-page-meta-runtime-values`

Proibir valores do contexto de execução no nível de avaliação imediata de `definePageMeta`, que é extraído para um fragmento separado durante a compilação e executado antes do setup do componente

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

[Incorreto](./ecosystem.md#nuxt-no-page-meta-runtime-values-bad)

<span id="nuxt-no-page-meta-runtime-values-good"></span>

[Correto](./ecosystem.md#nuxt-no-page-meta-runtime-values-good)

[Exemplos do grupo](./ecosystem.md#nuxt-no-page-meta-runtime-values) · [Referência da regra](/rules/reference/nuxt-no-page-meta-runtime-values.md)

### `nuxt/nuxt-config-keys-order`

Preferir a ordem recomendada das propriedades de configuração do Nuxt

<span id="nuxt-nuxt-config-keys-order-bad"></span>

[Incorreto](./ecosystem.md#nuxt-nuxt-config-keys-order-bad)

<span id="nuxt-nuxt-config-keys-order-good"></span>

[Correto](./ecosystem.md#nuxt-nuxt-config-keys-order-good)

[Exemplos do grupo](./ecosystem.md#nuxt-nuxt-config-keys-order) · [Referência da regra](/rules/reference/nuxt-nuxt-config-keys-order.md)

### `nuxt/prefer-import-meta`

Preferir `import.meta.*` a `process.*`

<span id="nuxt-prefer-import-meta-bad"></span>

[Incorreto](./ecosystem.md#nuxt-prefer-import-meta-bad)

<span id="nuxt-prefer-import-meta-good"></span>

[Correto](./ecosystem.md#nuxt-prefer-import-meta-good)

[Exemplos do grupo](./ecosystem.md#nuxt-prefer-import-meta) · [Referência da regra](/rules/reference/nuxt-prefer-import-meta.md)

### `petite-vue/no-unsupported-directive`

Proibir diretivas que o petite-vue não aceita

<span id="petite-vue-no-unsupported-directive-bad"></span>

[Incorreto](./petite-vue.md#petite-vue-no-unsupported-directive-bad)

<span id="petite-vue-no-unsupported-directive-good"></span>

[Correto](./petite-vue.md#petite-vue-no-unsupported-directive-good)

[Exemplos do grupo](./petite-vue.md#petite-vue-no-unsupported-directive) · [Referência da regra](/rules/reference/petite-vue-no-unsupported-directive.md)

### `petite-vue/valid-v-effect`

Exigir uma expressão não vazia em v-effect

<span id="petite-vue-valid-v-effect-bad"></span>

[Incorreto](./petite-vue.md#petite-vue-valid-v-effect-bad)

<span id="petite-vue-valid-v-effect-good"></span>

[Correto](./petite-vue.md#petite-vue-valid-v-effect-good)

[Exemplos do grupo](./petite-vue.md#petite-vue-valid-v-effect) · [Referência da regra](/rules/reference/petite-vue-valid-v-effect.md)

### `petite-vue/valid-v-scope`

Exigir que v-scope vincule um objeto literal

<span id="petite-vue-valid-v-scope-bad"></span>

[Incorreto](./petite-vue.md#petite-vue-valid-v-scope-bad)

<span id="petite-vue-valid-v-scope-good"></span>

[Correto](./petite-vue.md#petite-vue-valid-v-scope-good)

[Exemplos do grupo](./petite-vue.md#petite-vue-valid-v-scope) · [Referência da regra](/rules/reference/petite-vue-valid-v-scope.md)

### `script/component-options-name-casing`

Exigir PascalCase na opção `name` do componente

<span id="script-component-options-name-casing-bad"></span>

[Incorreto](./type-and-script.md#script-component-options-name-casing-bad)

<span id="script-component-options-name-casing-good"></span>

[Correto](./type-and-script.md#script-component-options-name-casing-good)

[Exemplos do grupo](./type-and-script.md#script-component-options-name-casing) · [Referência da regra](/rules/reference/script-component-options-name-casing.md)

### `script/custom-event-name-casing`

Exigir camelCase nos nomes de eventos personalizados emitidos

<span id="script-custom-event-name-casing-bad"></span>

[Incorreto](./type-and-script.md#script-custom-event-name-casing-bad)

<span id="script-custom-event-name-casing-good"></span>

[Correto](./type-and-script.md#script-custom-event-name-casing-good)

[Exemplos do grupo](./type-and-script.md#script-custom-event-name-casing) · [Referência da regra](/rules/reference/script-custom-event-name-casing.md)

### `script/define-emits-declaration`

Exigir a forma de defineEmits&lt;{}&gt;() baseada em tipos em vez da forma em tempo de execução ou de array

<span id="script-define-emits-declaration-bad"></span>

[Incorreto](./type-and-script.md#script-define-emits-declaration-bad)

<span id="script-define-emits-declaration-good"></span>

[Correto](./type-and-script.md#script-define-emits-declaration-good)

[Exemplos do grupo](./type-and-script.md#script-define-emits-declaration) · [Referência da regra](/rules/reference/script-define-emits-declaration.md)

### `script/define-macros-order`

Exigir uma ordem consistente para as macros do compilador Vue em &lt;script setup&gt;

<span id="script-define-macros-order-bad"></span>

[Incorreto](./type-and-script.md#script-define-macros-order-bad)

<span id="script-define-macros-order-good"></span>

[Correto](./type-and-script.md#script-define-macros-order-good)

[Exemplos do grupo](./type-and-script.md#script-define-macros-order) · [Referência da regra](/rules/reference/script-define-macros-order.md)

### `script/define-props-declaration`

Exigir defineProps&lt;{ ... }&gt;() baseado em tipos em vez da forma com objeto em tempo de execução

<span id="script-define-props-declaration-bad"></span>

[Incorreto](./type-and-script.md#script-define-props-declaration-bad)

<span id="script-define-props-declaration-good"></span>

[Correto](./type-and-script.md#script-define-props-declaration-good)

[Exemplos do grupo](./type-and-script.md#script-define-props-declaration) · [Referência da regra](/rules/reference/script-define-props-declaration.md)

### `script/define-props-destructuring`

Exigir um estilo consistente de desestruturação de defineProps em &lt;script setup&gt;

<span id="script-define-props-destructuring-bad"></span>

[Incorreto](./type-and-script.md#script-define-props-destructuring-bad)

<span id="script-define-props-destructuring-good"></span>

[Correto](./type-and-script.md#script-define-props-destructuring-good)

[Exemplos do grupo](./type-and-script.md#script-define-props-destructuring) · [Referência da regra](/rules/reference/script-define-props-destructuring.md)

### `script/no-arrow-functions-in-watch`

Proibir funções de seta como manipuladores de watch na Options API

<span id="script-no-arrow-functions-in-watch-bad"></span>

[Incorreto](./type-and-script.md#script-no-arrow-functions-in-watch-bad)

<span id="script-no-arrow-functions-in-watch-good"></span>

[Correto](./type-and-script.md#script-no-arrow-functions-in-watch-good)

[Exemplos do grupo](./type-and-script.md#script-no-arrow-functions-in-watch) · [Referência da regra](/rules/reference/script-no-arrow-functions-in-watch.md)

### `script/no-async-in-computed`

Proibir funções assíncronas em propriedades computadas

<span id="script-no-async-in-computed-bad"></span>

[Incorreto](./type-and-script.md#script-no-async-in-computed-bad)

<span id="script-no-async-in-computed-good"></span>

[Correto](./type-and-script.md#script-no-async-in-computed-good)

[Exemplos do grupo](./type-and-script.md#script-no-async-in-computed) · [Referência da regra](/rules/reference/script-no-async-in-computed.md)

### `script/no-boolean-default`

Proibir um valor padrão em uma prop Boolean

<span id="script-no-boolean-default-bad"></span>

[Incorreto](./type-and-script.md#script-no-boolean-default-bad)

<span id="script-no-boolean-default-good"></span>

[Correto](./type-and-script.md#script-no-boolean-default-good)

[Exemplos do grupo](./type-and-script.md#script-no-boolean-default) · [Referência da regra](/rules/reference/script-no-boolean-default.md)

### `script/no-computed-properties-in-data`

Proibir o uso de propriedades computadas dentro da função data do componente

<span id="script-no-computed-properties-in-data-bad"></span>

[Incorreto](./type-and-script.md#script-no-computed-properties-in-data-bad)

<span id="script-no-computed-properties-in-data-good"></span>

[Correto](./type-and-script.md#script-no-computed-properties-in-data-good)

[Exemplos do grupo](./type-and-script.md#script-no-computed-properties-in-data) · [Referência da regra](/rules/reference/script-no-computed-properties-in-data.md)

### `script/no-deep-destructure-in-props`

Proibir desestruturação profundamente aninhada em defineProps

<span id="script-no-deep-destructure-in-props-bad"></span>

[Incorreto](./type-and-script.md#script-no-deep-destructure-in-props-bad)

<span id="script-no-deep-destructure-in-props-good"></span>

[Correto](./type-and-script.md#script-no-deep-destructure-in-props-good)

[Exemplos do grupo](./type-and-script.md#script-no-deep-destructure-in-props) · [Referência da regra](/rules/reference/script-no-deep-destructure-in-props.md)

### `script/no-deprecated-data-object-declaration`

Proibir um literal de objeto como opção data do componente (o Vue 3 exige uma função)

<span id="script-no-deprecated-data-object-declaration-bad"></span>

[Incorreto](./type-and-script.md#script-no-deprecated-data-object-declaration-bad)

<span id="script-no-deprecated-data-object-declaration-good"></span>

[Correto](./type-and-script.md#script-no-deprecated-data-object-declaration-good)

[Exemplos do grupo](./type-and-script.md#script-no-deprecated-data-object-declaration) · [Referência da regra](/rules/reference/script-no-deprecated-data-object-declaration.md)

### `script/no-deprecated-delete-set`

Proibir $set e $delete do Vue 2, removidos no Vue 3

<span id="script-no-deprecated-delete-set-bad"></span>

[Incorreto](./type-and-script.md#script-no-deprecated-delete-set-bad)

<span id="script-no-deprecated-delete-set-good"></span>

[Correto](./type-and-script.md#script-no-deprecated-delete-set-good)

[Exemplos do grupo](./type-and-script.md#script-no-deprecated-delete-set) · [Referência da regra](/rules/reference/script-no-deprecated-delete-set.md)

### `script/no-deprecated-destroyed-lifecycle`

Proibir os hooks de ciclo de vida obsoletos destroyed e beforeDestroy

<span id="script-no-deprecated-destroyed-lifecycle-bad"></span>

[Incorreto](./type-and-script.md#script-no-deprecated-destroyed-lifecycle-bad)

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

[Correto](./type-and-script.md#script-no-deprecated-destroyed-lifecycle-good)

[Exemplos do grupo](./type-and-script.md#script-no-deprecated-destroyed-lifecycle) · [Referência da regra](/rules/reference/script-no-deprecated-destroyed-lifecycle.md)

### `script/no-deprecated-dollar-listeners-api`

Proibir a propriedade de instância $listeners removida no Vue 3 (incorporada a $attrs)

<span id="script-no-deprecated-dollar-listeners-api-bad"></span>

[Incorreto](./type-and-script.md#script-no-deprecated-dollar-listeners-api-bad)

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

[Correto](./type-and-script.md#script-no-deprecated-dollar-listeners-api-good)

[Exemplos do grupo](./type-and-script.md#script-no-deprecated-dollar-listeners-api) · [Referência da regra](/rules/reference/script-no-deprecated-dollar-listeners-api.md)

### `script/no-deprecated-dollar-scopedslots-api`

Proibir a propriedade de instância $scopedSlots removida no Vue 3 (usar $slots)

<span id="script-no-deprecated-dollar-scopedslots-api-bad"></span>

[Incorreto](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api-bad)

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

[Correto](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api-good)

[Exemplos do grupo](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api) · [Referência da regra](/rules/reference/script-no-deprecated-dollar-scopedslots-api.md)

### `script/no-deprecated-events-api`

Proibir a API de eventos do Vue 2 removida ($on / $off / $once)

<span id="script-no-deprecated-events-api-bad"></span>

[Incorreto](./type-and-script.md#script-no-deprecated-events-api-bad)

<span id="script-no-deprecated-events-api-good"></span>

[Correto](./type-and-script.md#script-no-deprecated-events-api-good)

[Exemplos do grupo](./type-and-script.md#script-no-deprecated-events-api) · [Referência da regra](/rules/reference/script-no-deprecated-events-api.md)

### `script/no-deprecated-model-definition`

Proibir a opção model do Vue 2, removida no Vue 3

<span id="script-no-deprecated-model-definition-bad"></span>

[Incorreto](./type-and-script.md#script-no-deprecated-model-definition-bad)

<span id="script-no-deprecated-model-definition-good"></span>

[Correto](./type-and-script.md#script-no-deprecated-model-definition-good)

[Exemplos do grupo](./type-and-script.md#script-no-deprecated-model-definition) · [Referência da regra](/rules/reference/script-no-deprecated-model-definition.md)

### `script/no-deprecated-props-default-this`

Proibir `this` dentro de uma função de valor padrão ou validação de prop (removido no Vue 3)

<span id="script-no-deprecated-props-default-this-bad"></span>

[Incorreto](./type-and-script.md#script-no-deprecated-props-default-this-bad)

<span id="script-no-deprecated-props-default-this-good"></span>

[Correto](./type-and-script.md#script-no-deprecated-props-default-this-good)

[Exemplos do grupo](./type-and-script.md#script-no-deprecated-props-default-this) · [Referência da regra](/rules/reference/script-no-deprecated-props-default-this.md)

### `script/no-deprecated-vue-config-keycodes`

Proibir Vue.config.keyCodes, removido no Vue 3

<span id="script-no-deprecated-vue-config-keycodes-bad"></span>

[Incorreto](./type-and-script.md#script-no-deprecated-vue-config-keycodes-bad)

<span id="script-no-deprecated-vue-config-keycodes-good"></span>

[Correto](./type-and-script.md#script-no-deprecated-vue-config-keycodes-good)

[Exemplos do grupo](./type-and-script.md#script-no-deprecated-vue-config-keycodes) · [Referência da regra](/rules/reference/script-no-deprecated-vue-config-keycodes.md)

### `script/no-dupe-keys`

Proibir chaves duplicadas entre props/data/computed/methods/setup/inject da Options API

<span id="script-no-dupe-keys-bad"></span>

[Incorreto](./type-and-script.md#script-no-dupe-keys-bad)

<span id="script-no-dupe-keys-good"></span>

[Correto](./type-and-script.md#script-no-dupe-keys-good)

[Exemplos do grupo](./type-and-script.md#script-no-dupe-keys) · [Referência da regra](/rules/reference/script-no-dupe-keys.md)

### `script/no-duplicate-attr-inheritance`

Sinalizar um componente que aplica duas vezes seus atributos repassados

<span id="script-no-duplicate-attr-inheritance-bad"></span>

[Incorreto](./type-and-script.md#script-no-duplicate-attr-inheritance-bad)

<span id="script-no-duplicate-attr-inheritance-good"></span>

[Correto](./type-and-script.md#script-no-duplicate-attr-inheritance-good)

[Exemplos do grupo](./type-and-script.md#script-no-duplicate-attr-inheritance) · [Referência da regra](/rules/reference/script-no-duplicate-attr-inheritance.md)

### `script/no-export-in-script-setup`

Proibir instruções export dentro de &lt;script setup&gt;

<span id="script-no-export-in-script-setup-bad"></span>

[Incorreto](./type-and-script.md#script-no-export-in-script-setup-bad)

<span id="script-no-export-in-script-setup-good"></span>

[Correto](./type-and-script.md#script-no-export-in-script-setup-good)

[Exemplos do grupo](./type-and-script.md#script-no-export-in-script-setup) · [Referência da regra](/rules/reference/script-no-export-in-script-setup.md)

### `script/no-expose-after-await`

Proibir expose() e defineExpose() após um await

<span id="script-no-expose-after-await-bad"></span>

[Incorreto](./type-and-script.md#script-no-expose-after-await-bad)

<span id="script-no-expose-after-await-good"></span>

[Correto](./type-and-script.md#script-no-expose-after-await-good)

[Exemplos do grupo](./type-and-script.md#script-no-expose-after-await) · [Referência da regra](/rules/reference/script-no-expose-after-await.md)

### `script/no-get-current-instance`

Proibir getCurrentInstance() no modo Vapor (retorna null)

<span id="script-no-get-current-instance-bad"></span>

[Incorreto](./type-and-script.md#script-no-get-current-instance-bad)

<span id="script-no-get-current-instance-good"></span>

[Correto](./type-and-script.md#script-no-get-current-instance-good)

[Exemplos do grupo](./type-and-script.md#script-no-get-current-instance) · [Referência da regra](/rules/reference/script-no-get-current-instance.md)

### `script/no-import-compiler-macros`

Proibir a importação de macros do compilador Vue que são importadas automaticamente

<span id="script-no-import-compiler-macros-bad"></span>

[Incorreto](./type-and-script.md#script-no-import-compiler-macros-bad)

<span id="script-no-import-compiler-macros-good"></span>

[Correto](./type-and-script.md#script-no-import-compiler-macros-good)

[Exemplos do grupo](./type-and-script.md#script-no-import-compiler-macros) · [Referência da regra](/rules/reference/script-no-import-compiler-macros.md)

### `script/no-internal-imports`

Proibir importações de módulos internos do Vue

<span id="script-no-internal-imports-bad"></span>

[Incorreto](./type-and-script.md#script-no-internal-imports-bad)

<span id="script-no-internal-imports-good"></span>

[Correto](./type-and-script.md#script-no-internal-imports-good)

[Exemplos do grupo](./type-and-script.md#script-no-internal-imports) · [Referência da regra](/rules/reference/script-no-internal-imports.md)

### `script/no-lifecycle-after-await`

Proibir hooks de ciclo de vida registrados após um await em setup()

<span id="script-no-lifecycle-after-await-bad"></span>

[Incorreto](./type-and-script.md#script-no-lifecycle-after-await-bad)

<span id="script-no-lifecycle-after-await-good"></span>

[Correto](./type-and-script.md#script-no-lifecycle-after-await-good)

[Exemplos do grupo](./type-and-script.md#script-no-lifecycle-after-await) · [Referência da regra](/rules/reference/script-no-lifecycle-after-await.md)

### `script/no-multiple-slot-args`

Proibir passar mais de um argumento a uma chamada de função de slot com escopo

<span id="script-no-multiple-slot-args-bad"></span>

[Incorreto](./type-and-script.md#script-no-multiple-slot-args-bad)

<span id="script-no-multiple-slot-args-good"></span>

[Correto](./type-and-script.md#script-no-multiple-slot-args-good)

[Exemplos do grupo](./type-and-script.md#script-no-multiple-slot-args) · [Referência da regra](/rules/reference/script-no-multiple-slot-args.md)

### `script/no-next-tick`

Proibir o uso de nextTick() em componentes orientados a Vapor

<span id="script-no-next-tick-bad"></span>

[Incorreto](./type-and-script.md#script-no-next-tick-bad)

<span id="script-no-next-tick-good"></span>

[Correto](./type-and-script.md#script-no-next-tick-good)

[Exemplos do grupo](./type-and-script.md#script-no-next-tick) · [Referência da regra](/rules/reference/script-no-next-tick.md)

### `script/no-options-api`

Proibir padrões da Options API no modo Vapor

<span id="script-no-options-api-bad"></span>

[Incorreto](./type-and-script.md#script-no-options-api-bad)

<span id="script-no-options-api-good"></span>

[Correto](./type-and-script.md#script-no-options-api-good)

[Exemplos do grupo](./type-and-script.md#script-no-options-api) · [Referência da regra](/rules/reference/script-no-options-api.md)

### `script/no-potential-component-option-typo`

Sinalizar prováveis erros de digitação nos nomes de opções de componentes da Options API

<span id="script-no-potential-component-option-typo-bad"></span>

[Incorreto](./type-and-script.md#script-no-potential-component-option-typo-bad)

<span id="script-no-potential-component-option-typo-good"></span>

[Correto](./type-and-script.md#script-no-potential-component-option-typo-good)

[Exemplos do grupo](./type-and-script.md#script-no-potential-component-option-typo) · [Referência da regra](/rules/reference/script-no-potential-component-option-typo.md)

### `script/no-reactive-destructure`

Proibir a desestruturação de objetos reativos que causa perda de reatividade

<span id="script-no-reactive-destructure-bad"></span>

[Incorreto](./type-and-script.md#script-no-reactive-destructure-bad)

<span id="script-no-reactive-destructure-good"></span>

[Correto](./type-and-script.md#script-no-reactive-destructure-good)

[Exemplos do grupo](./type-and-script.md#script-no-reactive-destructure) · [Referência da regra](/rules/reference/script-no-reactive-destructure.md)

### `script/no-ref-as-operand`

Exigir que variáveis vinculadas a refs sejam acessadas por `.value` quando usadas como operandos

<span id="script-no-ref-as-operand-bad"></span>

[Incorreto](./type-and-script.md#script-no-ref-as-operand-bad)

<span id="script-no-ref-as-operand-good"></span>

[Correto](./type-and-script.md#script-no-ref-as-operand-good)

[Exemplos do grupo](./type-and-script.md#script-no-ref-as-operand) · [Referência da regra](/rules/reference/script-no-ref-as-operand.md)

### `script/no-required-prop-with-default`

Proibir uma prop que tenha required: true e também um valor padrão

<span id="script-no-required-prop-with-default-bad"></span>

[Incorreto](./type-and-script.md#script-no-required-prop-with-default-bad)

<span id="script-no-required-prop-with-default-good"></span>

[Correto](./type-and-script.md#script-no-required-prop-with-default-good)

[Exemplos do grupo](./type-and-script.md#script-no-required-prop-with-default) · [Referência da regra](/rules/reference/script-no-required-prop-with-default.md)

### `script/no-reserved-identifiers`

Proibir o uso de identificadores reservados pelo compilador Vue

<span id="script-no-reserved-identifiers-bad"></span>

[Incorreto](./type-and-script.md#script-no-reserved-identifiers-bad)

<span id="script-no-reserved-identifiers-good"></span>

[Correto](./type-and-script.md#script-no-reserved-identifiers-good)

[Exemplos do grupo](./type-and-script.md#script-no-reserved-identifiers) · [Referência da regra](/rules/reference/script-no-reserved-identifiers.md)

### `script/no-reserved-keys`

Proibir nomes reservados pelo Vue como chaves de props/data/computed/methods/setup/inject na Options API

<span id="script-no-reserved-keys-bad"></span>

[Incorreto](./type-and-script.md#script-no-reserved-keys-bad)

<span id="script-no-reserved-keys-good"></span>

[Correto](./type-and-script.md#script-no-reserved-keys-good)

[Exemplos do grupo](./type-and-script.md#script-no-reserved-keys) · [Referência da regra](/rules/reference/script-no-reserved-keys.md)

### `script/no-reserved-props`

Proibir nomes reservados na declaração de props de um componente

<span id="script-no-reserved-props-bad"></span>

[Incorreto](./type-and-script.md#script-no-reserved-props-bad)

<span id="script-no-reserved-props-good"></span>

[Correto](./type-and-script.md#script-no-reserved-props-good)

[Exemplos do grupo](./type-and-script.md#script-no-reserved-props) · [Referência da regra](/rules/reference/script-no-reserved-props.md)

### `script/no-restricted-globals`

Proibir referências a variáveis globais do ambiente de execução que devem passar por um encapsulamento tipado

<span id="script-no-restricted-globals-bad"></span>

[Incorreto](./type-and-script.md#script-no-restricted-globals-bad)

<span id="script-no-restricted-globals-good"></span>

[Correto](./type-and-script.md#script-no-restricted-globals-good)

[Exemplos do grupo](./type-and-script.md#script-no-restricted-globals) · [Referência da regra](/rules/reference/script-no-restricted-globals.md)

### `script/no-restricted-members`

Proibir acessos a membros object.property configurados pelo projeto

<span id="script-no-restricted-members-bad"></span>

[Incorreto](./type-and-script.md#script-no-restricted-members-bad)

<span id="script-no-restricted-members-good"></span>

[Correto](./type-and-script.md#script-no-restricted-members-good)

[Exemplos do grupo](./type-and-script.md#script-no-restricted-members) · [Referência da regra](/rules/reference/script-no-restricted-members.md)

### `script/no-side-effects-in-computed-properties`

Proibir efeitos colaterais em getters computados da Options API

<span id="script-no-side-effects-in-computed-properties-bad"></span>

[Incorreto](./type-and-script.md#script-no-side-effects-in-computed-properties-bad)

<span id="script-no-side-effects-in-computed-properties-good"></span>

[Correto](./type-and-script.md#script-no-side-effects-in-computed-properties-good)

[Exemplos do grupo](./type-and-script.md#script-no-side-effects-in-computed-properties) · [Referência da regra](/rules/reference/script-no-side-effects-in-computed-properties.md)

### `script/no-top-level-ref-in-script`

Proibir ref/reactive no nível superior para evitar contaminação de estado entre requisições

<span id="script-no-top-level-ref-in-script-bad"></span>

[Incorreto](./type-and-script.md#script-no-top-level-ref-in-script-bad)

<span id="script-no-top-level-ref-in-script-good"></span>

[Correto](./type-and-script.md#script-no-top-level-ref-in-script-good)

[Exemplos do grupo](./type-and-script.md#script-no-top-level-ref-in-script) · [Referência da regra](/rules/reference/script-no-top-level-ref-in-script.md)

### `script/no-unstable-nested-components`

Proibir definições de componentes dentro de funções de setup ou renderização

<span id="script-no-unstable-nested-components-bad"></span>

[Incorreto](./type-and-script.md#script-no-unstable-nested-components-bad)

<span id="script-no-unstable-nested-components-good"></span>

[Correto](./type-and-script.md#script-no-unstable-nested-components-good)

[Exemplos do grupo](./type-and-script.md#script-no-unstable-nested-components) · [Referência da regra](/rules/reference/script-no-unstable-nested-components.md)

### `script/no-unused-emit-declarations`

Sinalizar eventos declarados que nunca são emitidos

<span id="script-no-unused-emit-declarations-bad"></span>

[Incorreto](./type-and-script.md#script-no-unused-emit-declarations-bad)

<span id="script-no-unused-emit-declarations-good"></span>

[Correto](./type-and-script.md#script-no-unused-emit-declarations-good)

[Exemplos do grupo](./type-and-script.md#script-no-unused-emit-declarations) · [Referência da regra](/rules/reference/script-no-unused-emit-declarations.md)

### `script/no-use-computed-property-like-method`

Proibir chamar uma propriedade computada da Options API como um método

<span id="script-no-use-computed-property-like-method-bad"></span>

[Incorreto](./type-and-script.md#script-no-use-computed-property-like-method-bad)

<span id="script-no-use-computed-property-like-method-good"></span>

[Correto](./type-and-script.md#script-no-use-computed-property-like-method-good)

[Exemplos do grupo](./type-and-script.md#script-no-use-computed-property-like-method) · [Referência da regra](/rules/reference/script-no-use-computed-property-like-method.md)

### `script/no-watch-after-await`

Proibir watchers registrados após um await em setup() sem guardar a função de parada

<span id="script-no-watch-after-await-bad"></span>

[Incorreto](./type-and-script.md#script-no-watch-after-await-bad)

<span id="script-no-watch-after-await-good"></span>

[Correto](./type-and-script.md#script-no-watch-after-await-good)

[Exemplos do grupo](./type-and-script.md#script-no-watch-after-await) · [Referência da regra](/rules/reference/script-no-watch-after-await.md)

### `script/no-with-defaults`

Desencorajar withDefaults em favor de valores padrão na desestruturação (Vue 3.5+)

<span id="script-no-with-defaults-bad"></span>

[Incorreto](./type-and-script.md#script-no-with-defaults-bad)

<span id="script-no-with-defaults-good"></span>

[Correto](./type-and-script.md#script-no-with-defaults-good)

[Exemplos do grupo](./type-and-script.md#script-no-with-defaults) · [Referência da regra](/rules/reference/script-no-with-defaults.md)

### `script/one-component-per-file`

Proibir mais de uma definição de componente em um arquivo

<span id="script-one-component-per-file-bad"></span>

[Incorreto](./type-and-script.md#script-one-component-per-file-bad)

<span id="script-one-component-per-file-good"></span>

[Correto](./type-and-script.md#script-one-component-per-file-good)

[Exemplos do grupo](./type-and-script.md#script-one-component-per-file) · [Referência da regra](/rules/reference/script-one-component-per-file.md)

### `script/order-in-components`

Exigir a ordem padrão das propriedades nas opções do componente

<span id="script-order-in-components-bad"></span>

[Incorreto](./type-and-script.md#script-order-in-components-bad)

<span id="script-order-in-components-good"></span>

[Correto](./type-and-script.md#script-order-in-components-good)

[Exemplos do grupo](./type-and-script.md#script-order-in-components) · [Referência da regra](/rules/reference/script-order-in-components.md)

### `script/prefer-computed`

Preferir computed() para estado reativo derivado

<span id="script-prefer-computed-bad"></span>

[Incorreto](./type-and-script.md#script-prefer-computed-bad)

<span id="script-prefer-computed-good"></span>

[Correto](./type-and-script.md#script-prefer-computed-good)

[Exemplos do grupo](./type-and-script.md#script-prefer-computed) · [Referência da regra](/rules/reference/script-prefer-computed.md)

### `script/prefer-define-options`

Preferir defineOptions() a um &lt;script&gt; comum que só define name/inheritAttrs

<span id="script-prefer-define-options-bad"></span>

[Incorreto](./type-and-script.md#script-prefer-define-options-bad)

<span id="script-prefer-define-options-good"></span>

[Correto](./type-and-script.md#script-prefer-define-options-good)

[Exemplos do grupo](./type-and-script.md#script-prefer-define-options) · [Referência da regra](/rules/reference/script-prefer-define-options.md)

### `script/prefer-import-from-vue`

Preferir importar de 'vue' em vez de pacotes internos

<span id="script-prefer-import-from-vue-bad"></span>

[Incorreto](./type-and-script.md#script-prefer-import-from-vue-bad)

<span id="script-prefer-import-from-vue-good"></span>

[Correto](./type-and-script.md#script-prefer-import-from-vue-good)

[Exemplos do grupo](./type-and-script.md#script-prefer-import-from-vue) · [Referência da regra](/rules/reference/script-prefer-import-from-vue.md)

### `script/prefer-ref-over-reactive`

Recomendar o uso de ref() em vez de reactive() para gerenciar estado

<span id="script-prefer-ref-over-reactive-bad"></span>

[Incorreto](./type-and-script.md#script-prefer-ref-over-reactive-bad)

<span id="script-prefer-ref-over-reactive-good"></span>

[Correto](./type-and-script.md#script-prefer-ref-over-reactive-good)

[Exemplos do grupo](./type-and-script.md#script-prefer-ref-over-reactive) · [Referência da regra](/rules/reference/script-prefer-ref-over-reactive.md)

### `script/prefer-use-attrs`

Recomendar o uso de useAttrs() em vez de context.attrs

<span id="script-prefer-use-attrs-bad"></span>

[Incorreto](./type-and-script.md#script-prefer-use-attrs-bad)

<span id="script-prefer-use-attrs-good"></span>

[Correto](./type-and-script.md#script-prefer-use-attrs-good)

[Exemplos do grupo](./type-and-script.md#script-prefer-use-attrs) · [Referência da regra](/rules/reference/script-prefer-use-attrs.md)

### `script/prefer-use-id`

Recomendar o uso de useId() para gerar IDs únicos (Vue 3.5+)

<span id="script-prefer-use-id-bad"></span>

[Incorreto](./type-and-script.md#script-prefer-use-id-bad)

<span id="script-prefer-use-id-good"></span>

[Correto](./type-and-script.md#script-prefer-use-id-good)

[Exemplos do grupo](./type-and-script.md#script-prefer-use-id) · [Referência da regra](/rules/reference/script-prefer-use-id.md)

### `script/prefer-use-slots`

Recomendar o uso de useSlots() em vez de context.slots

<span id="script-prefer-use-slots-bad"></span>

[Incorreto](./type-and-script.md#script-prefer-use-slots-bad)

<span id="script-prefer-use-slots-good"></span>

[Correto](./type-and-script.md#script-prefer-use-slots-good)

[Exemplos do grupo](./type-and-script.md#script-prefer-use-slots) · [Referência da regra](/rules/reference/script-prefer-use-slots.md)

### `script/prefer-use-template-ref`

Recomendar useTemplateRef em vez de ref para referências de template (Vue 3.5+)

<span id="script-prefer-use-template-ref-bad"></span>

[Incorreto](./type-and-script.md#script-prefer-use-template-ref-bad)

<span id="script-prefer-use-template-ref-good"></span>

[Correto](./type-and-script.md#script-prefer-use-template-ref-good)

[Exemplos do grupo](./type-and-script.md#script-prefer-use-template-ref) · [Referência da regra](/rules/reference/script-prefer-use-template-ref.md)

### `script/require-default-prop`

Exigir um valor padrão para toda prop opcional que não seja Boolean

<span id="script-require-default-prop-bad"></span>

[Incorreto](./type-and-script.md#script-require-default-prop-bad)

<span id="script-require-default-prop-good"></span>

[Correto](./type-and-script.md#script-require-default-prop-good)

[Exemplos do grupo](./type-and-script.md#script-require-default-prop) · [Referência da regra](/rules/reference/script-require-default-prop.md)

### `script/require-explicit-emits`

Exigir que os eventos emitidos sejam declarados em defineEmits ou na opção emits

<span id="script-require-explicit-emits-bad"></span>

[Incorreto](./type-and-script.md#script-require-explicit-emits-bad)

<span id="script-require-explicit-emits-good"></span>

[Correto](./type-and-script.md#script-require-explicit-emits-good)

[Exemplos do grupo](./type-and-script.md#script-require-explicit-emits) · [Referência da regra](/rules/reference/script-require-explicit-emits.md)

### `script/require-explicit-slots`

Exigir que os slots consumidos por useSlots() sejam tipados explicitamente com defineSlots&lt;...&gt;()

<span id="script-require-explicit-slots-bad"></span>

[Incorreto](./type-and-script.md#script-require-explicit-slots-bad)

<span id="script-require-explicit-slots-good"></span>

[Correto](./type-and-script.md#script-require-explicit-slots-good)

[Exemplos do grupo](./type-and-script.md#script-require-explicit-slots) · [Referência da regra](/rules/reference/script-require-explicit-slots.md)

### `script/require-function-return-type`

Exigir anotações de tipo de retorno nas funções

<span id="script-require-function-return-type-bad"></span>

[Incorreto](./type-and-script.md#script-require-function-return-type-bad)

<span id="script-require-function-return-type-good"></span>

[Correto](./type-and-script.md#script-require-function-return-type-good)

[Exemplos do grupo](./type-and-script.md#script-require-function-return-type) · [Referência da regra](/rules/reference/script-require-function-return-type.md)

### `script/require-prop-type-constructor`

Exigir que os valores de `type` de props sejam construtores em vez de literais de string

<span id="script-require-prop-type-constructor-bad"></span>

[Incorreto](./type-and-script.md#script-require-prop-type-constructor-bad)

<span id="script-require-prop-type-constructor-good"></span>

[Correto](./type-and-script.md#script-require-prop-type-constructor-good)

[Exemplos do grupo](./type-and-script.md#script-require-prop-type-constructor) · [Referência da regra](/rules/reference/script-require-prop-type-constructor.md)

### `script/require-prop-types`

Exigir que toda prop declare um tipo

<span id="script-require-prop-types-bad"></span>

[Incorreto](./type-and-script.md#script-require-prop-types-bad)

<span id="script-require-prop-types-good"></span>

[Correto](./type-and-script.md#script-require-prop-types-good)

[Exemplos do grupo](./type-and-script.md#script-require-prop-types) · [Referência da regra](/rules/reference/script-require-prop-types.md)

### `script/require-render-return`

Exigir que a função render do componente devolva um valor

<span id="script-require-render-return-bad"></span>

[Incorreto](./type-and-script.md#script-require-render-return-bad)

<span id="script-require-render-return-good"></span>

[Correto](./type-and-script.md#script-require-render-return-good)

[Exemplos do grupo](./type-and-script.md#script-require-render-return) · [Referência da regra](/rules/reference/script-require-render-return.md)

### `script/require-slots-as-functions`

Exigir que as propriedades de $slots sejam chamadas como funções

<span id="script-require-slots-as-functions-bad"></span>

[Incorreto](./type-and-script.md#script-require-slots-as-functions-bad)

<span id="script-require-slots-as-functions-good"></span>

[Correto](./type-and-script.md#script-require-slots-as-functions-good)

[Exemplos do grupo](./type-and-script.md#script-require-slots-as-functions) · [Referência da regra](/rules/reference/script-require-slots-as-functions.md)

### `script/require-symbol-provide`

Recomendar o uso de Symbol como chave de injeção para provide/inject

<span id="script-require-symbol-provide-bad"></span>

[Incorreto](./type-and-script.md#script-require-symbol-provide-bad)

<span id="script-require-symbol-provide-good"></span>

[Correto](./type-and-script.md#script-require-symbol-provide-good)

[Exemplos do grupo](./type-and-script.md#script-require-symbol-provide) · [Referência da regra](/rules/reference/script-require-symbol-provide.md)

### `script/require-typed-object-prop`

Exigir um tipo explícito em uma prop cujo tipo de tempo de execução seja `Object` ou `Array`

<span id="script-require-typed-object-prop-bad"></span>

[Incorreto](./type-and-script.md#script-require-typed-object-prop-bad)

<span id="script-require-typed-object-prop-good"></span>

[Correto](./type-and-script.md#script-require-typed-object-prop-good)

[Exemplos do grupo](./type-and-script.md#script-require-typed-object-prop) · [Referência da regra](/rules/reference/script-require-typed-object-prop.md)

### `script/require-typed-ref`

Exigir um argumento de tipo explícito em um ref() inicializado sem valor, com null ou com undefined

<span id="script-require-typed-ref-bad"></span>

[Incorreto](./type-and-script.md#script-require-typed-ref-bad)

<span id="script-require-typed-ref-good"></span>

[Correto](./type-and-script.md#script-require-typed-ref-good)

[Exemplos do grupo](./type-and-script.md#script-require-typed-ref) · [Referência da regra](/rules/reference/script-require-typed-ref.md)

### `script/require-valid-default-prop`

Exigir que o valor padrão de uma prop seja válido para seu tipo declarado

<span id="script-require-valid-default-prop-bad"></span>

[Incorreto](./type-and-script.md#script-require-valid-default-prop-bad)

<span id="script-require-valid-default-prop-good"></span>

[Correto](./type-and-script.md#script-require-valid-default-prop-good)

[Exemplos do grupo](./type-and-script.md#script-require-valid-default-prop) · [Referência da regra](/rules/reference/script-require-valid-default-prop.md)

### `script/return-in-computed-property`

Exigir um valor de retorno em todo getter computado

<span id="script-return-in-computed-property-bad"></span>

[Incorreto](./type-and-script.md#script-return-in-computed-property-bad)

<span id="script-return-in-computed-property-good"></span>

[Correto](./type-and-script.md#script-return-in-computed-property-good)

[Exemplos do grupo](./type-and-script.md#script-return-in-computed-property) · [Referência da regra](/rules/reference/script-return-in-computed-property.md)

### `script/return-in-emits-validator`

Exigir um valor de retorno em todo validador de emits da Options API

<span id="script-return-in-emits-validator-bad"></span>

[Incorreto](./type-and-script.md#script-return-in-emits-validator-bad)

<span id="script-return-in-emits-validator-good"></span>

[Correto](./type-and-script.md#script-return-in-emits-validator-good)

[Exemplos do grupo](./type-and-script.md#script-return-in-emits-validator) · [Referência da regra](/rules/reference/script-return-in-emits-validator.md)

### `script/valid-define-emits`

Exigir uso válido de defineEmits() (sem argumentos de tipo e de tempo de execução juntos, sem referências locais, uma única chamada)

<span id="script-valid-define-emits-bad"></span>

[Incorreto](./type-and-script.md#script-valid-define-emits-bad)

<span id="script-valid-define-emits-good"></span>

[Correto](./type-and-script.md#script-valid-define-emits-good)

[Exemplos do grupo](./type-and-script.md#script-valid-define-emits) · [Referência da regra](/rules/reference/script-valid-define-emits.md)

### `script/valid-define-options`

Exigir uso válido de defineOptions() (um único argumento de objeto, sem props/emits/expose/slots)

<span id="script-valid-define-options-bad"></span>

[Incorreto](./type-and-script.md#script-valid-define-options-bad)

<span id="script-valid-define-options-good"></span>

[Correto](./type-and-script.md#script-valid-define-options-good)

[Exemplos do grupo](./type-and-script.md#script-valid-define-options) · [Referência da regra](/rules/reference/script-valid-define-options.md)

### `script/valid-define-props`

Exigir uso válido de defineProps() (uma única chamada, sem argumentos de tipo e de tempo de execução juntos, sem referências locais)

<span id="script-valid-define-props-bad"></span>

[Incorreto](./type-and-script.md#script-valid-define-props-bad)

<span id="script-valid-define-props-good"></span>

[Correto](./type-and-script.md#script-valid-define-props-good)

[Exemplos do grupo](./type-and-script.md#script-valid-define-props) · [Referência da regra](/rules/reference/script-valid-define-props.md)

### `script/valid-next-tick`

Exigir que o resultado de uma chamada de nextTick() seja aguardado, encadeado ou receba uma função de retorno

<span id="script-valid-next-tick-bad"></span>

[Incorreto](./type-and-script.md#script-valid-next-tick-bad)

<span id="script-valid-next-tick-good"></span>

[Correto](./type-and-script.md#script-valid-next-tick-good)

[Exemplos do grupo](./type-and-script.md#script-valid-next-tick) · [Referência da regra](/rules/reference/script-valid-next-tick.md)

### `ssr/no-browser-globals-in-ssr`

Proibir variáveis globais exclusivas do navegador no contexto de SSR

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

[Incorreto](./ssr.md#ssr-no-browser-globals-in-ssr-bad)

<span id="ssr-no-browser-globals-in-ssr-good"></span>

[Correto](./ssr.md#ssr-no-browser-globals-in-ssr-good)

[Exemplos do grupo](./ssr.md#ssr-no-browser-globals-in-ssr) · [Referência da regra](/rules/reference/ssr-no-browser-globals-in-ssr.md)

### `ssr/no-hydration-mismatch`

Proibir valores não determinísticos que causam divergências na hidratação

<span id="ssr-no-hydration-mismatch-bad"></span>

[Incorreto](./ssr.md#ssr-no-hydration-mismatch-bad)

<span id="ssr-no-hydration-mismatch-good"></span>

[Correto](./ssr.md#ssr-no-hydration-mismatch-good)

[Exemplos do grupo](./ssr.md#ssr-no-hydration-mismatch) · [Referência da regra](/rules/reference/ssr-no-hydration-mismatch.md)

### `type/no-floating-promises`

Proibir Promises soltas (não tratadas)

<span id="type-no-floating-promises-bad"></span>

[Incorreto](./type-and-script.md#type-no-floating-promises-bad)

<span id="type-no-floating-promises-good"></span>

[Correto](./type-and-script.md#type-no-floating-promises-good)

[Exemplos do grupo](./type-and-script.md#type-no-floating-promises) · [Referência da regra](/rules/reference/type-no-floating-promises.md)

### `type/no-reactivity-loss`

Proibir cópias estáticas simples de valores reativos em atribuições e chamadas

<span id="type-no-reactivity-loss-bad"></span>

[Incorreto](./type-and-script.md#type-no-reactivity-loss-bad)

<span id="type-no-reactivity-loss-good"></span>

[Correto](./type-and-script.md#type-no-reactivity-loss-good)

[Exemplos do grupo](./type-and-script.md#type-no-reactivity-loss) · [Referência da regra](/rules/reference/type-no-reactivity-loss.md)

### `type/no-unsafe-template-binding`

Proibir vinculações de template que resultam em tipos inseguros

<span id="type-no-unsafe-template-binding-bad"></span>

[Incorreto](./type-and-script.md#type-no-unsafe-template-binding-bad)

<span id="type-no-unsafe-template-binding-good"></span>

[Correto](./type-and-script.md#type-no-unsafe-template-binding-good)

[Exemplos do grupo](./type-and-script.md#type-no-unsafe-template-binding) · [Referência da regra](/rules/reference/type-no-unsafe-template-binding.md)

### `type/require-typed-emits`

Exigir uma definição de tipo para defineEmits

<span id="type-require-typed-emits-bad"></span>

[Incorreto](./type-and-script.md#type-require-typed-emits-bad)

<span id="type-require-typed-emits-good"></span>

[Correto](./type-and-script.md#type-require-typed-emits-good)

[Exemplos do grupo](./type-and-script.md#type-require-typed-emits) · [Referência da regra](/rules/reference/type-require-typed-emits.md)

### `type/require-typed-props`

Exigir uma definição de tipo para defineProps

<span id="type-require-typed-props-bad"></span>

[Incorreto](./type-and-script.md#type-require-typed-props-bad)

<span id="type-require-typed-props-good"></span>

[Correto](./type-and-script.md#type-require-typed-props-good)

[Exemplos do grupo](./type-and-script.md#type-require-typed-props) · [Referência da regra](/rules/reference/type-require-typed-props.md)

### `type/strict-boolean-expressions`

Exigir expressões booleanas seguras nas condições de script e template

<span id="type-strict-boolean-expressions-bad"></span>

[Incorreto](./type-and-script.md#type-strict-boolean-expressions-bad)

<span id="type-strict-boolean-expressions-good"></span>

[Correto](./type-and-script.md#type-strict-boolean-expressions-good)

[Exemplos do grupo](./type-and-script.md#type-strict-boolean-expressions) · [Referência da regra](/rules/reference/type-strict-boolean-expressions.md)

### `vapor/no-inline-template`

Proibir o atributo obsoleto inline-template

<span id="vapor-no-inline-template-bad"></span>

[Incorreto](./vapor.md#vapor-no-inline-template-bad)

<span id="vapor-no-inline-template-good"></span>

[Correto](./vapor.md#vapor-no-inline-template-good)

[Exemplos do grupo](./vapor.md#vapor-no-inline-template) · [Referência da regra](/rules/reference/vapor-no-inline-template.md)

### `vapor/no-vue-lifecycle-events`

Proibir eventos de ciclo de vida @vue:xxx por elemento (não suportados em Vapor)

<span id="vapor-no-vue-lifecycle-events-bad"></span>

[Incorreto](./vapor.md#vapor-no-vue-lifecycle-events-bad)

<span id="vapor-no-vue-lifecycle-events-good"></span>

[Correto](./vapor.md#vapor-no-vue-lifecycle-events-good)

[Exemplos do grupo](./vapor.md#vapor-no-vue-lifecycle-events) · [Referência da regra](/rules/reference/vapor-no-vue-lifecycle-events.md)

### `vapor/prefer-static-class`

Preferir class estática a uma vinculação dinâmica de class para literais de string

<span id="vapor-prefer-static-class-bad"></span>

[Incorreto](./vapor.md#vapor-prefer-static-class-bad)

<span id="vapor-prefer-static-class-good"></span>

[Correto](./vapor.md#vapor-prefer-static-class-good)

[Exemplos do grupo](./vapor.md#vapor-prefer-static-class) · [Referência da regra](/rules/reference/vapor-prefer-static-class.md)

### `vapor/require-vapor-attribute`

Sugerir a adição do atributo vapor a script setup

<span id="vapor-require-vapor-attribute-bad"></span>

[Incorreto](./vapor.md#vapor-require-vapor-attribute-bad)

<span id="vapor-require-vapor-attribute-good"></span>

[Correto](./vapor.md#vapor-require-vapor-attribute-good)

[Exemplos do grupo](./vapor.md#vapor-require-vapor-attribute) · [Referência da regra](/rules/reference/vapor-require-vapor-attribute.md)

### `vize:croquis/cf/array-mutation`

Um array é alterado por índice, o que um array reativo não rastreia.

<span id="vize-croquis-cf-array-mutation-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-array-mutation-bad)

<span id="vize-croquis-cf-array-mutation-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-array-mutation-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-array-mutation) · [Referência da regra](/rules/project/vize-croquis-cf-array-mutation.md)

### `vize:croquis/cf/async-boundary`

O estado reativo atravessa um limite assíncrono e pode ser observado desatualizado.

<span id="vize-croquis-cf-async-boundary-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-async-boundary-bad)

<span id="vize-croquis-cf-async-boundary-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-async-boundary-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-async-boundary) · [Referência da regra](/rules/project/vize-croquis-cf-async-boundary.md)

### `vize:croquis/cf/async-no-suspense`

Um componente assíncrono é renderizado sem um limite Suspense.

<span id="vize-croquis-cf-async-no-suspense-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-async-no-suspense-bad)

<span id="vize-croquis-cf-async-no-suspense-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-async-no-suspense-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-async-no-suspense) · [Referência da regra](/rules/project/vize-croquis-cf-async-no-suspense.md)

### `vize:croquis/cf/browser-api-ssr`

Uma API exclusiva do navegador é usada onde o componente pode ser renderizado no servidor.

<span id="vize-croquis-cf-browser-api-ssr-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-browser-api-ssr-bad)

<span id="vize-croquis-cf-browser-api-ssr-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-browser-api-ssr-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-browser-api-ssr) · [Referência da regra](/rules/project/vize-croquis-cf-browser-api-ssr.md)

### `vize:croquis/cf/circular-dep`

Os componentes importam uns aos outros em um ciclo.

<span id="vize-croquis-cf-circular-dep-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-circular-dep-bad)

<span id="vize-croquis-cf-circular-dep-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-circular-dep-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-circular-dep) · [Referência da regra](/rules/project/vize-croquis-cf-circular-dep.md)

### `vize:croquis/cf/circular-reactive-dependency`

Os cálculos reativos dependem uns dos outros em um ciclo.

<span id="vize-croquis-cf-circular-reactive-dependency-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-circular-reactive-dependency-bad)

<span id="vize-croquis-cf-circular-reactive-dependency-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-circular-reactive-dependency-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-circular-reactive-dependency) · [Referência da regra](/rules/project/vize-croquis-cf-circular-reactive-dependency.md)

### `vize:croquis/cf/closure-captures-reactive`

Um fechamento captura um valor reativo e não verá atualizações posteriores.

<span id="vize-croquis-cf-closure-captures-reactive-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-closure-captures-reactive-bad)

<span id="vize-croquis-cf-closure-captures-reactive-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-closure-captures-reactive-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-closure-captures-reactive) · [Referência da regra](/rules/project/vize-croquis-cf-closure-captures-reactive.md)

### `vize:croquis/cf/composable-outside-setup`

Uma função de composição é chamada fora de `setup`.

<span id="vize-croquis-cf-composable-outside-setup-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-composable-outside-setup-bad)

<span id="vize-croquis-cf-composable-outside-setup-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-composable-outside-setup-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-composable-outside-setup) · [Referência da regra](/rules/project/vize-croquis-cf-composable-outside-setup.md)

### `vize:croquis/cf/computed-side-effects`

Uma função de leitura computada escreve no estado ou produz outro efeito colateral.

<span id="vize-croquis-cf-computed-side-effects-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-computed-side-effects-bad)

<span id="vize-croquis-cf-computed-side-effects-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-computed-side-effects-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-computed-side-effects) · [Referência da regra](/rules/project/vize-croquis-cf-computed-side-effects.md)

### `vize:croquis/cf/deep-import`

Uma cadeia de importações é mais profunda do que o projeto permite.

<span id="vize-croquis-cf-deep-import-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-deep-import-bad)

<span id="vize-croquis-cf-deep-import-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-deep-import-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-deep-import) · [Referência da regra](/rules/project/vize-croquis-cf-deep-import.md)

### `vize:croquis/cf/destructuring-breaks-reactivity`

Desestruturar um objeto reativo copia os campos e perde o rastreamento.

<span id="vize-croquis-cf-destructuring-breaks-reactivity-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity-bad)

<span id="vize-croquis-cf-destructuring-breaks-reactivity-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity) · [Referência da regra](/rules/project/vize-croquis-cf-destructuring-breaks-reactivity.md)

### `vize:croquis/cf/di-outside-setup`

`provide` ou `inject` é chamado fora de `setup`.

<span id="vize-croquis-cf-di-outside-setup-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-di-outside-setup-bad)

<span id="vize-croquis-cf-di-outside-setup-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-di-outside-setup-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-di-outside-setup) · [Referência da regra](/rules/project/vize-croquis-cf-di-outside-setup.md)

### `vize:croquis/cf/dom-access-without-next-tick`

O DOM é lido antes de o Vue aplicar a atualização.

<span id="vize-croquis-cf-dom-access-without-next-tick-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick-bad)

<span id="vize-croquis-cf-dom-access-without-next-tick-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick) · [Referência da regra](/rules/project/vize-croquis-cf-dom-access-without-next-tick.md)

### `vize:croquis/cf/duplicate-id`

O mesmo id de elemento é usado em mais de um componente.

<span id="vize-croquis-cf-duplicate-id-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-duplicate-id-bad)

<span id="vize-croquis-cf-duplicate-id-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-duplicate-id-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-duplicate-id) · [Referência da regra](/rules/project/vize-croquis-cf-duplicate-id.md)

### `vize:croquis/cf/event-listener-leak`

Um ouvinte de eventos é registrado e nunca removido.

<span id="vize-croquis-cf-event-listener-leak-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-event-listener-leak-bad)

<span id="vize-croquis-cf-event-listener-leak-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-event-listener-leak-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-event-listener-leak) · [Referência da regra](/rules/project/vize-croquis-cf-event-listener-leak.md)

### `vize:croquis/cf/event-modifier`

Um ouvinte de eventos usa um modificador que o evento emitido não suporta.

<span id="vize-croquis-cf-event-modifier-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-event-modifier-bad)

<span id="vize-croquis-cf-event-modifier-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-event-modifier-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-event-modifier) · [Referência da regra](/rules/project/vize-croquis-cf-event-modifier.md)

### `vize:croquis/cf/hydration-risk`

Este código de diagnóstico agrupa várias ocorrências de reatividade, incluindo uma prop copiada para uma ref. Ele não implica que toda expressão Date.now() seja detectada pela passagem de análise entre arquivos.

<span id="vize-croquis-cf-hydration-risk-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-hydration-risk-bad)

<span id="vize-croquis-cf-hydration-risk-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-hydration-risk-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-hydration-risk) · [Referência da regra](/rules/project/vize-croquis-cf-hydration-risk.md)

### `vize:croquis/cf/inherit-attrs-unused`

`inheritAttrs: false` está definido e o componente nunca lê os atributos.

<span id="vize-croquis-cf-inherit-attrs-unused-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-inherit-attrs-unused-bad)

<span id="vize-croquis-cf-inherit-attrs-unused-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-inherit-attrs-unused-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-inherit-attrs-unused) · [Referência da regra](/rules/project/vize-croquis-cf-inherit-attrs-unused.md)

### `vize:croquis/cf/inject-without-symbol`

`inject` usa uma chave comum em vez de um símbolo `InjectionKey`.

<span id="vize-croquis-cf-inject-without-symbol-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-inject-without-symbol-bad)

<span id="vize-croquis-cf-inject-without-symbol-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-inject-without-symbol-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-inject-without-symbol) · [Referência da regra](/rules/project/vize-croquis-cf-inject-without-symbol.md)

### `vize:croquis/cf/injected-async-mutation-race`

Um valor injetado é alterado por uma tarefa assíncrona sujeita a uma condição de corrida.

<span id="vize-croquis-cf-injected-async-mutation-race-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-injected-async-mutation-race-bad)

<span id="vize-croquis-cf-injected-async-mutation-race-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-injected-async-mutation-race-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-injected-async-mutation-race) · [Referência da regra](/rules/project/vize-croquis-cf-injected-async-mutation-race.md)

### `vize:croquis/cf/lifecycle-outside-setup`

Um gancho de ciclo de vida é registrado fora de `setup`.

<span id="vize-croquis-cf-lifecycle-outside-setup-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup-bad)

<span id="vize-croquis-cf-lifecycle-outside-setup-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup) · [Referência da regra](/rules/project/vize-croquis-cf-lifecycle-outside-setup.md)

### `vize:croquis/cf/lifecycle-without-cleanup`

Um gancho de ciclo de vida inicia um trabalho e nunca faz sua limpeza.

<span id="vize-croquis-cf-lifecycle-without-cleanup-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup-bad)

<span id="vize-croquis-cf-lifecycle-without-cleanup-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup) · [Referência da regra](/rules/project/vize-croquis-cf-lifecycle-without-cleanup.md)

### `vize:croquis/cf/missing-required-prop`

Uma prop obrigatória não é passada.

<span id="vize-croquis-cf-missing-required-prop-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-missing-required-prop-bad)

<span id="vize-croquis-cf-missing-required-prop-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-missing-required-prop-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-missing-required-prop) · [Referência da regra](/rules/project/vize-croquis-cf-missing-required-prop.md)

### `vize:croquis/cf/missing-suspense`

Uma dependência assíncrona é usada fora de um limite Suspense.

<span id="vize-croquis-cf-missing-suspense-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-missing-suspense-bad)

<span id="vize-croquis-cf-missing-suspense-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-missing-suspense-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-missing-suspense) · [Referência da regra](/rules/project/vize-croquis-cf-missing-suspense.md)

### `vize:croquis/cf/module-scope-reactive`

O estado reativo é criado no escopo do módulo e compartilhado por todos os chamadores.

<span id="vize-croquis-cf-module-scope-reactive-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-module-scope-reactive-bad)

<span id="vize-croquis-cf-module-scope-reactive-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-module-scope-reactive-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-module-scope-reactive) · [Referência da regra](/rules/project/vize-croquis-cf-module-scope-reactive.md)

### `vize:croquis/cf/multi-root-attrs`

Um componente com múltiplas raízes recebe atributos e não tem onde colocá-los.

<span id="vize-croquis-cf-multi-root-attrs-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-multi-root-attrs-bad)

<span id="vize-croquis-cf-multi-root-attrs-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-multi-root-attrs-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-multi-root-attrs) · [Referência da regra](/rules/project/vize-croquis-cf-multi-root-attrs.md)

### `vize:croquis/cf/mutated-after-escape`

Um objeto reativo é alterado depois de escapar de seu proprietário.

<span id="vize-croquis-cf-mutated-after-escape-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-mutated-after-escape-bad)

<span id="vize-croquis-cf-mutated-after-escape-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-mutated-after-escape-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-mutated-after-escape) · [Referência da regra](/rules/project/vize-croquis-cf-mutated-after-escape.md)

### `vize:croquis/cf/non-reactive-provide`

Um valor fornecido não é reativo, portanto os descendentes não verão atualizações.

<span id="vize-croquis-cf-non-reactive-provide-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-non-reactive-provide-bad)

<span id="vize-croquis-cf-non-reactive-provide-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-non-reactive-provide-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-non-reactive-provide) · [Referência da regra](/rules/project/vize-croquis-cf-non-reactive-provide.md)

### `vize:croquis/cf/non-unique-id`

Um id de elemento dentro de um laço não é único por item.

<span id="vize-croquis-cf-non-unique-id-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-non-unique-id-bad)

<span id="vize-croquis-cf-non-unique-id-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-non-unique-id-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-non-unique-id) · [Referência da regra](/rules/project/vize-croquis-cf-non-unique-id.md)

### `vize:croquis/cf/object-identity-comparison`

Um objeto reativo é comparado por identidade, que muda ao remover os invólucros.

<span id="vize-croquis-cf-object-identity-comparison-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-object-identity-comparison-bad)

<span id="vize-croquis-cf-object-identity-comparison-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-object-identity-comparison-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-object-identity-comparison) · [Referência da regra](/rules/project/vize-croquis-cf-object-identity-comparison.md)

### `vize:croquis/cf/pinia-getter`

Uma função de leitura do Pinia é lida sem `storeToRefs`, portanto não permanecerá reativa.

<span id="vize-croquis-cf-pinia-getter-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-pinia-getter-bad)

<span id="vize-croquis-cf-pinia-getter-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-pinia-getter-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-pinia-getter) · [Referência da regra](/rules/project/vize-croquis-cf-pinia-getter.md)

### `vize:croquis/cf/prop-type-mismatch`

O valor de uma prop passada não corresponde ao tipo declarado.

<span id="vize-croquis-cf-prop-type-mismatch-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-prop-type-mismatch-bad)

<span id="vize-croquis-cf-prop-type-mismatch-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-prop-type-mismatch-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-prop-type-mismatch) · [Referência da regra](/rules/project/vize-croquis-cf-prop-type-mismatch.md)

### `vize:croquis/cf/provide-inject-type`

Um valor fornecido e sua injeção não têm o mesmo tipo.

<span id="vize-croquis-cf-provide-inject-type-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-provide-inject-type-bad)

<span id="vize-croquis-cf-provide-inject-type-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-provide-inject-type-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-provide-inject-type) · [Referência da regra](/rules/project/vize-croquis-cf-provide-inject-type.md)

### `vize:croquis/cf/provide-without-symbol`

`provide` usa uma chave comum em vez de um símbolo `InjectionKey`.

<span id="vize-croquis-cf-provide-without-symbol-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-provide-without-symbol-bad)

<span id="vize-croquis-cf-provide-without-symbol-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-provide-without-symbol-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-provide-without-symbol) · [Referência da regra](/rules/project/vize-croquis-cf-provide-without-symbol.md)

### `vize:croquis/cf/reactive-export`

O estado reativo é exportado pelo módulo.

<span id="vize-croquis-cf-reactive-export-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-reactive-export-bad)

<span id="vize-croquis-cf-reactive-export-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-reactive-export-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-reactive-export) · [Referência da regra](/rules/project/vize-croquis-cf-reactive-export.md)

### `vize:croquis/cf/reactivity-outside-setup`

Uma API reativa é chamada fora de `setup`.

<span id="vize-croquis-cf-reactivity-outside-setup-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-reactivity-outside-setup-bad)

<span id="vize-croquis-cf-reactivity-outside-setup-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-reactivity-outside-setup-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-reactivity-outside-setup) · [Referência da regra](/rules/project/vize-croquis-cf-reactivity-outside-setup.md)

### `vize:croquis/cf/reassignment-breaks-reactivity`

Reatribuir uma variável reativa a substitui por um valor simples.

<span id="vize-croquis-cf-reassignment-breaks-reactivity-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity-bad)

<span id="vize-croquis-cf-reassignment-breaks-reactivity-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity) · [Referência da regra](/rules/project/vize-croquis-cf-reassignment-breaks-reactivity.md)

### `vize:croquis/cf/reference-escapes-scope`

Uma referência reativa escapa do escopo responsável por seu ciclo de vida.

<span id="vize-croquis-cf-reference-escapes-scope-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-reference-escapes-scope-bad)

<span id="vize-croquis-cf-reference-escapes-scope-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-reference-escapes-scope-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-reference-escapes-scope) · [Referência da regra](/rules/project/vize-croquis-cf-reference-escapes-scope.md)

### `vize:croquis/cf/setup-context-violation`

O contexto de setup é usado de uma forma que o Vue não permite.

<span id="vize-croquis-cf-setup-context-violation-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-setup-context-violation-bad)

<span id="vize-croquis-cf-setup-context-violation-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-setup-context-violation-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-setup-context-violation) · [Referência da regra](/rules/project/vize-croquis-cf-setup-context-violation.md)

### `vize:croquis/cf/shallow-deep-access`

Uma propriedade profunda de um valor `shallowReactive` ou `shallowRef` é lida como se fosse rastreada.

<span id="vize-croquis-cf-shallow-deep-access-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-shallow-deep-access-bad)

<span id="vize-croquis-cf-shallow-deep-access-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-shallow-deep-access-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-shallow-deep-access) · [Referência da regra](/rules/project/vize-croquis-cf-shallow-deep-access.md)

### `vize:croquis/cf/spread-breaks-reactivity`

Espalhar um objeto reativo copia seus valores e perde o rastreamento.

<span id="vize-croquis-cf-spread-breaks-reactivity-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity-bad)

<span id="vize-croquis-cf-spread-breaks-reactivity-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity) · [Referência da regra](/rules/project/vize-croquis-cf-spread-breaks-reactivity.md)

### `vize:croquis/cf/suspense-no-fallback`

`<Suspense>` não tem conteúdo alternativo.

<span id="vize-croquis-cf-suspense-no-fallback-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-suspense-no-fallback-bad)

<span id="vize-croquis-cf-suspense-no-fallback-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-suspense-no-fallback-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-suspense-no-fallback) · [Referência da regra](/rules/project/vize-croquis-cf-suspense-no-fallback.md)

### `vize:croquis/cf/template-ref-timing`

Uma ref de template é lida antes da montagem do componente.

<span id="vize-croquis-cf-template-ref-timing-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-template-ref-timing-bad)

<span id="vize-croquis-cf-template-ref-timing-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-template-ref-timing-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-template-ref-timing) · [Referência da regra](/rules/project/vize-croquis-cf-template-ref-timing.md)

### `vize:croquis/cf/toraw-mutation`

`toRaw` é usado e o objeto bruto é alterado em seguida.

<span id="vize-croquis-cf-toraw-mutation-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-toraw-mutation-bad)

<span id="vize-croquis-cf-toraw-mutation-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-toraw-mutation-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-toraw-mutation) · [Referência da regra](/rules/project/vize-croquis-cf-toraw-mutation.md)

### `vize:croquis/cf/uncaught-error`

Um componente pode lançar um erro e nenhum limite de erro o captura.

<span id="vize-croquis-cf-uncaught-error-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-uncaught-error-bad)

<span id="vize-croquis-cf-uncaught-error-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-uncaught-error-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-uncaught-error) · [Referência da regra](/rules/project/vize-croquis-cf-uncaught-error.md)

### `vize:croquis/cf/undeclared-emit`

O componente emite um evento que não foi declarado.

<span id="vize-croquis-cf-undeclared-emit-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-undeclared-emit-bad)

<span id="vize-croquis-cf-undeclared-emit-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-undeclared-emit-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-undeclared-emit) · [Referência da regra](/rules/project/vize-croquis-cf-undeclared-emit.md)

### `vize:croquis/cf/undeclared-prop`

Um pai passa uma prop que o filho não declara.

<span id="vize-croquis-cf-undeclared-prop-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-undeclared-prop-bad)

<span id="vize-croquis-cf-undeclared-prop-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-undeclared-prop-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-undeclared-prop) · [Referência da regra](/rules/project/vize-croquis-cf-undeclared-prop.md)

### `vize:croquis/cf/undefined-slot`

Um pai preenche um slot que o filho não expõe.

<span id="vize-croquis-cf-undefined-slot-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-undefined-slot-bad)

<span id="vize-croquis-cf-undefined-slot-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-undefined-slot-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-undefined-slot) · [Referência da regra](/rules/project/vize-croquis-cf-undefined-slot.md)

### `vize:croquis/cf/unhandled-event`

Um filho emite um evento que nenhum pai trata.

<span id="vize-croquis-cf-unhandled-event-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-unhandled-event-bad)

<span id="vize-croquis-cf-unhandled-event-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-unhandled-event-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-unhandled-event) · [Referência da regra](/rules/project/vize-croquis-cf-unhandled-event.md)

### `vize:croquis/cf/unmatched-inject`

`inject` nomeia uma chave que nenhum ancestral fornece.

<span id="vize-croquis-cf-unmatched-inject-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-unmatched-inject-bad)

<span id="vize-croquis-cf-unmatched-inject-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-unmatched-inject-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-unmatched-inject) · [Referência da regra](/rules/project/vize-croquis-cf-unmatched-inject.md)

### `vize:croquis/cf/unmatched-listener`

Um pai escuta um evento que o filho não emite.

<span id="vize-croquis-cf-unmatched-listener-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-unmatched-listener-bad)

<span id="vize-croquis-cf-unmatched-listener-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-unmatched-listener-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-unmatched-listener) · [Referência da regra](/rules/project/vize-croquis-cf-unmatched-listener.md)

### `vize:croquis/cf/unregistered-component`

Um template usa um componente que não foi registrado nem importado.

<span id="vize-croquis-cf-unregistered-component-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-unregistered-component-bad)

<span id="vize-croquis-cf-unregistered-component-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-unregistered-component-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-unregistered-component) · [Referência da regra](/rules/project/vize-croquis-cf-unregistered-component.md)

### `vize:croquis/cf/unresolved-import`

Uma importação não resolve para um módulo.

<span id="vize-croquis-cf-unresolved-import-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-unresolved-import-bad)

<span id="vize-croquis-cf-unresolved-import-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-unresolved-import-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-unresolved-import) · [Referência da regra](/rules/project/vize-croquis-cf-unresolved-import.md)

### `vize:croquis/cf/unused-attrs`

Atributos de herança automática são passados para um componente com múltiplas raízes que não os usa.

<span id="vize-croquis-cf-unused-attrs-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-unused-attrs-bad)

<span id="vize-croquis-cf-unused-attrs-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-unused-attrs-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-unused-attrs) · [Referência da regra](/rules/project/vize-croquis-cf-unused-attrs.md)

### `vize:croquis/cf/unused-emit`

Um evento declarado nunca é emitido.

<span id="vize-croquis-cf-unused-emit-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-unused-emit-bad)

<span id="vize-croquis-cf-unused-emit-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-unused-emit-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-unused-emit) · [Referência da regra](/rules/project/vize-croquis-cf-unused-emit.md)

### `vize:croquis/cf/unused-provide`

Uma chave fornecida nunca é injetada.

<span id="vize-croquis-cf-unused-provide-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-unused-provide-bad)

<span id="vize-croquis-cf-unused-provide-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-unused-provide-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-unused-provide) · [Referência da regra](/rules/project/vize-croquis-cf-unused-provide.md)

### `vize:croquis/cf/value-extraction-breaks-reactivity`

Ler um valor reativo para uma variável local perde as atualizações posteriores.

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity-bad)

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity) · [Referência da regra](/rules/project/vize-croquis-cf-value-extraction-breaks-reactivity.md)

### `vize:croquis/cf/watch-can-be-computed`

Um observador apenas copia um valor para o estado e pode ser um valor computado.

<span id="vize-croquis-cf-watch-can-be-computed-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-watch-can-be-computed-bad)

<span id="vize-croquis-cf-watch-can-be-computed-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-watch-can-be-computed-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-watch-can-be-computed) · [Referência da regra](/rules/project/vize-croquis-cf-watch-can-be-computed.md)

### `vize:croquis/cf/watcheffect-async`

`watchEffect` inicia uma tarefa assíncrona e não consegue limpar a execução anterior.

<span id="vize-croquis-cf-watcheffect-async-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-watcheffect-async-bad)

<span id="vize-croquis-cf-watcheffect-async-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-watcheffect-async-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-watcheffect-async) · [Referência da regra](/rules/project/vize-croquis-cf-watcheffect-async.md)

### `vize:croquis/cf/watcher-outside-setup`

`watch` ou `watchEffect` é chamado fora de `setup`.

<span id="vize-croquis-cf-watcher-outside-setup-bad"></span>

[Incorreto](./cross-file.md#vize-croquis-cf-watcher-outside-setup-bad)

<span id="vize-croquis-cf-watcher-outside-setup-good"></span>

[Correto](./cross-file.md#vize-croquis-cf-watcher-outside-setup-good)

[Exemplos do grupo](./cross-file.md#vize-croquis-cf-watcher-outside-setup) · [Referência da regra](/rules/project/vize-croquis-cf-watcher-outside-setup.md)

### `vue/a11y-img-alt`

Exigir o atributo alt nas imagens para garantir acessibilidade

<span id="vue-a11y-img-alt-bad"></span>

[Incorreto](./vue.md#vue-a11y-img-alt-bad)

<span id="vue-a11y-img-alt-good"></span>

[Correto](./vue.md#vue-a11y-img-alt-good)

[Exemplos do grupo](./vue.md#vue-a11y-img-alt) · [Referência da regra](/rules/reference/vue-a11y-img-alt.md)

### `vue/attribute-hyphenation`

Aplicar um padrão de nomes de atributos em componentes personalizados

<span id="vue-attribute-hyphenation-bad"></span>

[Incorreto](./vue.md#vue-attribute-hyphenation-bad)

<span id="vue-attribute-hyphenation-good"></span>

[Correto](./vue.md#vue-attribute-hyphenation-good)

[Exemplos do grupo](./vue.md#vue-attribute-hyphenation) · [Referência da regra](/rules/reference/vue-attribute-hyphenation.md)

### `vue/attribute-order`

Aplicar uma ordem consistente aos atributos

<span id="vue-attribute-order-bad"></span>

[Incorreto](./vue.md#vue-attribute-order-bad)

<span id="vue-attribute-order-good"></span>

[Correto](./vue.md#vue-attribute-order-good)

[Exemplos do grupo](./vue.md#vue-attribute-order) · [Referência da regra](/rules/reference/vue-attribute-order.md)

### `vue/component-definition-name-casing`

Exigir PascalCase ou kebab-case nos nomes de definição de componentes

<span id="vue-component-definition-name-casing-bad"></span>

[Incorreto](./vue.md#vue-component-definition-name-casing-bad)

<span id="vue-component-definition-name-casing-good"></span>

[Correto](./vue.md#vue-component-definition-name-casing-good)

[Exemplos do grupo](./vue.md#vue-component-definition-name-casing) · [Referência da regra](/rules/reference/vue-component-definition-name-casing.md)

### `vue/component-name-in-template-casing`

Aplicar um padrão específico de maiúsculas e minúsculas aos nomes de componentes nos templates

<span id="vue-component-name-in-template-casing-bad"></span>

[Incorreto](./vue.md#vue-component-name-in-template-casing-bad)

<span id="vue-component-name-in-template-casing-good"></span>

[Correto](./vue.md#vue-component-name-in-template-casing-good)

[Exemplos do grupo](./vue.md#vue-component-name-in-template-casing) · [Referência da regra](/rules/reference/vue-component-name-in-template-casing.md)

### `vue/cross-file-attrs-fallthrough`

Um pai passa atributos para um filho resolvido cuja raiz não pode herdá-los e que não usa $attrs explicitamente.

<span id="vue-cross-file-attrs-fallthrough-bad"></span>

[Incorreto](./cross-file.md#vue-cross-file-attrs-fallthrough-bad)

<span id="vue-cross-file-attrs-fallthrough-good"></span>

[Correto](./cross-file.md#vue-cross-file-attrs-fallthrough-good)

[Exemplos do grupo](./cross-file.md#vue-cross-file-attrs-fallthrough) · [Referência da regra](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `vue/html-button-has-type`

Exigir um type explícito e válido nos elementos button

<span id="vue-html-button-has-type-bad"></span>

[Incorreto](./vue.md#vue-html-button-has-type-bad)

<span id="vue-html-button-has-type-good"></span>

[Correto](./vue.md#vue-html-button-has-type-good)

[Exemplos do grupo](./vue.md#vue-html-button-has-type) · [Referência da regra](/rules/reference/vue-html-button-has-type.md)

### `vue/html-quotes`

Aplicar um padrão de aspas aos atributos HTML

<span id="vue-html-quotes-bad"></span>

[Incorreto](./vue.md#vue-html-quotes-bad)

<span id="vue-html-quotes-good"></span>

[Correto](./vue.md#vue-html-quotes-good)

[Exemplos do grupo](./vue.md#vue-html-quotes) · [Referência da regra](/rules/reference/vue-html-quotes.md)

### `vue/html-self-closing`

Aplicar um padrão de tags com fechamento automático

<span id="vue-html-self-closing-bad"></span>

[Incorreto](./vue.md#vue-html-self-closing-bad)

<span id="vue-html-self-closing-good"></span>

[Correto](./vue.md#vue-html-self-closing-good)

[Exemplos do grupo](./vue.md#vue-html-self-closing) · [Referência da regra](/rules/reference/vue-html-self-closing.md)

### `vue/max-template-complexity`

Limitar a complexidade do próprio template de um componente, tanto ciclomática quanto cognitiva

<span id="vue-max-template-complexity-bad"></span>

[Incorreto](./vue.md#vue-max-template-complexity-bad)

<span id="vue-max-template-complexity-good"></span>

[Correto](./vue.md#vue-max-template-complexity-good)

[Exemplos do grupo](./vue.md#vue-max-template-complexity) · [Referência da regra](/rules/reference/vue-max-template-complexity.md)

### `vue/multi-word-component-names`

Exigir nomes de componentes com mais de uma palavra

<span id="vue-multi-word-component-names-bad"></span>

[Incorreto](./vue.md#vue-multi-word-component-names-bad)

<span id="vue-multi-word-component-names-good"></span>

[Correto](./vue.md#vue-multi-word-component-names-good)

[Exemplos do grupo](./vue.md#vue-multi-word-component-names) · [Referência da regra](/rules/reference/vue-multi-word-component-names.md)

### `vue/mustache-interpolation-spacing`

Aplicar espaçamento consistente dentro das interpolações com chaves duplas

<span id="vue-mustache-interpolation-spacing-bad"></span>

[Incorreto](./vue.md#vue-mustache-interpolation-spacing-bad)

<span id="vue-mustache-interpolation-spacing-good"></span>

[Correto](./vue.md#vue-mustache-interpolation-spacing-good)

[Exemplos do grupo](./vue.md#vue-mustache-interpolation-spacing) · [Referência da regra](/rules/reference/vue-mustache-interpolation-spacing.md)

### `vue/no-array-index-key`

Proibir o uso direto da variável de índice de v-for como :key

<span id="vue-no-array-index-key-bad"></span>

[Incorreto](./vue.md#vue-no-array-index-key-bad)

<span id="vue-no-array-index-key-good"></span>

[Correto](./vue.md#vue-no-array-index-key-good)

[Exemplos do grupo](./vue.md#vue-no-array-index-key) · [Referência da regra](/rules/reference/vue-no-array-index-key.md)

### `vue/no-bare-strings-in-template`

Proibir texto legível por pessoas diretamente no template quando ele deve ser internacionalizado

<span id="vue-no-bare-strings-in-template-bad"></span>

[Incorreto](./vue.md#vue-no-bare-strings-in-template-bad)

<span id="vue-no-bare-strings-in-template-good"></span>

[Correto](./vue.md#vue-no-bare-strings-in-template-good)

[Exemplos do grupo](./vue.md#vue-no-bare-strings-in-template) · [Referência da regra](/rules/reference/vue-no-bare-strings-in-template.md)

### `vue/no-boolean-attr-value`

Proibir valores explícitos em atributos HTML booleanos

<span id="vue-no-boolean-attr-value-bad"></span>

[Incorreto](./vue.md#vue-no-boolean-attr-value-bad)

<span id="vue-no-boolean-attr-value-good"></span>

[Correto](./vue.md#vue-no-boolean-attr-value-good)

[Exemplos do grupo](./vue.md#vue-no-boolean-attr-value) · [Referência da regra](/rules/reference/vue-no-boolean-attr-value.md)

### `vue/no-child-content`

Proibir conteúdo filho ao usar v-html ou v-text

<span id="vue-no-child-content-bad"></span>

[Incorreto](./vue.md#vue-no-child-content-bad)

<span id="vue-no-child-content-good"></span>

[Correto](./vue.md#vue-no-child-content-good)

[Exemplos do grupo](./vue.md#vue-no-child-content) · [Referência da regra](/rules/reference/vue-no-child-content.md)

### `vue/no-deprecated-filter`

Proibir a sintaxe obsoleta de filtros do Vue 2 com o operador de barra vertical

<span id="vue-no-deprecated-filter-bad"></span>

[Incorreto](./vue.md#vue-no-deprecated-filter-bad)

<span id="vue-no-deprecated-filter-good"></span>

[Correto](./vue.md#vue-no-deprecated-filter-good)

[Exemplos do grupo](./vue.md#vue-no-deprecated-filter) · [Referência da regra](/rules/reference/vue-no-deprecated-filter.md)

### `vue/no-deprecated-functional-template`

Proibir o atributo `functional` no `<template>` de um SFC

<span id="vue-no-deprecated-functional-template-bad"></span>

[Incorreto](./vue.md#vue-no-deprecated-functional-template-bad)

<span id="vue-no-deprecated-functional-template-good"></span>

[Correto](./vue.md#vue-no-deprecated-functional-template-good)

[Exemplos do grupo](./vue.md#vue-no-deprecated-functional-template) · [Referência da regra](/rules/reference/vue-no-deprecated-functional-template.md)

### `vue/no-deprecated-html-element-is`

Proibir o atributo `is` em elementos HTML nativos

<span id="vue-no-deprecated-html-element-is-bad"></span>

[Incorreto](./vue.md#vue-no-deprecated-html-element-is-bad)

<span id="vue-no-deprecated-html-element-is-good"></span>

[Correto](./vue.md#vue-no-deprecated-html-element-is-good)

[Exemplos do grupo](./vue.md#vue-no-deprecated-html-element-is) · [Referência da regra](/rules/reference/vue-no-deprecated-html-element-is.md)

### `vue/no-deprecated-inline-template`

Proibir o atributo obsoleto `inline-template`

<span id="vue-no-deprecated-inline-template-bad"></span>

[Incorreto](./vue.md#vue-no-deprecated-inline-template-bad)

<span id="vue-no-deprecated-inline-template-good"></span>

[Correto](./vue.md#vue-no-deprecated-inline-template-good)

[Exemplos do grupo](./vue.md#vue-no-deprecated-inline-template) · [Referência da regra](/rules/reference/vue-no-deprecated-inline-template.md)

### `vue/no-deprecated-router-link-tag-prop`

Proibir a prop `tag` em &lt;router-link&gt;

<span id="vue-no-deprecated-router-link-tag-prop-bad"></span>

[Incorreto](./vue.md#vue-no-deprecated-router-link-tag-prop-bad)

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

[Correto](./vue.md#vue-no-deprecated-router-link-tag-prop-good)

[Exemplos do grupo](./vue.md#vue-no-deprecated-router-link-tag-prop) · [Referência da regra](/rules/reference/vue-no-deprecated-router-link-tag-prop.md)

### `vue/no-deprecated-scope-attribute`

Proibir o atributo obsoleto `scope` em &lt;template&gt;

<span id="vue-no-deprecated-scope-attribute-bad"></span>

[Incorreto](./vue.md#vue-no-deprecated-scope-attribute-bad)

<span id="vue-no-deprecated-scope-attribute-good"></span>

[Correto](./vue.md#vue-no-deprecated-scope-attribute-good)

[Exemplos do grupo](./vue.md#vue-no-deprecated-scope-attribute) · [Referência da regra](/rules/reference/vue-no-deprecated-scope-attribute.md)

### `vue/no-deprecated-slot-attribute`

Proibir o atributo obsoleto `slot`

<span id="vue-no-deprecated-slot-attribute-bad"></span>

[Incorreto](./vue.md#vue-no-deprecated-slot-attribute-bad)

<span id="vue-no-deprecated-slot-attribute-good"></span>

[Correto](./vue.md#vue-no-deprecated-slot-attribute-good)

[Exemplos do grupo](./vue.md#vue-no-deprecated-slot-attribute) · [Referência da regra](/rules/reference/vue-no-deprecated-slot-attribute.md)

### `vue/no-deprecated-slot-scope-attribute`

Proibir o atributo obsoleto `slot-scope`

<span id="vue-no-deprecated-slot-scope-attribute-bad"></span>

[Incorreto](./vue.md#vue-no-deprecated-slot-scope-attribute-bad)

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

[Correto](./vue.md#vue-no-deprecated-slot-scope-attribute-good)

[Exemplos do grupo](./vue.md#vue-no-deprecated-slot-scope-attribute) · [Referência da regra](/rules/reference/vue-no-deprecated-slot-scope-attribute.md)

### `vue/no-deprecated-v-bind-sync`

Proibir o modificador obsoleto `.sync` em `v-bind`

<span id="vue-no-deprecated-v-bind-sync-bad"></span>

[Incorreto](./vue.md#vue-no-deprecated-v-bind-sync-bad)

<span id="vue-no-deprecated-v-bind-sync-good"></span>

[Correto](./vue.md#vue-no-deprecated-v-bind-sync-good)

[Exemplos do grupo](./vue.md#vue-no-deprecated-v-bind-sync) · [Referência da regra](/rules/reference/vue-no-deprecated-v-bind-sync.md)

### `vue/no-deprecated-v-is`

Proibir a diretiva obsoleta `v-is`

<span id="vue-no-deprecated-v-is-bad"></span>

[Incorreto](./vue.md#vue-no-deprecated-v-is-bad)

<span id="vue-no-deprecated-v-is-good"></span>

[Correto](./vue.md#vue-no-deprecated-v-is-good)

[Exemplos do grupo](./vue.md#vue-no-deprecated-v-is) · [Referência da regra](/rules/reference/vue-no-deprecated-v-is.md)

### `vue/no-deprecated-v-on-native-modifier`

Proibir o modificador obsoleto `.native` em `v-on`

<span id="vue-no-deprecated-v-on-native-modifier-bad"></span>

[Incorreto](./vue.md#vue-no-deprecated-v-on-native-modifier-bad)

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

[Correto](./vue.md#vue-no-deprecated-v-on-native-modifier-good)

[Exemplos do grupo](./vue.md#vue-no-deprecated-v-on-native-modifier) · [Referência da regra](/rules/reference/vue-no-deprecated-v-on-native-modifier.md)

### `vue/no-deprecated-v-on-number-modifiers`

Proibir modificadores numéricos obsoletos de `keyCode` em `v-on`

<span id="vue-no-deprecated-v-on-number-modifiers-bad"></span>

[Incorreto](./vue.md#vue-no-deprecated-v-on-number-modifiers-bad)

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

[Correto](./vue.md#vue-no-deprecated-v-on-number-modifiers-good)

[Exemplos do grupo](./vue.md#vue-no-deprecated-v-on-number-modifiers) · [Referência da regra](/rules/reference/vue-no-deprecated-v-on-number-modifiers.md)

### `vue/no-dupe-v-else-if`

Proibir condições duplicadas em cadeias de `v-if` / `v-else-if`

<span id="vue-no-dupe-v-else-if-bad"></span>

[Incorreto](./vue.md#vue-no-dupe-v-else-if-bad)

<span id="vue-no-dupe-v-else-if-good"></span>

[Correto](./vue.md#vue-no-dupe-v-else-if-good)

[Exemplos do grupo](./vue.md#vue-no-dupe-v-else-if) · [Referência da regra](/rules/reference/vue-no-dupe-v-else-if.md)

### `vue/no-duplicate-attributes`

Proibir atributos duplicados no mesmo elemento

<span id="vue-no-duplicate-attributes-bad"></span>

[Incorreto](./vue.md#vue-no-duplicate-attributes-bad)

<span id="vue-no-duplicate-attributes-good"></span>

[Correto](./vue.md#vue-no-duplicate-attributes-good)

[Exemplos do grupo](./vue.md#vue-no-duplicate-attributes) · [Referência da regra](/rules/reference/vue-no-duplicate-attributes.md)

### `vue/no-empty-component-block`

Proibir blocos vazios em SFCs

<span id="vue-no-empty-component-block-bad"></span>

[Incorreto](./vue.md#vue-no-empty-component-block-bad)

<span id="vue-no-empty-component-block-good"></span>

[Correto](./vue.md#vue-no-empty-component-block-good)

[Exemplos do grupo](./vue.md#vue-no-empty-component-block) · [Referência da regra](/rules/reference/vue-no-empty-component-block.md)

### `vue/no-inline-style`

Desencorajar o uso de atributos de estilo inline

<span id="vue-no-inline-style-bad"></span>

[Incorreto](./vue.md#vue-no-inline-style-bad)

<span id="vue-no-inline-style-good"></span>

[Correto](./vue.md#vue-no-inline-style-good)

[Exemplos do grupo](./vue.md#vue-no-inline-style) · [Referência da regra](/rules/reference/vue-no-inline-style.md)

### `vue/no-invalid-html-attribute`

Proibir valores estáticos inválidos para atributos HTML

<span id="vue-no-invalid-html-attribute-bad"></span>

[Incorreto](./vue.md#vue-no-invalid-html-attribute-bad)

<span id="vue-no-invalid-html-attribute-good"></span>

[Correto](./vue.md#vue-no-invalid-html-attribute-good)

[Exemplos do grupo](./vue.md#vue-no-invalid-html-attribute) · [Referência da regra](/rules/reference/vue-no-invalid-html-attribute.md)

### `vue/no-lone-template`

Proibir elementos `<template>` desnecessários

<span id="vue-no-lone-template-bad"></span>

[Incorreto](./vue.md#vue-no-lone-template-bad)

<span id="vue-no-lone-template-good"></span>

[Correto](./vue.md#vue-no-lone-template-good)

[Exemplos do grupo](./vue.md#vue-no-lone-template) · [Referência da regra](/rules/reference/vue-no-lone-template.md)

### `vue/no-multi-spaces`

Proibir vários espaços consecutivos

<span id="vue-no-multi-spaces-bad"></span>

[Incorreto](./vue.md#vue-no-multi-spaces-bad)

<span id="vue-no-multi-spaces-good"></span>

[Correto](./vue.md#vue-no-multi-spaces-good)

[Exemplos do grupo](./vue.md#vue-no-multi-spaces) · [Referência da regra](/rules/reference/vue-no-multi-spaces.md)

### `vue/no-multiple-objects-in-class`

Proibir vários objetos literais dentro de uma vinculação de array em :class

<span id="vue-no-multiple-objects-in-class-bad"></span>

[Incorreto](./vue.md#vue-no-multiple-objects-in-class-bad)

<span id="vue-no-multiple-objects-in-class-good"></span>

[Correto](./vue.md#vue-no-multiple-objects-in-class-good)

[Exemplos do grupo](./vue.md#vue-no-multiple-objects-in-class) · [Referência da regra](/rules/reference/vue-no-multiple-objects-in-class.md)

### `vue/no-multiple-template-root`

Proibir vários nós raiz em um template

<span id="vue-no-multiple-template-root-bad"></span>

[Incorreto](./vue.md#vue-no-multiple-template-root-bad)

<span id="vue-no-multiple-template-root-good"></span>

[Correto](./vue.md#vue-no-multiple-template-root-good)

[Exemplos do grupo](./vue.md#vue-no-multiple-template-root) · [Referência da regra](/rules/reference/vue-no-multiple-template-root.md)

### `vue/no-mutating-props`

Proibir a mutação de props de componentes

<span id="vue-no-mutating-props-bad"></span>

[Incorreto](./vue.md#vue-no-mutating-props-bad)

<span id="vue-no-mutating-props-good"></span>

[Correto](./vue.md#vue-no-mutating-props-good)

[Exemplos do grupo](./vue.md#vue-no-mutating-props) · [Referência da regra](/rules/reference/vue-no-mutating-props.md)

### `vue/no-negated-v-if-condition`

Proibir uma condição negada em v-if quando a cadeia tiver v-else

<span id="vue-no-negated-v-if-condition-bad"></span>

[Incorreto](./vue.md#vue-no-negated-v-if-condition-bad)

<span id="vue-no-negated-v-if-condition-good"></span>

[Correto](./vue.md#vue-no-negated-v-if-condition-good)

[Exemplos do grupo](./vue.md#vue-no-negated-v-if-condition) · [Referência da regra](/rules/reference/vue-no-negated-v-if-condition.md)

### `vue/no-non-component-keep-alive-child`

Proibir invólucros de elementos comuns diretamente abaixo de `<KeepAlive>`

<span id="vue-no-non-component-keep-alive-child-bad"></span>

[Incorreto](./vue.md#vue-no-non-component-keep-alive-child-bad)

<span id="vue-no-non-component-keep-alive-child-good"></span>

[Correto](./vue.md#vue-no-non-component-keep-alive-child-good)

[Exemplos do grupo](./vue.md#vue-no-non-component-keep-alive-child) · [Referência da regra](/rules/reference/vue-no-non-component-keep-alive-child.md)

### `vue/no-preprocessor-lang`

Desencorajar o uso de preprocessadores CSS em favor de CSS moderno

<span id="vue-no-preprocessor-lang-bad"></span>

[Incorreto](./vue.md#vue-no-preprocessor-lang-bad)

<span id="vue-no-preprocessor-lang-good"></span>

[Correto](./vue.md#vue-no-preprocessor-lang-good)

[Exemplos do grupo](./vue.md#vue-no-preprocessor-lang) · [Referência da regra](/rules/reference/vue-no-preprocessor-lang.md)

### `vue/no-reserved-component-names`

Proibir o uso de nomes reservados como nomes de componentes

<span id="vue-no-reserved-component-names-bad"></span>

[Incorreto](./vue.md#vue-no-reserved-component-names-bad)

<span id="vue-no-reserved-component-names-good"></span>

[Correto](./vue.md#vue-no-reserved-component-names-good)

[Exemplos do grupo](./vue.md#vue-no-reserved-component-names) · [Referência da regra](/rules/reference/vue-no-reserved-component-names.md)

### `vue/no-root-v-if`

Proibir v-if no único elemento raiz de um template

<span id="vue-no-root-v-if-bad"></span>

[Incorreto](./vue.md#vue-no-root-v-if-bad)

<span id="vue-no-root-v-if-good"></span>

[Correto](./vue.md#vue-no-root-v-if-good)

[Exemplos do grupo](./vue.md#vue-no-root-v-if) · [Referência da regra](/rules/reference/vue-no-root-v-if.md)

### `vue/no-script-non-standard-lang`

Desencorajar valores não padronizados de lang em scripts

<span id="vue-no-script-non-standard-lang-bad"></span>

[Incorreto](./vue.md#vue-no-script-non-standard-lang-bad)

<span id="vue-no-script-non-standard-lang-good"></span>

[Correto](./vue.md#vue-no-script-non-standard-lang-good)

[Exemplos do grupo](./vue.md#vue-no-script-non-standard-lang) · [Referência da regra](/rules/reference/vue-no-script-non-standard-lang.md)

### `vue/no-src-attribute`

Desencorajar o atributo src em blocos de SFCs

<span id="vue-no-src-attribute-bad"></span>

[Incorreto](./vue.md#vue-no-src-attribute-bad)

<span id="vue-no-src-attribute-good"></span>

[Correto](./vue.md#vue-no-src-attribute-good)

[Exemplos do grupo](./vue.md#vue-no-src-attribute) · [Referência da regra](/rules/reference/vue-no-src-attribute.md)

### `vue/no-static-inline-styles`

Proibir atributos estáticos de estilo inline

<span id="vue-no-static-inline-styles-bad"></span>

[Incorreto](./vue.md#vue-no-static-inline-styles-bad)

<span id="vue-no-static-inline-styles-good"></span>

[Correto](./vue.md#vue-no-static-inline-styles-good)

[Exemplos do grupo](./vue.md#vue-no-static-inline-styles) · [Referência da regra](/rules/reference/vue-no-static-inline-styles.md)

### `vue/no-template-key`

Proibir o atributo `key` em `<template>`

<span id="vue-no-template-key-bad"></span>

[Incorreto](./vue.md#vue-no-template-key-bad)

<span id="vue-no-template-key-good"></span>

[Correto](./vue.md#vue-no-template-key-good)

[Exemplos do grupo](./vue.md#vue-no-template-key) · [Referência da regra](/rules/reference/vue-no-template-key.md)

### `vue/no-template-lang`

Desencorajar o atributo lang no bloco template

<span id="vue-no-template-lang-bad"></span>

[Incorreto](./vue.md#vue-no-template-lang-bad)

<span id="vue-no-template-lang-good"></span>

[Correto](./vue.md#vue-no-template-lang-good)

[Exemplos do grupo](./vue.md#vue-no-template-lang) · [Referência da regra](/rules/reference/vue-no-template-lang.md)

### `vue/no-template-shadow`

Proibir nomes de variáveis que ocultam variáveis de um escopo externo

<span id="vue-no-template-shadow-bad"></span>

[Incorreto](./vue.md#vue-no-template-shadow-bad)

<span id="vue-no-template-shadow-good"></span>

[Correto](./vue.md#vue-no-template-shadow-good)

[Exemplos do grupo](./vue.md#vue-no-template-shadow) · [Referência da regra](/rules/reference/vue-no-template-shadow.md)

### `vue/no-template-target-blank`

Proibir target="_blank" sem rel="noopener noreferrer"

<span id="vue-no-template-target-blank-bad"></span>

[Incorreto](./vue.md#vue-no-template-target-blank-bad)

<span id="vue-no-template-target-blank-good"></span>

[Correto](./vue.md#vue-no-template-target-blank-good)

[Exemplos do grupo](./vue.md#vue-no-template-target-blank) · [Referência da regra](/rules/reference/vue-no-template-target-blank.md)

### `vue/no-textarea-mustache`

Proibir interpolação com chaves duplas em `<textarea>`

<span id="vue-no-textarea-mustache-bad"></span>

[Incorreto](./vue.md#vue-no-textarea-mustache-bad)

<span id="vue-no-textarea-mustache-good"></span>

[Correto](./vue.md#vue-no-textarea-mustache-good)

[Exemplos do grupo](./vue.md#vue-no-textarea-mustache) · [Referência da regra](/rules/reference/vue-no-textarea-mustache.md)

### `vue/no-undefined-refs`

Proibir referências a variáveis não definidas nos templates

<span id="vue-no-undefined-refs-bad"></span>

[Incorreto](./vue.md#vue-no-undefined-refs-bad)

<span id="vue-no-undefined-refs-good"></span>

[Correto](./vue.md#vue-no-undefined-refs-good)

[Exemplos do grupo](./vue.md#vue-no-undefined-refs) · [Referência da regra](/rules/reference/vue-no-undefined-refs.md)

### `vue/no-unsafe-url`

Alertar sobre vinculações de URL potencialmente inseguras

<span id="vue-no-unsafe-url-bad"></span>

[Incorreto](./vue.md#vue-no-unsafe-url-bad)

<span id="vue-no-unsafe-url-good"></span>

[Correto](./vue.md#vue-no-unsafe-url-good)

[Exemplos do grupo](./vue.md#vue-no-unsafe-url) · [Referência da regra](/rules/reference/vue-no-unsafe-url.md)

### `vue/no-unsandboxed-iframe`

Exigir um atributo sandbox nos elementos iframe

<span id="vue-no-unsandboxed-iframe-bad"></span>

[Incorreto](./vue.md#vue-no-unsandboxed-iframe-bad)

<span id="vue-no-unsandboxed-iframe-good"></span>

[Correto](./vue.md#vue-no-unsandboxed-iframe-good)

[Exemplos do grupo](./vue.md#vue-no-unsandboxed-iframe) · [Referência da regra](/rules/reference/vue-no-unsandboxed-iframe.md)

### `vue/no-unused-components`

Proibir o registro de componentes não usados nos templates

<span id="vue-no-unused-components-bad"></span>

[Incorreto](./vue.md#vue-no-unused-components-bad)

<span id="vue-no-unused-components-good"></span>

[Correto](./vue.md#vue-no-unused-components-good)

[Exemplos do grupo](./vue.md#vue-no-unused-components) · [Referência da regra](/rules/reference/vue-no-unused-components.md)

### `vue/no-unused-properties`

Proibir propriedades não usadas definidas em defineProps

<span id="vue-no-unused-properties-bad"></span>

[Incorreto](./vue.md#vue-no-unused-properties-bad)

<span id="vue-no-unused-properties-good"></span>

[Correto](./vue.md#vue-no-unused-properties-good)

[Exemplos do grupo](./vue.md#vue-no-unused-properties) · [Referência da regra](/rules/reference/vue-no-unused-properties.md)

### `vue/no-unused-refs`

Reportar refs de template (ref="x") nunca referenciadas em &lt;script&gt;

<span id="vue-no-unused-refs-bad"></span>

[Incorreto](./vue.md#vue-no-unused-refs-bad)

<span id="vue-no-unused-refs-good"></span>

[Correto](./vue.md#vue-no-unused-refs-good)

[Exemplos do grupo](./vue.md#vue-no-unused-refs) · [Referência da regra](/rules/reference/vue-no-unused-refs.md)

### `vue/no-unused-setup-bindings`

Proibir variáveis de script setup que nunca são lidas

<span id="vue-no-unused-setup-bindings-bad"></span>

[Incorreto](./vue.md#vue-no-unused-setup-bindings-bad)

<span id="vue-no-unused-setup-bindings-good"></span>

[Correto](./vue.md#vue-no-unused-setup-bindings-good)

[Exemplos do grupo](./vue.md#vue-no-unused-setup-bindings) · [Referência da regra](/rules/reference/vue-no-unused-setup-bindings.md)

### `vue/no-unused-vars`

Proibir definições de variáveis não usadas nas diretivas v-for e v-slot

<span id="vue-no-unused-vars-bad"></span>

[Incorreto](./vue.md#vue-no-unused-vars-bad)

<span id="vue-no-unused-vars-good"></span>

[Correto](./vue.md#vue-no-unused-vars-good)

[Exemplos do grupo](./vue.md#vue-no-unused-vars) · [Referência da regra](/rules/reference/vue-no-unused-vars.md)

### `vue/no-use-v-else-with-v-for`

Proibir `v-else-if` ou `v-else` no mesmo elemento que `v-for`

<span id="vue-no-use-v-else-with-v-for-bad"></span>

[Incorreto](./vue.md#vue-no-use-v-else-with-v-for-bad)

<span id="vue-no-use-v-else-with-v-for-good"></span>

[Correto](./vue.md#vue-no-use-v-else-with-v-for-good)

[Exemplos do grupo](./vue.md#vue-no-use-v-else-with-v-for) · [Referência da regra](/rules/reference/vue-no-use-v-else-with-v-for.md)

### `vue/no-use-v-if-with-v-for`

Proibir `v-if` no mesmo elemento que `v-for`

<span id="vue-no-use-v-if-with-v-for-bad"></span>

[Incorreto](./vue.md#vue-no-use-v-if-with-v-for-bad)

<span id="vue-no-use-v-if-with-v-for-good"></span>

[Correto](./vue.md#vue-no-use-v-if-with-v-for-good)

[Exemplos do grupo](./vue.md#vue-no-use-v-if-with-v-for) · [Referência da regra](/rules/reference/vue-no-use-v-if-with-v-for.md)

### `vue/no-useless-mustaches`

Proibir interpolação com chaves duplas cuja expressão seja uma string literal constante

<span id="vue-no-useless-mustaches-bad"></span>

[Incorreto](./vue.md#vue-no-useless-mustaches-bad)

<span id="vue-no-useless-mustaches-good"></span>

[Correto](./vue.md#vue-no-useless-mustaches-good)

[Exemplos do grupo](./vue.md#vue-no-useless-mustaches) · [Referência da regra](/rules/reference/vue-no-useless-mustaches.md)

### `vue/no-useless-template-attributes`

Proibir atributos sem efeito em elementos `<template>`

<span id="vue-no-useless-template-attributes-bad"></span>

[Incorreto](./vue.md#vue-no-useless-template-attributes-bad)

<span id="vue-no-useless-template-attributes-good"></span>

[Correto](./vue.md#vue-no-useless-template-attributes-good)

[Exemplos do grupo](./vue.md#vue-no-useless-template-attributes) · [Referência da regra](/rules/reference/vue-no-useless-template-attributes.md)

### `vue/no-useless-v-bind`

Proibir um v-bind cujo valor seja uma string literal simples

<span id="vue-no-useless-v-bind-bad"></span>

[Incorreto](./vue.md#vue-no-useless-v-bind-bad)

<span id="vue-no-useless-v-bind-good"></span>

[Correto](./vue.md#vue-no-useless-v-bind-good)

[Exemplos do grupo](./vue.md#vue-no-useless-v-bind) · [Referência da regra](/rules/reference/vue-no-useless-v-bind.md)

### `vue/no-v-for-template-key-on-child`

Proibir `key` no filho de um `<template v-for>`

<span id="vue-no-v-for-template-key-on-child-bad"></span>

[Incorreto](./vue.md#vue-no-v-for-template-key-on-child-bad)

<span id="vue-no-v-for-template-key-on-child-good"></span>

[Correto](./vue.md#vue-no-v-for-template-key-on-child-good)

[Exemplos do grupo](./vue.md#vue-no-v-for-template-key-on-child) · [Referência da regra](/rules/reference/vue-no-v-for-template-key-on-child.md)

### `vue/no-v-html`

Alertar sobre v-html para prevenir vulnerabilidades XSS

<span id="vue-no-v-html-bad"></span>

[Incorreto](./vue.md#vue-no-v-html-bad)

<span id="vue-no-v-html-good"></span>

[Correto](./vue.md#vue-no-v-html-good)

[Exemplos do grupo](./vue.md#vue-no-v-html) · [Referência da regra](/rules/reference/vue-no-v-html.md)

### `vue/no-v-text`

Proibir a diretiva v-text; preferir interpolação com chaves duplas

<span id="vue-no-v-text-bad"></span>

[Incorreto](./vue.md#vue-no-v-text-bad)

<span id="vue-no-v-text-good"></span>

[Correto](./vue.md#vue-no-v-text-good)

[Exemplos do grupo](./vue.md#vue-no-v-text) · [Referência da regra](/rules/reference/vue-no-v-text.md)

### `vue/no-v-text-v-html-on-component`

Proibir v-text / v-html em elementos de componente

<span id="vue-no-v-text-v-html-on-component-bad"></span>

[Incorreto](./vue.md#vue-no-v-text-v-html-on-component-bad)

<span id="vue-no-v-text-v-html-on-component-good"></span>

[Correto](./vue.md#vue-no-v-text-v-html-on-component-good)

[Exemplos do grupo](./vue.md#vue-no-v-text-v-html-on-component) · [Referência da regra](/rules/reference/vue-no-v-text-v-html-on-component.md)

### `vue/permitted-contents`

Aplicar as regras do modelo de conteúdo HTML

<span id="vue-permitted-contents-bad"></span>

[Incorreto](./vue.md#vue-permitted-contents-bad)

<span id="vue-permitted-contents-good"></span>

[Correto](./vue.md#vue-permitted-contents-good)

[Exemplos do grupo](./vue.md#vue-permitted-contents) · [Referência da regra](/rules/reference/vue-permitted-contents.md)

### `vue/prefer-props-shorthand`

Recomendar sintaxe abreviada para props (Vue 3.4+)

<span id="vue-prefer-props-shorthand-bad"></span>

[Incorreto](./vue.md#vue-prefer-props-shorthand-bad)

<span id="vue-prefer-props-shorthand-good"></span>

[Correto](./vue.md#vue-prefer-props-shorthand-good)

[Exemplos do grupo](./vue.md#vue-prefer-props-shorthand) · [Referência da regra](/rules/reference/vue-prefer-props-shorthand.md)

### `vue/prefer-true-attribute-shorthand`

Preferir a forma abreviada para um atributo booleano vinculado a `true`

<span id="vue-prefer-true-attribute-shorthand-bad"></span>

[Incorreto](./vue.md#vue-prefer-true-attribute-shorthand-bad)

<span id="vue-prefer-true-attribute-shorthand-good"></span>

[Correto](./vue.md#vue-prefer-true-attribute-shorthand-good)

[Exemplos do grupo](./vue.md#vue-prefer-true-attribute-shorthand) · [Referência da regra](/rules/reference/vue-prefer-true-attribute-shorthand.md)

### `vue/prop-name-casing`

Aplicar um padrão de maiúsculas e minúsculas aos nomes de props declaradas

<span id="vue-prop-name-casing-bad"></span>

[Incorreto](./vue.md#vue-prop-name-casing-bad)

<span id="vue-prop-name-casing-good"></span>

[Correto](./vue.md#vue-prop-name-casing-good)

[Exemplos do grupo](./vue.md#vue-prop-name-casing) · [Referência da regra](/rules/reference/vue-prop-name-casing.md)

### `vue/require-component-is`

Exigir `v-bind:is` em elementos `<component>`

<span id="vue-require-component-is-bad"></span>

[Incorreto](./vue.md#vue-require-component-is-bad)

<span id="vue-require-component-is-good"></span>

[Correto](./vue.md#vue-require-component-is-good)

[Exemplos do grupo](./vue.md#vue-require-component-is) · [Referência da regra](/rules/reference/vue-require-component-is.md)

### `vue/require-component-registration`

Exigir importação ou registro explícito de componentes

<span id="vue-require-component-registration-bad"></span>

[Incorreto](./vue.md#vue-require-component-registration-bad)

<span id="vue-require-component-registration-good"></span>

[Correto](./vue.md#vue-require-component-registration-good)

[Exemplos do grupo](./vue.md#vue-require-component-registration) · [Referência da regra](/rules/reference/vue-require-component-registration.md)

### `vue/require-scoped-style`

Exigir o atributo scoped nas tags style

<span id="vue-require-scoped-style-bad"></span>

[Incorreto](./vue.md#vue-require-scoped-style-bad)

<span id="vue-require-scoped-style-good"></span>

[Correto](./vue.md#vue-require-scoped-style-good)

[Exemplos do grupo](./vue.md#vue-require-scoped-style) · [Referência da regra](/rules/reference/vue-require-scoped-style.md)

### `vue/require-toggle-inside-transition`

Exigir uma alternância no elemento envolvido por `<transition>`

<span id="vue-require-toggle-inside-transition-bad"></span>

[Incorreto](./vue.md#vue-require-toggle-inside-transition-bad)

<span id="vue-require-toggle-inside-transition-good"></span>

[Correto](./vue.md#vue-require-toggle-inside-transition-good)

[Exemplos do grupo](./vue.md#vue-require-toggle-inside-transition) · [Referência da regra](/rules/reference/vue-require-toggle-inside-transition.md)

### `vue/require-v-for-key`

Exigir `v-bind:key` nas diretivas `v-for`

<span id="vue-require-v-for-key-bad"></span>

[Incorreto](./vue.md#vue-require-v-for-key-bad)

<span id="vue-require-v-for-key-good"></span>

[Correto](./vue.md#vue-require-v-for-key-good)

[Exemplos do grupo](./vue.md#vue-require-v-for-key) · [Referência da regra](/rules/reference/vue-require-v-for-key.md)

### `vue/scoped-event-names`

Recomendar nomes de eventos com escopo no formato context:event

<span id="vue-scoped-event-names-bad"></span>

[Incorreto](./vue.md#vue-scoped-event-names-bad)

<span id="vue-scoped-event-names-good"></span>

[Correto](./vue.md#vue-scoped-event-names-good)

[Exemplos do grupo](./vue.md#vue-scoped-event-names) · [Referência da regra](/rules/reference/vue-scoped-event-names.md)

### `vue/sfc-element-order`

Aplicar uma ordem consistente aos elementos de nível superior dos SFCs

<span id="vue-sfc-element-order-bad"></span>

[Incorreto](./vue.md#vue-sfc-element-order-bad)

<span id="vue-sfc-element-order-good"></span>

[Correto](./vue.md#vue-sfc-element-order-good)

[Exemplos do grupo](./vue.md#vue-sfc-element-order) · [Referência da regra](/rules/reference/vue-sfc-element-order.md)

### `vue/single-style-block`

Recomendar um único bloco style

<span id="vue-single-style-block-bad"></span>

[Incorreto](./vue.md#vue-single-style-block-bad)

<span id="vue-single-style-block-good"></span>

[Correto](./vue.md#vue-single-style-block-good)

[Exemplos do grupo](./vue.md#vue-single-style-block) · [Referência da regra](/rules/reference/vue-single-style-block.md)

### `vue/slot-name-casing`

Exigir kebab-case nos slots nomeados usados por v-slot

<span id="vue-slot-name-casing-bad"></span>

[Incorreto](./vue.md#vue-slot-name-casing-bad)

<span id="vue-slot-name-casing-good"></span>

[Correto](./vue.md#vue-slot-name-casing-good)

[Exemplos do grupo](./vue.md#vue-slot-name-casing) · [Referência da regra](/rules/reference/vue-slot-name-casing.md)

### `vue/this-in-template`

Proibir `this.` nas expressões do template

<span id="vue-this-in-template-bad"></span>

[Incorreto](./vue.md#vue-this-in-template-bad)

<span id="vue-this-in-template-good"></span>

[Correto](./vue.md#vue-this-in-template-good)

[Exemplos do grupo](./vue.md#vue-this-in-template) · [Referência da regra](/rules/reference/vue-this-in-template.md)

### `vue/use-unique-element-ids`

Exigir IDs de elementos únicos por meio de useId(), em vez de literais estáticos

<span id="vue-use-unique-element-ids-bad"></span>

[Incorreto](./vue.md#vue-use-unique-element-ids-bad)

<span id="vue-use-unique-element-ids-good"></span>

[Correto](./vue.md#vue-use-unique-element-ids-good)

[Exemplos do grupo](./vue.md#vue-use-unique-element-ids) · [Referência da regra](/rules/reference/vue-use-unique-element-ids.md)

### `vue/use-v-on-exact`

Exigir o modificador `.exact` em `v-on` quando houver manipuladores baseados em modificadores

<span id="vue-use-v-on-exact-bad"></span>

[Incorreto](./vue.md#vue-use-v-on-exact-bad)

<span id="vue-use-v-on-exact-good"></span>

[Correto](./vue.md#vue-use-v-on-exact-good)

[Exemplos do grupo](./vue.md#vue-use-v-on-exact) · [Referência da regra](/rules/reference/vue-use-v-on-exact.md)

### `vue/v-bind-style`

Aplicar um padrão de sintaxe à diretiva `v-bind`

<span id="vue-v-bind-style-bad"></span>

[Incorreto](./vue.md#vue-v-bind-style-bad)

<span id="vue-v-bind-style-good"></span>

[Correto](./vue.md#vue-v-bind-style-good)

[Exemplos do grupo](./vue.md#vue-v-bind-style) · [Referência da regra](/rules/reference/vue-v-bind-style.md)

### `vue/v-on-event-hyphenation`

Exigir hífens nos nomes de eventos personalizados em v-on de componentes

<span id="vue-v-on-event-hyphenation-bad"></span>

[Incorreto](./vue.md#vue-v-on-event-hyphenation-bad)

<span id="vue-v-on-event-hyphenation-good"></span>

[Correto](./vue.md#vue-v-on-event-hyphenation-good)

[Exemplos do grupo](./vue.md#vue-v-on-event-hyphenation) · [Referência da regra](/rules/reference/vue-v-on-event-hyphenation.md)

### `vue/v-on-handler-style`

Exigir manipuladores de v-on escritos como referência a método ou função inline

<span id="vue-v-on-handler-style-bad"></span>

[Incorreto](./vue.md#vue-v-on-handler-style-bad)

<span id="vue-v-on-handler-style-good"></span>

[Correto](./vue.md#vue-v-on-handler-style-good)

[Exemplos do grupo](./vue.md#vue-v-on-handler-style) · [Referência da regra](/rules/reference/vue-v-on-handler-style.md)

### `vue/v-on-style`

Aplicar um padrão de sintaxe à diretiva `v-on`

<span id="vue-v-on-style-bad"></span>

[Incorreto](./vue.md#vue-v-on-style-bad)

<span id="vue-v-on-style-good"></span>

[Correto](./vue.md#vue-v-on-style-good)

[Exemplos do grupo](./vue.md#vue-v-on-style) · [Referência da regra](/rules/reference/vue-v-on-style.md)

### `vue/v-slot-style`

Aplicar um padrão de sintaxe à diretiva `v-slot`

<span id="vue-v-slot-style-bad"></span>

[Incorreto](./vue.md#vue-v-slot-style-bad)

<span id="vue-v-slot-style-good"></span>

[Correto](./vue.md#vue-v-slot-style-good)

[Exemplos do grupo](./vue.md#vue-v-slot-style) · [Referência da regra](/rules/reference/vue-v-slot-style.md)

### `vue/valid-attribute-name`

Exigir nomes de atributos válidos

<span id="vue-valid-attribute-name-bad"></span>

[Incorreto](./vue.md#vue-valid-attribute-name-bad)

<span id="vue-valid-attribute-name-good"></span>

[Correto](./vue.md#vue-valid-attribute-name-good)

[Exemplos do grupo](./vue.md#vue-valid-attribute-name) · [Referência da regra](/rules/reference/vue-valid-attribute-name.md)

### `vue/valid-template-root`

Exigir uma raiz `<template>` válida para a semântica de fragmentos do Vue 3

<span id="vue-valid-template-root-bad"></span>

[Incorreto](./vue.md#vue-valid-template-root-bad)

<span id="vue-valid-template-root-good"></span>

[Correto](./vue.md#vue-valid-template-root-good)

[Exemplos do grupo](./vue.md#vue-valid-template-root) · [Referência da regra](/rules/reference/vue-valid-template-root.md)

### `vue/valid-v-bind`

Exigir diretivas `v-bind` válidas

<span id="vue-valid-v-bind-bad"></span>

[Incorreto](./vue.md#vue-valid-v-bind-bad)

<span id="vue-valid-v-bind-good"></span>

[Correto](./vue.md#vue-valid-v-bind-good)

[Exemplos do grupo](./vue.md#vue-valid-v-bind) · [Referência da regra](/rules/reference/vue-valid-v-bind.md)

### `vue/valid-v-cloak`

Exigir diretivas `v-cloak` válidas

<span id="vue-valid-v-cloak-bad"></span>

[Incorreto](./vue.md#vue-valid-v-cloak-bad)

<span id="vue-valid-v-cloak-good"></span>

[Correto](./vue.md#vue-valid-v-cloak-good)

[Exemplos do grupo](./vue.md#vue-valid-v-cloak) · [Referência da regra](/rules/reference/vue-valid-v-cloak.md)

### `vue/valid-v-else`

Exigir diretivas `v-else` válidas

<span id="vue-valid-v-else-bad"></span>

[Incorreto](./vue.md#vue-valid-v-else-bad)

<span id="vue-valid-v-else-good"></span>

[Correto](./vue.md#vue-valid-v-else-good)

[Exemplos do grupo](./vue.md#vue-valid-v-else) · [Referência da regra](/rules/reference/vue-valid-v-else.md)

### `vue/valid-v-else-if`

Exigir diretivas `v-else-if` válidas

<span id="vue-valid-v-else-if-bad"></span>

[Incorreto](./vue.md#vue-valid-v-else-if-bad)

<span id="vue-valid-v-else-if-good"></span>

[Correto](./vue.md#vue-valid-v-else-if-good)

[Exemplos do grupo](./vue.md#vue-valid-v-else-if) · [Referência da regra](/rules/reference/vue-valid-v-else-if.md)

### `vue/valid-v-for`

Exigir diretivas `v-for` válidas

<span id="vue-valid-v-for-bad"></span>

[Incorreto](./vue.md#vue-valid-v-for-bad)

<span id="vue-valid-v-for-good"></span>

[Correto](./vue.md#vue-valid-v-for-good)

[Exemplos do grupo](./vue.md#vue-valid-v-for) · [Referência da regra](/rules/reference/vue-valid-v-for.md)

### `vue/valid-v-html`

Exigir diretivas `v-html` válidas

<span id="vue-valid-v-html-bad"></span>

[Incorreto](./vue.md#vue-valid-v-html-bad)

<span id="vue-valid-v-html-good"></span>

[Correto](./vue.md#vue-valid-v-html-good)

[Exemplos do grupo](./vue.md#vue-valid-v-html) · [Referência da regra](/rules/reference/vue-valid-v-html.md)

### `vue/valid-v-if`

Exigir diretivas `v-if` válidas

<span id="vue-valid-v-if-bad"></span>

[Incorreto](./vue.md#vue-valid-v-if-bad)

<span id="vue-valid-v-if-good"></span>

[Correto](./vue.md#vue-valid-v-if-good)

[Exemplos do grupo](./vue.md#vue-valid-v-if) · [Referência da regra](/rules/reference/vue-valid-v-if.md)

### `vue/valid-v-is`

Exigir diretivas `v-is` válidas

<span id="vue-valid-v-is-bad"></span>

[Incorreto](./vue.md#vue-valid-v-is-bad)

<span id="vue-valid-v-is-good"></span>

[Correto](./vue.md#vue-valid-v-is-good)

[Exemplos do grupo](./vue.md#vue-valid-v-is) · [Referência da regra](/rules/reference/vue-valid-v-is.md)

### `vue/valid-v-memo`

Exigir diretivas `v-memo` válidas

<span id="vue-valid-v-memo-bad"></span>

[Incorreto](./vue.md#vue-valid-v-memo-bad)

<span id="vue-valid-v-memo-good"></span>

[Correto](./vue.md#vue-valid-v-memo-good)

[Exemplos do grupo](./vue.md#vue-valid-v-memo) · [Referência da regra](/rules/reference/vue-valid-v-memo.md)

### `vue/valid-v-model`

Exigir diretivas `v-model` válidas

<span id="vue-valid-v-model-bad"></span>

[Incorreto](./vue.md#vue-valid-v-model-bad)

<span id="vue-valid-v-model-good"></span>

[Correto](./vue.md#vue-valid-v-model-good)

[Exemplos do grupo](./vue.md#vue-valid-v-model) · [Referência da regra](/rules/reference/vue-valid-v-model.md)

### `vue/valid-v-on`

Exigir diretivas `v-on` válidas

<span id="vue-valid-v-on-bad"></span>

[Incorreto](./vue.md#vue-valid-v-on-bad)

<span id="vue-valid-v-on-good"></span>

[Correto](./vue.md#vue-valid-v-on-good)

[Exemplos do grupo](./vue.md#vue-valid-v-on) · [Referência da regra](/rules/reference/vue-valid-v-on.md)

### `vue/valid-v-once`

Exigir diretivas `v-once` válidas

<span id="vue-valid-v-once-bad"></span>

[Incorreto](./vue.md#vue-valid-v-once-bad)

<span id="vue-valid-v-once-good"></span>

[Correto](./vue.md#vue-valid-v-once-good)

[Exemplos do grupo](./vue.md#vue-valid-v-once) · [Referência da regra](/rules/reference/vue-valid-v-once.md)

### `vue/valid-v-pre`

Exigir diretivas `v-pre` válidas

<span id="vue-valid-v-pre-bad"></span>

[Incorreto](./vue.md#vue-valid-v-pre-bad)

<span id="vue-valid-v-pre-good"></span>

[Correto](./vue.md#vue-valid-v-pre-good)

[Exemplos do grupo](./vue.md#vue-valid-v-pre) · [Referência da regra](/rules/reference/vue-valid-v-pre.md)

### `vue/valid-v-show`

Exigir diretivas `v-show` válidas

<span id="vue-valid-v-show-bad"></span>

[Incorreto](./vue.md#vue-valid-v-show-bad)

<span id="vue-valid-v-show-good"></span>

[Correto](./vue.md#vue-valid-v-show-good)

[Exemplos do grupo](./vue.md#vue-valid-v-show) · [Referência da regra](/rules/reference/vue-valid-v-show.md)

### `vue/valid-v-slot`

Exigir diretivas `v-slot` válidas

<span id="vue-valid-v-slot-bad"></span>

[Incorreto](./vue.md#vue-valid-v-slot-bad)

<span id="vue-valid-v-slot-good"></span>

[Correto](./vue.md#vue-valid-v-slot-good)

[Exemplos do grupo](./vue.md#vue-valid-v-slot) · [Referência da regra](/rules/reference/vue-valid-v-slot.md)

### `vue/valid-v-text`

Exigir diretivas `v-text` válidas

<span id="vue-valid-v-text-bad"></span>

[Incorreto](./vue.md#vue-valid-v-text-bad)

<span id="vue-valid-v-text-good"></span>

[Correto](./vue.md#vue-valid-v-text-good)

[Exemplos do grupo](./vue.md#vue-valid-v-text) · [Referência da regra](/rules/reference/vue-valid-v-text.md)

### `vue/warn-custom-block`

Alertar sobre blocos personalizados em arquivos SFC

<span id="vue-warn-custom-block-bad"></span>

[Incorreto](./vue.md#vue-warn-custom-block-bad)

<span id="vue-warn-custom-block-good"></span>

[Correto](./vue.md#vue-warn-custom-block-good)

[Exemplos do grupo](./vue.md#vue-warn-custom-block) · [Referência da regra](/rules/reference/vue-warn-custom-block.md)

### `vue/warn-custom-directive`

Alertar sobre diretivas personalizadas que precisam de registro

<span id="vue-warn-custom-directive-bad"></span>

[Incorreto](./vue.md#vue-warn-custom-directive-bad)

<span id="vue-warn-custom-directive-good"></span>

[Correto](./vue.md#vue-warn-custom-directive-good)

[Exemplos do grupo](./vue.md#vue-warn-custom-directive) · [Referência da regra](/rules/reference/vue-warn-custom-directive.md)
