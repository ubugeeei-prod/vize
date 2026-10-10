export const frenchVue2 = {
  "vue/require-component-is": [
    "Exiger `v-bind:is` sur les éléments `<component>`",
    "Le `<component>` dynamique n’a pas de cible `is` ; Vue ne peut donc pas choisir le composant à afficher.",
    '`:is="currentComponent"` fournit la sélection du composant ; la liaison peut changer à l’exécution.',
  ],
  "vue/require-component-registration": [
    "Exiger un import ou un enregistrement explicite des composants",
    "`MissingWidget` n’est ni enregistré ni inclus dans la liste configurée des composants globaux autorisés.",
    "`MyButton` figure dans l’option `globals` de l’exemple. Cette option exempte un composant global connu ; elle ne l’enregistre ni ne l’importe.",
  ],
  "vue/require-scoped-style": [
    "Exiger l’attribut scoped sur les balises style",
    "Le style `.button` n’est pas scoped et peut affecter les éléments correspondants en dehors de ce composant.",
    "Ajouter `scoped` applique la portée du composant Vue aux mêmes sélecteur et déclarations.",
  ],
  "vue/require-toggle-inside-transition": [
    "Exiger un mécanisme de bascule sur l’élément enveloppé par `<transition>`",
    "L’enfant statique à l’intérieur de `<Transition>` n’a ni visibilité conditionnelle ni sélection dynamique pouvant déclencher une entrée ou une sortie.",
    '`v-if="show"` modifie la présence de l’enfant et fournit à la transition une limite d’entrée et de sortie.',
  ],
  "vue/require-v-for-key": [
    "Exiger `v-bind:key` avec les directives `v-for`",
    "Chaque `<li>` répété ne possède pas de clé identifiant l’élément correspondant lors des mises à jour de la liste.",
    '`:key="item.id"` donne à chaque nœud répété l’identité de son élément plutôt que sa position actuelle.',
  ],
  "vue/scoped-event-names": [
    "Recommander des noms d’événements préfixés par leur contexte au format context:event",
    "`playAudio`, `pauseAudio` et `reloadAudio` encodent leur contexte sous forme de suffixes camelCase au lieu de la convention d’événements séparés par deux-points définie par la règle.",
    "`audio:play`, `audio:pause` et `audio:reload` partagent un contexte explicite `audio:`. Le composant émetteur doit utiliser les mêmes noms.",
  ],
  "vue/sfc-element-order": [
    "Imposer un ordre cohérent des éléments de premier niveau d’un SFC",
    "Le bloc style précède le bloc script, contrairement à l’ordre des blocs SFC configuré.",
    "Les blocs suivent l’ordre script → template → style. Les projets peuvent choisir un autre ordre avec l’option typée de cette règle.",
  ],
  "vue/single-style-block": [
    "Recommander un seul bloc style",
    "Le composant répartit ses styles scoped de panneau et de titre dans deux blocs style.",
    "Les deux sélecteurs restent scoped dans un seul bloc style, respectant la convention du bloc unique sans supprimer aucun style.",
  ],
  "vue/slot-name-casing": [
    "Imposer kebab-case aux slots nommés utilisés avec v-slot",
    "Le slot nommé `mySlot` utilise camelCase là où la règle exige un nom séparé par des tirets.",
    "`#my-slot` utilise kebab-case. Renommez le point d’insertion du slot correspondant avec le même nom.",
  ],
  "vue/this-in-template": [
    "Interdire `this.` dans les expressions des templates",
    "Les expressions du template accèdent explicitement à `this.message`, `this.className` et `this.handleClick`, alors que Vue expose directement ces liaisons.",
    "Utilisez directement `message`, `className` et `handleClick`. La chaîne littérale `'this.is.a.string'` reste inchangée, car elle ne constitue pas un accès à un membre.",
  ],
  "vue/use-unique-element-ids": [
    "Imposer des identifiants d’éléments uniques avec useId() plutôt que des littéraux statiques",
    "L’identifiant littéral `email` est réutilisé par chaque instance de ce composant, ce qui peut faire pointer son label vers la mauvaise instance lorsque plusieurs sont affichées.",
    "`useId()` produit l’`emailId` de l’instance ; liez la même valeur au `for` du label et à l’`id` de l’input.",
  ],
  "vue/use-v-on-exact": [
    "Imposer le modificateur `.exact` sur `v-on` en présence de gestionnaires utilisant des modificateurs",
    "Le gestionnaire de clic ordinaire peut également s’exécuter lors d’un Ctrl-clic et chevaucher le gestionnaire `.ctrl` distinct.",
    "`.exact` limite le gestionnaire de clic ordinaire aux clics sans touche modificatrice ; le gestionnaire propre à Ctrl reste distinct.",
  ],
  "vue/v-bind-style": [
    "Imposer un style de directive `v-bind`",
    "`v-bind:class` utilise la forme longue alors que le style de liaison configuré exige la syntaxe abrégée avec deux-points.",
    "`:class` conserve la même expression avec la syntaxe abrégée exigée ; cette règle concerne la graphie, pas le type de la valeur.",
  ],
  "vue/v-on-event-hyphenation": [
    "Imposer des tirets dans les noms d’événements personnalisés de v-on sur les composants",
    "L’écouteur du composant personnalisé utilise `@myEvent` au lieu d’un nom d’événement séparé par des tirets.",
    "`@my-event` utilise la graphie exigée pour les événements personnalisés. Les écouteurs d’éléments natifs et les arguments d’événements dynamiques montrés ci-dessous sont hors du champ de ce contrôle.",
  ],
  "vue/v-on-handler-style": [
    "Imposer une référence de méthode ou une fonction en ligne pour les gestionnaires v-on",
    "Les gestionnaires placent des modifications et plusieurs instructions directement dans l’attribut d’événement.",
    "Utilisez une référence de gestionnaire, ou une expression de fonction fléchée ou classique lorsqu’une logique en ligne est nécessaire. La limite de la fonction rend la forme du gestionnaire explicite.",
  ],
  "vue/v-on-style": [
    "Imposer un style de directive `v-on`",
    "`v-on:click` utilise la forme longue de l’écouteur d’événement alors que la règle exige la syntaxe abrégée.",
    "`@click` conserve le même gestionnaire tout en utilisant la syntaxe abrégée configurée.",
  ],
  "vue/v-slot-style": [
    "Imposer un style de directive `v-slot`",
    "Le composant utilise `#default` et le template utilise `v-slot:header`, contrairement aux styles définis par la règle pour chaque contexte.",
    "Utilisez `v-slot` pour le slot par défaut du composant et `#header` pour le slot nommé du template.",
  ],
  "vue/valid-attribute-name": [
    "Exiger des noms d’attributs valides",
    'Le guillemet dans `my"attr` rend le nom d’attribut mal formé. Cet exemple produit le diagnostic `parser/template` de l’analyseur, sans garantir un diagnostic distinct de cette règle.',
    "`my-attr` est un nom d’attribut bien formé ; l’analyseur du template peut donc lire l’attribut et sa valeur.",
  ],
  "vue/valid-template-root": [
    "Exiger une racine `<template>` valide selon la sémantique des fragments de Vue 3",
    "Un `<template>` imbriqué ordinaire occupe la racine du template sans directive lui donnant un rôle de rendu.",
    "Le `<div>` est un élément racine qui peut être affiché. Cet exemple n’impose pas de restriction universelle à une seule racine pour les fragments de Vue 3.",
  ],
  "vue/valid-v-bind": [
    "Exiger des directives `v-bind` valides",
    "Le `v-bind` sans argument n’a pas d’expression objet, et la forme à argument vide n’a pas de nom d’attribut.",
    "Fournissez un attribut et une expression, liez un objet ou utilisez la syntaxe abrégée de même nom de Vue 3.4+, telle que `:loading`.",
  ],
  "vue/valid-v-cloak": [
    "Exiger des directives `v-cloak` valides",
    "`v-cloak` reçoit une valeur, un argument ou un modificateur alors qu’il n’accepte aucun de ces éléments.",
    "Utilisez `v-cloak` seul ; le CSS peut masquer l’élément jusqu’à ce que Vue retire cet attribut après le montage.",
  ],
  "vue/valid-v-else": [
    "Exiger des directives `v-else` valides",
    "Les exemples donnent une expression à `v-else`, le combinent avec `v-if` ou omettent la branche conditionnelle qui doit le précéder immédiatement.",
    "Placez `v-else` seul immédiatement après la branche `v-if` correspondante.",
  ],
  "vue/valid-v-for": [
    "Exiger des directives `v-for` valides",
    "Les boucles omettent leur expression d’itération ou ajoutent un modificateur `.stop` non pris en charge.",
    "Utilisez `item in items` ou `(item, index) of items` avec une expression d’itération complète et les clés présentées.",
  ],
  "vue/valid-v-html": [
    "Exiger des directives `v-html` valides",
    "`v-html` n’a pas d’expression ou utilise un argument ou modificateur que cette directive ne prend pas en charge.",
    '`v-html="html"` fournit une expression valide. La validité de la syntaxe n’assainit pas le HTML et ne rend pas sûr un contenu non fiable.',
  ],
  "vue/valid-v-if": [
    "Exiger des directives `v-if` valides",
    "Les conditions omettent une expression ou combinent `v-if` avec une directive else sur le même nœud.",
    "Chaque `v-if` possède une condition non vide, telle que `ready` ou `count > 0`, sans directive else incompatible.",
  ],
  "vue/valid-v-memo": [
    "Exiger des directives `v-memo` valides",
    "Un `v-memo` seul ne donne à Vue aucune expression de dépendances pour décider quand réutiliser le sous-arbre.",
    '`v-memo="[valueA, valueB]"` fournit le tableau de dépendances utilisé pour la mémoïsation.',
  ],
  "vue/valid-v-model": [
    "Exiger des directives `v-model` valides",
    "Un `<div>` natif ne peut pas utiliser `v-model` comme contrôle de formulaire, et une directive sans valeur sur input n’a pas d’expression cible modifiable.",
    "Liez l’input, le select, le textarea ou le composant personnalisé aux variables modifiables présentées.",
  ],
  "vue/valid-v-on": [
    "Exiger des directives `v-on` valides",
    "Les formes d’écouteurs omettent un argument d’événement ou l’expression de gestionnaire ou d’objet nécessaire.",
    "Utilisez un événement avec son gestionnaire, ou transmettez un objet d’écouteurs à `v-on` sans argument.",
  ],
  "vue/valid-v-once": [
    "Exiger des directives `v-once` valides",
    "`v-once` possède une valeur, un argument ou un modificateur, alors que cette directive marque un rendu unique sans recevoir de valeur.",
    "`v-once` seul marque le sous-arbre pour un rendu unique sans syntaxe non prise en charge.",
  ],
  "vue/valid-v-show": [
    "Exiger des directives `v-show` valides",
    "`v-show` n’a pas d’expression de visibilité ou est placé sur un `<template>` qui ne produit aucun élément DOM dont l’affichage puisse être modifié.",
    "Appliquez l’expression de visibilité à un élément affiché, tel que `<div>`.",
  ],
  "vue/valid-v-slot": [
    "Exiger des directives `v-slot` valides",
    "La directive de slot est placée sur un `<div>` natif ou entre en conflit avec d’autres déclarations de slots par défaut ou nommés.",
    "Déclarez le slot par défaut d’un composant sur ce composant, ou son slot nommé sur un enfant `<template #header>`.",
  ],
  "vue/valid-v-text": [
    "Exiger des directives `v-text` valides",
    "`v-text` n’a pas d’expression de texte ou utilise un argument ou modificateur non pris en charge.",
    '`v-text="msg"` est syntaxiquement valide. La règle de style distincte `vue/no-v-text` peut toujours préférer l’interpolation.',
  ],
  "vue/warn-custom-block": [
    "Signaler les blocs personnalisés dans les fichiers SFC",
    "Le SFC contient un bloc personnalisé `<i18n>`, qui nécessite une intégration externe au-delà du traitement ordinaire de template, script et style.",
    "L’exemple utilise les blocs standard template et script-setup. Cet avertissement facultatif de portabilité ne signifie pas que tous les blocs personnalisés sont invalides en Vue.",
  ],
  "vue/warn-custom-directive": [
    "Signaler les directives personnalisées nécessitant un enregistrement",
    "`v-focus`, `v-mask` et `v-click-outside` nécessitent des implémentations de directives propres au projet, signalées par cette convention facultative.",
    "L’exemple utilise les directives intégrées `v-if`, `v-model` et `v-on`. Une directive personnalisée correctement enregistrée peut rester valide en Vue lorsque cette politique est désactivée.",
  ],
} satisfies Record<string, readonly [purpose: string, bad: string, good: string]>;

