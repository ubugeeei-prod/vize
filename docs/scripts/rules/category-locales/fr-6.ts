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
} satisfies Record<string, readonly [string, string, string]>;
