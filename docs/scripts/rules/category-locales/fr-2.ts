export const frenchRules2 = {
  "petite-vue/valid-v-scope": [
    "Exiger que v-scope reçoive un objet littéral",
    "Les quatre valeurs non vides de `v-scope` sont un identifiant, un appel, une expression arithmétique et un nombre ; aucune n’est analysée comme un objet littéral.",
    "Un `v-scope` sans valeur utilise la portée racine. Les autres valeurs sont des objets littéraux, y compris l’objet entre parenthèses, que la règle accepte.",
  ],
  "script/component-options-name-casing": [
    "Imposer PascalCase à l’option `name` du composant",
    "L’option de composant `name: 'my-component'` utilise kebab-case, alors que cette règle exige un nom de composant littéral en PascalCase.",
    "`MyComponent` commence par une majuscule et ne contient que des caractères alphanumériques, ce qui satisfait le contrôle du nom.",
  ],
  "script/custom-event-name-casing": [
    "Imposer camelCase aux noms des événements personnalisés émis",
    "La chaîne émise `my-event` contient un tiret et enfreint la convention camelCase par défaut pour les noms d’événements.",
    "La déclaration et l’appel utilisent tous deux `myEvent`, ce qui maintient la correspondance entre le nom de l’événement et son émission tout en respectant la convention de casse par défaut. Une convention kebab-case configurée impose une autre forme.",
  ],
  "script/define-emits-declaration": [
    "Imposer la forme typée defineEmits&lt;{}&gt;() à la place de la forme à l’exécution sous forme de tableau",
    '`defineEmits(["change"])` utilise une déclaration par tableau à l’exécution ; cette règle de style préfère une déclaration typée.',
    '`defineEmits<{ change: [id: number] }>()` place la déclaration de l’événement dans un argument de type et décrit explicitement la charge utile numérique utilisée par `emit("change", 1)`.',
  ],
  "script/define-macros-order": [
    "Imposer un ordre cohérent aux macros du compilateur Vue dans &lt;script setup&gt;",
    "`defineProps` apparaît avant `defineModel`, alors que `defineModel` occupe une position antérieure dans l’ordre canonique des macros.",
    "Les déclarations suivent exactement la séquence `defineOptions`, `defineModel`, `defineProps`, `defineEmits`, `defineSlots`, avant les instructions d’exécution sans rapport avec elles.",
  ],
  "script/define-props-declaration": [
    "Imposer la forme typée defineProps&lt;{ ... }&gt;() à la place de la forme à l’exécution sous forme d’objet",
    "`defineProps({ title: String })` fournit un objet à l’exécution, ce qui va à l’encontre de la préférence de cette règle pour les props typées.",
    "`defineProps<{ title: string }>()` déclare `title` dans l’argument de type et conserve l’accès `props.title` sans argument de déclaration à l’exécution.",
  ],
  "script/define-props-destructuring": [
    "Imposer un style cohérent de déstructuration de defineProps dans &lt;script setup&gt;",
    "`defineProps` est affecté à la seule liaison `props` au lieu d’être déstructuré, contrairement à la préférence par défaut pour la déstructuration.",
    "Le motif objet lie directement `foo` et `bar` et donne une valeur par défaut à `bar`, qui est facultatif. Cela repose sur la déstructuration réactive des props de Vue 3.5+ ; le mode configurable `never` préfère la forme opposée.",
  ],
  "script/no-arrow-functions-in-watch": [
    "Interdire les fonctions fléchées comme gestionnaires watch de l’Options API",
    "Le watcher `value` de l’Options API et le gestionnaire imbriqué `other.handler` sont des fonctions fléchées. Une fonction fléchée capture le `this` du contexte environnant au lieu de recevoir l’instance du composant.",
    "Les deux gestionnaires deviennent des méthodes ordinaires, ce qui permet à Vue de lier `this` au composant. L’option du watcher `deep: true` reste compatible avec la forme objet.",
  ],
  "script/no-async-in-computed": [
    "Interdire les fonctions asynchrones dans les propriétés calculées",
    "Le getter `computed` est `async` : la récupération produit donc une Promise au lieu d’une valeur calculée dérivée de manière synchrone.",
    "La récupération asynchrone est déplacée dans `watch`, qui stocke son résultat dans `data.value`. Le nettoyage annule l’ancienne requête et empêche un callback inactif d’écrire un résultat périmé ; aucun getter calculé asynchrone ne subsiste.",
  ],
  "script/no-boolean-default": [
    "Interdire une valeur par défaut sur une prop Boolean",
    "`disabled` et `checked` déclarent tous deux un `default` sur une prop dont le seul constructeur est `Boolean` ; la règle rejette même une valeur par défaut explicite `false`.",
    "Les props exclusivement booléennes omettent `default` et utilisent la valeur false implicite de Vue. L’union `[Boolean, String]` et la prop Number montrent que ce contrôle se limite au constructeur `Boolean` employé seul.",
  ],
  "script/no-deep-destructure-in-props": [
    "Interdire la déstructuration profondément imbriquée dans defineProps",
    "Le motif de liaison descend dans `user` pour déstructurer `name`, dépassant la profondeur limitée de déstructuration des props autorisée par défaut.",
    "L’objet des props reste intact, et un getter calculé lit `props.user.name`. L’accès imbriqué reste explicite sans motif de liaison profondément imbriqué.",
  ],
  "script/no-deprecated-data-object-declaration": [
    "Interdire un objet littéral comme option data du composant (Vue 3 exige une fonction)",
    "L’option `data` de l’Options API est un objet littéral, une forme de Vue 2 que Vue 3 n’accepte plus.",
    "`data()` renvoie un nouvel objet `{ count: 0 }`, fournissant la déclaration de données par fonction exigée par Vue 3.",
  ],
  "script/no-deprecated-destroyed-lifecycle": [
    "Interdire les hooks de cycle de vie dépréciés destroyed et beforeDestroy",
    "`beforeDestroy` est l’option de cycle de vie de Vue 2 supprimée, utilisée ici pour nettoyer le minuteur.",
    "Renommer le hook en `beforeUnmount` conserve le corps du nettoyage sous son nom de cycle de vie Vue 3.",
  ],
  "script/no-deprecated-dollar-listeners-api": [
    "Interdire la propriété d’instance $listeners supprimée dans Vue 3 (fusionnée dans $attrs)",
    "Les accès aux membres et la référence passée directement en argument utilisent tous `$listeners`, que Vue 3 a supprimé après avoir fusionné les écouteurs dans les attributs.",
    "Les accès passent à `this.$attrs` et à `ctx.attrs` dans le contexte setup. Ils remplacent l’API supprimée des écouteurs ; les objets récepteurs illustrés doivent exister dans le contexte environnant du composant.",
  ],
  "script/no-deprecated-dollar-scopedslots-api": [
    "Interdire la propriété d’instance $scopedSlots supprimée dans Vue 3 (utiliser $slots)",
    "`this.$scopedSlots`, `ctx.$scopedSlots` et la référence directe `$scopedSlots` utilisent l’API des slots à portée de Vue 2, supprimée dans Vue 3.",
    "Remplacer `$scopedSlots` par `$slots` utilise l’API unifiée des slots. L’exemple supprime l’écriture dépréciée sans établir un contexte setup pour les objets récepteurs.",
  ],
  "script/no-deprecated-events-api": [
    "Interdire l’API d’événements de Vue 2 supprimée ($on / $off / $once)",
    "Les appels `$on`, `$once` et `$off` utilisent les méthodes de bus d’événements de l’instance supprimées dans Vue 3.",
    "`$emit` reste valide, tandis que l’abonnement au bus d’événements passe à la méthode `on` de l’émetteur externe. La correction sépare l’émission destinée au parent du bus d’événements externe.",
  ],
  "script/no-deprecated-props-default-this": [
    "Interdire `this` dans une fonction de valeur par défaut ou de validation de prop (supprimé dans Vue 3)",
    "La valeur par défaut et le validateur de la prop lisent `this`, mais ces fonctions ne peuvent pas s’appuyer sur l’instance du composant dans Vue 3.",
    "La fonction de valeur par défaut lit `props.baseSize` dans son argument, et le validateur teste son argument `value`. Tous deux cessent de dépendre d’un récepteur d’instance indisponible.",
  ],
  "script/no-dupe-keys": [
    "Interdire les clés dupliquées entre props/data/computed/methods/setup/inject de l’Options API",
    "`foo` est déclaré à la fois dans props et data, et `bar` à la fois dans computed et methods. Ces déclarations se disputent les mêmes clés de l’instance du composant.",
    "Les déclarations de prop, de données et de propriété calculée utilisent des noms distincts (`foo`, `bar` et `baz`), éliminant les deux collisions entre options.",
  ],
  "script/no-duplicate-attr-inheritance": [
    "Signaler un composant qui applique deux fois ses attributs transmis automatiquement",
    "Les valeurs explicites `inheritAttrs: true` répètent le comportement par défaut de Vue. Cette règle signale ce littéral redondant même si aucun déploiement de `$attrs` sur la racine n’est montré.",
    "`inheritAttrs: false` exprime une véritable désactivation, tandis que l’objet d’options vide laisse l’héritage par défaut implicite. Aucun ne répète la valeur redondante `true`.",
  ],
  "script/no-export-in-script-setup": [
    "Interdire les instructions export dans &lt;script setup&gt;",
    "`export const count` tente d’exposer un export de module depuis `<script setup>`, où les exports à l’exécution sont interdits.",
    "Supprimer `export` conserve `count` comme liaison setup plutôt que comme export de module.",
  ],
  "script/no-get-current-instance": [
    "Interdire getCurrentInstance() en mode Vapor (renvoie null)",
    "Le setup marqué Vapor importe et appelle `getCurrentInstance`, s’appuyant sur une API d’instance que cette règle interdit pour les composants destinés à Vapor.",
    '`inject("app-config")` obtient la configuration explicitement fournie sans importer ni appeler `getCurrentInstance`.',
  ],
  "script/no-import-compiler-macros": [
    "Interdire l’import des macros du compilateur Vue importées automatiquement",
    "L’import depuis `vue` inclut `defineProps` et `defineEmits`, alors que ces macros du compilateur sont directement disponibles dans `<script setup>`.",
    "Supprimer les imports des macros conserve les deux appels typés ; aucune déclaration ne nécessite d’import à l’exécution.",
  ],
  "script/no-internal-imports": [
    "Interdire les imports depuis les modules internes de Vue",
    "Les deux imports ciblent des fichiers internes `dist` plutôt que le point d’entrée public du package Vue, couplant le composant aux chemins des fichiers de build.",
    "Importer les utilitaires nécessaires depuis `vue` supprime la dépendance aux emplacements des fichiers de distribution internes.",
  ],
  "script/no-multiple-slot-args": [
    "Interdire de passer plusieurs arguments à un appel de fonction de slot à portée",
    "Les appels de slots passent plusieurs arguments positionnels ou déploient une liste d’arguments inconnue. Les slots Vue reçoivent un seul objet de props, et non une liste de paramètres positionnels.",
    "`{ foo, bar }` regroupe les données dans un seul argument ; `slotProps` et l’appel sans argument respectent également la forme d’appel de slot prise en charge.",
  ],
  "script/no-next-tick": [
    "Interdire l’utilisation de nextTick() dans les composants destinés à Vapor",
    "Le composant destiné à Vapor importe `nextTick` et attend son résultat, introduisant la dépendance à la planification des mises à jour du DOM que cette règle de migration rejette.",
    "L’input est obtenu par `useTemplateRef` et reçoit le focus dans `onMounted`. Ce point de montage explicite remplace la dépendance de l’exemple à `nextTick`.",
  ],
  "script/no-options-api": [
    "Interdire les formes de l’Options API en mode Vapor",
    "L’objet exporté par défaut déclare `data()` de l’Options API, une forme d’option de composant interdite par cette règle.",
    "L’état du composant devient une `ref` de la Composition API dans le `<script setup>` Vapor, supprimant l’objet de l’Options API et son option `data`.",
  ],
  "script/no-potential-component-option-typo": [
    "Signaler les fautes de frappe probables dans les noms d’options de composant de l’Options API",
    "L’option est écrite `method`, à une modification près de l’option reconnue `methods` ; Vue ne la traiterait pas comme la déclaration de méthodes souhaitée.",
    "Changer la clé en `methods` place `save()` sous l’option de composant reconnue.",
  ],
  "script/no-reactive-destructure": [
    "Interdire la déstructuration d’objets réactifs qui fait perdre la réactivité",
    "`const { count, name } = state` copie les propriétés primitives hors de l’objet `reactive`, perdant leur lien avec les modifications ultérieures des propriétés.",
    "Déstructurer `toRefs(state)` crée des refs pour `count` et `name`, en maintenant chaque liaison reliée à la propriété réactive d’origine.",
  ],
  "script/no-ref-as-operand": [
    "Exiger l’accès via `.value` aux variables liées à une ref lorsqu’elles servent d’opérande",
    "`count + 1` utilise l’objet ref lui-même comme opérande arithmétique au lieu du nombre qu’il contient.",
    "`count.value + 1` lit le nombre contenu avant d’ajouter un ; l’arithmétique dans le script exige cet accès explicite à la ref.",
  ],
  "script/no-required-prop-with-default": [
    "Interdire une prop qui possède à la fois required: true et une valeur par défaut",
    '`title` est à la fois obligatoire et doté de la valeur de repli `"Untitled"`, combinant un contrat d’entrée obligatoire avec une valeur par défaut prévue pour une entrée manquante.',
    'Supprimer `required: true` rend `title` facultatif et conserve `"Untitled"` comme valeur de repli cohérente.',
  ],
  "script/no-reserved-identifiers": [
    "Interdire les identifiants réservés du compilateur Vue",
    "Les liaisons `__props`, `__emit` et `__sfc__` utilisent des identifiants réservés au code généré par le compilateur Vue.",
    "Les noms ordinaires `props`, `emit` et `componentData` évitent ces identifiants générés tout en conservant les déclarations de props et d’événements émis.",
  ],
  "script/no-reserved-keys": [
    "Interdire les noms réservés par Vue comme clés de props/data/computed/methods/setup/inject de l’Options API",
    "La clé de données renvoyée `$el` entre en conflit avec une propriété intégrée de l’instance du composant Vue et utilise également le préfixe réservé `$`.",
    "Renommer les données de l’application en `elementLabel` évite l’API intégrée de l’instance et le préfixe réservé.",
  ],
  "script/no-reserved-props": [
    "Interdire les noms réservés dans la déclaration des props d’un composant",
    "`ref` et `$foo` dans la forme objet, ainsi que `key` dans la forme tableau, sont des noms de props réservés. `ref` et `key` sont des mécanismes du framework, et les noms préfixés par `$` sont rejetés.",
    "Les noms de props ordinaires `name` et `refValue` évitent les noms réservés tant par leur graphie que par leur préfixe.",
  ],
  "script/no-restricted-globals": [
    "Interdire les références aux variables globales de l’environnement d’exécution qui doivent passer par une couche d’encapsulation typée",
    "L’exemple lit directement les variables globales restreintes par défaut `process`, `localStorage` et `sessionStorage`, en contournant les utilitaires explicites de configuration et de stockage du projet.",
    "`useFeatureFlag`, `authStorage.read` et `viewStorage.write` suppriment ces références directes aux variables globales restreintes. L’accès restant à `window.scrollY` ne fait pas partie des restrictions par défaut de cette règle ; la sécurité SSR est une question distincte.",
  ],
  "script/no-restricted-members": [
    "Interdire les accès aux membres object.property configurés par le projet",
    'Lorsque `{ object: "window", property: "localStorage" }` est configuré dans `ruleOptions`, `window.localStorage` accède à la paire objet/membre interdite. Cette règle n’interdit aucun membre par défaut.',
    '`authStorage.read("token")` délègue la lecture à l’utilitaire de stockage de l’application et n’accède plus au membre configuré `window.localStorage`.',
  ],
} satisfies Record<string, readonly [string, string, string]>;
