---
title: "Regras de tipo e script"
description: "Verificações de `<script>` e `<script setup>`: macros do compilador, reatividade, props e emits, e regras que usam informações de tipo do TypeScript."
---

# Regras de tipo e script

Verificações de `<script>` e `<script setup>`: macros do compilador, reatividade, props e emits, e regras que usam informações de tipo do TypeScript.

Cada página de regra mostra um exemplo que falha, um que passa e a configuração que ativa a regra.

## Essenciais (39)

Regras de correção da predefinição `essential`.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="script-no-arrow-functions-in-watch"></span>[`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | Proibir funções de seta como manipuladores de watch na Options API. |  |
| <span id="script-no-async-in-computed"></span>[`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | Proibir funções assíncronas em propriedades computadas. |  |
| <span id="script-no-computed-properties-in-data"></span>[`script/no-computed-properties-in-data`](./reference/script-no-computed-properties-in-data.md) | Proibir o uso de propriedades computadas dentro da função data do componente. |  |
| <span id="script-no-deprecated-data-object-declaration"></span>[`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | Proibir um literal de objeto como opção data do componente (o Vue 3 exige uma função). |  |
| <span id="script-no-deprecated-delete-set"></span>[`script/no-deprecated-delete-set`](./reference/script-no-deprecated-delete-set.md) | Proibir $set e $delete do Vue 2, removidos no Vue 3. |  |
| <span id="script-no-deprecated-destroyed-lifecycle"></span>[`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | Proibir os hooks de ciclo de vida obsoletos destroyed e beforeDestroy. | ✓ |
| <span id="script-no-deprecated-dollar-listeners-api"></span>[`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | Proibir a propriedade de instância $listeners removida no Vue 3 (incorporada a $attrs). |  |
| <span id="script-no-deprecated-dollar-scopedslots-api"></span>[`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | Proibir a propriedade de instância $scopedSlots removida no Vue 3 (usar $slots). |  |
| <span id="script-no-deprecated-events-api"></span>[`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | Proibir a API de eventos do Vue 2 removida ($on / $off / $once). |  |
| <span id="script-no-deprecated-model-definition"></span>[`script/no-deprecated-model-definition`](./reference/script-no-deprecated-model-definition.md) | Proibir a opção model do Vue 2, removida no Vue 3. |  |
| <span id="script-no-deprecated-props-default-this"></span>[`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | Proibir `this` dentro de uma função de valor padrão ou validação de prop (removido no Vue 3). |  |
| <span id="script-no-deprecated-vue-config-keycodes"></span>[`script/no-deprecated-vue-config-keycodes`](./reference/script-no-deprecated-vue-config-keycodes.md) | Proibir Vue.config.keyCodes, removido no Vue 3. |  |
| <span id="script-no-dupe-keys"></span>[`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | Proibir chaves duplicadas entre props/data/computed/methods/setup/inject da Options API. |  |
| <span id="script-no-duplicate-attr-inheritance"></span>[`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | Sinalizar um componente que aplica duas vezes seus atributos repassados. |  |
| <span id="script-no-export-in-script-setup"></span>[`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | Proibir instruções export dentro de &lt;script setup&gt;. |  |
| <span id="script-no-expose-after-await"></span>[`script/no-expose-after-await`](./reference/script-no-expose-after-await.md) | Proibir expose() e defineExpose() após um await. |  |
| <span id="script-no-import-compiler-macros"></span>[`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | Proibir a importação de macros do compilador Vue que são importadas automaticamente. |  |
| <span id="script-no-internal-imports"></span>[`script/no-internal-imports`](./reference/script-no-internal-imports.md) | Proibir importações de módulos internos do Vue. |  |
| <span id="script-no-lifecycle-after-await"></span>[`script/no-lifecycle-after-await`](./reference/script-no-lifecycle-after-await.md) | Proibir hooks de ciclo de vida registrados após um await em setup(). |  |
| <span id="script-no-potential-component-option-typo"></span>[`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | Sinalizar prováveis erros de digitação nos nomes de opções de componentes da Options API. |  |
| <span id="script-no-ref-as-operand"></span>[`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | Exigir que variáveis vinculadas a refs sejam acessadas por `.value` quando usadas como operandos. |  |
| <span id="script-no-required-prop-with-default"></span>[`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | Proibir uma prop que tenha required: true e também um valor padrão. |  |
| <span id="script-no-reserved-identifiers"></span>[`script/no-reserved-identifiers`](./reference/script-no-reserved-identifiers.md) | Proibir o uso de identificadores reservados pelo compilador Vue. |  |
| <span id="script-no-reserved-keys"></span>[`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | Proibir nomes reservados pelo Vue como chaves de props/data/computed/methods/setup/inject na Options API. |  |
| <span id="script-no-reserved-props"></span>[`script/no-reserved-props`](./reference/script-no-reserved-props.md) | Proibir nomes reservados na declaração de props de um componente. |  |
| <span id="script-no-side-effects-in-computed-properties"></span>[`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | Proibir efeitos colaterais em getters computados da Options API. |  |
| <span id="script-no-use-computed-property-like-method"></span>[`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | Proibir chamar uma propriedade computada da Options API como um método. |  |
| <span id="script-no-watch-after-await"></span>[`script/no-watch-after-await`](./reference/script-no-watch-after-await.md) | Proibir watchers registrados após um await em setup() sem guardar a função de parada. |  |
| <span id="script-prefer-import-from-vue"></span>[`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | Preferir importar de 'vue' em vez de pacotes internos. | ✓ |
| <span id="script-require-prop-type-constructor"></span>[`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | Exigir que os valores de `type` de props sejam construtores em vez de literais de string. |  |
| <span id="script-require-render-return"></span>[`script/require-render-return`](./reference/script-require-render-return.md) | Exigir que a função render do componente devolva um valor. |  |
| <span id="script-require-slots-as-functions"></span>[`script/require-slots-as-functions`](./reference/script-require-slots-as-functions.md) | Exigir que as propriedades de $slots sejam chamadas como funções. |  |
| <span id="script-require-valid-default-prop"></span>[`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | Exigir que o valor padrão de uma prop seja válido para seu tipo declarado. |  |
| <span id="script-return-in-computed-property"></span>[`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | Exigir um valor de retorno em todo getter computado. |  |
| <span id="script-return-in-emits-validator"></span>[`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | Exigir um valor de retorno em todo validador de emits da Options API. |  |
| <span id="script-valid-define-emits"></span>[`script/valid-define-emits`](./reference/script-valid-define-emits.md) | Exigir uso válido de defineEmits() (sem argumentos de tipo e de tempo de execução juntos, sem referências locais, uma única chamada). |  |
| <span id="script-valid-define-options"></span>[`script/valid-define-options`](./reference/script-valid-define-options.md) | Exigir uso válido de defineOptions() (um único argumento de objeto, sem props/emits/expose/slots). |  |
| <span id="script-valid-define-props"></span>[`script/valid-define-props`](./reference/script-valid-define-props.md) | Exigir uso válido de defineProps() (uma única chamada, sem argumentos de tipo e de tempo de execução juntos, sem referências locais). |  |
| <span id="script-valid-next-tick"></span>[`script/valid-next-tick`](./reference/script-valid-next-tick.md) | Exigir que o resultado de uma chamada de nextTick() seja aguardado, encadeado ou receba uma função de retorno. |  |

## Padrão (7)

Adicionadas por `happy-path`, a predefinição padrão.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="script-define-props-destructuring"></span>[`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | Exigir um estilo consistente de desestruturação de defineProps em &lt;script setup&gt;. |  |
| <span id="script-no-multiple-slot-args"></span>[`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | Proibir passar mais de um argumento a uma chamada de função de slot com escopo. |  |
| <span id="script-no-unstable-nested-components"></span>[`script/no-unstable-nested-components`](./reference/script-no-unstable-nested-components.md) | Proibir definições de componentes dentro de funções de setup ou renderização. |  |
| <span id="script-require-define-component"></span>[`script/require-define-component`](./reference/script-require-define-component.md) | Exigir defineComponent() em exports padrão de SFCs TypeScript. |  |
| <span id="script-require-explicit-emits"></span>[`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | Exigir que os eventos emitidos sejam declarados em defineEmits ou na opção emits. |  |
| <span id="type-require-typed-emits"></span>[`type/require-typed-emits`](./reference/type-require-typed-emits.md) | Exigir uma definição de tipo para defineEmits. |  |
| <span id="type-require-typed-props"></span>[`type/require-typed-props`](./reference/type-require-typed-props.md) | Exigir uma definição de tipo para defineProps. |  |

## Predefinições mais rígidas (61)

Só em `opinionated`, `nuxt` ou `ecosystem`. A coluna Predefinições mostra quais.

| Regra | O que verifica | Predefinições | Correção automática |
| --- | --- | --- | --- |
| <span id="script-component-options-name-casing"></span>[`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | Exigir PascalCase na opção `name` do componente. | `opinionated`, `nuxt` |  |
| <span id="script-custom-event-name-casing"></span>[`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | Exigir a capitalização configurada nos nomes de eventos personalizados emitidos. | `opinionated`, `nuxt` |  |
| <span id="script-define-emits-declaration"></span>[`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | Exigir a forma de defineEmits&lt;{}&gt;() baseada em tipos em vez da forma em tempo de execução ou de array. | `opinionated`, `nuxt` |  |
| <span id="script-define-macros-order"></span>[`script/define-macros-order`](./reference/script-define-macros-order.md) | Exigir uma ordem consistente para as macros do compilador Vue em &lt;script setup&gt;. | `opinionated`, `nuxt` |  |
| <span id="script-define-props-declaration"></span>[`script/define-props-declaration`](./reference/script-define-props-declaration.md) | Exigir defineProps&lt;{ ... }&gt;() baseado em tipos em vez da forma com objeto em tempo de execução. | `opinionated`, `nuxt` |  |
| <span id="script-no-boolean-default"></span>[`script/no-boolean-default`](./reference/script-no-boolean-default.md) | Proibir um valor padrão em uma prop Boolean. | `opinionated`, `nuxt` |  |
| <span id="script-no-deep-destructure-in-props"></span>[`script/no-deep-destructure-in-props`](./reference/script-no-deep-destructure-in-props.md) | Proibir desestruturação profundamente aninhada em defineProps. | `opinionated`, `nuxt` |  |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Proibir getCurrentInstance() no modo Vapor (retorna null). | `opinionated` |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | Proibir padrões da Options API no modo Vapor. | `opinionated` |  |
| <span id="script-no-unused-emit-declarations"></span>[`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | Sinalizar eventos declarados que nunca são emitidos. | `opinionated`, `nuxt` |  |
| <span id="script-no-with-defaults"></span>[`script/no-with-defaults`](./reference/script-no-with-defaults.md) | Desencorajar withDefaults em favor de valores padrão na desestruturação (Vue 3.5+). | `opinionated` |  |
| <span id="script-one-component-per-file"></span>[`script/one-component-per-file`](./reference/script-one-component-per-file.md) | Proibir mais de uma definição de componente em um arquivo. | `opinionated`, `nuxt` |  |
| <span id="script-order-in-components"></span>[`script/order-in-components`](./reference/script-order-in-components.md) | Exigir a ordem padrão das propriedades nas opções do componente. | `opinionated`, `nuxt` |  |
| <span id="script-prefer-define-options"></span>[`script/prefer-define-options`](./reference/script-prefer-define-options.md) | Preferir defineOptions() a um &lt;script&gt; comum que só define name/inheritAttrs. | `opinionated`, `nuxt` |  |
| <span id="script-prefer-use-template-ref"></span>[`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | Recomendar useTemplateRef em vez de ref para referências de template (Vue 3.5+). | `opinionated`, `nuxt` |  |
| <span id="script-require-default-prop"></span>[`script/require-default-prop`](./reference/script-require-default-prop.md) | Exigir um valor padrão para toda prop opcional que não seja Boolean. | `opinionated`, `nuxt` |  |
| <span id="script-require-explicit-slots"></span>[`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | Exigir que os slots consumidos por useSlots() sejam tipados explicitamente com defineSlots&lt;...&gt;(). | `opinionated`, `nuxt` |  |
| <span id="script-require-prop-types"></span>[`script/require-prop-types`](./reference/script-require-prop-types.md) | Exigir que toda prop declare um tipo. | `opinionated` |  |
| <span id="script-require-typed-object-prop"></span>[`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | Exigir um tipo explícito em uma prop cujo tipo de tempo de execução seja `Object` ou `Array`. | `opinionated`, `nuxt` |  |
| <span id="type-await-thenable"></span>[`type/await-thenable`](./reference/type-await-thenable.md) | Proibir await de valores que não são thenable. | `nuxt`, `opinionated` |  |
| <span id="type-no-array-delete"></span>[`type/no-array-delete`](./reference/type-no-array-delete.md) | Proibir delete de elementos em valores semelhantes a array. | `nuxt`, `opinionated` |  |
| <span id="type-no-base-to-string"></span>[`type/no-base-to-string`](./reference/type-no-base-to-string.md) | Proibir a conversão em string de valores que recaem em Object.prototype.toString(). | `nuxt`, `opinionated` |  |
| <span id="type-no-confusing-void-expression"></span>[`type/no-confusing-void-expression`](./reference/type-no-confusing-void-expression.md) | Proibir expressões do tipo void em posições de valor confusas. | `nuxt`, `opinionated` |  |
| <span id="type-no-deprecated"></span>[`type/no-deprecated`](./reference/type-no-deprecated.md) | Proibir o uso de declarações obsoletas. | `nuxt`, `opinionated` |  |
| <span id="type-no-duplicate-type-constituents"></span>[`type/no-duplicate-type-constituents`](./reference/type-no-duplicate-type-constituents.md) | Proibir constituintes duplicados em uniões ou interseções. | `nuxt`, `opinionated` |  |
| <span id="type-no-floating-promises"></span>[`type/no-floating-promises`](./reference/type-no-floating-promises.md) | Proibir Promises soltas (não tratadas). | `nuxt`, `opinionated` |  |
| <span id="type-no-for-in-array"></span>[`type/no-for-in-array`](./reference/type-no-for-in-array.md) | Proibir for-in sobre valores semelhantes a array. | `nuxt`, `opinionated` |  |
| <span id="type-no-generated-empty-object-type"></span>[`type/no-generated-empty-object-type`](./reference/type-no-generated-empty-object-type.md) | Proibir tipos de objeto vazios gerados por operações de tipo. | `nuxt`, `opinionated` |  |
| <span id="type-no-implied-eval"></span>[`type/no-implied-eval`](./reference/type-no-implied-eval.md) | Proibir APIs de execução dinâmica baseadas em string. | `nuxt`, `opinionated` |  |
| <span id="type-no-meaningless-void-operator"></span>[`type/no-meaningless-void-operator`](./reference/type-no-meaningless-void-operator.md) | Proibir o operador void em expressões que já são void. | `nuxt`, `opinionated` |  |
| <span id="type-no-misused-promises"></span>[`type/no-misused-promises`](./reference/type-no-misused-promises.md) | Proibir Promises em lugares que não foram feitos para tratá-las. | `nuxt`, `opinionated` |  |
| <span id="type-no-misused-spread"></span>[`type/no-misused-spread`](./reference/type-no-misused-spread.md) | Proibir espalhar valores em posições incompatíveis. | `nuxt`, `opinionated` |  |
| <span id="type-no-mixed-enums"></span>[`type/no-mixed-enums`](./reference/type-no-mixed-enums.md) | Proibir a mistura de membros string e numéricos num enum. | `nuxt`, `opinionated` |  |
| <span id="type-no-reactivity-loss"></span>[`type/no-reactivity-loss`](./reference/type-no-reactivity-loss.md) | Proibir cópias estáticas simples de valores reativos em atribuições e chamadas. | `nuxt`, `opinionated` |  |
| <span id="type-no-redundant-type-constituents"></span>[`type/no-redundant-type-constituents`](./reference/type-no-redundant-type-constituents.md) | Proibir membros de uniões e interseções que não acrescentam ou substituem informação de tipo. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-boolean-literal-compare"></span>[`type/no-unnecessary-boolean-literal-compare`](./reference/type-no-unnecessary-boolean-literal-compare.md) | Proibir comparações desnecessárias com literais booleanos. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-condition"></span>[`type/no-unnecessary-condition`](./reference/type-no-unnecessary-condition.md) | Proibir condições que são sempre verdadeiras, sempre falsas, never ou sempre nullish. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-template-expression"></span>[`type/no-unnecessary-template-expression`](./reference/type-no-unnecessary-template-expression.md) | Proibir template literals que envolvem uma única expressão sem necessidade. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-arguments"></span>[`type/no-unnecessary-type-arguments`](./reference/type-no-unnecessary-type-arguments.md) | Proibir argumentos de tipo iguais ao valor padrão. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-assertion"></span>[`type/no-unnecessary-type-assertion`](./reference/type-no-unnecessary-type-assertion.md) | Proibir asserções de tipo que não mudam o tipo da expressão. | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-conversion"></span>[`type/no-unnecessary-type-conversion`](./reference/type-no-unnecessary-type-conversion.md) | Proibir conversões que não mudam o tipo nem o valor da expressão. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-argument"></span>[`type/no-unsafe-argument`](./reference/type-no-unsafe-argument.md) | Proibir chamar uma função com um valor do tipo `any`. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-assignment"></span>[`type/no-unsafe-assignment`](./reference/type-no-unsafe-assignment.md) | Proibir atribuir valores any a alvos mais específicos. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-call"></span>[`type/no-unsafe-call`](./reference/type-no-unsafe-call.md) | Proibir chamar valores tipados como any. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-enum-assignment"></span>[`type/no-unsafe-enum-assignment`](./reference/type-no-unsafe-enum-assignment.md) | Proibir atribuir um valor que não é do enum a um enum. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-enum-comparison"></span>[`type/no-unsafe-enum-comparison`](./reference/type-no-unsafe-enum-comparison.md) | Proibir comparar um valor de enum com um valor de outro domínio. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-member-access"></span>[`type/no-unsafe-member-access`](./reference/type-no-unsafe-member-access.md) | Proibir acesso a membros de valores tipados como any. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-return"></span>[`type/no-unsafe-return`](./reference/type-no-unsafe-return.md) | Proibir devolver valores any a partir de funções. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-template-binding"></span>[`type/no-unsafe-template-binding`](./reference/type-no-unsafe-template-binding.md) | Proibir vinculações de template que resultam em tipos inseguros. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-unary-minus"></span>[`type/no-unsafe-unary-minus`](./reference/type-no-unsafe-unary-minus.md) | Proibir a negação unária em valores que não são number nem bigint. | `nuxt`, `opinionated` |  |
| <span id="type-only-throw-error"></span>[`type/only-throw-error`](./reference/type-only-throw-error.md) | Exigir que os valores lançados sejam semelhantes a Error. | `nuxt`, `opinionated` |  |
| <span id="type-prefer-promise-reject-errors"></span>[`type/prefer-promise-reject-errors`](./reference/type-prefer-promise-reject-errors.md) | Exigir que o motivo de rejeição de uma Promise seja semelhante a Error. | `nuxt`, `opinionated` |  |
| <span id="type-prefer-reduce-type-parameter"></span>[`type/prefer-reduce-type-parameter`](./reference/type-prefer-reduce-type-parameter.md) | Preferir o parâmetro de tipo de Array#reduce a uma asserção do valor inicial. | `nuxt`, `opinionated` |  |
| <span id="type-prefer-return-this-type"></span>[`type/prefer-return-this-type`](./reference/type-prefer-return-this-type.md) | Exigir `this` quando o método devolve apenas o tipo `this`. | `nuxt`, `opinionated` |  |
| <span id="type-related-getter-setter-pairs"></span>[`type/related-getter-setter-pairs`](./reference/type-related-getter-setter-pairs.md) | Exigir que getters e setters emparelhados usem tipos compatíveis. | `nuxt`, `opinionated` |  |
| <span id="type-require-await"></span>[`type/require-await`](./reference/type-require-await.md) | Proibir funções async que não contêm uma expressão await. | `nuxt`, `opinionated` |  |
| <span id="type-restrict-plus-operands"></span>[`type/restrict-plus-operands`](./reference/type-restrict-plus-operands.md) | Exigir que os operandos de + sejam explicitamente compatíveis. | `nuxt`, `opinionated` |  |
| <span id="type-restrict-template-expressions"></span>[`type/restrict-template-expressions`](./reference/type-restrict-template-expressions.md) | Exigir que as expressões de um template literal sejam compatíveis com string. | `nuxt`, `opinionated` |  |
| <span id="type-return-await"></span>[`type/return-await`](./reference/type-return-await.md) | Exigir o uso consistente de `return await` conforme o contexto e a configuração. | `nuxt`, `opinionated` |  |
| <span id="type-unbound-method"></span>[`type/unbound-method`](./reference/type-unbound-method.md) | Proibir referências a métodos sem o seu recetor (`this`). | `nuxt`, `opinionated` |  |
| <span id="type-use-unknown-in-catch-callback-variable"></span>[`type/use-unknown-in-catch-callback-variable`](./reference/type-use-unknown-in-catch-callback-variable.md) | Exigir que a variável do callback de catch de uma Promise tenha uma anotação unknown explícita. | `nuxt`, `opinionated` |  |

## Opcionais (16)

Não fazem parte de nenhuma predefinição. Ative pelo nome em `linter.rules`, passado a `withVue`.

| Regra | O que verifica | Correção automática |
| --- | --- | --- |
| <span id="script-consistent-props-emits-key-quotes"></span>[`script/consistent-props-emits-key-quotes`](./reference/script-consistent-props-emits-key-quotes.md) | Uniformizar chaves entre aspas nas declarações de props e emits. |  |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | Proibir o uso de nextTick() em componentes orientados a Vapor. |  |
| <span id="script-no-reactive-destructure"></span>[`script/no-reactive-destructure`](./reference/script-no-reactive-destructure.md) | Proibir a desestruturação de objetos reativos que causa perda de reatividade. |  |
| <span id="script-no-restricted-globals"></span>[`script/no-restricted-globals`](./reference/script-no-restricted-globals.md) | Proibir referências a variáveis globais do ambiente de execução que devem passar por um encapsulamento tipado. |  |
| <span id="script-no-restricted-members"></span>[`script/no-restricted-members`](./reference/script-no-restricted-members.md) | Proibir acessos a membros object.property configurados pelo projeto. |  |
| <span id="script-no-top-level-ref-in-script"></span>[`script/no-top-level-ref-in-script`](./reference/script-no-top-level-ref-in-script.md) | Proibir ref/reactive no nível superior para evitar contaminação de estado entre requisições. |  |
| <span id="script-prefer-computed"></span>[`script/prefer-computed`](./reference/script-prefer-computed.md) | Preferir computed() para estado reativo derivado. |  |
| <span id="script-prefer-define-emits-binding"></span>[`script/prefer-define-emits-binding`](./reference/script-prefer-define-emits-binding.md) | Preferir a função capturada de defineEmits ao $emit do template. | ✓ |
| <span id="script-prefer-ref-over-reactive"></span>[`script/prefer-ref-over-reactive`](./reference/script-prefer-ref-over-reactive.md) | Recomendar o uso de ref() em vez de reactive() para gerenciar estado. |  |
| <span id="script-prefer-use-attrs"></span>[`script/prefer-use-attrs`](./reference/script-prefer-use-attrs.md) | Recomendar o uso de useAttrs() em vez de context.attrs. |  |
| <span id="script-prefer-use-id"></span>[`script/prefer-use-id`](./reference/script-prefer-use-id.md) | Recomendar o uso de useId() para gerar IDs únicos (Vue 3.5+). |  |
| <span id="script-prefer-use-slots"></span>[`script/prefer-use-slots`](./reference/script-prefer-use-slots.md) | Recomendar o uso de useSlots() em vez de context.slots. |  |
| <span id="script-require-function-return-type"></span>[`script/require-function-return-type`](./reference/script-require-function-return-type.md) | Exigir anotações de tipo de retorno nas funções. |  |
| <span id="script-require-symbol-provide"></span>[`script/require-symbol-provide`](./reference/script-require-symbol-provide.md) | Recomendar o uso de Symbol como chave de injeção para provide/inject. |  |
| <span id="script-require-typed-ref"></span>[`script/require-typed-ref`](./reference/script-require-typed-ref.md) | Exigir um argumento de tipo explícito em um ref() inicializado sem valor, com null ou com undefined. |  |
| <span id="type-strict-boolean-expressions"></span>[`type/strict-boolean-expressions`](./reference/type-strict-boolean-expressions.md) | Exigir expressões booleanas seguras nas condições de script e template. |  |

[Visão geral das regras](../rules/index.md) · [Opções das regras](/rules/options.md) · [Migração do ESLint](/rules/migration.md)
