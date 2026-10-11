export const frenchVue0 = {
  "vue/a11y-img-alt": [
    "Exiger un attribut alt sur les images pour les rendre accessibles",
    "Ni l’image statique ni l’image dont la source est dynamique ne fournit d’attribut alt.",
    "Les images informatives reçoivent un texte alt descriptif, les images décoratives un alt vide, et l’image dynamique lie sa description.",
  ],
  "vue/attribute-hyphenation": [
    "Imposer un style de nommage des attributs sur les composants personnalisés",
    "L’attribut du composant utilise la graphie camelCase firstName.",
    "La graphie first-name respecte la convention configurée qui sépare les mots des attributs de composants par des tirets.",
  ],
  "vue/attribute-order": [
    "Imposer un ordre cohérent des attributs",
    "Le gestionnaire d’événement apparaît avant la directive structurelle v-if et l’attribut ordinaire id.",
    "v-if vient en premier, suivi de id puis du gestionnaire d’événement, conformément à l’ordre défini par la règle.",
  ],
  "vue/component-definition-name-casing": [
    "Imposer PascalCase ou kebab-case aux noms de définition des composants",
    "Le nom de fichier myComponent.vue combine une initiale minuscule et une majuscule interne au lieu d’utiliser PascalCase ou kebab-case.",
    "Renommer le fichier en MyComponent.vue adopte PascalCase ; le contenu de son template reste inchangé.",
  ],
  "vue/component-name-in-template-casing": [
    "Imposer une casse précise aux noms de composants dans les templates",
    "Le composant est écrit en kebab-case et en camelCase alors que la convention impose PascalCase.",
    "MyComponent utilise PascalCase ; la syntaxe native slot reste en minuscules.",
  ],
  "vue/html-button-has-type": [
    "Exiger un type explicite et valide sur les éléments button",
    "Un bouton omet type et un autre fournit le type foo, qui n’est pas pris en charge.",
    "Les boutons précisent button, submit ou reset ; un type lié est considéré comme dynamique.",
  ],
  "vue/html-quotes": [
    "Imposer un style de guillemets pour les attributs HTML",
    "Les attributs utilisent des apostrophes ou aucun guillemet au lieu des guillemets doubles exigés par la convention.",
    "Les attributs ordinaires et les expressions des directives utilisent des guillemets doubles.",
  ],
  "vue/html-self-closing": [
    "Imposer un style de fermeture automatique des balises",
    "Le composant vide utilise une paire de balises, tandis que les éléments vides img et br omettent la syntaxe autofermante configurée.",
    "Le composant et les éléments vides utilisent une syntaxe autofermante ; un div contenant du contenu conserve sa balise de fermeture.",
  ],
  "vue/max-template-complexity": [
    "Limiter la complexité propre du template d’un composant, cyclomatique et cognitive",
    "Les branches, la boucle, le contenu des slots et les décisions dans les expressions écrits dans le parent produisent des scores de 13 et 25, supérieurs aux limites par défaut de 11 et 16.",
    "Le template parent délègue le rendu à RowList et conserve un seul v-if ; ses propres scores sont de 2 et 1.",
  ],
  "vue/multi-word-component-names": [
    "Exiger des noms de composants composés de plusieurs mots",
    "Item.vue donne au composant un nom composé d’un seul mot.",
    "TodoItem.vue donne au même template un nom de composant composé de plusieurs mots.",
  ],
  "vue/mustache-interpolation-spacing": [
    "Imposer un espacement cohérent dans les interpolations à doubles accolades",
    "Il manque un espace à une ou aux deux limites des délimiteurs de l’interpolation de texte.",
    "Des espaces séparent l’expression des délimiteurs d’ouverture et de fermeture à doubles accolades.",
  ],
  "vue/no-array-index-key": [
    "Interdire l’utilisation directe de la variable d’index de v-for comme :key",
    "La clé de la liste est son index actuel ; l’identité de l’élément change donc lorsque la liste est réordonnée.",
    "La clé provient de item.id, ce qui préserve l’identité de chaque élément lorsqu’il change de position.",
  ],
  "vue/no-bare-strings-in-template": [
    "Interdire le texte brut destiné aux utilisateurs dans les templates lorsqu’il devrait être internationalisé",
    "Le texte visible et les attributs de nommage contiennent des chaînes non traduites directement dans le template.",
    "Le contenu traduisible appelle $t ; les exemples de ponctuation et de valeurs uniquement numériques sont des exceptions autorisées.",
  ],
  "vue/no-boolean-attr-value": [
    "Interdire les valeurs explicites des attributs HTML booléens",
    "Les attributs booléens disabled et checked contiennent inutilement des valeurs textuelles.",
    "La présence de chaque attribut booléen exprime le même état actif sans valeur.",
  ],
  "vue/no-child-content": [
    "Interdire le contenu enfant lors de l’utilisation de v-html ou v-text",
    "v-text remplace le contenu du paragraphe ; le texte de repli écrit dans le template ne peut donc pas être conservé avec cette directive.",
    "Supprimer le texte enfant laisse v-text comme unique source du contenu du paragraphe.",
  ],
  "vue/no-deprecated-filter": [
    "Interdire la syntaxe obsolète des filtres de Vue 2 utilisant l’opérateur pipe",
    "Le pipe utilise la syntaxe supprimée des filtres Vue pour appliquer capitalize.",
    "L’appel capitalize(message) applique la transformation sous la forme d’une expression ordinaire.",
  ],
  "vue/no-deprecated-functional-template": [
    "Interdire l’attribut `functional` sur le `<template>` d’un SFC",
    "Le template du SFC possède l’attribut functional supprimé et lit l’ancien contexte props.",
    "Le template ordinaire omet functional et lit directement la liaison msg du composant.",
  ],
  "vue/no-deprecated-html-element-is": [
    "Interdire l’attribut `is` sur les éléments HTML natifs",
    "Un div natif utilise l’ancien attribut is sans préfixe pour demander un composant Vue.",
    "Un composant dynamique utilise :is ; la syntaxe sur un élément natif utilise explicitement le préfixe vue:.",
  ],
  "vue/no-deprecated-inline-template": [
    "Interdire l’attribut obsolète `inline-template`",
    "Card utilise l’attribut obsolète inline-template pour le contenu qui lui est fourni.",
    "Le même contenu est transmis normalement, sans l’attribut inline-template.",
  ],
  "vue/no-deprecated-router-link-tag-prop": [
    "Interdire la prop `tag` sur &lt;router-link&gt;",
    "RouterLink utilise la prop tag supprimée pour demander un élément button.",
    "Le slot fournit navigate à un bouton explicitement écrit dans le template.",
  ],
  "vue/no-deprecated-scope-attribute": [
    "Interdire l’attribut obsolète `scope` sur &lt;template&gt;",
    "Le template du slot déclare props au moyen de l’attribut obsolète scope.",
    "La directive du slot par défaut déclare la même liaison props avec la syntaxe actuelle des slots.",
  ],
  "vue/no-deprecated-slot-attribute": [
    "Interdire l’attribut obsolète `slot`",
    "Le slot header est sélectionné au moyen de l’ancien attribut slot.",
    "v-slot:header sélectionne explicitement le slot header avec la directive actuelle.",
  ],
  "vue/no-deprecated-slot-scope-attribute": [
    "Interdire l’attribut obsolète `slot-scope`",
    "Le template reçoit les props du slot au moyen de l’attribut obsolète slot-scope.",
    "La directive #default reçoit ces props sans slot-scope.",
  ],
  "vue/no-deprecated-v-bind-sync": [
    "Interdire le modificateur obsolète `.sync` sur `v-bind`",
    "Les liaisons utilisent le modificateur .sync supprimé, y compris en combinaison avec .camel.",
    "Utilisez une liaison title ordinaire à sens unique, ou v-model:title lorsqu’un canal de mise à jour est nécessaire.",
  ],
  "vue/no-deprecated-v-on-native-modifier": [
    "Interdire le modificateur obsolète `.native` sur `v-on`",
    "Les gestionnaires du composant utilisent le modificateur d’événement .native supprimé.",
    "Les gestionnaires omettent .native et conservent les autres modificateurs d’événement, tels que .stop.",
  ],
  "vue/no-deprecated-v-on-number-modifiers": [
    "Interdire les modificateurs numériques obsolètes `keyCode` sur `v-on`",
    "Les gestionnaires de clavier identifient les touches par les codes numériques supprimés 13 et 27.",
    "Les gestionnaires utilisent les modificateurs de touches nommés enter et esc.",
  ],
  "vue/no-dupe-v-else-if": [
    "Interdire les conditions répétées dans les chaînes `v-if` / `v-else-if`",
    "Le else-if répète la condition ready déjà testée par la première branche, ce qui rend cette branche suivante inaccessible.",
    "La deuxième branche teste loading, un état distinct qui permet d’atteindre le else-if.",
  ],
  "vue/no-duplicate-attributes": [
    "Interdire les attributs en double sur un même élément",
    "Le même bouton déclare class deux fois au lieu de réunir les classes dans une seule valeur.",
    "Les deux noms de classes figurent dans un unique attribut class.",
  ],
  "vue/no-empty-component-block": [
    "Interdire les blocs SFC vides",
    "Les blocs template, script et style ne contiennent aucun contenu significatif.",
    "Chaque bloc conservé contient du balisage, des déclarations de script ou des déclarations de style effectifs.",
  ],
  "vue/no-inline-style": [
    "Déconseiller l’utilisation d’attributs de style en ligne",
    "L’attribut statique style place la déclaration de couleur dans l’élément.",
    "Les classes expriment la couleur fixe ; la largeur dépendant de ratio reste une liaison de style dynamique, hors du contrôle des attributs statiques.",
  ],
  "vue/no-invalid-html-attribute": [
    "Interdire les valeurs statiques invalides des attributs HTML",
    "Le lien utilise stylesheet comme valeur de rel, alors que cette valeur appartient aux éléments link de feuilles de style.",
    "Le lien utilise help, une valeur de rel adaptée à une ressource d’aide liée.",
  ],
  "vue/no-lone-template": [
    "Interdire les éléments `<template>` inutiles",
    "Le template interne n’a ni directive ni rôle de slot lui donnant une fonction structurelle.",
    "Supprimer l’enveloppe inutile laisse le paragraphe directement à l’intérieur de div.",
  ],
  "vue/no-multi-spaces": [
    "Interdire plusieurs espaces consécutifs",
    "Deux espaces séparent les attributs, ou le nom de l’élément et son premier attribut.",
    "Un seul espace sépare les mêmes attributs.",
  ],
  "vue/no-multiple-objects-in-class": [
    "Interdire plusieurs objets littéraux dans une liaison :class sous forme de tableau",
    "Un tableau de classes contient deux objets littéraux au premier niveau qui peuvent être fusionnés.",
    "Un seul objet contient les conditions des classes ; les tableaux comprenant un objet et une chaîne, ou des entrées non littérales, restent autorisés.",
  ],
  "vue/no-multiple-template-root": [
    "Interdire plusieurs nœuds racines dans un template",
    "La convention à racine unique, activée explicitement, trouve deux paragraphes frères à la racine du template.",
    "Un élément section enveloppe les paragraphes dans une seule racine ; activez cette convention uniquement lorsqu’un contrat à racine unique est souhaité.",
  ],
} satisfies Record<string, readonly [purpose: string, bad: string, good: string]>;
