---
title: "Toutes les règles Patina"
---

# Toutes les règles Patina

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.

Les 66 exemples de projets réunissent ici les fichiers communs et les exemples complets Mauvais et Bon. Utilisez les fichiers communs des deux côtés. Les 60 codes d’analyse conservent leurs limites réelles : 19 codes CLI (18 paires de sources qualifiées et un projet illustratif avec son graphe réactif conservé), 16 codes de l’analyseur Rust expérimental que cette passe CLI n’émet pas individuellement, et 25 contrats sans producteur actuel. Configurer un identifiant n’active pas un producteur indisponible.

Le CLI public expose la même passe avec `vize lint --cross-file`. Les codes affichés `vize:croquis/cf/*` utilisent `croquis/cf/*` dans `lint.vize.rules` (sans `vize:`). Les diagnostics de niveau information/hint deviennent des avertissements CLI. Les emplacements associés expliquent la relation entre la source et le consommateur.

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
| [`vue/valid-v-for`](#vue-valid-v-for) | [Mauvais](#vue-valid-v-for-bad) · [Bon](#vue-valid-v-for-good) | Exiger des directives `v-for` valides |
| [`vue/valid-v-html`](#vue-valid-v-html) | [Mauvais](#vue-valid-v-html-bad) · [Bon](#vue-valid-v-html-good) | Exiger des directives `v-html` valides |
| [`vue/valid-v-if`](#vue-valid-v-if) | [Mauvais](#vue-valid-v-if-bad) · [Bon](#vue-valid-v-if-good) | Exiger des directives `v-if` valides |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [Mauvais](#vue-valid-v-memo-bad) · [Bon](#vue-valid-v-memo-good) | Exiger des directives `v-memo` valides |
| [`vue/valid-v-model`](#vue-valid-v-model) | [Mauvais](#vue-valid-v-model-bad) · [Bon](#vue-valid-v-model-good) | Exiger des directives `v-model` valides |
| [`vue/valid-v-on`](#vue-valid-v-on) | [Mauvais](#vue-valid-v-on-bad) · [Bon](#vue-valid-v-on-good) | Exiger des directives `v-on` valides |
| [`vue/valid-v-once`](#vue-valid-v-once) | [Mauvais](#vue-valid-v-once-bad) · [Bon](#vue-valid-v-once-good) | Exiger des directives `v-once` valides |
| [`vue/valid-v-show`](#vue-valid-v-show) | [Mauvais](#vue-valid-v-show-bad) · [Bon](#vue-valid-v-show-good) | Exiger des directives `v-show` valides |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [Mauvais](#vue-valid-v-slot-bad) · [Bon](#vue-valid-v-slot-good) | Exiger des directives `v-slot` valides |
| [`vue/valid-v-text`](#vue-valid-v-text) | [Mauvais](#vue-valid-v-text-bad) · [Bon](#vue-valid-v-text-good) | Exiger des directives `v-text` valides |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [Mauvais](#vue-warn-custom-block-bad) · [Bon](#vue-warn-custom-block-good) | Signaler les blocs personnalisés dans les fichiers SFC |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [Mauvais](#vue-warn-custom-directive-bad) · [Bon](#vue-warn-custom-directive-good) | Signaler les directives personnalisées nécessitant un enregistrement |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `a11y/alt-text`

Exiger un texte alternatif pour les éléments multimédias

[Mauvais](#a11y-alt-text-bad) · [Bon](#a11y-alt-text-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le contrôle de soumission sous forme d’image ne fournit que l’URL de son image ; il n’a pas de texte `alt` décrivant l’action.

```vue annotate="remove:2"
<template>
  <input type="image" src="/submit.png" />
</template>
```

<span id="a11y-alt-text-good"></span>

**Bon**

`alt="Submit search"` donne au contrôle image un nom accessible décrivant la soumission de la recherche.

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) · [Toutes les règles](all.md)

### `a11y/anchor-has-content`

Exiger un contenu accessible pour les éléments de lien

[Mauvais](#a11y-anchor-has-content-bad) · [Bon](#a11y-anchor-has-content-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le lien `/settings` n’a ni texte ni autre contenu servant à le nommer ; sa destination n’a donc aucune description accessible.

```vue annotate="remove:2"
<template>
  <a href="/settings"></a>
</template>
```

<span id="a11y-anchor-has-content-good"></span>

**Bon**

Le texte visible `Settings` fournit le contenu du lien vers la même destination.

```vue annotate="add:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) · [Toutes les règles](all.md)

### `a11y/anchor-is-valid`

Imposer un href valide sur les éléments de lien

[Mauvais](#a11y-anchor-is-valid-bad) · [Bon](#a11y-anchor-is-valid-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le premier lien utilise `#` pour une action ; le second utilise une URL JavaScript. Aucun ne fournit une destination de navigation ordinaire.

```vue annotate="remove:2,3"
<template>
  <a href="#" @click="openPanel">Open panel</a>
  <a href="JaVaScRiPt:void(0)">Run action</a>
</template>
```

<span id="a11y-anchor-is-valid-good"></span>

**Bon**

Un bouton natif exécute `openPanel`, tandis que le lien restant possède la destination réelle `/docs/javascript-urls`.

```vue annotate="add:2,3"
<template>
  <button type="button" @click="openPanel">Open panel</button>
  <a href="/docs/javascript-urls">JavaScript URL guide</a>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) · [Toutes les règles](all.md)

### `a11y/aria-props`

Interdire les attributs ARIA invalides

[Mauvais](#a11y-aria-props-bad) · [Bon](#a11y-aria-props-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`aria-lable` est mal orthographié et n’est pas un attribut ARIA pris en charge.

```vue annotate="remove:2"
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

<span id="a11y-aria-props-good"></span>

**Bon**

L’attribut pris en charge `aria-label` fournit le nom du bouton.

```vue annotate="add:2"
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) · [Toutes les règles](all.md)

### `a11y/aria-role`

Exiger un rôle ARIA valide et non abstrait pour les éléments possédant un rôle ARIA

[Mauvais](#a11y-aria-role-bad) · [Bon](#a11y-aria-role-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`datepicker` n’est pas un rôle ARIA reconnu pour cette section.

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

<span id="a11y-aria-role-good"></span>

**Bon**

La section utilise le rôle reconnu `dialog` et un libellé décrivant la sélection de la date.

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) · [Toutes les règles](all.md)

### `a11y/aria-unsupported-elements`

Interdire les attributs ARIA sur les éléments qui ne les prennent pas en charge

[Mauvais](#a11y-aria-unsupported-elements-bad) · [Bon](#a11y-aria-unsupported-elements-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’élément de métadonnées porte `aria-hidden`, alors que `meta` ne prend pas en charge les attributs ARIA.

```vue annotate="remove:2"
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

<span id="a11y-aria-unsupported-elements-good"></span>

**Bon**

Supprimer l’attribut ARIA conserve intacte la déclaration de jeu de caractères.

```vue annotate="add:2"
<template>
  <meta charset="utf-8" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) · [Toutes les règles](all.md)

### `a11y/click-events-have-key-events`

Exiger des gestionnaires de clavier avec les événements de clic

[Mauvais](#a11y-click-events-have-key-events-bad) · [Bon](#a11y-click-events-have-key-events-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Contrôle les éléments non interactifs sans rôle interactif. Les boutons natifs et les éléments possédant un rôle ARIA interactif sont hors du champ du diagnostic de cette règle.

**Configuration (Vite+)**

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

**Mauvais**

Le `div` non interactif possède un gestionnaire de clic, mais aucune gestion des événements de clavier.

```vue annotate="remove:2"
<template>
<div @click="activate">Activate</div>
</template>
```

<span id="a11y-click-events-have-key-events-good"></span>

**Bon**

Un `button` natif permet une activation au clavier avec le même gestionnaire `activate`.

```vue annotate="add:2"
<template>
<button @click="activate">Activate</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) · [Toutes les règles](all.md)

### `a11y/form-control-has-label`

Exiger des libellés associés aux contrôles de formulaire

[Mauvais](#a11y-form-control-has-label-bad) · [Bon](#a11y-form-control-has-label-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le champ de recherche n’a pas de libellé indiquant ce que l’utilisateur doit saisir.

```vue annotate="remove:2"
<template>
  <input type="search" />
</template>
```

<span id="a11y-form-control-has-label-good"></span>

**Bon**

Envelopper le champ dans un label associe le texte visible `Search` au contrôle.

```vue annotate="add:2,3,4,5"
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) · [Toutes les règles](all.md)

### `a11y/heading-has-content`

Exiger un contenu accessible pour les éléments de titre

[Mauvais](#a11y-heading-has-content-bad) · [Bon](#a11y-heading-has-content-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le `h2` apporte un niveau de titre, mais ne contient aucun texte de titre.

```vue annotate="remove:2"
<template>
  <h2></h2>
</template>
```

<span id="a11y-heading-has-content-good"></span>

**Bon**

`Billing settings` fournit le contenu du titre de niveau deux existant.

```vue annotate="add:2"
<template>
  <h2>Billing settings</h2>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) · [Toutes les règles](all.md)

### `a11y/heading-levels`

Interdire de sauter des niveaux de titre

[Mauvais](#a11y-heading-levels-bad) · [Bon](#a11y-heading-levels-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La séquence des titres passe directement de `h1` à `h3`, sans niveau deux.

```vue annotate="remove:3"
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

<span id="a11y-heading-levels-good"></span>

**Bon**

Remplacer le titre de facturation par `h2` conserve une hiérarchie de titres consécutive.

```vue annotate="add:3"
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) · [Toutes les règles](all.md)

### `a11y/iframe-has-title`

Exiger un attribut title sur les éléments iframe

[Mauvais](#a11y-iframe-has-title-bad) · [Bon](#a11y-iframe-has-title-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le cadre de paiement possède une URL source, mais aucun `title` décrivant le contenu intégré.

```vue annotate="remove:2"
<template>
  <iframe src="/checkout"></iframe>
</template>
```

<span id="a11y-iframe-has-title-good"></span>

**Bon**

`title="Checkout preview"` nomme le contenu de ce cadre.

```vue annotate="add:2"
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) · [Toutes les règles](all.md)

### `a11y/img-alt`

Exiger un attribut alt sur les images pour les rendre accessibles

[Mauvais](#a11y-img-alt-bad) · [Bon](#a11y-img-alt-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’image de l’avatar ne possède pas d’attribut `alt`.

```vue annotate="remove:2"
<template>
  <img src="/avatar.png" />
</template>
```

<span id="a11y-img-alt-good"></span>

**Bon**

`alt="User avatar"` fournit une alternative textuelle à l’avatar.

```vue annotate="add:2"
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) · [Toutes les règles](all.md)

### `a11y/interactive-supports-focus`

Exiger que les éléments possédant un rôle interactif puissent recevoir le focus

[Mauvais](#a11y-interactive-supports-focus-bad) · [Bon](#a11y-interactive-supports-focus-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Donner à un `span` le rôle button et un gestionnaire de clic ne rend pas l’élément accessible au focus clavier.

```vue annotate="remove:2"
<template>
  <span role="button" @click="open">Open</span>
</template>
```

<span id="a11y-interactive-supports-focus-good"></span>

**Bon**

Le bouton natif peut recevoir le focus et conserve la même action `open`.

```vue annotate="add:2"
<template>
  <button type="button" @click="open">Open</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) · [Toutes les règles](all.md)

### `a11y/label-has-for`

Exiger des contrôles de formulaire associés aux libellés

[Mauvais](#a11y-label-has-for-bad) · [Bon](#a11y-label-has-for-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le label séparé n’est ni associé au moyen de `for` ni placé autour du champ.

```vue annotate="remove:2"
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

<span id="a11y-label-has-for-good"></span>

**Bon**

`for="email"` correspond à l’identifiant du champ et associe explicitement les deux éléments.

```vue annotate="add:2"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) · [Toutes les règles](all.md)

### `a11y/landmark-roles`

Valider l’emplacement et l’unicité des rôles de zones de repère

[Mauvais](#a11y-landmark-roles-bad) · [Bon](#a11y-landmark-roles-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Deux éléments `main` déclarent des zones principales en double dans le même template.

```vue annotate="remove:3"
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

<span id="a11y-landmark-roles-good"></span>

**Bon**

Le tableau de bord reste la zone principale ; la zone des paramètres devient une zone de navigation nommée.

```vue annotate="add:3"
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) · [Toutes les règles](all.md)

### `a11y/media-has-caption`

Exiger des sous-titres pour les éléments multimédias

[Mauvais](#a11y-media-has-caption-bad) · [Bon](#a11y-media-has-caption-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La vidéo possède des contrôles de lecture, mais aucune piste de sous-titres.

```vue annotate="remove:2"
<template>
  <video src="/demo.mp4" controls />
</template>
```

<span id="a11y-media-has-caption-good"></span>

**Bon**

Un `track` avec `kind="captions"` fournit les sous-titres anglais de la même vidéo.

```vue annotate="add:2,3,4"
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) · [Toutes les règles](all.md)

### `a11y/mouse-events-have-key-events`

Exiger des événements de focus et de perte du focus avec les événements de souris

[Mauvais](#a11y-mouse-events-have-key-events-bad) · [Bon](#a11y-mouse-events-have-key-events-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La visibilité de l’aperçu change uniquement avec les gestionnaires d’entrée et de sortie de la souris.

```vue annotate="remove:2"
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

<span id="a11y-mouse-events-have-key-events-good"></span>

**Bon**

Les mêmes actions d’aperçu s’exécutent lors du focus et de sa perte, et le bouton peut recevoir le focus clavier.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) · [Toutes les règles](all.md)

### `a11y/no-access-key`

Interdire l’utilisation de l’attribut accesskey

[Mauvais](#a11y-no-access-key-bad) · [Bon](#a11y-no-access-key-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le raccourci `accesskey="s"` peut entrer en conflit avec les raccourcis du navigateur ou des technologies d’assistance.

```vue annotate="remove:2"
<template>
  <button accesskey="s">Save</button>
</template>
```

<span id="a11y-no-access-key-good"></span>

**Bon**

Supprimer `accesskey` conserve le bouton Save ordinaire.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) · [Toutes les règles](all.md)

### `a11y/no-aria-hidden-on-focusable`

Interdire aria-hidden="true" sur les éléments pouvant recevoir le focus

[Mauvais](#a11y-no-aria-hidden-on-focusable-bad) · [Bon](#a11y-no-aria-hidden-on-focusable-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le bouton Close pouvant recevoir le focus est masqué dans l’arbre d’accessibilité avec `aria-hidden="true"`.

```vue annotate="remove:2"
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

**Bon**

Le bouton reste exposé et reçoit un libellé `Close` au lieu d’être masqué.

```vue annotate="add:2"
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) · [Toutes les règles](all.md)

### `a11y/no-autofocus`

Interdire l’utilisation de l’attribut autofocus

[Mauvais](#a11y-no-autofocus-bad) · [Bon](#a11y-no-autofocus-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le champ demande à recevoir automatiquement le focus lorsqu’il apparaît.

```vue annotate="remove:2"
<template>
  <input autofocus name="query" />
</template>
```

<span id="a11y-no-autofocus-good"></span>

**Bon**

Supprimer `autofocus` évite cette demande de focus automatique tout en conservant le champ de requête.

```vue annotate="add:2"
<template>
  <input name="query" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) · [Toutes les règles](all.md)

### `a11y/no-distracting-elements`

Interdire les éléments distrayants tels que &lt;marquee&gt; et &lt;blink&gt;

[Mauvais](#a11y-no-distracting-elements-bad) · [Bon](#a11y-no-distracting-elements-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’élément `marquee` introduit un texte en mouvement automatique.

```vue annotate="remove:2"
<template>
  <marquee>Limited offer</marquee>
</template>
```

<span id="a11y-no-distracting-elements-good"></span>

**Bon**

Un paragraphe affiche la même offre sans l’élément marquee distrayant.

```vue annotate="add:2"
<template>
  <p>Limited offer</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) · [Toutes les règles](all.md)

### `a11y/no-i-for-icon`

Interdire l’élément &lt;i&gt; pour les icônes

[Mauvais](#a11y-no-i-for-icon-bad) · [Bon](#a11y-no-i-for-icon-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’icône est affichée avec `i`, dont la sémantique textuelle ne décrit pas une action représentée uniquement par une icône.

```vue annotate="remove:3"
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

<span id="a11y-no-i-for-icon-good"></span>

**Bon**

Un span décoratif masque le glyphe de l’icône, tandis que le texte séparé `Delete item` nomme l’action du bouton.

```vue annotate="add:3,4"
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) · [Toutes les règles](all.md)

### `a11y/no-redundant-roles`

Interdire les rôles ARIA redondants

[Mauvais](#a11y-no-redundant-roles-bad) · [Bon](#a11y-no-redundant-roles-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le bouton natif possède déjà le rôle button ; `role="button"` répète donc sa sémantique implicite.

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

<span id="a11y-no-redundant-roles-good"></span>

**Bon**

Supprimer le rôle répété conserve la sémantique du bouton fournie par HTML.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) · [Toutes les règles](all.md)

### `a11y/no-refer-to-non-existent-id`

Interdire les références à des identifiants inexistants

[Mauvais](#a11y-no-refer-to-non-existent-id-bad) · [Bon](#a11y-no-refer-to-non-existent-id-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`aria-labelledby` pointe vers `save-label`, mais aucun élément ne déclare cet identifiant.

```vue
<template>
  <button aria-labelledby="save-label">Save</button>
</template>
```

<span id="a11y-no-refer-to-non-existent-id-good"></span>

**Bon**

Ajouter le span correspondant résout la référence et fournit le libellé du bouton.

```vue annotate="add:2"
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) · [Toutes les règles](all.md)

### `a11y/no-role-presentation-on-focusable`

Interdire role="presentation" ou role="none" sur les éléments pouvant recevoir le focus

[Mauvais](#a11y-no-role-presentation-on-focusable-bad) · [Bon](#a11y-no-role-presentation-on-focusable-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le lien de facturation pouvant recevoir le focus demande role=presentation, ce qui entre en conflit avec son rôle de lien interactif ; les navigateurs doivent ignorer cette demande de présentation.

```vue annotate="remove:2"
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

<span id="a11y-no-role-presentation-on-focusable-good"></span>

**Bon**

Supprimez la demande de présentation contradictoire et utilisez le rôle de lien natif ainsi que la destination de facturation.

```vue annotate="add:2"
<template>
  <a href="/billing">Billing</a>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) · [Toutes les règles](all.md)

### `a11y/no-static-element-interactions`

Interdire les gestionnaires d’événements sur les éléments statiques

[Mauvais](#a11y-no-static-element-interactions-bad) · [Bon](#a11y-no-static-element-interactions-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Une section statique reçoit une action liée à la touche Entrée sans posséder de rôle interactif.

```vue annotate="remove:2"
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

<span id="a11y-no-static-element-interactions-good"></span>

**Bon**

Un bouton natif porte la même action dans un élément interactif approprié.

```vue annotate="add:2"
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) · [Toutes les règles](all.md)

### `a11y/placeholder-label-option`

Exiger disabled ou hidden sur l’option d’invite d’un select

[Mauvais](#a11y-placeholder-label-option-bad) · [Bon](#a11y-placeholder-label-option-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’invite à valeur vide reste sélectionnable comme s’il s’agissait d’une valeur de pays.

```vue annotate="remove:3"
<template>
  <select v-model="country">
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

<span id="a11y-placeholder-label-option-good"></span>

**Bon**

Ajouter `disabled` distingue l’invite de l’option Japan sélectionnable.

```vue annotate="add:3"
<template>
  <select v-model="country">
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L36) · [Toutes les règles](all.md)

### `a11y/role-has-required-aria-props`

Exiger les propriétés obligatoires des rôles ARIA

[Mauvais](#a11y-role-has-required-aria-props-bad) · [Bon](#a11y-role-has-required-aria-props-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le rôle checkbox omet `aria-checked`, qui communique l’état de la case à cocher.

```vue annotate="remove:2"
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

<span id="a11y-role-has-required-aria-props-good"></span>

**Bon**

`aria-checked="false"` fournit l’état exigé par le rôle checkbox.

```vue annotate="add:2"
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) · [Toutes les règles](all.md)

### `a11y/tabindex-no-positive`

Interdire les valeurs positives de tabindex

[Mauvais](#a11y-tabindex-no-positive-bad) · [Bon](#a11y-tabindex-no-positive-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un tabindex positif de 3 crée un ordre de focus personnalisé avant les contrôles ordinaires.

```vue annotate="remove:2"
<template>
  <button tabindex="3">Save</button>
</template>
```

<span id="a11y-tabindex-no-positive-good"></span>

**Bon**

Le bouton utilise son ordre de focus natif sans tabindex positif.

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) · [Toutes les règles](all.md)

### `a11y/use-list`

Suggérer des éléments de liste pour les textes ressemblant à des listes à puces

[Mauvais](#a11y-use-list-bad) · [Bon](#a11y-use-list-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les tâches sont des paragraphes séparés précédés de tirets saisis, plutôt que des éléments de liste.

```vue annotate="remove:2,3"
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

<span id="a11y-use-list-good"></span>

**Bon**

Une liste non ordonnée et ses éléments expriment les mêmes tâches avec une sémantique de liste.

```vue annotate="add:2,3,4,5"
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) · [Toutes les règles](all.md)

### `css/no-display-none`

Suggérer v-show plutôt que display: none

[Mauvais](#css-no-display-none-bad) · [Bon](#css-no-display-none-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La déclaration `.message` masque le paragraphe local au moyen du CSS plutôt que d’une condition de visibilité dans le template.

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

**Bon**

`v-show="isSaved"` rend la condition de visibilité explicite sur le paragraphe local et supprime `display: none`.

```vue annotate="add:2"
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) · [Toutes les règles](all.md)

### `css/no-hardcoded-values`

Suggérer des variables CSS plutôt que des valeurs codées en dur

[Mauvais](#css-no-hardcoded-values-bad) · [Bon](#css-no-hardcoded-values-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le bouton place directement des nombres d’espacement et une couleur hexadécimale dans les déclarations.

```vue annotate="remove:3,4"
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

<span id="css-no-hardcoded-values-good"></span>

**Bon**

Les déclarations référencent des propriétés personnalisées d’espacement et de couleur nommées, afin que ces valeurs puissent être maintenues sous forme de tokens.

```vue annotate="add:3,4"
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30) · [Toutes les règles](all.md)

### `css/no-id-selectors`

Déconseiller les sélecteurs d’identifiant en CSS

[Mauvais](#css-no-id-selectors-bad) · [Bon](#css-no-id-selectors-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`#submit` lie la règle de style à un sélecteur d’identifiant.

```vue annotate="remove:2"
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

<span id="css-no-id-selectors-good"></span>

**Bon**

La classe `.submit` fournit un point d’accroche de style réutilisable sans sélecteur d’identifiant.

```vue annotate="add:2"
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) · [Toutes les règles](all.md)

### `css/no-important`

Déconseiller !important en CSS

[Mauvais](#css-no-important-bad) · [Bon](#css-no-important-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La déclaration de couleur remplace la priorité normale de la cascade avec `!important`.

```vue annotate="remove:3"
<style scoped>
.button {
  color: red !important;
}
</style>
```

<span id="css-no-important-good"></span>

**Bon**

La couleur provient d’une propriété personnalisée sans déclaration important.

```vue annotate="add:3"
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) · [Toutes les règles](all.md)

### `css/no-utility-classes`

Déconseiller l’implémentation de classes utilitaires dans les styles des composants

[Mauvais](#css-no-utility-classes-bad) · [Bon](#css-no-utility-classes-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les sélecteurs écrits utilisent des noms de forme utilitaire, tels que `.flex`, `.mt-4` et `.text-center`.

```vue annotate="remove:2,3,4"
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

<span id="css-no-utility-classes-good"></span>

**Bon**

Un sélecteur `.my-component` propre au composant rassemble ses styles sous un nom sémantique unique.

```vue annotate="add:2"
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) · [Toutes les règles](all.md)

### `css/no-v-bind-performance`

Signaler le coût de performance de v-bind() en CSS

[Mauvais](#css-no-v-bind-performance-bad) · [Bon](#css-no-v-bind-performance-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La feuille de style lit la valeur changeante `offset` au moyen du mécanisme CSS `v-bind()` du SFC.

```vue annotate="remove:1,2,3,4,5"
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

<span id="css-no-v-bind-performance-good"></span>

**Bon**

L’élément reçoit directement la transformation changeante via sa liaison de style.

```vue annotate="add:1,2,3"
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) · [Toutes les règles](all.md)

### `css/prefer-logical-properties`

Recommander les propriétés logiques CSS pour mieux prendre en charge l’internationalisation

[Mauvais](#css-prefer-logical-properties-bad) · [Bon](#css-prefer-logical-properties-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`margin-left` fixe la marge sur un côté physique indépendamment du sens d’écriture.

```vue annotate="remove:3"
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

<span id="css-prefer-logical-properties-good"></span>

**Bon**

`margin-inline-start` suit plutôt le début de la direction en ligne.

```vue annotate="add:3"
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) · [Toutes les règles](all.md)

### `css/prefer-nested-selectors`

Recommander l’imbrication CSS pour les sélecteurs de descendants

[Mauvais](#css-prefer-nested-selectors-bad) · [Bon](#css-prefer-nested-selectors-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le sélecteur de descendant `.card .title` répète le sélecteur parent dans une règle à plat.

```vue annotate="remove:2"
<style scoped>
.card .title { color: red; }
</style>
```

<span id="css-prefer-nested-selectors-good"></span>

**Bon**

La règle `.title` est imbriquée dans `.card`, ce qui rassemble la relation de style parent-enfant.

```vue annotate="add:2"
<style scoped>
.card { .title { color: red; } }
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) · [Toutes les règles](all.md)

### `css/prefer-slotted`

Recommander ::v-slotted() pour styliser le contenu des slots

[Mauvais](#css-prefer-slotted-bad) · [Bon](#css-prefer-slotted-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La feuille de style scoped cible le point d’insertion `slot` plutôt que les éléments fournis au slot.

```vue annotate="remove:2"
<style scoped>
slot { color: red; }
</style>
```

<span id="css-prefer-slotted-good"></span>

**Bon**

`:slotted(.label)` cible l’élément label fourni au moyen du sélecteur de slot scoped.

```vue annotate="add:2"
<style scoped>
:slotted(.label) { color: red; }
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) · [Toutes les règles](all.md)

### `css/require-font-display`

Exiger font-display dans les règles @font-face

[Mauvais](#css-require-font-display-bad) · [Bon](#css-require-font-display-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`, `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: CSS dans les blocs style des SFC  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La déclaration font-face définit la source de la police, mais omet sa politique font-display.

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

<span id="css-require-font-display-good"></span>

**Bon**

`font-display: swap` sélectionne explicitement la politique d’affichage passant d’une police de repli à la police chargée.

```vue annotate="add:5"
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13) · [Toutes les règles](all.md)

### `ecosystem/nuxt-prefer-nuxt-link`

Préférer NuxtLink pour les liens internes à l’application

[Mauvais](#ecosystem-nuxt-prefer-nuxt-link-bad) · [Bon](#ecosystem-nuxt-prefer-nuxt-link-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La destination interne des paramètres utilise un lien ordinaire dans une application Nuxt.

```vue annotate="remove:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

**Bon**

NuxtLink gère la même destination interne au moyen du routeur Nuxt.

```vue annotate="add:2"
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) · [Toutes les règles](all.md)

### `ecosystem/pinia-prefer-store-to-refs`

Préférer storeToRefs() lors de la déstructuration des stores Pinia

[Mauvais](#ecosystem-pinia-prefer-store-to-refs-bad) · [Bon](#ecosystem-pinia-prefer-store-to-refs-good)

Gravité par défaut: `warning`  
Préréglages: `ecosystem`  
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

**Mauvais**

Déstructurer `name` directement depuis le store sépare la valeur de son accès réactif au store.

```vue annotate="remove:2"
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

**Bon**

Le store reste intact et storeToRefs crée une référence réactive pour name.

```vue annotate="add:2,3"
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) · [Toutes les règles](all.md)

### `ecosystem/router-link-require-to`

Exiger une destination `to` sur les composants RouterLink et NuxtLink

[Mauvais](#ecosystem-router-link-require-to-bad) · [Bon](#ecosystem-router-link-require-to-good)

Gravité par défaut: `error`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Un lien constituant l’unique racine d’un SFC peut hériter de sa destination via les attributs du parent. Cet exemple utilise un lien imbriqué, dont la destination doit être explicite.

**Configuration (Vite+)**

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

**Mauvais**

Le RouterLink imbriqué n’a pas de destination `to` ; il ne peut pas compter sur la transmission automatique des attributs racines.

```vue annotate="remove:2"
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

<span id="ecosystem-router-link-require-to-good"></span>

**Bon**

`to="/settings"` fournit explicitement la destination du lien imbriqué.

```vue annotate="add:2"
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) · [Toutes les règles](all.md)

### `ecosystem/void-link-require-href`

Exiger `href` sur les composants Link de Void Vue

[Mauvais](#ecosystem-void-link-require-href-bad) · [Bon](#ecosystem-void-link-require-href-good)

Gravité par défaut: `error`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le Link importé depuis @void/vue omet sa destination href.

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

<span id="ecosystem-void-link-require-href-good"></span>

**Bon**

Le même Link importé reçoit la destination des paramètres via href.

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) · [Toutes les règles](all.md)

### `ecosystem/void-link-valid-method`

Valider les props method statiques des composants Link de Void Vue

[Mauvais](#ecosystem-void-link-valid-method-bad) · [Bon](#ecosystem-void-link-valid-method-good)

Gravité par défaut: `warning`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’action DELETE demande un préchargement, alors que prefetch est destiné aux requêtes de navigation.

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

<span id="ecosystem-void-link-valid-method-good"></span>

**Bon**

Supprimer prefetch conserve l’action DELETE sans précharger cette requête non GET.

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14) · [Toutes les règles](all.md)

### `ecosystem/vue-i18n-no-missing-key`

Signaler les clés vue-i18n statiques absentes des messages locaux du SFC

[Mauvais](#ecosystem-vue-i18n-no-missing-key-bad) · [Bon](#ecosystem-vue-i18n-no-missing-key-good)

Gravité par défaut: `warning`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template demande auth.missing, mais les messages anglais locaux ne déclarent que auth.login.

```vue annotate="remove:1"
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

**Bon**

Le template demande la clé auth.login qui existe dans les messages locaux.

```vue annotate="add:1"
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17) · [Toutes les règles](all.md)

### `ecosystem/vue-router-extra-param`

La route ne déclare pas tab ; Vue Router l’ignore.

Gravité par défaut: error  
Champ d’application: Déclarations du projet accessibles et composants importés  
Options: crossFile ; gravité de la règle (off/warn/error)  
Correction automatique: Aucune

Le routeur installé complet doit être accessible depuis createApp(...).use(router) dans l’application. Des tables de routes inconnues ou dynamiques ne prouvent pas les diagnostics de noms inconnus. Les paramètres manquants sont des avertissements, car la navigation peut hériter d’une valeur de la route actuelle.

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le chemin `user-post` déclare `userId` et `postId`, mais la navigation fournit également `tab`, non déclaré, comme paramètre de chemin.

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

**Bon**

Supprimer `tab` de params et conserver uniquement les clés présentes dans le chemin de la route. Utiliser query séparément si l’application a besoin de sélectionner un onglet.

`src/UserPost.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Index des règles inter-fichiers](cross-file.md)

### `ecosystem/vue-router-missing-param`

Le paramètre obligatoire postId manque ; dépendre de la route actuelle est fragile.

Gravité par défaut: warning  
Champ d’application: Déclarations du projet accessibles et composants importés  
Options: crossFile ; gravité de la règle (off/warn/error)  
Correction automatique: Aucune

Le routeur installé complet doit être accessible depuis createApp(...).use(router) dans l’application. Des tables de routes inconnues ou dynamiques ne prouvent pas les diagnostics de noms inconnus. Les paramètres manquants sont des avertissements, car la navigation peut hériter d’une valeur de la route actuelle.

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

La navigation omet le paramètre obligatoire `postId` du chemin `user-post`. Il s’agit d’un avertissement, car Vue Router peut hériter d’une valeur de la route actuelle.

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

**Bon**

Passer explicitement `userId` et `postId` pour que la navigation ne dépende pas de l’état des paramètres de la route actuelle.

`src/UserPost.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Index des règles inter-fichiers](cross-file.md)

### `ecosystem/vue-router-param-type`

postId n’est pas répétable ; un tableau est donc invalide.

Gravité par défaut: error  
Champ d’application: Déclarations du projet accessibles et composants importés  
Options: crossFile ; gravité de la règle (off/warn/error)  
Correction automatique: Aucune

Le routeur installé complet doit être accessible depuis createApp(...).use(router) dans l’application. Des tables de routes inconnues ou dynamiques ne prouvent pas les diagnostics de noms inconnus. Les paramètres manquants sont des avertissements, car la navigation peut hériter d’une valeur de la route actuelle.

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`postId` est un paramètre de chemin scalaire, mais la navigation lui donne le tableau `["2"]`.

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

**Bon**

Passer la valeur scalaire `"2"` pour le segment `postId` non répétable.

`src/UserPost.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Index des règles inter-fichiers](cross-file.md)

### `ecosystem/vue-router-prefer-named-link`

Préférer les objets de routes nommées aux chaînes de chemin statiques dans RouterLink

[Mauvais](#ecosystem-vue-router-prefer-named-link-bad) · [Bon](#ecosystem-vue-router-prefer-named-link-good)

Gravité par défaut: `warning`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La destination de RouterLink est un chemin littéral plutôt qu’une route nommée.

```vue annotate="remove:2"
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

**Bon**

L’objet de route lié identifie la destination par son nom de route settings.

```vue annotate="add:2"
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) · [Toutes les règles](all.md)

### `ecosystem/vue-router-prefer-named-push`

Préférer les objets de routes nommées pour la navigation programmatique de Vue Router

[Mauvais](#ecosystem-vue-router-prefer-named-push-bad) · [Bon](#ecosystem-vue-router-prefer-named-push-good)

Gravité par défaut: `warning`  
Préréglages: `ecosystem`  
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

**Mauvais**

router.push reçoit une chaîne de chemin liée à la graphie actuelle de l’URL.

```vue annotate="remove:2"
<script setup lang="ts">
router.push("/settings");
</script>
```

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

**Bon**

router.push reçoit un objet de route possédant le nom stable settings.

```vue annotate="add:2"
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) · [Toutes les règles](all.md)

### `ecosystem/vue-router-unknown-route`

Le nom est absent du routeur installé complet.

Gravité par défaut: error  
Champ d’application: Déclarations du projet accessibles et composants importés  
Options: crossFile ; gravité de la règle (off/warn/error)  
Correction automatique: Aucune

Le routeur installé complet doit être accessible depuis createApp(...).use(router) dans l’application. Des tables de routes inconnues ou dynamiques ne prouvent pas les diagnostics de noms inconnus. Les paramètres manquants sont des avertissements, car la navigation peut hériter d’une valeur de la route actuelle.

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le routeur installé accessible déclare `user-post`, mais la navigation utilise le nom mal orthographié `user-posts`.

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

**Bon**

Utiliser le nom enregistré `user-post` tout en conservant les deux paramètres de chemin déclarés.

`src/UserPost.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Index des règles inter-fichiers](cross-file.md)

### `ecosystem/vue-test-utils-no-html-snapshot`

Éviter les instantanés de wrapper.html() dans les tests Vue Test Utils

[Mauvais](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Bon](#ecosystem-vue-test-utils-no-html-snapshot-good)

Gravité par défaut: `warning`  
Préréglages: `ecosystem`  
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

**Mauvais**

L’assertion capture l’ensemble du HTML du wrapper au lieu de vérifier le comportement attendu.

```vue annotate="remove:2"
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

**Bon**

L’assertion vérifie que le texte affiché contient Saved.

```vue annotate="add:2"
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) · [Toutes les règles](all.md)

### `html/cross-component-nesting`

Vérifier l’imbrication HTML réelle après la composition des composants importés.

Gravité par défaut: warning  
Champ d’application: Déclarations du projet accessibles et composants importés  
Options: crossFile ; gravité de la règle (off/warn/error)  
Correction automatique: Aucune

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le `<p>` du parent contient un enfant résolu dont la racine est `<div>`, ce qui produit une imbrication paragraphe/bloc invalide après composition.

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

**Bon**

Utiliser un conteneur `<section>` pouvant contenir l’élément de bloc de l’enfant ; l’enfant reste inchangé.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Index des règles inter-fichiers](cross-file.md)

### `html/deprecated-attr`

Interdire les attributs HTML obsolètes

[Mauvais](#html-deprecated-attr-bad) · [Bon](#html-deprecated-attr-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le paragraphe utilise l’attribut de présentation obsolète `align`.

```vue annotate="remove:1,2,3"
<template>
<p align="center">Notice</p>
</template>
```

<span id="html-deprecated-attr-good"></span>

**Bon**

La classe et la déclaration `text-align: center` expriment l’alignement au moyen du CSS.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) · [Toutes les règles](all.md)

### `html/deprecated-element`

Interdire les éléments HTML obsolètes

[Mauvais](#html-deprecated-element-bad) · [Bon](#html-deprecated-element-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’élément `center` utilise un élément de présentation HTML obsolète.

```vue annotate="remove:2"
<template>
  <center>Profile</center>
</template>
```

<span id="html-deprecated-element-good"></span>

**Bon**

Une section et une classe de style remplacent l’élément obsolète tout en préservant le contenu.

```vue annotate="add:2"
<template>
  <section class="profile">Profile</section>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33) · [Toutes les règles](all.md)

### `html/id-duplication`

Interdire les identifiants d’éléments en double

[Mauvais](#html-id-duplication-bad) · [Bon](#html-id-duplication-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le champ et le paragraphe d’aide déclarent tous deux `id="email"`, rendant la cible du label ambiguë.

```vue annotate="remove:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

<span id="html-id-duplication-good"></span>

**Bon**

Le champ conserve `email` ; le paragraphe d’aide utilise `email-help`, et aria-describedby référence cet identifiant distinct.

```vue annotate="add:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) · [Toutes les règles](all.md)

### `html/no-consecutive-br`

Interdire les éléments &lt;br&gt; consécutifs

[Mauvais](#html-no-consecutive-br-bad) · [Bon](#html-no-consecutive-br-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Deux éléments de saut de ligne consécutifs créent un espacement entre des blocs au sein d’un seul paragraphe.

```vue annotate="remove:2"
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

<span id="html-no-consecutive-br-good"></span>

**Bon**

Des paragraphes séparés expriment les deux blocs de contenu sans répéter les éléments de saut de ligne.

```vue annotate="add:2,3"
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) · [Toutes les règles](all.md)

### `html/no-dupe-style-properties`

Interdire les propriétés en double dans les attributs de style en ligne

[Mauvais](#html-no-dupe-style-properties-bad) · [Bon](#html-no-dupe-style-properties-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Chaque style statique répète une propriété ; `margin` et `MARGIN` sont également considérés comme la même propriété.

```vue annotate="remove:2,3"
<template>
  <div style="color: red; color: blue">text</div>
  <div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

<span id="html-no-dupe-style-properties-good"></span>

**Bon**

Le style statique utilise des propriétés distinctes de couleur et d’arrière-plan. Les liaisons de style dynamiques sont hors du champ de ce contrôle des attributs statiques.

```vue annotate="add:2,3"
<template>
  <div style="color: red; background: blue">text</div>
  <div :style="{ color: a, color: b }">text</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) · [Toutes les règles](all.md)

### `html/no-duplicate-class`

Interdire les noms de classes en double dans un attribut class statique

[Mauvais](#html-no-duplicate-class-bad) · [Bon](#html-no-duplicate-class-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La liste de classes statique répète le nom `btn`.

```vue annotate="remove:2"
<template>
  <div class="btn btn primary">click</div>
</template>
```

<span id="html-no-duplicate-class-good"></span>

**Bon**

La liste de classes conserve une seule occurrence de `btn` et le nom distinct `primary`.

```vue annotate="add:2"
<template>
  <div class="btn primary">click</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) · [Toutes les règles](all.md)

### `html/no-duplicate-dt`

Interdire les noms &lt;dt&gt; en double dans &lt;dl&gt;

[Mauvais](#html-no-duplicate-dt-bad) · [Bon](#html-no-duplicate-dt-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La même liste de définitions répète le terme `API` pour deux descriptions.

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

**Bon**

Un seul terme API est suivi des deux descriptions, évitant de répéter le terme.

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_duplicate_dt.rs#L41) · [Toutes les règles](all.md)

### `html/no-empty-palpable-content`

Interdire les éléments vides qui attendent un contenu visible

[Mauvais](#html-no-empty-palpable-content-bad) · [Bon](#html-no-empty-palpable-content-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

Le paragraphe, l’élément de liste et la cellule de tableau ont tous un contenu palpable vide.

```vue annotate="remove:2,3,4"
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

<span id="html-no-empty-palpable-content-good"></span>

**Bon**

Du texte remplit le paragraphe, une interpolation fournit le contenu de l’élément de liste, et aria-label nomme explicitement la cellule autrement vide.

```vue annotate="add:2,3,4"
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) · [Toutes les règles](all.md)

### `html/require-datetime`

Exiger l’attribut datetime sur l’élément &lt;time&gt;

[Mauvais](#html-require-datetime-bad) · [Bon](#html-require-datetime-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’élément time contient une date lisible par une personne, mais aucune valeur datetime lisible par une machine.

```vue annotate="remove:2"
<template>
  <time>May 13, 2026</time>
</template>
```

<span id="html-require-datetime-good"></span>

**Bon**

`datetime="2026-05-13"` fournit la date correspondante lisible par une machine.

```vue annotate="add:2"
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) · [Toutes les règles](all.md)

### `musea/no-empty-variant`

Interdire les blocs &lt;variant&gt; vides

[Mauvais](#musea-no-empty-variant-bad) · [Bon](#musea-no-empty-variant-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Blocs art, variant et style des fichiers .art.vue de Musea  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le variant nommé primary est vide et ne fournit donc aucun contenu d’aperçu.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-no-empty-variant-good"></span>

**Bon**

Le variant affiche un Button primary avec son contenu Save.

```vue annotate="add:2,3,4"
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8) · [Toutes les règles](all.md)

### `musea/prefer-design-tokens`

Préférer les variables CSS de design tokens aux valeurs primitives codées en dur

[Mauvais](#musea-prefer-design-tokens-bad) · [Bon](#musea-prefer-design-tokens-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Blocs art, variant et style des fichiers .art.vue de Musea  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

Nécessite un fichier .art.vue et l’inventaire des tokens présenté ci-dessous. Aucun token n’est déduit d’une couleur arbitraire.

**Configuration (Vite+)**

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

**Mauvais**

L’exemple art utilise la couleur bleue littérale au lieu du design token primaire configuré.

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

**Bon**

Le style référence --color-primary, le token configuré pour cet exemple.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) · [Toutes les règles](all.md)

### `musea/require-component`

Exiger l’attribut component dans le bloc &lt;art&gt;

[Mauvais](#musea-require-component-bad) · [Bon](#musea-require-component-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Blocs art, variant et style des fichiers .art.vue de Musea  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le bloc art fournit un titre, mais n’identifie pas le composant dont il présente l’aperçu.

```vue annotate="remove:1"
<art title="Button">
  <variant name="primary" />
</art>
```

<span id="musea-require-component-good"></span>

**Bon**

defineArt fournit ./Button.vue comme composant du bloc art.

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) · [Toutes les règles](all.md)

### `musea/require-title`

Exiger l’attribut title dans le bloc &lt;art&gt;

[Mauvais](#musea-require-title-bad) · [Bon](#musea-require-title-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Blocs art, variant et style des fichiers .art.vue de Musea  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le bloc art identifie Button.vue, mais ne fournit aucun titre.

```vue annotate="remove:1"
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-require-title-good"></span>

**Bon**

Les options de defineArt fournissent le titre Button pour le bloc art.

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) · [Toutes les règles](all.md)

### `musea/unique-variant-names`

Exiger des noms de variants uniques

[Mauvais](#musea-unique-variant-names-bad) · [Bon](#musea-unique-variant-names-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Blocs art, variant et style des fichiers .art.vue de Musea  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Deux variants du même bloc art utilisent tous deux le nom primary.

```vue annotate="remove:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

<span id="musea-unique-variant-names-good"></span>

**Bon**

Les variants possèdent les noms distincts primary et secondary.

```vue annotate="add:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10) · [Toutes les règles](all.md)

### `musea/valid-variant`

Exiger un attribut name dans les blocs &lt;variant&gt;

[Mauvais](#musea-valid-variant-bad) · [Bon](#musea-valid-variant-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Blocs art, variant et style des fichiers .art.vue de Musea  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le variant omet le nom nécessaire pour identifier l’aperçu.

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

<span id="musea-valid-variant-good"></span>

**Bon**

Le nom primary identifie ce variant.

```vue annotate="add:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) · [Toutes les règles](all.md)

### `nuxt/no-nuxt-config-test-key`

Interdire la clé `test` dans la configuration Nuxt

[Mauvais](#nuxt-no-nuxt-config-test-key-bad) · [Bon](#nuxt-no-nuxt-config-test-key-good)

Gravité par défaut: `error`  
Préréglages: `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Fichiers de configuration Nuxt (nuxt.config.ts)  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La configuration Nuxt exportée définit la clé d’identifiant `test` sur le booléen `true`, la forme de configuration obsolète rejetée par cette règle.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ test: true });
```

<span id="nuxt-no-nuxt-config-test-key-good"></span>

**Bon**

La configuration vide supprime cette propriété booléenne `test`. Cet exemple n’interdit pas un objet de configuration de test.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({});
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) · [Toutes les règles](all.md)

### `nuxt/no-page-meta-runtime-values`

Interdire les valeurs du contexte d’exécution évaluées immédiatement dans `definePageMeta`, extrait dans un chunk séparé à la compilation et exécuté avant le setup du composant

[Mauvais](#nuxt-no-page-meta-runtime-values-bad) · [Bon](#nuxt-no-page-meta-runtime-values-good)

Gravité par défaut: `error`  
Préréglages: `nuxt`  
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

**Mauvais**

`useRoute()` est évalué immédiatement lors de la construction de l’objet `definePageMeta`, alors que la macro déplace ces métadonnées hors du contexte d’exécution du setup.

```vue annotate="remove:2"
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

<span id="nuxt-no-page-meta-runtime-values-good"></span>

**Bon**

`validate` reçoit un callback ; son accès à `useRoute().params.id` est donc différé jusqu’à l’exécution de ce callback. La règle distingue les corps de fonctions différés des valeurs de métadonnées évaluées immédiatement.

```vue annotate="add:2"
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) · [Toutes les règles](all.md)

### `nuxt/nuxt-config-keys-order`

Préférer l’ordre recommandé des propriétés de configuration Nuxt

[Mauvais](#nuxt-nuxt-config-keys-order-bad) · [Bon](#nuxt-nuxt-config-keys-order-good)

Gravité par défaut: `error`  
Préréglages: `nuxt`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Fichiers de configuration Nuxt (nuxt.config.ts)  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La configuration place `ssr` avant `modules`, inversant leur ordre dans la séquence de clés Nuxt recommandée par la règle.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ ssr: true, modules: [] });
```

<span id="nuxt-nuxt-config-keys-order-good"></span>

**Bon**

Placer `modules` avant `ssr` préserve les deux valeurs tout en respectant l’ordre prescrit ; la correction change la disposition plutôt que le sens des options.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({ modules: [], ssr: true });
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) · [Toutes les règles](all.md)

### `nuxt/prefer-import-meta`

Préférer `import.meta.*` à `process.*`

[Mauvais](#nuxt-prefer-import-meta-bad) · [Bon](#nuxt-prefer-import-meta-good)

Gravité par défaut: `error`  
Préréglages: `nuxt`  
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

**Mauvais**

`process.client` utilise un ancien indicateur d’environnement Nuxt que la règle demande de migrer vers `import.meta`.

```vue annotate="remove:2"
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

<span id="nuxt-prefer-import-meta-good"></span>

**Bon**

`import.meta.client` conserve la branche réservée au navigateur de manière explicite avec l’indicateur d’environnement de remplacement.

```vue annotate="add:2"
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) · [Toutes les règles](all.md)

### `petite-vue/no-unsupported-directive`

Interdire les directives non prises en charge par petite-vue

[Mauvais](#petite-vue-no-unsupported-directive-bad) · [Bon](#petite-vue-no-unsupported-directive-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Documents HTML détectés comme petite-vue ; les SFC Vue ordinaires sont hors du champ de cette règle.  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-memo`, `v-slot:header` et la directive personnalisée `v-my-directive` sont absents de la liste des directives prises en charge par petite-vue. Le script petite-vue désigne ce HTML comme relevant de ce dialecte.

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

**Bon**

Le remplacement utilise les syntaxes prises en charge `v-scope`, `v-effect`, `v-if`, `v-bind` et `v-on` au lieu de dépendre de directives non prises en charge.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) · [Toutes les règles](all.md)

### `petite-vue/valid-v-effect`

Exiger une expression non vide pour v-effect

[Mauvais](#petite-vue-valid-v-effect-bad) · [Bon](#petite-vue-valid-v-effect-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Documents HTML détectés comme petite-vue ; les SFC Vue ordinaires sont hors du champ de cette règle.  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Chaque `v-effect` ne possède aucune expression exécutable : sa valeur est absente, vide ou composée uniquement d’espaces.

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

**Bon**

Les deux valeurs de `v-effect` contiennent une expression : l’une met à jour `el.textContent` et l’autre incrémente `count`. Cette règle vérifie que l’expression n’est pas vide, et non la logique métier de l’effet.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) · [Toutes les règles](all.md)

### `petite-vue/valid-v-scope`

Exiger que v-scope reçoive un objet littéral

[Mauvais](#petite-vue-valid-v-scope-bad) · [Bon](#petite-vue-valid-v-scope-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Documents HTML détectés comme petite-vue ; les SFC Vue ordinaires sont hors du champ de cette règle.  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les quatre valeurs non vides de `v-scope` sont un identifiant, un appel, une expression arithmétique et un nombre ; aucune n’est analysée comme un objet littéral.

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

**Bon**

Un `v-scope` sans valeur utilise la portée racine. Les autres valeurs sont des objets littéraux, y compris l’objet entre parenthèses, que la règle accepte.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) · [Toutes les règles](all.md)

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

### `ssr/no-browser-globals-in-ssr`

Interdire les variables globales propres au navigateur dans un contexte SSR

[Mauvais](#ssr-no-browser-globals-in-ssr-bad) · [Bon](#ssr-no-browser-globals-in-ssr-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Setup lit immédiatement `window.innerWidth`, alors que `window` n’existe pas lorsque le composant s’exécute sur le serveur.

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

<span id="ssr-no-browser-globals-in-ssr-good"></span>

**Bon**

La largeur initiale est une valeur de ref utilisable sur le serveur, et l’accès au navigateur est déplacé dans `onMounted`, qui s’exécute sur le client plutôt que pendant le setup SSR.

```vue annotate="add:2,3,4,5,6"
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L158) · [Toutes les règles](all.md)

### `ssr/no-hydration-mismatch`

Interdire les valeurs non déterministes qui causent des divergences d’hydratation

[Mauvais](#ssr-no-hydration-mismatch-bad) · [Bon](#ssr-no-hydration-mismatch-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template évalue `Math.random()` pendant le rendu ; le serveur et le client peuvent donc produire des textes différents pour le même paragraphe.

```vue annotate="remove:2"
<template>
  <p>{{ Math.random() }}</p>
</template>
```

<span id="ssr-no-hydration-mismatch-good"></span>

**Bon**

Le paragraphe affiche l’état stable `seed` au lieu d’un nouveau résultat aléatoire. Dans cet exemple de style Nuxt, `useState` fournit l’état partagé et la valeur d’initialisation est la constante `"stable"`.

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) · [Toutes les règles](all.md)

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

### `vapor/no-inline-template`

Interdire l’attribut obsolète inline-template

[Mauvais](#vapor-no-inline-template-bad) · [Bon](#vapor-no-inline-template-good)

Gravité par défaut: `error`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

LegacyCard utilise l’attribut inline-template pour le balisage de son enfant.

```vue annotate="remove:2,3"
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

<span id="vapor-no-inline-template-good"></span>

**Bon**

Le balisage est transmis par le slot par défaut au lieu d’un template en ligne.

```vue annotate="add:2,3,4,5"
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) · [Toutes les règles](all.md)

### `vapor/no-vue-lifecycle-events`

Interdire les événements de cycle de vie @vue:xxx par élément (non pris en charge dans Vapor)

[Mauvais](#vapor-no-vue-lifecycle-events-bad) · [Bon](#vapor-no-vue-lifecycle-events-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le champ de saisie utilise l’événement de cycle de vie de template @vue:mounted.

```vue annotate="remove:2"
<template>
  <input @vue:mounted="focusInput" />
</template>
```

<span id="vapor-no-vue-lifecycle-events-good"></span>

**Bon**

onMounted accède à la référence de template nommée et donne le focus au champ de saisie au moyen du hook de cycle de vie de script pris en charge.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) · [Toutes les règles](all.md)

### `vapor/prefer-static-class`

Préférer une classe statique à une liaison de classe dynamique pour les chaînes littérales

[Mauvais](#vapor-prefer-static-class-bad) · [Bon](#vapor-prefer-static-class-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La liaison de classe évalue une chaîne constante alors que la classe ne change pas.

```vue annotate="remove:2"
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

<span id="vapor-prefer-static-class-good"></span>

**Bon**

Un attribut class statique exprime les mêmes classes du panneau sans liaison.

```vue annotate="add:2"
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) · [Toutes les règles](all.md)

### `vapor/require-vapor-attribute`

Suggérer l’ajout de l’attribut vapor à script setup

[Mauvais](#vapor-require-vapor-attribute-bad) · [Bon](#vapor-require-vapor-attribute-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Non implémentée pour le lint des SFC  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Prise en charge actuelle: `no-sfc-finding`

Cette règle est un emplacement réservé doté d’un callback vide. Ajouter vapor sélectionne la compilation Vapor ; le linter actuel ne signale pas cet identifiant du catalogue lorsque vapor est absent.

**ID configuré (aucun diagnostic SFC actuellement)**

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

**Mauvais**

Le bloc script setup ne possède pas l’attribut de compilation Vapor. Il s’agit d’une convention prévue : la fonction de rappel actuellement vide de la règle ne le signale pas.

```vue annotate="remove:1"
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

<span id="vapor-require-vapor-attribute-good"></span>

**Bon**

L’ajout de vapor sélectionne la compilation Vapor. Il illustre la correction prévue et ne signifie pas que le linter actuel émet cette règle du catalogue.

```vue annotate="add:1"
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

Le bon exemple illustre la convention visée ; le traitement actuel des SFC n’émet le diagnostic propre à cette règle pour aucun des deux exemples.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) · [Toutes les règles](all.md)

### `vize:croquis/cf/array-mutation`

Un tableau est modifié par index, ce qu’un tableau réactif ne suit pas.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Uniquement pour l’ancien Vue 2.7 : utilisez des dépendances Vue 2.7 et de compilateur SFC correspondantes pour ce scénario. Les proxies de Vue 3 suivent les affectations aux indices des tableaux ; `items[0] = next` est donc réactif en Vue 3 et ne constitue pas un défaut de Vue 3. Ce code publié n’a actuellement aucun producteur de diagnostics.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Dans ce projet historique Vue 2.7, `items[0] = next` modifie le tableau sans avertir l’observateur de tableaux de Vue 2 ; le premier élément affiché peut donc ne pas être mis à jour.

`replace-first.ts`

```ts annotate="remove:2"
export function replaceFirst(items: string[], next: string): void {
  items[0] = next;
}

```

<span id="vize-croquis-cf-array-mutation-good"></span>

**Bon**

`splice(0, 1, next)` utilise la méthode de modification de tableau observée par Vue 2, permettant au même remplacement de mettre à jour la vue.

`replace-first.ts`

```ts annotate="add:2"
export function replaceFirst(items: string[], next: string): void {
  items.splice(0, 1, next);
}

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/async-boundary`

Un état réactif traverse une frontière asynchrone et peut être observé dans un état périmé.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Une ancienne requête plus lente peut se terminer après une requête plus récente et écraser `result`, car l’observateur n’effectue aucun nettoyage lors de l’invalidation.

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

**Bon**

Enregistrer le nettoyage avant d’attendre : annuler l’ancienne requête et invalider son indicateur `active`, puis affecter uniquement une réponse encore active.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/async-no-suspense`

Un composant asynchrone est rendu sans frontière Suspense.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

Prise en charge actuelle: `no-source-async-fact`

Le producteur de diagnostics de cette frontière lit macros.is_async(), mais l’analyse du code source enregistre actuellement le await de premier niveau dans la portée script-setup à la place. La paire complète de sources Mauvais/Bon ci-dessous ne produit donc aucun diagnostic async-no-suspense via le CLI actuel. Elle explique la convention Suspense ; fournir le fait de macro manquant reste un travail d’implémentation à réaliser.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

L’enfant contient un await au niveau supérieur, mais son parent ne fournit aucune frontière `<Suspense>`. L’analyse actuelle du code source ne fournit pas l’information de macro nécessaire pour émettre ce code de diagnostic.

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

**Bon**

Le parent enveloppe le même enfant asynchrone dans `<Suspense>` avec un contenu de secours pendant le chargement. Cela illustre la convention ; aucune des deux variantes de code source ne produit ce diagnostic dans la passe actuelle.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/browser-api-ssr`

Une API réservée au navigateur est utilisée alors que le composant peut être rendu sur le serveur.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`window.innerWidth` s’exécute pendant setup, alors qu’un environnement SSR ne dispose pas du `window` du navigateur.

`App.vue`

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-browser-api-ssr-good"></span>

**Bon**

Initialiser une ref avec une valeur sûre pour le serveur et lire `window` dans `onMounted`, qui s’exécute après le montage côté client.

`App.vue`

```vue annotate="add:2,3,4"
<script setup lang="ts">
import { onMounted, ref } from "vue";
const width = ref(0);
onMounted(() => { width.value = window.innerWidth; });
</script>
<template><p>Content</p></template>
```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/circular-dep`

Des composants s’importent mutuellement en formant un cycle.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Cela illustre un cycle concret d’initialisation immédiate. Un composant Vue récursif ou un import circulaire n’est pas automatiquement erroné. Aucun producteur actuel n’émet ce code de contrat.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`a.ts` importe `b.ts`, qui importe à son tour `a.ts`. Tous deux initialisent immédiatement une constante à partir de la constante encore non initialisée de l’autre module, provoquant une erreur de zone morte temporelle.

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

**Bon**

Les deux modules lisent des préfixes initialisés dans le module indépendant `labels.ts`, supprimant le cycle et la lecture croisée immédiate.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/circular-reactive-dependency`

Des calculs réactifs dépendent les uns des autres en formant un cycle.

Gravité par défaut: Selon le contexte  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

Qualification de l’exemple: `illustrative-source-pair`

Le projet Vue complet ci-dessous illustre une boucle de rétroaction de mises à jour et sa correction. Il ne constitue pas un témoin qualifié de diagnostic CLI : le producteur de diagnostics exige des identités de références et des arêtes de flux réactif conservées, comme le montre le graphe joint. Ces sources ne prouvent pas que le traitement actuel du code source émettra ce code précis. Les contrôles dédiés de diagnostics sur les graphes à identifiants suivis restent distincts des contrôles de grammaire du code source.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

App possède et fournit count (A). CycleView en dérive nextCount (B), puis réécrit immédiatement chaque valeur dérivée dans le même count injecté. Chaque écriture modifie à nouveau l’entrée du calcul, créant une boucle de mises à jour A → B → A. Les identités du graphe conservé ci-dessous représentent ces deux références, et non des liaisons sans rapport portant les mêmes noms.

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

**Bon**

Supprimer l’observateur qui réécrit B dans A. App conserve la propriété de count et ne le modifie que par son action explicite Increment ; CycleView lit la valeur dérivée nextCount sans réinjecter le résultat. Les mêmes références ne conservent que la dépendance A → B.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/closure-captures-reactive`

Une fermeture capture une valeur réactive et ne verra pas les mises à jour ultérieures.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`makeReader` copie `count.value` avant de créer la fermeture. Le lecteur calculé renvoie ensuite ce nombre initial sans lire de dépendance réactive.

`reader.ts`

```ts annotate="remove:3,4"
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  const captured = count.value;
  return () => captured;
}

```

<span id="vize-croquis-cf-closure-captures-reactive-good"></span>

**Bon**

La fermeture lit `count.value` lorsqu’elle est appelée ; l’accesseur calculé peut donc suivre la ref et mettre à jour `shown` après les incréments.

`reader.ts`

```ts annotate="add:3"
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  return () => count.value;
}

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/composable-outside-setup`

Un composable est appelé en dehors de `setup`.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Le problème concerne ce composable dépendant du cycle de vie ; il ne s’agit pas d’une interdiction générale des fonctions utilitaires ordinaires ou de tous les appels à la Composition API en dehors du setup. Ce contrat n’a actuellement aucun producteur de diagnostics.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

L’import de `use-title.ts` enregistre `onMounted` avant qu’un setup de composant soit actif. Appeler sa fonction exportée plus tard ne fait que renvoyer cette ref au niveau du module ; cela ne peut pas rétablir l’appartenance manquée au cycle de vie.

`use-title.ts`

```ts annotate="remove:2,3,4"
import { onMounted, ref } from 'vue';
const title = ref('Before mount');
onMounted(() => { title.value = 'Mounted'; });
export function useTitle() { return title; }

```

<span id="vize-croquis-cf-composable-outside-setup-good"></span>

**Bon**

La création de l’état et l’enregistrement du hook sont tous deux déplacés dans `useTitle`, qu’App appelle de manière synchrone dans setup. Le hook de montage appartient désormais à cette instance d’App.

`use-title.ts`

```ts annotate="add:2,3,4,5,6"
import { onMounted, ref } from 'vue';
export function useTitle() {
  const title = ref('Before mount');
  onMounted(() => { title.value = 'Mounted'; });
  return title;
}

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/computed-side-effects`

L’accesseur d’une propriété calculée écrit dans l’état ou produit un autre effet de bord.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

L’évaluation de `doubled` écrit dans `lastCalculated` ; lire une valeur calculée modifie donc aussi un état distinct. Cela lie l’effet de bord au moment où l’accesseur paresseux est lu.

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

**Bon**

L’accesseur renvoie uniquement le nombre dérivé. Un observateur distinct se charge d’écrire dans `lastCalculated` lorsque `count` change, y compris pour sa valeur initiale.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/deep-import`

Une chaîne d’imports est plus profonde que ce que le projet autorise.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Il s’agit d’une politique d’organisation du projet choisie explicitement ; elle n’invente ni seuil de profondeur ni option pris en charge. Aucun producteur de diagnostics actuel n’existe pour ce contrat.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le point d’entrée fait transiter une valeur simple par `level-one`, `level-two` et `level-three`, créant une chaîne d’imports inutilement profonde pour un projet qui souhaite une interface publique peu profonde.

`entry.ts`

```ts annotate="remove:1"
export { label } from './level-one';

```

<span id="vize-croquis-cf-deep-import-good"></span>

**Bon**

Le point d’entrée utilise `public-api.ts`, qui réexporte directement la valeur. Le consommateur conserve le même nom importé tandis que la chaîne se raccourcit.

`entry.ts`

```ts annotate="add:1"
export { label } from './public-api';

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/destructuring-breaks-reactivity`

La déstructuration d’un objet réactif copie ses champs et supprime leur suivi.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

La déstructuration ordinaire de l’objet `props` copie sa valeur actuelle de `item` ; elle est distincte de la déstructuration directe de `defineProps()` dans Vue 3.5.

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

**Bon**

`toRef(props, "item")` conserve le lien avec la propriété de `props`.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/di-outside-setup`

`provide` ou `inject` est appelé en dehors de `setup`.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Cet exemple utilise provide/inject dans les composants. `app.provide` et l’injection prise en charge avec `app.runWithContext` sont d’autres contextes de propriété valides, que ce scénario n’interdit pas. Aucun producteur actuel n’émet ce code de contrat.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`main.ts` appelle le `provide` de composant sans instance de composant active. Le `inject` de l’enfant ne peut donc pas recevoir cette valeur prévue de l’ancêtre et utilise `light`.

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

**Bon**

App appelle le fournisseur depuis son setup avant de rendre l’enfant. L’enfant hérite désormais de la valeur `dark` de son ancêtre composant.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/dom-access-without-next-tick`

Le DOM est lu avant que Vue ait appliqué la mise à jour.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le gestionnaire de clic incrémente `count` et lit immédiatement le paragraphe rendu, avant que Vue applique la mise à jour du DOM planifiée. `sampled` peut contenir le décompte précédent.

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

**Bon**

Attendre `nextTick()` après l’écriture de l’état permet à Vue de mettre à jour le paragraphe avant que `readLabel` relève son texte.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/duplicate-id`

Le même identifiant d’élément est utilisé dans plusieurs composants.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Les composants de livraison et de facturation accessibles rendent tous deux `id="postal-code"` ; leurs libellés partagent donc une cible ambiguë dans le document.

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

**Bon**

Chaque composant appelle `useId()` et lie sa propre valeur au libellé et au champ de saisie, conservant leur association sans répéter un identifiant littéral.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/element_id.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/event-listener-leak`

Un écouteur d’événement est enregistré sans jamais être supprimé.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le montage ajoute un écouteur de redimensionnement de window qui capture la ref de largeur du composant, mais le démontage ne le supprime jamais. Des montages répétés peuvent conserver des écouteurs et un état inutilisés.

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

**Bon**

`onUnmounted` supprime exactement la même fonction `resize` que celle enregistrée au montage, mettant fin à la durée de vie de l’écouteur externe de cette instance.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/event-modifier`

Un écouteur d’événement utilise un modificateur que l’événement émis ne prend pas en charge.

Gravité par défaut: info  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`.stop` suppose que l’événement personnalisé `save` de l’enfant possède la méthode de propagation d’un événement natif, alors que son argument n’est pas nécessairement un DOM Event.

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

**Bon**

Supprimer `.stop` de l’écouteur d’événement personnalisé ; gérer la propagation native au niveau de l’écouteur DOM réel lorsque nécessaire.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/hydration-risk`

Ce code regroupe plusieurs diagnostics de réactivité, dont une prop copiée dans une ref. Il ne signifie pas que chaque expression Date.now() est détectée par la passe inter-fichiers.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

L’enfant initialise `ref(props.count)` une seule fois ; son décompte local ne suit donc plus les changements ultérieurs de la prop du parent. Il s’agit du producteur actuel pour la copie d’une prop dans une ref, et non d’un exemple général de SSR non déterministe.

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

**Bon**

`toRef(props, "count")` pointe vers la prop au lieu de copier sa valeur initiale dans un état indépendant.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/inherit-attrs-unused`

`inheritAttrs: false` est défini et le composant ne lit jamais les attributs.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

L’enfant définit `inheritAttrs: false`, mais ne transmet jamais l’attribut `class="notice"` du parent.

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

**Bon**

Conserver le contrôle explicite de l’héritage et lier `$attrs` à la cible `<main>` prévue.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/inject-without-symbol`

`inject` utilise une clé ordinaire au lieu d’un symbole `InjectionKey`.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le consommateur injecte la clé chaîne non typée `"theme"`, qui ne fournit aucune identité de symbole partagée avec le fournisseur.

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

**Bon**

Le consommateur et le fournisseur importent le même `ThemeKey` au lieu de dupliquer des noms sous forme de chaînes.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/keys.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/injected-async-mutation-race`

Une valeur injectée est modifiée par une tâche asynchrone susceptible de provoquer une condition de concurrence.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`CountLoader.vue` écrit directement le résultat obtenu après attente dans le store injecté partagé avec `CountSummary.vue`, permettant à un travail périmé d’affecter les deux consommateurs.

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

**Bon**

Le chargeur annule le travail invalidé et n’émet qu’un résultat actif. Le fournisseur se charge de modifier le store via `applyLoadedCount`.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/lifecycle-outside-setup`

Un hook de cycle de vie est enregistré en dehors de `setup`.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le point d’entrée appelle `installTitle()` avant de monter une application ; `onMounted` est donc enregistré sans contexte setup de composant actif.

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

**Bon**

Appeler le même utilitaire de manière synchrone depuis le setup d’App associe la fonction de rappel de cycle de vie au montage de cette instance.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/lifecycle-without-cleanup`

Un hook de cycle de vie lance un travail sans jamais le nettoyer.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le montage enregistre un écouteur de redimensionnement de window, mais le démontage ne supprime jamais cette même fonction de rappel.

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

**Bon**

`onUnmounted` supprime l’écouteur avec le même nom d’événement et la même identité de fonction que ceux utilisés par `addEventListener`.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/missing-required-prop`

Une prop obligatoire n’est pas transmise.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le parent rend `<Child />` sans la prop obligatoire `title: string` de l’enfant.

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

**Bon**

`title="Hello"` fournit la prop obligatoire déclarée par l’enfant résolu.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/missing-suspense`

Une dépendance asynchrone est utilisée en dehors d’une frontière Suspense.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`AsyncCard` contient un await au niveau supérieur, ce qui rend son setup asynchrone, mais App le rend sans frontière Suspense pour coordonner cette dépendance.

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

**Bon**

App enveloppe l’enfant asynchrone dans `Suspense` et fournit un contenu de secours pendant le chargement, jusqu’à ce que le setup de l’enfant soit terminé.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/module-scope-reactive`

Un état réactif est créé au niveau du module et partagé par tous les appelants.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Un état réactif à la portée du module est autorisé pour des stores applicatifs conçus à cet effet. Cet exemple suppose une isolation par composant ou requête ; le contrat publié n’a actuellement aucun producteur de diagnostics.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le module initialise `count` une seule fois et les deux instances de Counter reçoivent la même ref. Cliquer sur l’une modifie les deux compteurs, alors que cet exemple prévoit un état indépendant pour chaque instance.

`counter.ts`

```ts annotate="remove:2,3"
import { ref } from 'vue';
const count = ref(0);
export function createCounter() { return { count }; }

```

<span id="vize-croquis-cf-module-scope-reactive-good"></span>

**Bon**

Créer la ref dans `createCounter` donne à chaque appel synchrone de setup un objet d’état distinct ; chaque bouton possède donc son propre compteur.

`counter.ts`

```ts annotate="add:2,3,4,5"
import { ref } from 'vue';
export function createCounter() {
  const count = ref(0);
  return { count };
}

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/multi-root-attrs`

Un composant à plusieurs racines reçoit des attributs sans avoir d’endroit où les placer.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

L’enfant a pour racines `<main>` et `<aside>` ; Vue ne dispose donc pas d’une racine unique pouvant recevoir automatiquement la classe du parent.

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

**Bon**

Transmettre explicitement `$attrs` à `<main>` tout en conservant la seconde racine.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/mutated-after-escape`

Un objet réactif est modifié après avoir échappé à son propriétaire.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Il s’agit d’une politique explicite de propriété d’un historique immuable, et non d’une interdiction générale de transmettre des objets réactifs ou de les modifier ensuite. Aucun producteur actuel n’émet ce contrat.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

L’archive conserve le même objet que celui passé à `publish`. Le propriétaire modifie ensuite le nom de cet objet, remplaçant rétroactivement par Grace le nom dans l’enregistrement censé être historique. Le paramètre Readonly de TypeScript ne copie pas l’objet.

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

**Bon**

Publier une copie ordinaire sépare l’enregistrement archivé d’Ada des modifications ultérieures du profil réactif. La politique d’instantanés de l’archive est désormais respectée.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/non-reactive-provide`

Une valeur fournie n’est pas réactive ; les descendants ne verront donc pas les mises à jour.

Gravité par défaut: Selon le contexte  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`ThemeProvider.vue` fournit un objet ordinaire. Modifier les champs de cet objet ne donne pas de dépendance réactive Vue au consommateur qui l’injecte.

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

**Bon**

Le fournisseur enveloppe le thème dans `ref` ; la même référence injectée peut suivre les changements ultérieurs.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/non-unique-id`

L’identifiant d’un élément dans une boucle n’est pas unique pour chaque entrée.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Chaque itération de `v-for` rend le même identifiant littéral `result-title` ; la clé de la boucle ne rend pas les identifiants DOM uniques.

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

**Bon**

L’identifiant du titre inclut l’identifiant stable du résultat, produisant un identifiant distinct dans le document pour chaque entrée.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/element_id.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/object-identity-comparison`

Un objet réactif est comparé par identité, laquelle change lorsque l’enveloppe est retirée.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

L’exemple suppose que les identifiants distinguent les enregistrements de façon unique. Comparer deux références au même proxy réactif reste valide ; ce contrat n’a actuellement aucun producteur de diagnostics.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`proxy === raw` compare l’identité de l’enveloppe ; le résultat est donc faux même si les deux représentent le même enregistrement utilisateur. L’application visait l’identité de l’enregistrement, et non celle de l’enveloppe de l’objet.

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

**Bon**

Comparer le `id` stable de l’enregistrement répond à la question voulue sans dépendre du fait que l’objet soit brut ou enveloppé dans un proxy.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/pinia-getter`

Un getter Pinia est lu sans `storeToRefs` ; il ne restera donc pas réactif.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Pinia doit être installé, et main.ts installe son plugin avant le montage. Lire directement `store.doubled` dans un calcul suivi ou un template est valide ; le défaut consiste ici à prendre un simple instantané. Ce contrat n’a actuellement aucun producteur de diagnostics.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`const doubled = store.doubled` copie le nombre actuel du getter pendant setup. Le nombre copié ne suit pas les mises à jour ultérieures de `store.count`.

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

**Bon**

`storeToRefs(store)` fournit une ref de getter réactive qui peut être déstructurée et déballée par le template tout en restant liée au store.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/prop-type-mismatch`

La valeur d’une prop transmise ne correspond pas au type déclaré.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le parent passe l’expression numérique `42` à la prop `title: string` de l’enfant résolu.

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

**Bon**

Le littéral `title="Hello"` fournit une chaîne correspondant à la déclaration de l’enfant.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/provide-inject-type`

Une valeur fournie et son injection n’ont pas le même type.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

Ce contrôle compare les annotations de type explicites du fournisseur et du consommateur, et non les types de valeurs littérales inférés. Conservez l’annotation `as string` du fournisseur dans cet exemple.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le fournisseur annote explicitement `title` avec le type `string`, tandis que le descendant demande `inject<number>` pour la même clé.

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

**Bon**

L’appel explicite `inject<string>` du consommateur correspond à l’annotation du fournisseur. Conserver `as string` : ce producteur compare les annotations explicites, et non les types littéraux inférés.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis/diagnostics.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/provide-without-symbol`

`provide` utilise une clé ordinaire au lieu d’un symbole `InjectionKey`.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Les deux composants utilisent la chaîne `"theme"` ; des fonctionnalités sans rapport peuvent accidentellement réutiliser cette clé.

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

**Bon**

Exporter un seul symbole typé `ThemeKey` et importer cette même valeur aux endroits où provide et inject sont appelés. Créer des symboles distincts avec la même description ne les relierait pas.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/keys.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/reactive-export`

Un état réactif est exporté depuis le module.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Les stores applicatifs volontairement partagés peuvent exporter un état réactif. Ce scénario exige un état isolé et ne prétend pas que tout export réactif est invalide. Aucun producteur actuel n’émet ce contrat.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le module exporte un seul objet réactif initialisé ; chaque module qui l’importe reçoit donc le même décompte. Dans un module SSR partagé entre les requêtes, cela compromet l’isolation de l’état par instance ou par requête prévue par l’exemple.

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

**Bon**

Le module exporte une fabrique et App l’appelle dans setup. Chaque instance obtient un nouveau décompte réactif au lieu du singleton exporté.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/reactivity-outside-setup`

Une API réactive est appelée en dehors de `setup`.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Vue autorise ref/reactive/computed en dehors du setup des composants. Le risque concerne ici une propriété ou un partage indésirable dans le cadre d’une politique explicite d’isolation des instances, et non un usage interdit de l’API. Ce contrat n’a actuellement aucun producteur de diagnostics.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Les deux API réactives s’exécutent pendant le chargement du module. Les deux instances de Counter partagent donc une seule ref et une seule valeur calculée, malgré l’intention d’avoir des compteurs indépendants.

`use-counter.ts`

```ts annotate="remove:2,3,4"
import { computed, ref } from 'vue';
const count = ref(0);
const doubled = computed(() => count.value * 2);
export function useCounter() { return { count, doubled }; }

```

<span id="vize-croquis-cf-reactivity-outside-setup-good"></span>

**Bon**

`useCounter` crée la ref et la valeur calculée de manière synchrone dans chaque appel de setup d’un composant, donnant à chaque widget son propre état et sa propre dérivation suivie.

`use-counter.ts`

```ts annotate="add:2,3,4,5,6"
import { computed, ref } from 'vue';
export function useCounter() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/reassignment-breaks-reactivity`

Réaffecter une liaison réactive la remplace par une valeur ordinaire.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

L’enfant crée une ref de prop, puis écrase la variable avec `props.user`, supprimant le lien fourni par cette ref.

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

**Bon**

Conserver le `toRef` dans une liaison `const` et supprimer la réaffectation qui le remplace.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/reference-escapes-scope`

Une référence réactive échappe à la portée qui gère sa durée de vie.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Les refs peuvent légitimement être renvoyées par des composables ou partagées entre des portées. Cet exemple exige explicitement un cache d’instantanés ; il ne prétend pas que le démontage invalide une ref. Aucun producteur actuel n’émet ce contrat.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le cache au niveau du processus conserve la ref active du décompte du composant. Il peut maintenir l’état de cette instance accessible après le démontage et observer des modifications ultérieures, alors que ce cache est censé stocker un instantané.

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

**Bon**

Le cache reçoit le nombre ordinaire actuel ; il conserve donc un instantané sans retenir la ref appartenant au composant.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/setup-context-violation`

Le contexte setup est utilisé d’une manière que Vue n’autorise pas.

Gravité par défaut: Selon le contexte  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`ref(0)` est créé au niveau du module d’un script normal, en dehors du contexte setup propre à chaque instance représenté par ce scénario d’analyse.

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

**Bon**

Déplacer la liaison dans script setup, où chaque instance de composant possède son décompte et où le template peut le lire.

`App.vue`

```vue annotate="add:1,5"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
</script>
<template><p>{{ count }}</p></template>
```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/shallow-deep-access`

Une propriété profonde d’une valeur `shallowReactive` ou `shallowRef` est lue comme si elle était suivie.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`shallowReactive` suit la propriété racine `user`, mais laisse l’objet imbriqué brut. Modifier `profile.user.name` n’avertit pas le template comme le ferait une modification profonde suivie.

`profile.ts`

```ts annotate="remove:1,2"
import { shallowReactive } from 'vue';
export function makeProfile() { return shallowReactive({ user: { name: 'Ada' } }); }

```

<span id="vize-croquis-cf-shallow-deep-access-good"></span>

**Bon**

La réactivité profonde de `reactive` enveloppe l’objet utilisateur imbriqué ; la même affectation du nom peut donc déclencher la mise à jour du nom affiché.

`profile.ts`

```ts annotate="add:1,2"
import { reactive } from 'vue';
export function makeProfile() { return reactive({ user: { name: 'Ada' } }); }

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/spread-breaks-reactivity`

Décomposer un objet réactif avec l’opérateur spread copie ses valeurs et supprime leur suivi.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`UserSummary.vue` décompose `props.user` dans un nouvel objet avec l’opérateur spread, prenant un instantané des données réactives reçues.

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

**Bon**

`toRef(props, "user")` conserve une référence vers la prop reçue au lieu de copier ses champs.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/suspense-no-fallback`

`<Suspense>` n’a aucun contenu de secours.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Suspense sans contenu de repli est une syntaxe Vue valide. Il s’agit d’une convention choisie pour l’interface de chargement, et non d’une erreur de compilation ; le contrat publié n’a actuellement aucun producteur de diagnostics.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

La frontière Suspense a un enfant asynchrone, mais aucun contenu de secours ; cet exemple n’affiche donc aucun contenu de chargement pendant l’attente.

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

**Bon**

Le slot `#fallback` fournit un paragraphe de chargement explicite jusqu’à ce que l’enfant asynchrone soit prêt.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/template-ref-timing`

Une référence de template est lue avant le montage du composant.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Setup lit la référence de template avant le montage, alors que sa valeur est encore null. L’appel optionnel de focus ne donne donc le focus à aucun élément.

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

**Bon**

`onMounted` diffère la lecture jusqu’à ce que Vue ait affecté l’élément de saisie à la référence de template, permettant à l’utilitaire de focus d’agir dessus.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/toraw-mutation`

`toRaw` est utilisé, puis l’objet brut est modifié.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`rename` obtient la cible brute et écrit dans `raw.name`, contournant le setter du proxy qui déclencherait la mise à jour du nom réactif affiché.

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

**Bon**

Écrire dans `profile.name` au moyen du proxy réactif transmis conserve le même renommage tout en notifiant ce qui en dépend.

`profile.ts`

```ts annotate="add:1,4"
import { reactive } from 'vue';
export function makeProfile() { return reactive({ name: 'Ada' }); }
export function rename(profile: { name: string }): void {
  profile.name = 'Grace';
}

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/uncaught-error`

Un composant peut lever une exception sans qu’aucune frontière d’erreur ne la capture.

Gravité par défaut: info  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

Le producteur actuel analyse les expressions de template telles que JSON.parse(input). Il ne signale pas une instruction throw présente uniquement dans le bloc script.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le template de l’enfant appelle `JSON.parse` sur une entrée mal formée et le parent accessible n’a aucune frontière de capture d’erreur.

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

**Bon**

Le parent enregistre `onErrorCaptured` autour de cet enfant. Renvoyer `false` arrête la propagation ; une frontière utilisée en production doit aussi présenter une interface de récupération utile.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/undeclared-emit`

Le composant émet un événement qui n’est pas déclaré.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

L’enfant appelle `emit("save")`, mais son contrat `defineEmits` ne déclare que `cancel`.

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

**Bon**

Déclarer `save` avec son tuple d’arguments vide pour que l’événement émis corresponde au contrat du composant.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/undeclared-prop`

Un parent transmet une prop que l’enfant ne déclare pas.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le parent transmet `typo` alors que l’enfant résolu ne déclare que `title`. Cette convention de l’analyseur est distincte du comportement général de transmission automatique des attributs de Vue.

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

**Bon**

Supprimer la liaison involontaire `typo` et conserver la prop déclarée `title`.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/undefined-slot`

Un parent remplit un slot que l’enfant n’expose pas.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

App fournit un slot `footer`, mais Card ne déclare et ne rend que `header`. Le contenu Notice fourni ne dispose d’aucun emplacement de slot correspondant dans cet enfant.

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

**Bon**

App fournit `header`, correspondant à la fois à la déclaration de slot typée de l’enfant et à son emplacement rendu ; Notice apparaît donc à cet endroit.

`App.vue`

```vue annotate="add:6"
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #header>Notice</template></Card>
</template>

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/unhandled-event`

Un enfant émet un événement qu’aucun parent ne gère.

Gravité par défaut: info  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`Child.vue` émet `save`, mais le composant qui l’enveloppe directement ne l’écoute pas ; les événements de composants ne remontent pas automatiquement à travers les composants intermédiaires.

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

**Bon**

`Wrapper.vue` associe un écouteur `save` à son enfant direct. La fonction de rappel vide illustre la gestion de l’événement pour cette règle, et non une implémentation complète de l’enregistrement.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/unmatched-inject`

`inject` nomme une clé qu’aucun ancêtre ne fournit.

Gravité par défaut: error / warning (avec valeur par défaut)  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`ThemeLabel.vue` injecte `ThemeKey`, mais son ancêtre accessible `App.vue` ne fournit jamais cette clé.

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

**Bon**

`App.vue` fournit un thème réactif au moyen du même `ThemeKey` exporté, avant de rendre le descendant qui l’injecte.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis/diagnostics.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/unmatched-listener`

Un parent écoute un événement que l’enfant n’émet pas.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le parent écoute `save`, tandis que l’enfant résolu ne déclare que `cancel`.

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

**Bon**

L’enfant déclare et émet `save`, correspondant au nom de l’écouteur du parent.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/unregistered-component`

Un template utilise un composant qui n’est ni enregistré ni importé.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Un fichier `Child.vue` existe, mais le parent n’importe pas `Child` et ne l’enregistre pas non plus d’une autre manière pour son template.

`App.vue`

```vue
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

<span id="vize-croquis-cf-unregistered-component-good"></span>

**Bon**

Importer `Child` dans le script setup du parent pour que le template résolve la liaison du composant.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/component_resolution.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/unresolved-import`

Un import ne se résout pas vers un module.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le parent importe `./Missing.vue`, mais le projet contient `Child.vue` plutôt que ce chemin.

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

**Bon**

Faire pointer l’import vers le fichier existant `./Child.vue`, en conservant la même liaison dans le template.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/component_resolution.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/unused-attrs`

Des attributs à transmettre automatiquement sont passés à un composant à plusieurs racines qui ne les utilise pas.

Gravité par défaut: info  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le `tracking-code` du parent n’est ni consommé comme prop ni transmis par l’enfant à plusieurs racines.

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

**Bon**

Lier `$attrs` à `<main>` donne une destination explicite à cet attribut transmis automatiquement.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/unused-emit`

Un événement déclaré n’est jamais émis.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

L’enfant déclare `save`, mais n’appelle jamais la fonction d’émission d’événements avec ce nom.

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

**Bon**

L’exemple appelle `emit("save")`, utilisant ainsi l’événement déclaré. Les interactions réelles doivent l’émettre lorsque l’action correspondante se produit.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/unused-provide`

Une clé fournie n’est jamais injectée.

Gravité par défaut: warning  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

`App.vue` fournit `ThemeKey`, mais son sous-arbre rendu `Dashboard.vue` ne contient aucun consommateur de cette clé.

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

**Bon**

Le tableau de bord rend désormais `ThemeLabel.vue`, qui injecte exactement la même identité `ThemeKey` que celle de l’ancêtre.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/value-extraction-breaks-reactivity`

Extraire une valeur réactive dans une variable locale fait perdre les mises à jour ultérieures.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le `item` déstructuré réactif de Vue 3.5 est lu une seule fois dans `itemSnapshot` ; le remplacement ultérieur de la prop ne met pas à jour cet instantané.

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

**Bon**

Lire `item` dans `computed` pour que la transformation de déstructuration réactive des props de Vue puisse suivre chaque évaluation.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/watch-can-be-computed`

Un observateur ne fait que copier une valeur dans l’état et peut être remplacé par une propriété calculée.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Cela illustre la préférence publiée pour un état purement dérivé. Les watchers restent appropriés pour les effets externes ou les états modifiables indépendamment ; aucun producteur actuel n’émet ce contrat.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

L’observateur ne produit aucun effet externe ; il maintient uniquement une seconde ref modifiable synchronisée avec le double de `count`. Cet exemple n’effectue aucune écriture indépendante dans cette valeur dérivée.

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

**Bon**

L’accesseur d’une propriété calculée exprime directement la même dérivation et supprime la synchronisation manuelle ainsi que l’état modifiable supplémentaire.

`use-double.ts`

```ts annotate="add:1,4"
import { computed, ref } from 'vue';
export function useDouble() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/watcheffect-async`

`watchEffect` lance une tâche asynchrone et ne peut pas nettoyer l’exécution précédente.

Gravité par défaut: error  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le `watchEffect` asynchrone mêle la collecte implicite des dépendances à une requête dont il attend le résultat, sans garde contre l’invalidation.

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

**Bon**

Un `watch(() => props.query, ...)` explicite déclare la source, enregistre le nettoyage de la requête et refuse une réponse périmée après invalidation.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Producteur de diagnostics](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[Index des règles inter-fichiers](cross-file.md)

### `vize:croquis/cf/watcher-outside-setup`

`watch` ou `watchEffect` est appelé en dehors de `setup`.

Gravité par défaut: Non émis  
Champ d’application: Graphe de composants analysé et faits pris en charge décrits ci-dessous  
Correction automatique: Aucune ; examinez les fichiers concernés et appliquez la correction  
Options: Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité

Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.

Les watchers à la portée du module sont valides lorsque leur propriétaire conserve et appelle une fonction d’arrêt, ou leur donne volontairement la durée de vie de l’application. Cet exemple exige des durées de vie appartenant au composant ; le contrat n’a actuellement aucun producteur de diagnostics.

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

L’observateur est créé au chargement du module, en dehors du setup de chacune des deux instances d’Observer, et les deux instances partagent ses refs. Il n’est pas automatiquement arrêté lorsqu’une instance donnée d’Observer est démontée.

`use-observer.ts`

```ts annotate="remove:2,3,4,5"
import { ref, watch } from 'vue';
const count = ref(0);
const observed = ref(0);
watch(count, next => { observed.value = next; });
export function useObserver() { return { count, observed }; }

```

<span id="vize-croquis-cf-watcher-outside-setup-good"></span>

**Bon**

Chaque appel synchrone de setup crée ses propres refs et son propre observateur dans `useObserver`. Vue associe cet observateur à la durée de vie du composant appelant.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Explication publique](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[Index des règles inter-fichiers](cross-file.md)

### `vue/a11y-img-alt`

Exiger un attribut alt sur les images pour les rendre accessibles

[Mauvais](#vue-a11y-img-alt-bad) · [Bon](#vue-a11y-img-alt-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Ni l’image statique ni l’image dont la source est dynamique ne fournit d’attribut alt.

```vue annotate="remove:2,3"
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

<span id="vue-a11y-img-alt-good"></span>

**Bon**

Les images informatives reçoivent un texte alt descriptif, les images décoratives un alt vide, et l’image dynamique lie sa description.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33) · [Toutes les règles](all.md)

### `vue/attribute-hyphenation`

Imposer un style de nommage des attributs sur les composants personnalisés

[Mauvais](#vue-attribute-hyphenation-bad) · [Bon](#vue-attribute-hyphenation-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

L’attribut du composant utilise la graphie camelCase firstName.

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

<span id="vue-attribute-hyphenation-good"></span>

**Bon**

La graphie first-name respecte la convention configurée qui sépare les mots des attributs de composants par des tirets.

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) · [Toutes les règles](all.md)

### `vue/attribute-order`

Imposer un ordre cohérent des attributs

[Mauvais](#vue-attribute-order-bad) · [Bon](#vue-attribute-order-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le gestionnaire d’événement apparaît avant la directive structurelle v-if et l’attribut ordinaire id.

```vue annotate="remove:2"
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

<span id="vue-attribute-order-good"></span>

**Bon**

v-if vient en premier, suivi de id puis du gestionnaire d’événement, conformément à l’ordre défini par la règle.

```vue annotate="add:2"
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [Toutes les règles](all.md)

### `vue/component-definition-name-casing`

Imposer PascalCase ou kebab-case aux noms de définition des composants

[Mauvais](#vue-component-definition-name-casing-bad) · [Bon](#vue-component-definition-name-casing-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Le nom de fichier du composant est contrôlé. PascalCase et kebab-case sont acceptés ; une casse mixte est signalée.

**Configuration (Vite+)**

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

**Mauvais**

Le nom de fichier myComponent.vue combine une initiale minuscule et une majuscule interne au lieu d’utiliser PascalCase ou kebab-case.

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

<span id="vue-component-definition-name-casing-good"></span>

**Bon**

Renommer le fichier en MyComponent.vue adopte PascalCase ; le contenu de son template reste inchangé.

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) · [Toutes les règles](all.md)

### `vue/component-name-in-template-casing`

Imposer une casse précise aux noms de composants dans les templates

[Mauvais](#vue-component-name-in-template-casing-bad) · [Bon](#vue-component-name-in-template-casing-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

Le composant est écrit en kebab-case et en camelCase alors que la convention impose PascalCase.

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

**Bon**

MyComponent utilise PascalCase ; la syntaxe native slot reste en minuscules.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31) · [Toutes les règles](all.md)

### `vue/cross-file-attrs-fallthrough`

Un parent transmet des attributs à un enfant résolu dont la racine ne peut pas en hériter et qui n’utilise pas explicitement $attrs.

Gravité par défaut: warning  
Champ d’application: Déclarations du projet accessibles et composants importés  
Options: crossFile ; gravité de la règle (off/warn/error)  
Correction automatique: Aucune

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

**Fichiers communs du projet**

Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.

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

**Mauvais**

Le parent transmet `class="notice"` à un enfant résolu constitué d’un fragment, qui n’a aucune cible automatique pour les attributs et ne lit jamais `$attrs`.

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

**Bon**

L’enfant choisit `<main>` comme cible en y liant `$attrs` ; son frère `<aside>` reste distinct.

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

Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.

[Index des règles inter-fichiers](cross-file.md)

### `vue/html-button-has-type`

Exiger un type explicite et valide sur les éléments button

[Mauvais](#vue-html-button-has-type-bad) · [Bon](#vue-html-button-has-type-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un bouton omet type et un autre fournit le type foo, qui n’est pas pris en charge.

```vue annotate="remove:2,3"
<template>
<button>Click</button>
<button type="foo">Click</button>
</template>
```

<span id="vue-html-button-has-type-good"></span>

**Bon**

Les boutons précisent button, submit ou reset ; un type lié est considéré comme dynamique.

```vue annotate="add:2,3,4,5"
<template>
<button type="button">Click</button>
<button type="submit">Save</button>
<button type="reset">Reset</button>
<button :type="dynamicType">Click</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39) · [Toutes les règles](all.md)

### `vue/html-quotes`

Imposer un style de guillemets pour les attributs HTML

[Mauvais](#vue-html-quotes-bad) · [Bon](#vue-html-quotes-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les attributs utilisent des apostrophes ou aucun guillemet au lieu des guillemets doubles exigés par la convention.

```vue annotate="remove:2,3,4"
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

<span id="vue-html-quotes-good"></span>

**Bon**

Les attributs ordinaires et les expressions des directives utilisent des guillemets doubles.

```vue annotate="add:2,3"
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) · [Toutes les règles](all.md)

### `vue/html-self-closing`

Imposer un style de fermeture automatique des balises

[Mauvais](#vue-html-self-closing-bad) · [Bon](#vue-html-self-closing-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

Le composant vide utilise une paire de balises, tandis que les éléments vides img et br omettent la syntaxe autofermante configurée.

```vue annotate="remove:2,3,4"
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

<span id="vue-html-self-closing-good"></span>

**Bon**

Le composant et les éléments vides utilisent une syntaxe autofermante ; un div contenant du contenu conserve sa balise de fermeture.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) · [Toutes les règles](all.md)

### `vue/max-template-complexity`

Limiter la complexité propre du template d’un composant, cyclomatique et cognitive

[Mauvais](#vue-max-template-complexity-bad) · [Bon](#vue-max-template-complexity-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

L’exemple Mauvais a une complexité cyclomatique de 13 et une complexité cognitive de 25 (limites : 11 et 16). Chaque composant est mesuré séparément ; seuls les templates HTML intégrés sont pris en charge.

Consultez [le calcul de la complexité et les limites des composants](../guide/cross-file-complexity.md) pour connaître les contributions aux deux scores de l’exemple.

**Configuration (Vite+)**

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

**Mauvais**

Les branches, la boucle, le contenu des slots et les décisions dans les expressions écrits dans le parent produisent des scores de 13 et 25, supérieurs aux limites par défaut de 11 et 16.

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

**Bon**

Le template parent délègue le rendu à RowList et conserve un seul v-if ; ses propres scores sont de 2 et 1.

```vue annotate="add:2"
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) · [Toutes les règles](all.md)

### `vue/multi-word-component-names`

Exiger des noms de composants composés de plusieurs mots

[Mauvais](#vue-multi-word-component-names-bad) · [Bon](#vue-multi-word-component-names-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Le diagnostic concerne le nom de fichier. Renommez ce même composant ; modifier une balise enfant ne corrige pas le problème.

**Configuration (Vite+)**

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

**Mauvais**

Item.vue donne au composant un nom composé d’un seul mot.

`Item.vue`

```vue
<template><p>Item</p></template>
```

<span id="vue-multi-word-component-names-good"></span>

**Bon**

TodoItem.vue donne au même template un nom de composant composé de plusieurs mots.

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) · [Toutes les règles](all.md)

### `vue/mustache-interpolation-spacing`

Imposer un espacement cohérent dans les interpolations à doubles accolades

[Mauvais](#vue-mustache-interpolation-spacing-bad) · [Bon](#vue-mustache-interpolation-spacing-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Il manque un espace à une ou aux deux limites des délimiteurs de l’interpolation de texte.

```vue annotate="remove:2,3,4"
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

<span id="vue-mustache-interpolation-spacing-good"></span>

**Bon**

Des espaces séparent l’expression des délimiteurs d’ouverture et de fermeture à doubles accolades.

```vue annotate="add:2,3,4"
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) · [Toutes les règles](all.md)

### `vue/no-array-index-key`

Interdire l’utilisation directe de la variable d’index de v-for comme :key

[Mauvais](#vue-no-array-index-key-bad) · [Bon](#vue-no-array-index-key-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La clé de la liste est son index actuel ; l’identité de l’élément change donc lorsque la liste est réordonnée.

```vue annotate="remove:2"
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

<span id="vue-no-array-index-key-good"></span>

**Bon**

La clé provient de item.id, ce qui préserve l’identité de chaque élément lorsqu’il change de position.

```vue annotate="add:2"
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) · [Toutes les règles](all.md)

### `vue/no-bare-strings-in-template`

Interdire le texte brut destiné aux utilisateurs dans les templates lorsqu’il devrait être internationalisé

[Mauvais](#vue-no-bare-strings-in-template-bad) · [Bon](#vue-no-bare-strings-in-template-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le texte visible et les attributs de nommage contiennent des chaînes non traduites directement dans le template.

```vue annotate="remove:2,3,4,5"
<template>
<div>hello</div>
<img alt="a cat" />
<input placeholder="Search" />
<button title="Close">x</button>
</template>
```

<span id="vue-no-bare-strings-in-template-good"></span>

**Bon**

Le contenu traduisible appelle $t ; les exemples de ponctuation et de valeurs uniquement numériques sont des exceptions autorisées.

```vue annotate="add:2,3,4,5,6"
<template>
<div>{{ $t('hello') }}</div>
<img :alt="$t('cat')" />
<div>-</div>
<div>123</div>
<button :title="$t('close')">{{ $t('x') }}</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L47) · [Toutes les règles](all.md)

### `vue/no-boolean-attr-value`

Interdire les valeurs explicites des attributs HTML booléens

[Mauvais](#vue-no-boolean-attr-value-bad) · [Bon](#vue-no-boolean-attr-value-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les attributs booléens disabled et checked contiennent inutilement des valeurs textuelles.

```vue annotate="remove:2,3,4"
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

<span id="vue-no-boolean-attr-value-good"></span>

**Bon**

La présence de chaque attribut booléen exprime le même état actif sans valeur.

```vue annotate="add:2,3,4"
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) · [Toutes les règles](all.md)

### `vue/no-child-content`

Interdire le contenu enfant lors de l’utilisation de v-html ou v-text

[Mauvais](#vue-no-child-content-bad) · [Bon](#vue-no-child-content-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

v-text remplace le contenu du paragraphe ; le texte de repli écrit dans le template ne peut donc pas être conservé avec cette directive.

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

<span id="vue-no-child-content-good"></span>

**Bon**

Supprimer le texte enfant laisse v-text comme unique source du contenu du paragraphe.

```vue annotate="add:2"
<template>
  <p v-text="message" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [Toutes les règles](all.md)

### `vue/no-deprecated-filter`

Interdire la syntaxe obsolète des filtres de Vue 2 utilisant l’opérateur pipe

[Mauvais](#vue-no-deprecated-filter-bad) · [Bon](#vue-no-deprecated-filter-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le pipe utilise la syntaxe supprimée des filtres Vue pour appliquer capitalize.

```vue annotate="remove:2"
<template>
{{ message | capitalize }}
</template>
```

<span id="vue-no-deprecated-filter-good"></span>

**Bon**

L’appel capitalize(message) applique la transformation sous la forme d’une expression ordinaire.

```vue annotate="add:2"
<template>
{{ capitalize(message) }}
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L53) · [Toutes les règles](all.md)

### `vue/no-deprecated-functional-template`

Interdire l’attribut `functional` sur le `<template>` d’un SFC

[Mauvais](#vue-no-deprecated-functional-template-bad) · [Bon](#vue-no-deprecated-functional-template-good)

Gravité par défaut: `error`  
Préréglages: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template du SFC possède l’attribut functional supprimé et lit l’ancien contexte props.

```vue annotate="remove:1,2"
<template functional>
  <div>{{ props.msg }}</div>
</template>
```

<span id="vue-no-deprecated-functional-template-good"></span>

**Bon**

Le template ordinaire omet functional et lit directement la liaison msg du composant.

```vue annotate="add:1,2"
<template>
  <div>{{ msg }}</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) · [Toutes les règles](all.md)

### `vue/no-deprecated-html-element-is`

Interdire l’attribut `is` sur les éléments HTML natifs

[Mauvais](#vue-no-deprecated-html-element-is-bad) · [Bon](#vue-no-deprecated-html-element-is-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un div natif utilise l’ancien attribut is sans préfixe pour demander un composant Vue.

```vue annotate="remove:2"
<template>
  <div is="MyComponent" />
</template>
```

<span id="vue-no-deprecated-html-element-is-good"></span>

**Bon**

Un composant dynamique utilise :is ; la syntaxe sur un élément natif utilise explicitement le préfixe vue:.

```vue annotate="add:2,3"
<template>
  <component :is="MyComponent" />
  <div is="vue:MyComponent" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) · [Toutes les règles](all.md)

### `vue/no-deprecated-inline-template`

Interdire l’attribut obsolète `inline-template`

[Mauvais](#vue-no-deprecated-inline-template-bad) · [Bon](#vue-no-deprecated-inline-template-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Card utilise l’attribut obsolète inline-template pour le contenu qui lui est fourni.

```vue annotate="remove:2"
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

<span id="vue-no-deprecated-inline-template-good"></span>

**Bon**

Le même contenu est transmis normalement, sans l’attribut inline-template.

```vue annotate="add:2"
<template>
<Card><p>Details</p></Card>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) · [Toutes les règles](all.md)

### `vue/no-deprecated-router-link-tag-prop`

Interdire la prop `tag` sur &lt;router-link&gt;

[Mauvais](#vue-no-deprecated-router-link-tag-prop-bad) · [Bon](#vue-no-deprecated-router-link-tag-prop-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

RouterLink utilise la prop tag supprimée pour demander un élément button.

```vue annotate="remove:2"
<template>
  <router-link to="/home" tag="button">Home</router-link>
</template>
```

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

**Bon**

Le slot fournit navigate à un bouton explicitement écrit dans le template.

```vue annotate="add:2,3,4"
<template>
  <router-link to="/home" v-slot="{ navigate }">
    <button @click="navigate">Home</button>
  </router-link>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) · [Toutes les règles](all.md)

### `vue/no-deprecated-scope-attribute`

Interdire l’attribut obsolète `scope` sur &lt;template&gt;

[Mauvais](#vue-no-deprecated-scope-attribute-bad) · [Bon](#vue-no-deprecated-scope-attribute-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template du slot déclare props au moyen de l’attribut obsolète scope.

```vue annotate="remove:2"
<template>
<Card><template scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-scope-attribute-good"></span>

**Bon**

La directive du slot par défaut déclare la même liaison props avec la syntaxe actuelle des slots.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) · [Toutes les règles](all.md)

### `vue/no-deprecated-slot-attribute`

Interdire l’attribut obsolète `slot`

[Mauvais](#vue-no-deprecated-slot-attribute-bad) · [Bon](#vue-no-deprecated-slot-attribute-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le slot header est sélectionné au moyen de l’ancien attribut slot.

```vue annotate="remove:3,4"
<template>
  <Foo>
    <template slot="header"><h1>Title</h1></template>
    <div :slot="name">Title</div>
  </Foo>
</template>
```

<span id="vue-no-deprecated-slot-attribute-good"></span>

**Bon**

v-slot:header sélectionne explicitement le slot header avec la directive actuelle.

```vue annotate="add:3"
<template>
  <Foo>
    <template v-slot:header><h1>Title</h1></template>
  </Foo>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) · [Toutes les règles](all.md)

### `vue/no-deprecated-slot-scope-attribute`

Interdire l’attribut obsolète `slot-scope`

[Mauvais](#vue-no-deprecated-slot-scope-attribute-bad) · [Bon](#vue-no-deprecated-slot-scope-attribute-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template reçoit les props du slot au moyen de l’attribut obsolète slot-scope.

```vue annotate="remove:2"
<template>
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

**Bon**

La directive #default reçoit ces props sans slot-scope.

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) · [Toutes les règles](all.md)

### `vue/no-deprecated-v-bind-sync`

Interdire le modificateur obsolète `.sync` sur `v-bind`

[Mauvais](#vue-no-deprecated-v-bind-sync-bad) · [Bon](#vue-no-deprecated-v-bind-sync-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les liaisons utilisent le modificateur .sync supprimé, y compris en combinaison avec .camel.

```vue annotate="remove:2,3,4"
<template>
<MyComponent :title.sync="title" />
<MyComponent v-bind:title.sync="title" />
<MyComponent :title.sync.camel="title" />
</template>
```

<span id="vue-no-deprecated-v-bind-sync-good"></span>

**Bon**

Utilisez une liaison title ordinaire à sens unique, ou v-model:title lorsqu’un canal de mise à jour est nécessaire.

```vue annotate="add:2,3"
<template>
<MyComponent :title="title" />
<MyComponent v-model:title="title" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) · [Toutes les règles](all.md)

### `vue/no-deprecated-v-on-native-modifier`

Interdire le modificateur obsolète `.native` sur `v-on`

[Mauvais](#vue-no-deprecated-v-on-native-modifier-bad) · [Bon](#vue-no-deprecated-v-on-native-modifier-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les gestionnaires du composant utilisent le modificateur d’événement .native supprimé.

```vue annotate="remove:2,3,4"
<template>
<MyComponent @click.native="handler" />
<MyComponent v-on:click.native="handler" />
<MyComponent @click.native.stop="handler" />
</template>
```

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

**Bon**

Les gestionnaires omettent .native et conservent les autres modificateurs d’événement, tels que .stop.

```vue annotate="add:2,3"
<template>
<MyComponent @click="handler" />
<MyComponent @click.stop="handler" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) · [Toutes les règles](all.md)

### `vue/no-deprecated-v-on-number-modifiers`

Interdire les modificateurs numériques obsolètes `keyCode` sur `v-on`

[Mauvais](#vue-no-deprecated-v-on-number-modifiers-bad) · [Bon](#vue-no-deprecated-v-on-number-modifiers-good)

Gravité par défaut: `error`  
Préréglages: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les gestionnaires de clavier identifient les touches par les codes numériques supprimés 13 et 27.

```vue annotate="remove:2,3,4"
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

**Bon**

Les gestionnaires utilisent les modificateurs de touches nommés enter et esc.

```vue annotate="add:2,3"
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) · [Toutes les règles](all.md)

### `vue/no-dupe-v-else-if`

Interdire les conditions répétées dans les chaînes `v-if` / `v-else-if`

[Mauvais](#vue-no-dupe-v-else-if-bad) · [Bon](#vue-no-dupe-v-else-if-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le else-if répète la condition ready déjà testée par la première branche, ce qui rend cette branche suivante inaccessible.

```vue annotate="remove:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

<span id="vue-no-dupe-v-else-if-good"></span>

**Bon**

La deuxième branche teste loading, un état distinct qui permet d’atteindre le else-if.

```vue annotate="add:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) · [Toutes les règles](all.md)

### `vue/no-duplicate-attributes`

Interdire les attributs en double sur un même élément

[Mauvais](#vue-no-duplicate-attributes-bad) · [Bon](#vue-no-duplicate-attributes-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le même bouton déclare class deux fois au lieu de réunir les classes dans une seule valeur.

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

<span id="vue-no-duplicate-attributes-good"></span>

**Bon**

Les deux noms de classes figurent dans un unique attribut class.

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) · [Toutes les règles](all.md)

### `vue/no-empty-component-block`

Interdire les blocs SFC vides

[Mauvais](#vue-no-empty-component-block-bad) · [Bon](#vue-no-empty-component-block-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les blocs template, script et style ne contiennent aucun contenu significatif.

```vue annotate="remove:1,3,5"
<template></template>

<script></script>

<style>
</style>
```

<span id="vue-no-empty-component-block-good"></span>

**Bon**

Chaque bloc conservé contient du balisage, des déclarations de script ou des déclarations de style effectifs.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42) · [Toutes les règles](all.md)

### `vue/no-inline-style`

Déconseiller l’utilisation d’attributs de style en ligne

[Mauvais](#vue-no-inline-style-bad) · [Bon](#vue-no-inline-style-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’attribut statique style place la déclaration de couleur dans l’élément.

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

<span id="vue-no-inline-style-good"></span>

**Bon**

Les classes expriment la couleur fixe ; la largeur dépendant de ratio reste une liaison de style dynamique, hors du contrôle des attributs statiques.

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) · [Toutes les règles](all.md)

### `vue/no-invalid-html-attribute`

Interdire les valeurs statiques invalides des attributs HTML

[Mauvais](#vue-no-invalid-html-attribute-bad) · [Bon](#vue-no-invalid-html-attribute-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le lien utilise stylesheet comme valeur de rel, alors que cette valeur appartient aux éléments link de feuilles de style.

```vue annotate="remove:2"
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

<span id="vue-no-invalid-html-attribute-good"></span>

**Bon**

Le lien utilise help, une valeur de rel adaptée à une ressource d’aide liée.

```vue annotate="add:2"
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) · [Toutes les règles](all.md)

### `vue/no-lone-template`

Interdire les éléments `<template>` inutiles

[Mauvais](#vue-no-lone-template-bad) · [Bon](#vue-no-lone-template-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template interne n’a ni directive ni rôle de slot lui donnant une fonction structurelle.

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

<span id="vue-no-lone-template-good"></span>

**Bon**

Supprimer l’enveloppe inutile laisse le paragraphe directement à l’intérieur de div.

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) · [Toutes les règles](all.md)

### `vue/no-multi-spaces`

Interdire plusieurs espaces consécutifs

[Mauvais](#vue-no-multi-spaces-bad) · [Bon](#vue-no-multi-spaces-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Deux espaces séparent les attributs, ou le nom de l’élément et son premier attribut.

```vue annotate="remove:2,3"
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

<span id="vue-no-multi-spaces-good"></span>

**Bon**

Un seul espace sépare les mêmes attributs.

```vue annotate="add:2,3"
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) · [Toutes les règles](all.md)

### `vue/no-multiple-objects-in-class`

Interdire plusieurs objets littéraux dans une liaison :class sous forme de tableau

[Mauvais](#vue-no-multiple-objects-in-class-bad) · [Bon](#vue-no-multiple-objects-in-class-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un tableau de classes contient deux objets littéraux au premier niveau qui peuvent être fusionnés.

```vue annotate="remove:2,3"
<template>
<div :class="[{ a }, { b }]"></div>
<div :class="[{ active: isActive }, { error: hasError }]"></div>
</template>
```

<span id="vue-no-multiple-objects-in-class-good"></span>

**Bon**

Un seul objet contient les conditions des classes ; les tableaux comprenant un objet et une chaîne, ou des entrées non littérales, restent autorisés.

```vue annotate="add:2,3,4"
<template>
<div :class="{ a, b }"></div>
<div :class="[{ active: isActive }, 'static']"></div>
<div :class="[foo, bar]"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) · [Toutes les règles](all.md)

### `vue/no-multiple-template-root`

Interdire plusieurs nœuds racines dans un template

[Mauvais](#vue-no-multiple-template-root-bad) · [Bon](#vue-no-multiple-template-root-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Activez cette règle uniquement pour un contrat à racine unique. Vue 3 prend normalement en charge les fragments.

**Configuration (Vite+)**

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

**Mauvais**

La convention à racine unique, activée explicitement, trouve deux paragraphes frères à la racine du template.

```vue annotate="remove:2,3"
<template>
<p>First</p>
<p>Second</p>
</template>
```

<span id="vue-no-multiple-template-root-good"></span>

**Bon**

Un élément section enveloppe les paragraphes dans une seule racine ; activez cette convention uniquement lorsqu’un contrat à racine unique est souhaité.

```vue annotate="add:2"
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) · [Toutes les règles](all.md)

### `vue/no-mutating-props`

Interdire la modification des props d’un composant

[Mauvais](#vue-no-mutating-props-bad) · [Bon](#vue-no-mutating-props-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

Incrémenter props.count modifie directement une valeur fournie par le parent.

```vue annotate="remove:4"
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

<span id="vue-no-mutating-props-good"></span>

**Bon**

Le composant émet update:count avec la nouvelle valeur et laisse au parent la responsabilité de mettre à jour la prop.

```vue annotate="add:3,5,6,7"
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L42) · [Toutes les règles](all.md)

### `vue/no-negated-v-if-condition`

Interdire une condition v-if négative lorsque la chaîne comporte un v-else

[Mauvais](#vue-no-negated-v-if-condition-bad) · [Bon](#vue-no-negated-v-if-condition-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les branches associées v-if et v-else commencent par une condition négative.

```vue
<template>
<div v-if="!ok">A</div>
<div v-else>B</div>
</template>
```

<span id="vue-no-negated-v-if-condition-good"></span>

**Bon**

Une condition ok positive vient en premier ; lors de l’inversion d’une condition, placez d’abord la branche opposée d’origine. Un v-if négatif isolé et les comparaisons !== restent autorisés.

```vue annotate="add:2,3,4,6,7"
<template>
<div v-if="ok">B</div>
<div v-else>A</div>

<div v-if="!ok">A</div>

<div v-if="a !== b">A</div>
<div v-else>B</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L37) · [Toutes les règles](all.md)

### `vue/no-non-component-keep-alive-child`

Interdire les enveloppes d’éléments ordinaires directement sous `<KeepAlive>`

[Mauvais](#vue-no-non-component-keep-alive-child-bad) · [Bon](#vue-no-non-component-keep-alive-child-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

KeepAlive enveloppe conditionnellement un div natif au lieu de mettre directement UserCard en cache.

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

**Bon**

Le premier exemple fait de UserCard l’enfant conditionnel. L’enveloppe avec v-show illustre une structure hors du contrôle des enfants conditionnels et ne garantit pas la mise en cache de l’enveloppe native.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L14) · [Toutes les règles](all.md)

### `vue/no-preprocessor-lang`

Déconseiller les préprocesseurs CSS au profit du CSS moderne

[Mauvais](#vue-no-preprocessor-lang-bad) · [Bon](#vue-no-preprocessor-lang-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Non implémentée pour le lint des SFC  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Prise en charge actuelle: `no-sfc-finding`

Cette entrée du catalogue n’émet actuellement aucun diagnostic propre à cette règle lors du lint des SFC. La paire Mauvais/Bon décrit la convention souhaitée, et non un diagnostic exécutable. Activer l’identifiant ne fournit pas le contrôle SFC manquant.

**ID configuré (aucun diagnostic SFC actuellement)**

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

**Mauvais**

Le bloc style sélectionne SCSS avec lang. Cela décrit la convention souhaitée sans préprocesseur ; le traitement SFC actuel n’émet pas de diagnostic pour cette règle.

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

<span id="vue-no-preprocessor-lang-good"></span>

**Bon**

Les mêmes déclarations CSS omettent le lang du préprocesseur. C’est une correction de la convention, et non une différence de diagnostics Mauvais/Bon exécutable actuellement.

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

Le bon exemple illustre la convention visée ; le traitement actuel des SFC n’émet le diagnostic propre à cette règle pour aucun des deux exemples.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) · [Toutes les règles](all.md)

### `vue/no-reserved-component-names`

Interdire l’utilisation de noms réservés comme noms de composants

[Mauvais](#vue-no-reserved-component-names-bad) · [Bon](#vue-no-reserved-component-names-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le nom de composant button entre en conflit avec le nom d’un élément HTML natif.

```vue annotate="remove:1,2,3,4"
<script>
export default {
  name: "button",
};
</script>
```

<span id="vue-no-reserved-component-names-good"></span>

**Bon**

AppButton est un nom de composant applicatif qui ne réutilise pas le nom natif button.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) · [Toutes les règles](all.md)

### `vue/no-root-v-if`

Interdire v-if sur l’unique élément racine d’un template

[Mauvais](#vue-no-root-v-if-bad) · [Bon](#vue-no-root-v-if-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La racine du composant elle-même apparaît et disparaît sous le contrôle de v-if.

```vue annotate="remove:2"
<template>
  <div v-if="show">content</div>
</template>
```

<span id="vue-no-root-v-if-good"></span>

**Bon**

Un div externe stable reste la racine, tandis que le paragraphe imbriqué porte la condition de visibilité.

```vue annotate="add:2,3,4"
<template>
  <div>
    <p v-if="show">content</p>
  </div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) · [Toutes les règles](all.md)

### `vue/no-script-non-standard-lang`

Déconseiller les valeurs lang non standard dans les scripts

[Mauvais](#vue-no-script-non-standard-lang-bad) · [Bon](#vue-no-script-non-standard-lang-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Non implémentée pour le lint des SFC  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Prise en charge actuelle: `no-sfc-finding`

Cette entrée du catalogue n’émet actuellement aucun diagnostic propre à cette règle lors du lint des SFC. La paire Mauvais/Bon décrit la convention souhaitée, et non un diagnostic exécutable. Activer l’identifiant ne fournit pas le contrôle SFC manquant.

**ID configuré (aucun diagnostic SFC actuellement)**

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

**Mauvais**

Le script utilise la syntaxe CoffeeScript avec lang=coffee. Le traitement SFC actuel n’émet pas cette règle du catalogue pour ce langage.

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

<span id="vue-no-script-non-standard-lang-good"></span>

**Bon**

Le script utilise une déclaration TypeScript ordinaire avec lang=ts, illustrant la convention de langage souhaitée.

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

Le bon exemple illustre la convention visée ; le traitement actuel des SFC n’émet le diagnostic propre à cette règle pour aucun des deux exemples.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [Toutes les règles](all.md)

### `vue/no-src-attribute`

Déconseiller l’attribut src sur les blocs SFC

[Mauvais](#vue-no-src-attribute-bad) · [Bon](#vue-no-src-attribute-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les blocs du SFC délèguent leur contenu template, script et style aux fichiers désignés par src.

```vue annotate="remove:1,2,3"
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

<span id="vue-no-src-attribute-good"></span>

**Bon**

Chaque bloc SFC contient son propre contenu sans attribut src externe.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16) · [Toutes les règles](all.md)

### `vue/no-static-inline-styles`

Interdire les attributs de style en ligne statiques

[Mauvais](#vue-no-static-inline-styles-bad) · [Bon](#vue-no-static-inline-styles-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le paragraphe porte la déclaration de couleur constante dans son attribut style.

```vue annotate="remove:1,2,3"
<template>
<p style="color: red">Notice</p>
</template>
```

<span id="vue-no-static-inline-styles-good"></span>

**Bon**

Une classe notice et une feuille de style scoped définissent la couleur constante en dehors de l’attribut du template.

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) · [Toutes les règles](all.md)

### `vue/no-template-key`

Interdire l’attribut `key` sur `<template>`

[Mauvais](#vue-no-template-key-bad) · [Bon](#vue-no-template-key-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Une enveloppe template sans boucle possède une key, alors qu’elle ne constitue pas la limite d’une itération à clé.

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

<span id="vue-no-template-key-good"></span>

**Bon**

La clé appartient à une itération template v-for, où elle identifie chaque fragment répété.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) · [Toutes les règles](all.md)

### `vue/no-template-lang`

Déconseiller l’attribut lang sur le bloc template

[Mauvais](#vue-no-template-lang-bad) · [Bon](#vue-no-template-lang-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Non implémentée pour le lint des SFC  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Prise en charge actuelle: `no-sfc-finding`

Cette entrée du catalogue n’émet actuellement aucun diagnostic propre à cette règle lors du lint des SFC. La paire Mauvais/Bon décrit la convention souhaitée, et non un diagnostic exécutable. Activer l’identifiant ne fournit pas le contrôle SFC manquant.

**ID configuré (aucun diagnostic SFC actuellement)**

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

**Mauvais**

Le template sélectionne Pug avec lang. Il s’agit d’une convention souhaitée limitant les templates au HTML ; le traitement SFC actuel n’émet pas de diagnostic pour cet identifiant du catalogue.

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

<span id="vue-no-template-lang-good"></span>

**Bon**

Un template HTML ordinaire omet lang et utilise directement le paragraphe. Cela illustre la convention sans prétendre qu’un diagnostic SFC est actuellement émis.

```vue annotate="add:1,2"
<template>
<p>Notice</p>
</template>
```

Le bon exemple illustre la convention visée ; le traitement actuel des SFC n’émet le diagnostic propre à cette règle pour aucun des deux exemples.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [Toutes les règles](all.md)

### `vue/no-template-shadow`

Interdire les noms de variables qui masquent des variables d’une portée externe

[Mauvais](#vue-no-template-shadow-bad) · [Bon](#vue-no-template-shadow-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Le contrôle actuel compare les liaisons de v-for imbriqués. Il ne signale pas une liaison v-for isolée simplement parce qu’elle porte le même nom qu’une liaison du script.

**Configuration (Vite+)**

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

**Mauvais**

Le v-for interne déclare à nouveau item et masque la liaison item externe dans la boucle imbriquée.

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

<span id="vue-no-template-shadow-good"></span>

**Bon**

La boucle interne déclare child, laissant item disponible pour la ligne externe et child pour la ligne imbriquée.

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [Toutes les règles](all.md)

### `vue/no-template-target-blank`

Interdire target="_blank" sans rel="noopener noreferrer"

[Mauvais](#vue-no-template-target-blank-bad) · [Bon](#vue-no-template-target-blank-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le lien externe ouvre un nouveau contexte de navigation sans la protection rel attendue.

```vue annotate="remove:2"
<template>
<a href="https://example.com" target="_blank">x</a>
</template>
```

<span id="vue-no-template-target-blank-good"></span>

**Bon**

Le même lien inclut noopener noreferrer en plus de target=_blank.

```vue annotate="add:2"
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) · [Toutes les règles](all.md)

### `vue/no-textarea-mustache`

Interdire l’interpolation à doubles accolades dans `<textarea>`

[Mauvais](#vue-no-textarea-mustache-bad) · [Bon](#vue-no-textarea-mustache-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le textarea place message dans une interpolation enfant au lieu de lier sa valeur.

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

<span id="vue-no-textarea-mustache-good"></span>

**Bon**

v-model lie la valeur modifiable du textarea à message.

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [Toutes les règles](all.md)

### `vue/no-undefined-refs`

Interdire les références à des variables non définies dans les templates

[Mauvais](#vue-no-undefined-refs-bad) · [Bon](#vue-no-undefined-refs-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template lit missing, alors que le script ne déclare que message.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

<span id="vue-no-undefined-refs-good"></span>

**Bon**

L’interpolation lit la liaison message existante.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) · [Toutes les règles](all.md)

### `vue/no-unsafe-url`

Signaler les liaisons d’URL potentiellement dangereuses

[Mauvais](#vue-no-unsafe-url-bad) · [Bon](#vue-no-unsafe-url-good)

Gravité par défaut: `warning`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La destination du lien commence par le schéma exécutable javascript:.

```vue annotate="remove:2"
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

<span id="vue-no-unsafe-url-good"></span>

**Bon**

Le lien utilise la destination de navigation locale ordinaire /next.

```vue annotate="add:2"
<template>
<a href="/next">Continue</a>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [Toutes les règles](all.md)

### `vue/no-unsandboxed-iframe`

Exiger un attribut sandbox sur les éléments iframe

[Mauvais](#vue-no-unsandboxed-iframe-bad) · [Bon](#vue-no-unsandboxed-iframe-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le cadre intégré n’a pas d’attribut sandbox limitant ses capacités.

```vue annotate="remove:2"
<template>
<iframe src="/embed"></iframe>
</template>
```

<span id="vue-no-unsandboxed-iframe-good"></span>

**Bon**

sandbox applique des restrictions ; allow-scripts autorise explicitement cette seule capacité lorsque cela est nécessaire.

```vue annotate="add:2,3"
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) · [Toutes les règles](all.md)

### `vue/no-unused-components`

Interdire l’enregistrement de composants inutilisés dans les templates

[Mauvais](#vue-no-unused-components-bad) · [Bon](#vue-no-unused-components-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

UserAvatar est importé comme composant, mais le template ne l’affiche jamais.

```vue annotate="remove:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <p>{{ user.name }}</p>
</template>
```

<span id="vue-no-unused-components-good"></span>

**Bon**

Le template affiche le composant UserAvatar importé et lui transmet la liaison user.

```vue annotate="add:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L46) · [Toutes les règles](all.md)

### `vue/no-unused-properties`

Interdire les propriétés inutilisées définies dans defineProps

[Mauvais](#vue-no-unused-properties-bad) · [Bon](#vue-no-unused-properties-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le composant déclare description comme prop, mais n’affiche que title.

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
```

<span id="vue-no-unused-properties-good"></span>

**Bon**

Les deux props déclarées sont référencées par le template.

```vue annotate="add:7"
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ description }}</p>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) · [Toutes les règles](all.md)

### `vue/no-unused-refs`

Signaler les refs de template (ref="x") jamais référencées dans &lt;script&gt;

[Mauvais](#vue-no-unused-refs-bad) · [Bon](#vue-no-unused-refs-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template déclare le nom de ref unused sans liaison de référence correspondante dans le script.

```vue annotate="remove:1,3"
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

<span id="vue-no-unused-refs-good"></span>

**Bon**

La ref de template inputEl possède une liaison ref du même nom dans script setup.

```vue annotate="add:1,3,4"
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) · [Toutes les règles](all.md)

### `vue/no-unused-setup-bindings`

Interdire les liaisons script setup qui ne sont jamais lues

[Mauvais](#vue-no-unused-setup-bindings-bad) · [Bon](#vue-no-unused-setup-bindings-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La liaison message de script setup n’est jamais lue par le template.

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

<span id="vue-no-unused-setup-bindings-good"></span>

**Bon**

Le paragraphe interpole message et utilise ainsi la liaison déclarée.

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) · [Toutes les règles](all.md)

### `vue/no-unused-vars`

Interdire les variables inutilisées définies dans les directives v-for et v-slot

[Mauvais](#vue-no-unused-vars-bad) · [Bon](#vue-no-unused-vars-good)

Gravité par défaut: `warning`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La boucle déclare un index inutilisé et le slot déclare foo sans y faire référence.

```vue annotate="remove:2,3,4"
<template>
  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ foo }">
    <span>Hello</span>
  </template>
</template>
```

<span id="vue-no-unused-vars-good"></span>

**Bon**

Les exemples utilisent index ou indiquent qu’il est volontairement inutilisé en le nommant _index, et le slot affiche data. Les clés fondées sur l’index illustrent uniquement un usage ici, sans recommander cet index pour une identité stable des éléments.

```vue annotate="add:2,3,4,5"
<template>
  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
  <li v-for="(item, _index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ data }">
    <span>{{ data }}</span>
  </template>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) · [Toutes les règles](all.md)

### `vue/no-use-v-else-with-v-for`

Interdire `v-else-if` ou `v-else` sur le même élément que `v-for`

[Mauvais](#vue-no-use-v-else-with-v-for-bad) · [Bon](#vue-no-use-v-else-with-v-for-good)

Gravité par défaut: `warning`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La branche else et l’itération v-for sont attachées au même paragraphe.

```vue annotate="remove:3"
<template>
<p v-if="ready">Ready</p>
<p v-else v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

<span id="vue-no-use-v-else-with-v-for-good"></span>

**Bon**

Un template distinct porte v-else, et son paragraphe enfant porte v-for.

```vue annotate="add:3"
<template>
<p v-if="ready">Ready</p>
<template v-else><p v-for="item in items" :key="item.id">{{ item.name }}</p></template>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) · [Toutes les règles](all.md)

### `vue/no-use-v-if-with-v-for`

Interdire `v-if` sur le même élément que `v-for`

[Mauvais](#vue-no-use-v-if-with-v-for-bad) · [Bon](#vue-no-use-v-if-with-v-for-good)

Gravité par défaut: `warning`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le même élément de liste combine v-if et v-for et teste la visibilité au moyen de la liaison de la boucle.

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

<span id="vue-no-use-v-if-with-v-for-good"></span>

**Bon**

Une collection calculée filtre les éléments visibles avant que le template n’itère dessus.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) · [Toutes les règles](all.md)

### `vue/no-useless-mustaches`

Interdire une interpolation à doubles accolades dont l’expression est une chaîne littérale constante

[Mauvais](#vue-no-useless-mustaches-bad) · [Bon](#vue-no-useless-mustaches-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’interpolation ne contient qu’une chaîne constante et ne nécessite pas d’évaluation d’expression.

```vue annotate="remove:2,3,4"
<template>
<div>{{ 'x' }}</div>
<div>{{ "x" }}</div>
<div>{{ `x` }}</div>
</template>
```

<span id="vue-no-useless-mustaches-good"></span>

**Bon**

Le texte littéral est écrit directement ; les expressions de variables, les chaînes de template avec interpolation et les espaces de séparation intentionnels restent des cas d’interpolation.

```vue annotate="add:2,3,4,5"
<template>
<div>x</div>
<div>{{ x }}</div>
<div>{{ `pre-${x}` }}</div>
<span>A</span> {{ " " }} <span>B</span>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) · [Toutes les règles](all.md)

### `vue/no-useless-template-attributes`

Interdire les attributs inutiles sur les éléments `<template>`

[Mauvais](#vue-no-useless-template-attributes-bad) · [Bon](#vue-no-useless-template-attributes-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le template conditionnel possède une classe, mais cette enveloppe structurelle ne produit pas d’élément DOM qui puisse la recevoir.

```vue annotate="remove:2"
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

<span id="vue-no-useless-template-attributes-good"></span>

**Bon**

La classe est déplacée vers le paragraphe qui est effectivement affiché, tandis que v-if reste sur le template structurel.

```vue annotate="add:2"
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [Toutes les règles](all.md)

### `vue/no-useless-v-bind`

Interdire un v-bind dont la valeur est une simple chaîne littérale

[Mauvais](#vue-no-useless-v-bind-bad) · [Bon](#vue-no-useless-v-bind-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La liaison foo évalue une chaîne constante entre guillemets ou une chaîne de template sans interpolation.

```vue annotate="remove:2,3"
<template>
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

<span id="vue-no-useless-v-bind-good"></span>

**Bon**

La valeur constante devient un attribut statique ; les valeurs variables et interpolées conservent leur liaison.

```vue annotate="add:2,3,4"
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) · [Toutes les règles](all.md)

### `vue/no-v-for-template-key-on-child`

Interdire `key` sur l’enfant d’un `<template v-for>`

[Mauvais](#vue-no-v-for-template-key-on-child-bad) · [Bon](#vue-no-v-for-template-key-on-child-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le paragraphe enfant porte la clé, tandis que l’itération du template elle-même n’en possède pas.

```vue annotate="remove:2"
<template>
<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>
</template>
```

<span id="vue-no-v-for-template-key-on-child-good"></span>

**Bon**

La clé est déplacée vers template v-for et identifie le fragment répété dans son ensemble.

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) · [Toutes les règles](all.md)

### `vue/no-v-html`

Déconseiller v-html pour prévenir les vulnérabilités XSS

[Mauvais](#vue-no-v-html-bad) · [Bon](#vue-no-v-html-good)

Gravité par défaut: `warning`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

v-html interprète le contenu comme du HTML plutôt que comme du texte ordinaire.

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

<span id="vue-no-v-html-good"></span>

**Bon**

L’interpolation à doubles accolades affiche le contenu comme du texte échappé au lieu d’injecter du HTML.

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) · [Toutes les règles](all.md)

### `vue/no-v-text`

Interdire la directive v-text ; préférer l’interpolation à doubles accolades

[Mauvais](#vue-no-v-text-bad) · [Bon](#vue-no-v-text-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le contenu du div est fourni par la directive v-text.

```vue annotate="remove:2"
<template>
<div v-text="message"></div>
</template>
```

<span id="vue-no-v-text-good"></span>

**Bon**

L’interpolation à doubles accolades exprime la même liaison de texte directement dans le contenu de l’élément.

```vue annotate="add:2"
<template>
<div>{{ message }}</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) · [Toutes les règles](all.md)

### `vue/no-v-text-v-html-on-component`

Interdire v-text / v-html sur les éléments de composants

[Mauvais](#vue-no-v-text-v-html-on-component-bad) · [Bon](#vue-no-v-text-v-html-on-component-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

La balise du composant reçoit v-html ou v-text, qui remplace le contenu d’un élément au lieu de fournir les slots du composant.

```vue annotate="remove:2,3"
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

<span id="vue-no-v-text-v-html-on-component-good"></span>

**Bon**

Les cibles HTML natives peuvent recevoir les directives ; MyComponent reçoit son contenu via le slot par défaut.

```vue annotate="add:2,3,4"
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) · [Toutes les règles](all.md)

### `vue/permitted-contents`

Faire respecter les règles du modèle de contenu HTML

[Mauvais](#vue-permitted-contents-bad) · [Bon](#vue-permitted-contents-good)

Gravité par défaut: `error`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les exemples placent du contenu de bloc dans p, omettent le corps du tableau, imbriquent des contrôles interactifs ou placent un div directement dans ul.

```vue annotate="remove:2,3,4,5"
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

<span id="vue-permitted-contents-good"></span>

**Bon**

Les exemples utilisent du contenu en ligne dans le paragraphe, un tbody explicite et des enfants li. Le composant personnalisé MyItem n’est pas considéré comme un enfant natif connu de ul.

```vue annotate="add:2,3,4"
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) · [Toutes les règles](all.md)

### `vue/prefer-props-shorthand`

Recommander la syntaxe abrégée des props (Vue 3.4+)

[Mauvais](#vue-prefer-props-shorthand-bad) · [Bon](#vue-prefer-props-shorthand-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Chaque liaison répète le nom de la variable correspondante, y compris l’équivalent camelCase d’un argument séparé par des tirets.

```vue annotate="remove:2,3,4,5"
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
  <span :style="style" />
  <div :aria-label="ariaLabel" />
</template>
```

<span id="vue-prefer-props-shorthand-good"></span>

**Bon**

La syntaxe abrégée des liaisons de même nom de Vue 3.4+ supprime les expressions répétées ; une variable source différente, telle que bar, reste explicite.

```vue annotate="add:2,3,4,5,6"
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <span :style />
  <div :aria-label />
  <MyComponent :foo="bar" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) · [Toutes les règles](all.md)

### `vue/prefer-true-attribute-shorthand`

Préférer la syntaxe abrégée pour un attribut booléen lié à `true`

[Mauvais](#vue-prefer-true-attribute-shorthand-bad) · [Bon](#vue-prefer-true-attribute-shorthand-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un attribut booléen natif disabled est lié à la valeur constante true.

```vue annotate="remove:2"
<template>
<input :disabled="true" />
</template>
```

<span id="vue-prefer-true-attribute-shorthand-good"></span>

**Bon**

L’attribut natif utilise sa syntaxe booléenne abrégée. Les liaisons à false et les props de composants conservent leurs valeurs explicites.

```vue annotate="add:2,3,4,5"
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) · [Toutes les règles](all.md)

### `vue/prop-name-casing`

Imposer une casse aux noms des props déclarées

[Mauvais](#vue-prop-name-casing-bad) · [Bon](#vue-prop-name-casing-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Contrôle les noms des props déclarées, et non la casse des attributs transmis à un enfant.

**Configuration (Vite+)**

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

**Mauvais**

Le nom de prop déclaré user_name utilise des mots séparés par des traits de soulignement.

```vue annotate="remove:2,4"
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

<span id="vue-prop-name-casing-good"></span>

**Bon**

La déclaration et sa référence dans le template utilisent le nom camelCase userName.

```vue annotate="add:2,4"
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) · [Toutes les règles](all.md)

### `vue/require-component-is`

Exiger `v-bind:is` sur les éléments `<component>`

[Mauvais](#vue-require-component-is-bad) · [Bon](#vue-require-component-is-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le `<component>` dynamique n’a pas de cible `is` ; Vue ne peut donc pas choisir le composant à afficher.

```vue annotate="remove:2"
<template>
  <component />
</template>
```

<span id="vue-require-component-is-good"></span>

**Bon**

`:is="currentComponent"` fournit la sélection du composant ; la liaison peut changer à l’exécution.

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) · [Toutes les règles](all.md)

### `vue/require-component-registration`

Exiger un import ou un enregistrement explicite des composants

[Mauvais](#vue-require-component-registration-bad) · [Bon](#vue-require-component-registration-good)

Gravité par défaut: `warning`  
Préréglages: `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

Listez les noms explicites des composants fournis par les plugins de l’application ou par previewSetup de Musea. Les graphies PascalCase et kebab-case sont acceptées ; les expressions régulières ne sont pas interprétées. Les options n’activent pas la règle. Les couches suivantes remplacent la liste ; une liste vide efface les noms hérités.

**Configuration (Vite+)**

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

**Mauvais**

`MissingWidget` n’est ni enregistré ni inclus dans la liste configurée des composants globaux autorisés.

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

<span id="vue-require-component-registration-good"></span>

**Bon**

`MyButton` figure dans l’option `globals` de l’exemple. Cette option exempte un composant global connu ; elle ne l’enregistre ni ne l’importe.

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [Toutes les règles](all.md)

### `vue/require-scoped-style`

Exiger l’attribut scoped sur les balises style

[Mauvais](#vue-require-scoped-style-bad) · [Bon](#vue-require-scoped-style-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le style `.button` n’est pas scoped et peut affecter les éléments correspondants en dehors de ce composant.

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

<span id="vue-require-scoped-style-good"></span>

**Bon**

Ajouter `scoped` applique la portée du composant Vue aux mêmes sélecteur et déclarations.

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
}
</style>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [Toutes les règles](all.md)

### `vue/require-toggle-inside-transition`

Exiger un mécanisme de bascule sur l’élément enveloppé par `<transition>`

[Mauvais](#vue-require-toggle-inside-transition-bad) · [Bon](#vue-require-toggle-inside-transition-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’enfant statique à l’intérieur de `<Transition>` n’a ni visibilité conditionnelle ni sélection dynamique pouvant déclencher une entrée ou une sortie.

```vue annotate="remove:3"
<template>
<transition>
  <div>content</div>
</transition>
</template>
```

<span id="vue-require-toggle-inside-transition-good"></span>

**Bon**

`v-if="show"` modifie la présence de l’enfant et fournit à la transition une limite d’entrée et de sortie.

```vue annotate="add:3"
<template>
<transition>
  <div v-if="show">content</div>
</transition>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L48) · [Toutes les règles](all.md)

### `vue/require-v-for-key`

Exiger `v-bind:key` avec les directives `v-for`

[Mauvais](#vue-require-v-for-key-bad) · [Bon](#vue-require-v-for-key-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Chaque `<li>` répété ne possède pas de clé identifiant l’élément correspondant lors des mises à jour de la liste.

```vue annotate="remove:2"
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

<span id="vue-require-v-for-key-good"></span>

**Bon**

`:key="item.id"` donne à chaque nœud répété l’identité de son élément plutôt que sa position actuelle.

```vue annotate="add:2"
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L35) · [Toutes les règles](all.md)

### `vue/scoped-event-names`

Recommander des noms d’événements préfixés par leur contexte au format context:event

[Mauvais](#vue-scoped-event-names-bad) · [Bon](#vue-scoped-event-names-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`playAudio`, `pauseAudio` et `reloadAudio` encodent leur contexte sous forme de suffixes camelCase au lieu de la convention d’événements séparés par deux-points définie par la règle.

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

**Bon**

`audio:play`, `audio:pause` et `audio:reload` partagent un contexte explicite `audio:`. Le composant émetteur doit utiliser les mêmes noms.

```vue annotate="add:3,4,5"
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) · [Toutes les règles](all.md)

### `vue/sfc-element-order`

Imposer un ordre cohérent des éléments de premier niveau d’un SFC

[Mauvais](#vue-sfc-element-order-bad) · [Bon](#vue-sfc-element-order-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

Le bloc style précède le bloc script, contrairement à l’ordre des blocs SFC configuré.

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

**Bon**

Les blocs suivent l’ordre script → template → style. Les projets peuvent choisir un autre ordre avec l’option typée de cette règle.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) · [Toutes les règles](all.md)

### `vue/single-style-block`

Recommander un seul bloc style

[Mauvais](#vue-single-style-block-bad) · [Bon](#vue-single-style-block-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le composant répartit ses styles scoped de panneau et de titre dans deux blocs style.

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

**Bon**

Les deux sélecteurs restent scoped dans un seul bloc style, respectant la convention du bloc unique sans supprimer aucun style.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) · [Toutes les règles](all.md)

### `vue/slot-name-casing`

Imposer kebab-case aux slots nommés utilisés avec v-slot

[Mauvais](#vue-slot-name-casing-bad) · [Bon](#vue-slot-name-casing-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le slot nommé `mySlot` utilise camelCase là où la règle exige un nom séparé par des tirets.

```vue annotate="remove:2"
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

<span id="vue-slot-name-casing-good"></span>

**Bon**

`#my-slot` utilise kebab-case. Renommez le point d’insertion du slot correspondant avec le même nom.

```vue annotate="add:2"
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [Toutes les règles](all.md)

### `vue/this-in-template`

Interdire `this.` dans les expressions des templates

[Mauvais](#vue-this-in-template-bad) · [Bon](#vue-this-in-template-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les expressions du template accèdent explicitement à `this.message`, `this.className` et `this.handleClick`, alors que Vue expose directement ces liaisons.

```vue annotate="remove:2,3,4"
<template>
<div>{{ this.message }}</div>
<div :class="this.className"></div>
<button @click="this.handleClick()"></button>
</template>
```

<span id="vue-this-in-template-good"></span>

**Bon**

Utilisez directement `message`, `className` et `handleClick`. La chaîne littérale `'this.is.a.string'` reste inchangée, car elle ne constitue pas un accès à un membre.

```vue annotate="add:2,3,4,5"
<template>
<div>{{ message }}</div>
<div :class="className"></div>
<button @click="handleClick()"></button>
<div>{{ 'this.is.a.string' }}</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L33) · [Toutes les règles](all.md)

### `vue/use-unique-element-ids`

Imposer des identifiants d’éléments uniques avec useId() plutôt que des littéraux statiques

[Mauvais](#vue-use-unique-element-ids-bad) · [Bon](#vue-use-unique-element-ids-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

L’identifiant littéral `email` est réutilisé par chaque instance de ce composant, ce qui peut faire pointer son label vers la mauvaise instance lorsque plusieurs sont affichées.

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**Bon**

`useId()` produit l’`emailId` de l’instance ; liez la même valeur au `for` du label et à l’`id` de l’input.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [Toutes les règles](all.md)

### `vue/use-v-on-exact`

Imposer le modificateur `.exact` sur `v-on` en présence de gestionnaires utilisant des modificateurs

[Mauvais](#vue-use-v-on-exact-bad) · [Bon](#vue-use-v-on-exact-good)

Gravité par défaut: `warning`  
Préréglages: `essential`, `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le gestionnaire de clic ordinaire peut également s’exécuter lors d’un Ctrl-clic et chevaucher le gestionnaire `.ctrl` distinct.

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

<span id="vue-use-v-on-exact-good"></span>

**Bon**

`.exact` limite le gestionnaire de clic ordinaire aux clics sans touche modificatrice ; le gestionnaire propre à Ctrl reste distinct.

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

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L28) · [Toutes les règles](all.md)

### `vue/v-bind-style`

Imposer un style de directive `v-bind`

[Mauvais](#vue-v-bind-style-bad) · [Bon](#vue-v-bind-style-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-bind:class` utilise la forme longue alors que le style de liaison configuré exige la syntaxe abrégée avec deux-points.

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

<span id="vue-v-bind-style-good"></span>

**Bon**

`:class` conserve la même expression avec la syntaxe abrégée exigée ; cette règle concerne la graphie, pas le type de la valeur.

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) · [Toutes les règles](all.md)

### `vue/v-on-event-hyphenation`

Imposer des tirets dans les noms d’événements personnalisés de v-on sur les composants

[Mauvais](#vue-v-on-event-hyphenation-bad) · [Bon](#vue-v-on-event-hyphenation-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

L’écouteur du composant personnalisé utilise `@myEvent` au lieu d’un nom d’événement séparé par des tirets.

```vue annotate="remove:2,3"
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
</template>
```

<span id="vue-v-on-event-hyphenation-good"></span>

**Bon**

`@my-event` utilise la graphie exigée pour les événements personnalisés. Les écouteurs d’éléments natifs et les arguments d’événements dynamiques montrés ci-dessous sont hors du champ de ce contrôle.

```vue annotate="add:2,3,4"
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L35) · [Toutes les règles](all.md)

### `vue/v-on-handler-style`

Imposer une référence de méthode ou une fonction en ligne pour les gestionnaires v-on

[Mauvais](#vue-v-on-handler-style-bad) · [Bon](#vue-v-on-handler-style-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les gestionnaires placent des modifications et plusieurs instructions directement dans l’attribut d’événement.

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

<span id="vue-v-on-handler-style-good"></span>

**Bon**

Utilisez une référence de gestionnaire, ou une expression de fonction fléchée ou classique lorsqu’une logique en ligne est nécessaire. La limite de la fonction rend la forme du gestionnaire explicite.

```vue annotate="add:2,3,4,5"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
<button @click="() => count++"></button>
<button @click="function () { count++ }"></button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33) · [Toutes les règles](all.md)

### `vue/v-on-style`

Imposer un style de directive `v-on`

[Mauvais](#vue-v-on-style-bad) · [Bon](#vue-v-on-style-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-on:click` utilise la forme longue de l’écouteur d’événement alors que la règle exige la syntaxe abrégée.

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

<span id="vue-v-on-style-good"></span>

**Bon**

`@click` conserve le même gestionnaire tout en utilisant la syntaxe abrégée configurée.

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) · [Toutes les règles](all.md)

### `vue/v-slot-style`

Imposer un style de directive `v-slot`

[Mauvais](#vue-v-slot-style-bad) · [Bon](#vue-v-slot-style-good)

Gravité par défaut: `warning`  
Préréglages: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le composant utilise `#default` et le template utilise `v-slot:header`, contrairement aux styles définis par la règle pour chaque contexte.

```vue annotate="remove:2,4"
<template>
  <MyComponent #default="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template v-slot:header>Header</template>
  </MyComponent>
</template>
```

<span id="vue-v-slot-style-good"></span>

**Bon**

Utilisez `v-slot` pour le slot par défaut du composant et `#header` pour le slot nommé du template.

```vue annotate="add:2,4"
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [Toutes les règles](all.md)

### `vue/valid-attribute-name`

Exiger des noms d’attributs valides

[Mauvais](#vue-valid-attribute-name-bad) · [Bon](#vue-valid-attribute-name-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Diagnostic du mauvais exemple: `parser/template`

Un nom d’attribut mal formé est diagnostiqué par parser/template avant que cette règle défensive ne voie un attribut. L’exemple Mauvais signale donc parser/template et ne garantit pas un diagnostic distinct de vue/valid-attribute-name.

**Configuration (Vite+)**

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

**Mauvais**

Le guillemet dans `my"attr` rend le nom d’attribut mal formé. Cet exemple produit le diagnostic `parser/template` de l’analyseur, sans garantir un diagnostic distinct de cette règle.

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

<span id="vue-valid-attribute-name-good"></span>

**Bon**

`my-attr` est un nom d’attribut bien formé ; l’analyseur du template peut donc lire l’attribut et sa valeur.

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [Toutes les règles](all.md)

### `vue/valid-template-root`

Exiger une racine `<template>` valide selon la sémantique des fragments de Vue 3

[Mauvais](#vue-valid-template-root-bad) · [Bon](#vue-valid-template-root-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un `<template>` imbriqué ordinaire occupe la racine du template sans directive lui donnant un rôle de rendu.

```vue annotate="remove:2"
<template>
  <template>content</template>
</template>
```

<span id="vue-valid-template-root-good"></span>

**Bon**

Le `<div>` est un élément racine qui peut être affiché. Cet exemple n’impose pas de restriction universelle à une seule racine pour les fragments de Vue 3.

```vue annotate="add:2"
<template>
  <div>content</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) · [Toutes les règles](all.md)

### `vue/valid-v-bind`

Exiger des directives `v-bind` valides

[Mauvais](#vue-valid-v-bind-bad) · [Bon](#vue-valid-v-bind-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le `v-bind` sans argument n’a pas d’expression objet, et la forme à argument vide n’a pas de nom d’attribut.

```vue annotate="remove:2,3"
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

<span id="vue-valid-v-bind-good"></span>

**Bon**

Fournissez un attribut et une expression, liez un objet ou utilisez la syntaxe abrégée de même nom de Vue 3.4+, telle que `:loading`.

```vue annotate="add:2,3,4"
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) · [Toutes les règles](all.md)

### `vue/valid-v-cloak`

Exiger des directives `v-cloak` valides

[Mauvais](#vue-valid-v-cloak-bad) · [Bon](#vue-valid-v-cloak-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-cloak` reçoit une valeur, un argument ou un modificateur alors qu’il n’accepte aucun de ces éléments.

```vue annotate="remove:2,3,4"
<template>
<div v-cloak="foo"></div>
<div v-cloak:arg></div>
<div v-cloak.mod></div>
</template>
```

<span id="vue-valid-v-cloak-good"></span>

**Bon**

Utilisez `v-cloak` seul ; le CSS peut masquer l’élément jusqu’à ce que Vue retire cet attribut après le montage.

```vue annotate="add:2"
<template>
<div v-cloak></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) · [Toutes les règles](all.md)

### `vue/valid-v-else`

Exiger des directives `v-else` valides

[Mauvais](#vue-valid-v-else-bad) · [Bon](#vue-valid-v-else-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les exemples donnent une expression à `v-else`, le combinent avec `v-if` ou omettent la branche conditionnelle qui doit le précéder immédiatement.

```vue annotate="remove:2,3"
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

<span id="vue-valid-v-else-good"></span>

**Bon**

Placez `v-else` seul immédiatement après la branche `v-if` correspondante.

```vue annotate="add:2"
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L32) · [Toutes les règles](all.md)

### `vue/valid-v-for`

Exiger des directives `v-for` valides

[Mauvais](#vue-valid-v-for-bad) · [Bon](#vue-valid-v-for-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les boucles omettent leur expression d’itération ou ajoutent un modificateur `.stop` non pris en charge.

```vue annotate="remove:2,3,4"
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

<span id="vue-valid-v-for-good"></span>

**Bon**

Utilisez `item in items` ou `(item, index) of items` avec une expression d’itération complète et les clés présentées.

```vue annotate="add:2,3"
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L31) · [Toutes les règles](all.md)

### `vue/valid-v-html`

Exiger des directives `v-html` valides

[Mauvais](#vue-valid-v-html-bad) · [Bon](#vue-valid-v-html-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-html` n’a pas d’expression ou utilise un argument ou modificateur que cette directive ne prend pas en charge.

```vue annotate="remove:2,3,4"
<template>
<div v-html></div>
<div v-html:arg="foo"></div>
<div v-html.mod="foo"></div>
</template>
```

<span id="vue-valid-v-html-good"></span>

**Bon**

`v-html="html"` fournit une expression valide. La validité de la syntaxe n’assainit pas le HTML et ne rend pas sûr un contenu non fiable.

```vue annotate="add:2"
<template>
<div v-html="html"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) · [Toutes les règles](all.md)

### `vue/valid-v-if`

Exiger des directives `v-if` valides

[Mauvais](#vue-valid-v-if-bad) · [Bon](#vue-valid-v-if-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les conditions omettent une expression ou combinent `v-if` avec une directive else sur le même nœud.

```vue annotate="remove:2,3,4"
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

<span id="vue-valid-v-if-good"></span>

**Bon**

Chaque `v-if` possède une condition non vide, telle que `ready` ou `count > 0`, sans directive else incompatible.

```vue annotate="add:2,3"
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) · [Toutes les règles](all.md)

### `vue/valid-v-memo`

Exiger des directives `v-memo` valides

[Mauvais](#vue-valid-v-memo-bad) · [Bon](#vue-valid-v-memo-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un `v-memo` seul ne donne à Vue aucune expression de dépendances pour décider quand réutiliser le sous-arbre.

```vue annotate="remove:2"
<template>
  <div v-memo></div>
</template>
```

<span id="vue-valid-v-memo-good"></span>

**Bon**

`v-memo="[valueA, valueB]"` fournit le tableau de dépendances utilisé pour la mémoïsation.

```vue annotate="add:2"
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) · [Toutes les règles](all.md)

### `vue/valid-v-model`

Exiger des directives `v-model` valides

[Mauvais](#vue-valid-v-model-bad) · [Bon](#vue-valid-v-model-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Un `<div>` natif ne peut pas utiliser `v-model` comme contrôle de formulaire, et une directive sans valeur sur input n’a pas d’expression cible modifiable.

```vue annotate="remove:2,3"
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

<span id="vue-valid-v-model-good"></span>

**Bon**

Liez l’input, le select, le textarea ou le composant personnalisé aux variables modifiables présentées.

```vue annotate="add:2,3,4,5"
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) · [Toutes les règles](all.md)

### `vue/valid-v-on`

Exiger des directives `v-on` valides

[Mauvais](#vue-valid-v-on-bad) · [Bon](#vue-valid-v-on-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Les formes d’écouteurs omettent un argument d’événement ou l’expression de gestionnaire ou d’objet nécessaire.

```vue annotate="remove:2,3,4"
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
</template>
```

<span id="vue-valid-v-on-good"></span>

**Bon**

Utilisez un événement avec son gestionnaire, ou transmettez un objet d’écouteurs à `v-on` sans argument.

```vue annotate="add:2,3"
<template>
  <div @click="handleClick"></div>
  <div v-on="{ click: handleClick }"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) · [Toutes les règles](all.md)

### `vue/valid-v-once`

Exiger des directives `v-once` valides

[Mauvais](#vue-valid-v-once-bad) · [Bon](#vue-valid-v-once-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-once` possède une valeur, un argument ou un modificateur, alors que cette directive marque un rendu unique sans recevoir de valeur.

```vue annotate="remove:2,3,4"
<template>
<div v-once="foo"></div>
<div v-once:arg></div>
<div v-once.mod></div>
</template>
```

<span id="vue-valid-v-once-good"></span>

**Bon**

`v-once` seul marque le sous-arbre pour un rendu unique sans syntaxe non prise en charge.

```vue annotate="add:2"
<template>
<div v-once></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) · [Toutes les règles](all.md)

### `vue/valid-v-show`

Exiger des directives `v-show` valides

[Mauvais](#vue-valid-v-show-bad) · [Bon](#vue-valid-v-show-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-show` n’a pas d’expression de visibilité ou est placé sur un `<template>` qui ne produit aucun élément DOM dont l’affichage puisse être modifié.

```vue annotate="remove:2,3"
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

<span id="vue-valid-v-show-good"></span>

**Bon**

Appliquez l’expression de visibilité à un élément affiché, tel que `<div>`.

```vue annotate="add:2,3"
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) · [Toutes les règles](all.md)

### `vue/valid-v-slot`

Exiger des directives `v-slot` valides

[Mauvais](#vue-valid-v-slot-bad) · [Bon](#vue-valid-v-slot-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Consultez les [options typées et leurs valeurs par défaut](/rules/options.md).

**Configuration (Vite+)**

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

**Mauvais**

La directive de slot est placée sur un `<div>` natif ou entre en conflit avec d’autres déclarations de slots par défaut ou nommés.

```vue annotate="remove:2,3,4"
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

<span id="vue-valid-v-slot-good"></span>

**Bon**

Déclarez le slot par défaut d’un composant sur ce composant, ou son slot nommé sur un enfant `<template #header>`.

```vue annotate="add:2,3,4,5"
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) · [Toutes les règles](all.md)

### `vue/valid-v-text`

Exiger des directives `v-text` valides

[Mauvais](#vue-valid-v-text-bad) · [Bon](#vue-valid-v-text-good)

Gravité par défaut: `error`  
Préréglages: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-text` n’a pas d’expression de texte ou utilise un argument ou modificateur non pris en charge.

```vue annotate="remove:2,3,4"
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

<span id="vue-valid-v-text-good"></span>

**Bon**

`v-text="msg"` est syntaxiquement valide. La règle de style distincte `vue/no-v-text` peut toujours préférer l’interpolation.

```vue annotate="add:2"
<template>
<div v-text="msg"></div>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) · [Toutes les règles](all.md)

### `vue/warn-custom-block`

Signaler les blocs personnalisés dans les fichiers SFC

[Mauvais](#vue-warn-custom-block-bad) · [Bon](#vue-warn-custom-block-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

Le SFC contient un bloc personnalisé `<i18n>`, qui nécessite une intégration externe au-delà du traitement ordinaire de template, script et style.

```vue annotate="remove:1,2,3,4"
<i18n>
{ "en": { "hello": "Hello" } }
</i18n>

<template>
  <p>{{ hello }}</p>
</template>
```

<span id="vue-warn-custom-block-good"></span>

**Bon**

L’exemple utilise les blocs standard template et script-setup. Cet avertissement facultatif de portabilité ne signifie pas que tous les blocs personnalisés sont invalides en Vue.

```vue annotate="add:4,5,6,7"
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) · [Toutes les règles](all.md)

### `vue/warn-custom-directive`

Signaler les directives personnalisées nécessitant un enregistrement

[Mauvais](#vue-warn-custom-directive-bad) · [Bon](#vue-warn-custom-directive-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`, `opinionated`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

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

**Mauvais**

`v-focus`, `v-mask` et `v-click-outside` nécessitent des implémentations de directives propres au projet, signalées par cette convention facultative.

```vue annotate="remove:2,3,4"
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

<span id="vue-warn-custom-directive-good"></span>

**Bon**

L’exemple utilise les directives intégrées `v-if`, `v-model` et `v-on`. Une directive personnalisée correctement enregistrée peut rester valide en Vue lorsque cette politique est désactivée.

```vue annotate="add:2,3,4"
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) · [Toutes les règles](all.md)
