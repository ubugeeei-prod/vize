---
title: "Règles de type et de script"
---

# Règles de type et de script

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.

<span id="checker-configuration"></span>

| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`script/component-options-name-casing`](#script-component-options-name-casing) | [Mauvais](#script-component-options-name-casing-bad) · [Bon](#script-component-options-name-casing-good) | Imposer PascalCase à l’option `name` du composant |
| [`script/custom-event-name-casing`](#script-custom-event-name-casing) | [Mauvais](#script-custom-event-name-casing-bad) · [Bon](#script-custom-event-name-casing-good) | Imposer camelCase aux noms des événements personnalisés émis |
| [`script/define-emits-declaration`](#script-define-emits-declaration) | [Mauvais](#script-define-emits-declaration-bad) · [Bon](#script-define-emits-declaration-good) | Imposer la forme typée defineEmits&lt;{}&gt;() à la place de la forme à l’exécution sous forme de tableau |
| [`script/define-macros-order`](#script-define-macros-order) | [Mauvais](#script-define-macros-order-bad) · [Bon](#script-define-macros-order-good) | Imposer un ordre cohérent aux macros du compilateur Vue dans &lt;script setup&gt; |
| [`script/define-props-declaration`](#script-define-props-declaration) | [Mauvais](#script-define-props-declaration-bad) · [Bon](#script-define-props-declaration-good) | Imposer la forme typée defineProps&lt;{ ... }&gt;() à la place de la forme à l’exécution sous forme d’objet |
| [`script/define-props-destructuring`](#script-define-props-destructuring) | [Mauvais](#script-define-props-destructuring-bad) · [Bon](#script-define-props-destructuring-good) | Imposer un style cohérent de déstructuration de defineProps dans &lt;script setup&gt; |
| [`script/no-arrow-functions-in-watch`](#script-no-arrow-functions-in-watch) | [Mauvais](#script-no-arrow-functions-in-watch-bad) · [Bon](#script-no-arrow-functions-in-watch-good) | Interdire les fonctions fléchées comme gestionnaires watch de l’Options API |
| [`script/no-async-in-computed`](#script-no-async-in-computed) | [Mauvais](#script-no-async-in-computed-bad) · [Bon](#script-no-async-in-computed-good) | Interdire les fonctions asynchrones dans les propriétés calculées |
| [`script/no-boolean-default`](#script-no-boolean-default) | [Mauvais](#script-no-boolean-default-bad) · [Bon](#script-no-boolean-default-good) | Interdire une valeur par défaut sur une prop Boolean |
| [`script/no-deep-destructure-in-props`](#script-no-deep-destructure-in-props) | [Mauvais](#script-no-deep-destructure-in-props-bad) · [Bon](#script-no-deep-destructure-in-props-good) | Interdire la déstructuration profondément imbriquée dans defineProps |
| [`script/no-deprecated-data-object-declaration`](#script-no-deprecated-data-object-declaration) | [Mauvais](#script-no-deprecated-data-object-declaration-bad) · [Bon](#script-no-deprecated-data-object-declaration-good) | Interdire un objet littéral comme option data du composant (Vue 3 exige une fonction) |
| [`script/no-deprecated-destroyed-lifecycle`](#script-no-deprecated-destroyed-lifecycle) | [Mauvais](#script-no-deprecated-destroyed-lifecycle-bad) · [Bon](#script-no-deprecated-destroyed-lifecycle-good) | Interdire les hooks de cycle de vie dépréciés destroyed et beforeDestroy |
| [`script/no-deprecated-dollar-listeners-api`](#script-no-deprecated-dollar-listeners-api) | [Mauvais](#script-no-deprecated-dollar-listeners-api-bad) · [Bon](#script-no-deprecated-dollar-listeners-api-good) | Interdire la propriété d’instance $listeners supprimée dans Vue 3 (fusionnée dans $attrs) |
| [`script/no-deprecated-dollar-scopedslots-api`](#script-no-deprecated-dollar-scopedslots-api) | [Mauvais](#script-no-deprecated-dollar-scopedslots-api-bad) · [Bon](#script-no-deprecated-dollar-scopedslots-api-good) | Interdire la propriété d’instance $scopedSlots supprimée dans Vue 3 (utiliser $slots) |
| [`script/no-deprecated-events-api`](#script-no-deprecated-events-api) | [Mauvais](#script-no-deprecated-events-api-bad) · [Bon](#script-no-deprecated-events-api-good) | Interdire l’API d’événements de Vue 2 supprimée ($on / $off / $once) |
| [`script/no-deprecated-props-default-this`](#script-no-deprecated-props-default-this) | [Mauvais](#script-no-deprecated-props-default-this-bad) · [Bon](#script-no-deprecated-props-default-this-good) | Interdire `this` dans une fonction de valeur par défaut ou de validation de prop (supprimé dans Vue 3) |
| [`script/no-dupe-keys`](#script-no-dupe-keys) | [Mauvais](#script-no-dupe-keys-bad) · [Bon](#script-no-dupe-keys-good) | Interdire les clés dupliquées entre props/data/computed/methods/setup/inject de l’Options API |
| [`script/no-duplicate-attr-inheritance`](#script-no-duplicate-attr-inheritance) | [Mauvais](#script-no-duplicate-attr-inheritance-bad) · [Bon](#script-no-duplicate-attr-inheritance-good) | Signaler un composant qui applique deux fois ses attributs transmis automatiquement |
| [`script/no-export-in-script-setup`](#script-no-export-in-script-setup) | [Mauvais](#script-no-export-in-script-setup-bad) · [Bon](#script-no-export-in-script-setup-good) | Interdire les instructions export dans &lt;script setup&gt; |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [Mauvais](#script-no-get-current-instance-bad) · [Bon](#script-no-get-current-instance-good) | Interdire getCurrentInstance() en mode Vapor (renvoie null) |
| [`script/no-import-compiler-macros`](#script-no-import-compiler-macros) | [Mauvais](#script-no-import-compiler-macros-bad) · [Bon](#script-no-import-compiler-macros-good) | Interdire l’import des macros du compilateur Vue importées automatiquement |
| [`script/no-internal-imports`](#script-no-internal-imports) | [Mauvais](#script-no-internal-imports-bad) · [Bon](#script-no-internal-imports-good) | Interdire les imports depuis les modules internes de Vue |
| [`script/no-multiple-slot-args`](#script-no-multiple-slot-args) | [Mauvais](#script-no-multiple-slot-args-bad) · [Bon](#script-no-multiple-slot-args-good) | Interdire de passer plusieurs arguments à un appel de fonction de slot à portée |
| [`script/no-next-tick`](#script-no-next-tick) | [Mauvais](#script-no-next-tick-bad) · [Bon](#script-no-next-tick-good) | Interdire l’utilisation de nextTick() dans les composants destinés à Vapor |
| [`script/no-options-api`](#script-no-options-api) | [Mauvais](#script-no-options-api-bad) · [Bon](#script-no-options-api-good) | Interdire les formes de l’Options API en mode Vapor |
| [`script/no-potential-component-option-typo`](#script-no-potential-component-option-typo) | [Mauvais](#script-no-potential-component-option-typo-bad) · [Bon](#script-no-potential-component-option-typo-good) | Signaler les fautes de frappe probables dans les noms d’options de composant de l’Options API |
| [`script/no-reactive-destructure`](#script-no-reactive-destructure) | [Mauvais](#script-no-reactive-destructure-bad) · [Bon](#script-no-reactive-destructure-good) | Interdire la déstructuration d’objets réactifs qui fait perdre la réactivité |
| [`script/no-ref-as-operand`](#script-no-ref-as-operand) | [Mauvais](#script-no-ref-as-operand-bad) · [Bon](#script-no-ref-as-operand-good) | Exiger l’accès via `.value` aux variables liées à une ref lorsqu’elles servent d’opérande |
| [`script/no-required-prop-with-default`](#script-no-required-prop-with-default) | [Mauvais](#script-no-required-prop-with-default-bad) · [Bon](#script-no-required-prop-with-default-good) | Interdire une prop qui possède à la fois required: true et une valeur par défaut |
| [`script/no-reserved-identifiers`](#script-no-reserved-identifiers) | [Mauvais](#script-no-reserved-identifiers-bad) · [Bon](#script-no-reserved-identifiers-good) | Interdire les identifiants réservés du compilateur Vue |
| [`script/no-reserved-keys`](#script-no-reserved-keys) | [Mauvais](#script-no-reserved-keys-bad) · [Bon](#script-no-reserved-keys-good) | Interdire les noms réservés par Vue comme clés de props/data/computed/methods/setup/inject de l’Options API |
| [`script/no-reserved-props`](#script-no-reserved-props) | [Mauvais](#script-no-reserved-props-bad) · [Bon](#script-no-reserved-props-good) | Interdire les noms réservés dans la déclaration des props d’un composant |
| [`script/no-restricted-globals`](#script-no-restricted-globals) | [Mauvais](#script-no-restricted-globals-bad) · [Bon](#script-no-restricted-globals-good) | Interdire les références aux variables globales de l’environnement d’exécution qui doivent passer par une couche d’encapsulation typée |
| [`script/no-restricted-members`](#script-no-restricted-members) | [Mauvais](#script-no-restricted-members-bad) · [Bon](#script-no-restricted-members-good) | Interdire les accès aux membres object.property configurés par le projet |
| [`script/no-side-effects-in-computed-properties`](#script-no-side-effects-in-computed-properties) | [Mauvais](#script-no-side-effects-in-computed-properties-bad) · [Bon](#script-no-side-effects-in-computed-properties-good) | Interdire les effets de bord dans les getters calculés de l’Options API |
| [`script/no-top-level-ref-in-script`](#script-no-top-level-ref-in-script) | [Mauvais](#script-no-top-level-ref-in-script-bad) · [Bon](#script-no-top-level-ref-in-script-good) | Interdire ref/reactive au niveau supérieur pour éviter la contamination de l’état entre requêtes |
| [`script/no-unstable-nested-components`](#script-no-unstable-nested-components) | [Mauvais](#script-no-unstable-nested-components-bad) · [Bon](#script-no-unstable-nested-components-good) | Interdire les définitions de composants dans les fonctions setup ou de rendu |
| [`script/no-unused-emit-declarations`](#script-no-unused-emit-declarations) | [Mauvais](#script-no-unused-emit-declarations-bad) · [Bon](#script-no-unused-emit-declarations-good) | Signaler les événements déclarés qui ne sont jamais émis |
| [`script/no-use-computed-property-like-method`](#script-no-use-computed-property-like-method) | [Mauvais](#script-no-use-computed-property-like-method-bad) · [Bon](#script-no-use-computed-property-like-method-good) | Interdire d’appeler une propriété calculée de l’Options API comme une méthode |
| [`script/no-with-defaults`](#script-no-with-defaults) | [Mauvais](#script-no-with-defaults-bad) · [Bon](#script-no-with-defaults-good) | Déconseiller withDefaults au profit des valeurs par défaut dans la déstructuration (Vue 3.5+) |
| [`script/prefer-computed`](#script-prefer-computed) | [Mauvais](#script-prefer-computed-bad) · [Bon](#script-prefer-computed-good) | Préférer computed() pour l’état réactif dérivé |
| [`script/prefer-define-options`](#script-prefer-define-options) | [Mauvais](#script-prefer-define-options-bad) · [Bon](#script-prefer-define-options-good) | Préférer defineOptions() à un &lt;script&gt; ordinaire qui ne définit que name/inheritAttrs |
| [`script/prefer-import-from-vue`](#script-prefer-import-from-vue) | [Mauvais](#script-prefer-import-from-vue-bad) · [Bon](#script-prefer-import-from-vue-good) | Préférer les imports depuis 'vue' plutôt que depuis les packages internes |
| [`script/prefer-ref-over-reactive`](#script-prefer-ref-over-reactive) | [Mauvais](#script-prefer-ref-over-reactive-bad) · [Bon](#script-prefer-ref-over-reactive-good) | Recommander ref() plutôt que reactive() pour gérer l’état |
| [`script/prefer-use-attrs`](#script-prefer-use-attrs) | [Mauvais](#script-prefer-use-attrs-bad) · [Bon](#script-prefer-use-attrs-good) | Recommander useAttrs() plutôt que context.attrs |
| [`script/prefer-use-id`](#script-prefer-use-id) | [Mauvais](#script-prefer-use-id-bad) · [Bon](#script-prefer-use-id-good) | Recommander useId() pour générer des identifiants uniques (Vue 3.5+) |
| [`script/prefer-use-slots`](#script-prefer-use-slots) | [Mauvais](#script-prefer-use-slots-bad) · [Bon](#script-prefer-use-slots-good) | Recommander useSlots() plutôt que context.slots |
| [`script/prefer-use-template-ref`](#script-prefer-use-template-ref) | [Mauvais](#script-prefer-use-template-ref-bad) · [Bon](#script-prefer-use-template-ref-good) | Recommander useTemplateRef plutôt que ref pour les références de template (Vue 3.5+) |
| [`script/require-default-prop`](#script-require-default-prop) | [Mauvais](#script-require-default-prop-bad) · [Bon](#script-require-default-prop-good) | Exiger une valeur par défaut pour chaque prop facultative non booléenne |
| [`script/require-explicit-emits`](#script-require-explicit-emits) | [Mauvais](#script-require-explicit-emits-bad) · [Bon](#script-require-explicit-emits-good) | Exiger la déclaration des événements émis dans defineEmits ou l’option emits |
| [`script/require-explicit-slots`](#script-require-explicit-slots) | [Mauvais](#script-require-explicit-slots-bad) · [Bon](#script-require-explicit-slots-good) | Exiger que les slots utilisés via useSlots() soient explicitement typés avec defineSlots&lt;...&gt;() |
| [`script/require-function-return-type`](#script-require-function-return-type) | [Mauvais](#script-require-function-return-type-bad) · [Bon](#script-require-function-return-type-good) | Exiger des annotations de type de retour sur les fonctions |
| [`script/require-prop-type-constructor`](#script-require-prop-type-constructor) | [Mauvais](#script-require-prop-type-constructor-bad) · [Bon](#script-require-prop-type-constructor-good) | Exiger que les valeurs `type` des props soient des constructeurs plutôt que des chaînes littérales |
| [`script/require-prop-types`](#script-require-prop-types) | [Mauvais](#script-require-prop-types-bad) · [Bon](#script-require-prop-types-good) | Exiger que chaque prop déclare un type |
| [`script/require-symbol-provide`](#script-require-symbol-provide) | [Mauvais](#script-require-symbol-provide-bad) · [Bon](#script-require-symbol-provide-good) | Recommander Symbol comme clé d’injection pour provide/inject |
| [`script/require-typed-object-prop`](#script-require-typed-object-prop) | [Mauvais](#script-require-typed-object-prop-bad) · [Bon](#script-require-typed-object-prop-good) | Exiger un type explicite sur une prop dont le type à l’exécution est `Object` ou `Array` |
| [`script/require-typed-ref`](#script-require-typed-ref) | [Mauvais](#script-require-typed-ref-bad) · [Bon](#script-require-typed-ref-good) | Exiger un argument de type explicite sur un ref() initialisé sans valeur, avec null ou avec undefined |
| [`script/require-valid-default-prop`](#script-require-valid-default-prop) | [Mauvais](#script-require-valid-default-prop-bad) · [Bon](#script-require-valid-default-prop-good) | Exiger que la valeur par défaut d’une prop soit valide pour son type déclaré |
| [`script/return-in-computed-property`](#script-return-in-computed-property) | [Mauvais](#script-return-in-computed-property-bad) · [Bon](#script-return-in-computed-property-good) | Exiger une valeur de retour dans chaque getter calculé |
| [`script/return-in-emits-validator`](#script-return-in-emits-validator) | [Mauvais](#script-return-in-emits-validator-bad) · [Bon](#script-return-in-emits-validator-good) | Exiger une valeur de retour dans chaque validateur emits de l’Options API |
| [`script/valid-define-emits`](#script-valid-define-emits) | [Mauvais](#script-valid-define-emits-bad) · [Bon](#script-valid-define-emits-good) | Imposer une utilisation valide de defineEmits() (pas d’arguments de type et d’exécution combinés, pas de références locales, un seul appel) |
| [`script/valid-define-options`](#script-valid-define-options) | [Mauvais](#script-valid-define-options-bad) · [Bon](#script-valid-define-options-good) | Imposer une utilisation valide de defineOptions() (un seul argument objet, sans props/emits/expose/slots) |
| [`script/valid-define-props`](#script-valid-define-props) | [Mauvais](#script-valid-define-props-bad) · [Bon](#script-valid-define-props-good) | Imposer une utilisation valide de defineProps() (un seul appel, pas d’arguments de type et d’exécution combinés, pas de références locales) |
| [`script/valid-next-tick`](#script-valid-next-tick) | [Mauvais](#script-valid-next-tick-bad) · [Bon](#script-valid-next-tick-good) | Exiger que le résultat d’un appel nextTick() soit attendu, chaîné ou associé à un callback |
| [`type/no-floating-promises`](#type-no-floating-promises) | [Mauvais](#type-no-floating-promises-bad) · [Bon](#type-no-floating-promises-good) | Interdire les Promises laissées sans traitement |
| [`type/no-reactivity-loss`](#type-no-reactivity-loss) | [Mauvais](#type-no-reactivity-loss-bad) · [Bon](#type-no-reactivity-loss-good) | Interdire les instantanés ordinaires de valeurs réactives lors des affectations et des appels |
| [`type/no-unsafe-template-binding`](#type-no-unsafe-template-binding) | [Mauvais](#type-no-unsafe-template-binding-bad) · [Bon](#type-no-unsafe-template-binding-good) | Interdire les liaisons de template dont le type résolu est non sûr |
| [`type/require-typed-emits`](#type-require-typed-emits) | [Mauvais](#type-require-typed-emits-bad) · [Bon](#type-require-typed-emits-good) | Exiger une définition de type pour defineEmits |
| [`type/require-typed-props`](#type-require-typed-props) | [Mauvais](#type-require-typed-props-bad) · [Bon](#type-require-typed-props-good) | Exiger une définition de type pour defineProps |
| [`type/strict-boolean-expressions`](#type-strict-boolean-expressions) | [Mauvais](#type-strict-boolean-expressions-bad) · [Bon](#type-strict-boolean-expressions-good) | Exiger des expressions booléennes sûres dans les conditions des scripts et des templates |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `script/component-options-name-casing`

Imposer PascalCase à l’option `name` du composant

[Mauvais](#script-component-options-name-casing-bad) · [Bon](#script-component-options-name-casing-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’option de composant `name: 'my-component'` utilise kebab-case, alors que cette règle exige un nom de composant littéral en PascalCase.

```vue annotate="remove:3"
<script lang="ts">
export default {
name: 'my-component' // kebab-case
}
</script>
```

<span id="script-component-options-name-casing-good"></span>

**Bon**

`MyComponent` commence par une majuscule et ne contient que des caractères alphanumériques, ce qui satisfait le contrôle du nom.

```vue annotate="add:3"
<script lang="ts">
export default {
name: 'MyComponent'
}
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) · [Toutes les règles](all.md)

### `script/custom-event-name-casing`

Imposer camelCase aux noms des événements personnalisés émis

[Mauvais](#script-custom-event-name-casing-bad) · [Bon](#script-custom-event-name-casing-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

La chaîne émise `my-event` contient un tiret et enfreint la convention camelCase par défaut pour les noms d’événements.

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits(['my-event'])
emit('my-event')         // kebab-case → report
</script>
```

<span id="script-custom-event-name-casing-good"></span>

**Bon**

La déclaration et l’appel utilisent tous deux `myEvent`, ce qui maintient la correspondance entre le nom de l’événement et son émission tout en respectant la convention de casse par défaut. Une convention kebab-case configurée impose une autre forme.

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits(['myEvent'])
emit('myEvent')
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61) · [Toutes les règles](all.md)

### `script/define-emits-declaration`

Imposer la forme typée defineEmits&lt;{}&gt;() à la place de la forme à l’exécution sous forme de tableau

[Mauvais](#script-define-emits-declaration-bad) · [Bon](#script-define-emits-declaration-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`defineEmits(["change"])` utilise une déclaration par tableau à l’exécution ; cette règle de style préfère une déclaration typée.

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

<span id="script-define-emits-declaration-good"></span>

**Bon**

`defineEmits<{ change: [id: number] }>()` place la déclaration de l’événement dans un argument de type et décrit explicitement la charge utile numérique utilisée par `emit("change", 1)`.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) · [Toutes les règles](all.md)

### `script/define-macros-order`

Imposer un ordre cohérent aux macros du compilateur Vue dans &lt;script setup&gt;

[Mauvais](#script-define-macros-order-bad) · [Bon](#script-define-macros-order-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`defineProps` apparaît avant `defineModel`, alors que `defineModel` occupe une position antérieure dans l’ordre canonique des macros.

```vue annotate="remove:2,3"
<script setup lang="ts">
// defineProps before defineModel (out of canonical order)
const props = defineProps<{ count: number }>()
const model = defineModel<string>()
</script>
```

<span id="script-define-macros-order-good"></span>

**Bon**

Les déclarations suivent exactement la séquence `defineOptions`, `defineModel`, `defineProps`, `defineEmits`, `defineSlots`, avant les instructions d’exécution sans rapport avec elles.

```vue annotate="add:2,4,5,6"
<script setup lang="ts">
defineOptions({ name: 'MyComponent' })
const model = defineModel<string>()
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
defineSlots<{ default(props: {}): any }>()
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L46) · [Toutes les règles](all.md)

### `script/define-props-declaration`

Imposer la forme typée defineProps&lt;{ ... }&gt;() à la place de la forme à l’exécution sous forme d’objet

[Mauvais](#script-define-props-declaration-bad) · [Bon](#script-define-props-declaration-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`defineProps({ title: String })` fournit un objet à l’exécution, ce qui va à l’encontre de la préférence de cette règle pour les props typées.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

<span id="script-define-props-declaration-good"></span>

**Bon**

`defineProps<{ title: string }>()` déclare `title` dans l’argument de type et conserve l’accès `props.title` sans argument de déclaration à l’exécution.

```vue annotate="add:2"
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) · [Toutes les règles](all.md)

### `script/define-props-destructuring`

Imposer un style cohérent de déstructuration de defineProps dans &lt;script setup&gt;

[Mauvais](#script-define-props-destructuring-bad) · [Bon](#script-define-props-destructuring-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

`defineProps` est affecté à la seule liaison `props` au lieu d’être déstructuré, contrairement à la préférence par défaut pour la déstructuration.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

<span id="script-define-props-destructuring-good"></span>

**Bon**

Le motif objet lie directement `foo` et `bar` et donne une valeur par défaut à `bar`, qui est facultatif. Cela repose sur la déstructuration réactive des props de Vue 3.5+ ; le mode configurable `never` préfère la forme opposée.

```vue annotate="add:2"
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) · [Toutes les règles](all.md)

### `script/no-arrow-functions-in-watch`

Interdire les fonctions fléchées comme gestionnaires watch de l’Options API

[Mauvais](#script-no-arrow-functions-in-watch-bad) · [Bon](#script-no-arrow-functions-in-watch-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le watcher `value` de l’Options API et le gestionnaire imbriqué `other.handler` sont des fonctions fléchées. Une fonction fléchée capture le `this` du contexte environnant au lieu de recevoir l’instance du composant.

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

**Bon**

Les deux gestionnaires deviennent des méthodes ordinaires, ce qui permet à Vue de lier `this` au composant. L’option du watcher `deep: true` reste compatible avec la forme objet.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59) · [Toutes les règles](all.md)

### `script/no-async-in-computed`

Interdire les fonctions asynchrones dans les propriétés calculées

[Mauvais](#script-no-async-in-computed-bad) · [Bon](#script-no-async-in-computed-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le getter `computed` est `async` : la récupération produit donc une Promise au lieu d’une valeur calculée dérivée de manière synchrone.

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

**Bon**

La récupération asynchrone est déplacée dans `watch`, qui stocke son résultat dans `data.value`. Le nettoyage annule l’ancienne requête et empêche un callback inactif d’écrire un résultat périmé ; aucun getter calculé asynchrone ne subsiste.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L46) · [Toutes les règles](all.md)

### `script/no-boolean-default`

Interdire une valeur par défaut sur une prop Boolean

[Mauvais](#script-no-boolean-default-bad) · [Bon](#script-no-boolean-default-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`disabled` et `checked` déclarent tous deux un `default` sur une prop dont le seul constructeur est `Boolean` ; la règle rejette même une valeur par défaut explicite `false`.

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

**Bon**

Les props exclusivement booléennes omettent `default` et utilisent la valeur false implicite de Vue. L’union `[Boolean, String]` et la prop Number montrent que ce contrôle se limite au constructeur `Boolean` employé seul.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54) · [Toutes les règles](all.md)

### `script/no-deep-destructure-in-props`

Interdire la déstructuration profondément imbriquée dans defineProps

[Mauvais](#script-no-deep-destructure-in-props-bad) · [Bon](#script-no-deep-destructure-in-props-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le motif de liaison descend dans `user` pour déstructurer `name`, dépassant la profondeur limitée de déstructuration des props autorisée par défaut.

```vue annotate="remove:2"
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

<span id="script-no-deep-destructure-in-props-good"></span>

**Bon**

L’objet des props reste intact, et un getter calculé lit `props.user.name`. L’accès imbriqué reste explicite sans motif de liaison profondément imbriqué.

```vue annotate="add:2,3,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) · [Toutes les règles](all.md)

### `script/no-deprecated-data-object-declaration`

Interdire un objet littéral comme option data du composant (Vue 3 exige une fonction)

[Mauvais](#script-no-deprecated-data-object-declaration-bad) · [Bon](#script-no-deprecated-data-object-declaration-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’option `data` de l’Options API est un objet littéral, une forme de Vue 2 que Vue 3 n’accepte plus.

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

**Bon**

`data()` renvoie un nouvel objet `{ count: 0 }`, fournissant la déclaration de données par fonction exigée par Vue 3.

```vue annotate="add:3,4"
<script lang="ts">
export default {
data() {
return { count: 0 }
}
}
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) · [Toutes les règles](all.md)

### `script/no-deprecated-destroyed-lifecycle`

Interdire les hooks de cycle de vie dépréciés destroyed et beforeDestroy

[Mauvais](#script-no-deprecated-destroyed-lifecycle-bad) · [Bon](#script-no-deprecated-destroyed-lifecycle-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`beforeDestroy` est l’option de cycle de vie de Vue 2 supprimée, utilisée ici pour nettoyer le minuteur.

```vue annotate="remove:2"
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

**Bon**

Renommer le hook en `beforeUnmount` conserve le corps du nettoyage sous son nom de cycle de vie Vue 3.

```vue annotate="add:2"
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) · [Toutes les règles](all.md)

### `script/no-deprecated-dollar-listeners-api`

Interdire la propriété d’instance $listeners supprimée dans Vue 3 (fusionnée dans $attrs)

[Mauvais](#script-no-deprecated-dollar-listeners-api-bad) · [Bon](#script-no-deprecated-dollar-listeners-api-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les accès aux membres et la référence passée directement en argument utilisent tous `$listeners`, que Vue 3 a supprimé après avoir fusionné les écouteurs dans les attributs.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const handlers = this.$listeners
const forwarded = ctx.$listeners
emit('input', $listeners)
</script>
```

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

**Bon**

Les accès passent à `this.$attrs` et à `ctx.attrs` dans le contexte setup. Ils remplacent l’API supprimée des écouteurs ; les objets récepteurs illustrés doivent exister dans le contexte environnant du composant.

```vue annotate="add:2,3"
<script setup lang="ts">
const handlers = this.$attrs
const forwarded = ctx.attrs
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L40) · [Toutes les règles](all.md)

### `script/no-deprecated-dollar-scopedslots-api`

Interdire la propriété d’instance $scopedSlots supprimée dans Vue 3 (utiliser $slots)

[Mauvais](#script-no-deprecated-dollar-scopedslots-api-bad) · [Bon](#script-no-deprecated-dollar-scopedslots-api-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`this.$scopedSlots`, `ctx.$scopedSlots` et la référence directe `$scopedSlots` utilisent l’API des slots à portée de Vue 2, supprimée dans Vue 3.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

**Bon**

Remplacer `$scopedSlots` par `$slots` utilise l’API unifiée des slots. L’exemple supprime l’écriture dépréciée sans établir un contexte setup pour les objets récepteurs.

```vue annotate="add:2,3,4"
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L44) · [Toutes les règles](all.md)

### `script/no-deprecated-events-api`

Interdire l’API d’événements de Vue 2 supprimée ($on / $off / $once)

[Mauvais](#script-no-deprecated-events-api-bad) · [Bon](#script-no-deprecated-events-api-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les appels `$on`, `$once` et `$off` utilisent les méthodes de bus d’événements de l’instance supprimées dans Vue 3.

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
this.$on('event', handler)
this.$once('event', handler)
this.$off('event', handler)
emitter.$off('event')
</script>
```

<span id="script-no-deprecated-events-api-good"></span>

**Bon**

`$emit` reste valide, tandis que l’abonnement au bus d’événements passe à la méthode `on` de l’émetteur externe. La correction sépare l’émission destinée au parent du bus d’événements externe.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L42) · [Toutes les règles](all.md)

### `script/no-deprecated-props-default-this`

Interdire `this` dans une fonction de valeur par défaut ou de validation de prop (supprimé dans Vue 3)

[Mauvais](#script-no-deprecated-props-default-this-bad) · [Bon](#script-no-deprecated-props-default-this-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La valeur par défaut et le validateur de la prop lisent `this`, mais ces fonctions ne peuvent pas s’appuyer sur l’instance du composant dans Vue 3.

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

**Bon**

La fonction de valeur par défaut lit `props.baseSize` dans son argument, et le validateur teste son argument `value`. Tous deux cessent de dépendre d’un récepteur d’instance indisponible.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) · [Toutes les règles](all.md)

### `script/no-dupe-keys`

Interdire les clés dupliquées entre props/data/computed/methods/setup/inject de l’Options API

[Mauvais](#script-no-dupe-keys-bad) · [Bon](#script-no-dupe-keys-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`foo` est déclaré à la fois dans props et data, et `bar` à la fois dans computed et methods. Ces déclarations se disputent les mêmes clés de l’instance du composant.

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

**Bon**

Les déclarations de prop, de données et de propriété calculée utilisent des noms distincts (`foo`, `bar` et `baz`), éliminant les deux collisions entre options.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L52) · [Toutes les règles](all.md)

### `script/no-duplicate-attr-inheritance`

Signaler un composant qui applique deux fois ses attributs transmis automatiquement

[Mauvais](#script-no-duplicate-attr-inheritance-bad) · [Bon](#script-no-duplicate-attr-inheritance-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les valeurs explicites `inheritAttrs: true` répètent le comportement par défaut de Vue. Cette règle signale ce littéral redondant même si aucun déploiement de `$attrs` sur la racine n’est montré.

```vue annotate="remove:2,3"
<script lang="ts">
defineOptions({ inheritAttrs: true })
export default { inheritAttrs: true }
</script>
```

<span id="script-no-duplicate-attr-inheritance-good"></span>

**Bon**

`inheritAttrs: false` exprime une véritable désactivation, tandis que l’objet d’options vide laisse l’héritage par défaut implicite. Aucun ne répète la valeur redondante `true`.

```vue annotate="add:2,3"
<script lang="ts">
defineOptions({ inheritAttrs: false }) // intentional opt-out
export default {}                      // default inheritance, unstated
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L73) · [Toutes les règles](all.md)

### `script/no-export-in-script-setup`

Interdire les instructions export dans &lt;script setup&gt;

[Mauvais](#script-no-export-in-script-setup-bad) · [Bon](#script-no-export-in-script-setup-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`export const count` tente d’exposer un export de module depuis `<script setup>`, où les exports à l’exécution sont interdits.

```vue annotate="remove:2"
<script setup lang="ts">
export const count = 1;
</script>
```

<span id="script-no-export-in-script-setup-good"></span>

**Bon**

Supprimer `export` conserve `count` comme liaison setup plutôt que comme export de module.

```vue annotate="add:2"
<script setup lang="ts">
const count = 1;
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) · [Toutes les règles](all.md)

### `script/no-get-current-instance`

Interdire getCurrentInstance() en mode Vapor (renvoie null)

[Mauvais](#script-no-get-current-instance-bad) · [Bon](#script-no-get-current-instance-good)

Gravité par défaut: `error`  
Préréglages: `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Contrôles de script destinés à Vapor ; une activation explicite applique aussi la restriction aux scripts ordinaires  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le setup marqué Vapor importe et appelle `getCurrentInstance`, s’appuyant sur une API d’instance que cette règle interdit pour les composants destinés à Vapor.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

<span id="script-no-get-current-instance-good"></span>

**Bon**

`inject("app-config")` obtient la configuration explicitement fournie sans importer ni appeler `getCurrentInstance`.

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [Toutes les règles](all.md)

### `script/no-import-compiler-macros`

Interdire l’import des macros du compilateur Vue importées automatiquement

[Mauvais](#script-no-import-compiler-macros-bad) · [Bon](#script-no-import-compiler-macros-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’import depuis `vue` inclut `defineProps` et `defineEmits`, alors que ces macros du compilateur sont directement disponibles dans `<script setup>`.

```vue annotate="remove:2"
<script setup lang="ts">
import { defineProps, defineEmits } from "vue";
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

<span id="script-no-import-compiler-macros-good"></span>

**Bon**

Supprimer les imports des macros conserve les deux appels typés ; aucune déclaration ne nécessite d’import à l’exécution.

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) · [Toutes les règles](all.md)

### `script/no-internal-imports`

Interdire les imports depuis les modules internes de Vue

[Mauvais](#script-no-internal-imports-bad) · [Bon](#script-no-internal-imports-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les deux imports ciblent des fichiers internes `dist` plutôt que le point d’entrée public du package Vue, couplant le composant aux chemins des fichiers de build.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { foo } from '@vue/runtime-core/dist/runtime-core.esm-bundler'
import { bar } from 'vue/dist/vue.esm-bundler'
</script>
```

<span id="script-no-internal-imports-good"></span>

**Bon**

Importer les utilitaires nécessaires depuis `vue` supprime la dépendance aux emplacements des fichiers de distribution internes.

```vue annotate="add:2"
<script setup lang="ts">
import { ref, computed } from 'vue'
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28) · [Toutes les règles](all.md)

### `script/no-multiple-slot-args`

Interdire de passer plusieurs arguments à un appel de fonction de slot à portée

[Mauvais](#script-no-multiple-slot-args-bad) · [Bon](#script-no-multiple-slot-args-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les appels de slots passent plusieurs arguments positionnels ou déploient une liste d’arguments inconnue. Les slots Vue reçoivent un seul objet de props, et non une liste de paramètres positionnels.

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

**Bon**

`{ foo, bar }` regroupe les données dans un seul argument ; `slotProps` et l’appel sans argument respectent également la forme d’appel de slot prise en charge.

```vue annotate="add:2,3,4"
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) · [Toutes les règles](all.md)

### `script/no-next-tick`

Interdire l’utilisation de nextTick() dans les composants destinés à Vapor

[Mauvais](#script-no-next-tick-bad) · [Bon](#script-no-next-tick-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Contrôles de script destinés à Vapor ; une activation explicite applique aussi la restriction aux scripts ordinaires  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le composant destiné à Vapor importe `nextTick` et attend son résultat, introduisant la dépendance à la planification des mises à jour du DOM que cette règle de migration rejette.

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

<span id="script-no-next-tick-good"></span>

**Bon**

L’input est obtenu par `useTemplateRef` et reçoit le focus dans `onMounted`. Ce point de montage explicite remplace la dépendance de l’exemple à `nextTick`.

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [Toutes les règles](all.md)

### `script/no-options-api`

Interdire les formes de l’Options API en mode Vapor

[Mauvais](#script-no-options-api-bad) · [Bon](#script-no-options-api-good)

Gravité par défaut: `error`  
Préréglages: `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Contrôles de script destinés à Vapor ; une activation explicite applique aussi la restriction aux scripts ordinaires  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’objet exporté par défaut déclare `data()` de l’Options API, une forme d’option de composant interdite par cette règle.

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

**Bon**

L’état du composant devient une `ref` de la Composition API dans le `<script setup>` Vapor, supprimant l’objet de l’Options API et son option `data`.

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [Toutes les règles](all.md)

### `script/no-potential-component-option-typo`

Signaler les fautes de frappe probables dans les noms d’options de composant de l’Options API

[Mauvais](#script-no-potential-component-option-typo-bad) · [Bon](#script-no-potential-component-option-typo-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’option est écrite `method`, à une modification près de l’option reconnue `methods` ; Vue ne la traiterait pas comme la déclaration de méthodes souhaitée.

```vue annotate="remove:2"
<script lang="ts">
export default { method: { save() {} } };
</script>
```

<span id="script-no-potential-component-option-typo-good"></span>

**Bon**

Changer la clé en `methods` place `save()` sous l’option de composant reconnue.

```vue annotate="add:2"
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) · [Toutes les règles](all.md)

### `script/no-reactive-destructure`

Interdire la déstructuration d’objets réactifs qui fait perdre la réactivité

[Mauvais](#script-no-reactive-destructure-bad) · [Bon](#script-no-reactive-destructure-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`const { count, name } = state` copie les propriétés primitives hors de l’objet `reactive`, perdant leur lien avec les modifications ultérieures des propriétés.

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = state;
</script>
```

<span id="script-no-reactive-destructure-good"></span>

**Bon**

Déstructurer `toRefs(state)` crée des refs pour `count` et `name`, en maintenant chaque liaison reliée à la propriété réactive d’origine.

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRefs } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = toRefs(state);
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) · [Toutes les règles](all.md)

### `script/no-ref-as-operand`

Exiger l’accès via `.value` aux variables liées à une ref lorsqu’elles servent d’opérande

[Mauvais](#script-no-ref-as-operand-bad) · [Bon](#script-no-ref-as-operand-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`count + 1` utilise l’objet ref lui-même comme opérande arithmétique au lieu du nombre qu’il contient.

```vue annotate="remove:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count + 1;
</script>
```

<span id="script-no-ref-as-operand-good"></span>

**Bon**

`count.value + 1` lit le nombre contenu avant d’ajouter un ; l’arithmétique dans le script exige cet accès explicite à la ref.

```vue annotate="add:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count.value + 1;
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) · [Toutes les règles](all.md)

### `script/no-required-prop-with-default`

Interdire une prop qui possède à la fois required: true et une valeur par défaut

[Mauvais](#script-no-required-prop-with-default-bad) · [Bon](#script-no-required-prop-with-default-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`title` est à la fois obligatoire et doté de la valeur de repli `"Untitled"`, combinant un contrat d’entrée obligatoire avec une valeur par défaut prévue pour une entrée manquante.

```vue annotate="remove:2"
<script lang="ts">
export default { props: { title: { type: String, required: true, default: "Untitled" } } };
</script>
```

<span id="script-no-required-prop-with-default-good"></span>

**Bon**

Supprimer `required: true` rend `title` facultatif et conserve `"Untitled"` comme valeur de repli cohérente.

```vue annotate="add:2"
<script lang="ts">
export default { props: { title: { type: String, default: "Untitled" } } };
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L29) · [Toutes les règles](all.md)

### `script/no-reserved-identifiers`

Interdire les identifiants réservés du compilateur Vue

[Mauvais](#script-no-reserved-identifiers-bad) · [Bon](#script-no-reserved-identifiers-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les liaisons `__props`, `__emit` et `__sfc__` utilisent des identifiants réservés au code généré par le compilateur Vue.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const __props = { name: "Ada" };
const __emit = () => {};
const __sfc__ = {};
</script>
```

<span id="script-no-reserved-identifiers-good"></span>

**Bon**

Les noms ordinaires `props`, `emit` et `componentData` évitent ces identifiants générés tout en conservant les déclarations de props et d’événements émis.

```vue annotate="add:2,3,4"
<script setup lang="ts">
const props = defineProps<{ name: string }>();
const emit = defineEmits<{ save: [] }>();
const componentData = {};
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) · [Toutes les règles](all.md)

### `script/no-reserved-keys`

Interdire les noms réservés par Vue comme clés de props/data/computed/methods/setup/inject de l’Options API

[Mauvais](#script-no-reserved-keys-bad) · [Bon](#script-no-reserved-keys-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La clé de données renvoyée `$el` entre en conflit avec une propriété intégrée de l’instance du composant Vue et utilise également le préfixe réservé `$`.

```vue annotate="remove:2"
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

<span id="script-no-reserved-keys-good"></span>

**Bon**

Renommer les données de l’application en `elementLabel` évite l’API intégrée de l’instance et le préfixe réservé.

```vue annotate="add:2"
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L32) · [Toutes les règles](all.md)

### `script/no-reserved-props`

Interdire les noms réservés dans la déclaration des props d’un composant

[Mauvais](#script-no-reserved-props-bad) · [Bon](#script-no-reserved-props-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`ref` et `$foo` dans la forme objet, ainsi que `key` dans la forme tableau, sont des noms de props réservés. `ref` et `key` sont des mécanismes du framework, et les noms préfixés par `$` sont rejetés.

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

**Bon**

Les noms de props ordinaires `name` et `refValue` évitent les noms réservés tant par leur graphie que par leur préfixe.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) · [Toutes les règles](all.md)

### `script/no-restricted-globals`

Interdire les références aux variables globales de l’environnement d’exécution qui doivent passer par une couche d’encapsulation typée

[Mauvais](#script-no-restricted-globals-bad) · [Bon](#script-no-restricted-globals-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

L’exemple lit directement les variables globales restreintes par défaut `process`, `localStorage` et `sessionStorage`, en contournant les utilitaires explicites de configuration et de stockage du projet.

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const flag = process.env.FEATURE_FLAG
const token = localStorage.getItem('auth.token')
sessionStorage.setItem('view.scroll', String(window.scrollY))
</script>
```

<span id="script-no-restricted-globals-good"></span>

**Bon**

`useFeatureFlag`, `authStorage.read` et `viewStorage.write` suppriment ces références directes aux variables globales restreintes. L’accès restant à `window.scrollY` ne fait pas partie des restrictions par défaut de cette règle ; la sécurité SSR est une question distincte.

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
// Use a typed config helper that distinguishes server vs. client.
const flag = useFeatureFlag('FEATURE_FLAG')

// Use a typed wrapper that scopes keys and handles SSR / disabled storage.
const token = authStorage.read('auth.token')
viewStorage.write('view.scroll', String(window.scrollY))
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) · [Toutes les règles](all.md)

### `script/no-restricted-members`

Interdire les accès aux membres object.property configurés par le projet

[Mauvais](#script-no-restricted-members-bad) · [Bon](#script-no-restricted-members-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

Cet exemple configure window.localStorage. La règle n’a aucune liste d’interdiction par défaut ; son activation seule ne signale pas de membre.

**Configuration (Vite+)**

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

**Mauvais**

Lorsque `{ object: "window", property: "localStorage" }` est configuré dans `ruleOptions`, `window.localStorage` accède à la paire objet/membre interdite. Cette règle n’interdit aucun membre par défaut.

```vue annotate="remove:2"
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

<span id="script-no-restricted-members-good"></span>

**Bon**

`authStorage.read("token")` délègue la lecture à l’utilitaire de stockage de l’application et n’accède plus au membre configuré `window.localStorage`.

```vue annotate="add:2"
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) · [Toutes les règles](all.md)

### `script/no-side-effects-in-computed-properties`

Interdire les effets de bord dans les getters calculés de l’Options API

[Mauvais](#script-no-side-effects-in-computed-properties-bad) · [Bon](#script-no-side-effects-in-computed-properties-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`doubled` affecte une valeur à `this.count`, et `reversed` modifie `this.items` par `reverse()`. Les deux getters modifient l’état dont ils sont censés dériver leur valeur.

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

**Bon**

`doubled` renvoie le résultat de la multiplication sans affectation. `reversed` copie le tableau avant de l’inverser, de sorte que le getter ne modifie pas l’état d’origine du composant.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L70) · [Toutes les règles](all.md)

### `script/no-top-level-ref-in-script`

Interdire ref/reactive au niveau supérieur pour éviter la contamination de l’état entre requêtes

[Mauvais](#script-no-top-level-ref-in-script-bad) · [Bon](#script-no-top-level-ref-in-script-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le `<script>` ordinaire initialise `count` et `user` dans la portée du module. Pendant le SSR, ces objets d’état peuvent être partagés entre les instances du composant et les requêtes.

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

**Bon**

La ref de setup est initialisée pour chaque instance du composant ; le script ordinaire ne conserve qu’une constante, une fonction produisant de l’état et une ref créée dans `setup()`. Aucun ne crée d’état réactif dans la portée du module ordinaire.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) · [Toutes les règles](all.md)

### `script/no-unstable-nested-components`

Interdire les définitions de composants dans les fonctions setup ou de rendu

[Mauvais](#script-no-unstable-nested-components-bad) · [Bon](#script-no-unstable-nested-components-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`defineComponent` s’exécute dans le `setup()` du parent et crée une nouvelle définition du composant `Child` chaque fois que ce setup s’exécute.

```vue annotate="remove:3"
<script lang="ts">
import { defineComponent } from "vue";
export default { setup() { const Child = defineComponent({ render() { return null; } }); return { Child }; } };
</script>
```

<span id="script-no-unstable-nested-components-good"></span>

**Bon**

La définition de `Child` passe dans la portée du module, et `setup()` renvoie cette définition existante au lieu de la recréer.

```vue annotate="add:3,4"
<script lang="ts">
import { defineComponent } from "vue";
const Child = defineComponent({ render() { return null; } });
export default { setup() { return { Child }; } };
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) · [Toutes les règles](all.md)

### `script/no-unused-emit-declarations`

Signaler les événements déclarés qui ne sont jamais émis

[Mauvais](#script-no-unused-emit-declarations-bad) · [Bon](#script-no-unused-emit-declarations-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`defineEmits` déclare à la fois `change` et `unused`, mais la fonction `emit` récupérée n’émet que l’événement littéral `change`.

```vue annotate="remove:2,4"
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

<span id="script-no-unused-emit-declarations-good"></span>

**Bon**

Supprimer `unused` fait correspondre la liste des événements déclarés à l’émission observée. L’exemple utilise une liaison emit récupérée et non transmise à l’extérieur, ce qui permet cette conclusion sur son utilisation locale.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) · [Toutes les règles](all.md)

### `script/no-use-computed-property-like-method`

Interdire d’appeler une propriété calculée de l’Options API comme une méthode

[Mauvais](#script-no-use-computed-property-like-method-bad) · [Bon](#script-no-use-computed-property-like-method-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`this.total()` appelle la valeur exposée par le getter calculé ; ce getter renvoie `3`, qui ne peut pas être appelé.

```vue annotate="remove:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total()); } } };
</script>
```

<span id="script-no-use-computed-property-like-method-good"></span>

**Bon**

`this.total` lit la valeur calculée sans parenthèses d’appel, de sorte que `log` affiche le nombre dérivé.

```vue annotate="add:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total); } } };
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44) · [Toutes les règles](all.md)

### `script/no-with-defaults`

Déconseiller withDefaults au profit des valeurs par défaut dans la déstructuration (Vue 3.5+)

[Mauvais](#script-no-with-defaults-bad) · [Bon](#script-no-with-defaults-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`withDefaults` enveloppe la déclaration typée des props uniquement pour fournir les valeurs par défaut de `count` et `name`, au lieu du style de valeurs par défaut dans la déstructuration de Vue 3.5+ préféré ici.

```vue annotate="remove:2"
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

<span id="script-no-with-defaults-good"></span>

**Bon**

Le motif de déstructuration place `count = 0` et `name = "Ada"` à côté de leurs liaisons et supprime l’enveloppe `withDefaults`.

```vue annotate="add:2"
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) · [Toutes les règles](all.md)

### `script/prefer-computed`

Préférer computed() pour l’état réactif dérivé

[Mauvais](#script-prefer-computed-bad) · [Bon](#script-prefer-computed-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Le watcher doit uniquement dériver la destination. Les copies modifiables et les callbacks ayant d’autres effets de bord sont autorisés.

**Configuration (Vite+)**

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

**Mauvais**

Le watcher ne fait que copier une valeur dérivée de `count` dans une seconde ref, `doubled` ; l’état dérivé est donc maintenu par synchronisation manuelle.

```vue annotate="remove:2,4,5"
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
const doubled = ref(0);
watch(count, (value) => { doubled.value = value * 2; });
</script>
```

<span id="script-prefer-computed-good"></span>

**Bon**

`computed(() => count.value * 2)` exprime directement la dérivation et supprime à la fois la ref modifiable supplémentaire et le watcher qui la synchronise.

```vue annotate="add:2,4"
<script setup lang="ts">
import { ref, computed } from "vue";
const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) · [Toutes les règles](all.md)

### `script/prefer-define-options`

Préférer defineOptions() à un &lt;script&gt; ordinaire qui ne définit que name/inheritAttrs

[Mauvais](#script-prefer-define-options-bad) · [Bon](#script-prefer-define-options-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La seule instruction significative du script ordinaire exporte un objet contenant uniquement `name` et `inheritAttrs` ; ces options peuvent être exprimées par `defineOptions`.

```vue annotate="remove:2"
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

<span id="script-prefer-define-options-good"></span>

**Bon**

La méthode `data()` montrée donne au script une véritable logique de l’Options API ; il échappe donc à la suggestion prudente de cette règle, limitée aux options seules. Cet exemple Bon démontre une exception autorisée ; la migration directe placerait `defineOptions({ name: 'MyComponent', inheritAttrs: false })` dans `<script setup>`.

```vue annotate="add:2,3,4,5,6"
<script lang="ts">
// Real options logic — keep the plain script.
export default {
name: 'MyComponent',
data() { return { count: 0 } },
}
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) · [Toutes les règles](all.md)

### `script/prefer-import-from-vue`

Préférer les imports depuis 'vue' plutôt que depuis les packages internes

[Mauvais](#script-prefer-import-from-vue-bad) · [Bon](#script-prefer-import-from-vue-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`ref` et `h` sont importés depuis les packages internes `@vue/runtime-core` et `@vue/runtime-dom` plutôt que depuis le package public `vue`.

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

<span id="script-prefer-import-from-vue-good"></span>

**Bon**

Les deux utilitaires sont importés ensemble depuis `vue`, utilisant le point d’entrée public du package plutôt que l’un ou l’autre des packages internes.

```vue annotate="add:2"
<script setup lang="ts">
import { ref, h } from 'vue'
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L32) · [Toutes les règles](all.md)

### `script/prefer-ref-over-reactive`

Recommander ref() plutôt que reactive() pour gérer l’état

[Mauvais](#script-prefer-ref-over-reactive-bad) · [Bon](#script-prefer-ref-over-reactive-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’état est créé avec `reactive`, contrairement à la préférence de cette règle de convention pour les refs. L’exemple illustre une préférence de style ; il ne s’agit pas d’un objet réactif intrinsèquement invalide.

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

**Bon**

Les exemples créent aussi bien l’état scalaire que l’état objet avec `ref` ; les champs liés peuvent également être répartis dans des refs distinctes. Cela respecte la forme de création d’état préférée.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) · [Toutes les règles](all.md)

### `script/prefer-use-attrs`

Recommander useAttrs() plutôt que context.attrs

[Mauvais](#script-prefer-use-attrs-bad) · [Bon](#script-prefer-use-attrs-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`setup` obtient `attrs` en déstructurant son paramètre de contexte, ce que cette règle demande de remplacer par l’utilitaire de la Composition API.

```vue annotate="remove:2"
<script lang="ts">
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

<span id="script-prefer-use-attrs-good"></span>

**Bon**

`useAttrs()` fournit `attrs` dans setup, en conservant la lecture de `attrs.class` sans dépendre du second paramètre de setup.

```vue annotate="add:2,3"
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) · [Toutes les règles](all.md)

### `script/prefer-use-id`

Recommander useId() pour générer des identifiants uniques (Vue 3.5+)

[Mauvais](#script-prefer-use-id-bad) · [Bon](#script-prefer-use-id-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`id` contient `Math.random()` : l’identifiant généré pour l’input et le label peut donc différer entre les rendus serveur et client. Sa liaison dont le nom désigne un ID est le contexte de génération reconnu par la règle.

```vue annotate="remove:2"
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

<span id="script-prefer-use-id-good"></span>

**Bon**

`useId()` de Vue 3.5+ génère l’identifiant, et `:for` comme `:id` continuent de lire la même liaison au lieu de générer indépendamment des valeurs aléatoires.

```vue annotate="add:2,3"
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) · [Toutes les règles](all.md)

### `script/prefer-use-slots`

Recommander useSlots() plutôt que context.slots

[Mauvais](#script-prefer-use-slots-bad) · [Bon](#script-prefer-use-slots-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`setup` déstructure `slots` depuis son argument de contexte, la forme d’accès que cette règle préfère remplacer.

```vue annotate="remove:2,4"
<script lang="ts">
import { defineComponent, h } from "vue";
export default defineComponent({
  setup(_props, { slots }) { return () => h("div", slots.default?.()); },
});
</script>
```

<span id="script-prefer-use-slots-good"></span>

**Bon**

`useSlots()` récupère les slots dans setup, en conservant la fonction de rendu et son appel facultatif au slot par défaut sans paramètre de contexte.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) · [Toutes les règles](all.md)

### `script/prefer-use-template-ref`

Recommander useTemplateRef plutôt que ref pour les références de template (Vue 3.5+)

[Mauvais](#script-prefer-use-template-ref-bad) · [Bon](#script-prefer-use-template-ref-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La ref nullable `input` est associée au littéral `ref="input"` du template, ce qui l’identifie comme une référence d’élément plutôt que comme une donnée nullable ordinaire.

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

**Bon**

`useTemplateRef<HTMLInputElement>('input')` de Vue 3.5+ rend cette référence de template explicite. `error = ref(null)`, sans association, reste une donnée ordinaire et est volontairement hors du champ de cette règle.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75) · [Toutes les règles](all.md)

### `script/require-default-prop`

Exiger une valeur par défaut pour chaque prop facultative non booléenne

[Mauvais](#script-require-default-prop-bad) · [Bon](#script-require-default-prop-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`name` et `age` sont des props à l’exécution facultatives, non booléennes et sans valeur par défaut ; leurs valeurs en cas d’omission restent indéterminées.

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

**Bon**

`name` reçoit `default: ''`. `enabled` utilise la valeur false implicite de Boolean, et `id`, obligatoire, n’a pas besoin de valeur de repli, ce qui illustre les deux exemptions.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) · [Toutes les règles](all.md)

### `script/require-explicit-emits`

Exiger la déclaration des événements émis dans defineEmits ou l’option emits

[Mauvais](#script-require-explicit-emits-bad) · [Bon](#script-require-explicit-emits-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La fonction emit récupérée émet `save`, mais `defineEmits([])` ne déclare pas cet événement.

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits([]);
emit("save");
</script>
```

<span id="script-require-explicit-emits-good"></span>

**Bon**

Ajouter `"save"` à la déclaration intègre l’événement littéral émis au contrat explicite d’événements du composant.

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(["save"]);
emit("save");
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L61) · [Toutes les règles](all.md)

### `script/require-explicit-slots`

Exiger que les slots utilisés via useSlots() soient explicitement typés avec defineSlots&lt;...&gt;()

[Mauvais](#script-require-explicit-slots-bad) · [Bon](#script-require-explicit-slots-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La déclaration typée `defineProps<{ id: number }>()` établit une syntaxe TypeScript, mais setup utilise `useSlots()` sans déclaration `defineSlots`. La règle détecte donc des slots utilisés sans contrat de slots explicite.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ id: number }>()
const slots = useSlots()
</script>
```

<span id="script-require-explicit-slots-good"></span>

**Bon**

`defineSlots` déclare un slot `default` dont les props incluent `msg: string` ; `useSlots()` apparaît désormais aux côtés d’un contrat de slots explicitement typé.

```vue annotate="add:2"
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) · [Toutes les règles](all.md)

### `script/require-function-return-type`

Exiger des annotations de type de retour sur les fonctions

[Mauvais](#script-require-function-return-type-bad) · [Bon](#script-require-function-return-type-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`add` et `greet` annotent tous deux leurs paramètres mais omettent une annotation de type de retour ; les retours inférés ne satisfont pas cette convention d’annotation explicite.

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

**Bon**

`add` déclare `: number`, et `greet` déclare `: string`, rendant leurs contrats de retour explicites sans changer leurs corps.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L49) · [Toutes les règles](all.md)

### `script/require-prop-type-constructor`

Exiger que les valeurs `type` des props soient des constructeurs plutôt que des chaînes littérales

[Mauvais](#script-require-prop-type-constructor-bad) · [Bon](#script-require-prop-type-constructor-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les déclarations de props utilisent les chaînes `"String"` et `"Number"` comme types à l’exécution, y compris dans le tableau de constructeurs. Ces chaînes ne sont pas des fonctions constructeurs.

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

**Bon**

Les déclarations utilisent les véritables identifiants `String` et `Number`, y compris dans le tableau d’union `[String, Number]`.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L59) · [Toutes les règles](all.md)

### `script/require-prop-types`

Exiger que chaque prop déclare un type

[Mauvais](#script-require-prop-types-bad) · [Bon](#script-require-prop-types-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’entrée du tableau ne déclare que le nom `status` ; la valeur `null` et le descripteur vide ne déclarent pas non plus de type de prop à l’exécution.

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

**Bon**

`status: String` fournit un constructeur sous forme abrégée, et `other` fournit `type: Number` dans son descripteur. Les deux props possèdent désormais des déclarations de type.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58) · [Toutes les règles](all.md)

### `script/require-symbol-provide`

Recommander Symbol comme clé d’injection pour provide/inject

[Mauvais](#script-require-symbol-provide-bad) · [Bon](#script-require-symbol-provide-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`provide` et `inject` utilisent des clés sous forme de chaînes littérales telles que `'user'` et `'theme'`, qui peuvent entrer en conflit avec un autre fournisseur utilisant la même graphie.

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

**Bon**

La clé partagée `UserKey` est créée avec `Symbol` et annotée comme `InjectionKey<User>` ; les deux appels passent cette clé au lieu d’une chaîne littérale.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) · [Toutes les règles](all.md)

### `script/require-typed-object-prop`

Exiger un type explicite sur une prop dont le type à l’exécution est `Object` ou `Array`

[Mauvais](#script-require-typed-object-prop-bad) · [Bon](#script-require-typed-object-prop-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les constructeurs `Object` et `Array` employés seuls ne décrivent que des catégories générales à l’exécution ; ni `user` ni la structure des éléments de `items` n’a donc de type statique explicite.

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

<span id="script-require-typed-object-prop-good"></span>

**Bon**

`PropType<User>` et `PropType<User[]>` ajoutent les types de l’objet et des éléments tout en conservant les mêmes constructeurs à l’exécution.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) · [Toutes les règles](all.md)

### `script/require-typed-ref`

Exiger un argument de type explicite sur un ref() initialisé sans valeur, avec null ou avec undefined

[Mauvais](#script-require-typed-ref-bad) · [Bon](#script-require-typed-ref-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les appels à `ref` importé n’ont ni argument de type ni valeur initiale utile : l’absence d’argument, `null` et `undefined` ne permettent pas d’inférer le type de valeur futur souhaité.

```vue annotate="remove:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref()           // Ref<undefined>
const b = ref(null)       // Ref<null>
const c = ref(undefined)  // Ref<undefined>
</script>
```

<span id="script-require-typed-ref-good"></span>

**Bon**

Des arguments de type explicites décrivent les refs de chaîne et de User nullable. `ref(0)` possède déjà une valeur initiale numérique concrète et peut s’appuyer sur l’inférence.

```vue annotate="add:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref<string>()
const b = ref<User | null>(null)
const c = ref(0)          // inferred Ref<number>
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55) · [Toutes les règles](all.md)

### `script/require-valid-default-prop`

Exiger que la valeur par défaut d’une prop soit valide pour son type déclaré

[Mauvais](#script-require-valid-default-prop-bad) · [Bon](#script-require-valid-default-prop-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les props Number et Boolean reçoivent des valeurs scalaires par défaut incompatibles, et les props Array et Object utilisent des valeurs littérales partagées au lieu de fonctions de création.

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

**Bon**

Les valeurs scalaires par défaut deviennent `0` et `false` ; les valeurs par défaut du tableau et de l’objet deviennent des fonctions renvoyant de nouvelles valeurs. L’exemple `[String, Number]` accepte sa valeur par défaut de type chaîne, car elle correspond à l’un des types déclarés.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) · [Toutes les règles](all.md)

### `script/return-in-computed-property`

Exiger une valeur de retour dans chaque getter calculé

[Mauvais](#script-return-in-computed-property-bad) · [Bon](#script-return-in-computed-property-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le getter calculé dont le corps est un bloc évalue `1 + 2` mais ne le renvoie jamais, laissant la valeur calculée à undefined.

```vue annotate="remove:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { 1 + 2; });
</script>
```

<span id="script-return-in-computed-property-good"></span>

**Bon**

`return 1 + 2` transforme l’expression en valeur renvoyée par le getter. La règle recherche un return renvoyant une valeur dans le getter lui-même, et non une simple instruction d’expression.

```vue annotate="add:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { return 1 + 2; });
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) · [Toutes les règles](all.md)

### `script/return-in-emits-validator`

Exiger une valeur de retour dans chaque validateur emits de l’Options API

[Mauvais](#script-return-in-emits-validator-bad) · [Bon](#script-return-in-emits-validator-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Utilisez une fonction fléchée à corps de bloc pour le filtre SFC actuellement pris en charge. Le validateur sous-jacent gère aussi la syntaxe abrégée des méthodes, mais le préfiltre SFC actuel ne transmet pas cette forme de façon fiable.

**Configuration (Vite+)**

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

**Mauvais**

Le validateur `submit` journalise la charge utile mais ne renvoie aucun résultat de validation ; son corps de bloc produit donc undefined.

```vue annotate="remove:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

<span id="script-return-in-emits-validator-good"></span>

**Bon**

`return payload != null` fournit un résultat de validation booléen pour la charge utile soumise au lieu de se terminer sans valeur de retour.

```vue annotate="add:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L59) · [Toutes les règles](all.md)

### `script/valid-define-emits`

Imposer une utilisation valide de defineEmits() (pas d’arguments de type et d’exécution combinés, pas de références locales, un seul appel)

[Mauvais](#script-valid-define-emits-bad) · [Bon](#script-valid-define-emits-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le même appel `defineEmits` fournit à la fois un argument de type et le tableau à l’exécution `["save"]`, mélangeant deux déclarations mutuellement exclusives.

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

<span id="script-valid-define-emits-good"></span>

**Bon**

Supprimer l’argument d’exécution laisse une seule déclaration d’événement typée pour `save`.

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) · [Toutes les règles](all.md)

### `script/valid-define-options`

Imposer une utilisation valide de defineOptions() (un seul argument objet, sans props/emits/expose/slots)

[Mauvais](#script-valid-define-options-bad) · [Bon](#script-valid-define-options-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le premier appel place la déclaration dédiée `props` dans `defineOptions` ; les appels suivants répètent également la macro et incluent un argument qui n’est pas un objet. Ils illustrent les contraintes sur les formes interdites et les appels répétés.

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
defineOptions({ props: ['foo'] })   // use defineProps instead
defineOptions({ name: 'Foo' })
defineOptions({ name: 'Bar' })      // duplicate call
defineOptions('Foo')                // not an object literal
</script>
```

<span id="script-valid-define-options-good"></span>

**Bon**

Un seul appel `defineOptions` reçoit un objet contenant uniquement les options ordinaires prises en charge `name` et `inheritAttrs`.

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: 'Foo', inheritAttrs: false })
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) · [Toutes les règles](all.md)

### `script/valid-define-props`

Imposer une utilisation valide de defineProps() (un seul appel, pas d’arguments de type et d’exécution combinés, pas de références locales)

[Mauvais](#script-valid-define-props-bad) · [Bon](#script-valid-define-props-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le même appel `defineProps` fournit à la fois `{ title: string }` comme argument de type et `{ title: String }` comme argument d’exécution, ce que le compilateur n’autorise pas conjointement.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

<span id="script-valid-define-props-good"></span>

**Bon**

Supprimer l’objet d’exécution laisse une déclaration typée unique pour `title` au lieu de combiner les deux formes de déclaration.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) · [Toutes les règles](all.md)

### `script/valid-next-tick`

Exiger que le résultat d’un appel nextTick() soit attendu, chaîné ou associé à un callback

[Mauvais](#script-valid-next-tick-bad) · [Bon](#script-valid-next-tick-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’appel à `nextTick()` importé est une expression seule sans callback ; la Promise renvoyée est donc ignorée et aucun traitement n’attend l’application des mises à jour du DOM.

```vue annotate="remove:3"
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

<span id="script-valid-next-tick-good"></span>

**Bon**

`await nextTick()` consomme la Promise et attend explicitement la prochaine mise à jour du DOM avant que le code setup suivant ne continue.

```vue annotate="add:3"
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) · [Toutes les règles](all.md)

### `type/no-floating-promises`

Interdire les Promises laissées sans traitement

[Mauvais](#type-no-floating-promises-bad) · [Bon](#type-no-floating-promises-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Les contrôles utilisant les types s’appuient sur le runtime natif Corsa et le projet TypeScript. `typeAware` seul n’active pas une règle nécessitant une activation explicite.

**Configuration (Vite+)**

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

**Mauvais**

La fonction asynchrone `save` renvoie une Promise, mais l’appel isolé `save()` ne l’attend ni ne la renvoie, et ne marque pas explicitement son abandon volontaire.

```vue annotate="remove:3"
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

<span id="type-no-floating-promises-good"></span>

**Bon**

`void save()` marque explicitement l’intention de lancer l’opération sans attendre son résultat, acceptée par cette règle. Il s’agit d’un marqueur explicite d’abandon, et non d’un gestionnaire de rejet.

```vue annotate="add:3"
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) · [Toutes les règles](all.md)

### `type/no-reactivity-loss`

Interdire les instantanés ordinaires de valeurs réactives lors des affectations et des appels

[Mauvais](#type-no-reactivity-loss-bad) · [Bon](#type-no-reactivity-loss-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Les contrôles utilisant les types s’appuient sur le runtime natif Corsa et le projet TypeScript. `typeAware` seul n’active pas une règle nécessitant une activation explicite.

**Configuration (Vite+)**

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

**Mauvais**

`const count = state.count` prend un simple instantané numérique de la propriété réactive ; les mises à jour ultérieures de `state.count` ne sont donc pas répercutées dans cette liaison.

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0 });
const count = state.count;
</script>
```

<span id="type-no-reactivity-loss-good"></span>

**Bon**

`toRef(state, "count")` maintient `count` relié à la propriété réactive d’origine plutôt que de copier sa valeur primitive actuelle.

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRef } from "vue";
const state = reactive({ count: 0 });
const count = toRef(state, "count");
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) · [Toutes les règles](all.md)

### `type/no-unsafe-template-binding`

Interdire les liaisons de template dont le type résolu est non sûr

[Mauvais](#type-no-unsafe-template-binding-bad) · [Bon](#type-no-unsafe-template-binding-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Les contrôles utilisant les types s’appuient sur le runtime natif Corsa et le projet TypeScript. `typeAware` seul n’active pas une règle nécessitant une activation explicite.

**Configuration (Vite+)**

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

**Mauvais**

La valeur interpolée `value` est explicitement typée comme `any` ; le vérificateur ne peut donc pas attribuer à la liaison du template un type concret sûr.

```vue annotate="remove:2"
<script setup lang="ts">
const value: any = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

<span id="type-no-unsafe-template-binding-good"></span>

**Bon**

Changer l’annotation en `string` donne à la même interpolation un type concret vérifiable sans changer la valeur affichée.

```vue annotate="add:2"
<script setup lang="ts">
const value: string = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) · [Toutes les règles](all.md)

### `type/require-typed-emits`

Exiger une définition de type pour defineEmits

[Mauvais](#type-require-typed-emits-bad) · [Bon](#type-require-typed-emits-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Les contrôles utilisant les types s’appuient sur le runtime natif Corsa et le projet TypeScript. `typeAware` seul n’active pas une règle nécessitant une activation explicite.

**Configuration (Vite+)**

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

**Mauvais**

La déclaration uniquement sous forme de tableau `defineEmits(["save"])` déclare le nom de l’événement sans contrat typé pour ses arguments.

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

<span id="type-require-typed-emits-good"></span>

**Bon**

`defineEmits<{ save: [] }>()` déclare l’événement typé `save` avec un tuple d’arguments vide, indiquant explicitement qu’il n’accepte aucun argument.

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) · [Toutes les règles](all.md)

### `type/require-typed-props`

Exiger une définition de type pour defineProps

[Mauvais](#type-require-typed-props-bad) · [Bon](#type-require-typed-props-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Les contrôles utilisant les types s’appuient sur le runtime natif Corsa et le projet TypeScript. `typeAware` seul n’active pas une règle nécessitant une activation explicite.

**Configuration (Vite+)**

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

**Mauvais**

La déclaration uniquement sous forme de tableau `defineProps(["title"])` déclare `title` par son nom sans lui donner de type.

```vue annotate="remove:2"
<script setup lang="ts">
defineProps(["title"]);
</script>
```

<span id="type-require-typed-props-good"></span>

**Bon**

`defineProps<{ title: string }>()` donne à `title` un type chaîne explicite au lieu d’une déclaration à l’exécution limitée à son nom.

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57) · [Toutes les règles](all.md)

### `type/strict-boolean-expressions`

Exiger des expressions booléennes sûres dans les conditions des scripts et des templates

[Mauvais](#type-strict-boolean-expressions-bad) · [Bon](#type-strict-boolean-expressions-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

Activez explicitement typeAware et cette règle. Par défaut, les nombres pouvant être nuls sont interdits, tandis que les nombres non nuls sont autorisés.

Les contrôles utilisant les types s’appuient sur le runtime natif Corsa et le projet TypeScript. `typeAware` seul n’active pas une règle nécessitant une activation explicite.

**Configuration (Vite+)**

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

**Mauvais**

`if (count)` repose sur la conversion implicite en booléen d’une liaison numérique nullable plutôt que sur un test booléen explicite ; cela confond aussi zéro avec l’absence de valeur.

```vue annotate="remove:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

<span id="type-strict-boolean-expressions-good"></span>

**Bon**

`count !== undefined && count > 0` teste séparément la présence et la positivité, produisant une condition booléenne explicite après avoir affiné le type de la valeur optionnelle.

```vue annotate="add:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) · [Toutes les règles](all.md)
