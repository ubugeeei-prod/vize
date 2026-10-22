export const frenchRules6 = {
  "vize:croquis/cf/watcheffect-async": [
    "`watchEffect` lance une tâche asynchrone et ne peut pas nettoyer l’exécution précédente.",
    "Le `watchEffect` asynchrone mêle la collecte implicite des dépendances à une requête dont il attend le résultat, sans garde contre l’invalidation.",
    "Un `watch(() => props.query, ...)` explicite déclare la source, enregistre le nettoyage de la requête et refuse une réponse périmée après invalidation.",
  ],
  "vize:croquis/cf/watcher-outside-setup": [
    "`watch` ou `watchEffect` est appelé en dehors de `setup`.",
    "L’observateur est créé au chargement du module, en dehors du setup de chacune des deux instances d’Observer, et les deux instances partagent ses refs. Il n’est pas automatiquement arrêté lorsqu’une instance donnée d’Observer est démontée.",
    "Chaque appel synchrone de setup crée ses propres refs et son propre observateur dans `useObserver`. Vue associe cet observateur à la durée de vie du composant appelant.",
  ],
  "vue/cross-file-attrs-fallthrough": [
    "Un parent transmet des attributs à un enfant résolu dont la racine ne peut pas en hériter et qui n’utilise pas explicitement $attrs.",
    'Le parent transmet `class="notice"` à un enfant résolu constitué d’un fragment, qui n’a aucune cible automatique pour les attributs et ne lit jamais `$attrs`.',
    "L’enfant choisit `<main>` comme cible en y liant `$attrs` ; son frère `<aside>` reste distinct.",
  ],
  "script/no-expose-after-await": [
    "Interdire expose() et defineExpose() après un await",
    "`defineExpose()` s’exécute après un `await` de premier niveau, alors qu’aucune instance de composant n’est courante ; rien n’est donc exposé.",
    "`defineExpose()` s’exécute avant le premier `await`, tant que l’instance du composant est courante.",
  ],
  "script/no-lifecycle-after-await": [
    "Interdire l’enregistrement de hooks de cycle de vie après un await dans setup()",
    "`setup()` appelle `onMounted()` après `await loadData()` ; sans instance de composant courante à ce moment, le hook n’est jamais enregistré.",
    "`onMounted()` est enregistré avant le premier `await`, tant que `setup()` s’exécute de manière synchrone pour le composant.",
  ],
  "script/no-watch-after-await": [
    "Interdire les observateurs créés après un await dans setup() sans conserver leur fonction d’arrêt",
    "`watch()` est créé après `await loadData()` et sa fonction d’arrêt est ignorée ; l’observateur n’est donc jamais arrêté avec le composant.",
    "L’observateur est créé avant le premier `await`, donc Vue l’arrête lorsque le composant est démonté.",
  ],
  "script/no-computed-properties-in-data": [
    "Interdire l’usage d’une propriété calculée dans la fonction data du composant",
    "`data()` renvoie `this.doubled` alors que `doubled` est encore une propriété calculée, lue avant son initialisation.",
    "`data()` renvoie une valeur simple, et `doubled` lit `this.value` après l’initialisation.",
  ],
  "script/no-deprecated-delete-set": [
    "Interdire $set et $delete de Vue 2, retirés dans Vue 3",
    "`created()` appelle `this.$set`, retiré, pour ajouter `ready`.",
    "Le même champ est affecté directement avec `this.item.ready = true`.",
  ],
  "script/no-deprecated-model-definition": [
    "Interdire l’option model de Vue 2, retirée dans Vue 3",
    "Le composant déclare encore l’option Vue 2 `model` avec `prop` et `event`.",
    "Le composant déclare la prop `modelValue` et l’événement `update:modelValue`.",
  ],
  "script/no-deprecated-vue-config-keycodes": [
    "Interdire Vue.config.keyCodes, retiré dans Vue 3",
    "`Vue.config.keyCodes.enter` affecte un alias de code de touche retiré.",
    "Le code de touche est une constante ordinaire, `const enter = 13`.",
  ],
  "script/require-render-return": [
    "Exiger que la fonction render du composant renvoie une valeur",
    "La fonction `render` lit `this.count` dans une branche et ne renvoie rien.",
    "La fonction `render` renvoie `this.count`.",
  ],
  "script/require-slots-as-functions": [
    "Exiger que les propriétés de $slots soient appelées comme des fonctions",
    "`render` renvoie `this.$slots.default` sans l’appeler, ce qui était le tableau de vnodes de Vue 2.",
    "`render` appelle `this.$slots.default()` et utilise le vnode renvoyé.",
  ],
  "script/one-component-per-file": [
    "Interdire plus d’une définition de composant dans un fichier",
    "Le fichier exporte `Parent` et définit aussi un composant `Child` dans `components`.",
    "Le fichier ne définit que `Parent`.",
  ],
  "script/order-in-components": [
    "Imposer l’ordre par défaut des propriétés des options de composant",
    "`data` est écrit avant `props`, donc un groupe plus tardif précède un groupe plus précoce.",
    "`props` précède `data`, ce qui suit l’ordre des groupes par défaut.",
  ],
} satisfies Record<string, readonly [string, string, string]>;
