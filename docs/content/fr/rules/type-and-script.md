---
title: "Règles de type et de script"
description: "Vérifications de `<script>` et `<script setup>` : macros du compilateur, réactivité, props et emits, et règles qui utilisent les informations de type de TypeScript."
---

# Règles de type et de script

Vérifications de `<script>` et `<script setup>` : macros du compilateur, réactivité, props et emits, et règles qui utilisent les informations de type de TypeScript.

Chaque page de règle montre un exemple qui échoue, un exemple qui passe et la configuration qui active la règle.

## Essentielles (39)

Règles de justesse du préréglage `essential`.

| Règle | Ce qu’elle vérifie | Correction automatique |
| --- | --- | --- |
| <span id="script-no-arrow-functions-in-watch"></span>[`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | Interdire les fonctions fléchées comme gestionnaires watch de l’Options API. |  |
| <span id="script-no-async-in-computed"></span>[`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | Interdire les fonctions asynchrones dans les propriétés calculées. |  |
| <span id="script-no-computed-properties-in-data"></span>[`script/no-computed-properties-in-data`](./reference/script-no-computed-properties-in-data.md) | Interdire l’usage d’une propriété calculée dans la fonction data du composant. |  |
| <span id="script-no-deprecated-data-object-declaration"></span>[`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | Interdire un objet littéral comme option data du composant (Vue 3 exige une fonction). |  |
| <span id="script-no-deprecated-delete-set"></span>[`script/no-deprecated-delete-set`](./reference/script-no-deprecated-delete-set.md) | Interdire $set et $delete de Vue 2, retirés dans Vue 3. |  |
| <span id="script-no-deprecated-destroyed-lifecycle"></span>[`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | Interdire les hooks de cycle de vie dépréciés destroyed et beforeDestroy. | ✓ |
| <span id="script-no-deprecated-dollar-listeners-api"></span>[`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | Interdire la propriété d’instance $listeners supprimée dans Vue 3 (fusionnée dans $attrs). |  |
| <span id="script-no-deprecated-dollar-scopedslots-api"></span>[`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | Interdire la propriété d’instance $scopedSlots supprimée dans Vue 3 (utiliser $slots). |  |
| <span id="script-no-deprecated-events-api"></span>[`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | Interdire l’API d’événements de Vue 2 supprimée ($on / $off / $once). |  |
| <span id="script-no-deprecated-model-definition"></span>[`script/no-deprecated-model-definition`](./reference/script-no-deprecated-model-definition.md) | Interdire l’option model de Vue 2, retirée dans Vue 3. |  |
| <span id="script-no-deprecated-props-default-this"></span>[`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | Interdire `this` dans une fonction de valeur par défaut ou de validation de prop (supprimé dans Vue 3). |  |
| <span id="script-no-deprecated-vue-config-keycodes"></span>[`script/no-deprecated-vue-config-keycodes`](./reference/script-no-deprecated-vue-config-keycodes.md) | Interdire Vue.config.keyCodes, retiré dans Vue 3. |  |
| <span id="script-no-dupe-keys"></span>[`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | Interdire les clés dupliquées entre props/data/computed/methods/setup/inject de l’Options API. |  |
| <span id="script-no-duplicate-attr-inheritance"></span>[`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | Signaler un composant qui applique deux fois ses attributs transmis automatiquement. |  |
| <span id="script-no-export-in-script-setup"></span>[`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | Interdire les instructions export dans &lt;script setup&gt;. |  |
| <span id="script-no-expose-after-await"></span>[`script/no-expose-after-await`](./reference/script-no-expose-after-await.md) | Interdire expose() et defineExpose() après un await. |  |
| <span id="script-no-import-compiler-macros"></span>[`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | Interdire l’import des macros du compilateur Vue importées automatiquement. |  |
| <span id="script-no-internal-imports"></span>[`script/no-internal-imports`](./reference/script-no-internal-imports.md) | Interdire les imports depuis les modules internes de Vue. |  |
| <span id="script-no-lifecycle-after-await"></span>[`script/no-lifecycle-after-await`](./reference/script-no-lifecycle-after-await.md) | Interdire l’enregistrement de hooks de cycle de vie après un await dans setup(). |  |
| <span id="script-no-potential-component-option-typo"></span>[`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | Signaler les fautes de frappe probables dans les noms d’options de composant de l’Options API. |  |
| <span id="script-no-ref-as-operand"></span>[`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | Exiger l’accès via `.value` aux variables liées à une ref lorsqu’elles servent d’opérande. |  |
| <span id="script-no-required-prop-with-default"></span>[`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | Interdire une prop qui possède à la fois required: true et une valeur par défaut. |  |
| <span id="script-no-reserved-identifiers"></span>[`script/no-reserved-identifiers`](./reference/script-no-reserved-identifiers.md) | Interdire les identifiants réservés du compilateur Vue. |  |
| <span id="script-no-reserved-keys"></span>[`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | Interdire les noms réservés par Vue comme clés de props/data/computed/methods/setup/inject de l’Options API. |  |
| <span id="script-no-reserved-props"></span>[`script/no-reserved-props`](./reference/script-no-reserved-props.md) | Interdire les noms réservés dans la déclaration des props d’un composant. |  |
| <span id="script-no-side-effects-in-computed-properties"></span>[`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | Interdire les effets de bord dans les getters calculés de l’Options API. |  |
| <span id="script-no-use-computed-property-like-method"></span>[`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | Interdire d’appeler une propriété calculée de l’Options API comme une méthode. |  |
| <span id="script-no-watch-after-await"></span>[`script/no-watch-after-await`](./reference/script-no-watch-after-await.md) | Interdire les observateurs créés après un await dans setup() sans conserver leur fonction d’arrêt. |  |
| <span id="script-prefer-import-from-vue"></span>[`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | Préférer les imports depuis 'vue' plutôt que depuis les packages internes. | ✓ |
| <span id="script-require-prop-type-constructor"></span>[`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | Exiger que les valeurs `type` des props soient des constructeurs plutôt que des chaînes littérales. |  |
| <span id="script-require-render-return"></span>[`script/require-render-return`](./reference/script-require-render-return.md) | Exiger que la fonction render du composant renvoie une valeur. |  |
| <span id="script-require-slots-as-functions"></span>[`script/require-slots-as-functions`](./reference/script-require-slots-as-functions.md) | Exiger que les propriétés de $slots soient appelées comme des fonctions. |  |
| <span id="script-require-valid-default-prop"></span>[`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | Exiger que la valeur par défaut d’une prop soit valide pour son type déclaré. |  |
| <span id="script-return-in-computed-property"></span>[`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | Exiger une valeur de retour dans chaque getter calculé. |  |
| <span id="script-return-in-emits-validator"></span>[`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | Exiger une valeur de retour dans chaque validateur emits de l’Options API. |  |
| <span id="script-valid-define-emits"></span>[`script/valid-define-emits`](./reference/script-valid-define-emits.md) | Imposer une utilisation valide de defineEmits() (pas d’arguments de type et d’exécution combinés, pas de références locales, un seul appel). |  |
| <span id="script-valid-define-options"></span>[`script/valid-define-options`](./reference/script-valid-define-options.md) | Imposer une utilisation valide de defineOptions() (un seul argument objet, sans props/emits/expose/slots). |  |
| <span id="script-valid-define-props"></span>[`script/valid-define-props`](./reference/script-valid-define-props.md) | Imposer une utilisation valide de defineProps() (un seul appel, pas d’arguments de type et d’exécution combinés, pas de références locales). |  |
| <span id="script-valid-next-tick"></span>[`script/valid-next-tick`](./reference/script-valid-next-tick.md) | Exiger que le résultat d’un appel nextTick() soit attendu, chaîné ou associé à un callback. |  |

## Par défaut (6)

Ajoutées par `happy-path`, le préréglage par défaut.

| Règle | Ce qu’elle vérifie | Correction automatique |
| --- | --- | --- |
| <span id="script-no-multiple-slot-args"></span>[`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | Interdire de passer plusieurs arguments à un appel de fonction de slot à portée. |  |
| <span id="script-no-unstable-nested-components"></span>[`script/no-unstable-nested-components`](./reference/script-no-unstable-nested-components.md) | Interdire les définitions de composants dans les fonctions setup ou de rendu. |  |
| <span id="script-require-define-component"></span>[`script/require-define-component`](./reference/script-require-define-component.md) | Exiger defineComponent() pour les exports par défaut des SFC TypeScript. |  |
| <span id="script-require-explicit-emits"></span>[`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | Exiger la déclaration des événements émis dans defineEmits ou l’option emits. |  |
| <span id="type-require-typed-emits"></span>[`type/require-typed-emits`](./reference/type-require-typed-emits.md) | Exiger une définition de type pour defineEmits. |  |
| <span id="type-require-typed-props"></span>[`type/require-typed-props`](./reference/type-require-typed-props.md) | Exiger une définition de type pour defineProps. |  |

## Préréglages plus stricts (22)

Seulement dans `opinionated`, `nuxt` ou `ecosystem`. La colonne Préréglages indique lesquels.

| Règle | Ce qu’elle vérifie | Préréglages | Correction automatique |
| --- | --- | --- | --- |
| <span id="script-component-options-name-casing"></span>[`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | Imposer PascalCase à l’option `name` du composant. | `opinionated`, `nuxt` |  |
| <span id="script-custom-event-name-casing"></span>[`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | Imposer la casse configurée aux noms des événements personnalisés émis. | `opinionated`, `nuxt` |  |
| <span id="script-define-emits-declaration"></span>[`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | Imposer la forme typée defineEmits&lt;{}&gt;() à la place de la forme à l’exécution sous forme de tableau. | `opinionated`, `nuxt` |  |
| <span id="script-define-macros-order"></span>[`script/define-macros-order`](./reference/script-define-macros-order.md) | Imposer un ordre cohérent aux macros du compilateur Vue dans &lt;script setup&gt;. | `opinionated`, `nuxt` |  |
| <span id="script-define-props-declaration"></span>[`script/define-props-declaration`](./reference/script-define-props-declaration.md) | Imposer la forme typée defineProps&lt;{ ... }&gt;() à la place de la forme à l’exécution sous forme d’objet. | `opinionated`, `nuxt` |  |
| <span id="script-no-boolean-default"></span>[`script/no-boolean-default`](./reference/script-no-boolean-default.md) | Interdire une valeur par défaut sur une prop Boolean. | `opinionated`, `nuxt` |  |
| <span id="script-no-deep-destructure-in-props"></span>[`script/no-deep-destructure-in-props`](./reference/script-no-deep-destructure-in-props.md) | Interdire la déstructuration profondément imbriquée dans defineProps. | `opinionated`, `nuxt` |  |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Interdire getCurrentInstance() en mode Vapor (renvoie null). | `opinionated` |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | Interdire les formes de l’Options API en mode Vapor. | `opinionated` |  |
| <span id="script-no-unused-emit-declarations"></span>[`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | Signaler les événements déclarés qui ne sont jamais émis. | `opinionated`, `nuxt` |  |
| <span id="script-no-with-defaults"></span>[`script/no-with-defaults`](./reference/script-no-with-defaults.md) | Déconseiller withDefaults au profit des valeurs par défaut dans la déstructuration (Vue 3.5+). | `opinionated` |  |
| <span id="script-one-component-per-file"></span>[`script/one-component-per-file`](./reference/script-one-component-per-file.md) | Interdire plus d’une définition de composant dans un fichier. | `opinionated`, `nuxt` |  |
| <span id="script-order-in-components"></span>[`script/order-in-components`](./reference/script-order-in-components.md) | Imposer l’ordre par défaut des propriétés des options de composant. | `opinionated`, `nuxt` |  |
| <span id="script-prefer-define-options"></span>[`script/prefer-define-options`](./reference/script-prefer-define-options.md) | Préférer defineOptions() à un &lt;script&gt; ordinaire qui ne définit que name/inheritAttrs. | `opinionated`, `nuxt` |  |
| <span id="script-prefer-use-template-ref"></span>[`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | Recommander useTemplateRef plutôt que ref pour les références de template (Vue 3.5+). | `opinionated`, `nuxt` |  |
| <span id="script-require-default-prop"></span>[`script/require-default-prop`](./reference/script-require-default-prop.md) | Exiger une valeur par défaut pour chaque prop facultative non booléenne. | `opinionated`, `nuxt` |  |
| <span id="script-require-explicit-slots"></span>[`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | Exiger que les slots utilisés via useSlots() soient explicitement typés avec defineSlots&lt;...&gt;(). | `opinionated`, `nuxt` |  |
| <span id="script-require-prop-types"></span>[`script/require-prop-types`](./reference/script-require-prop-types.md) | Exiger que chaque prop déclare un type. | `opinionated` |  |
| <span id="script-require-typed-object-prop"></span>[`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | Exiger un type explicite sur une prop dont le type à l’exécution est `Object` ou `Array`. | `opinionated`, `nuxt` |  |
| <span id="type-no-floating-promises"></span>[`type/no-floating-promises`](./reference/type-no-floating-promises.md) | Interdire les Promises laissées sans traitement. | `nuxt`, `opinionated` |  |
| <span id="type-no-reactivity-loss"></span>[`type/no-reactivity-loss`](./reference/type-no-reactivity-loss.md) | Interdire les instantanés ordinaires de valeurs réactives lors des affectations et des appels. | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-template-binding"></span>[`type/no-unsafe-template-binding`](./reference/type-no-unsafe-template-binding.md) | Interdire les liaisons de template dont le type résolu est non sûr. | `nuxt`, `opinionated` |  |

## À activer (16)

Dans aucun préréglage. Activez-les par leur nom dans `linter.rules`, passé à `withVue`.

| Règle | Ce qu’elle vérifie | Correction automatique |
| --- | --- | --- |
| <span id="script-define-props-destructuring"></span>[`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | Imposer un style cohérent de déstructuration de defineProps dans &lt;script setup&gt;. |  |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | Interdire l’utilisation de nextTick() dans les composants destinés à Vapor. |  |
| <span id="script-no-reactive-destructure"></span>[`script/no-reactive-destructure`](./reference/script-no-reactive-destructure.md) | Interdire la déstructuration d’objets réactifs qui fait perdre la réactivité. |  |
| <span id="script-no-restricted-globals"></span>[`script/no-restricted-globals`](./reference/script-no-restricted-globals.md) | Interdire les références aux variables globales de l’environnement d’exécution qui doivent passer par une couche d’encapsulation typée. |  |
| <span id="script-no-restricted-members"></span>[`script/no-restricted-members`](./reference/script-no-restricted-members.md) | Interdire les accès aux membres object.property configurés par le projet. |  |
| <span id="script-no-top-level-ref-in-script"></span>[`script/no-top-level-ref-in-script`](./reference/script-no-top-level-ref-in-script.md) | Interdire ref/reactive au niveau supérieur pour éviter la contamination de l’état entre requêtes. |  |
| <span id="script-prefer-computed"></span>[`script/prefer-computed`](./reference/script-prefer-computed.md) | Préférer computed() pour l’état réactif dérivé. |  |
| <span id="script-prefer-define-emits-binding"></span>[`script/prefer-define-emits-binding`](./reference/script-prefer-define-emits-binding.md) | Préférer la fonction capturée de defineEmits au $emit du template. | ✓ |
| <span id="script-prefer-ref-over-reactive"></span>[`script/prefer-ref-over-reactive`](./reference/script-prefer-ref-over-reactive.md) | Recommander ref() plutôt que reactive() pour gérer l’état. |  |
| <span id="script-prefer-use-attrs"></span>[`script/prefer-use-attrs`](./reference/script-prefer-use-attrs.md) | Recommander useAttrs() plutôt que context.attrs. |  |
| <span id="script-prefer-use-id"></span>[`script/prefer-use-id`](./reference/script-prefer-use-id.md) | Recommander useId() pour générer des identifiants uniques (Vue 3.5+). |  |
| <span id="script-prefer-use-slots"></span>[`script/prefer-use-slots`](./reference/script-prefer-use-slots.md) | Recommander useSlots() plutôt que context.slots. |  |
| <span id="script-require-function-return-type"></span>[`script/require-function-return-type`](./reference/script-require-function-return-type.md) | Exiger des annotations de type de retour sur les fonctions. |  |
| <span id="script-require-symbol-provide"></span>[`script/require-symbol-provide`](./reference/script-require-symbol-provide.md) | Recommander Symbol comme clé d’injection pour provide/inject. |  |
| <span id="script-require-typed-ref"></span>[`script/require-typed-ref`](./reference/script-require-typed-ref.md) | Exiger un argument de type explicite sur un ref() initialisé sans valeur, avec null ou avec undefined. |  |
| <span id="type-strict-boolean-expressions"></span>[`type/strict-boolean-expressions`](./reference/type-strict-boolean-expressions.md) | Exiger des expressions booléennes sûres dans les conditions des scripts et des templates. |  |

[Vue d’ensemble des règles](../rules/index.md) · [Options des règles](/rules/options.md) · [Migrer depuis ESLint](/rules/migration.md)
