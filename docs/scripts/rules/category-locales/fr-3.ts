export const frenchRules3 = {
  "script/no-side-effects-in-computed-properties": [
    "Interdire les effets de bord dans les getters calculés de l’Options API",
    "`doubled` affecte une valeur à `this.count`, et `reversed` modifie `this.items` par `reverse()`. Les deux getters modifient l’état dont ils sont censés dériver leur valeur.",
    "`doubled` renvoie le résultat de la multiplication sans affectation. `reversed` copie le tableau avant de l’inverser, de sorte que le getter ne modifie pas l’état d’origine du composant.",
  ],
  "script/no-top-level-ref-in-script": [
    "Interdire ref/reactive au niveau supérieur pour éviter la contamination de l’état entre requêtes",
    "Le `<script>` ordinaire initialise `count` et `user` dans la portée du module. Pendant le SSR, ces objets d’état peuvent être partagés entre les instances du composant et les requêtes.",
    "La ref de setup est initialisée pour chaque instance du composant ; le script ordinaire ne conserve qu’une constante, une fonction produisant de l’état et une ref créée dans `setup()`. Aucun ne crée d’état réactif dans la portée du module ordinaire.",
  ],
  "script/no-unstable-nested-components": [
    "Interdire les définitions de composants dans les fonctions setup ou de rendu",
    "`defineComponent` s’exécute dans le `setup()` du parent et crée une nouvelle définition du composant `Child` chaque fois que ce setup s’exécute.",
    "La définition de `Child` passe dans la portée du module, et `setup()` renvoie cette définition existante au lieu de la recréer.",
  ],
  "script/no-unused-emit-declarations": [
    "Signaler les événements déclarés qui ne sont jamais émis",
    "`defineEmits` déclare à la fois `change` et `unused`, mais la fonction `emit` récupérée n’émet que l’événement littéral `change`.",
    "Supprimer `unused` fait correspondre la liste des événements déclarés à l’émission observée. L’exemple utilise une liaison emit récupérée et non transmise à l’extérieur, ce qui permet cette conclusion sur son utilisation locale.",
  ],
  "script/no-use-computed-property-like-method": [
    "Interdire d’appeler une propriété calculée de l’Options API comme une méthode",
    "`this.total()` appelle la valeur exposée par le getter calculé ; ce getter renvoie `3`, qui ne peut pas être appelé.",
    "`this.total` lit la valeur calculée sans parenthèses d’appel, de sorte que `log` affiche le nombre dérivé.",
  ],
  "script/no-with-defaults": [
    "Déconseiller withDefaults au profit des valeurs par défaut dans la déstructuration (Vue 3.5+)",
    "`withDefaults` enveloppe la déclaration typée des props uniquement pour fournir les valeurs par défaut de `count` et `name`, au lieu du style de valeurs par défaut dans la déstructuration de Vue 3.5+ préféré ici.",
    'Le motif de déstructuration place `count = 0` et `name = "Ada"` à côté de leurs liaisons et supprime l’enveloppe `withDefaults`.',
  ],
  "script/prefer-computed": [
    "Préférer computed() pour l’état réactif dérivé",
    "Le watcher ne fait que copier une valeur dérivée de `count` dans une seconde ref, `doubled` ; l’état dérivé est donc maintenu par synchronisation manuelle.",
    "`computed(() => count.value * 2)` exprime directement la dérivation et supprime à la fois la ref modifiable supplémentaire et le watcher qui la synchronise.",
  ],
  "script/prefer-define-options": [
    "Préférer defineOptions() à un &lt;script&gt; ordinaire qui ne définit que name/inheritAttrs",
    "La seule instruction significative du script ordinaire exporte un objet contenant uniquement `name` et `inheritAttrs` ; ces options peuvent être exprimées par `defineOptions`.",
    "La méthode `data()` montrée donne au script une véritable logique de l’Options API ; il échappe donc à la suggestion prudente de cette règle, limitée aux options seules. Cet exemple Bon démontre une exception autorisée ; la migration directe placerait `defineOptions({ name: 'MyComponent', inheritAttrs: false })` dans `<script setup>`.",
  ],
  "script/prefer-import-from-vue": [
    "Préférer les imports depuis 'vue' plutôt que depuis les packages internes",
    "`ref` et `h` sont importés depuis les packages internes `@vue/runtime-core` et `@vue/runtime-dom` plutôt que depuis le package public `vue`.",
    "Les deux utilitaires sont importés ensemble depuis `vue`, utilisant le point d’entrée public du package plutôt que l’un ou l’autre des packages internes.",
  ],
  "script/prefer-ref-over-reactive": [
    "Recommander ref() plutôt que reactive() pour gérer l’état",
    "L’état est créé avec `reactive`, contrairement à la préférence de cette règle de convention pour les refs. L’exemple illustre une préférence de style ; il ne s’agit pas d’un objet réactif intrinsèquement invalide.",
    "Les exemples créent aussi bien l’état scalaire que l’état objet avec `ref` ; les champs liés peuvent également être répartis dans des refs distinctes. Cela respecte la forme de création d’état préférée.",
  ],
  "script/prefer-use-attrs": [
    "Recommander useAttrs() plutôt que context.attrs",
    "`setup` obtient `attrs` en déstructurant son paramètre de contexte, ce que cette règle demande de remplacer par l’utilitaire de la Composition API.",
    "`useAttrs()` fournit `attrs` dans setup, en conservant la lecture de `attrs.class` sans dépendre du second paramètre de setup.",
  ],
  "script/prefer-use-id": [
    "Recommander useId() pour générer des identifiants uniques (Vue 3.5+)",
    "`id` contient `Math.random()` : l’identifiant généré pour l’input et le label peut donc différer entre les rendus serveur et client. Sa liaison dont le nom désigne un ID est le contexte de génération reconnu par la règle.",
    "`useId()` de Vue 3.5+ génère l’identifiant, et `:for` comme `:id` continuent de lire la même liaison au lieu de générer indépendamment des valeurs aléatoires.",
  ],
  "script/prefer-use-slots": [
    "Recommander useSlots() plutôt que context.slots",
    "`setup` déstructure `slots` depuis son argument de contexte, la forme d’accès que cette règle préfère remplacer.",
    "`useSlots()` récupère les slots dans setup, en conservant la fonction de rendu et son appel facultatif au slot par défaut sans paramètre de contexte.",
  ],
  "script/prefer-use-template-ref": [
    "Recommander useTemplateRef plutôt que ref pour les références de template (Vue 3.5+)",
    'La ref nullable `input` est associée au littéral `ref="input"` du template, ce qui l’identifie comme une référence d’élément plutôt que comme une donnée nullable ordinaire.',
    "`useTemplateRef<HTMLInputElement>('input')` de Vue 3.5+ rend cette référence de template explicite. `error = ref(null)`, sans association, reste une donnée ordinaire et est volontairement hors du champ de cette règle.",
  ],
  "script/require-default-prop": [
    "Exiger une valeur par défaut pour chaque prop facultative non booléenne",
    "`name` et `age` sont des props à l’exécution facultatives, non booléennes et sans valeur par défaut ; leurs valeurs en cas d’omission restent indéterminées.",
    "`name` reçoit `default: ''`. `enabled` utilise la valeur false implicite de Boolean, et `id`, obligatoire, n’a pas besoin de valeur de repli, ce qui illustre les deux exemptions.",
  ],
  "script/require-explicit-emits": [
    "Exiger la déclaration des événements émis dans defineEmits ou l’option emits",
    "La fonction emit récupérée émet `save`, mais `defineEmits([])` ne déclare pas cet événement.",
    'Ajouter `"save"` à la déclaration intègre l’événement littéral émis au contrat explicite d’événements du composant.',
  ],
  "script/require-explicit-slots": [
    "Exiger que les slots utilisés via useSlots() soient explicitement typés avec defineSlots&lt;...&gt;()",
    "La déclaration typée `defineProps<{ id: number }>()` établit une syntaxe TypeScript, mais setup utilise `useSlots()` sans déclaration `defineSlots`. La règle détecte donc des slots utilisés sans contrat de slots explicite.",
    "`defineSlots` déclare un slot `default` dont les props incluent `msg: string` ; `useSlots()` apparaît désormais aux côtés d’un contrat de slots explicitement typé.",
  ],
  "script/require-function-return-type": [
    "Exiger des annotations de type de retour sur les fonctions",
    "`add` et `greet` annotent tous deux leurs paramètres mais omettent une annotation de type de retour ; les retours inférés ne satisfont pas cette convention d’annotation explicite.",
    "`add` déclare `: number`, et `greet` déclare `: string`, rendant leurs contrats de retour explicites sans changer leurs corps.",
  ],
  "script/require-prop-type-constructor": [
    "Exiger que les valeurs `type` des props soient des constructeurs plutôt que des chaînes littérales",
    'Les déclarations de props utilisent les chaînes `"String"` et `"Number"` comme types à l’exécution, y compris dans le tableau de constructeurs. Ces chaînes ne sont pas des fonctions constructeurs.',
    "Les déclarations utilisent les véritables identifiants `String` et `Number`, y compris dans le tableau d’union `[String, Number]`.",
  ],
  "script/require-prop-types": [
    "Exiger que chaque prop déclare un type",
    "L’entrée du tableau ne déclare que le nom `status` ; la valeur `null` et le descripteur vide ne déclarent pas non plus de type de prop à l’exécution.",
    "`status: String` fournit un constructeur sous forme abrégée, et `other` fournit `type: Number` dans son descripteur. Les deux props possèdent désormais des déclarations de type.",
  ],
  "script/require-symbol-provide": [
    "Recommander Symbol comme clé d’injection pour provide/inject",
    "`provide` et `inject` utilisent des clés sous forme de chaînes littérales telles que `'user'` et `'theme'`, qui peuvent entrer en conflit avec un autre fournisseur utilisant la même graphie.",
    "La clé partagée `UserKey` est créée avec `Symbol` et annotée comme `InjectionKey<User>` ; les deux appels passent cette clé au lieu d’une chaîne littérale.",
  ],
  "script/require-typed-object-prop": [
    "Exiger un type explicite sur une prop dont le type à l’exécution est `Object` ou `Array`",
    "Les constructeurs `Object` et `Array` employés seuls ne décrivent que des catégories générales à l’exécution ; ni `user` ni la structure des éléments de `items` n’a donc de type statique explicite.",
    "`PropType<User>` et `PropType<User[]>` ajoutent les types de l’objet et des éléments tout en conservant les mêmes constructeurs à l’exécution.",
  ],
  "script/require-typed-ref": [
    "Exiger un argument de type explicite sur un ref() initialisé sans valeur, avec null ou avec undefined",
    "Les appels à `ref` importé n’ont ni argument de type ni valeur initiale utile : l’absence d’argument, `null` et `undefined` ne permettent pas d’inférer le type de valeur futur souhaité.",
    "Des arguments de type explicites décrivent les refs de chaîne et de User nullable. `ref(0)` possède déjà une valeur initiale numérique concrète et peut s’appuyer sur l’inférence.",
  ],
  "script/require-valid-default-prop": [
    "Exiger que la valeur par défaut d’une prop soit valide pour son type déclaré",
    "Les props Number et Boolean reçoivent des valeurs scalaires par défaut incompatibles, et les props Array et Object utilisent des valeurs littérales partagées au lieu de fonctions de création.",
    "Les valeurs scalaires par défaut deviennent `0` et `false` ; les valeurs par défaut du tableau et de l’objet deviennent des fonctions renvoyant de nouvelles valeurs. L’exemple `[String, Number]` accepte sa valeur par défaut de type chaîne, car elle correspond à l’un des types déclarés.",
  ],
  "script/return-in-computed-property": [
    "Exiger une valeur de retour dans chaque getter calculé",
    "Le getter calculé dont le corps est un bloc évalue `1 + 2` mais ne le renvoie jamais, laissant la valeur calculée à undefined.",
    "`return 1 + 2` transforme l’expression en valeur renvoyée par le getter. La règle recherche un return renvoyant une valeur dans le getter lui-même, et non une simple instruction d’expression.",
  ],
  "script/return-in-emits-validator": [
    "Exiger une valeur de retour dans chaque validateur emits de l’Options API",
    "Le validateur `submit` journalise la charge utile mais ne renvoie aucun résultat de validation ; son corps de bloc produit donc undefined.",
    "`return payload != null` fournit un résultat de validation booléen pour la charge utile soumise au lieu de se terminer sans valeur de retour.",
  ],
  "script/valid-define-emits": [
    "Imposer une utilisation valide de defineEmits() (pas d’arguments de type et d’exécution combinés, pas de références locales, un seul appel)",
    'Le même appel `defineEmits` fournit à la fois un argument de type et le tableau à l’exécution `["save"]`, mélangeant deux déclarations mutuellement exclusives.',
    "Supprimer l’argument d’exécution laisse une seule déclaration d’événement typée pour `save`.",
  ],
  "script/valid-define-options": [
    "Imposer une utilisation valide de defineOptions() (un seul argument objet, sans props/emits/expose/slots)",
    "Le premier appel place la déclaration dédiée `props` dans `defineOptions` ; les appels suivants répètent également la macro et incluent un argument qui n’est pas un objet. Ils illustrent les contraintes sur les formes interdites et les appels répétés.",
    "Un seul appel `defineOptions` reçoit un objet contenant uniquement les options ordinaires prises en charge `name` et `inheritAttrs`.",
  ],
  "script/valid-define-props": [
    "Imposer une utilisation valide de defineProps() (un seul appel, pas d’arguments de type et d’exécution combinés, pas de références locales)",
    "Le même appel `defineProps` fournit à la fois `{ title: string }` comme argument de type et `{ title: String }` comme argument d’exécution, ce que le compilateur n’autorise pas conjointement.",
    "Supprimer l’objet d’exécution laisse une déclaration typée unique pour `title` au lieu de combiner les deux formes de déclaration.",
  ],
  "script/valid-next-tick": [
    "Exiger que le résultat d’un appel nextTick() soit attendu, chaîné ou associé à un callback",
    "L’appel à `nextTick()` importé est une expression seule sans callback ; la Promise renvoyée est donc ignorée et aucun traitement n’attend l’application des mises à jour du DOM.",
    "`await nextTick()` consomme la Promise et attend explicitement la prochaine mise à jour du DOM avant que le code setup suivant ne continue.",
  ],
  "ssr/no-browser-globals-in-ssr": [
    "Interdire les variables globales propres au navigateur dans un contexte SSR",
    "Setup lit immédiatement `window.innerWidth`, alors que `window` n’existe pas lorsque le composant s’exécute sur le serveur.",
    "La largeur initiale est une valeur de ref utilisable sur le serveur, et l’accès au navigateur est déplacé dans `onMounted`, qui s’exécute sur le client plutôt que pendant le setup SSR.",
  ],
  "ssr/no-hydration-mismatch": [
    "Interdire les valeurs non déterministes qui causent des divergences d’hydratation",
    "Le template évalue `Math.random()` pendant le rendu ; le serveur et le client peuvent donc produire des textes différents pour le même paragraphe.",
    'Le paragraphe affiche l’état stable `seed` au lieu d’un nouveau résultat aléatoire. Dans cet exemple de style Nuxt, `useState` fournit l’état partagé et la valeur d’initialisation est la constante `"stable"`.',
  ],
  "type/no-floating-promises": [
    "Interdire les Promises laissées sans traitement",
    "La fonction asynchrone `save` renvoie une Promise, mais l’appel isolé `save()` ne l’attend ni ne la renvoie, et ne marque pas explicitement son abandon volontaire.",
    "`void save()` marque explicitement l’intention de lancer l’opération sans attendre son résultat, acceptée par cette règle. Il s’agit d’un marqueur explicite d’abandon, et non d’un gestionnaire de rejet.",
  ],
  "type/no-reactivity-loss": [
    "Interdire les instantanés ordinaires de valeurs réactives lors des affectations et des appels",
    "`const count = state.count` prend un simple instantané numérique de la propriété réactive ; les mises à jour ultérieures de `state.count` ne sont donc pas répercutées dans cette liaison.",
    '`toRef(state, "count")` maintient `count` relié à la propriété réactive d’origine plutôt que de copier sa valeur primitive actuelle.',
  ],
  "type/no-unsafe-template-binding": [
    "Interdire les liaisons de template dont le type résolu est non sûr",
    "La valeur interpolée `value` est explicitement typée comme `any` ; le vérificateur ne peut donc pas attribuer à la liaison du template un type concret sûr.",
    "Changer l’annotation en `string` donne à la même interpolation un type concret vérifiable sans changer la valeur affichée.",
  ],
} satisfies Record<string, readonly [string, string, string]>;
