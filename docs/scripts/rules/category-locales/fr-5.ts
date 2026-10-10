export const frenchRules5 = {
  "vize:croquis/cf/missing-suspense": [
    "Une dépendance asynchrone est utilisée en dehors d’une frontière Suspense.",
    "`AsyncCard` contient un await au niveau supérieur, ce qui rend son setup asynchrone, mais App le rend sans frontière Suspense pour coordonner cette dépendance.",
    "App enveloppe l’enfant asynchrone dans `Suspense` et fournit un contenu de secours pendant le chargement, jusqu’à ce que le setup de l’enfant soit terminé.",
  ],
  "vize:croquis/cf/module-scope-reactive": [
    "Un état réactif est créé au niveau du module et partagé par tous les appelants.",
    "Le module initialise `count` une seule fois et les deux instances de Counter reçoivent la même ref. Cliquer sur l’une modifie les deux compteurs, alors que cet exemple prévoit un état indépendant pour chaque instance.",
    "Créer la ref dans `createCounter` donne à chaque appel synchrone de setup un objet d’état distinct ; chaque bouton possède donc son propre compteur.",
  ],
  "vize:croquis/cf/multi-root-attrs": [
    "Un composant à plusieurs racines reçoit des attributs sans avoir d’endroit où les placer.",
    "L’enfant a pour racines `<main>` et `<aside>` ; Vue ne dispose donc pas d’une racine unique pouvant recevoir automatiquement la classe du parent.",
    "Transmettre explicitement `$attrs` à `<main>` tout en conservant la seconde racine.",
  ],
  "vize:croquis/cf/mutated-after-escape": [
    "Un objet réactif est modifié après avoir échappé à son propriétaire.",
    "L’archive conserve le même objet que celui passé à `publish`. Le propriétaire modifie ensuite le nom de cet objet, remplaçant rétroactivement par Grace le nom dans l’enregistrement censé être historique. Le paramètre Readonly de TypeScript ne copie pas l’objet.",
    "Publier une copie ordinaire sépare l’enregistrement archivé d’Ada des modifications ultérieures du profil réactif. La politique d’instantanés de l’archive est désormais respectée.",
  ],
  "vize:croquis/cf/non-reactive-provide": [
    "Une valeur fournie n’est pas réactive ; les descendants ne verront donc pas les mises à jour.",
    "`ThemeProvider.vue` fournit un objet ordinaire. Modifier les champs de cet objet ne donne pas de dépendance réactive Vue au consommateur qui l’injecte.",
    "Le fournisseur enveloppe le thème dans `ref` ; la même référence injectée peut suivre les changements ultérieurs.",
  ],
  "vize:croquis/cf/non-unique-id": [
    "L’identifiant d’un élément dans une boucle n’est pas unique pour chaque entrée.",
    "Chaque itération de `v-for` rend le même identifiant littéral `result-title` ; la clé de la boucle ne rend pas les identifiants DOM uniques.",
    "L’identifiant du titre inclut l’identifiant stable du résultat, produisant un identifiant distinct dans le document pour chaque entrée.",
  ],
  "vize:croquis/cf/object-identity-comparison": [
    "Un objet réactif est comparé par identité, laquelle change lorsque l’enveloppe est retirée.",
    "`proxy === raw` compare l’identité de l’enveloppe ; le résultat est donc faux même si les deux représentent le même enregistrement utilisateur. L’application visait l’identité de l’enregistrement, et non celle de l’enveloppe de l’objet.",
    "Comparer le `id` stable de l’enregistrement répond à la question voulue sans dépendre du fait que l’objet soit brut ou enveloppé dans un proxy.",
  ],
  "vize:croquis/cf/pinia-getter": [
    "Un getter Pinia est lu sans `storeToRefs` ; il ne restera donc pas réactif.",
    "`const doubled = store.doubled` copie le nombre actuel du getter pendant setup. Le nombre copié ne suit pas les mises à jour ultérieures de `store.count`.",
    "`storeToRefs(store)` fournit une ref de getter réactive qui peut être déstructurée et déballée par le template tout en restant liée au store.",
  ],
  "vize:croquis/cf/prop-type-mismatch": [
    "La valeur d’une prop transmise ne correspond pas au type déclaré.",
    "Le parent passe l’expression numérique `42` à la prop `title: string` de l’enfant résolu.",
    'Le littéral `title="Hello"` fournit une chaîne correspondant à la déclaration de l’enfant.',
  ],
  "vize:croquis/cf/provide-inject-type": [
    "Une valeur fournie et son injection n’ont pas le même type.",
    "Le fournisseur annote explicitement `title` avec le type `string`, tandis que le descendant demande `inject<number>` pour la même clé.",
    "L’appel explicite `inject<string>` du consommateur correspond à l’annotation du fournisseur. Conserver `as string` : ce producteur compare les annotations explicites, et non les types littéraux inférés.",
  ],
  "vize:croquis/cf/provide-without-symbol": [
    "`provide` utilise une clé ordinaire au lieu d’un symbole `InjectionKey`.",
    'Les deux composants utilisent la chaîne `"theme"` ; des fonctionnalités sans rapport peuvent accidentellement réutiliser cette clé.',
    "Exporter un seul symbole typé `ThemeKey` et importer cette même valeur aux endroits où provide et inject sont appelés. Créer des symboles distincts avec la même description ne les relierait pas.",
  ],
  "vize:croquis/cf/reactive-export": [
    "Un état réactif est exporté depuis le module.",
    "Le module exporte un seul objet réactif initialisé ; chaque module qui l’importe reçoit donc le même décompte. Dans un module SSR partagé entre les requêtes, cela compromet l’isolation de l’état par instance ou par requête prévue par l’exemple.",
    "Le module exporte une fabrique et App l’appelle dans setup. Chaque instance obtient un nouveau décompte réactif au lieu du singleton exporté.",
  ],
  "vize:croquis/cf/reactivity-outside-setup": [
    "Une API réactive est appelée en dehors de `setup`.",
    "Les deux API réactives s’exécutent pendant le chargement du module. Les deux instances de Counter partagent donc une seule ref et une seule valeur calculée, malgré l’intention d’avoir des compteurs indépendants.",
    "`useCounter` crée la ref et la valeur calculée de manière synchrone dans chaque appel de setup d’un composant, donnant à chaque widget son propre état et sa propre dérivation suivie.",
  ],
  "vize:croquis/cf/reassignment-breaks-reactivity": [
    "Réaffecter une liaison réactive la remplace par une valeur ordinaire.",
    "L’enfant crée une ref de prop, puis écrase la variable avec `props.user`, supprimant le lien fourni par cette ref.",
    "Conserver le `toRef` dans une liaison `const` et supprimer la réaffectation qui le remplace.",
  ],
  "vize:croquis/cf/reference-escapes-scope": [
    "Une référence réactive échappe à la portée qui gère sa durée de vie.",
    "Le cache au niveau du processus conserve la ref active du décompte du composant. Il peut maintenir l’état de cette instance accessible après le démontage et observer des modifications ultérieures, alors que ce cache est censé stocker un instantané.",
    "Le cache reçoit le nombre ordinaire actuel ; il conserve donc un instantané sans retenir la ref appartenant au composant.",
  ],
  "vize:croquis/cf/setup-context-violation": [
    "Le contexte setup est utilisé d’une manière que Vue n’autorise pas.",
    "`ref(0)` est créé au niveau du module d’un script normal, en dehors du contexte setup propre à chaque instance représenté par ce scénario d’analyse.",
    "Déplacer la liaison dans script setup, où chaque instance de composant possède son décompte et où le template peut le lire.",
  ],
  "vize:croquis/cf/shallow-deep-access": [
    "Une propriété profonde d’une valeur `shallowReactive` ou `shallowRef` est lue comme si elle était suivie.",
    "`shallowReactive` suit la propriété racine `user`, mais laisse l’objet imbriqué brut. Modifier `profile.user.name` n’avertit pas le template comme le ferait une modification profonde suivie.",
    "La réactivité profonde de `reactive` enveloppe l’objet utilisateur imbriqué ; la même affectation du nom peut donc déclencher la mise à jour du nom affiché.",
  ],
  "vize:croquis/cf/spread-breaks-reactivity": [
    "Décomposer un objet réactif avec l’opérateur spread copie ses valeurs et supprime leur suivi.",
    "`UserSummary.vue` décompose `props.user` dans un nouvel objet avec l’opérateur spread, prenant un instantané des données réactives reçues.",
    '`toRef(props, "user")` conserve une référence vers la prop reçue au lieu de copier ses champs.',
  ],
  "vize:croquis/cf/suspense-no-fallback": [
    "`<Suspense>` n’a aucun contenu de secours.",
    "La frontière Suspense a un enfant asynchrone, mais aucun contenu de secours ; cet exemple n’affiche donc aucun contenu de chargement pendant l’attente.",
    "Le slot `#fallback` fournit un paragraphe de chargement explicite jusqu’à ce que l’enfant asynchrone soit prêt.",
  ],
  "vize:croquis/cf/template-ref-timing": [
    "Une référence de template est lue avant le montage du composant.",
    "Setup lit la référence de template avant le montage, alors que sa valeur est encore null. L’appel optionnel de focus ne donne donc le focus à aucun élément.",
    "`onMounted` diffère la lecture jusqu’à ce que Vue ait affecté l’élément de saisie à la référence de template, permettant à l’utilitaire de focus d’agir dessus.",
  ],
  "vize:croquis/cf/toraw-mutation": [
    "`toRaw` est utilisé, puis l’objet brut est modifié.",
    "`rename` obtient la cible brute et écrit dans `raw.name`, contournant le setter du proxy qui déclencherait la mise à jour du nom réactif affiché.",
    "Écrire dans `profile.name` au moyen du proxy réactif transmis conserve le même renommage tout en notifiant ce qui en dépend.",
  ],
  "vize:croquis/cf/uncaught-error": [
    "Un composant peut lever une exception sans qu’aucune frontière d’erreur ne la capture.",
    "Le template de l’enfant appelle `JSON.parse` sur une entrée mal formée et le parent accessible n’a aucune frontière de capture d’erreur.",
    "Le parent enregistre `onErrorCaptured` autour de cet enfant. Renvoyer `false` arrête la propagation ; une frontière utilisée en production doit aussi présenter une interface de récupération utile.",
  ],
  "vize:croquis/cf/undeclared-emit": [
    "Le composant émet un événement qui n’est pas déclaré.",
    'L’enfant appelle `emit("save")`, mais son contrat `defineEmits` ne déclare que `cancel`.',
    "Déclarer `save` avec son tuple d’arguments vide pour que l’événement émis corresponde au contrat du composant.",
  ],
  "vize:croquis/cf/undeclared-prop": [
    "Un parent transmet une prop que l’enfant ne déclare pas.",
    "Le parent transmet `typo` alors que l’enfant résolu ne déclare que `title`. Cette convention de l’analyseur est distincte du comportement général de transmission automatique des attributs de Vue.",
    "Supprimer la liaison involontaire `typo` et conserver la prop déclarée `title`.",
  ],
  "vize:croquis/cf/undefined-slot": [
    "Un parent remplit un slot que l’enfant n’expose pas.",
    "App fournit un slot `footer`, mais Card ne déclare et ne rend que `header`. Le contenu Notice fourni ne dispose d’aucun emplacement de slot correspondant dans cet enfant.",
    "App fournit `header`, correspondant à la fois à la déclaration de slot typée de l’enfant et à son emplacement rendu ; Notice apparaît donc à cet endroit.",
  ],
  "vize:croquis/cf/unhandled-event": [
    "Un enfant émet un événement qu’aucun parent ne gère.",
    "`Child.vue` émet `save`, mais le composant qui l’enveloppe directement ne l’écoute pas ; les événements de composants ne remontent pas automatiquement à travers les composants intermédiaires.",
    "`Wrapper.vue` associe un écouteur `save` à son enfant direct. La fonction de rappel vide illustre la gestion de l’événement pour cette règle, et non une implémentation complète de l’enregistrement.",
  ],
  "vize:croquis/cf/unmatched-inject": [
    "`inject` nomme une clé qu’aucun ancêtre ne fournit.",
    "`ThemeLabel.vue` injecte `ThemeKey`, mais son ancêtre accessible `App.vue` ne fournit jamais cette clé.",
    "`App.vue` fournit un thème réactif au moyen du même `ThemeKey` exporté, avant de rendre le descendant qui l’injecte.",
  ],
  "vize:croquis/cf/unmatched-listener": [
    "Un parent écoute un événement que l’enfant n’émet pas.",
    "Le parent écoute `save`, tandis que l’enfant résolu ne déclare que `cancel`.",
    "L’enfant déclare et émet `save`, correspondant au nom de l’écouteur du parent.",
  ],
  "vize:croquis/cf/unregistered-component": [
    "Un template utilise un composant qui n’est ni enregistré ni importé.",
    "Un fichier `Child.vue` existe, mais le parent n’importe pas `Child` et ne l’enregistre pas non plus d’une autre manière pour son template.",
    "Importer `Child` dans le script setup du parent pour que le template résolve la liaison du composant.",
  ],
  "vize:croquis/cf/unresolved-import": [
    "Un import ne se résout pas vers un module.",
    "Le parent importe `./Missing.vue`, mais le projet contient `Child.vue` plutôt que ce chemin.",
    "Faire pointer l’import vers le fichier existant `./Child.vue`, en conservant la même liaison dans le template.",
  ],
  "vize:croquis/cf/unused-attrs": [
    "Des attributs à transmettre automatiquement sont passés à un composant à plusieurs racines qui ne les utilise pas.",
    "Le `tracking-code` du parent n’est ni consommé comme prop ni transmis par l’enfant à plusieurs racines.",
    "Lier `$attrs` à `<main>` donne une destination explicite à cet attribut transmis automatiquement.",
  ],
  "vize:croquis/cf/unused-emit": [
    "Un événement déclaré n’est jamais émis.",
    "L’enfant déclare `save`, mais n’appelle jamais la fonction d’émission d’événements avec ce nom.",
    'L’exemple appelle `emit("save")`, utilisant ainsi l’événement déclaré. Les interactions réelles doivent l’émettre lorsque l’action correspondante se produit.',
  ],
  "vize:croquis/cf/unused-provide": [
    "Une clé fournie n’est jamais injectée.",
    "`App.vue` fournit `ThemeKey`, mais son sous-arbre rendu `Dashboard.vue` ne contient aucun consommateur de cette clé.",
    "Le tableau de bord rend désormais `ThemeLabel.vue`, qui injecte exactement la même identité `ThemeKey` que celle de l’ancêtre.",
  ],
  "vize:croquis/cf/value-extraction-breaks-reactivity": [
    "Extraire une valeur réactive dans une variable locale fait perdre les mises à jour ultérieures.",
    "Le `item` déstructuré réactif de Vue 3.5 est lu une seule fois dans `itemSnapshot` ; le remplacement ultérieur de la prop ne met pas à jour cet instantané.",
    "Lire `item` dans `computed` pour que la transformation de déstructuration réactive des props de Vue puisse suivre chaque évaluation.",
  ],
  "vize:croquis/cf/watch-can-be-computed": [
    "Un observateur ne fait que copier une valeur dans l’état et peut être remplacé par une propriété calculée.",
    "L’observateur ne produit aucun effet externe ; il maintient uniquement une seconde ref modifiable synchronisée avec le double de `count`. Cet exemple n’effectue aucune écriture indépendante dans cette valeur dérivée.",
    "L’accesseur d’une propriété calculée exprime directement la même dérivation et supprime la synchronisation manuelle ainsi que l’état modifiable supplémentaire.",
  ],
} satisfies Record<string, readonly [string, string, string]>;
