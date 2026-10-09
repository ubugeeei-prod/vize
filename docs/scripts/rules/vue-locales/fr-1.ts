export const frenchVue1 = {
  "vue/no-mutating-props": [
    "Interdire la modification des props d’un composant",
    "Incrémenter props.count modifie directement une valeur fournie par le parent.",
    "Le composant émet update:count avec la nouvelle valeur et laisse au parent la responsabilité de mettre à jour la prop.",
  ],
  "vue/no-negated-v-if-condition": [
    "Interdire une condition v-if négative lorsque la chaîne comporte un v-else",
    "Les branches associées v-if et v-else commencent par une condition négative.",
    "Une condition ok positive vient en premier ; lors de l’inversion d’une condition, placez d’abord la branche opposée d’origine. Un v-if négatif isolé et les comparaisons !== restent autorisés.",
  ],
  "vue/no-non-component-keep-alive-child": [
    "Interdire les enveloppes d’éléments ordinaires directement sous `<KeepAlive>`",
    "KeepAlive enveloppe conditionnellement un div natif au lieu de mettre directement UserCard en cache.",
    "Le premier exemple fait de UserCard l’enfant conditionnel. L’enveloppe avec v-show illustre une structure hors du contrôle des enfants conditionnels et ne garantit pas la mise en cache de l’enveloppe native.",
  ],
  "vue/no-preprocessor-lang": [
    "Déconseiller les préprocesseurs CSS au profit du CSS moderne",
    "Le bloc style sélectionne SCSS avec lang. Cela décrit la convention souhaitée sans préprocesseur ; le traitement SFC actuel n’émet pas de diagnostic pour cette règle.",
    "Les mêmes déclarations CSS omettent le lang du préprocesseur. C’est une correction de la convention, et non une différence de diagnostics Mauvais/Bon exécutable actuellement.",
  ],
  "vue/no-reserved-component-names": [
    "Interdire l’utilisation de noms réservés comme noms de composants",
    "Le nom de composant button entre en conflit avec le nom d’un élément HTML natif.",
    "AppButton est un nom de composant applicatif qui ne réutilise pas le nom natif button.",
  ],
  "vue/no-root-v-if": [
    "Interdire v-if sur l’unique élément racine d’un template",
    "La racine du composant elle-même apparaît et disparaît sous le contrôle de v-if.",
    "Un div externe stable reste la racine, tandis que le paragraphe imbriqué porte la condition de visibilité.",
  ],
  "vue/no-script-non-standard-lang": [
    "Déconseiller les valeurs lang non standard dans les scripts",
    "Le script utilise la syntaxe CoffeeScript avec lang=coffee. Le traitement SFC actuel n’émet pas cette règle du catalogue pour ce langage.",
    "Le script utilise une déclaration TypeScript ordinaire avec lang=ts, illustrant la convention de langage souhaitée.",
  ],
  "vue/no-src-attribute": [
    "Déconseiller l’attribut src sur les blocs SFC",
    "Les blocs du SFC délèguent leur contenu template, script et style aux fichiers désignés par src.",
    "Chaque bloc SFC contient son propre contenu sans attribut src externe.",
  ],
  "vue/no-static-inline-styles": [
    "Interdire les attributs de style en ligne statiques",
    "Le paragraphe porte la déclaration de couleur constante dans son attribut style.",
    "Une classe notice et une feuille de style scoped définissent la couleur constante en dehors de l’attribut du template.",
  ],
  "vue/no-template-key": [
    "Interdire l’attribut `key` sur `<template>`",
    "Une enveloppe template sans boucle possède une key, alors qu’elle ne constitue pas la limite d’une itération à clé.",
    "La clé appartient à une itération template v-for, où elle identifie chaque fragment répété.",
  ],
  "vue/no-template-lang": [
    "Déconseiller l’attribut lang sur le bloc template",
    "Le template sélectionne Pug avec lang. Il s’agit d’une convention souhaitée limitant les templates au HTML ; le traitement SFC actuel n’émet pas de diagnostic pour cet identifiant du catalogue.",
    "Un template HTML ordinaire omet lang et utilise directement le paragraphe. Cela illustre la convention sans prétendre qu’un diagnostic SFC est actuellement émis.",
  ],
  "vue/no-template-shadow": [
    "Interdire les noms de variables qui masquent des variables d’une portée externe",
    "Le v-for interne déclare à nouveau item et masque la liaison item externe dans la boucle imbriquée.",
    "La boucle interne déclare child, laissant item disponible pour la ligne externe et child pour la ligne imbriquée.",
  ],
  "vue/no-template-target-blank": [
    'Interdire target="_blank" sans rel="noopener noreferrer"',
    "Le lien externe ouvre un nouveau contexte de navigation sans la protection rel attendue.",
    "Le même lien inclut noopener noreferrer en plus de target=_blank.",
  ],
  "vue/no-textarea-mustache": [
    "Interdire l’interpolation à doubles accolades dans `<textarea>`",
    "Le textarea place message dans une interpolation enfant au lieu de lier sa valeur.",
    "v-model lie la valeur modifiable du textarea à message.",
  ],
  "vue/no-undefined-refs": [
    "Interdire les références à des variables non définies dans les templates",
    "Le template lit missing, alors que le script ne déclare que message.",
    "L’interpolation lit la liaison message existante.",
  ],
  "vue/no-unsafe-url": [
    "Signaler les liaisons d’URL potentiellement dangereuses",
    "La destination du lien commence par le schéma exécutable javascript:.",
    "Le lien utilise la destination de navigation locale ordinaire /next.",
  ],
  "vue/no-unsandboxed-iframe": [
    "Exiger un attribut sandbox sur les éléments iframe",
    "Le cadre intégré n’a pas d’attribut sandbox limitant ses capacités.",
    "sandbox applique des restrictions ; allow-scripts autorise explicitement cette seule capacité lorsque cela est nécessaire.",
  ],
  "vue/no-unused-components": [
    "Interdire l’enregistrement de composants inutilisés dans les templates",
    "UserAvatar est importé comme composant, mais le template ne l’affiche jamais.",
    "Le template affiche le composant UserAvatar importé et lui transmet la liaison user.",
  ],
  "vue/no-unused-properties": [
    "Interdire les propriétés inutilisées définies dans defineProps",
    "Le composant déclare description comme prop, mais n’affiche que title.",
    "Les deux props déclarées sont référencées par le template.",
  ],
  "vue/no-unused-refs": [
    'Signaler les refs de template (ref="x") jamais référencées dans &lt;script&gt;',
    "Le template déclare le nom de ref unused sans liaison de référence correspondante dans le script.",
    "La ref de template inputEl possède une liaison ref du même nom dans script setup.",
  ],
  "vue/no-unused-setup-bindings": [
    "Interdire les liaisons script setup qui ne sont jamais lues",
    "La liaison message de script setup n’est jamais lue par le template.",
    "Le paragraphe interpole message et utilise ainsi la liaison déclarée.",
  ],
  "vue/no-unused-vars": [
    "Interdire les variables inutilisées définies dans les directives v-for et v-slot",
    "La boucle déclare un index inutilisé et le slot déclare foo sans y faire référence.",
    "Les exemples utilisent index ou indiquent qu’il est volontairement inutilisé en le nommant _index, et le slot affiche data. Les clés fondées sur l’index illustrent uniquement un usage ici, sans recommander cet index pour une identité stable des éléments.",
  ],
  "vue/no-use-v-else-with-v-for": [
    "Interdire `v-else-if` ou `v-else` sur le même élément que `v-for`",
    "La branche else et l’itération v-for sont attachées au même paragraphe.",
    "Un template distinct porte v-else, et son paragraphe enfant porte v-for.",
  ],
  "vue/no-use-v-if-with-v-for": [
    "Interdire `v-if` sur le même élément que `v-for`",
    "Le même élément de liste combine v-if et v-for et teste la visibilité au moyen de la liaison de la boucle.",
    "Une collection calculée filtre les éléments visibles avant que le template n’itère dessus.",
  ],
  "vue/no-useless-mustaches": [
    "Interdire une interpolation à doubles accolades dont l’expression est une chaîne littérale constante",
    "L’interpolation ne contient qu’une chaîne constante et ne nécessite pas d’évaluation d’expression.",
    "Le texte littéral est écrit directement ; les expressions de variables, les chaînes de template avec interpolation et les espaces de séparation intentionnels restent des cas d’interpolation.",
  ],
  "vue/no-useless-template-attributes": [
    "Interdire les attributs inutiles sur les éléments `<template>`",
    "Le template conditionnel possède une classe, mais cette enveloppe structurelle ne produit pas d’élément DOM qui puisse la recevoir.",
    "La classe est déplacée vers le paragraphe qui est effectivement affiché, tandis que v-if reste sur le template structurel.",
  ],
  "vue/no-useless-v-bind": [
    "Interdire un v-bind dont la valeur est une simple chaîne littérale",
    "La liaison foo évalue une chaîne constante entre guillemets ou une chaîne de template sans interpolation.",
    "La valeur constante devient un attribut statique ; les valeurs variables et interpolées conservent leur liaison.",
  ],
  "vue/no-v-for-template-key-on-child": [
    "Interdire `key` sur l’enfant d’un `<template v-for>`",
    "Le paragraphe enfant porte la clé, tandis que l’itération du template elle-même n’en possède pas.",
    "La clé est déplacée vers template v-for et identifie le fragment répété dans son ensemble.",
  ],
  "vue/no-v-html": [
    "Déconseiller v-html pour prévenir les vulnérabilités XSS",
    "v-html interprète le contenu comme du HTML plutôt que comme du texte ordinaire.",
    "L’interpolation à doubles accolades affiche le contenu comme du texte échappé au lieu d’injecter du HTML.",
  ],
  "vue/no-v-text-v-html-on-component": [
    "Interdire v-text / v-html sur les éléments de composants",
    "La balise du composant reçoit v-html ou v-text, qui remplace le contenu d’un élément au lieu de fournir les slots du composant.",
    "Les cibles HTML natives peuvent recevoir les directives ; MyComponent reçoit son contenu via le slot par défaut.",
  ],
  "vue/no-v-text": [
    "Interdire la directive v-text ; préférer l’interpolation à doubles accolades",
    "Le contenu du div est fourni par la directive v-text.",
    "L’interpolation à doubles accolades exprime la même liaison de texte directement dans le contenu de l’élément.",
  ],
  "vue/permitted-contents": [
    "Faire respecter les règles du modèle de contenu HTML",
    "Les exemples placent du contenu de bloc dans p, omettent le corps du tableau, imbriquent des contrôles interactifs ou placent un div directement dans ul.",
    "Les exemples utilisent du contenu en ligne dans le paragraphe, un tbody explicite et des enfants li. Le composant personnalisé MyItem n’est pas considéré comme un enfant natif connu de ul.",
  ],
  "vue/prefer-props-shorthand": [
    "Recommander la syntaxe abrégée des props (Vue 3.4+)",
    "Chaque liaison répète le nom de la variable correspondante, y compris l’équivalent camelCase d’un argument séparé par des tirets.",
    "La syntaxe abrégée des liaisons de même nom de Vue 3.4+ supprime les expressions répétées ; une variable source différente, telle que bar, reste explicite.",
  ],
  "vue/prefer-true-attribute-shorthand": [
    "Préférer la syntaxe abrégée pour un attribut booléen lié à `true`",
    "Un attribut booléen natif disabled est lié à la valeur constante true.",
    "L’attribut natif utilise sa syntaxe booléenne abrégée. Les liaisons à false et les props de composants conservent leurs valeurs explicites.",
  ],
  "vue/prop-name-casing": [
    "Imposer une casse aux noms des props déclarées",
    "Le nom de prop déclaré user_name utilise des mots séparés par des traits de soulignement.",
    "La déclaration et sa référence dans le template utilisent le nom camelCase userName.",
  ],
} satisfies Record<string, readonly [purpose: string, bad: string, good: string]>;
