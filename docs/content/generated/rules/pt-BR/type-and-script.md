---
title: "Regras de Tipo e Script"
---

# Regras de Tipo e Script

Cada regra desta categoria reúne nesta página sua finalidade, configuração e exemplos incorreto e correto completos. Os exemplos de projeto também incluem os arquivos compartilhados, que devem ser usados nos dois casos. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo. As notas de suporte distinguem os diagnósticos disponíveis das convenções ilustrativas e dos contratos sem produtor atual; ativar um ID não implementa uma verificação ausente.

<span id="configuração-do-checker"></span>

| Regra | Exemplos | Finalidade |
| --- | --- | --- |
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
| [`type/no-floating-promises`](#type-no-floating-promises) | [Incorreto](#type-no-floating-promises-bad) · [Correto](#type-no-floating-promises-good) | Proibir Promises soltas (não tratadas) |
| [`type/no-reactivity-loss`](#type-no-reactivity-loss) | [Incorreto](#type-no-reactivity-loss-bad) · [Correto](#type-no-reactivity-loss-good) | Proibir cópias estáticas simples de valores reativos em atribuições e chamadas |
| [`type/no-unsafe-template-binding`](#type-no-unsafe-template-binding) | [Incorreto](#type-no-unsafe-template-binding-bad) · [Correto](#type-no-unsafe-template-binding-good) | Proibir vinculações de template que resultam em tipos inseguros |
| [`type/require-typed-emits`](#type-require-typed-emits) | [Incorreto](#type-require-typed-emits-bad) · [Correto](#type-require-typed-emits-good) | Exigir uma definição de tipo para defineEmits |
| [`type/require-typed-props`](#type-require-typed-props) | [Incorreto](#type-require-typed-props-bad) · [Correto](#type-require-typed-props-good) | Exigir uma definição de tipo para defineProps |
| [`type/strict-boolean-expressions`](#type-strict-boolean-expressions) | [Incorreto](#type-strict-boolean-expressions-bad) · [Correto](#type-strict-boolean-expressions-good) | Exigir expressões booleanas seguras nas condições de script e template |

[Todas as regras](./all.md) · [Opções das regras](/rules/options.md) · [Mapa de migração do ESLint](/rules/migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](/rules/project/vue-cross-file-attrs-fallthrough.md)

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

```vue annotate="remove:4,5,8,9,10,11"
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