export const frenchVueNotes = {
  "The component filename is checked. PascalCase and kebab-case are accepted; mixed casing is reported.":
    "Le nom de fichier du composant est contrôlé. PascalCase et kebab-case sont acceptés ; une casse mixte est signalée.",
  "Bad has cyclomatic complexity 13 and cognitive complexity 25 (limits: 11 and 16). Each component is measured separately; only inline HTML templates are supported.":
    "L’exemple Mauvais a une complexité cyclomatique de 13 et une complexité cognitive de 25 (limites : 11 et 16). Chaque composant est mesuré séparément ; seuls les templates HTML intégrés sont pris en charge.",
  "See [complexity scoring and component boundaries](../../guide/cross-file-complexity.md) for the contributions behind the example's two scores.":
    "Consultez [le calcul de la complexité et les limites des composants](../../guide/cross-file-complexity.md) pour connaître les contributions aux deux scores de l’exemple.",
  "The filename is the finding. Rename the same component; changing a child tag does not fix it.":
    "Le diagnostic concerne le nom de fichier. Renommez ce même composant ; modifier une balise enfant ne corrige pas le problème.",
  "Enable only for a single-root contract. Vue 3 normally supports fragments.":
    "Activez cette règle uniquement pour un contrat à racine unique. Vue 3 prend normalement en charge les fragments.",
  "This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.":
    "Cette entrée du catalogue n’émet actuellement aucun diagnostic propre à cette règle lors du lint des SFC. La paire Mauvais/Bon décrit la convention souhaitée, et non un diagnostic exécutable. Activer l’identifiant ne fournit pas le contrôle SFC manquant.",
  "The current check compares nested v-for bindings. It does not report a single v-for binding merely because it shares a script binding's name.":
    "Le contrôle actuel compare les liaisons de v-for imbriqués. Il ne signale pas une liaison v-for isolée simplement parce qu’elle porte le même nom qu’une liaison du script.",
  "Checks declared prop names, not the casing of attributes passed to a child.":
    "Contrôle les noms des props déclarées, et non la casse des attributs transmis à un enfant.",
  "List explicit component names supplied by application plugins or Musea previewSetup. PascalCase and kebab-case spellings are accepted; regular expressions are not interpreted. Options do not enable the rule. Later layers replace the list; an empty list clears inherited names.":
    "Listez les noms explicites des composants fournis par les plugins de l’application ou par previewSetup de Musea. Les graphies PascalCase et kebab-case sont acceptées ; les expressions régulières ne sont pas interprétées. Les options n’activent pas la règle. Les couches suivantes remplacent la liste ; une liste vide efface les noms hérités.",
  "Malformed attribute spelling is diagnosed by parser/template before this defensive rule sees an attribute. Bad therefore reports parser/template; it does not promise a separate vue/valid-attribute-name finding.":
    "Un nom d’attribut mal formé est diagnostiqué par parser/template avant que cette règle défensive ne voie un attribut. L’exemple Mauvais signale donc parser/template et ne garantit pas un diagnostic distinct de vue/valid-attribute-name.",
} satisfies Record<string, string>;
