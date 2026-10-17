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
  "vue/valid-v-else-if": [
    "Exiger des directives `v-else-if` valides",
    "Les exemples laissent `v-else-if` sans branche conditionnelle précédente ou sans condition, ou lui donnent un argument ou un modificateur.",
    "Placez `v-else-if` avec une condition juste après la branche `v-if` ou `v-else-if` qu’il prolonge.",
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
  "vue/valid-v-is": [
    "Exiger des directives `v-is` valides",
    "`v-is` possède un argument ou un modificateur, n’a pas de valeur, ou est placé sur un composant plutôt que sur un élément HTML natif.",
    "`v-is` avec une valeur sur un élément natif tel que `<tr>` désigne le composant à afficher à sa place.",
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
    "Les écouteurs omettent un événement ou une expression obligatoire, ou utilisent des modificateurs non pris en charge tels que .sync et .nativeish.",
    "Utilisez un gestionnaire, un modificateur d’action sans gestionnaire, un modificateur de clavier pris en charge ou un objet d’écouteurs pour `v-on` sans argument.",
  ],
  "vue/valid-v-once": [
    "Exiger des directives `v-once` valides",
    "`v-once` possède une valeur, un argument ou un modificateur, alors que cette directive marque un rendu unique sans recevoir de valeur.",
    "`v-once` seul marque le sous-arbre pour un rendu unique sans syntaxe non prise en charge.",
  ],
  "vue/valid-v-pre": [
    "Exiger des directives `v-pre` valides",
    "`v-pre` possède un argument, un modificateur ou une valeur, alors que cette directive est un marqueur sans valeur qui ignore la compilation.",
    "`v-pre` seul laisse le sous-arbre non compilé ; ses moustaches s’affichent telles qu’elles sont écrites.",
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
  "## Numeric selection and fixes": "## Sélection des modificateurs numériques et corrections",
  'The first decimal-prefix numeric modifier determines the finding. `@keyup.13.27="submit"` reports only `13` and offers `@keyup.enter.27="submit"`; another fix pass can replace `27` with `escape`. `@keyup.13suffix="submit"` also becomes `@keyup.enter="submit"`. Values 0–9 are exempt, so `@keyup.0.13="submit"` is clean under this rule. Unknown values such as `999` report without a fix. The existing Vue 2 and petite-vue exemption remains in effect.':
    'Le premier modificateur numérique dont le nom commence par un entier décimal détermine le diagnostic. `@keyup.13.27="submit"` ne signale que `13` et propose `@keyup.enter.27="submit"` ; un autre passage de correction peut remplacer `27` par `escape`. `@keyup.13suffix="submit"` devient également `@keyup.enter="submit"`. Les valeurs de 0 à 9 sont exemptées ; `@keyup.0.13="submit"` ne produit donc aucun diagnostic de cette règle. Les valeurs inconnues telles que `999` sont signalées sans correction. L’exemption existante pour Vue 2 et petite-vue reste en vigueur.',
  "## Modifier validation": "## Validation des modificateurs",
  "Unsupported modifiers are reported individually at the modifier name, including on long-form `v-on:click`, dynamic event arguments, and object listener syntax. The accepted names match `eslint-plugin-vue` 10.9.2: Vue's built-in event modifiers, the complete keyboard alias table, any single Unicode code point, and names with a finite decimal integer prefix (JavaScript `Number.parseInt(name, 10)` semantics). For example, `.13suffix` is accepted by this rule. Numeric-key deprecation and `.native` migration diagnostics belong to separate rules.":
    "Les modificateurs non pris en charge sont signalés individuellement sur leur nom, y compris avec la forme longue `v-on:click`, les arguments d’événement dynamiques et la syntaxe objet des écouteurs. Les noms acceptés correspondent à ceux d’`eslint-plugin-vue` 10.9.2 : les modificateurs d’événement intégrés de Vue, la table complète des alias de touches, tout point de code Unicode unique et les noms ayant un préfixe entier décimal fini (selon la sémantique de `Number.parseInt(name, 10)` en JavaScript). Par exemple, cette règle accepte `.13suffix`. Les diagnostics concernant l’abandon des codes de touche numériques et la migration de `.native` relèvent de règles distinctes.",
  "Only `.stop` and `.prevent` provide behavior without a handler. A modifier such as `.once`, `.enter`, or a configured custom name still needs a handler expression.":
    "Seuls `.stop` et `.prevent` ont un effet sans gestionnaire. Un modificateur tel que `.once`, `.enter` ou un nom personnalisé configuré nécessite toujours une expression de gestionnaire.",
  "Projects with custom event handling can add names under `linter.ruleOptions`:":
    "Les projets ayant une gestion personnalisée des événements peuvent ajouter des noms sous `linter.ruleOptions` :",
  'This allows `@click.custom="handler"`. Names are exact and case-sensitive; options supplement the built-in names and do not enable a disabled rule. Scoped config entries can replace inherited names with a new list or reset them with `[]`.':
    'Cela autorise `@click.custom="handler"`. Les noms doivent correspondre exactement, en respectant la casse ; les options complètent les noms intégrés et n’activent pas une règle désactivée. Les entrées de configuration à portée limitée peuvent remplacer les noms hérités par une nouvelle liste ou les réinitialiser avec `[]`.',
  "The keyboard table is adapted from the MIT-licensed [eslint-plugin-vue keyboard aliases](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json). Its source and license are preserved alongside the implementation.":
    "La table des touches est adaptée des [alias de touches d’eslint-plugin-vue](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json), sous licence MIT. Son code source et sa licence sont conservés aux côtés de l’implémentation.",
  "## When handlers overlap": "## Quand les gestionnaires se chevauchent",
  "The rule compares listeners for the same event on a native element. It ignores custom component events unless both listeners use `.native`. Different event names and disjoint keyboard filters do not conflict.":
    "La règle compare les écouteurs du même événement sur un élément natif. Elle ignore les événements des composants personnalisés, sauf si les deux écouteurs utilisent `.native`. Des noms d’événements différents et des filtres de touches sans touche commune n’entrent pas en conflit.",
  "System modifiers are required keys, so `@keydown.alt.shift` also accepts Ctrl+Alt+Shift. If a second handler uses `@keydown.alt.ctrl.shift`, add `.exact` to the broader handler to keep those actions separate. `.exact` permits its declared system modifiers and rejects additional Ctrl, Shift, Alt or Meta keys.":
    "Les modificateurs système désignent des touches qui doivent être enfoncées ; `@keydown.alt.shift` accepte donc également Ctrl+Alt+Shift. Si un second gestionnaire utilise `@keydown.alt.ctrl.shift`, ajoutez `.exact` au gestionnaire le plus général pour séparer ces actions. `.exact` autorise les modificateurs système qu’il déclare et refuse toute touche Ctrl, Shift, Alt ou Meta supplémentaire.",
  "Multiple keyboard modifiers are alternatives: `@keydown.up.down` accepts either arrow key. It can overlap `@keydown.ctrl.down` on Ctrl+ArrowDown. The rule reports the directive key that needs `.exact`, excluding its handler expression.":
    "Plusieurs modificateurs de touches représentent des alternatives : `@keydown.up.down` accepte l’une ou l’autre des deux touches fléchées. Il peut se chevaucher avec `@keydown.ctrl.down` sur Ctrl+FlècheBas. La règle signale la clé de directive qui nécessite `.exact`, sans inclure son expression de gestionnaire.",
  "The check compares `v-for`, `v-slot` (including `#name`), and legacy scoped-slot bindings within the template. It reports the identifier that hides an earlier binding, including declarations on the same element in attribute order. Sibling elements have independent scopes. Property keys and default-value references are not declarations, and `v-pre` makes its subtree inert.":
    "Le contrôle compare les liaisons `v-for`, `v-slot` (y compris `#name`) et les anciennes liaisons de slots à portée limitée au sein du template. Il signale l’identifiant qui masque une liaison précédente, y compris les déclarations du même élément dans l’ordre des attributs. Les éléments frères ont des portées indépendantes. Les clés de propriétés et les références dans les valeurs par défaut ne sont pas des déclarations, et `v-pre` rend son sous-arbre inerte.",
  "Script and component property names are outside the current check: a single template binding does not produce a warning merely because a script binding has the same name. Diagnostic ranges cover the authored identifier, including escaped spellings and HTML entities.":
    "Les noms du script et des propriétés du composant sont hors du champ du contrôle actuel : une liaison isolée du template ne produit pas d’avertissement simplement parce qu’une liaison du script porte le même nom. Les plages de diagnostic couvrent l’identifiant tel qu’il est écrit dans le code source, y compris les graphies échappées et les entités HTML.",
} satisfies Record<string, string>;
