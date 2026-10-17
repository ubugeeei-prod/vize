---
title: "Toutes les règles Patina"
---

# Toutes les règles Patina

Cet index permet de trouver l'objectif d'une règle, ses réglages par défaut et son groupe. Les pages des groupes présentent les explications, la configuration, les exemples complets incorrects et corrects ainsi que les limites actuelles. Les liens existants vers les règles et leurs exemples restent utilisables ici.

Les 66 entrées de projet renvoient aux fichiers partagés complets et aux exemples incorrects et corrects de la page des règles inter-fichiers. Les 60 codes d'analyse conservent leurs limites réelles : 19 codes CLI (18 paires de sources validées et un projet illustratif avec son graphe réactif conservé), 16 codes expérimentaux de l'analyseur Rust que cette passe CLI n'émet pas individuellement et 25 contrats sans producteur actuel. Configurer un identifiant n'active pas un producteur indisponible.

Le CLI public expose la même passe avec `vize lint --cross-file`. Les codes affichés `vize:croquis/cf/*` utilisent `croquis/cf/*` dans `vue.linter.rules` (sans `vize:`). Les diagnostics de niveau information/hint deviennent des avertissements CLI. Les emplacements associés expliquent la relation entre la source et le consommateur.

<span id="toutes-les-règles-de-patine"></span>
<span id="catégories"></span>
<span id="essentiel-48"></span>
<span id="fortement-recommandé-12"></span>
<span id="recommandé-42"></span>
<span id="accessibilité-31"></span>
<span id="conformité-html-9"></span>
<span id="type-aware-5"></span>
<span id="vapeur-7"></span>
<span id="écosystème-9"></span>
<span id="css-10"></span>
<span id="musea-6"></span>
<span id="script-60"></span>
<span id="essential-48"></span>
<span id="strongly-recommended-12"></span>
<span id="recommended-42"></span>
<span id="accessibility-31"></span>
<span id="html-conformance-9"></span>
<span id="vapor-7"></span>
<span id="ecosystem-9"></span>

| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`a11y/alt-text`](#a11y-alt-text) | [Mauvais](#a11y-alt-text-bad) · [Bon](#a11y-alt-text-good) | Exiger un texte alternatif pour les éléments multimédias |
| [`a11y/anchor-has-content`](#a11y-anchor-has-content) | [Mauvais](#a11y-anchor-has-content-bad) · [Bon](#a11y-anchor-has-content-good) | Exiger un contenu accessible pour les éléments de lien |
| [`a11y/anchor-is-valid`](#a11y-anchor-is-valid) | [Mauvais](#a11y-anchor-is-valid-bad) · [Bon](#a11y-anchor-is-valid-good) | Imposer un href valide sur les éléments de lien |
| [`a11y/aria-props`](#a11y-aria-props) | [Mauvais](#a11y-aria-props-bad) · [Bon](#a11y-aria-props-good) | Interdire les attributs ARIA invalides |
| [`a11y/aria-role`](#a11y-aria-role) | [Mauvais](#a11y-aria-role-bad) · [Bon](#a11y-aria-role-good) | Exiger un rôle ARIA valide et non abstrait pour les éléments possédant un rôle ARIA |
| [`a11y/aria-unsupported-elements`](#a11y-aria-unsupported-elements) | [Mauvais](#a11y-aria-unsupported-elements-bad) · [Bon](#a11y-aria-unsupported-elements-good) | Interdire les attributs ARIA sur les éléments qui ne les prennent pas en charge |
| [`a11y/click-events-have-key-events`](#a11y-click-events-have-key-events) | [Mauvais](#a11y-click-events-have-key-events-bad) · [Bon](#a11y-click-events-have-key-events-good) | Exiger des gestionnaires de clavier avec les événements de clic |
| [`a11y/form-control-has-label`](#a11y-form-control-has-label) | [Mauvais](#a11y-form-control-has-label-bad) · [Bon](#a11y-form-control-has-label-good) | Exiger des libellés associés aux contrôles de formulaire |
| [`a11y/heading-has-content`](#a11y-heading-has-content) | [Mauvais](#a11y-heading-has-content-bad) · [Bon](#a11y-heading-has-content-good) | Exiger un contenu accessible pour les éléments de titre |
| [`a11y/heading-levels`](#a11y-heading-levels) | [Mauvais](#a11y-heading-levels-bad) · [Bon](#a11y-heading-levels-good) | Interdire de sauter des niveaux de titre |
| [`a11y/iframe-has-title`](#a11y-iframe-has-title) | [Mauvais](#a11y-iframe-has-title-bad) · [Bon](#a11y-iframe-has-title-good) | Exiger un attribut title sur les éléments iframe |
| [`a11y/img-alt`](#a11y-img-alt) | [Mauvais](#a11y-img-alt-bad) · [Bon](#a11y-img-alt-good) | Exiger un attribut alt sur les images pour les rendre accessibles |
| [`a11y/interactive-supports-focus`](#a11y-interactive-supports-focus) | [Mauvais](#a11y-interactive-supports-focus-bad) · [Bon](#a11y-interactive-supports-focus-good) | Exiger que les éléments possédant un rôle interactif puissent recevoir le focus |
| [`a11y/label-has-for`](#a11y-label-has-for) | [Mauvais](#a11y-label-has-for-bad) · [Bon](#a11y-label-has-for-good) | Exiger des contrôles de formulaire associés aux libellés |
| [`a11y/landmark-roles`](#a11y-landmark-roles) | [Mauvais](#a11y-landmark-roles-bad) · [Bon](#a11y-landmark-roles-good) | Valider l’emplacement et l’unicité des rôles de zones de repère |
| [`a11y/media-has-caption`](#a11y-media-has-caption) | [Mauvais](#a11y-media-has-caption-bad) · [Bon](#a11y-media-has-caption-good) | Exiger des sous-titres pour les éléments multimédias |
| [`a11y/mouse-events-have-key-events`](#a11y-mouse-events-have-key-events) | [Mauvais](#a11y-mouse-events-have-key-events-bad) · [Bon](#a11y-mouse-events-have-key-events-good) | Exiger des événements de focus et de perte du focus avec les événements de souris |
| [`a11y/no-access-key`](#a11y-no-access-key) | [Mauvais](#a11y-no-access-key-bad) · [Bon](#a11y-no-access-key-good) | Interdire l’utilisation de l’attribut accesskey |
| [`a11y/no-aria-hidden-on-focusable`](#a11y-no-aria-hidden-on-focusable) | [Mauvais](#a11y-no-aria-hidden-on-focusable-bad) · [Bon](#a11y-no-aria-hidden-on-focusable-good) | Interdire aria-hidden="true" sur les éléments pouvant recevoir le focus |
| [`a11y/no-autofocus`](#a11y-no-autofocus) | [Mauvais](#a11y-no-autofocus-bad) · [Bon](#a11y-no-autofocus-good) | Interdire l’utilisation de l’attribut autofocus |
| [`a11y/no-distracting-elements`](#a11y-no-distracting-elements) | [Mauvais](#a11y-no-distracting-elements-bad) · [Bon](#a11y-no-distracting-elements-good) | Interdire les éléments distrayants tels que &lt;marquee&gt; et &lt;blink&gt; |
| [`a11y/no-i-for-icon`](#a11y-no-i-for-icon) | [Mauvais](#a11y-no-i-for-icon-bad) · [Bon](#a11y-no-i-for-icon-good) | Interdire l’élément &lt;i&gt; pour les icônes |
| [`a11y/no-redundant-roles`](#a11y-no-redundant-roles) | [Mauvais](#a11y-no-redundant-roles-bad) · [Bon](#a11y-no-redundant-roles-good) | Interdire les rôles ARIA redondants |
| [`a11y/no-refer-to-non-existent-id`](#a11y-no-refer-to-non-existent-id) | [Mauvais](#a11y-no-refer-to-non-existent-id-bad) · [Bon](#a11y-no-refer-to-non-existent-id-good) | Interdire les références à des identifiants inexistants |
| [`a11y/no-role-presentation-on-focusable`](#a11y-no-role-presentation-on-focusable) | [Mauvais](#a11y-no-role-presentation-on-focusable-bad) · [Bon](#a11y-no-role-presentation-on-focusable-good) | Interdire role="presentation" ou role="none" sur les éléments pouvant recevoir le focus |
| [`a11y/no-static-element-interactions`](#a11y-no-static-element-interactions) | [Mauvais](#a11y-no-static-element-interactions-bad) · [Bon](#a11y-no-static-element-interactions-good) | Interdire les gestionnaires d’événements sur les éléments statiques |
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [Mauvais](#a11y-placeholder-label-option-bad) · [Bon](#a11y-placeholder-label-option-good) | Exiger disabled ou hidden sur l’option d’invite d’un select |
| [`a11y/role-has-required-aria-props`](#a11y-role-has-required-aria-props) | [Mauvais](#a11y-role-has-required-aria-props-bad) · [Bon](#a11y-role-has-required-aria-props-good) | Exiger les propriétés obligatoires des rôles ARIA |
| [`a11y/tabindex-no-positive`](#a11y-tabindex-no-positive) | [Mauvais](#a11y-tabindex-no-positive-bad) · [Bon](#a11y-tabindex-no-positive-good) | Interdire les valeurs positives de tabindex |
| [`a11y/use-list`](#a11y-use-list) | [Mauvais](#a11y-use-list-bad) · [Bon](#a11y-use-list-good) | Suggérer des éléments de liste pour les textes ressemblant à des listes à puces |
| [`css/no-display-none`](#css-no-display-none) | [Mauvais](#css-no-display-none-bad) · [Bon](#css-no-display-none-good) | Suggérer v-show plutôt que display: none |
| [`css/no-hardcoded-values`](#css-no-hardcoded-values) | [Mauvais](#css-no-hardcoded-values-bad) · [Bon](#css-no-hardcoded-values-good) | Suggérer des variables CSS plutôt que des valeurs codées en dur |
| [`css/no-id-selectors`](#css-no-id-selectors) | [Mauvais](#css-no-id-selectors-bad) · [Bon](#css-no-id-selectors-good) | Déconseiller les sélecteurs d’identifiant en CSS |
| [`css/no-important`](#css-no-important) | [Mauvais](#css-no-important-bad) · [Bon](#css-no-important-good) | Déconseiller !important en CSS |
| [`css/no-utility-classes`](#css-no-utility-classes) | [Mauvais](#css-no-utility-classes-bad) · [Bon](#css-no-utility-classes-good) | Déconseiller l’implémentation de classes utilitaires dans les styles des composants |
| [`css/no-v-bind-performance`](#css-no-v-bind-performance) | [Mauvais](#css-no-v-bind-performance-bad) · [Bon](#css-no-v-bind-performance-good) | Signaler le coût de performance de v-bind() en CSS |
| [`css/prefer-logical-properties`](#css-prefer-logical-properties) | [Mauvais](#css-prefer-logical-properties-bad) · [Bon](#css-prefer-logical-properties-good) | Recommander les propriétés logiques CSS pour mieux prendre en charge l’internationalisation |
| [`css/prefer-nested-selectors`](#css-prefer-nested-selectors) | [Mauvais](#css-prefer-nested-selectors-bad) · [Bon](#css-prefer-nested-selectors-good) | Recommander l’imbrication CSS pour les sélecteurs de descendants |
| [`css/prefer-slotted`](#css-prefer-slotted) | [Mauvais](#css-prefer-slotted-bad) · [Bon](#css-prefer-slotted-good) | Recommander ::v-slotted() pour styliser le contenu des slots |
| [`css/require-font-display`](#css-require-font-display) | [Mauvais](#css-require-font-display-bad) · [Bon](#css-require-font-display-good) | Exiger font-display dans les règles @font-face |
| [`ecosystem/nuxt-prefer-nuxt-link`](#ecosystem-nuxt-prefer-nuxt-link) | [Mauvais](#ecosystem-nuxt-prefer-nuxt-link-bad) · [Bon](#ecosystem-nuxt-prefer-nuxt-link-good) | Préférer NuxtLink pour les liens internes à l’application |
| [`ecosystem/pinia-prefer-store-to-refs`](#ecosystem-pinia-prefer-store-to-refs) | [Mauvais](#ecosystem-pinia-prefer-store-to-refs-bad) · [Bon](#ecosystem-pinia-prefer-store-to-refs-good) | Préférer storeToRefs() lors de la déstructuration des stores Pinia |
| [`ecosystem/router-link-require-to`](#ecosystem-router-link-require-to) | [Mauvais](#ecosystem-router-link-require-to-bad) · [Bon](#ecosystem-router-link-require-to-good) | Exiger une destination `to` sur les composants RouterLink et NuxtLink |
| [`ecosystem/void-link-require-href`](#ecosystem-void-link-require-href) | [Mauvais](#ecosystem-void-link-require-href-bad) · [Bon](#ecosystem-void-link-require-href-good) | Exiger `href` sur les composants Link de Void Vue |
| [`ecosystem/void-link-valid-method`](#ecosystem-void-link-valid-method) | [Mauvais](#ecosystem-void-link-valid-method-bad) · [Bon](#ecosystem-void-link-valid-method-good) | Valider les props method statiques des composants Link de Void Vue |
| [`ecosystem/vue-i18n-no-missing-key`](#ecosystem-vue-i18n-no-missing-key) | [Mauvais](#ecosystem-vue-i18n-no-missing-key-bad) · [Bon](#ecosystem-vue-i18n-no-missing-key-good) | Signaler les clés vue-i18n statiques absentes des messages locaux du SFC |
| [`ecosystem/vue-router-extra-param`](#ecosystem-vue-router-extra-param) | [Mauvais](#ecosystem-vue-router-extra-param-bad) · [Bon](#ecosystem-vue-router-extra-param-good) | La route ne déclare pas tab ; Vue Router l’ignore. |
| [`ecosystem/vue-router-missing-param`](#ecosystem-vue-router-missing-param) | [Mauvais](#ecosystem-vue-router-missing-param-bad) · [Bon](#ecosystem-vue-router-missing-param-good) | Le paramètre obligatoire postId manque ; dépendre de la route actuelle est fragile. |
| [`ecosystem/vue-router-param-type`](#ecosystem-vue-router-param-type) | [Mauvais](#ecosystem-vue-router-param-type-bad) · [Bon](#ecosystem-vue-router-param-type-good) | postId n’est pas répétable ; un tableau est donc invalide. |
| [`ecosystem/vue-router-prefer-named-link`](#ecosystem-vue-router-prefer-named-link) | [Mauvais](#ecosystem-vue-router-prefer-named-link-bad) · [Bon](#ecosystem-vue-router-prefer-named-link-good) | Préférer les objets de routes nommées aux chaînes de chemin statiques dans RouterLink |
| [`ecosystem/vue-router-prefer-named-push`](#ecosystem-vue-router-prefer-named-push) | [Mauvais](#ecosystem-vue-router-prefer-named-push-bad) · [Bon](#ecosystem-vue-router-prefer-named-push-good) | Préférer les objets de routes nommées pour la navigation programmatique de Vue Router |
| [`ecosystem/vue-router-unknown-route`](#ecosystem-vue-router-unknown-route) | [Mauvais](#ecosystem-vue-router-unknown-route-bad) · [Bon](#ecosystem-vue-router-unknown-route-good) | Le nom est absent du routeur installé complet. |
| [`ecosystem/vue-test-utils-no-html-snapshot`](#ecosystem-vue-test-utils-no-html-snapshot) | [Mauvais](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Bon](#ecosystem-vue-test-utils-no-html-snapshot-good) | Éviter les instantanés de wrapper.html() dans les tests Vue Test Utils |
| [`html/cross-component-nesting`](#html-cross-component-nesting) | [Mauvais](#html-cross-component-nesting-bad) · [Bon](#html-cross-component-nesting-good) | Vérifier l’imbrication HTML réelle après la composition des composants importés. |
| [`html/deprecated-attr`](#html-deprecated-attr) | [Mauvais](#html-deprecated-attr-bad) · [Bon](#html-deprecated-attr-good) | Interdire les attributs HTML obsolètes |
| [`html/deprecated-element`](#html-deprecated-element) | [Mauvais](#html-deprecated-element-bad) · [Bon](#html-deprecated-element-good) | Interdire les éléments HTML obsolètes |
| [`html/id-duplication`](#html-id-duplication) | [Mauvais](#html-id-duplication-bad) · [Bon](#html-id-duplication-good) | Interdire les identifiants d’éléments en double |
| [`html/no-consecutive-br`](#html-no-consecutive-br) | [Mauvais](#html-no-consecutive-br-bad) · [Bon](#html-no-consecutive-br-good) | Interdire les éléments &lt;br&gt; consécutifs |
| [`html/no-dupe-style-properties`](#html-no-dupe-style-properties) | [Mauvais](#html-no-dupe-style-properties-bad) · [Bon](#html-no-dupe-style-properties-good) | Interdire les propriétés en double dans les attributs de style en ligne |
| [`html/no-duplicate-class`](#html-no-duplicate-class) | [Mauvais](#html-no-duplicate-class-bad) · [Bon](#html-no-duplicate-class-good) | Interdire les noms de classes en double dans un attribut class statique |
| [`html/no-duplicate-dt`](#html-no-duplicate-dt) | [Mauvais](#html-no-duplicate-dt-bad) · [Bon](#html-no-duplicate-dt-good) | Interdire les noms &lt;dt&gt; en double dans &lt;dl&gt; |
| [`html/no-empty-palpable-content`](#html-no-empty-palpable-content) | [Mauvais](#html-no-empty-palpable-content-bad) · [Bon](#html-no-empty-palpable-content-good) | Interdire les éléments vides qui attendent un contenu visible |
| [`html/require-datetime`](#html-require-datetime) | [Mauvais](#html-require-datetime-bad) · [Bon](#html-require-datetime-good) | Exiger l’attribut datetime sur l’élément &lt;time&gt; |
| [`musea/no-empty-variant`](#musea-no-empty-variant) | [Mauvais](#musea-no-empty-variant-bad) · [Bon](#musea-no-empty-variant-good) | Interdire les blocs &lt;variant&gt; vides |
| [`musea/prefer-design-tokens`](#musea-prefer-design-tokens) | [Mauvais](#musea-prefer-design-tokens-bad) · [Bon](#musea-prefer-design-tokens-good) | Préférer les variables CSS de design tokens aux valeurs primitives codées en dur |
| [`musea/require-component`](#musea-require-component) | [Mauvais](#musea-require-component-bad) · [Bon](#musea-require-component-good) | Exiger l’attribut component dans le bloc &lt;art&gt; |
| [`musea/require-title`](#musea-require-title) | [Mauvais](#musea-require-title-bad) · [Bon](#musea-require-title-good) | Exiger l’attribut title dans le bloc &lt;art&gt; |
| [`musea/unique-variant-names`](#musea-unique-variant-names) | [Mauvais](#musea-unique-variant-names-bad) · [Bon](#musea-unique-variant-names-good) | Exiger des noms de variants uniques |
| [`musea/valid-variant`](#musea-valid-variant) | [Mauvais](#musea-valid-variant-bad) · [Bon](#musea-valid-variant-good) | Exiger un attribut name dans les blocs &lt;variant&gt; |
| [`nuxt/no-nuxt-config-test-key`](#nuxt-no-nuxt-config-test-key) | [Mauvais](#nuxt-no-nuxt-config-test-key-bad) · [Bon](#nuxt-no-nuxt-config-test-key-good) | Interdire la clé `test` dans la configuration Nuxt |
| [`nuxt/no-page-meta-runtime-values`](#nuxt-no-page-meta-runtime-values) | [Mauvais](#nuxt-no-page-meta-runtime-values-bad) · [Bon](#nuxt-no-page-meta-runtime-values-good) | Interdire les valeurs du contexte d’exécution évaluées immédiatement dans `definePageMeta`, extrait dans un chunk séparé à la compilation et exécuté avant le setup du composant |
| [`nuxt/nuxt-config-keys-order`](#nuxt-nuxt-config-keys-order) | [Mauvais](#nuxt-nuxt-config-keys-order-bad) · [Bon](#nuxt-nuxt-config-keys-order-good) | Préférer l’ordre recommandé des propriétés de configuration Nuxt |
| [`nuxt/prefer-import-meta`](#nuxt-prefer-import-meta) | [Mauvais](#nuxt-prefer-import-meta-bad) · [Bon](#nuxt-prefer-import-meta-good) | Préférer `import.meta.*` à `process.*` |
| [`petite-vue/no-unsupported-directive`](#petite-vue-no-unsupported-directive) | [Mauvais](#petite-vue-no-unsupported-directive-bad) · [Bon](#petite-vue-no-unsupported-directive-good) | Interdire les directives non prises en charge par petite-vue |
| [`petite-vue/valid-v-effect`](#petite-vue-valid-v-effect) | [Mauvais](#petite-vue-valid-v-effect-bad) · [Bon](#petite-vue-valid-v-effect-good) | Exiger une expression non vide pour v-effect |
| [`petite-vue/valid-v-scope`](#petite-vue-valid-v-scope) | [Mauvais](#petite-vue-valid-v-scope-bad) · [Bon](#petite-vue-valid-v-scope-good) | Exiger que v-scope reçoive un objet littéral |
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
| [`ssr/no-browser-globals-in-ssr`](#ssr-no-browser-globals-in-ssr) | [Mauvais](#ssr-no-browser-globals-in-ssr-bad) · [Bon](#ssr-no-browser-globals-in-ssr-good) | Interdire les variables globales propres au navigateur dans un contexte SSR |
| [`ssr/no-hydration-mismatch`](#ssr-no-hydration-mismatch) | [Mauvais](#ssr-no-hydration-mismatch-bad) · [Bon](#ssr-no-hydration-mismatch-good) | Interdire les valeurs non déterministes qui causent des divergences d’hydratation |
| [`type/no-floating-promises`](#type-no-floating-promises) | [Mauvais](#type-no-floating-promises-bad) · [Bon](#type-no-floating-promises-good) | Interdire les Promises laissées sans traitement |
| [`type/no-reactivity-loss`](#type-no-reactivity-loss) | [Mauvais](#type-no-reactivity-loss-bad) · [Bon](#type-no-reactivity-loss-good) | Interdire les instantanés ordinaires de valeurs réactives lors des affectations et des appels |
| [`type/no-unsafe-template-binding`](#type-no-unsafe-template-binding) | [Mauvais](#type-no-unsafe-template-binding-bad) · [Bon](#type-no-unsafe-template-binding-good) | Interdire les liaisons de template dont le type résolu est non sûr |
| [`type/require-typed-emits`](#type-require-typed-emits) | [Mauvais](#type-require-typed-emits-bad) · [Bon](#type-require-typed-emits-good) | Exiger une définition de type pour defineEmits |
| [`type/require-typed-props`](#type-require-typed-props) | [Mauvais](#type-require-typed-props-bad) · [Bon](#type-require-typed-props-good) | Exiger une définition de type pour defineProps |
| [`type/strict-boolean-expressions`](#type-strict-boolean-expressions) | [Mauvais](#type-strict-boolean-expressions-bad) · [Bon](#type-strict-boolean-expressions-good) | Exiger des expressions booléennes sûres dans les conditions des scripts et des templates |
| [`vapor/no-inline-template`](#vapor-no-inline-template) | [Mauvais](#vapor-no-inline-template-bad) · [Bon](#vapor-no-inline-template-good) | Interdire l’attribut obsolète inline-template |
| [`vapor/no-vue-lifecycle-events`](#vapor-no-vue-lifecycle-events) | [Mauvais](#vapor-no-vue-lifecycle-events-bad) · [Bon](#vapor-no-vue-lifecycle-events-good) | Interdire les événements de cycle de vie @vue:xxx par élément (non pris en charge dans Vapor) |
| [`vapor/prefer-static-class`](#vapor-prefer-static-class) | [Mauvais](#vapor-prefer-static-class-bad) · [Bon](#vapor-prefer-static-class-good) | Préférer une classe statique à une liaison de classe dynamique pour les chaînes littérales |
| [`vapor/require-vapor-attribute`](#vapor-require-vapor-attribute) | [Mauvais](#vapor-require-vapor-attribute-bad) · [Bon](#vapor-require-vapor-attribute-good) | Suggérer l’ajout de l’attribut vapor à script setup |
| [`vize:croquis/cf/array-mutation`](#vize-croquis-cf-array-mutation) | [Mauvais](#vize-croquis-cf-array-mutation-bad) · [Bon](#vize-croquis-cf-array-mutation-good) | Un tableau est modifié par index, ce qu’un tableau réactif ne suit pas. |
| [`vize:croquis/cf/async-boundary`](#vize-croquis-cf-async-boundary) | [Mauvais](#vize-croquis-cf-async-boundary-bad) · [Bon](#vize-croquis-cf-async-boundary-good) | Un état réactif traverse une frontière asynchrone et peut être observé dans un état périmé. |
| [`vize:croquis/cf/async-no-suspense`](#vize-croquis-cf-async-no-suspense) | [Mauvais](#vize-croquis-cf-async-no-suspense-bad) · [Bon](#vize-croquis-cf-async-no-suspense-good) | Un composant asynchrone est rendu sans frontière Suspense. |
| [`vize:croquis/cf/browser-api-ssr`](#vize-croquis-cf-browser-api-ssr) | [Mauvais](#vize-croquis-cf-browser-api-ssr-bad) · [Bon](#vize-croquis-cf-browser-api-ssr-good) | Une API réservée au navigateur est utilisée alors que le composant peut être rendu sur le serveur. |
| [`vize:croquis/cf/circular-dep`](#vize-croquis-cf-circular-dep) | [Mauvais](#vize-croquis-cf-circular-dep-bad) · [Bon](#vize-croquis-cf-circular-dep-good) | Des composants s’importent mutuellement en formant un cycle. |
| [`vize:croquis/cf/circular-reactive-dependency`](#vize-croquis-cf-circular-reactive-dependency) | [Mauvais](#vize-croquis-cf-circular-reactive-dependency-bad) · [Bon](#vize-croquis-cf-circular-reactive-dependency-good) | Des calculs réactifs dépendent les uns des autres en formant un cycle. |
| [`vize:croquis/cf/closure-captures-reactive`](#vize-croquis-cf-closure-captures-reactive) | [Mauvais](#vize-croquis-cf-closure-captures-reactive-bad) · [Bon](#vize-croquis-cf-closure-captures-reactive-good) | Une fermeture capture une valeur réactive et ne verra pas les mises à jour ultérieures. |
| [`vize:croquis/cf/composable-outside-setup`](#vize-croquis-cf-composable-outside-setup) | [Mauvais](#vize-croquis-cf-composable-outside-setup-bad) · [Bon](#vize-croquis-cf-composable-outside-setup-good) | Un composable est appelé en dehors de `setup`. |
| [`vize:croquis/cf/computed-side-effects`](#vize-croquis-cf-computed-side-effects) | [Mauvais](#vize-croquis-cf-computed-side-effects-bad) · [Bon](#vize-croquis-cf-computed-side-effects-good) | L’accesseur d’une propriété calculée écrit dans l’état ou produit un autre effet de bord. |
| [`vize:croquis/cf/deep-import`](#vize-croquis-cf-deep-import) | [Mauvais](#vize-croquis-cf-deep-import-bad) · [Bon](#vize-croquis-cf-deep-import-good) | Une chaîne d’imports est plus profonde que ce que le projet autorise. |
| [`vize:croquis/cf/destructuring-breaks-reactivity`](#vize-croquis-cf-destructuring-breaks-reactivity) | [Mauvais](#vize-croquis-cf-destructuring-breaks-reactivity-bad) · [Bon](#vize-croquis-cf-destructuring-breaks-reactivity-good) | La déstructuration d’un objet réactif copie ses champs et supprime leur suivi. |
| [`vize:croquis/cf/di-outside-setup`](#vize-croquis-cf-di-outside-setup) | [Mauvais](#vize-croquis-cf-di-outside-setup-bad) · [Bon](#vize-croquis-cf-di-outside-setup-good) | `provide` ou `inject` est appelé en dehors de `setup`. |
| [`vize:croquis/cf/dom-access-without-next-tick`](#vize-croquis-cf-dom-access-without-next-tick) | [Mauvais](#vize-croquis-cf-dom-access-without-next-tick-bad) · [Bon](#vize-croquis-cf-dom-access-without-next-tick-good) | Le DOM est lu avant que Vue ait appliqué la mise à jour. |
| [`vize:croquis/cf/duplicate-id`](#vize-croquis-cf-duplicate-id) | [Mauvais](#vize-croquis-cf-duplicate-id-bad) · [Bon](#vize-croquis-cf-duplicate-id-good) | Le même identifiant d’élément est utilisé dans plusieurs composants. |
| [`vize:croquis/cf/event-listener-leak`](#vize-croquis-cf-event-listener-leak) | [Mauvais](#vize-croquis-cf-event-listener-leak-bad) · [Bon](#vize-croquis-cf-event-listener-leak-good) | Un écouteur d’événement est enregistré sans jamais être supprimé. |
| [`vize:croquis/cf/event-modifier`](#vize-croquis-cf-event-modifier) | [Mauvais](#vize-croquis-cf-event-modifier-bad) · [Bon](#vize-croquis-cf-event-modifier-good) | Un écouteur d’événement utilise un modificateur que l’événement émis ne prend pas en charge. |
| [`vize:croquis/cf/hydration-risk`](#vize-croquis-cf-hydration-risk) | [Mauvais](#vize-croquis-cf-hydration-risk-bad) · [Bon](#vize-croquis-cf-hydration-risk-good) | Ce code regroupe plusieurs diagnostics de réactivité, dont une prop copiée dans une ref. Il ne signifie pas que chaque expression Date.now() est détectée par la passe inter-fichiers. |
| [`vize:croquis/cf/inherit-attrs-unused`](#vize-croquis-cf-inherit-attrs-unused) | [Mauvais](#vize-croquis-cf-inherit-attrs-unused-bad) · [Bon](#vize-croquis-cf-inherit-attrs-unused-good) | `inheritAttrs: false` est défini et le composant ne lit jamais les attributs. |
| [`vize:croquis/cf/inject-without-symbol`](#vize-croquis-cf-inject-without-symbol) | [Mauvais](#vize-croquis-cf-inject-without-symbol-bad) · [Bon](#vize-croquis-cf-inject-without-symbol-good) | `inject` utilise une clé ordinaire au lieu d’un symbole `InjectionKey`. |
| [`vize:croquis/cf/injected-async-mutation-race`](#vize-croquis-cf-injected-async-mutation-race) | [Mauvais](#vize-croquis-cf-injected-async-mutation-race-bad) · [Bon](#vize-croquis-cf-injected-async-mutation-race-good) | Une valeur injectée est modifiée par une tâche asynchrone susceptible de provoquer une condition de concurrence. |
| [`vize:croquis/cf/lifecycle-outside-setup`](#vize-croquis-cf-lifecycle-outside-setup) | [Mauvais](#vize-croquis-cf-lifecycle-outside-setup-bad) · [Bon](#vize-croquis-cf-lifecycle-outside-setup-good) | Un hook de cycle de vie est enregistré en dehors de `setup`. |
| [`vize:croquis/cf/lifecycle-without-cleanup`](#vize-croquis-cf-lifecycle-without-cleanup) | [Mauvais](#vize-croquis-cf-lifecycle-without-cleanup-bad) · [Bon](#vize-croquis-cf-lifecycle-without-cleanup-good) | Un hook de cycle de vie lance un travail sans jamais le nettoyer. |
| [`vize:croquis/cf/missing-required-prop`](#vize-croquis-cf-missing-required-prop) | [Mauvais](#vize-croquis-cf-missing-required-prop-bad) · [Bon](#vize-croquis-cf-missing-required-prop-good) | Une prop obligatoire n’est pas transmise. |
| [`vize:croquis/cf/missing-suspense`](#vize-croquis-cf-missing-suspense) | [Mauvais](#vize-croquis-cf-missing-suspense-bad) · [Bon](#vize-croquis-cf-missing-suspense-good) | Une dépendance asynchrone est utilisée en dehors d’une frontière Suspense. |
| [`vize:croquis/cf/module-scope-reactive`](#vize-croquis-cf-module-scope-reactive) | [Mauvais](#vize-croquis-cf-module-scope-reactive-bad) · [Bon](#vize-croquis-cf-module-scope-reactive-good) | Un état réactif est créé au niveau du module et partagé par tous les appelants. |
| [`vize:croquis/cf/multi-root-attrs`](#vize-croquis-cf-multi-root-attrs) | [Mauvais](#vize-croquis-cf-multi-root-attrs-bad) · [Bon](#vize-croquis-cf-multi-root-attrs-good) | Un composant à plusieurs racines reçoit des attributs sans avoir d’endroit où les placer. |
| [`vize:croquis/cf/mutated-after-escape`](#vize-croquis-cf-mutated-after-escape) | [Mauvais](#vize-croquis-cf-mutated-after-escape-bad) · [Bon](#vize-croquis-cf-mutated-after-escape-good) | Un objet réactif est modifié après avoir échappé à son propriétaire. |
| [`vize:croquis/cf/non-reactive-provide`](#vize-croquis-cf-non-reactive-provide) | [Mauvais](#vize-croquis-cf-non-reactive-provide-bad) · [Bon](#vize-croquis-cf-non-reactive-provide-good) | Une valeur fournie n’est pas réactive ; les descendants ne verront donc pas les mises à jour. |
| [`vize:croquis/cf/non-unique-id`](#vize-croquis-cf-non-unique-id) | [Mauvais](#vize-croquis-cf-non-unique-id-bad) · [Bon](#vize-croquis-cf-non-unique-id-good) | L’identifiant d’un élément dans une boucle n’est pas unique pour chaque entrée. |
| [`vize:croquis/cf/object-identity-comparison`](#vize-croquis-cf-object-identity-comparison) | [Mauvais](#vize-croquis-cf-object-identity-comparison-bad) · [Bon](#vize-croquis-cf-object-identity-comparison-good) | Un objet réactif est comparé par identité, laquelle change lorsque l’enveloppe est retirée. |
| [`vize:croquis/cf/pinia-getter`](#vize-croquis-cf-pinia-getter) | [Mauvais](#vize-croquis-cf-pinia-getter-bad) · [Bon](#vize-croquis-cf-pinia-getter-good) | Un getter Pinia est lu sans `storeToRefs` ; il ne restera donc pas réactif. |
| [`vize:croquis/cf/prop-type-mismatch`](#vize-croquis-cf-prop-type-mismatch) | [Mauvais](#vize-croquis-cf-prop-type-mismatch-bad) · [Bon](#vize-croquis-cf-prop-type-mismatch-good) | La valeur d’une prop transmise ne correspond pas au type déclaré. |
| [`vize:croquis/cf/provide-inject-type`](#vize-croquis-cf-provide-inject-type) | [Mauvais](#vize-croquis-cf-provide-inject-type-bad) · [Bon](#vize-croquis-cf-provide-inject-type-good) | Une valeur fournie et son injection n’ont pas le même type. |
| [`vize:croquis/cf/provide-without-symbol`](#vize-croquis-cf-provide-without-symbol) | [Mauvais](#vize-croquis-cf-provide-without-symbol-bad) · [Bon](#vize-croquis-cf-provide-without-symbol-good) | `provide` utilise une clé ordinaire au lieu d’un symbole `InjectionKey`. |
| [`vize:croquis/cf/reactive-export`](#vize-croquis-cf-reactive-export) | [Mauvais](#vize-croquis-cf-reactive-export-bad) · [Bon](#vize-croquis-cf-reactive-export-good) | Un état réactif est exporté depuis le module. |
| [`vize:croquis/cf/reactivity-outside-setup`](#vize-croquis-cf-reactivity-outside-setup) | [Mauvais](#vize-croquis-cf-reactivity-outside-setup-bad) · [Bon](#vize-croquis-cf-reactivity-outside-setup-good) | Une API réactive est appelée en dehors de `setup`. |
| [`vize:croquis/cf/reassignment-breaks-reactivity`](#vize-croquis-cf-reassignment-breaks-reactivity) | [Mauvais](#vize-croquis-cf-reassignment-breaks-reactivity-bad) · [Bon](#vize-croquis-cf-reassignment-breaks-reactivity-good) | Réaffecter une liaison réactive la remplace par une valeur ordinaire. |
| [`vize:croquis/cf/reference-escapes-scope`](#vize-croquis-cf-reference-escapes-scope) | [Mauvais](#vize-croquis-cf-reference-escapes-scope-bad) · [Bon](#vize-croquis-cf-reference-escapes-scope-good) | Une référence réactive échappe à la portée qui gère sa durée de vie. |
| [`vize:croquis/cf/setup-context-violation`](#vize-croquis-cf-setup-context-violation) | [Mauvais](#vize-croquis-cf-setup-context-violation-bad) · [Bon](#vize-croquis-cf-setup-context-violation-good) | Le contexte setup est utilisé d’une manière que Vue n’autorise pas. |
| [`vize:croquis/cf/shallow-deep-access`](#vize-croquis-cf-shallow-deep-access) | [Mauvais](#vize-croquis-cf-shallow-deep-access-bad) · [Bon](#vize-croquis-cf-shallow-deep-access-good) | Une propriété profonde d’une valeur `shallowReactive` ou `shallowRef` est lue comme si elle était suivie. |
| [`vize:croquis/cf/spread-breaks-reactivity`](#vize-croquis-cf-spread-breaks-reactivity) | [Mauvais](#vize-croquis-cf-spread-breaks-reactivity-bad) · [Bon](#vize-croquis-cf-spread-breaks-reactivity-good) | Décomposer un objet réactif avec l’opérateur spread copie ses valeurs et supprime leur suivi. |
| [`vize:croquis/cf/suspense-no-fallback`](#vize-croquis-cf-suspense-no-fallback) | [Mauvais](#vize-croquis-cf-suspense-no-fallback-bad) · [Bon](#vize-croquis-cf-suspense-no-fallback-good) | `<Suspense>` n’a aucun contenu de secours. |
| [`vize:croquis/cf/template-ref-timing`](#vize-croquis-cf-template-ref-timing) | [Mauvais](#vize-croquis-cf-template-ref-timing-bad) · [Bon](#vize-croquis-cf-template-ref-timing-good) | Une référence de template est lue avant le montage du composant. |
| [`vize:croquis/cf/toraw-mutation`](#vize-croquis-cf-toraw-mutation) | [Mauvais](#vize-croquis-cf-toraw-mutation-bad) · [Bon](#vize-croquis-cf-toraw-mutation-good) | `toRaw` est utilisé, puis l’objet brut est modifié. |
| [`vize:croquis/cf/uncaught-error`](#vize-croquis-cf-uncaught-error) | [Mauvais](#vize-croquis-cf-uncaught-error-bad) · [Bon](#vize-croquis-cf-uncaught-error-good) | Un composant peut lever une exception sans qu’aucune frontière d’erreur ne la capture. |
| [`vize:croquis/cf/undeclared-emit`](#vize-croquis-cf-undeclared-emit) | [Mauvais](#vize-croquis-cf-undeclared-emit-bad) · [Bon](#vize-croquis-cf-undeclared-emit-good) | Le composant émet un événement qui n’est pas déclaré. |
| [`vize:croquis/cf/undeclared-prop`](#vize-croquis-cf-undeclared-prop) | [Mauvais](#vize-croquis-cf-undeclared-prop-bad) · [Bon](#vize-croquis-cf-undeclared-prop-good) | Un parent transmet une prop que l’enfant ne déclare pas. |
| [`vize:croquis/cf/undefined-slot`](#vize-croquis-cf-undefined-slot) | [Mauvais](#vize-croquis-cf-undefined-slot-bad) · [Bon](#vize-croquis-cf-undefined-slot-good) | Un parent remplit un slot que l’enfant n’expose pas. |
| [`vize:croquis/cf/unhandled-event`](#vize-croquis-cf-unhandled-event) | [Mauvais](#vize-croquis-cf-unhandled-event-bad) · [Bon](#vize-croquis-cf-unhandled-event-good) | Un enfant émet un événement qu’aucun parent ne gère. |
| [`vize:croquis/cf/unmatched-inject`](#vize-croquis-cf-unmatched-inject) | [Mauvais](#vize-croquis-cf-unmatched-inject-bad) · [Bon](#vize-croquis-cf-unmatched-inject-good) | `inject` nomme une clé qu’aucun ancêtre ne fournit. |
| [`vize:croquis/cf/unmatched-listener`](#vize-croquis-cf-unmatched-listener) | [Mauvais](#vize-croquis-cf-unmatched-listener-bad) · [Bon](#vize-croquis-cf-unmatched-listener-good) | Un parent écoute un événement que l’enfant n’émet pas. |
| [`vize:croquis/cf/unregistered-component`](#vize-croquis-cf-unregistered-component) | [Mauvais](#vize-croquis-cf-unregistered-component-bad) · [Bon](#vize-croquis-cf-unregistered-component-good) | Un template utilise un composant qui n’est ni enregistré ni importé. |
| [`vize:croquis/cf/unresolved-import`](#vize-croquis-cf-unresolved-import) | [Mauvais](#vize-croquis-cf-unresolved-import-bad) · [Bon](#vize-croquis-cf-unresolved-import-good) | Un import ne se résout pas vers un module. |
| [`vize:croquis/cf/unused-attrs`](#vize-croquis-cf-unused-attrs) | [Mauvais](#vize-croquis-cf-unused-attrs-bad) · [Bon](#vize-croquis-cf-unused-attrs-good) | Des attributs à transmettre automatiquement sont passés à un composant à plusieurs racines qui ne les utilise pas. |
| [`vize:croquis/cf/unused-emit`](#vize-croquis-cf-unused-emit) | [Mauvais](#vize-croquis-cf-unused-emit-bad) · [Bon](#vize-croquis-cf-unused-emit-good) | Un événement déclaré n’est jamais émis. |
| [`vize:croquis/cf/unused-provide`](#vize-croquis-cf-unused-provide) | [Mauvais](#vize-croquis-cf-unused-provide-bad) · [Bon](#vize-croquis-cf-unused-provide-good) | Une clé fournie n’est jamais injectée. |
| [`vize:croquis/cf/value-extraction-breaks-reactivity`](#vize-croquis-cf-value-extraction-breaks-reactivity) | [Mauvais](#vize-croquis-cf-value-extraction-breaks-reactivity-bad) · [Bon](#vize-croquis-cf-value-extraction-breaks-reactivity-good) | Extraire une valeur réactive dans une variable locale fait perdre les mises à jour ultérieures. |
| [`vize:croquis/cf/watch-can-be-computed`](#vize-croquis-cf-watch-can-be-computed) | [Mauvais](#vize-croquis-cf-watch-can-be-computed-bad) · [Bon](#vize-croquis-cf-watch-can-be-computed-good) | Un observateur ne fait que copier une valeur dans l’état et peut être remplacé par une propriété calculée. |
| [`vize:croquis/cf/watcheffect-async`](#vize-croquis-cf-watcheffect-async) | [Mauvais](#vize-croquis-cf-watcheffect-async-bad) · [Bon](#vize-croquis-cf-watcheffect-async-good) | `watchEffect` lance une tâche asynchrone et ne peut pas nettoyer l’exécution précédente. |
| [`vize:croquis/cf/watcher-outside-setup`](#vize-croquis-cf-watcher-outside-setup) | [Mauvais](#vize-croquis-cf-watcher-outside-setup-bad) · [Bon](#vize-croquis-cf-watcher-outside-setup-good) | `watch` ou `watchEffect` est appelé en dehors de `setup`. |
| [`vue/a11y-img-alt`](#vue-a11y-img-alt) | [Mauvais](#vue-a11y-img-alt-bad) · [Bon](#vue-a11y-img-alt-good) | Exiger un attribut alt sur les images pour les rendre accessibles |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [Mauvais](#vue-attribute-hyphenation-bad) · [Bon](#vue-attribute-hyphenation-good) | Imposer un style de nommage des attributs sur les composants personnalisés |
| [`vue/attribute-order`](#vue-attribute-order) | [Mauvais](#vue-attribute-order-bad) · [Bon](#vue-attribute-order-good) | Imposer un ordre cohérent des attributs |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [Mauvais](#vue-component-definition-name-casing-bad) · [Bon](#vue-component-definition-name-casing-good) | Imposer PascalCase ou kebab-case aux noms de définition des composants |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [Mauvais](#vue-component-name-in-template-casing-bad) · [Bon](#vue-component-name-in-template-casing-good) | Imposer une casse précise aux noms de composants dans les templates |
| [`vue/cross-file-attrs-fallthrough`](#vue-cross-file-attrs-fallthrough) | [Mauvais](#vue-cross-file-attrs-fallthrough-bad) · [Bon](#vue-cross-file-attrs-fallthrough-good) | Un parent transmet des attributs à un enfant résolu dont la racine ne peut pas en hériter et qui n’utilise pas explicitement $attrs. |
| [`vue/html-button-has-type`](#vue-html-button-has-type) | [Mauvais](#vue-html-button-has-type-bad) · [Bon](#vue-html-button-has-type-good) | Exiger un type explicite et valide sur les éléments button |
| [`vue/html-quotes`](#vue-html-quotes) | [Mauvais](#vue-html-quotes-bad) · [Bon](#vue-html-quotes-good) | Imposer un style de guillemets pour les attributs HTML |
| [`vue/html-self-closing`](#vue-html-self-closing) | [Mauvais](#vue-html-self-closing-bad) · [Bon](#vue-html-self-closing-good) | Imposer un style de fermeture automatique des balises |
| [`vue/max-template-complexity`](#vue-max-template-complexity) | [Mauvais](#vue-max-template-complexity-bad) · [Bon](#vue-max-template-complexity-good) | Limiter la complexité propre du template d’un composant, cyclomatique et cognitive |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [Mauvais](#vue-multi-word-component-names-bad) · [Bon](#vue-multi-word-component-names-good) | Exiger des noms de composants composés de plusieurs mots |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [Mauvais](#vue-mustache-interpolation-spacing-bad) · [Bon](#vue-mustache-interpolation-spacing-good) | Imposer un espacement cohérent dans les interpolations à doubles accolades |
| [`vue/no-array-index-key`](#vue-no-array-index-key) | [Mauvais](#vue-no-array-index-key-bad) · [Bon](#vue-no-array-index-key-good) | Interdire l’utilisation directe de la variable d’index de v-for comme :key |
| [`vue/no-bare-strings-in-template`](#vue-no-bare-strings-in-template) | [Mauvais](#vue-no-bare-strings-in-template-bad) · [Bon](#vue-no-bare-strings-in-template-good) | Interdire le texte brut destiné aux utilisateurs dans les templates lorsqu’il devrait être internationalisé |
| [`vue/no-boolean-attr-value`](#vue-no-boolean-attr-value) | [Mauvais](#vue-no-boolean-attr-value-bad) · [Bon](#vue-no-boolean-attr-value-good) | Interdire les valeurs explicites des attributs HTML booléens |
| [`vue/no-child-content`](#vue-no-child-content) | [Mauvais](#vue-no-child-content-bad) · [Bon](#vue-no-child-content-good) | Interdire le contenu enfant lors de l’utilisation de v-html ou v-text |
| [`vue/no-deprecated-filter`](#vue-no-deprecated-filter) | [Mauvais](#vue-no-deprecated-filter-bad) · [Bon](#vue-no-deprecated-filter-good) | Interdire la syntaxe obsolète des filtres de Vue 2 utilisant l’opérateur pipe |
| [`vue/no-deprecated-functional-template`](#vue-no-deprecated-functional-template) | [Mauvais](#vue-no-deprecated-functional-template-bad) · [Bon](#vue-no-deprecated-functional-template-good) | Interdire l’attribut `functional` sur le `<template>` d’un SFC |
| [`vue/no-deprecated-html-element-is`](#vue-no-deprecated-html-element-is) | [Mauvais](#vue-no-deprecated-html-element-is-bad) · [Bon](#vue-no-deprecated-html-element-is-good) | Interdire l’attribut `is` sur les éléments HTML natifs |
| [`vue/no-deprecated-inline-template`](#vue-no-deprecated-inline-template) | [Mauvais](#vue-no-deprecated-inline-template-bad) · [Bon](#vue-no-deprecated-inline-template-good) | Interdire l’attribut obsolète `inline-template` |
| [`vue/no-deprecated-router-link-tag-prop`](#vue-no-deprecated-router-link-tag-prop) | [Mauvais](#vue-no-deprecated-router-link-tag-prop-bad) · [Bon](#vue-no-deprecated-router-link-tag-prop-good) | Interdire la prop `tag` sur &lt;router-link&gt; |
| [`vue/no-deprecated-scope-attribute`](#vue-no-deprecated-scope-attribute) | [Mauvais](#vue-no-deprecated-scope-attribute-bad) · [Bon](#vue-no-deprecated-scope-attribute-good) | Interdire l’attribut obsolète `scope` sur &lt;template&gt; |
| [`vue/no-deprecated-slot-attribute`](#vue-no-deprecated-slot-attribute) | [Mauvais](#vue-no-deprecated-slot-attribute-bad) · [Bon](#vue-no-deprecated-slot-attribute-good) | Interdire l’attribut obsolète `slot` |
| [`vue/no-deprecated-slot-scope-attribute`](#vue-no-deprecated-slot-scope-attribute) | [Mauvais](#vue-no-deprecated-slot-scope-attribute-bad) · [Bon](#vue-no-deprecated-slot-scope-attribute-good) | Interdire l’attribut obsolète `slot-scope` |
| [`vue/no-deprecated-v-bind-sync`](#vue-no-deprecated-v-bind-sync) | [Mauvais](#vue-no-deprecated-v-bind-sync-bad) · [Bon](#vue-no-deprecated-v-bind-sync-good) | Interdire le modificateur obsolète `.sync` sur `v-bind` |
| [`vue/no-deprecated-v-is`](#vue-no-deprecated-v-is) | [Mauvais](#vue-no-deprecated-v-is-bad) · [Bon](#vue-no-deprecated-v-is-good) | Interdire la directive obsolète `v-is` |
| [`vue/no-deprecated-v-on-native-modifier`](#vue-no-deprecated-v-on-native-modifier) | [Mauvais](#vue-no-deprecated-v-on-native-modifier-bad) · [Bon](#vue-no-deprecated-v-on-native-modifier-good) | Interdire le modificateur obsolète `.native` sur `v-on` |
| [`vue/no-deprecated-v-on-number-modifiers`](#vue-no-deprecated-v-on-number-modifiers) | [Mauvais](#vue-no-deprecated-v-on-number-modifiers-bad) · [Bon](#vue-no-deprecated-v-on-number-modifiers-good) | Interdire les modificateurs numériques obsolètes `keyCode` sur `v-on` |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [Mauvais](#vue-no-dupe-v-else-if-bad) · [Bon](#vue-no-dupe-v-else-if-good) | Interdire les conditions répétées dans les chaînes `v-if` / `v-else-if` |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [Mauvais](#vue-no-duplicate-attributes-bad) · [Bon](#vue-no-duplicate-attributes-good) | Interdire les attributs en double sur un même élément |
| [`vue/no-empty-component-block`](#vue-no-empty-component-block) | [Mauvais](#vue-no-empty-component-block-bad) · [Bon](#vue-no-empty-component-block-good) | Interdire les blocs SFC vides |
| [`vue/no-inline-style`](#vue-no-inline-style) | [Mauvais](#vue-no-inline-style-bad) · [Bon](#vue-no-inline-style-good) | Déconseiller l’utilisation d’attributs de style en ligne |
| [`vue/no-invalid-html-attribute`](#vue-no-invalid-html-attribute) | [Mauvais](#vue-no-invalid-html-attribute-bad) · [Bon](#vue-no-invalid-html-attribute-good) | Interdire les valeurs statiques invalides des attributs HTML |
| [`vue/no-lone-template`](#vue-no-lone-template) | [Mauvais](#vue-no-lone-template-bad) · [Bon](#vue-no-lone-template-good) | Interdire les éléments `<template>` inutiles |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [Mauvais](#vue-no-multi-spaces-bad) · [Bon](#vue-no-multi-spaces-good) | Interdire plusieurs espaces consécutifs |
| [`vue/no-multiple-objects-in-class`](#vue-no-multiple-objects-in-class) | [Mauvais](#vue-no-multiple-objects-in-class-bad) · [Bon](#vue-no-multiple-objects-in-class-good) | Interdire plusieurs objets littéraux dans une liaison :class sous forme de tableau |
| [`vue/no-multiple-template-root`](#vue-no-multiple-template-root) | [Mauvais](#vue-no-multiple-template-root-bad) · [Bon](#vue-no-multiple-template-root-good) | Interdire plusieurs nœuds racines dans un template |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [Mauvais](#vue-no-mutating-props-bad) · [Bon](#vue-no-mutating-props-good) | Interdire la modification des props d’un composant |
| [`vue/no-negated-v-if-condition`](#vue-no-negated-v-if-condition) | [Mauvais](#vue-no-negated-v-if-condition-bad) · [Bon](#vue-no-negated-v-if-condition-good) | Interdire une condition v-if négative lorsque la chaîne comporte un v-else |
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [Mauvais](#vue-no-non-component-keep-alive-child-bad) · [Bon](#vue-no-non-component-keep-alive-child-good) | Interdire les enveloppes d’éléments ordinaires directement sous `<KeepAlive>` |
| [`vue/no-preprocessor-lang`](#vue-no-preprocessor-lang) | [Mauvais](#vue-no-preprocessor-lang-bad) · [Bon](#vue-no-preprocessor-lang-good) | Déconseiller les préprocesseurs CSS au profit du CSS moderne |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [Mauvais](#vue-no-reserved-component-names-bad) · [Bon](#vue-no-reserved-component-names-good) | Interdire l’utilisation de noms réservés comme noms de composants |
| [`vue/no-root-v-if`](#vue-no-root-v-if) | [Mauvais](#vue-no-root-v-if-bad) · [Bon](#vue-no-root-v-if-good) | Interdire v-if sur l’unique élément racine d’un template |
| [`vue/no-script-non-standard-lang`](#vue-no-script-non-standard-lang) | [Mauvais](#vue-no-script-non-standard-lang-bad) · [Bon](#vue-no-script-non-standard-lang-good) | Déconseiller les valeurs lang non standard dans les scripts |
| [`vue/no-src-attribute`](#vue-no-src-attribute) | [Mauvais](#vue-no-src-attribute-bad) · [Bon](#vue-no-src-attribute-good) | Déconseiller l’attribut src sur les blocs SFC |
| [`vue/no-static-inline-styles`](#vue-no-static-inline-styles) | [Mauvais](#vue-no-static-inline-styles-bad) · [Bon](#vue-no-static-inline-styles-good) | Interdire les attributs de style en ligne statiques |
| [`vue/no-template-key`](#vue-no-template-key) | [Mauvais](#vue-no-template-key-bad) · [Bon](#vue-no-template-key-good) | Interdire l’attribut `key` sur `<template>` |
| [`vue/no-template-lang`](#vue-no-template-lang) | [Mauvais](#vue-no-template-lang-bad) · [Bon](#vue-no-template-lang-good) | Déconseiller l’attribut lang sur le bloc template |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [Mauvais](#vue-no-template-shadow-bad) · [Bon](#vue-no-template-shadow-good) | Interdire les noms de variables qui masquent des variables d’une portée externe |
| [`vue/no-template-target-blank`](#vue-no-template-target-blank) | [Mauvais](#vue-no-template-target-blank-bad) · [Bon](#vue-no-template-target-blank-good) | Interdire target="_blank" sans rel="noopener noreferrer" |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [Mauvais](#vue-no-textarea-mustache-bad) · [Bon](#vue-no-textarea-mustache-good) | Interdire l’interpolation à doubles accolades dans `<textarea>` |
| [`vue/no-undefined-refs`](#vue-no-undefined-refs) | [Mauvais](#vue-no-undefined-refs-bad) · [Bon](#vue-no-undefined-refs-good) | Interdire les références à des variables non définies dans les templates |
| [`vue/no-unsafe-url`](#vue-no-unsafe-url) | [Mauvais](#vue-no-unsafe-url-bad) · [Bon](#vue-no-unsafe-url-good) | Signaler les liaisons d’URL potentiellement dangereuses |
| [`vue/no-unsandboxed-iframe`](#vue-no-unsandboxed-iframe) | [Mauvais](#vue-no-unsandboxed-iframe-bad) · [Bon](#vue-no-unsandboxed-iframe-good) | Exiger un attribut sandbox sur les éléments iframe |
| [`vue/no-unused-components`](#vue-no-unused-components) | [Mauvais](#vue-no-unused-components-bad) · [Bon](#vue-no-unused-components-good) | Interdire l’enregistrement de composants inutilisés dans les templates |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [Mauvais](#vue-no-unused-properties-bad) · [Bon](#vue-no-unused-properties-good) | Interdire les propriétés inutilisées définies dans defineProps |
| [`vue/no-unused-refs`](#vue-no-unused-refs) | [Mauvais](#vue-no-unused-refs-bad) · [Bon](#vue-no-unused-refs-good) | Signaler les refs de template (ref="x") jamais référencées dans &lt;script&gt; |
| [`vue/no-unused-setup-bindings`](#vue-no-unused-setup-bindings) | [Mauvais](#vue-no-unused-setup-bindings-bad) · [Bon](#vue-no-unused-setup-bindings-good) | Interdire les liaisons script setup qui ne sont jamais lues |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [Mauvais](#vue-no-unused-vars-bad) · [Bon](#vue-no-unused-vars-good) | Interdire les variables inutilisées définies dans les directives v-for et v-slot |
| [`vue/no-use-v-else-with-v-for`](#vue-no-use-v-else-with-v-for) | [Mauvais](#vue-no-use-v-else-with-v-for-bad) · [Bon](#vue-no-use-v-else-with-v-for-good) | Interdire `v-else-if` ou `v-else` sur le même élément que `v-for` |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [Mauvais](#vue-no-use-v-if-with-v-for-bad) · [Bon](#vue-no-use-v-if-with-v-for-good) | Interdire `v-if` sur le même élément que `v-for` |
| [`vue/no-useless-mustaches`](#vue-no-useless-mustaches) | [Mauvais](#vue-no-useless-mustaches-bad) · [Bon](#vue-no-useless-mustaches-good) | Interdire une interpolation à doubles accolades dont l’expression est une chaîne littérale constante |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [Mauvais](#vue-no-useless-template-attributes-bad) · [Bon](#vue-no-useless-template-attributes-good) | Interdire les attributs inutiles sur les éléments `<template>` |
| [`vue/no-useless-v-bind`](#vue-no-useless-v-bind) | [Mauvais](#vue-no-useless-v-bind-bad) · [Bon](#vue-no-useless-v-bind-good) | Interdire un v-bind dont la valeur est une simple chaîne littérale |
| [`vue/no-v-for-template-key-on-child`](#vue-no-v-for-template-key-on-child) | [Mauvais](#vue-no-v-for-template-key-on-child-bad) · [Bon](#vue-no-v-for-template-key-on-child-good) | Interdire `key` sur l’enfant d’un `<template v-for>` |
| [`vue/no-v-html`](#vue-no-v-html) | [Mauvais](#vue-no-v-html-bad) · [Bon](#vue-no-v-html-good) | Déconseiller v-html pour prévenir les vulnérabilités XSS |
| [`vue/no-v-text`](#vue-no-v-text) | [Mauvais](#vue-no-v-text-bad) · [Bon](#vue-no-v-text-good) | Interdire la directive v-text ; préférer l’interpolation à doubles accolades |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [Mauvais](#vue-no-v-text-v-html-on-component-bad) · [Bon](#vue-no-v-text-v-html-on-component-good) | Interdire v-text / v-html sur les éléments de composants |
| [`vue/permitted-contents`](#vue-permitted-contents) | [Mauvais](#vue-permitted-contents-bad) · [Bon](#vue-permitted-contents-good) | Faire respecter les règles du modèle de contenu HTML |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [Mauvais](#vue-prefer-props-shorthand-bad) · [Bon](#vue-prefer-props-shorthand-good) | Recommander la syntaxe abrégée des props (Vue 3.4+) |
| [`vue/prefer-true-attribute-shorthand`](#vue-prefer-true-attribute-shorthand) | [Mauvais](#vue-prefer-true-attribute-shorthand-bad) · [Bon](#vue-prefer-true-attribute-shorthand-good) | Préférer la syntaxe abrégée pour un attribut booléen lié à `true` |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [Mauvais](#vue-prop-name-casing-bad) · [Bon](#vue-prop-name-casing-good) | Imposer une casse aux noms des props déclarées |
| [`vue/require-component-is`](#vue-require-component-is) | [Mauvais](#vue-require-component-is-bad) · [Bon](#vue-require-component-is-good) | Exiger `v-bind:is` sur les éléments `<component>` |
| [`vue/require-component-registration`](#vue-require-component-registration) | [Mauvais](#vue-require-component-registration-bad) · [Bon](#vue-require-component-registration-good) | Exiger un import ou un enregistrement explicite des composants |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [Mauvais](#vue-require-scoped-style-bad) · [Bon](#vue-require-scoped-style-good) | Exiger l’attribut scoped sur les balises style |
| [`vue/require-toggle-inside-transition`](#vue-require-toggle-inside-transition) | [Mauvais](#vue-require-toggle-inside-transition-bad) · [Bon](#vue-require-toggle-inside-transition-good) | Exiger un mécanisme de bascule sur l’élément enveloppé par `<transition>` |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [Mauvais](#vue-require-v-for-key-bad) · [Bon](#vue-require-v-for-key-good) | Exiger `v-bind:key` avec les directives `v-for` |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [Mauvais](#vue-scoped-event-names-bad) · [Bon](#vue-scoped-event-names-good) | Recommander des noms d’événements préfixés par leur contexte au format context:event |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [Mauvais](#vue-sfc-element-order-bad) · [Bon](#vue-sfc-element-order-good) | Imposer un ordre cohérent des éléments de premier niveau d’un SFC |
| [`vue/single-style-block`](#vue-single-style-block) | [Mauvais](#vue-single-style-block-bad) · [Bon](#vue-single-style-block-good) | Recommander un seul bloc style |
| [`vue/slot-name-casing`](#vue-slot-name-casing) | [Mauvais](#vue-slot-name-casing-bad) · [Bon](#vue-slot-name-casing-good) | Imposer kebab-case aux slots nommés utilisés avec v-slot |
| [`vue/this-in-template`](#vue-this-in-template) | [Mauvais](#vue-this-in-template-bad) · [Bon](#vue-this-in-template-good) | Interdire `this.` dans les expressions des templates |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [Mauvais](#vue-use-unique-element-ids-bad) · [Bon](#vue-use-unique-element-ids-good) | Imposer des identifiants d’éléments uniques avec useId() plutôt que des littéraux statiques |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [Mauvais](#vue-use-v-on-exact-bad) · [Bon](#vue-use-v-on-exact-good) | Imposer le modificateur `.exact` sur `v-on` en présence de gestionnaires utilisant des modificateurs |
| [`vue/v-bind-style`](#vue-v-bind-style) | [Mauvais](#vue-v-bind-style-bad) · [Bon](#vue-v-bind-style-good) | Imposer un style de directive `v-bind` |
| [`vue/v-on-event-hyphenation`](#vue-v-on-event-hyphenation) | [Mauvais](#vue-v-on-event-hyphenation-bad) · [Bon](#vue-v-on-event-hyphenation-good) | Imposer des tirets dans les noms d’événements personnalisés de v-on sur les composants |
| [`vue/v-on-handler-style`](#vue-v-on-handler-style) | [Mauvais](#vue-v-on-handler-style-bad) · [Bon](#vue-v-on-handler-style-good) | Imposer une référence de méthode ou une fonction en ligne pour les gestionnaires v-on |
| [`vue/v-on-style`](#vue-v-on-style) | [Mauvais](#vue-v-on-style-bad) · [Bon](#vue-v-on-style-good) | Imposer un style de directive `v-on` |
| [`vue/v-slot-style`](#vue-v-slot-style) | [Mauvais](#vue-v-slot-style-bad) · [Bon](#vue-v-slot-style-good) | Imposer un style de directive `v-slot` |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [Mauvais](#vue-valid-attribute-name-bad) · [Bon](#vue-valid-attribute-name-good) | Exiger des noms d’attributs valides |
| [`vue/valid-template-root`](#vue-valid-template-root) | [Mauvais](#vue-valid-template-root-bad) · [Bon](#vue-valid-template-root-good) | Exiger une racine `<template>` valide selon la sémantique des fragments de Vue 3 |
| [`vue/valid-v-bind`](#vue-valid-v-bind) | [Mauvais](#vue-valid-v-bind-bad) · [Bon](#vue-valid-v-bind-good) | Exiger des directives `v-bind` valides |
| [`vue/valid-v-cloak`](#vue-valid-v-cloak) | [Mauvais](#vue-valid-v-cloak-bad) · [Bon](#vue-valid-v-cloak-good) | Exiger des directives `v-cloak` valides |
| [`vue/valid-v-else`](#vue-valid-v-else) | [Mauvais](#vue-valid-v-else-bad) · [Bon](#vue-valid-v-else-good) | Exiger des directives `v-else` valides |
| [`vue/valid-v-else-if`](#vue-valid-v-else-if) | [Mauvais](#vue-valid-v-else-if-bad) · [Bon](#vue-valid-v-else-if-good) | Exiger des directives `v-else-if` valides |
| [`vue/valid-v-for`](#vue-valid-v-for) | [Mauvais](#vue-valid-v-for-bad) · [Bon](#vue-valid-v-for-good) | Exiger des directives `v-for` valides |
| [`vue/valid-v-html`](#vue-valid-v-html) | [Mauvais](#vue-valid-v-html-bad) · [Bon](#vue-valid-v-html-good) | Exiger des directives `v-html` valides |
| [`vue/valid-v-if`](#vue-valid-v-if) | [Mauvais](#vue-valid-v-if-bad) · [Bon](#vue-valid-v-if-good) | Exiger des directives `v-if` valides |
| [`vue/valid-v-is`](#vue-valid-v-is) | [Mauvais](#vue-valid-v-is-bad) · [Bon](#vue-valid-v-is-good) | Exiger des directives `v-is` valides |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [Mauvais](#vue-valid-v-memo-bad) · [Bon](#vue-valid-v-memo-good) | Exiger des directives `v-memo` valides |
| [`vue/valid-v-model`](#vue-valid-v-model) | [Mauvais](#vue-valid-v-model-bad) · [Bon](#vue-valid-v-model-good) | Exiger des directives `v-model` valides |
| [`vue/valid-v-on`](#vue-valid-v-on) | [Mauvais](#vue-valid-v-on-bad) · [Bon](#vue-valid-v-on-good) | Exiger des directives `v-on` valides |
| [`vue/valid-v-once`](#vue-valid-v-once) | [Mauvais](#vue-valid-v-once-bad) · [Bon](#vue-valid-v-once-good) | Exiger des directives `v-once` valides |
| [`vue/valid-v-pre`](#vue-valid-v-pre) | [Mauvais](#vue-valid-v-pre-bad) · [Bon](#vue-valid-v-pre-good) | Exiger des directives `v-pre` valides |
| [`vue/valid-v-show`](#vue-valid-v-show) | [Mauvais](#vue-valid-v-show-bad) · [Bon](#vue-valid-v-show-good) | Exiger des directives `v-show` valides |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [Mauvais](#vue-valid-v-slot-bad) · [Bon](#vue-valid-v-slot-good) | Exiger des directives `v-slot` valides |
| [`vue/valid-v-text`](#vue-valid-v-text) | [Mauvais](#vue-valid-v-text-bad) · [Bon](#vue-valid-v-text-good) | Exiger des directives `v-text` valides |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [Mauvais](#vue-warn-custom-block-bad) · [Bon](#vue-warn-custom-block-good) | Signaler les blocs personnalisés dans les fichiers SFC |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [Mauvais](#vue-warn-custom-directive-bad) · [Bon](#vue-warn-custom-directive-good) | Signaler les directives personnalisées nécessitant un enregistrement |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `a11y/alt-text`

Exiger un texte alternatif pour les éléments multimédias

<span id="a11y-alt-text-bad"></span>

[Mauvais](./accessibility.md#a11y-alt-text-bad)

<span id="a11y-alt-text-good"></span>

[Bon](./accessibility.md#a11y-alt-text-good)

[Exemples du groupe](./accessibility.md#a11y-alt-text) · [Fiche de la règle](/rules/reference/a11y-alt-text.md)

### `a11y/anchor-has-content`

Exiger un contenu accessible pour les éléments de lien

<span id="a11y-anchor-has-content-bad"></span>

[Mauvais](./accessibility.md#a11y-anchor-has-content-bad)

<span id="a11y-anchor-has-content-good"></span>

[Bon](./accessibility.md#a11y-anchor-has-content-good)

[Exemples du groupe](./accessibility.md#a11y-anchor-has-content) · [Fiche de la règle](/rules/reference/a11y-anchor-has-content.md)

### `a11y/anchor-is-valid`

Imposer un href valide sur les éléments de lien

<span id="a11y-anchor-is-valid-bad"></span>

[Mauvais](./accessibility.md#a11y-anchor-is-valid-bad)

<span id="a11y-anchor-is-valid-good"></span>

[Bon](./accessibility.md#a11y-anchor-is-valid-good)

[Exemples du groupe](./accessibility.md#a11y-anchor-is-valid) · [Fiche de la règle](/rules/reference/a11y-anchor-is-valid.md)

### `a11y/aria-props`

Interdire les attributs ARIA invalides

<span id="a11y-aria-props-bad"></span>

[Mauvais](./accessibility.md#a11y-aria-props-bad)

<span id="a11y-aria-props-good"></span>

[Bon](./accessibility.md#a11y-aria-props-good)

[Exemples du groupe](./accessibility.md#a11y-aria-props) · [Fiche de la règle](/rules/reference/a11y-aria-props.md)

### `a11y/aria-role`

Exiger un rôle ARIA valide et non abstrait pour les éléments possédant un rôle ARIA

<span id="a11y-aria-role-bad"></span>

[Mauvais](./accessibility.md#a11y-aria-role-bad)

<span id="a11y-aria-role-good"></span>

[Bon](./accessibility.md#a11y-aria-role-good)

[Exemples du groupe](./accessibility.md#a11y-aria-role) · [Fiche de la règle](/rules/reference/a11y-aria-role.md)

### `a11y/aria-unsupported-elements`

Interdire les attributs ARIA sur les éléments qui ne les prennent pas en charge

<span id="a11y-aria-unsupported-elements-bad"></span>

[Mauvais](./accessibility.md#a11y-aria-unsupported-elements-bad)

<span id="a11y-aria-unsupported-elements-good"></span>

[Bon](./accessibility.md#a11y-aria-unsupported-elements-good)

[Exemples du groupe](./accessibility.md#a11y-aria-unsupported-elements) · [Fiche de la règle](/rules/reference/a11y-aria-unsupported-elements.md)

### `a11y/click-events-have-key-events`

Exiger des gestionnaires de clavier avec les événements de clic

<span id="a11y-click-events-have-key-events-bad"></span>

[Mauvais](./accessibility.md#a11y-click-events-have-key-events-bad)

<span id="a11y-click-events-have-key-events-good"></span>

[Bon](./accessibility.md#a11y-click-events-have-key-events-good)

[Exemples du groupe](./accessibility.md#a11y-click-events-have-key-events) · [Fiche de la règle](/rules/reference/a11y-click-events-have-key-events.md)

### `a11y/form-control-has-label`

Exiger des libellés associés aux contrôles de formulaire

<span id="a11y-form-control-has-label-bad"></span>

[Mauvais](./accessibility.md#a11y-form-control-has-label-bad)

<span id="a11y-form-control-has-label-good"></span>

[Bon](./accessibility.md#a11y-form-control-has-label-good)

[Exemples du groupe](./accessibility.md#a11y-form-control-has-label) · [Fiche de la règle](/rules/reference/a11y-form-control-has-label.md)

### `a11y/heading-has-content`

Exiger un contenu accessible pour les éléments de titre

<span id="a11y-heading-has-content-bad"></span>

[Mauvais](./accessibility.md#a11y-heading-has-content-bad)

<span id="a11y-heading-has-content-good"></span>

[Bon](./accessibility.md#a11y-heading-has-content-good)

[Exemples du groupe](./accessibility.md#a11y-heading-has-content) · [Fiche de la règle](/rules/reference/a11y-heading-has-content.md)

### `a11y/heading-levels`

Interdire de sauter des niveaux de titre

<span id="a11y-heading-levels-bad"></span>

[Mauvais](./accessibility.md#a11y-heading-levels-bad)

<span id="a11y-heading-levels-good"></span>

[Bon](./accessibility.md#a11y-heading-levels-good)

[Exemples du groupe](./accessibility.md#a11y-heading-levels) · [Fiche de la règle](/rules/reference/a11y-heading-levels.md)

### `a11y/iframe-has-title`

Exiger un attribut title sur les éléments iframe

<span id="a11y-iframe-has-title-bad"></span>

[Mauvais](./accessibility.md#a11y-iframe-has-title-bad)

<span id="a11y-iframe-has-title-good"></span>

[Bon](./accessibility.md#a11y-iframe-has-title-good)

[Exemples du groupe](./accessibility.md#a11y-iframe-has-title) · [Fiche de la règle](/rules/reference/a11y-iframe-has-title.md)

### `a11y/img-alt`

Exiger un attribut alt sur les images pour les rendre accessibles

<span id="a11y-img-alt-bad"></span>

[Mauvais](./accessibility.md#a11y-img-alt-bad)

<span id="a11y-img-alt-good"></span>

[Bon](./accessibility.md#a11y-img-alt-good)

[Exemples du groupe](./accessibility.md#a11y-img-alt) · [Fiche de la règle](/rules/reference/a11y-img-alt.md)

### `a11y/interactive-supports-focus`

Exiger que les éléments possédant un rôle interactif puissent recevoir le focus

<span id="a11y-interactive-supports-focus-bad"></span>

[Mauvais](./accessibility.md#a11y-interactive-supports-focus-bad)

<span id="a11y-interactive-supports-focus-good"></span>

[Bon](./accessibility.md#a11y-interactive-supports-focus-good)

[Exemples du groupe](./accessibility.md#a11y-interactive-supports-focus) · [Fiche de la règle](/rules/reference/a11y-interactive-supports-focus.md)

### `a11y/label-has-for`

Exiger des contrôles de formulaire associés aux libellés

<span id="a11y-label-has-for-bad"></span>

[Mauvais](./accessibility.md#a11y-label-has-for-bad)

<span id="a11y-label-has-for-good"></span>

[Bon](./accessibility.md#a11y-label-has-for-good)

[Exemples du groupe](./accessibility.md#a11y-label-has-for) · [Fiche de la règle](/rules/reference/a11y-label-has-for.md)

### `a11y/landmark-roles`

Valider l’emplacement et l’unicité des rôles de zones de repère

<span id="a11y-landmark-roles-bad"></span>

[Mauvais](./accessibility.md#a11y-landmark-roles-bad)

<span id="a11y-landmark-roles-good"></span>

[Bon](./accessibility.md#a11y-landmark-roles-good)

[Exemples du groupe](./accessibility.md#a11y-landmark-roles) · [Fiche de la règle](/rules/reference/a11y-landmark-roles.md)

### `a11y/media-has-caption`

Exiger des sous-titres pour les éléments multimédias

<span id="a11y-media-has-caption-bad"></span>

[Mauvais](./accessibility.md#a11y-media-has-caption-bad)

<span id="a11y-media-has-caption-good"></span>

[Bon](./accessibility.md#a11y-media-has-caption-good)

[Exemples du groupe](./accessibility.md#a11y-media-has-caption) · [Fiche de la règle](/rules/reference/a11y-media-has-caption.md)

### `a11y/mouse-events-have-key-events`

Exiger des événements de focus et de perte du focus avec les événements de souris

<span id="a11y-mouse-events-have-key-events-bad"></span>

[Mauvais](./accessibility.md#a11y-mouse-events-have-key-events-bad)

<span id="a11y-mouse-events-have-key-events-good"></span>

[Bon](./accessibility.md#a11y-mouse-events-have-key-events-good)

[Exemples du groupe](./accessibility.md#a11y-mouse-events-have-key-events) · [Fiche de la règle](/rules/reference/a11y-mouse-events-have-key-events.md)

### `a11y/no-access-key`

Interdire l’utilisation de l’attribut accesskey

<span id="a11y-no-access-key-bad"></span>

[Mauvais](./accessibility.md#a11y-no-access-key-bad)

<span id="a11y-no-access-key-good"></span>

[Bon](./accessibility.md#a11y-no-access-key-good)

[Exemples du groupe](./accessibility.md#a11y-no-access-key) · [Fiche de la règle](/rules/reference/a11y-no-access-key.md)

### `a11y/no-aria-hidden-on-focusable`

Interdire aria-hidden="true" sur les éléments pouvant recevoir le focus

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

[Mauvais](./accessibility.md#a11y-no-aria-hidden-on-focusable-bad)

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

[Bon](./accessibility.md#a11y-no-aria-hidden-on-focusable-good)

[Exemples du groupe](./accessibility.md#a11y-no-aria-hidden-on-focusable) · [Fiche de la règle](/rules/reference/a11y-no-aria-hidden-on-focusable.md)

### `a11y/no-autofocus`

Interdire l’utilisation de l’attribut autofocus

<span id="a11y-no-autofocus-bad"></span>

[Mauvais](./accessibility.md#a11y-no-autofocus-bad)

<span id="a11y-no-autofocus-good"></span>

[Bon](./accessibility.md#a11y-no-autofocus-good)

[Exemples du groupe](./accessibility.md#a11y-no-autofocus) · [Fiche de la règle](/rules/reference/a11y-no-autofocus.md)

### `a11y/no-distracting-elements`

Interdire les éléments distrayants tels que &lt;marquee&gt; et &lt;blink&gt;

<span id="a11y-no-distracting-elements-bad"></span>

[Mauvais](./accessibility.md#a11y-no-distracting-elements-bad)

<span id="a11y-no-distracting-elements-good"></span>

[Bon](./accessibility.md#a11y-no-distracting-elements-good)

[Exemples du groupe](./accessibility.md#a11y-no-distracting-elements) · [Fiche de la règle](/rules/reference/a11y-no-distracting-elements.md)

### `a11y/no-i-for-icon`

Interdire l’élément &lt;i&gt; pour les icônes

<span id="a11y-no-i-for-icon-bad"></span>

[Mauvais](./accessibility.md#a11y-no-i-for-icon-bad)

<span id="a11y-no-i-for-icon-good"></span>

[Bon](./accessibility.md#a11y-no-i-for-icon-good)

[Exemples du groupe](./accessibility.md#a11y-no-i-for-icon) · [Fiche de la règle](/rules/reference/a11y-no-i-for-icon.md)

### `a11y/no-redundant-roles`

Interdire les rôles ARIA redondants

<span id="a11y-no-redundant-roles-bad"></span>

[Mauvais](./accessibility.md#a11y-no-redundant-roles-bad)

<span id="a11y-no-redundant-roles-good"></span>

[Bon](./accessibility.md#a11y-no-redundant-roles-good)

[Exemples du groupe](./accessibility.md#a11y-no-redundant-roles) · [Fiche de la règle](/rules/reference/a11y-no-redundant-roles.md)

### `a11y/no-refer-to-non-existent-id`

Interdire les références à des identifiants inexistants

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

[Mauvais](./accessibility.md#a11y-no-refer-to-non-existent-id-bad)

<span id="a11y-no-refer-to-non-existent-id-good"></span>

[Bon](./accessibility.md#a11y-no-refer-to-non-existent-id-good)

[Exemples du groupe](./accessibility.md#a11y-no-refer-to-non-existent-id) · [Fiche de la règle](/rules/reference/a11y-no-refer-to-non-existent-id.md)

### `a11y/no-role-presentation-on-focusable`

Interdire role="presentation" ou role="none" sur les éléments pouvant recevoir le focus

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

[Mauvais](./accessibility.md#a11y-no-role-presentation-on-focusable-bad)

<span id="a11y-no-role-presentation-on-focusable-good"></span>

[Bon](./accessibility.md#a11y-no-role-presentation-on-focusable-good)

[Exemples du groupe](./accessibility.md#a11y-no-role-presentation-on-focusable) · [Fiche de la règle](/rules/reference/a11y-no-role-presentation-on-focusable.md)

### `a11y/no-static-element-interactions`

Interdire les gestionnaires d’événements sur les éléments statiques

<span id="a11y-no-static-element-interactions-bad"></span>

[Mauvais](./accessibility.md#a11y-no-static-element-interactions-bad)

<span id="a11y-no-static-element-interactions-good"></span>

[Bon](./accessibility.md#a11y-no-static-element-interactions-good)

[Exemples du groupe](./accessibility.md#a11y-no-static-element-interactions) · [Fiche de la règle](/rules/reference/a11y-no-static-element-interactions.md)

### `a11y/placeholder-label-option`

Exiger disabled ou hidden sur l’option d’invite d’un select

<span id="a11y-placeholder-label-option-bad"></span>

[Mauvais](./accessibility.md#a11y-placeholder-label-option-bad)

<span id="a11y-placeholder-label-option-good"></span>

[Bon](./accessibility.md#a11y-placeholder-label-option-good)

[Exemples du groupe](./accessibility.md#a11y-placeholder-label-option) · [Fiche de la règle](/rules/reference/a11y-placeholder-label-option.md)

### `a11y/role-has-required-aria-props`

Exiger les propriétés obligatoires des rôles ARIA

<span id="a11y-role-has-required-aria-props-bad"></span>

[Mauvais](./accessibility.md#a11y-role-has-required-aria-props-bad)

<span id="a11y-role-has-required-aria-props-good"></span>

[Bon](./accessibility.md#a11y-role-has-required-aria-props-good)

[Exemples du groupe](./accessibility.md#a11y-role-has-required-aria-props) · [Fiche de la règle](/rules/reference/a11y-role-has-required-aria-props.md)

### `a11y/tabindex-no-positive`

Interdire les valeurs positives de tabindex

<span id="a11y-tabindex-no-positive-bad"></span>

[Mauvais](./accessibility.md#a11y-tabindex-no-positive-bad)

<span id="a11y-tabindex-no-positive-good"></span>

[Bon](./accessibility.md#a11y-tabindex-no-positive-good)

[Exemples du groupe](./accessibility.md#a11y-tabindex-no-positive) · [Fiche de la règle](/rules/reference/a11y-tabindex-no-positive.md)

### `a11y/use-list`

Suggérer des éléments de liste pour les textes ressemblant à des listes à puces

<span id="a11y-use-list-bad"></span>

[Mauvais](./accessibility.md#a11y-use-list-bad)

<span id="a11y-use-list-good"></span>

[Bon](./accessibility.md#a11y-use-list-good)

[Exemples du groupe](./accessibility.md#a11y-use-list) · [Fiche de la règle](/rules/reference/a11y-use-list.md)

### `css/no-display-none`

Suggérer v-show plutôt que display: none

<span id="css-no-display-none-bad"></span>

[Mauvais](./musea-and-css.md#css-no-display-none-bad)

<span id="css-no-display-none-good"></span>

[Bon](./musea-and-css.md#css-no-display-none-good)

[Exemples du groupe](./musea-and-css.md#css-no-display-none) · [Fiche de la règle](/rules/reference/css-no-display-none.md)

### `css/no-hardcoded-values`

Suggérer des variables CSS plutôt que des valeurs codées en dur

<span id="css-no-hardcoded-values-bad"></span>

[Mauvais](./musea-and-css.md#css-no-hardcoded-values-bad)

<span id="css-no-hardcoded-values-good"></span>

[Bon](./musea-and-css.md#css-no-hardcoded-values-good)

[Exemples du groupe](./musea-and-css.md#css-no-hardcoded-values) · [Fiche de la règle](/rules/reference/css-no-hardcoded-values.md)

### `css/no-id-selectors`

Déconseiller les sélecteurs d’identifiant en CSS

<span id="css-no-id-selectors-bad"></span>

[Mauvais](./musea-and-css.md#css-no-id-selectors-bad)

<span id="css-no-id-selectors-good"></span>

[Bon](./musea-and-css.md#css-no-id-selectors-good)

[Exemples du groupe](./musea-and-css.md#css-no-id-selectors) · [Fiche de la règle](/rules/reference/css-no-id-selectors.md)

### `css/no-important`

Déconseiller !important en CSS

<span id="css-no-important-bad"></span>

[Mauvais](./musea-and-css.md#css-no-important-bad)

<span id="css-no-important-good"></span>

[Bon](./musea-and-css.md#css-no-important-good)

[Exemples du groupe](./musea-and-css.md#css-no-important) · [Fiche de la règle](/rules/reference/css-no-important.md)

### `css/no-utility-classes`

Déconseiller l’implémentation de classes utilitaires dans les styles des composants

<span id="css-no-utility-classes-bad"></span>

[Mauvais](./musea-and-css.md#css-no-utility-classes-bad)

<span id="css-no-utility-classes-good"></span>

[Bon](./musea-and-css.md#css-no-utility-classes-good)

[Exemples du groupe](./musea-and-css.md#css-no-utility-classes) · [Fiche de la règle](/rules/reference/css-no-utility-classes.md)

### `css/no-v-bind-performance`

Signaler le coût de performance de v-bind() en CSS

<span id="css-no-v-bind-performance-bad"></span>

[Mauvais](./musea-and-css.md#css-no-v-bind-performance-bad)

<span id="css-no-v-bind-performance-good"></span>

[Bon](./musea-and-css.md#css-no-v-bind-performance-good)

[Exemples du groupe](./musea-and-css.md#css-no-v-bind-performance) · [Fiche de la règle](/rules/reference/css-no-v-bind-performance.md)

### `css/prefer-logical-properties`

Recommander les propriétés logiques CSS pour mieux prendre en charge l’internationalisation

<span id="css-prefer-logical-properties-bad"></span>

[Mauvais](./musea-and-css.md#css-prefer-logical-properties-bad)

<span id="css-prefer-logical-properties-good"></span>

[Bon](./musea-and-css.md#css-prefer-logical-properties-good)

[Exemples du groupe](./musea-and-css.md#css-prefer-logical-properties) · [Fiche de la règle](/rules/reference/css-prefer-logical-properties.md)

### `css/prefer-nested-selectors`

Recommander l’imbrication CSS pour les sélecteurs de descendants

<span id="css-prefer-nested-selectors-bad"></span>

[Mauvais](./musea-and-css.md#css-prefer-nested-selectors-bad)

<span id="css-prefer-nested-selectors-good"></span>

[Bon](./musea-and-css.md#css-prefer-nested-selectors-good)

[Exemples du groupe](./musea-and-css.md#css-prefer-nested-selectors) · [Fiche de la règle](/rules/reference/css-prefer-nested-selectors.md)

### `css/prefer-slotted`

Recommander ::v-slotted() pour styliser le contenu des slots

<span id="css-prefer-slotted-bad"></span>

[Mauvais](./musea-and-css.md#css-prefer-slotted-bad)

<span id="css-prefer-slotted-good"></span>

[Bon](./musea-and-css.md#css-prefer-slotted-good)

[Exemples du groupe](./musea-and-css.md#css-prefer-slotted) · [Fiche de la règle](/rules/reference/css-prefer-slotted.md)

### `css/require-font-display`

Exiger font-display dans les règles @font-face

<span id="css-require-font-display-bad"></span>

[Mauvais](./musea-and-css.md#css-require-font-display-bad)

<span id="css-require-font-display-good"></span>

[Bon](./musea-and-css.md#css-require-font-display-good)

[Exemples du groupe](./musea-and-css.md#css-require-font-display) · [Fiche de la règle](/rules/reference/css-require-font-display.md)

### `ecosystem/nuxt-prefer-nuxt-link`

Préférer NuxtLink pour les liens internes à l’application

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

[Mauvais](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link-bad)

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

[Bon](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link-good)

[Exemples du groupe](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link) · [Fiche de la règle](/rules/reference/ecosystem-nuxt-prefer-nuxt-link.md)

### `ecosystem/pinia-prefer-store-to-refs`

Préférer storeToRefs() lors de la déstructuration des stores Pinia

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

[Mauvais](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs-bad)

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

[Bon](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs-good)

[Exemples du groupe](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs) · [Fiche de la règle](/rules/reference/ecosystem-pinia-prefer-store-to-refs.md)

### `ecosystem/router-link-require-to`

Exiger une destination `to` sur les composants RouterLink et NuxtLink

<span id="ecosystem-router-link-require-to-bad"></span>

[Mauvais](./ecosystem.md#ecosystem-router-link-require-to-bad)

<span id="ecosystem-router-link-require-to-good"></span>

[Bon](./ecosystem.md#ecosystem-router-link-require-to-good)

[Exemples du groupe](./ecosystem.md#ecosystem-router-link-require-to) · [Fiche de la règle](/rules/reference/ecosystem-router-link-require-to.md)

### `ecosystem/void-link-require-href`

Exiger `href` sur les composants Link de Void Vue

<span id="ecosystem-void-link-require-href-bad"></span>

[Mauvais](./ecosystem.md#ecosystem-void-link-require-href-bad)

<span id="ecosystem-void-link-require-href-good"></span>

[Bon](./ecosystem.md#ecosystem-void-link-require-href-good)

[Exemples du groupe](./ecosystem.md#ecosystem-void-link-require-href) · [Fiche de la règle](/rules/reference/ecosystem-void-link-require-href.md)

### `ecosystem/void-link-valid-method`

Valider les props method statiques des composants Link de Void Vue

<span id="ecosystem-void-link-valid-method-bad"></span>

[Mauvais](./ecosystem.md#ecosystem-void-link-valid-method-bad)

<span id="ecosystem-void-link-valid-method-good"></span>

[Bon](./ecosystem.md#ecosystem-void-link-valid-method-good)

[Exemples du groupe](./ecosystem.md#ecosystem-void-link-valid-method) · [Fiche de la règle](/rules/reference/ecosystem-void-link-valid-method.md)

### `ecosystem/vue-i18n-no-missing-key`

Signaler les clés vue-i18n statiques absentes des messages locaux du SFC

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

[Mauvais](./ecosystem.md#ecosystem-vue-i18n-no-missing-key-bad)

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

[Bon](./ecosystem.md#ecosystem-vue-i18n-no-missing-key-good)

[Exemples du groupe](./ecosystem.md#ecosystem-vue-i18n-no-missing-key) · [Fiche de la règle](/rules/reference/ecosystem-vue-i18n-no-missing-key.md)

### `ecosystem/vue-router-extra-param`

La route ne déclare pas tab ; Vue Router l’ignore.

<span id="ecosystem-vue-router-extra-param-bad"></span>

[Mauvais](./cross-file.md#ecosystem-vue-router-extra-param-bad)

<span id="ecosystem-vue-router-extra-param-good"></span>

[Bon](./cross-file.md#ecosystem-vue-router-extra-param-good)

[Exemples du groupe](./cross-file.md#ecosystem-vue-router-extra-param) · [Fiche de la règle](/rules/project/ecosystem-vue-router-extra-param.md)

### `ecosystem/vue-router-missing-param`

Le paramètre obligatoire postId manque ; dépendre de la route actuelle est fragile.

<span id="ecosystem-vue-router-missing-param-bad"></span>

[Mauvais](./cross-file.md#ecosystem-vue-router-missing-param-bad)

<span id="ecosystem-vue-router-missing-param-good"></span>

[Bon](./cross-file.md#ecosystem-vue-router-missing-param-good)

[Exemples du groupe](./cross-file.md#ecosystem-vue-router-missing-param) · [Fiche de la règle](/rules/project/ecosystem-vue-router-missing-param.md)

### `ecosystem/vue-router-param-type`

postId n’est pas répétable ; un tableau est donc invalide.

<span id="ecosystem-vue-router-param-type-bad"></span>

[Mauvais](./cross-file.md#ecosystem-vue-router-param-type-bad)

<span id="ecosystem-vue-router-param-type-good"></span>

[Bon](./cross-file.md#ecosystem-vue-router-param-type-good)

[Exemples du groupe](./cross-file.md#ecosystem-vue-router-param-type) · [Fiche de la règle](/rules/project/ecosystem-vue-router-param-type.md)

### `ecosystem/vue-router-prefer-named-link`

Préférer les objets de routes nommées aux chaînes de chemin statiques dans RouterLink

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

[Mauvais](./ecosystem.md#ecosystem-vue-router-prefer-named-link-bad)

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

[Bon](./ecosystem.md#ecosystem-vue-router-prefer-named-link-good)

[Exemples du groupe](./ecosystem.md#ecosystem-vue-router-prefer-named-link) · [Fiche de la règle](/rules/reference/ecosystem-vue-router-prefer-named-link.md)

### `ecosystem/vue-router-prefer-named-push`

Préférer les objets de routes nommées pour la navigation programmatique de Vue Router

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

[Mauvais](./ecosystem.md#ecosystem-vue-router-prefer-named-push-bad)

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

[Bon](./ecosystem.md#ecosystem-vue-router-prefer-named-push-good)

[Exemples du groupe](./ecosystem.md#ecosystem-vue-router-prefer-named-push) · [Fiche de la règle](/rules/reference/ecosystem-vue-router-prefer-named-push.md)

### `ecosystem/vue-router-unknown-route`

Le nom est absent du routeur installé complet.

<span id="ecosystem-vue-router-unknown-route-bad"></span>

[Mauvais](./cross-file.md#ecosystem-vue-router-unknown-route-bad)

<span id="ecosystem-vue-router-unknown-route-good"></span>

[Bon](./cross-file.md#ecosystem-vue-router-unknown-route-good)

[Exemples du groupe](./cross-file.md#ecosystem-vue-router-unknown-route) · [Fiche de la règle](/rules/project/ecosystem-vue-router-unknown-route.md)

### `ecosystem/vue-test-utils-no-html-snapshot`

Éviter les instantanés de wrapper.html() dans les tests Vue Test Utils

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

[Mauvais](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot-bad)

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

[Bon](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot-good)

[Exemples du groupe](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot) · [Fiche de la règle](/rules/reference/ecosystem-vue-test-utils-no-html-snapshot.md)

### `html/cross-component-nesting`

Vérifier l’imbrication HTML réelle après la composition des composants importés.

<span id="html-cross-component-nesting-bad"></span>

[Mauvais](./cross-file.md#html-cross-component-nesting-bad)

<span id="html-cross-component-nesting-good"></span>

[Bon](./cross-file.md#html-cross-component-nesting-good)

[Exemples du groupe](./cross-file.md#html-cross-component-nesting) · [Fiche de la règle](/rules/project/html-cross-component-nesting.md)

### `html/deprecated-attr`

Interdire les attributs HTML obsolètes

<span id="html-deprecated-attr-bad"></span>

[Mauvais](./html.md#html-deprecated-attr-bad)

<span id="html-deprecated-attr-good"></span>

[Bon](./html.md#html-deprecated-attr-good)

[Exemples du groupe](./html.md#html-deprecated-attr) · [Fiche de la règle](/rules/reference/html-deprecated-attr.md)

### `html/deprecated-element`

Interdire les éléments HTML obsolètes

<span id="html-deprecated-element-bad"></span>

[Mauvais](./html.md#html-deprecated-element-bad)

<span id="html-deprecated-element-good"></span>

[Bon](./html.md#html-deprecated-element-good)

[Exemples du groupe](./html.md#html-deprecated-element) · [Fiche de la règle](/rules/reference/html-deprecated-element.md)

### `html/id-duplication`

Interdire les identifiants d’éléments en double

<span id="html-id-duplication-bad"></span>

[Mauvais](./html.md#html-id-duplication-bad)

<span id="html-id-duplication-good"></span>

[Bon](./html.md#html-id-duplication-good)

[Exemples du groupe](./html.md#html-id-duplication) · [Fiche de la règle](/rules/reference/html-id-duplication.md)

### `html/no-consecutive-br`

Interdire les éléments &lt;br&gt; consécutifs

<span id="html-no-consecutive-br-bad"></span>

[Mauvais](./html.md#html-no-consecutive-br-bad)

<span id="html-no-consecutive-br-good"></span>

[Bon](./html.md#html-no-consecutive-br-good)

[Exemples du groupe](./html.md#html-no-consecutive-br) · [Fiche de la règle](/rules/reference/html-no-consecutive-br.md)

### `html/no-dupe-style-properties`

Interdire les propriétés en double dans les attributs de style en ligne

<span id="html-no-dupe-style-properties-bad"></span>

[Mauvais](./html.md#html-no-dupe-style-properties-bad)

<span id="html-no-dupe-style-properties-good"></span>

[Bon](./html.md#html-no-dupe-style-properties-good)

[Exemples du groupe](./html.md#html-no-dupe-style-properties) · [Fiche de la règle](/rules/reference/html-no-dupe-style-properties.md)

### `html/no-duplicate-class`

Interdire les noms de classes en double dans un attribut class statique

<span id="html-no-duplicate-class-bad"></span>

[Mauvais](./html.md#html-no-duplicate-class-bad)

<span id="html-no-duplicate-class-good"></span>

[Bon](./html.md#html-no-duplicate-class-good)

[Exemples du groupe](./html.md#html-no-duplicate-class) · [Fiche de la règle](/rules/reference/html-no-duplicate-class.md)

### `html/no-duplicate-dt`

Interdire les noms &lt;dt&gt; en double dans &lt;dl&gt;

<span id="html-no-duplicate-dt-bad"></span>

[Mauvais](./html.md#html-no-duplicate-dt-bad)

<span id="html-no-duplicate-dt-good"></span>

[Bon](./html.md#html-no-duplicate-dt-good)

[Exemples du groupe](./html.md#html-no-duplicate-dt) · [Fiche de la règle](/rules/reference/html-no-duplicate-dt.md)

### `html/no-empty-palpable-content`

Interdire les éléments vides qui attendent un contenu visible

<span id="html-no-empty-palpable-content-bad"></span>

[Mauvais](./html.md#html-no-empty-palpable-content-bad)

<span id="html-no-empty-palpable-content-good"></span>

[Bon](./html.md#html-no-empty-palpable-content-good)

[Exemples du groupe](./html.md#html-no-empty-palpable-content) · [Fiche de la règle](/rules/reference/html-no-empty-palpable-content.md)

### `html/require-datetime`

Exiger l’attribut datetime sur l’élément &lt;time&gt;

<span id="html-require-datetime-bad"></span>

[Mauvais](./html.md#html-require-datetime-bad)

<span id="html-require-datetime-good"></span>

[Bon](./html.md#html-require-datetime-good)

[Exemples du groupe](./html.md#html-require-datetime) · [Fiche de la règle](/rules/reference/html-require-datetime.md)

### `musea/no-empty-variant`

Interdire les blocs &lt;variant&gt; vides

<span id="musea-no-empty-variant-bad"></span>

[Mauvais](./musea-and-css.md#musea-no-empty-variant-bad)

<span id="musea-no-empty-variant-good"></span>

[Bon](./musea-and-css.md#musea-no-empty-variant-good)

[Exemples du groupe](./musea-and-css.md#musea-no-empty-variant) · [Fiche de la règle](/rules/reference/musea-no-empty-variant.md)

### `musea/prefer-design-tokens`

Préférer les variables CSS de design tokens aux valeurs primitives codées en dur

<span id="musea-prefer-design-tokens-bad"></span>

[Mauvais](./musea-and-css.md#musea-prefer-design-tokens-bad)

<span id="musea-prefer-design-tokens-good"></span>

[Bon](./musea-and-css.md#musea-prefer-design-tokens-good)

[Exemples du groupe](./musea-and-css.md#musea-prefer-design-tokens) · [Fiche de la règle](/rules/reference/musea-prefer-design-tokens.md)

### `musea/require-component`

Exiger l’attribut component dans le bloc &lt;art&gt;

<span id="musea-require-component-bad"></span>

[Mauvais](./musea-and-css.md#musea-require-component-bad)

<span id="musea-require-component-good"></span>

[Bon](./musea-and-css.md#musea-require-component-good)

[Exemples du groupe](./musea-and-css.md#musea-require-component) · [Fiche de la règle](/rules/reference/musea-require-component.md)

### `musea/require-title`

Exiger l’attribut title dans le bloc &lt;art&gt;

<span id="musea-require-title-bad"></span>

[Mauvais](./musea-and-css.md#musea-require-title-bad)

<span id="musea-require-title-good"></span>

[Bon](./musea-and-css.md#musea-require-title-good)

[Exemples du groupe](./musea-and-css.md#musea-require-title) · [Fiche de la règle](/rules/reference/musea-require-title.md)

### `musea/unique-variant-names`

Exiger des noms de variants uniques

<span id="musea-unique-variant-names-bad"></span>

[Mauvais](./musea-and-css.md#musea-unique-variant-names-bad)

<span id="musea-unique-variant-names-good"></span>

[Bon](./musea-and-css.md#musea-unique-variant-names-good)

[Exemples du groupe](./musea-and-css.md#musea-unique-variant-names) · [Fiche de la règle](/rules/reference/musea-unique-variant-names.md)

### `musea/valid-variant`

Exiger un attribut name dans les blocs &lt;variant&gt;

<span id="musea-valid-variant-bad"></span>

[Mauvais](./musea-and-css.md#musea-valid-variant-bad)

<span id="musea-valid-variant-good"></span>

[Bon](./musea-and-css.md#musea-valid-variant-good)

[Exemples du groupe](./musea-and-css.md#musea-valid-variant) · [Fiche de la règle](/rules/reference/musea-valid-variant.md)

### `nuxt/no-nuxt-config-test-key`

Interdire la clé `test` dans la configuration Nuxt

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

[Mauvais](./ecosystem.md#nuxt-no-nuxt-config-test-key-bad)

<span id="nuxt-no-nuxt-config-test-key-good"></span>

[Bon](./ecosystem.md#nuxt-no-nuxt-config-test-key-good)

[Exemples du groupe](./ecosystem.md#nuxt-no-nuxt-config-test-key) · [Fiche de la règle](/rules/reference/nuxt-no-nuxt-config-test-key.md)

### `nuxt/no-page-meta-runtime-values`

Interdire les valeurs du contexte d’exécution évaluées immédiatement dans `definePageMeta`, extrait dans un chunk séparé à la compilation et exécuté avant le setup du composant

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

[Mauvais](./ecosystem.md#nuxt-no-page-meta-runtime-values-bad)

<span id="nuxt-no-page-meta-runtime-values-good"></span>

[Bon](./ecosystem.md#nuxt-no-page-meta-runtime-values-good)

[Exemples du groupe](./ecosystem.md#nuxt-no-page-meta-runtime-values) · [Fiche de la règle](/rules/reference/nuxt-no-page-meta-runtime-values.md)

### `nuxt/nuxt-config-keys-order`

Préférer l’ordre recommandé des propriétés de configuration Nuxt

<span id="nuxt-nuxt-config-keys-order-bad"></span>

[Mauvais](./ecosystem.md#nuxt-nuxt-config-keys-order-bad)

<span id="nuxt-nuxt-config-keys-order-good"></span>

[Bon](./ecosystem.md#nuxt-nuxt-config-keys-order-good)

[Exemples du groupe](./ecosystem.md#nuxt-nuxt-config-keys-order) · [Fiche de la règle](/rules/reference/nuxt-nuxt-config-keys-order.md)

### `nuxt/prefer-import-meta`

Préférer `import.meta.*` à `process.*`

<span id="nuxt-prefer-import-meta-bad"></span>

[Mauvais](./ecosystem.md#nuxt-prefer-import-meta-bad)

<span id="nuxt-prefer-import-meta-good"></span>

[Bon](./ecosystem.md#nuxt-prefer-import-meta-good)

[Exemples du groupe](./ecosystem.md#nuxt-prefer-import-meta) · [Fiche de la règle](/rules/reference/nuxt-prefer-import-meta.md)

### `petite-vue/no-unsupported-directive`

Interdire les directives non prises en charge par petite-vue

<span id="petite-vue-no-unsupported-directive-bad"></span>

[Mauvais](./petite-vue.md#petite-vue-no-unsupported-directive-bad)

<span id="petite-vue-no-unsupported-directive-good"></span>

[Bon](./petite-vue.md#petite-vue-no-unsupported-directive-good)

[Exemples du groupe](./petite-vue.md#petite-vue-no-unsupported-directive) · [Fiche de la règle](/rules/reference/petite-vue-no-unsupported-directive.md)

### `petite-vue/valid-v-effect`

Exiger une expression non vide pour v-effect

<span id="petite-vue-valid-v-effect-bad"></span>

[Mauvais](./petite-vue.md#petite-vue-valid-v-effect-bad)

<span id="petite-vue-valid-v-effect-good"></span>

[Bon](./petite-vue.md#petite-vue-valid-v-effect-good)

[Exemples du groupe](./petite-vue.md#petite-vue-valid-v-effect) · [Fiche de la règle](/rules/reference/petite-vue-valid-v-effect.md)

### `petite-vue/valid-v-scope`

Exiger que v-scope reçoive un objet littéral

<span id="petite-vue-valid-v-scope-bad"></span>

[Mauvais](./petite-vue.md#petite-vue-valid-v-scope-bad)

<span id="petite-vue-valid-v-scope-good"></span>

[Bon](./petite-vue.md#petite-vue-valid-v-scope-good)

[Exemples du groupe](./petite-vue.md#petite-vue-valid-v-scope) · [Fiche de la règle](/rules/reference/petite-vue-valid-v-scope.md)

### `script/component-options-name-casing`

Imposer PascalCase à l’option `name` du composant

<span id="script-component-options-name-casing-bad"></span>

[Mauvais](./type-and-script.md#script-component-options-name-casing-bad)

<span id="script-component-options-name-casing-good"></span>

[Bon](./type-and-script.md#script-component-options-name-casing-good)

[Exemples du groupe](./type-and-script.md#script-component-options-name-casing) · [Fiche de la règle](/rules/reference/script-component-options-name-casing.md)

### `script/custom-event-name-casing`

Imposer camelCase aux noms des événements personnalisés émis

<span id="script-custom-event-name-casing-bad"></span>

[Mauvais](./type-and-script.md#script-custom-event-name-casing-bad)

<span id="script-custom-event-name-casing-good"></span>

[Bon](./type-and-script.md#script-custom-event-name-casing-good)

[Exemples du groupe](./type-and-script.md#script-custom-event-name-casing) · [Fiche de la règle](/rules/reference/script-custom-event-name-casing.md)

### `script/define-emits-declaration`

Imposer la forme typée defineEmits&lt;{}&gt;() à la place de la forme à l’exécution sous forme de tableau

<span id="script-define-emits-declaration-bad"></span>

[Mauvais](./type-and-script.md#script-define-emits-declaration-bad)

<span id="script-define-emits-declaration-good"></span>

[Bon](./type-and-script.md#script-define-emits-declaration-good)

[Exemples du groupe](./type-and-script.md#script-define-emits-declaration) · [Fiche de la règle](/rules/reference/script-define-emits-declaration.md)

### `script/define-macros-order`

Imposer un ordre cohérent aux macros du compilateur Vue dans &lt;script setup&gt;

<span id="script-define-macros-order-bad"></span>

[Mauvais](./type-and-script.md#script-define-macros-order-bad)

<span id="script-define-macros-order-good"></span>

[Bon](./type-and-script.md#script-define-macros-order-good)

[Exemples du groupe](./type-and-script.md#script-define-macros-order) · [Fiche de la règle](/rules/reference/script-define-macros-order.md)

### `script/define-props-declaration`

Imposer la forme typée defineProps&lt;{ ... }&gt;() à la place de la forme à l’exécution sous forme d’objet

<span id="script-define-props-declaration-bad"></span>

[Mauvais](./type-and-script.md#script-define-props-declaration-bad)

<span id="script-define-props-declaration-good"></span>

[Bon](./type-and-script.md#script-define-props-declaration-good)

[Exemples du groupe](./type-and-script.md#script-define-props-declaration) · [Fiche de la règle](/rules/reference/script-define-props-declaration.md)

### `script/define-props-destructuring`

Imposer un style cohérent de déstructuration de defineProps dans &lt;script setup&gt;

<span id="script-define-props-destructuring-bad"></span>

[Mauvais](./type-and-script.md#script-define-props-destructuring-bad)

<span id="script-define-props-destructuring-good"></span>

[Bon](./type-and-script.md#script-define-props-destructuring-good)

[Exemples du groupe](./type-and-script.md#script-define-props-destructuring) · [Fiche de la règle](/rules/reference/script-define-props-destructuring.md)

### `script/no-arrow-functions-in-watch`

Interdire les fonctions fléchées comme gestionnaires watch de l’Options API

<span id="script-no-arrow-functions-in-watch-bad"></span>

[Mauvais](./type-and-script.md#script-no-arrow-functions-in-watch-bad)

<span id="script-no-arrow-functions-in-watch-good"></span>

[Bon](./type-and-script.md#script-no-arrow-functions-in-watch-good)

[Exemples du groupe](./type-and-script.md#script-no-arrow-functions-in-watch) · [Fiche de la règle](/rules/reference/script-no-arrow-functions-in-watch.md)

### `script/no-async-in-computed`

Interdire les fonctions asynchrones dans les propriétés calculées

<span id="script-no-async-in-computed-bad"></span>

[Mauvais](./type-and-script.md#script-no-async-in-computed-bad)

<span id="script-no-async-in-computed-good"></span>

[Bon](./type-and-script.md#script-no-async-in-computed-good)

[Exemples du groupe](./type-and-script.md#script-no-async-in-computed) · [Fiche de la règle](/rules/reference/script-no-async-in-computed.md)

### `script/no-boolean-default`

Interdire une valeur par défaut sur une prop Boolean

<span id="script-no-boolean-default-bad"></span>

[Mauvais](./type-and-script.md#script-no-boolean-default-bad)

<span id="script-no-boolean-default-good"></span>

[Bon](./type-and-script.md#script-no-boolean-default-good)

[Exemples du groupe](./type-and-script.md#script-no-boolean-default) · [Fiche de la règle](/rules/reference/script-no-boolean-default.md)

### `script/no-deep-destructure-in-props`

Interdire la déstructuration profondément imbriquée dans defineProps

<span id="script-no-deep-destructure-in-props-bad"></span>

[Mauvais](./type-and-script.md#script-no-deep-destructure-in-props-bad)

<span id="script-no-deep-destructure-in-props-good"></span>

[Bon](./type-and-script.md#script-no-deep-destructure-in-props-good)

[Exemples du groupe](./type-and-script.md#script-no-deep-destructure-in-props) · [Fiche de la règle](/rules/reference/script-no-deep-destructure-in-props.md)

### `script/no-deprecated-data-object-declaration`

Interdire un objet littéral comme option data du composant (Vue 3 exige une fonction)

<span id="script-no-deprecated-data-object-declaration-bad"></span>

[Mauvais](./type-and-script.md#script-no-deprecated-data-object-declaration-bad)

<span id="script-no-deprecated-data-object-declaration-good"></span>

[Bon](./type-and-script.md#script-no-deprecated-data-object-declaration-good)

[Exemples du groupe](./type-and-script.md#script-no-deprecated-data-object-declaration) · [Fiche de la règle](/rules/reference/script-no-deprecated-data-object-declaration.md)

### `script/no-deprecated-destroyed-lifecycle`

Interdire les hooks de cycle de vie dépréciés destroyed et beforeDestroy

<span id="script-no-deprecated-destroyed-lifecycle-bad"></span>

[Mauvais](./type-and-script.md#script-no-deprecated-destroyed-lifecycle-bad)

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

[Bon](./type-and-script.md#script-no-deprecated-destroyed-lifecycle-good)

[Exemples du groupe](./type-and-script.md#script-no-deprecated-destroyed-lifecycle) · [Fiche de la règle](/rules/reference/script-no-deprecated-destroyed-lifecycle.md)

### `script/no-deprecated-dollar-listeners-api`

Interdire la propriété d’instance $listeners supprimée dans Vue 3 (fusionnée dans $attrs)

<span id="script-no-deprecated-dollar-listeners-api-bad"></span>

[Mauvais](./type-and-script.md#script-no-deprecated-dollar-listeners-api-bad)

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

[Bon](./type-and-script.md#script-no-deprecated-dollar-listeners-api-good)

[Exemples du groupe](./type-and-script.md#script-no-deprecated-dollar-listeners-api) · [Fiche de la règle](/rules/reference/script-no-deprecated-dollar-listeners-api.md)

### `script/no-deprecated-dollar-scopedslots-api`

Interdire la propriété d’instance $scopedSlots supprimée dans Vue 3 (utiliser $slots)

<span id="script-no-deprecated-dollar-scopedslots-api-bad"></span>

[Mauvais](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api-bad)

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

[Bon](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api-good)

[Exemples du groupe](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api) · [Fiche de la règle](/rules/reference/script-no-deprecated-dollar-scopedslots-api.md)

### `script/no-deprecated-events-api`

Interdire l’API d’événements de Vue 2 supprimée ($on / $off / $once)

<span id="script-no-deprecated-events-api-bad"></span>

[Mauvais](./type-and-script.md#script-no-deprecated-events-api-bad)

<span id="script-no-deprecated-events-api-good"></span>

[Bon](./type-and-script.md#script-no-deprecated-events-api-good)

[Exemples du groupe](./type-and-script.md#script-no-deprecated-events-api) · [Fiche de la règle](/rules/reference/script-no-deprecated-events-api.md)

### `script/no-deprecated-props-default-this`

Interdire `this` dans une fonction de valeur par défaut ou de validation de prop (supprimé dans Vue 3)

<span id="script-no-deprecated-props-default-this-bad"></span>

[Mauvais](./type-and-script.md#script-no-deprecated-props-default-this-bad)

<span id="script-no-deprecated-props-default-this-good"></span>

[Bon](./type-and-script.md#script-no-deprecated-props-default-this-good)

[Exemples du groupe](./type-and-script.md#script-no-deprecated-props-default-this) · [Fiche de la règle](/rules/reference/script-no-deprecated-props-default-this.md)

### `script/no-dupe-keys`

Interdire les clés dupliquées entre props/data/computed/methods/setup/inject de l’Options API

<span id="script-no-dupe-keys-bad"></span>

[Mauvais](./type-and-script.md#script-no-dupe-keys-bad)

<span id="script-no-dupe-keys-good"></span>

[Bon](./type-and-script.md#script-no-dupe-keys-good)

[Exemples du groupe](./type-and-script.md#script-no-dupe-keys) · [Fiche de la règle](/rules/reference/script-no-dupe-keys.md)

### `script/no-duplicate-attr-inheritance`

Signaler un composant qui applique deux fois ses attributs transmis automatiquement

<span id="script-no-duplicate-attr-inheritance-bad"></span>

[Mauvais](./type-and-script.md#script-no-duplicate-attr-inheritance-bad)

<span id="script-no-duplicate-attr-inheritance-good"></span>

[Bon](./type-and-script.md#script-no-duplicate-attr-inheritance-good)

[Exemples du groupe](./type-and-script.md#script-no-duplicate-attr-inheritance) · [Fiche de la règle](/rules/reference/script-no-duplicate-attr-inheritance.md)

### `script/no-export-in-script-setup`

Interdire les instructions export dans &lt;script setup&gt;

<span id="script-no-export-in-script-setup-bad"></span>

[Mauvais](./type-and-script.md#script-no-export-in-script-setup-bad)

<span id="script-no-export-in-script-setup-good"></span>

[Bon](./type-and-script.md#script-no-export-in-script-setup-good)

[Exemples du groupe](./type-and-script.md#script-no-export-in-script-setup) · [Fiche de la règle](/rules/reference/script-no-export-in-script-setup.md)

### `script/no-get-current-instance`

Interdire getCurrentInstance() en mode Vapor (renvoie null)

<span id="script-no-get-current-instance-bad"></span>

[Mauvais](./type-and-script.md#script-no-get-current-instance-bad)

<span id="script-no-get-current-instance-good"></span>

[Bon](./type-and-script.md#script-no-get-current-instance-good)

[Exemples du groupe](./type-and-script.md#script-no-get-current-instance) · [Fiche de la règle](/rules/reference/script-no-get-current-instance.md)

### `script/no-import-compiler-macros`

Interdire l’import des macros du compilateur Vue importées automatiquement

<span id="script-no-import-compiler-macros-bad"></span>

[Mauvais](./type-and-script.md#script-no-import-compiler-macros-bad)

<span id="script-no-import-compiler-macros-good"></span>

[Bon](./type-and-script.md#script-no-import-compiler-macros-good)

[Exemples du groupe](./type-and-script.md#script-no-import-compiler-macros) · [Fiche de la règle](/rules/reference/script-no-import-compiler-macros.md)

### `script/no-internal-imports`

Interdire les imports depuis les modules internes de Vue

<span id="script-no-internal-imports-bad"></span>

[Mauvais](./type-and-script.md#script-no-internal-imports-bad)

<span id="script-no-internal-imports-good"></span>

[Bon](./type-and-script.md#script-no-internal-imports-good)

[Exemples du groupe](./type-and-script.md#script-no-internal-imports) · [Fiche de la règle](/rules/reference/script-no-internal-imports.md)

### `script/no-multiple-slot-args`

Interdire de passer plusieurs arguments à un appel de fonction de slot à portée

<span id="script-no-multiple-slot-args-bad"></span>

[Mauvais](./type-and-script.md#script-no-multiple-slot-args-bad)

<span id="script-no-multiple-slot-args-good"></span>

[Bon](./type-and-script.md#script-no-multiple-slot-args-good)

[Exemples du groupe](./type-and-script.md#script-no-multiple-slot-args) · [Fiche de la règle](/rules/reference/script-no-multiple-slot-args.md)

### `script/no-next-tick`

Interdire l’utilisation de nextTick() dans les composants destinés à Vapor

<span id="script-no-next-tick-bad"></span>

[Mauvais](./type-and-script.md#script-no-next-tick-bad)

<span id="script-no-next-tick-good"></span>

[Bon](./type-and-script.md#script-no-next-tick-good)

[Exemples du groupe](./type-and-script.md#script-no-next-tick) · [Fiche de la règle](/rules/reference/script-no-next-tick.md)

### `script/no-options-api`

Interdire les formes de l’Options API en mode Vapor

<span id="script-no-options-api-bad"></span>

[Mauvais](./type-and-script.md#script-no-options-api-bad)

<span id="script-no-options-api-good"></span>

[Bon](./type-and-script.md#script-no-options-api-good)

[Exemples du groupe](./type-and-script.md#script-no-options-api) · [Fiche de la règle](/rules/reference/script-no-options-api.md)

### `script/no-potential-component-option-typo`

Signaler les fautes de frappe probables dans les noms d’options de composant de l’Options API

<span id="script-no-potential-component-option-typo-bad"></span>

[Mauvais](./type-and-script.md#script-no-potential-component-option-typo-bad)

<span id="script-no-potential-component-option-typo-good"></span>

[Bon](./type-and-script.md#script-no-potential-component-option-typo-good)

[Exemples du groupe](./type-and-script.md#script-no-potential-component-option-typo) · [Fiche de la règle](/rules/reference/script-no-potential-component-option-typo.md)

### `script/no-reactive-destructure`

Interdire la déstructuration d’objets réactifs qui fait perdre la réactivité

<span id="script-no-reactive-destructure-bad"></span>

[Mauvais](./type-and-script.md#script-no-reactive-destructure-bad)

<span id="script-no-reactive-destructure-good"></span>

[Bon](./type-and-script.md#script-no-reactive-destructure-good)

[Exemples du groupe](./type-and-script.md#script-no-reactive-destructure) · [Fiche de la règle](/rules/reference/script-no-reactive-destructure.md)

### `script/no-ref-as-operand`

Exiger l’accès via `.value` aux variables liées à une ref lorsqu’elles servent d’opérande

<span id="script-no-ref-as-operand-bad"></span>

[Mauvais](./type-and-script.md#script-no-ref-as-operand-bad)

<span id="script-no-ref-as-operand-good"></span>

[Bon](./type-and-script.md#script-no-ref-as-operand-good)

[Exemples du groupe](./type-and-script.md#script-no-ref-as-operand) · [Fiche de la règle](/rules/reference/script-no-ref-as-operand.md)

### `script/no-required-prop-with-default`

Interdire une prop qui possède à la fois required: true et une valeur par défaut

<span id="script-no-required-prop-with-default-bad"></span>

[Mauvais](./type-and-script.md#script-no-required-prop-with-default-bad)

<span id="script-no-required-prop-with-default-good"></span>

[Bon](./type-and-script.md#script-no-required-prop-with-default-good)

[Exemples du groupe](./type-and-script.md#script-no-required-prop-with-default) · [Fiche de la règle](/rules/reference/script-no-required-prop-with-default.md)

### `script/no-reserved-identifiers`

Interdire les identifiants réservés du compilateur Vue

<span id="script-no-reserved-identifiers-bad"></span>

[Mauvais](./type-and-script.md#script-no-reserved-identifiers-bad)

<span id="script-no-reserved-identifiers-good"></span>

[Bon](./type-and-script.md#script-no-reserved-identifiers-good)

[Exemples du groupe](./type-and-script.md#script-no-reserved-identifiers) · [Fiche de la règle](/rules/reference/script-no-reserved-identifiers.md)

### `script/no-reserved-keys`

Interdire les noms réservés par Vue comme clés de props/data/computed/methods/setup/inject de l’Options API

<span id="script-no-reserved-keys-bad"></span>

[Mauvais](./type-and-script.md#script-no-reserved-keys-bad)

<span id="script-no-reserved-keys-good"></span>

[Bon](./type-and-script.md#script-no-reserved-keys-good)

[Exemples du groupe](./type-and-script.md#script-no-reserved-keys) · [Fiche de la règle](/rules/reference/script-no-reserved-keys.md)

### `script/no-reserved-props`

Interdire les noms réservés dans la déclaration des props d’un composant

<span id="script-no-reserved-props-bad"></span>

[Mauvais](./type-and-script.md#script-no-reserved-props-bad)

<span id="script-no-reserved-props-good"></span>

[Bon](./type-and-script.md#script-no-reserved-props-good)

[Exemples du groupe](./type-and-script.md#script-no-reserved-props) · [Fiche de la règle](/rules/reference/script-no-reserved-props.md)

### `script/no-restricted-globals`

Interdire les références aux variables globales de l’environnement d’exécution qui doivent passer par une couche d’encapsulation typée

<span id="script-no-restricted-globals-bad"></span>

[Mauvais](./type-and-script.md#script-no-restricted-globals-bad)

<span id="script-no-restricted-globals-good"></span>

[Bon](./type-and-script.md#script-no-restricted-globals-good)

[Exemples du groupe](./type-and-script.md#script-no-restricted-globals) · [Fiche de la règle](/rules/reference/script-no-restricted-globals.md)

### `script/no-restricted-members`

Interdire les accès aux membres object.property configurés par le projet

<span id="script-no-restricted-members-bad"></span>

[Mauvais](./type-and-script.md#script-no-restricted-members-bad)

<span id="script-no-restricted-members-good"></span>

[Bon](./type-and-script.md#script-no-restricted-members-good)

[Exemples du groupe](./type-and-script.md#script-no-restricted-members) · [Fiche de la règle](/rules/reference/script-no-restricted-members.md)

### `script/no-side-effects-in-computed-properties`

Interdire les effets de bord dans les getters calculés de l’Options API

<span id="script-no-side-effects-in-computed-properties-bad"></span>

[Mauvais](./type-and-script.md#script-no-side-effects-in-computed-properties-bad)

<span id="script-no-side-effects-in-computed-properties-good"></span>

[Bon](./type-and-script.md#script-no-side-effects-in-computed-properties-good)

[Exemples du groupe](./type-and-script.md#script-no-side-effects-in-computed-properties) · [Fiche de la règle](/rules/reference/script-no-side-effects-in-computed-properties.md)

### `script/no-top-level-ref-in-script`

Interdire ref/reactive au niveau supérieur pour éviter la contamination de l’état entre requêtes

<span id="script-no-top-level-ref-in-script-bad"></span>

[Mauvais](./type-and-script.md#script-no-top-level-ref-in-script-bad)

<span id="script-no-top-level-ref-in-script-good"></span>

[Bon](./type-and-script.md#script-no-top-level-ref-in-script-good)

[Exemples du groupe](./type-and-script.md#script-no-top-level-ref-in-script) · [Fiche de la règle](/rules/reference/script-no-top-level-ref-in-script.md)

### `script/no-unstable-nested-components`

Interdire les définitions de composants dans les fonctions setup ou de rendu

<span id="script-no-unstable-nested-components-bad"></span>

[Mauvais](./type-and-script.md#script-no-unstable-nested-components-bad)

<span id="script-no-unstable-nested-components-good"></span>

[Bon](./type-and-script.md#script-no-unstable-nested-components-good)

[Exemples du groupe](./type-and-script.md#script-no-unstable-nested-components) · [Fiche de la règle](/rules/reference/script-no-unstable-nested-components.md)

### `script/no-unused-emit-declarations`

Signaler les événements déclarés qui ne sont jamais émis

<span id="script-no-unused-emit-declarations-bad"></span>

[Mauvais](./type-and-script.md#script-no-unused-emit-declarations-bad)

<span id="script-no-unused-emit-declarations-good"></span>

[Bon](./type-and-script.md#script-no-unused-emit-declarations-good)

[Exemples du groupe](./type-and-script.md#script-no-unused-emit-declarations) · [Fiche de la règle](/rules/reference/script-no-unused-emit-declarations.md)

### `script/no-use-computed-property-like-method`

Interdire d’appeler une propriété calculée de l’Options API comme une méthode

<span id="script-no-use-computed-property-like-method-bad"></span>

[Mauvais](./type-and-script.md#script-no-use-computed-property-like-method-bad)

<span id="script-no-use-computed-property-like-method-good"></span>

[Bon](./type-and-script.md#script-no-use-computed-property-like-method-good)

[Exemples du groupe](./type-and-script.md#script-no-use-computed-property-like-method) · [Fiche de la règle](/rules/reference/script-no-use-computed-property-like-method.md)

### `script/no-with-defaults`

Déconseiller withDefaults au profit des valeurs par défaut dans la déstructuration (Vue 3.5+)

<span id="script-no-with-defaults-bad"></span>

[Mauvais](./type-and-script.md#script-no-with-defaults-bad)

<span id="script-no-with-defaults-good"></span>

[Bon](./type-and-script.md#script-no-with-defaults-good)

[Exemples du groupe](./type-and-script.md#script-no-with-defaults) · [Fiche de la règle](/rules/reference/script-no-with-defaults.md)

### `script/prefer-computed`

Préférer computed() pour l’état réactif dérivé

<span id="script-prefer-computed-bad"></span>

[Mauvais](./type-and-script.md#script-prefer-computed-bad)

<span id="script-prefer-computed-good"></span>

[Bon](./type-and-script.md#script-prefer-computed-good)

[Exemples du groupe](./type-and-script.md#script-prefer-computed) · [Fiche de la règle](/rules/reference/script-prefer-computed.md)

### `script/prefer-define-options`

Préférer defineOptions() à un &lt;script&gt; ordinaire qui ne définit que name/inheritAttrs

<span id="script-prefer-define-options-bad"></span>

[Mauvais](./type-and-script.md#script-prefer-define-options-bad)

<span id="script-prefer-define-options-good"></span>

[Bon](./type-and-script.md#script-prefer-define-options-good)

[Exemples du groupe](./type-and-script.md#script-prefer-define-options) · [Fiche de la règle](/rules/reference/script-prefer-define-options.md)

### `script/prefer-import-from-vue`

Préférer les imports depuis 'vue' plutôt que depuis les packages internes

<span id="script-prefer-import-from-vue-bad"></span>

[Mauvais](./type-and-script.md#script-prefer-import-from-vue-bad)

<span id="script-prefer-import-from-vue-good"></span>

[Bon](./type-and-script.md#script-prefer-import-from-vue-good)

[Exemples du groupe](./type-and-script.md#script-prefer-import-from-vue) · [Fiche de la règle](/rules/reference/script-prefer-import-from-vue.md)

### `script/prefer-ref-over-reactive`

Recommander ref() plutôt que reactive() pour gérer l’état

<span id="script-prefer-ref-over-reactive-bad"></span>

[Mauvais](./type-and-script.md#script-prefer-ref-over-reactive-bad)

<span id="script-prefer-ref-over-reactive-good"></span>

[Bon](./type-and-script.md#script-prefer-ref-over-reactive-good)

[Exemples du groupe](./type-and-script.md#script-prefer-ref-over-reactive) · [Fiche de la règle](/rules/reference/script-prefer-ref-over-reactive.md)

### `script/prefer-use-attrs`

Recommander useAttrs() plutôt que context.attrs

<span id="script-prefer-use-attrs-bad"></span>

[Mauvais](./type-and-script.md#script-prefer-use-attrs-bad)

<span id="script-prefer-use-attrs-good"></span>

[Bon](./type-and-script.md#script-prefer-use-attrs-good)

[Exemples du groupe](./type-and-script.md#script-prefer-use-attrs) · [Fiche de la règle](/rules/reference/script-prefer-use-attrs.md)

### `script/prefer-use-id`

Recommander useId() pour générer des identifiants uniques (Vue 3.5+)

<span id="script-prefer-use-id-bad"></span>

[Mauvais](./type-and-script.md#script-prefer-use-id-bad)

<span id="script-prefer-use-id-good"></span>

[Bon](./type-and-script.md#script-prefer-use-id-good)

[Exemples du groupe](./type-and-script.md#script-prefer-use-id) · [Fiche de la règle](/rules/reference/script-prefer-use-id.md)

### `script/prefer-use-slots`

Recommander useSlots() plutôt que context.slots

<span id="script-prefer-use-slots-bad"></span>

[Mauvais](./type-and-script.md#script-prefer-use-slots-bad)

<span id="script-prefer-use-slots-good"></span>

[Bon](./type-and-script.md#script-prefer-use-slots-good)

[Exemples du groupe](./type-and-script.md#script-prefer-use-slots) · [Fiche de la règle](/rules/reference/script-prefer-use-slots.md)

### `script/prefer-use-template-ref`

Recommander useTemplateRef plutôt que ref pour les références de template (Vue 3.5+)

<span id="script-prefer-use-template-ref-bad"></span>

[Mauvais](./type-and-script.md#script-prefer-use-template-ref-bad)

<span id="script-prefer-use-template-ref-good"></span>

[Bon](./type-and-script.md#script-prefer-use-template-ref-good)

[Exemples du groupe](./type-and-script.md#script-prefer-use-template-ref) · [Fiche de la règle](/rules/reference/script-prefer-use-template-ref.md)

### `script/require-default-prop`

Exiger une valeur par défaut pour chaque prop facultative non booléenne

<span id="script-require-default-prop-bad"></span>

[Mauvais](./type-and-script.md#script-require-default-prop-bad)

<span id="script-require-default-prop-good"></span>

[Bon](./type-and-script.md#script-require-default-prop-good)

[Exemples du groupe](./type-and-script.md#script-require-default-prop) · [Fiche de la règle](/rules/reference/script-require-default-prop.md)

### `script/require-explicit-emits`

Exiger la déclaration des événements émis dans defineEmits ou l’option emits

<span id="script-require-explicit-emits-bad"></span>

[Mauvais](./type-and-script.md#script-require-explicit-emits-bad)

<span id="script-require-explicit-emits-good"></span>

[Bon](./type-and-script.md#script-require-explicit-emits-good)

[Exemples du groupe](./type-and-script.md#script-require-explicit-emits) · [Fiche de la règle](/rules/reference/script-require-explicit-emits.md)

### `script/require-explicit-slots`

Exiger que les slots utilisés via useSlots() soient explicitement typés avec defineSlots&lt;...&gt;()

<span id="script-require-explicit-slots-bad"></span>

[Mauvais](./type-and-script.md#script-require-explicit-slots-bad)

<span id="script-require-explicit-slots-good"></span>

[Bon](./type-and-script.md#script-require-explicit-slots-good)

[Exemples du groupe](./type-and-script.md#script-require-explicit-slots) · [Fiche de la règle](/rules/reference/script-require-explicit-slots.md)

### `script/require-function-return-type`

Exiger des annotations de type de retour sur les fonctions

<span id="script-require-function-return-type-bad"></span>

[Mauvais](./type-and-script.md#script-require-function-return-type-bad)

<span id="script-require-function-return-type-good"></span>

[Bon](./type-and-script.md#script-require-function-return-type-good)

[Exemples du groupe](./type-and-script.md#script-require-function-return-type) · [Fiche de la règle](/rules/reference/script-require-function-return-type.md)

### `script/require-prop-type-constructor`

Exiger que les valeurs `type` des props soient des constructeurs plutôt que des chaînes littérales

<span id="script-require-prop-type-constructor-bad"></span>

[Mauvais](./type-and-script.md#script-require-prop-type-constructor-bad)

<span id="script-require-prop-type-constructor-good"></span>

[Bon](./type-and-script.md#script-require-prop-type-constructor-good)

[Exemples du groupe](./type-and-script.md#script-require-prop-type-constructor) · [Fiche de la règle](/rules/reference/script-require-prop-type-constructor.md)

### `script/require-prop-types`

Exiger que chaque prop déclare un type

<span id="script-require-prop-types-bad"></span>

[Mauvais](./type-and-script.md#script-require-prop-types-bad)

<span id="script-require-prop-types-good"></span>

[Bon](./type-and-script.md#script-require-prop-types-good)

[Exemples du groupe](./type-and-script.md#script-require-prop-types) · [Fiche de la règle](/rules/reference/script-require-prop-types.md)

### `script/require-symbol-provide`

Recommander Symbol comme clé d’injection pour provide/inject

<span id="script-require-symbol-provide-bad"></span>

[Mauvais](./type-and-script.md#script-require-symbol-provide-bad)

<span id="script-require-symbol-provide-good"></span>

[Bon](./type-and-script.md#script-require-symbol-provide-good)

[Exemples du groupe](./type-and-script.md#script-require-symbol-provide) · [Fiche de la règle](/rules/reference/script-require-symbol-provide.md)

### `script/require-typed-object-prop`

Exiger un type explicite sur une prop dont le type à l’exécution est `Object` ou `Array`

<span id="script-require-typed-object-prop-bad"></span>

[Mauvais](./type-and-script.md#script-require-typed-object-prop-bad)

<span id="script-require-typed-object-prop-good"></span>

[Bon](./type-and-script.md#script-require-typed-object-prop-good)

[Exemples du groupe](./type-and-script.md#script-require-typed-object-prop) · [Fiche de la règle](/rules/reference/script-require-typed-object-prop.md)

### `script/require-typed-ref`

Exiger un argument de type explicite sur un ref() initialisé sans valeur, avec null ou avec undefined

<span id="script-require-typed-ref-bad"></span>

[Mauvais](./type-and-script.md#script-require-typed-ref-bad)

<span id="script-require-typed-ref-good"></span>

[Bon](./type-and-script.md#script-require-typed-ref-good)

[Exemples du groupe](./type-and-script.md#script-require-typed-ref) · [Fiche de la règle](/rules/reference/script-require-typed-ref.md)

### `script/require-valid-default-prop`

Exiger que la valeur par défaut d’une prop soit valide pour son type déclaré

<span id="script-require-valid-default-prop-bad"></span>

[Mauvais](./type-and-script.md#script-require-valid-default-prop-bad)

<span id="script-require-valid-default-prop-good"></span>

[Bon](./type-and-script.md#script-require-valid-default-prop-good)

[Exemples du groupe](./type-and-script.md#script-require-valid-default-prop) · [Fiche de la règle](/rules/reference/script-require-valid-default-prop.md)

### `script/return-in-computed-property`

Exiger une valeur de retour dans chaque getter calculé

<span id="script-return-in-computed-property-bad"></span>

[Mauvais](./type-and-script.md#script-return-in-computed-property-bad)

<span id="script-return-in-computed-property-good"></span>

[Bon](./type-and-script.md#script-return-in-computed-property-good)

[Exemples du groupe](./type-and-script.md#script-return-in-computed-property) · [Fiche de la règle](/rules/reference/script-return-in-computed-property.md)

### `script/return-in-emits-validator`

Exiger une valeur de retour dans chaque validateur emits de l’Options API

<span id="script-return-in-emits-validator-bad"></span>

[Mauvais](./type-and-script.md#script-return-in-emits-validator-bad)

<span id="script-return-in-emits-validator-good"></span>

[Bon](./type-and-script.md#script-return-in-emits-validator-good)

[Exemples du groupe](./type-and-script.md#script-return-in-emits-validator) · [Fiche de la règle](/rules/reference/script-return-in-emits-validator.md)

### `script/valid-define-emits`

Imposer une utilisation valide de defineEmits() (pas d’arguments de type et d’exécution combinés, pas de références locales, un seul appel)

<span id="script-valid-define-emits-bad"></span>

[Mauvais](./type-and-script.md#script-valid-define-emits-bad)

<span id="script-valid-define-emits-good"></span>

[Bon](./type-and-script.md#script-valid-define-emits-good)

[Exemples du groupe](./type-and-script.md#script-valid-define-emits) · [Fiche de la règle](/rules/reference/script-valid-define-emits.md)

### `script/valid-define-options`

Imposer une utilisation valide de defineOptions() (un seul argument objet, sans props/emits/expose/slots)

<span id="script-valid-define-options-bad"></span>

[Mauvais](./type-and-script.md#script-valid-define-options-bad)

<span id="script-valid-define-options-good"></span>

[Bon](./type-and-script.md#script-valid-define-options-good)

[Exemples du groupe](./type-and-script.md#script-valid-define-options) · [Fiche de la règle](/rules/reference/script-valid-define-options.md)

### `script/valid-define-props`

Imposer une utilisation valide de defineProps() (un seul appel, pas d’arguments de type et d’exécution combinés, pas de références locales)

<span id="script-valid-define-props-bad"></span>

[Mauvais](./type-and-script.md#script-valid-define-props-bad)

<span id="script-valid-define-props-good"></span>

[Bon](./type-and-script.md#script-valid-define-props-good)

[Exemples du groupe](./type-and-script.md#script-valid-define-props) · [Fiche de la règle](/rules/reference/script-valid-define-props.md)

### `script/valid-next-tick`

Exiger que le résultat d’un appel nextTick() soit attendu, chaîné ou associé à un callback

<span id="script-valid-next-tick-bad"></span>

[Mauvais](./type-and-script.md#script-valid-next-tick-bad)

<span id="script-valid-next-tick-good"></span>

[Bon](./type-and-script.md#script-valid-next-tick-good)

[Exemples du groupe](./type-and-script.md#script-valid-next-tick) · [Fiche de la règle](/rules/reference/script-valid-next-tick.md)

### `ssr/no-browser-globals-in-ssr`

Interdire les variables globales propres au navigateur dans un contexte SSR

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

[Mauvais](./ssr.md#ssr-no-browser-globals-in-ssr-bad)

<span id="ssr-no-browser-globals-in-ssr-good"></span>

[Bon](./ssr.md#ssr-no-browser-globals-in-ssr-good)

[Exemples du groupe](./ssr.md#ssr-no-browser-globals-in-ssr) · [Fiche de la règle](/rules/reference/ssr-no-browser-globals-in-ssr.md)

### `ssr/no-hydration-mismatch`

Interdire les valeurs non déterministes qui causent des divergences d’hydratation

<span id="ssr-no-hydration-mismatch-bad"></span>

[Mauvais](./ssr.md#ssr-no-hydration-mismatch-bad)

<span id="ssr-no-hydration-mismatch-good"></span>

[Bon](./ssr.md#ssr-no-hydration-mismatch-good)

[Exemples du groupe](./ssr.md#ssr-no-hydration-mismatch) · [Fiche de la règle](/rules/reference/ssr-no-hydration-mismatch.md)

### `type/no-floating-promises`

Interdire les Promises laissées sans traitement

<span id="type-no-floating-promises-bad"></span>

[Mauvais](./type-and-script.md#type-no-floating-promises-bad)

<span id="type-no-floating-promises-good"></span>

[Bon](./type-and-script.md#type-no-floating-promises-good)

[Exemples du groupe](./type-and-script.md#type-no-floating-promises) · [Fiche de la règle](/rules/reference/type-no-floating-promises.md)

### `type/no-reactivity-loss`

Interdire les instantanés ordinaires de valeurs réactives lors des affectations et des appels

<span id="type-no-reactivity-loss-bad"></span>

[Mauvais](./type-and-script.md#type-no-reactivity-loss-bad)

<span id="type-no-reactivity-loss-good"></span>

[Bon](./type-and-script.md#type-no-reactivity-loss-good)

[Exemples du groupe](./type-and-script.md#type-no-reactivity-loss) · [Fiche de la règle](/rules/reference/type-no-reactivity-loss.md)

### `type/no-unsafe-template-binding`

Interdire les liaisons de template dont le type résolu est non sûr

<span id="type-no-unsafe-template-binding-bad"></span>

[Mauvais](./type-and-script.md#type-no-unsafe-template-binding-bad)

<span id="type-no-unsafe-template-binding-good"></span>

[Bon](./type-and-script.md#type-no-unsafe-template-binding-good)

[Exemples du groupe](./type-and-script.md#type-no-unsafe-template-binding) · [Fiche de la règle](/rules/reference/type-no-unsafe-template-binding.md)

### `type/require-typed-emits`

Exiger une définition de type pour defineEmits

<span id="type-require-typed-emits-bad"></span>

[Mauvais](./type-and-script.md#type-require-typed-emits-bad)

<span id="type-require-typed-emits-good"></span>

[Bon](./type-and-script.md#type-require-typed-emits-good)

[Exemples du groupe](./type-and-script.md#type-require-typed-emits) · [Fiche de la règle](/rules/reference/type-require-typed-emits.md)

### `type/require-typed-props`

Exiger une définition de type pour defineProps

<span id="type-require-typed-props-bad"></span>

[Mauvais](./type-and-script.md#type-require-typed-props-bad)

<span id="type-require-typed-props-good"></span>

[Bon](./type-and-script.md#type-require-typed-props-good)

[Exemples du groupe](./type-and-script.md#type-require-typed-props) · [Fiche de la règle](/rules/reference/type-require-typed-props.md)

### `type/strict-boolean-expressions`

Exiger des expressions booléennes sûres dans les conditions des scripts et des templates

<span id="type-strict-boolean-expressions-bad"></span>

[Mauvais](./type-and-script.md#type-strict-boolean-expressions-bad)

<span id="type-strict-boolean-expressions-good"></span>

[Bon](./type-and-script.md#type-strict-boolean-expressions-good)

[Exemples du groupe](./type-and-script.md#type-strict-boolean-expressions) · [Fiche de la règle](/rules/reference/type-strict-boolean-expressions.md)

### `vapor/no-inline-template`

Interdire l’attribut obsolète inline-template

<span id="vapor-no-inline-template-bad"></span>

[Mauvais](./vapor.md#vapor-no-inline-template-bad)

<span id="vapor-no-inline-template-good"></span>

[Bon](./vapor.md#vapor-no-inline-template-good)

[Exemples du groupe](./vapor.md#vapor-no-inline-template) · [Fiche de la règle](/rules/reference/vapor-no-inline-template.md)

### `vapor/no-vue-lifecycle-events`

Interdire les événements de cycle de vie @vue:xxx par élément (non pris en charge dans Vapor)

<span id="vapor-no-vue-lifecycle-events-bad"></span>

[Mauvais](./vapor.md#vapor-no-vue-lifecycle-events-bad)

<span id="vapor-no-vue-lifecycle-events-good"></span>

[Bon](./vapor.md#vapor-no-vue-lifecycle-events-good)

[Exemples du groupe](./vapor.md#vapor-no-vue-lifecycle-events) · [Fiche de la règle](/rules/reference/vapor-no-vue-lifecycle-events.md)

### `vapor/prefer-static-class`

Préférer une classe statique à une liaison de classe dynamique pour les chaînes littérales

<span id="vapor-prefer-static-class-bad"></span>

[Mauvais](./vapor.md#vapor-prefer-static-class-bad)

<span id="vapor-prefer-static-class-good"></span>

[Bon](./vapor.md#vapor-prefer-static-class-good)

[Exemples du groupe](./vapor.md#vapor-prefer-static-class) · [Fiche de la règle](/rules/reference/vapor-prefer-static-class.md)

### `vapor/require-vapor-attribute`

Suggérer l’ajout de l’attribut vapor à script setup

<span id="vapor-require-vapor-attribute-bad"></span>

[Mauvais](./vapor.md#vapor-require-vapor-attribute-bad)

<span id="vapor-require-vapor-attribute-good"></span>

[Bon](./vapor.md#vapor-require-vapor-attribute-good)

[Exemples du groupe](./vapor.md#vapor-require-vapor-attribute) · [Fiche de la règle](/rules/reference/vapor-require-vapor-attribute.md)

### `vize:croquis/cf/array-mutation`

Un tableau est modifié par index, ce qu’un tableau réactif ne suit pas.

<span id="vize-croquis-cf-array-mutation-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-array-mutation-bad)

<span id="vize-croquis-cf-array-mutation-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-array-mutation-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-array-mutation) · [Fiche de la règle](/rules/project/vize-croquis-cf-array-mutation.md)

### `vize:croquis/cf/async-boundary`

Un état réactif traverse une frontière asynchrone et peut être observé dans un état périmé.

<span id="vize-croquis-cf-async-boundary-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-async-boundary-bad)

<span id="vize-croquis-cf-async-boundary-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-async-boundary-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-async-boundary) · [Fiche de la règle](/rules/project/vize-croquis-cf-async-boundary.md)

### `vize:croquis/cf/async-no-suspense`

Un composant asynchrone est rendu sans frontière Suspense.

<span id="vize-croquis-cf-async-no-suspense-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-async-no-suspense-bad)

<span id="vize-croquis-cf-async-no-suspense-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-async-no-suspense-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-async-no-suspense) · [Fiche de la règle](/rules/project/vize-croquis-cf-async-no-suspense.md)

### `vize:croquis/cf/browser-api-ssr`

Une API réservée au navigateur est utilisée alors que le composant peut être rendu sur le serveur.

<span id="vize-croquis-cf-browser-api-ssr-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-browser-api-ssr-bad)

<span id="vize-croquis-cf-browser-api-ssr-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-browser-api-ssr-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-browser-api-ssr) · [Fiche de la règle](/rules/project/vize-croquis-cf-browser-api-ssr.md)

### `vize:croquis/cf/circular-dep`

Des composants s’importent mutuellement en formant un cycle.

<span id="vize-croquis-cf-circular-dep-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-circular-dep-bad)

<span id="vize-croquis-cf-circular-dep-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-circular-dep-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-circular-dep) · [Fiche de la règle](/rules/project/vize-croquis-cf-circular-dep.md)

### `vize:croquis/cf/circular-reactive-dependency`

Des calculs réactifs dépendent les uns des autres en formant un cycle.

<span id="vize-croquis-cf-circular-reactive-dependency-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-circular-reactive-dependency-bad)

<span id="vize-croquis-cf-circular-reactive-dependency-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-circular-reactive-dependency-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-circular-reactive-dependency) · [Fiche de la règle](/rules/project/vize-croquis-cf-circular-reactive-dependency.md)

### `vize:croquis/cf/closure-captures-reactive`

Une fermeture capture une valeur réactive et ne verra pas les mises à jour ultérieures.

<span id="vize-croquis-cf-closure-captures-reactive-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-closure-captures-reactive-bad)

<span id="vize-croquis-cf-closure-captures-reactive-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-closure-captures-reactive-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-closure-captures-reactive) · [Fiche de la règle](/rules/project/vize-croquis-cf-closure-captures-reactive.md)

### `vize:croquis/cf/composable-outside-setup`

Un composable est appelé en dehors de `setup`.

<span id="vize-croquis-cf-composable-outside-setup-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-composable-outside-setup-bad)

<span id="vize-croquis-cf-composable-outside-setup-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-composable-outside-setup-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-composable-outside-setup) · [Fiche de la règle](/rules/project/vize-croquis-cf-composable-outside-setup.md)

### `vize:croquis/cf/computed-side-effects`

L’accesseur d’une propriété calculée écrit dans l’état ou produit un autre effet de bord.

<span id="vize-croquis-cf-computed-side-effects-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-computed-side-effects-bad)

<span id="vize-croquis-cf-computed-side-effects-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-computed-side-effects-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-computed-side-effects) · [Fiche de la règle](/rules/project/vize-croquis-cf-computed-side-effects.md)

### `vize:croquis/cf/deep-import`

Une chaîne d’imports est plus profonde que ce que le projet autorise.

<span id="vize-croquis-cf-deep-import-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-deep-import-bad)

<span id="vize-croquis-cf-deep-import-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-deep-import-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-deep-import) · [Fiche de la règle](/rules/project/vize-croquis-cf-deep-import.md)

### `vize:croquis/cf/destructuring-breaks-reactivity`

La déstructuration d’un objet réactif copie ses champs et supprime leur suivi.

<span id="vize-croquis-cf-destructuring-breaks-reactivity-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity-bad)

<span id="vize-croquis-cf-destructuring-breaks-reactivity-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity) · [Fiche de la règle](/rules/project/vize-croquis-cf-destructuring-breaks-reactivity.md)

### `vize:croquis/cf/di-outside-setup`

`provide` ou `inject` est appelé en dehors de `setup`.

<span id="vize-croquis-cf-di-outside-setup-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-di-outside-setup-bad)

<span id="vize-croquis-cf-di-outside-setup-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-di-outside-setup-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-di-outside-setup) · [Fiche de la règle](/rules/project/vize-croquis-cf-di-outside-setup.md)

### `vize:croquis/cf/dom-access-without-next-tick`

Le DOM est lu avant que Vue ait appliqué la mise à jour.

<span id="vize-croquis-cf-dom-access-without-next-tick-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick-bad)

<span id="vize-croquis-cf-dom-access-without-next-tick-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick) · [Fiche de la règle](/rules/project/vize-croquis-cf-dom-access-without-next-tick.md)

### `vize:croquis/cf/duplicate-id`

Le même identifiant d’élément est utilisé dans plusieurs composants.

<span id="vize-croquis-cf-duplicate-id-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-duplicate-id-bad)

<span id="vize-croquis-cf-duplicate-id-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-duplicate-id-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-duplicate-id) · [Fiche de la règle](/rules/project/vize-croquis-cf-duplicate-id.md)

### `vize:croquis/cf/event-listener-leak`

Un écouteur d’événement est enregistré sans jamais être supprimé.

<span id="vize-croquis-cf-event-listener-leak-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-event-listener-leak-bad)

<span id="vize-croquis-cf-event-listener-leak-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-event-listener-leak-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-event-listener-leak) · [Fiche de la règle](/rules/project/vize-croquis-cf-event-listener-leak.md)

### `vize:croquis/cf/event-modifier`

Un écouteur d’événement utilise un modificateur que l’événement émis ne prend pas en charge.

<span id="vize-croquis-cf-event-modifier-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-event-modifier-bad)

<span id="vize-croquis-cf-event-modifier-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-event-modifier-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-event-modifier) · [Fiche de la règle](/rules/project/vize-croquis-cf-event-modifier.md)

### `vize:croquis/cf/hydration-risk`

Ce code regroupe plusieurs diagnostics de réactivité, dont une prop copiée dans une ref. Il ne signifie pas que chaque expression Date.now() est détectée par la passe inter-fichiers.

<span id="vize-croquis-cf-hydration-risk-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-hydration-risk-bad)

<span id="vize-croquis-cf-hydration-risk-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-hydration-risk-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-hydration-risk) · [Fiche de la règle](/rules/project/vize-croquis-cf-hydration-risk.md)

### `vize:croquis/cf/inherit-attrs-unused`

`inheritAttrs: false` est défini et le composant ne lit jamais les attributs.

<span id="vize-croquis-cf-inherit-attrs-unused-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-inherit-attrs-unused-bad)

<span id="vize-croquis-cf-inherit-attrs-unused-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-inherit-attrs-unused-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-inherit-attrs-unused) · [Fiche de la règle](/rules/project/vize-croquis-cf-inherit-attrs-unused.md)

### `vize:croquis/cf/inject-without-symbol`

`inject` utilise une clé ordinaire au lieu d’un symbole `InjectionKey`.

<span id="vize-croquis-cf-inject-without-symbol-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-inject-without-symbol-bad)

<span id="vize-croquis-cf-inject-without-symbol-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-inject-without-symbol-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-inject-without-symbol) · [Fiche de la règle](/rules/project/vize-croquis-cf-inject-without-symbol.md)

### `vize:croquis/cf/injected-async-mutation-race`

Une valeur injectée est modifiée par une tâche asynchrone susceptible de provoquer une condition de concurrence.

<span id="vize-croquis-cf-injected-async-mutation-race-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-injected-async-mutation-race-bad)

<span id="vize-croquis-cf-injected-async-mutation-race-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-injected-async-mutation-race-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-injected-async-mutation-race) · [Fiche de la règle](/rules/project/vize-croquis-cf-injected-async-mutation-race.md)

### `vize:croquis/cf/lifecycle-outside-setup`

Un hook de cycle de vie est enregistré en dehors de `setup`.

<span id="vize-croquis-cf-lifecycle-outside-setup-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup-bad)

<span id="vize-croquis-cf-lifecycle-outside-setup-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup) · [Fiche de la règle](/rules/project/vize-croquis-cf-lifecycle-outside-setup.md)

### `vize:croquis/cf/lifecycle-without-cleanup`

Un hook de cycle de vie lance un travail sans jamais le nettoyer.

<span id="vize-croquis-cf-lifecycle-without-cleanup-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup-bad)

<span id="vize-croquis-cf-lifecycle-without-cleanup-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup) · [Fiche de la règle](/rules/project/vize-croquis-cf-lifecycle-without-cleanup.md)

### `vize:croquis/cf/missing-required-prop`

Une prop obligatoire n’est pas transmise.

<span id="vize-croquis-cf-missing-required-prop-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-missing-required-prop-bad)

<span id="vize-croquis-cf-missing-required-prop-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-missing-required-prop-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-missing-required-prop) · [Fiche de la règle](/rules/project/vize-croquis-cf-missing-required-prop.md)

### `vize:croquis/cf/missing-suspense`

Une dépendance asynchrone est utilisée en dehors d’une frontière Suspense.

<span id="vize-croquis-cf-missing-suspense-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-missing-suspense-bad)

<span id="vize-croquis-cf-missing-suspense-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-missing-suspense-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-missing-suspense) · [Fiche de la règle](/rules/project/vize-croquis-cf-missing-suspense.md)

### `vize:croquis/cf/module-scope-reactive`

Un état réactif est créé au niveau du module et partagé par tous les appelants.

<span id="vize-croquis-cf-module-scope-reactive-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-module-scope-reactive-bad)

<span id="vize-croquis-cf-module-scope-reactive-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-module-scope-reactive-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-module-scope-reactive) · [Fiche de la règle](/rules/project/vize-croquis-cf-module-scope-reactive.md)

### `vize:croquis/cf/multi-root-attrs`

Un composant à plusieurs racines reçoit des attributs sans avoir d’endroit où les placer.

<span id="vize-croquis-cf-multi-root-attrs-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-multi-root-attrs-bad)

<span id="vize-croquis-cf-multi-root-attrs-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-multi-root-attrs-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-multi-root-attrs) · [Fiche de la règle](/rules/project/vize-croquis-cf-multi-root-attrs.md)

### `vize:croquis/cf/mutated-after-escape`

Un objet réactif est modifié après avoir échappé à son propriétaire.

<span id="vize-croquis-cf-mutated-after-escape-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-mutated-after-escape-bad)

<span id="vize-croquis-cf-mutated-after-escape-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-mutated-after-escape-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-mutated-after-escape) · [Fiche de la règle](/rules/project/vize-croquis-cf-mutated-after-escape.md)

### `vize:croquis/cf/non-reactive-provide`

Une valeur fournie n’est pas réactive ; les descendants ne verront donc pas les mises à jour.

<span id="vize-croquis-cf-non-reactive-provide-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-non-reactive-provide-bad)

<span id="vize-croquis-cf-non-reactive-provide-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-non-reactive-provide-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-non-reactive-provide) · [Fiche de la règle](/rules/project/vize-croquis-cf-non-reactive-provide.md)

### `vize:croquis/cf/non-unique-id`

L’identifiant d’un élément dans une boucle n’est pas unique pour chaque entrée.

<span id="vize-croquis-cf-non-unique-id-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-non-unique-id-bad)

<span id="vize-croquis-cf-non-unique-id-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-non-unique-id-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-non-unique-id) · [Fiche de la règle](/rules/project/vize-croquis-cf-non-unique-id.md)

### `vize:croquis/cf/object-identity-comparison`

Un objet réactif est comparé par identité, laquelle change lorsque l’enveloppe est retirée.

<span id="vize-croquis-cf-object-identity-comparison-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-object-identity-comparison-bad)

<span id="vize-croquis-cf-object-identity-comparison-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-object-identity-comparison-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-object-identity-comparison) · [Fiche de la règle](/rules/project/vize-croquis-cf-object-identity-comparison.md)

### `vize:croquis/cf/pinia-getter`

Un getter Pinia est lu sans `storeToRefs` ; il ne restera donc pas réactif.

<span id="vize-croquis-cf-pinia-getter-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-pinia-getter-bad)

<span id="vize-croquis-cf-pinia-getter-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-pinia-getter-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-pinia-getter) · [Fiche de la règle](/rules/project/vize-croquis-cf-pinia-getter.md)

### `vize:croquis/cf/prop-type-mismatch`

La valeur d’une prop transmise ne correspond pas au type déclaré.

<span id="vize-croquis-cf-prop-type-mismatch-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-prop-type-mismatch-bad)

<span id="vize-croquis-cf-prop-type-mismatch-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-prop-type-mismatch-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-prop-type-mismatch) · [Fiche de la règle](/rules/project/vize-croquis-cf-prop-type-mismatch.md)

### `vize:croquis/cf/provide-inject-type`

Une valeur fournie et son injection n’ont pas le même type.

<span id="vize-croquis-cf-provide-inject-type-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-provide-inject-type-bad)

<span id="vize-croquis-cf-provide-inject-type-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-provide-inject-type-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-provide-inject-type) · [Fiche de la règle](/rules/project/vize-croquis-cf-provide-inject-type.md)

### `vize:croquis/cf/provide-without-symbol`

`provide` utilise une clé ordinaire au lieu d’un symbole `InjectionKey`.

<span id="vize-croquis-cf-provide-without-symbol-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-provide-without-symbol-bad)

<span id="vize-croquis-cf-provide-without-symbol-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-provide-without-symbol-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-provide-without-symbol) · [Fiche de la règle](/rules/project/vize-croquis-cf-provide-without-symbol.md)

### `vize:croquis/cf/reactive-export`

Un état réactif est exporté depuis le module.

<span id="vize-croquis-cf-reactive-export-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-reactive-export-bad)

<span id="vize-croquis-cf-reactive-export-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-reactive-export-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-reactive-export) · [Fiche de la règle](/rules/project/vize-croquis-cf-reactive-export.md)

### `vize:croquis/cf/reactivity-outside-setup`

Une API réactive est appelée en dehors de `setup`.

<span id="vize-croquis-cf-reactivity-outside-setup-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-reactivity-outside-setup-bad)

<span id="vize-croquis-cf-reactivity-outside-setup-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-reactivity-outside-setup-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-reactivity-outside-setup) · [Fiche de la règle](/rules/project/vize-croquis-cf-reactivity-outside-setup.md)

### `vize:croquis/cf/reassignment-breaks-reactivity`

Réaffecter une liaison réactive la remplace par une valeur ordinaire.

<span id="vize-croquis-cf-reassignment-breaks-reactivity-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity-bad)

<span id="vize-croquis-cf-reassignment-breaks-reactivity-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity) · [Fiche de la règle](/rules/project/vize-croquis-cf-reassignment-breaks-reactivity.md)

### `vize:croquis/cf/reference-escapes-scope`

Une référence réactive échappe à la portée qui gère sa durée de vie.

<span id="vize-croquis-cf-reference-escapes-scope-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-reference-escapes-scope-bad)

<span id="vize-croquis-cf-reference-escapes-scope-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-reference-escapes-scope-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-reference-escapes-scope) · [Fiche de la règle](/rules/project/vize-croquis-cf-reference-escapes-scope.md)

### `vize:croquis/cf/setup-context-violation`

Le contexte setup est utilisé d’une manière que Vue n’autorise pas.

<span id="vize-croquis-cf-setup-context-violation-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-setup-context-violation-bad)

<span id="vize-croquis-cf-setup-context-violation-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-setup-context-violation-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-setup-context-violation) · [Fiche de la règle](/rules/project/vize-croquis-cf-setup-context-violation.md)

### `vize:croquis/cf/shallow-deep-access`

Une propriété profonde d’une valeur `shallowReactive` ou `shallowRef` est lue comme si elle était suivie.

<span id="vize-croquis-cf-shallow-deep-access-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-shallow-deep-access-bad)

<span id="vize-croquis-cf-shallow-deep-access-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-shallow-deep-access-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-shallow-deep-access) · [Fiche de la règle](/rules/project/vize-croquis-cf-shallow-deep-access.md)

### `vize:croquis/cf/spread-breaks-reactivity`

Décomposer un objet réactif avec l’opérateur spread copie ses valeurs et supprime leur suivi.

<span id="vize-croquis-cf-spread-breaks-reactivity-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity-bad)

<span id="vize-croquis-cf-spread-breaks-reactivity-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity) · [Fiche de la règle](/rules/project/vize-croquis-cf-spread-breaks-reactivity.md)

### `vize:croquis/cf/suspense-no-fallback`

`<Suspense>` n’a aucun contenu de secours.

<span id="vize-croquis-cf-suspense-no-fallback-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-suspense-no-fallback-bad)

<span id="vize-croquis-cf-suspense-no-fallback-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-suspense-no-fallback-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-suspense-no-fallback) · [Fiche de la règle](/rules/project/vize-croquis-cf-suspense-no-fallback.md)

### `vize:croquis/cf/template-ref-timing`

Une référence de template est lue avant le montage du composant.

<span id="vize-croquis-cf-template-ref-timing-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-template-ref-timing-bad)

<span id="vize-croquis-cf-template-ref-timing-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-template-ref-timing-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-template-ref-timing) · [Fiche de la règle](/rules/project/vize-croquis-cf-template-ref-timing.md)

### `vize:croquis/cf/toraw-mutation`

`toRaw` est utilisé, puis l’objet brut est modifié.

<span id="vize-croquis-cf-toraw-mutation-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-toraw-mutation-bad)

<span id="vize-croquis-cf-toraw-mutation-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-toraw-mutation-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-toraw-mutation) · [Fiche de la règle](/rules/project/vize-croquis-cf-toraw-mutation.md)

### `vize:croquis/cf/uncaught-error`

Un composant peut lever une exception sans qu’aucune frontière d’erreur ne la capture.

<span id="vize-croquis-cf-uncaught-error-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-uncaught-error-bad)

<span id="vize-croquis-cf-uncaught-error-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-uncaught-error-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-uncaught-error) · [Fiche de la règle](/rules/project/vize-croquis-cf-uncaught-error.md)

### `vize:croquis/cf/undeclared-emit`

Le composant émet un événement qui n’est pas déclaré.

<span id="vize-croquis-cf-undeclared-emit-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-undeclared-emit-bad)

<span id="vize-croquis-cf-undeclared-emit-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-undeclared-emit-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-undeclared-emit) · [Fiche de la règle](/rules/project/vize-croquis-cf-undeclared-emit.md)

### `vize:croquis/cf/undeclared-prop`

Un parent transmet une prop que l’enfant ne déclare pas.

<span id="vize-croquis-cf-undeclared-prop-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-undeclared-prop-bad)

<span id="vize-croquis-cf-undeclared-prop-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-undeclared-prop-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-undeclared-prop) · [Fiche de la règle](/rules/project/vize-croquis-cf-undeclared-prop.md)

### `vize:croquis/cf/undefined-slot`

Un parent remplit un slot que l’enfant n’expose pas.

<span id="vize-croquis-cf-undefined-slot-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-undefined-slot-bad)

<span id="vize-croquis-cf-undefined-slot-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-undefined-slot-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-undefined-slot) · [Fiche de la règle](/rules/project/vize-croquis-cf-undefined-slot.md)

### `vize:croquis/cf/unhandled-event`

Un enfant émet un événement qu’aucun parent ne gère.

<span id="vize-croquis-cf-unhandled-event-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-unhandled-event-bad)

<span id="vize-croquis-cf-unhandled-event-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-unhandled-event-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-unhandled-event) · [Fiche de la règle](/rules/project/vize-croquis-cf-unhandled-event.md)

### `vize:croquis/cf/unmatched-inject`

`inject` nomme une clé qu’aucun ancêtre ne fournit.

<span id="vize-croquis-cf-unmatched-inject-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-unmatched-inject-bad)

<span id="vize-croquis-cf-unmatched-inject-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-unmatched-inject-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-unmatched-inject) · [Fiche de la règle](/rules/project/vize-croquis-cf-unmatched-inject.md)

### `vize:croquis/cf/unmatched-listener`

Un parent écoute un événement que l’enfant n’émet pas.

<span id="vize-croquis-cf-unmatched-listener-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-unmatched-listener-bad)

<span id="vize-croquis-cf-unmatched-listener-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-unmatched-listener-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-unmatched-listener) · [Fiche de la règle](/rules/project/vize-croquis-cf-unmatched-listener.md)

### `vize:croquis/cf/unregistered-component`

Un template utilise un composant qui n’est ni enregistré ni importé.

<span id="vize-croquis-cf-unregistered-component-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-unregistered-component-bad)

<span id="vize-croquis-cf-unregistered-component-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-unregistered-component-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-unregistered-component) · [Fiche de la règle](/rules/project/vize-croquis-cf-unregistered-component.md)

### `vize:croquis/cf/unresolved-import`

Un import ne se résout pas vers un module.

<span id="vize-croquis-cf-unresolved-import-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-unresolved-import-bad)

<span id="vize-croquis-cf-unresolved-import-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-unresolved-import-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-unresolved-import) · [Fiche de la règle](/rules/project/vize-croquis-cf-unresolved-import.md)

### `vize:croquis/cf/unused-attrs`

Des attributs à transmettre automatiquement sont passés à un composant à plusieurs racines qui ne les utilise pas.

<span id="vize-croquis-cf-unused-attrs-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-unused-attrs-bad)

<span id="vize-croquis-cf-unused-attrs-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-unused-attrs-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-unused-attrs) · [Fiche de la règle](/rules/project/vize-croquis-cf-unused-attrs.md)

### `vize:croquis/cf/unused-emit`

Un événement déclaré n’est jamais émis.

<span id="vize-croquis-cf-unused-emit-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-unused-emit-bad)

<span id="vize-croquis-cf-unused-emit-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-unused-emit-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-unused-emit) · [Fiche de la règle](/rules/project/vize-croquis-cf-unused-emit.md)

### `vize:croquis/cf/unused-provide`

Une clé fournie n’est jamais injectée.

<span id="vize-croquis-cf-unused-provide-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-unused-provide-bad)

<span id="vize-croquis-cf-unused-provide-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-unused-provide-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-unused-provide) · [Fiche de la règle](/rules/project/vize-croquis-cf-unused-provide.md)

### `vize:croquis/cf/value-extraction-breaks-reactivity`

Extraire une valeur réactive dans une variable locale fait perdre les mises à jour ultérieures.

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity-bad)

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity) · [Fiche de la règle](/rules/project/vize-croquis-cf-value-extraction-breaks-reactivity.md)

### `vize:croquis/cf/watch-can-be-computed`

Un observateur ne fait que copier une valeur dans l’état et peut être remplacé par une propriété calculée.

<span id="vize-croquis-cf-watch-can-be-computed-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-watch-can-be-computed-bad)

<span id="vize-croquis-cf-watch-can-be-computed-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-watch-can-be-computed-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-watch-can-be-computed) · [Fiche de la règle](/rules/project/vize-croquis-cf-watch-can-be-computed.md)

### `vize:croquis/cf/watcheffect-async`

`watchEffect` lance une tâche asynchrone et ne peut pas nettoyer l’exécution précédente.

<span id="vize-croquis-cf-watcheffect-async-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-watcheffect-async-bad)

<span id="vize-croquis-cf-watcheffect-async-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-watcheffect-async-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-watcheffect-async) · [Fiche de la règle](/rules/project/vize-croquis-cf-watcheffect-async.md)

### `vize:croquis/cf/watcher-outside-setup`

`watch` ou `watchEffect` est appelé en dehors de `setup`.

<span id="vize-croquis-cf-watcher-outside-setup-bad"></span>

[Mauvais](./cross-file.md#vize-croquis-cf-watcher-outside-setup-bad)

<span id="vize-croquis-cf-watcher-outside-setup-good"></span>

[Bon](./cross-file.md#vize-croquis-cf-watcher-outside-setup-good)

[Exemples du groupe](./cross-file.md#vize-croquis-cf-watcher-outside-setup) · [Fiche de la règle](/rules/project/vize-croquis-cf-watcher-outside-setup.md)

### `vue/a11y-img-alt`

Exiger un attribut alt sur les images pour les rendre accessibles

<span id="vue-a11y-img-alt-bad"></span>

[Mauvais](./vue.md#vue-a11y-img-alt-bad)

<span id="vue-a11y-img-alt-good"></span>

[Bon](./vue.md#vue-a11y-img-alt-good)

[Exemples du groupe](./vue.md#vue-a11y-img-alt) · [Fiche de la règle](/rules/reference/vue-a11y-img-alt.md)

### `vue/attribute-hyphenation`

Imposer un style de nommage des attributs sur les composants personnalisés

<span id="vue-attribute-hyphenation-bad"></span>

[Mauvais](./vue.md#vue-attribute-hyphenation-bad)

<span id="vue-attribute-hyphenation-good"></span>

[Bon](./vue.md#vue-attribute-hyphenation-good)

[Exemples du groupe](./vue.md#vue-attribute-hyphenation) · [Fiche de la règle](/rules/reference/vue-attribute-hyphenation.md)

### `vue/attribute-order`

Imposer un ordre cohérent des attributs

<span id="vue-attribute-order-bad"></span>

[Mauvais](./vue.md#vue-attribute-order-bad)

<span id="vue-attribute-order-good"></span>

[Bon](./vue.md#vue-attribute-order-good)

[Exemples du groupe](./vue.md#vue-attribute-order) · [Fiche de la règle](/rules/reference/vue-attribute-order.md)

### `vue/component-definition-name-casing`

Imposer PascalCase ou kebab-case aux noms de définition des composants

<span id="vue-component-definition-name-casing-bad"></span>

[Mauvais](./vue.md#vue-component-definition-name-casing-bad)

<span id="vue-component-definition-name-casing-good"></span>

[Bon](./vue.md#vue-component-definition-name-casing-good)

[Exemples du groupe](./vue.md#vue-component-definition-name-casing) · [Fiche de la règle](/rules/reference/vue-component-definition-name-casing.md)

### `vue/component-name-in-template-casing`

Imposer une casse précise aux noms de composants dans les templates

<span id="vue-component-name-in-template-casing-bad"></span>

[Mauvais](./vue.md#vue-component-name-in-template-casing-bad)

<span id="vue-component-name-in-template-casing-good"></span>

[Bon](./vue.md#vue-component-name-in-template-casing-good)

[Exemples du groupe](./vue.md#vue-component-name-in-template-casing) · [Fiche de la règle](/rules/reference/vue-component-name-in-template-casing.md)

### `vue/cross-file-attrs-fallthrough`

Un parent transmet des attributs à un enfant résolu dont la racine ne peut pas en hériter et qui n’utilise pas explicitement $attrs.

<span id="vue-cross-file-attrs-fallthrough-bad"></span>

[Mauvais](./cross-file.md#vue-cross-file-attrs-fallthrough-bad)

<span id="vue-cross-file-attrs-fallthrough-good"></span>

[Bon](./cross-file.md#vue-cross-file-attrs-fallthrough-good)

[Exemples du groupe](./cross-file.md#vue-cross-file-attrs-fallthrough) · [Fiche de la règle](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `vue/html-button-has-type`

Exiger un type explicite et valide sur les éléments button

<span id="vue-html-button-has-type-bad"></span>

[Mauvais](./vue.md#vue-html-button-has-type-bad)

<span id="vue-html-button-has-type-good"></span>

[Bon](./vue.md#vue-html-button-has-type-good)

[Exemples du groupe](./vue.md#vue-html-button-has-type) · [Fiche de la règle](/rules/reference/vue-html-button-has-type.md)

### `vue/html-quotes`

Imposer un style de guillemets pour les attributs HTML

<span id="vue-html-quotes-bad"></span>

[Mauvais](./vue.md#vue-html-quotes-bad)

<span id="vue-html-quotes-good"></span>

[Bon](./vue.md#vue-html-quotes-good)

[Exemples du groupe](./vue.md#vue-html-quotes) · [Fiche de la règle](/rules/reference/vue-html-quotes.md)

### `vue/html-self-closing`

Imposer un style de fermeture automatique des balises

<span id="vue-html-self-closing-bad"></span>

[Mauvais](./vue.md#vue-html-self-closing-bad)

<span id="vue-html-self-closing-good"></span>

[Bon](./vue.md#vue-html-self-closing-good)

[Exemples du groupe](./vue.md#vue-html-self-closing) · [Fiche de la règle](/rules/reference/vue-html-self-closing.md)

### `vue/max-template-complexity`

Limiter la complexité propre du template d’un composant, cyclomatique et cognitive

<span id="vue-max-template-complexity-bad"></span>

[Mauvais](./vue.md#vue-max-template-complexity-bad)

<span id="vue-max-template-complexity-good"></span>

[Bon](./vue.md#vue-max-template-complexity-good)

[Exemples du groupe](./vue.md#vue-max-template-complexity) · [Fiche de la règle](/rules/reference/vue-max-template-complexity.md)

### `vue/multi-word-component-names`

Exiger des noms de composants composés de plusieurs mots

<span id="vue-multi-word-component-names-bad"></span>

[Mauvais](./vue.md#vue-multi-word-component-names-bad)

<span id="vue-multi-word-component-names-good"></span>

[Bon](./vue.md#vue-multi-word-component-names-good)

[Exemples du groupe](./vue.md#vue-multi-word-component-names) · [Fiche de la règle](/rules/reference/vue-multi-word-component-names.md)

### `vue/mustache-interpolation-spacing`

Imposer un espacement cohérent dans les interpolations à doubles accolades

<span id="vue-mustache-interpolation-spacing-bad"></span>

[Mauvais](./vue.md#vue-mustache-interpolation-spacing-bad)

<span id="vue-mustache-interpolation-spacing-good"></span>

[Bon](./vue.md#vue-mustache-interpolation-spacing-good)

[Exemples du groupe](./vue.md#vue-mustache-interpolation-spacing) · [Fiche de la règle](/rules/reference/vue-mustache-interpolation-spacing.md)

### `vue/no-array-index-key`

Interdire l’utilisation directe de la variable d’index de v-for comme :key

<span id="vue-no-array-index-key-bad"></span>

[Mauvais](./vue.md#vue-no-array-index-key-bad)

<span id="vue-no-array-index-key-good"></span>

[Bon](./vue.md#vue-no-array-index-key-good)

[Exemples du groupe](./vue.md#vue-no-array-index-key) · [Fiche de la règle](/rules/reference/vue-no-array-index-key.md)

### `vue/no-bare-strings-in-template`

Interdire le texte brut destiné aux utilisateurs dans les templates lorsqu’il devrait être internationalisé

<span id="vue-no-bare-strings-in-template-bad"></span>

[Mauvais](./vue.md#vue-no-bare-strings-in-template-bad)

<span id="vue-no-bare-strings-in-template-good"></span>

[Bon](./vue.md#vue-no-bare-strings-in-template-good)

[Exemples du groupe](./vue.md#vue-no-bare-strings-in-template) · [Fiche de la règle](/rules/reference/vue-no-bare-strings-in-template.md)

### `vue/no-boolean-attr-value`

Interdire les valeurs explicites des attributs HTML booléens

<span id="vue-no-boolean-attr-value-bad"></span>

[Mauvais](./vue.md#vue-no-boolean-attr-value-bad)

<span id="vue-no-boolean-attr-value-good"></span>

[Bon](./vue.md#vue-no-boolean-attr-value-good)

[Exemples du groupe](./vue.md#vue-no-boolean-attr-value) · [Fiche de la règle](/rules/reference/vue-no-boolean-attr-value.md)

### `vue/no-child-content`

Interdire le contenu enfant lors de l’utilisation de v-html ou v-text

<span id="vue-no-child-content-bad"></span>

[Mauvais](./vue.md#vue-no-child-content-bad)

<span id="vue-no-child-content-good"></span>

[Bon](./vue.md#vue-no-child-content-good)

[Exemples du groupe](./vue.md#vue-no-child-content) · [Fiche de la règle](/rules/reference/vue-no-child-content.md)

### `vue/no-deprecated-filter`

Interdire la syntaxe obsolète des filtres de Vue 2 utilisant l’opérateur pipe

<span id="vue-no-deprecated-filter-bad"></span>

[Mauvais](./vue.md#vue-no-deprecated-filter-bad)

<span id="vue-no-deprecated-filter-good"></span>

[Bon](./vue.md#vue-no-deprecated-filter-good)

[Exemples du groupe](./vue.md#vue-no-deprecated-filter) · [Fiche de la règle](/rules/reference/vue-no-deprecated-filter.md)

### `vue/no-deprecated-functional-template`

Interdire l’attribut `functional` sur le `<template>` d’un SFC

<span id="vue-no-deprecated-functional-template-bad"></span>

[Mauvais](./vue.md#vue-no-deprecated-functional-template-bad)

<span id="vue-no-deprecated-functional-template-good"></span>

[Bon](./vue.md#vue-no-deprecated-functional-template-good)

[Exemples du groupe](./vue.md#vue-no-deprecated-functional-template) · [Fiche de la règle](/rules/reference/vue-no-deprecated-functional-template.md)

### `vue/no-deprecated-html-element-is`

Interdire l’attribut `is` sur les éléments HTML natifs

<span id="vue-no-deprecated-html-element-is-bad"></span>

[Mauvais](./vue.md#vue-no-deprecated-html-element-is-bad)

<span id="vue-no-deprecated-html-element-is-good"></span>

[Bon](./vue.md#vue-no-deprecated-html-element-is-good)

[Exemples du groupe](./vue.md#vue-no-deprecated-html-element-is) · [Fiche de la règle](/rules/reference/vue-no-deprecated-html-element-is.md)

### `vue/no-deprecated-inline-template`

Interdire l’attribut obsolète `inline-template`

<span id="vue-no-deprecated-inline-template-bad"></span>

[Mauvais](./vue.md#vue-no-deprecated-inline-template-bad)

<span id="vue-no-deprecated-inline-template-good"></span>

[Bon](./vue.md#vue-no-deprecated-inline-template-good)

[Exemples du groupe](./vue.md#vue-no-deprecated-inline-template) · [Fiche de la règle](/rules/reference/vue-no-deprecated-inline-template.md)

### `vue/no-deprecated-router-link-tag-prop`

Interdire la prop `tag` sur &lt;router-link&gt;

<span id="vue-no-deprecated-router-link-tag-prop-bad"></span>

[Mauvais](./vue.md#vue-no-deprecated-router-link-tag-prop-bad)

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

[Bon](./vue.md#vue-no-deprecated-router-link-tag-prop-good)

[Exemples du groupe](./vue.md#vue-no-deprecated-router-link-tag-prop) · [Fiche de la règle](/rules/reference/vue-no-deprecated-router-link-tag-prop.md)

### `vue/no-deprecated-scope-attribute`

Interdire l’attribut obsolète `scope` sur &lt;template&gt;

<span id="vue-no-deprecated-scope-attribute-bad"></span>

[Mauvais](./vue.md#vue-no-deprecated-scope-attribute-bad)

<span id="vue-no-deprecated-scope-attribute-good"></span>

[Bon](./vue.md#vue-no-deprecated-scope-attribute-good)

[Exemples du groupe](./vue.md#vue-no-deprecated-scope-attribute) · [Fiche de la règle](/rules/reference/vue-no-deprecated-scope-attribute.md)

### `vue/no-deprecated-slot-attribute`

Interdire l’attribut obsolète `slot`

<span id="vue-no-deprecated-slot-attribute-bad"></span>

[Mauvais](./vue.md#vue-no-deprecated-slot-attribute-bad)

<span id="vue-no-deprecated-slot-attribute-good"></span>

[Bon](./vue.md#vue-no-deprecated-slot-attribute-good)

[Exemples du groupe](./vue.md#vue-no-deprecated-slot-attribute) · [Fiche de la règle](/rules/reference/vue-no-deprecated-slot-attribute.md)

### `vue/no-deprecated-slot-scope-attribute`

Interdire l’attribut obsolète `slot-scope`

<span id="vue-no-deprecated-slot-scope-attribute-bad"></span>

[Mauvais](./vue.md#vue-no-deprecated-slot-scope-attribute-bad)

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

[Bon](./vue.md#vue-no-deprecated-slot-scope-attribute-good)

[Exemples du groupe](./vue.md#vue-no-deprecated-slot-scope-attribute) · [Fiche de la règle](/rules/reference/vue-no-deprecated-slot-scope-attribute.md)

### `vue/no-deprecated-v-bind-sync`

Interdire le modificateur obsolète `.sync` sur `v-bind`

<span id="vue-no-deprecated-v-bind-sync-bad"></span>

[Mauvais](./vue.md#vue-no-deprecated-v-bind-sync-bad)

<span id="vue-no-deprecated-v-bind-sync-good"></span>

[Bon](./vue.md#vue-no-deprecated-v-bind-sync-good)

[Exemples du groupe](./vue.md#vue-no-deprecated-v-bind-sync) · [Fiche de la règle](/rules/reference/vue-no-deprecated-v-bind-sync.md)

### `vue/no-deprecated-v-is`

Interdire la directive obsolète `v-is`

<span id="vue-no-deprecated-v-is-bad"></span>

[Mauvais](./vue.md#vue-no-deprecated-v-is-bad)

<span id="vue-no-deprecated-v-is-good"></span>

[Bon](./vue.md#vue-no-deprecated-v-is-good)

[Exemples du groupe](./vue.md#vue-no-deprecated-v-is) · [Fiche de la règle](/rules/reference/vue-no-deprecated-v-is.md)

### `vue/no-deprecated-v-on-native-modifier`

Interdire le modificateur obsolète `.native` sur `v-on`

<span id="vue-no-deprecated-v-on-native-modifier-bad"></span>

[Mauvais](./vue.md#vue-no-deprecated-v-on-native-modifier-bad)

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

[Bon](./vue.md#vue-no-deprecated-v-on-native-modifier-good)

[Exemples du groupe](./vue.md#vue-no-deprecated-v-on-native-modifier) · [Fiche de la règle](/rules/reference/vue-no-deprecated-v-on-native-modifier.md)

### `vue/no-deprecated-v-on-number-modifiers`

Interdire les modificateurs numériques obsolètes `keyCode` sur `v-on`

<span id="vue-no-deprecated-v-on-number-modifiers-bad"></span>

[Mauvais](./vue.md#vue-no-deprecated-v-on-number-modifiers-bad)

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

[Bon](./vue.md#vue-no-deprecated-v-on-number-modifiers-good)

[Exemples du groupe](./vue.md#vue-no-deprecated-v-on-number-modifiers) · [Fiche de la règle](/rules/reference/vue-no-deprecated-v-on-number-modifiers.md)

### `vue/no-dupe-v-else-if`

Interdire les conditions répétées dans les chaînes `v-if` / `v-else-if`

<span id="vue-no-dupe-v-else-if-bad"></span>

[Mauvais](./vue.md#vue-no-dupe-v-else-if-bad)

<span id="vue-no-dupe-v-else-if-good"></span>

[Bon](./vue.md#vue-no-dupe-v-else-if-good)

[Exemples du groupe](./vue.md#vue-no-dupe-v-else-if) · [Fiche de la règle](/rules/reference/vue-no-dupe-v-else-if.md)

### `vue/no-duplicate-attributes`

Interdire les attributs en double sur un même élément

<span id="vue-no-duplicate-attributes-bad"></span>

[Mauvais](./vue.md#vue-no-duplicate-attributes-bad)

<span id="vue-no-duplicate-attributes-good"></span>

[Bon](./vue.md#vue-no-duplicate-attributes-good)

[Exemples du groupe](./vue.md#vue-no-duplicate-attributes) · [Fiche de la règle](/rules/reference/vue-no-duplicate-attributes.md)

### `vue/no-empty-component-block`

Interdire les blocs SFC vides

<span id="vue-no-empty-component-block-bad"></span>

[Mauvais](./vue.md#vue-no-empty-component-block-bad)

<span id="vue-no-empty-component-block-good"></span>

[Bon](./vue.md#vue-no-empty-component-block-good)

[Exemples du groupe](./vue.md#vue-no-empty-component-block) · [Fiche de la règle](/rules/reference/vue-no-empty-component-block.md)

### `vue/no-inline-style`

Déconseiller l’utilisation d’attributs de style en ligne

<span id="vue-no-inline-style-bad"></span>

[Mauvais](./vue.md#vue-no-inline-style-bad)

<span id="vue-no-inline-style-good"></span>

[Bon](./vue.md#vue-no-inline-style-good)

[Exemples du groupe](./vue.md#vue-no-inline-style) · [Fiche de la règle](/rules/reference/vue-no-inline-style.md)

### `vue/no-invalid-html-attribute`

Interdire les valeurs statiques invalides des attributs HTML

<span id="vue-no-invalid-html-attribute-bad"></span>

[Mauvais](./vue.md#vue-no-invalid-html-attribute-bad)

<span id="vue-no-invalid-html-attribute-good"></span>

[Bon](./vue.md#vue-no-invalid-html-attribute-good)

[Exemples du groupe](./vue.md#vue-no-invalid-html-attribute) · [Fiche de la règle](/rules/reference/vue-no-invalid-html-attribute.md)

### `vue/no-lone-template`

Interdire les éléments `<template>` inutiles

<span id="vue-no-lone-template-bad"></span>

[Mauvais](./vue.md#vue-no-lone-template-bad)

<span id="vue-no-lone-template-good"></span>

[Bon](./vue.md#vue-no-lone-template-good)

[Exemples du groupe](./vue.md#vue-no-lone-template) · [Fiche de la règle](/rules/reference/vue-no-lone-template.md)

### `vue/no-multi-spaces`

Interdire plusieurs espaces consécutifs

<span id="vue-no-multi-spaces-bad"></span>

[Mauvais](./vue.md#vue-no-multi-spaces-bad)

<span id="vue-no-multi-spaces-good"></span>

[Bon](./vue.md#vue-no-multi-spaces-good)

[Exemples du groupe](./vue.md#vue-no-multi-spaces) · [Fiche de la règle](/rules/reference/vue-no-multi-spaces.md)

### `vue/no-multiple-objects-in-class`

Interdire plusieurs objets littéraux dans une liaison :class sous forme de tableau

<span id="vue-no-multiple-objects-in-class-bad"></span>

[Mauvais](./vue.md#vue-no-multiple-objects-in-class-bad)

<span id="vue-no-multiple-objects-in-class-good"></span>

[Bon](./vue.md#vue-no-multiple-objects-in-class-good)

[Exemples du groupe](./vue.md#vue-no-multiple-objects-in-class) · [Fiche de la règle](/rules/reference/vue-no-multiple-objects-in-class.md)

### `vue/no-multiple-template-root`

Interdire plusieurs nœuds racines dans un template

<span id="vue-no-multiple-template-root-bad"></span>

[Mauvais](./vue.md#vue-no-multiple-template-root-bad)

<span id="vue-no-multiple-template-root-good"></span>

[Bon](./vue.md#vue-no-multiple-template-root-good)

[Exemples du groupe](./vue.md#vue-no-multiple-template-root) · [Fiche de la règle](/rules/reference/vue-no-multiple-template-root.md)

### `vue/no-mutating-props`

Interdire la modification des props d’un composant

<span id="vue-no-mutating-props-bad"></span>

[Mauvais](./vue.md#vue-no-mutating-props-bad)

<span id="vue-no-mutating-props-good"></span>

[Bon](./vue.md#vue-no-mutating-props-good)

[Exemples du groupe](./vue.md#vue-no-mutating-props) · [Fiche de la règle](/rules/reference/vue-no-mutating-props.md)

### `vue/no-negated-v-if-condition`

Interdire une condition v-if négative lorsque la chaîne comporte un v-else

<span id="vue-no-negated-v-if-condition-bad"></span>

[Mauvais](./vue.md#vue-no-negated-v-if-condition-bad)

<span id="vue-no-negated-v-if-condition-good"></span>

[Bon](./vue.md#vue-no-negated-v-if-condition-good)

[Exemples du groupe](./vue.md#vue-no-negated-v-if-condition) · [Fiche de la règle](/rules/reference/vue-no-negated-v-if-condition.md)

### `vue/no-non-component-keep-alive-child`

Interdire les enveloppes d’éléments ordinaires directement sous `<KeepAlive>`

<span id="vue-no-non-component-keep-alive-child-bad"></span>

[Mauvais](./vue.md#vue-no-non-component-keep-alive-child-bad)

<span id="vue-no-non-component-keep-alive-child-good"></span>

[Bon](./vue.md#vue-no-non-component-keep-alive-child-good)

[Exemples du groupe](./vue.md#vue-no-non-component-keep-alive-child) · [Fiche de la règle](/rules/reference/vue-no-non-component-keep-alive-child.md)

### `vue/no-preprocessor-lang`

Déconseiller les préprocesseurs CSS au profit du CSS moderne

<span id="vue-no-preprocessor-lang-bad"></span>

[Mauvais](./vue.md#vue-no-preprocessor-lang-bad)

<span id="vue-no-preprocessor-lang-good"></span>

[Bon](./vue.md#vue-no-preprocessor-lang-good)

[Exemples du groupe](./vue.md#vue-no-preprocessor-lang) · [Fiche de la règle](/rules/reference/vue-no-preprocessor-lang.md)

### `vue/no-reserved-component-names`

Interdire l’utilisation de noms réservés comme noms de composants

<span id="vue-no-reserved-component-names-bad"></span>

[Mauvais](./vue.md#vue-no-reserved-component-names-bad)

<span id="vue-no-reserved-component-names-good"></span>

[Bon](./vue.md#vue-no-reserved-component-names-good)

[Exemples du groupe](./vue.md#vue-no-reserved-component-names) · [Fiche de la règle](/rules/reference/vue-no-reserved-component-names.md)

### `vue/no-root-v-if`

Interdire v-if sur l’unique élément racine d’un template

<span id="vue-no-root-v-if-bad"></span>

[Mauvais](./vue.md#vue-no-root-v-if-bad)

<span id="vue-no-root-v-if-good"></span>

[Bon](./vue.md#vue-no-root-v-if-good)

[Exemples du groupe](./vue.md#vue-no-root-v-if) · [Fiche de la règle](/rules/reference/vue-no-root-v-if.md)

### `vue/no-script-non-standard-lang`

Déconseiller les valeurs lang non standard dans les scripts

<span id="vue-no-script-non-standard-lang-bad"></span>

[Mauvais](./vue.md#vue-no-script-non-standard-lang-bad)

<span id="vue-no-script-non-standard-lang-good"></span>

[Bon](./vue.md#vue-no-script-non-standard-lang-good)

[Exemples du groupe](./vue.md#vue-no-script-non-standard-lang) · [Fiche de la règle](/rules/reference/vue-no-script-non-standard-lang.md)

### `vue/no-src-attribute`

Déconseiller l’attribut src sur les blocs SFC

<span id="vue-no-src-attribute-bad"></span>

[Mauvais](./vue.md#vue-no-src-attribute-bad)

<span id="vue-no-src-attribute-good"></span>

[Bon](./vue.md#vue-no-src-attribute-good)

[Exemples du groupe](./vue.md#vue-no-src-attribute) · [Fiche de la règle](/rules/reference/vue-no-src-attribute.md)

### `vue/no-static-inline-styles`

Interdire les attributs de style en ligne statiques

<span id="vue-no-static-inline-styles-bad"></span>

[Mauvais](./vue.md#vue-no-static-inline-styles-bad)

<span id="vue-no-static-inline-styles-good"></span>

[Bon](./vue.md#vue-no-static-inline-styles-good)

[Exemples du groupe](./vue.md#vue-no-static-inline-styles) · [Fiche de la règle](/rules/reference/vue-no-static-inline-styles.md)

### `vue/no-template-key`

Interdire l’attribut `key` sur `<template>`

<span id="vue-no-template-key-bad"></span>

[Mauvais](./vue.md#vue-no-template-key-bad)

<span id="vue-no-template-key-good"></span>

[Bon](./vue.md#vue-no-template-key-good)

[Exemples du groupe](./vue.md#vue-no-template-key) · [Fiche de la règle](/rules/reference/vue-no-template-key.md)

### `vue/no-template-lang`

Déconseiller l’attribut lang sur le bloc template

<span id="vue-no-template-lang-bad"></span>

[Mauvais](./vue.md#vue-no-template-lang-bad)

<span id="vue-no-template-lang-good"></span>

[Bon](./vue.md#vue-no-template-lang-good)

[Exemples du groupe](./vue.md#vue-no-template-lang) · [Fiche de la règle](/rules/reference/vue-no-template-lang.md)

### `vue/no-template-shadow`

Interdire les noms de variables qui masquent des variables d’une portée externe

<span id="vue-no-template-shadow-bad"></span>

[Mauvais](./vue.md#vue-no-template-shadow-bad)

<span id="vue-no-template-shadow-good"></span>

[Bon](./vue.md#vue-no-template-shadow-good)

[Exemples du groupe](./vue.md#vue-no-template-shadow) · [Fiche de la règle](/rules/reference/vue-no-template-shadow.md)

### `vue/no-template-target-blank`

Interdire target="_blank" sans rel="noopener noreferrer"

<span id="vue-no-template-target-blank-bad"></span>

[Mauvais](./vue.md#vue-no-template-target-blank-bad)

<span id="vue-no-template-target-blank-good"></span>

[Bon](./vue.md#vue-no-template-target-blank-good)

[Exemples du groupe](./vue.md#vue-no-template-target-blank) · [Fiche de la règle](/rules/reference/vue-no-template-target-blank.md)

### `vue/no-textarea-mustache`

Interdire l’interpolation à doubles accolades dans `<textarea>`

<span id="vue-no-textarea-mustache-bad"></span>

[Mauvais](./vue.md#vue-no-textarea-mustache-bad)

<span id="vue-no-textarea-mustache-good"></span>

[Bon](./vue.md#vue-no-textarea-mustache-good)

[Exemples du groupe](./vue.md#vue-no-textarea-mustache) · [Fiche de la règle](/rules/reference/vue-no-textarea-mustache.md)

### `vue/no-undefined-refs`

Interdire les références à des variables non définies dans les templates

<span id="vue-no-undefined-refs-bad"></span>

[Mauvais](./vue.md#vue-no-undefined-refs-bad)

<span id="vue-no-undefined-refs-good"></span>

[Bon](./vue.md#vue-no-undefined-refs-good)

[Exemples du groupe](./vue.md#vue-no-undefined-refs) · [Fiche de la règle](/rules/reference/vue-no-undefined-refs.md)

### `vue/no-unsafe-url`

Signaler les liaisons d’URL potentiellement dangereuses

<span id="vue-no-unsafe-url-bad"></span>

[Mauvais](./vue.md#vue-no-unsafe-url-bad)

<span id="vue-no-unsafe-url-good"></span>

[Bon](./vue.md#vue-no-unsafe-url-good)

[Exemples du groupe](./vue.md#vue-no-unsafe-url) · [Fiche de la règle](/rules/reference/vue-no-unsafe-url.md)

### `vue/no-unsandboxed-iframe`

Exiger un attribut sandbox sur les éléments iframe

<span id="vue-no-unsandboxed-iframe-bad"></span>

[Mauvais](./vue.md#vue-no-unsandboxed-iframe-bad)

<span id="vue-no-unsandboxed-iframe-good"></span>

[Bon](./vue.md#vue-no-unsandboxed-iframe-good)

[Exemples du groupe](./vue.md#vue-no-unsandboxed-iframe) · [Fiche de la règle](/rules/reference/vue-no-unsandboxed-iframe.md)

### `vue/no-unused-components`

Interdire l’enregistrement de composants inutilisés dans les templates

<span id="vue-no-unused-components-bad"></span>

[Mauvais](./vue.md#vue-no-unused-components-bad)

<span id="vue-no-unused-components-good"></span>

[Bon](./vue.md#vue-no-unused-components-good)

[Exemples du groupe](./vue.md#vue-no-unused-components) · [Fiche de la règle](/rules/reference/vue-no-unused-components.md)

### `vue/no-unused-properties`

Interdire les propriétés inutilisées définies dans defineProps

<span id="vue-no-unused-properties-bad"></span>

[Mauvais](./vue.md#vue-no-unused-properties-bad)

<span id="vue-no-unused-properties-good"></span>

[Bon](./vue.md#vue-no-unused-properties-good)

[Exemples du groupe](./vue.md#vue-no-unused-properties) · [Fiche de la règle](/rules/reference/vue-no-unused-properties.md)

### `vue/no-unused-refs`

Signaler les refs de template (ref="x") jamais référencées dans &lt;script&gt;

<span id="vue-no-unused-refs-bad"></span>

[Mauvais](./vue.md#vue-no-unused-refs-bad)

<span id="vue-no-unused-refs-good"></span>

[Bon](./vue.md#vue-no-unused-refs-good)

[Exemples du groupe](./vue.md#vue-no-unused-refs) · [Fiche de la règle](/rules/reference/vue-no-unused-refs.md)

### `vue/no-unused-setup-bindings`

Interdire les liaisons script setup qui ne sont jamais lues

<span id="vue-no-unused-setup-bindings-bad"></span>

[Mauvais](./vue.md#vue-no-unused-setup-bindings-bad)

<span id="vue-no-unused-setup-bindings-good"></span>

[Bon](./vue.md#vue-no-unused-setup-bindings-good)

[Exemples du groupe](./vue.md#vue-no-unused-setup-bindings) · [Fiche de la règle](/rules/reference/vue-no-unused-setup-bindings.md)

### `vue/no-unused-vars`

Interdire les variables inutilisées définies dans les directives v-for et v-slot

<span id="vue-no-unused-vars-bad"></span>

[Mauvais](./vue.md#vue-no-unused-vars-bad)

<span id="vue-no-unused-vars-good"></span>

[Bon](./vue.md#vue-no-unused-vars-good)

[Exemples du groupe](./vue.md#vue-no-unused-vars) · [Fiche de la règle](/rules/reference/vue-no-unused-vars.md)

### `vue/no-use-v-else-with-v-for`

Interdire `v-else-if` ou `v-else` sur le même élément que `v-for`

<span id="vue-no-use-v-else-with-v-for-bad"></span>

[Mauvais](./vue.md#vue-no-use-v-else-with-v-for-bad)

<span id="vue-no-use-v-else-with-v-for-good"></span>

[Bon](./vue.md#vue-no-use-v-else-with-v-for-good)

[Exemples du groupe](./vue.md#vue-no-use-v-else-with-v-for) · [Fiche de la règle](/rules/reference/vue-no-use-v-else-with-v-for.md)

### `vue/no-use-v-if-with-v-for`

Interdire `v-if` sur le même élément que `v-for`

<span id="vue-no-use-v-if-with-v-for-bad"></span>

[Mauvais](./vue.md#vue-no-use-v-if-with-v-for-bad)

<span id="vue-no-use-v-if-with-v-for-good"></span>

[Bon](./vue.md#vue-no-use-v-if-with-v-for-good)

[Exemples du groupe](./vue.md#vue-no-use-v-if-with-v-for) · [Fiche de la règle](/rules/reference/vue-no-use-v-if-with-v-for.md)

### `vue/no-useless-mustaches`

Interdire une interpolation à doubles accolades dont l’expression est une chaîne littérale constante

<span id="vue-no-useless-mustaches-bad"></span>

[Mauvais](./vue.md#vue-no-useless-mustaches-bad)

<span id="vue-no-useless-mustaches-good"></span>

[Bon](./vue.md#vue-no-useless-mustaches-good)

[Exemples du groupe](./vue.md#vue-no-useless-mustaches) · [Fiche de la règle](/rules/reference/vue-no-useless-mustaches.md)

### `vue/no-useless-template-attributes`

Interdire les attributs inutiles sur les éléments `<template>`

<span id="vue-no-useless-template-attributes-bad"></span>

[Mauvais](./vue.md#vue-no-useless-template-attributes-bad)

<span id="vue-no-useless-template-attributes-good"></span>

[Bon](./vue.md#vue-no-useless-template-attributes-good)

[Exemples du groupe](./vue.md#vue-no-useless-template-attributes) · [Fiche de la règle](/rules/reference/vue-no-useless-template-attributes.md)

### `vue/no-useless-v-bind`

Interdire un v-bind dont la valeur est une simple chaîne littérale

<span id="vue-no-useless-v-bind-bad"></span>

[Mauvais](./vue.md#vue-no-useless-v-bind-bad)

<span id="vue-no-useless-v-bind-good"></span>

[Bon](./vue.md#vue-no-useless-v-bind-good)

[Exemples du groupe](./vue.md#vue-no-useless-v-bind) · [Fiche de la règle](/rules/reference/vue-no-useless-v-bind.md)

### `vue/no-v-for-template-key-on-child`

Interdire `key` sur l’enfant d’un `<template v-for>`

<span id="vue-no-v-for-template-key-on-child-bad"></span>

[Mauvais](./vue.md#vue-no-v-for-template-key-on-child-bad)

<span id="vue-no-v-for-template-key-on-child-good"></span>

[Bon](./vue.md#vue-no-v-for-template-key-on-child-good)

[Exemples du groupe](./vue.md#vue-no-v-for-template-key-on-child) · [Fiche de la règle](/rules/reference/vue-no-v-for-template-key-on-child.md)

### `vue/no-v-html`

Déconseiller v-html pour prévenir les vulnérabilités XSS

<span id="vue-no-v-html-bad"></span>

[Mauvais](./vue.md#vue-no-v-html-bad)

<span id="vue-no-v-html-good"></span>

[Bon](./vue.md#vue-no-v-html-good)

[Exemples du groupe](./vue.md#vue-no-v-html) · [Fiche de la règle](/rules/reference/vue-no-v-html.md)

### `vue/no-v-text`

Interdire la directive v-text ; préférer l’interpolation à doubles accolades

<span id="vue-no-v-text-bad"></span>

[Mauvais](./vue.md#vue-no-v-text-bad)

<span id="vue-no-v-text-good"></span>

[Bon](./vue.md#vue-no-v-text-good)

[Exemples du groupe](./vue.md#vue-no-v-text) · [Fiche de la règle](/rules/reference/vue-no-v-text.md)

### `vue/no-v-text-v-html-on-component`

Interdire v-text / v-html sur les éléments de composants

<span id="vue-no-v-text-v-html-on-component-bad"></span>

[Mauvais](./vue.md#vue-no-v-text-v-html-on-component-bad)

<span id="vue-no-v-text-v-html-on-component-good"></span>

[Bon](./vue.md#vue-no-v-text-v-html-on-component-good)

[Exemples du groupe](./vue.md#vue-no-v-text-v-html-on-component) · [Fiche de la règle](/rules/reference/vue-no-v-text-v-html-on-component.md)

### `vue/permitted-contents`

Faire respecter les règles du modèle de contenu HTML

<span id="vue-permitted-contents-bad"></span>

[Mauvais](./vue.md#vue-permitted-contents-bad)

<span id="vue-permitted-contents-good"></span>

[Bon](./vue.md#vue-permitted-contents-good)

[Exemples du groupe](./vue.md#vue-permitted-contents) · [Fiche de la règle](/rules/reference/vue-permitted-contents.md)

### `vue/prefer-props-shorthand`

Recommander la syntaxe abrégée des props (Vue 3.4+)

<span id="vue-prefer-props-shorthand-bad"></span>

[Mauvais](./vue.md#vue-prefer-props-shorthand-bad)

<span id="vue-prefer-props-shorthand-good"></span>

[Bon](./vue.md#vue-prefer-props-shorthand-good)

[Exemples du groupe](./vue.md#vue-prefer-props-shorthand) · [Fiche de la règle](/rules/reference/vue-prefer-props-shorthand.md)

### `vue/prefer-true-attribute-shorthand`

Préférer la syntaxe abrégée pour un attribut booléen lié à `true`

<span id="vue-prefer-true-attribute-shorthand-bad"></span>

[Mauvais](./vue.md#vue-prefer-true-attribute-shorthand-bad)

<span id="vue-prefer-true-attribute-shorthand-good"></span>

[Bon](./vue.md#vue-prefer-true-attribute-shorthand-good)

[Exemples du groupe](./vue.md#vue-prefer-true-attribute-shorthand) · [Fiche de la règle](/rules/reference/vue-prefer-true-attribute-shorthand.md)

### `vue/prop-name-casing`

Imposer une casse aux noms des props déclarées

<span id="vue-prop-name-casing-bad"></span>

[Mauvais](./vue.md#vue-prop-name-casing-bad)

<span id="vue-prop-name-casing-good"></span>

[Bon](./vue.md#vue-prop-name-casing-good)

[Exemples du groupe](./vue.md#vue-prop-name-casing) · [Fiche de la règle](/rules/reference/vue-prop-name-casing.md)

### `vue/require-component-is`

Exiger `v-bind:is` sur les éléments `<component>`

<span id="vue-require-component-is-bad"></span>

[Mauvais](./vue.md#vue-require-component-is-bad)

<span id="vue-require-component-is-good"></span>

[Bon](./vue.md#vue-require-component-is-good)

[Exemples du groupe](./vue.md#vue-require-component-is) · [Fiche de la règle](/rules/reference/vue-require-component-is.md)

### `vue/require-component-registration`

Exiger un import ou un enregistrement explicite des composants

<span id="vue-require-component-registration-bad"></span>

[Mauvais](./vue.md#vue-require-component-registration-bad)

<span id="vue-require-component-registration-good"></span>

[Bon](./vue.md#vue-require-component-registration-good)

[Exemples du groupe](./vue.md#vue-require-component-registration) · [Fiche de la règle](/rules/reference/vue-require-component-registration.md)

### `vue/require-scoped-style`

Exiger l’attribut scoped sur les balises style

<span id="vue-require-scoped-style-bad"></span>

[Mauvais](./vue.md#vue-require-scoped-style-bad)

<span id="vue-require-scoped-style-good"></span>

[Bon](./vue.md#vue-require-scoped-style-good)

[Exemples du groupe](./vue.md#vue-require-scoped-style) · [Fiche de la règle](/rules/reference/vue-require-scoped-style.md)

### `vue/require-toggle-inside-transition`

Exiger un mécanisme de bascule sur l’élément enveloppé par `<transition>`

<span id="vue-require-toggle-inside-transition-bad"></span>

[Mauvais](./vue.md#vue-require-toggle-inside-transition-bad)

<span id="vue-require-toggle-inside-transition-good"></span>

[Bon](./vue.md#vue-require-toggle-inside-transition-good)

[Exemples du groupe](./vue.md#vue-require-toggle-inside-transition) · [Fiche de la règle](/rules/reference/vue-require-toggle-inside-transition.md)

### `vue/require-v-for-key`

Exiger `v-bind:key` avec les directives `v-for`

<span id="vue-require-v-for-key-bad"></span>

[Mauvais](./vue.md#vue-require-v-for-key-bad)

<span id="vue-require-v-for-key-good"></span>

[Bon](./vue.md#vue-require-v-for-key-good)

[Exemples du groupe](./vue.md#vue-require-v-for-key) · [Fiche de la règle](/rules/reference/vue-require-v-for-key.md)

### `vue/scoped-event-names`

Recommander des noms d’événements préfixés par leur contexte au format context:event

<span id="vue-scoped-event-names-bad"></span>

[Mauvais](./vue.md#vue-scoped-event-names-bad)

<span id="vue-scoped-event-names-good"></span>

[Bon](./vue.md#vue-scoped-event-names-good)

[Exemples du groupe](./vue.md#vue-scoped-event-names) · [Fiche de la règle](/rules/reference/vue-scoped-event-names.md)

### `vue/sfc-element-order`

Imposer un ordre cohérent des éléments de premier niveau d’un SFC

<span id="vue-sfc-element-order-bad"></span>

[Mauvais](./vue.md#vue-sfc-element-order-bad)

<span id="vue-sfc-element-order-good"></span>

[Bon](./vue.md#vue-sfc-element-order-good)

[Exemples du groupe](./vue.md#vue-sfc-element-order) · [Fiche de la règle](/rules/reference/vue-sfc-element-order.md)

### `vue/single-style-block`

Recommander un seul bloc style

<span id="vue-single-style-block-bad"></span>

[Mauvais](./vue.md#vue-single-style-block-bad)

<span id="vue-single-style-block-good"></span>

[Bon](./vue.md#vue-single-style-block-good)

[Exemples du groupe](./vue.md#vue-single-style-block) · [Fiche de la règle](/rules/reference/vue-single-style-block.md)

### `vue/slot-name-casing`

Imposer kebab-case aux slots nommés utilisés avec v-slot

<span id="vue-slot-name-casing-bad"></span>

[Mauvais](./vue.md#vue-slot-name-casing-bad)

<span id="vue-slot-name-casing-good"></span>

[Bon](./vue.md#vue-slot-name-casing-good)

[Exemples du groupe](./vue.md#vue-slot-name-casing) · [Fiche de la règle](/rules/reference/vue-slot-name-casing.md)

### `vue/this-in-template`

Interdire `this.` dans les expressions des templates

<span id="vue-this-in-template-bad"></span>

[Mauvais](./vue.md#vue-this-in-template-bad)

<span id="vue-this-in-template-good"></span>

[Bon](./vue.md#vue-this-in-template-good)

[Exemples du groupe](./vue.md#vue-this-in-template) · [Fiche de la règle](/rules/reference/vue-this-in-template.md)

### `vue/use-unique-element-ids`

Imposer des identifiants d’éléments uniques avec useId() plutôt que des littéraux statiques

<span id="vue-use-unique-element-ids-bad"></span>

[Mauvais](./vue.md#vue-use-unique-element-ids-bad)

<span id="vue-use-unique-element-ids-good"></span>

[Bon](./vue.md#vue-use-unique-element-ids-good)

[Exemples du groupe](./vue.md#vue-use-unique-element-ids) · [Fiche de la règle](/rules/reference/vue-use-unique-element-ids.md)

### `vue/use-v-on-exact`

Imposer le modificateur `.exact` sur `v-on` en présence de gestionnaires utilisant des modificateurs

<span id="vue-use-v-on-exact-bad"></span>

[Mauvais](./vue.md#vue-use-v-on-exact-bad)

<span id="vue-use-v-on-exact-good"></span>

[Bon](./vue.md#vue-use-v-on-exact-good)

[Exemples du groupe](./vue.md#vue-use-v-on-exact) · [Fiche de la règle](/rules/reference/vue-use-v-on-exact.md)

### `vue/v-bind-style`

Imposer un style de directive `v-bind`

<span id="vue-v-bind-style-bad"></span>

[Mauvais](./vue.md#vue-v-bind-style-bad)

<span id="vue-v-bind-style-good"></span>

[Bon](./vue.md#vue-v-bind-style-good)

[Exemples du groupe](./vue.md#vue-v-bind-style) · [Fiche de la règle](/rules/reference/vue-v-bind-style.md)

### `vue/v-on-event-hyphenation`

Imposer des tirets dans les noms d’événements personnalisés de v-on sur les composants

<span id="vue-v-on-event-hyphenation-bad"></span>

[Mauvais](./vue.md#vue-v-on-event-hyphenation-bad)

<span id="vue-v-on-event-hyphenation-good"></span>

[Bon](./vue.md#vue-v-on-event-hyphenation-good)

[Exemples du groupe](./vue.md#vue-v-on-event-hyphenation) · [Fiche de la règle](/rules/reference/vue-v-on-event-hyphenation.md)

### `vue/v-on-handler-style`

Imposer une référence de méthode ou une fonction en ligne pour les gestionnaires v-on

<span id="vue-v-on-handler-style-bad"></span>

[Mauvais](./vue.md#vue-v-on-handler-style-bad)

<span id="vue-v-on-handler-style-good"></span>

[Bon](./vue.md#vue-v-on-handler-style-good)

[Exemples du groupe](./vue.md#vue-v-on-handler-style) · [Fiche de la règle](/rules/reference/vue-v-on-handler-style.md)

### `vue/v-on-style`

Imposer un style de directive `v-on`

<span id="vue-v-on-style-bad"></span>

[Mauvais](./vue.md#vue-v-on-style-bad)

<span id="vue-v-on-style-good"></span>

[Bon](./vue.md#vue-v-on-style-good)

[Exemples du groupe](./vue.md#vue-v-on-style) · [Fiche de la règle](/rules/reference/vue-v-on-style.md)

### `vue/v-slot-style`

Imposer un style de directive `v-slot`

<span id="vue-v-slot-style-bad"></span>

[Mauvais](./vue.md#vue-v-slot-style-bad)

<span id="vue-v-slot-style-good"></span>

[Bon](./vue.md#vue-v-slot-style-good)

[Exemples du groupe](./vue.md#vue-v-slot-style) · [Fiche de la règle](/rules/reference/vue-v-slot-style.md)

### `vue/valid-attribute-name`

Exiger des noms d’attributs valides

<span id="vue-valid-attribute-name-bad"></span>

[Mauvais](./vue.md#vue-valid-attribute-name-bad)

<span id="vue-valid-attribute-name-good"></span>

[Bon](./vue.md#vue-valid-attribute-name-good)

[Exemples du groupe](./vue.md#vue-valid-attribute-name) · [Fiche de la règle](/rules/reference/vue-valid-attribute-name.md)

### `vue/valid-template-root`

Exiger une racine `<template>` valide selon la sémantique des fragments de Vue 3

<span id="vue-valid-template-root-bad"></span>

[Mauvais](./vue.md#vue-valid-template-root-bad)

<span id="vue-valid-template-root-good"></span>

[Bon](./vue.md#vue-valid-template-root-good)

[Exemples du groupe](./vue.md#vue-valid-template-root) · [Fiche de la règle](/rules/reference/vue-valid-template-root.md)

### `vue/valid-v-bind`

Exiger des directives `v-bind` valides

<span id="vue-valid-v-bind-bad"></span>

[Mauvais](./vue.md#vue-valid-v-bind-bad)

<span id="vue-valid-v-bind-good"></span>

[Bon](./vue.md#vue-valid-v-bind-good)

[Exemples du groupe](./vue.md#vue-valid-v-bind) · [Fiche de la règle](/rules/reference/vue-valid-v-bind.md)

### `vue/valid-v-cloak`

Exiger des directives `v-cloak` valides

<span id="vue-valid-v-cloak-bad"></span>

[Mauvais](./vue.md#vue-valid-v-cloak-bad)

<span id="vue-valid-v-cloak-good"></span>

[Bon](./vue.md#vue-valid-v-cloak-good)

[Exemples du groupe](./vue.md#vue-valid-v-cloak) · [Fiche de la règle](/rules/reference/vue-valid-v-cloak.md)

### `vue/valid-v-else`

Exiger des directives `v-else` valides

<span id="vue-valid-v-else-bad"></span>

[Mauvais](./vue.md#vue-valid-v-else-bad)

<span id="vue-valid-v-else-good"></span>

[Bon](./vue.md#vue-valid-v-else-good)

[Exemples du groupe](./vue.md#vue-valid-v-else) · [Fiche de la règle](/rules/reference/vue-valid-v-else.md)

### `vue/valid-v-else-if`

Exiger des directives `v-else-if` valides

<span id="vue-valid-v-else-if-bad"></span>

[Mauvais](./vue.md#vue-valid-v-else-if-bad)

<span id="vue-valid-v-else-if-good"></span>

[Bon](./vue.md#vue-valid-v-else-if-good)

[Exemples du groupe](./vue.md#vue-valid-v-else-if) · [Fiche de la règle](/rules/reference/vue-valid-v-else-if.md)

### `vue/valid-v-for`

Exiger des directives `v-for` valides

<span id="vue-valid-v-for-bad"></span>

[Mauvais](./vue.md#vue-valid-v-for-bad)

<span id="vue-valid-v-for-good"></span>

[Bon](./vue.md#vue-valid-v-for-good)

[Exemples du groupe](./vue.md#vue-valid-v-for) · [Fiche de la règle](/rules/reference/vue-valid-v-for.md)

### `vue/valid-v-html`

Exiger des directives `v-html` valides

<span id="vue-valid-v-html-bad"></span>

[Mauvais](./vue.md#vue-valid-v-html-bad)

<span id="vue-valid-v-html-good"></span>

[Bon](./vue.md#vue-valid-v-html-good)

[Exemples du groupe](./vue.md#vue-valid-v-html) · [Fiche de la règle](/rules/reference/vue-valid-v-html.md)

### `vue/valid-v-if`

Exiger des directives `v-if` valides

<span id="vue-valid-v-if-bad"></span>

[Mauvais](./vue.md#vue-valid-v-if-bad)

<span id="vue-valid-v-if-good"></span>

[Bon](./vue.md#vue-valid-v-if-good)

[Exemples du groupe](./vue.md#vue-valid-v-if) · [Fiche de la règle](/rules/reference/vue-valid-v-if.md)

### `vue/valid-v-is`

Exiger des directives `v-is` valides

<span id="vue-valid-v-is-bad"></span>

[Mauvais](./vue.md#vue-valid-v-is-bad)

<span id="vue-valid-v-is-good"></span>

[Bon](./vue.md#vue-valid-v-is-good)

[Exemples du groupe](./vue.md#vue-valid-v-is) · [Fiche de la règle](/rules/reference/vue-valid-v-is.md)

### `vue/valid-v-memo`

Exiger des directives `v-memo` valides

<span id="vue-valid-v-memo-bad"></span>

[Mauvais](./vue.md#vue-valid-v-memo-bad)

<span id="vue-valid-v-memo-good"></span>

[Bon](./vue.md#vue-valid-v-memo-good)

[Exemples du groupe](./vue.md#vue-valid-v-memo) · [Fiche de la règle](/rules/reference/vue-valid-v-memo.md)

### `vue/valid-v-model`

Exiger des directives `v-model` valides

<span id="vue-valid-v-model-bad"></span>

[Mauvais](./vue.md#vue-valid-v-model-bad)

<span id="vue-valid-v-model-good"></span>

[Bon](./vue.md#vue-valid-v-model-good)

[Exemples du groupe](./vue.md#vue-valid-v-model) · [Fiche de la règle](/rules/reference/vue-valid-v-model.md)

### `vue/valid-v-on`

Exiger des directives `v-on` valides

<span id="vue-valid-v-on-bad"></span>

[Mauvais](./vue.md#vue-valid-v-on-bad)

<span id="vue-valid-v-on-good"></span>

[Bon](./vue.md#vue-valid-v-on-good)

[Exemples du groupe](./vue.md#vue-valid-v-on) · [Fiche de la règle](/rules/reference/vue-valid-v-on.md)

### `vue/valid-v-once`

Exiger des directives `v-once` valides

<span id="vue-valid-v-once-bad"></span>

[Mauvais](./vue.md#vue-valid-v-once-bad)

<span id="vue-valid-v-once-good"></span>

[Bon](./vue.md#vue-valid-v-once-good)

[Exemples du groupe](./vue.md#vue-valid-v-once) · [Fiche de la règle](/rules/reference/vue-valid-v-once.md)

### `vue/valid-v-pre`

Exiger des directives `v-pre` valides

<span id="vue-valid-v-pre-bad"></span>

[Mauvais](./vue.md#vue-valid-v-pre-bad)

<span id="vue-valid-v-pre-good"></span>

[Bon](./vue.md#vue-valid-v-pre-good)

[Exemples du groupe](./vue.md#vue-valid-v-pre) · [Fiche de la règle](/rules/reference/vue-valid-v-pre.md)

### `vue/valid-v-show`

Exiger des directives `v-show` valides

<span id="vue-valid-v-show-bad"></span>

[Mauvais](./vue.md#vue-valid-v-show-bad)

<span id="vue-valid-v-show-good"></span>

[Bon](./vue.md#vue-valid-v-show-good)

[Exemples du groupe](./vue.md#vue-valid-v-show) · [Fiche de la règle](/rules/reference/vue-valid-v-show.md)

### `vue/valid-v-slot`

Exiger des directives `v-slot` valides

<span id="vue-valid-v-slot-bad"></span>

[Mauvais](./vue.md#vue-valid-v-slot-bad)

<span id="vue-valid-v-slot-good"></span>

[Bon](./vue.md#vue-valid-v-slot-good)

[Exemples du groupe](./vue.md#vue-valid-v-slot) · [Fiche de la règle](/rules/reference/vue-valid-v-slot.md)

### `vue/valid-v-text`

Exiger des directives `v-text` valides

<span id="vue-valid-v-text-bad"></span>

[Mauvais](./vue.md#vue-valid-v-text-bad)

<span id="vue-valid-v-text-good"></span>

[Bon](./vue.md#vue-valid-v-text-good)

[Exemples du groupe](./vue.md#vue-valid-v-text) · [Fiche de la règle](/rules/reference/vue-valid-v-text.md)

### `vue/warn-custom-block`

Signaler les blocs personnalisés dans les fichiers SFC

<span id="vue-warn-custom-block-bad"></span>

[Mauvais](./vue.md#vue-warn-custom-block-bad)

<span id="vue-warn-custom-block-good"></span>

[Bon](./vue.md#vue-warn-custom-block-good)

[Exemples du groupe](./vue.md#vue-warn-custom-block) · [Fiche de la règle](/rules/reference/vue-warn-custom-block.md)

### `vue/warn-custom-directive`

Signaler les directives personnalisées nécessitant un enregistrement

<span id="vue-warn-custom-directive-bad"></span>

[Mauvais](./vue.md#vue-warn-custom-directive-bad)

<span id="vue-warn-custom-directive-good"></span>

[Bon](./vue.md#vue-warn-custom-directive-good)

[Exemples du groupe](./vue.md#vue-warn-custom-directive) · [Fiche de la règle](/rules/reference/vue-warn-custom-directive.md)
