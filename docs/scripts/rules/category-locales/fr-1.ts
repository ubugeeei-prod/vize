export const frenchRules1 = {
  "css/no-v-bind-performance": [
    "Signaler le coût de performance de v-bind() en CSS",
    "La feuille de style lit la valeur changeante `offset` au moyen du mécanisme CSS `v-bind()` du SFC.",
    "L’élément reçoit directement la transformation changeante via sa liaison de style.",
  ],
  "css/prefer-logical-properties": [
    "Recommander les propriétés logiques CSS pour mieux prendre en charge l’internationalisation",
    "`margin-left` fixe la marge sur un côté physique indépendamment du sens d’écriture.",
    "`margin-inline-start` suit plutôt le début de la direction en ligne.",
  ],
  "css/prefer-nested-selectors": [
    "Recommander l’imbrication CSS pour les sélecteurs de descendants",
    "Le sélecteur de descendant `.card .title` répète le sélecteur parent dans une règle à plat.",
    "La règle `.title` est imbriquée dans `.card`, ce qui rassemble la relation de style parent-enfant.",
  ],
  "css/prefer-slotted": [
    "Recommander ::v-slotted() pour styliser le contenu des slots",
    "La feuille de style scoped cible le point d’insertion `slot` plutôt que les éléments fournis au slot.",
    "`:slotted(.label)` cible l’élément label fourni au moyen du sélecteur de slot scoped.",
  ],
  "css/require-font-display": [
    "Exiger font-display dans les règles @font-face",
    "La déclaration font-face définit la source de la police, mais omet sa politique font-display.",
    "`font-display: swap` sélectionne explicitement la politique d’affichage passant d’une police de repli à la police chargée.",
  ],
  "ecosystem/nuxt-prefer-nuxt-link": [
    "Préférer NuxtLink pour les liens internes à l’application",
    "La destination interne des paramètres utilise un lien ordinaire dans une application Nuxt.",
    "NuxtLink gère la même destination interne au moyen du routeur Nuxt.",
  ],
  "ecosystem/pinia-prefer-store-to-refs": [
    "Préférer storeToRefs() lors de la déstructuration des stores Pinia",
    "Déstructurer `name` directement depuis le store sépare la valeur de son accès réactif au store.",
    "Le store reste intact et storeToRefs crée une référence réactive pour name.",
  ],
  "ecosystem/router-link-require-to": [
    "Exiger une destination `to` sur les composants RouterLink et NuxtLink",
    "Le RouterLink imbriqué n’a pas de destination `to` ; il ne peut pas compter sur la transmission automatique des attributs racines.",
    '`to="/settings"` fournit explicitement la destination du lien imbriqué.',
  ],
  "ecosystem/void-link-require-href": [
    "Exiger `href` sur les composants Link de Void Vue",
    "Le Link importé depuis @void/vue omet sa destination href.",
    "Le même Link importé reçoit la destination des paramètres via href.",
  ],
  "ecosystem/void-link-valid-method": [
    "Valider les props method statiques des composants Link de Void Vue",
    "L’action DELETE demande un préchargement, alors que prefetch est destiné aux requêtes de navigation.",
    "Supprimer prefetch conserve l’action DELETE sans précharger cette requête non GET.",
  ],
  "ecosystem/vue-i18n-no-missing-key": [
    "Signaler les clés vue-i18n statiques absentes des messages locaux du SFC",
    "Le template demande auth.missing, mais les messages anglais locaux ne déclarent que auth.login.",
    "Le template demande la clé auth.login qui existe dans les messages locaux.",
  ],
  "ecosystem/vue-router-prefer-named-link": [
    "Préférer les objets de routes nommées aux chaînes de chemin statiques dans RouterLink",
    "La destination de RouterLink est un chemin littéral plutôt qu’une route nommée.",
    "L’objet de route lié identifie la destination par son nom de route settings.",
  ],
  "ecosystem/vue-router-prefer-named-push": [
    "Préférer les objets de routes nommées pour la navigation programmatique de Vue Router",
    "router.push reçoit une chaîne de chemin liée à la graphie actuelle de l’URL.",
    "router.push reçoit un objet de route possédant le nom stable settings.",
  ],
  "ecosystem/vue-test-utils-no-html-snapshot": [
    "Éviter les instantanés de wrapper.html() dans les tests Vue Test Utils",
    "L’assertion capture l’ensemble du HTML du wrapper au lieu de vérifier le comportement attendu.",
    "L’assertion vérifie que le texte affiché contient Saved.",
  ],
  "html/deprecated-attr": [
    "Interdire les attributs HTML obsolètes",
    "Le paragraphe utilise l’attribut de présentation obsolète `align`.",
    "La classe et la déclaration `text-align: center` expriment l’alignement au moyen du CSS.",
  ],
  "html/deprecated-element": [
    "Interdire les éléments HTML obsolètes",
    "L’élément `center` utilise un élément de présentation HTML obsolète.",
    "Une section et une classe de style remplacent l’élément obsolète tout en préservant le contenu.",
  ],
  "html/id-duplication": [
    "Interdire les identifiants d’éléments en double",
    'Le champ et le paragraphe d’aide déclarent tous deux `id="email"`, rendant la cible du label ambiguë.',
    "Le champ conserve `email` ; le paragraphe d’aide utilise `email-help`, et aria-describedby référence cet identifiant distinct.",
  ],
  "html/no-consecutive-br": [
    "Interdire les éléments &lt;br&gt; consécutifs",
    "Deux éléments de saut de ligne consécutifs créent un espacement entre des blocs au sein d’un seul paragraphe.",
    "Des paragraphes séparés expriment les deux blocs de contenu sans répéter les éléments de saut de ligne.",
  ],
  "html/no-dupe-style-properties": [
    "Interdire les propriétés en double dans les attributs de style en ligne",
    "Chaque style statique répète une propriété ; `margin` et `MARGIN` sont également considérés comme la même propriété.",
    "Le style statique utilise des propriétés distinctes de couleur et d’arrière-plan. Les liaisons de style dynamiques sont hors du champ de ce contrôle des attributs statiques.",
  ],
  "html/no-duplicate-class": [
    "Interdire les noms de classes en double dans un attribut class statique",
    "La liste de classes statique répète le nom `btn`.",
    "La liste de classes conserve une seule occurrence de `btn` et le nom distinct `primary`.",
  ],
  "html/no-duplicate-dt": [
    "Interdire les noms &lt;dt&gt; en double dans &lt;dl&gt;",
    "La même liste de définitions répète le terme `API` pour deux descriptions.",
    "Un seul terme API est suivi des deux descriptions, évitant de répéter le terme.",
  ],
  "html/no-empty-palpable-content": [
    "Interdire les éléments vides qui attendent un contenu visible",
    "Le paragraphe, l’élément de liste et la cellule de tableau ont tous un contenu palpable vide.",
    "Du texte remplit le paragraphe, une interpolation fournit le contenu de l’élément de liste, et aria-label nomme explicitement la cellule autrement vide.",
  ],
  "html/require-datetime": [
    "Exiger l’attribut datetime sur l’élément &lt;time&gt;",
    "L’élément time contient une date lisible par une personne, mais aucune valeur datetime lisible par une machine.",
    '`datetime="2026-05-13"` fournit la date correspondante lisible par une machine.',
  ],
  "musea/no-empty-variant": [
    "Interdire les blocs &lt;variant&gt; vides",
    "Le variant nommé primary est vide et ne fournit donc aucun contenu d’aperçu.",
    "Le variant affiche un Button primary avec son contenu Save.",
  ],
  "musea/prefer-design-tokens": [
    "Préférer les variables CSS de design tokens aux valeurs primitives codées en dur",
    "L’exemple art utilise la couleur bleue littérale au lieu du design token primaire configuré.",
    "Le style référence --color-primary, le token configuré pour cet exemple.",
  ],
  "musea/require-component": [
    "Exiger l’attribut component dans le bloc &lt;art&gt;",
    "Le bloc art fournit un titre, mais n’identifie pas le composant dont il présente l’aperçu.",
    "defineArt fournit ./Button.vue comme composant du bloc art.",
  ],
  "musea/require-title": [
    "Exiger l’attribut title dans le bloc &lt;art&gt;",
    "Le bloc art identifie Button.vue, mais ne fournit aucun titre.",
    "Les options de defineArt fournissent le titre Button pour le bloc art.",
  ],
  "musea/unique-variant-names": [
    "Exiger des noms de variants uniques",
    "Deux variants du même bloc art utilisent tous deux le nom primary.",
    "Les variants possèdent les noms distincts primary et secondary.",
  ],
  "musea/valid-variant": [
    "Exiger un attribut name dans les blocs &lt;variant&gt;",
    "Le variant omet le nom nécessaire pour identifier l’aperçu.",
    "Le nom primary identifie ce variant.",
  ],
  "nuxt/no-nuxt-config-test-key": [
    "Interdire la clé `test` dans la configuration Nuxt",
    "La configuration Nuxt exportée définit la clé d’identifiant `test` sur le booléen `true`, la forme de configuration obsolète rejetée par cette règle.",
    "La configuration vide supprime cette propriété booléenne `test`. Cet exemple n’interdit pas un objet de configuration de test.",
  ],
  "nuxt/no-page-meta-runtime-values": [
    "Interdire les valeurs du contexte d’exécution évaluées immédiatement dans `definePageMeta`, extrait dans un chunk séparé à la compilation et exécuté avant le setup du composant",
    "`useRoute()` est évalué immédiatement lors de la construction de l’objet `definePageMeta`, alors que la macro déplace ces métadonnées hors du contexte d’exécution du setup.",
    "`validate` reçoit un callback ; son accès à `useRoute().params.id` est donc différé jusqu’à l’exécution de ce callback. La règle distingue les corps de fonctions différés des valeurs de métadonnées évaluées immédiatement.",
  ],
  "nuxt/nuxt-config-keys-order": [
    "Préférer l’ordre recommandé des propriétés de configuration Nuxt",
    "La configuration place `ssr` avant `modules`, inversant leur ordre dans la séquence de clés Nuxt recommandée par la règle.",
    "Placer `modules` avant `ssr` préserve les deux valeurs tout en respectant l’ordre prescrit ; la correction change la disposition plutôt que le sens des options.",
  ],
  "nuxt/prefer-import-meta": [
    "Préférer `import.meta.*` à `process.*`",
    "`process.client` utilise un ancien indicateur d’environnement Nuxt que la règle demande de migrer vers `import.meta`.",
    "`import.meta.client` conserve la branche réservée au navigateur de manière explicite avec l’indicateur d’environnement de remplacement.",
  ],
  "petite-vue/no-unsupported-directive": [
    "Interdire les directives non prises en charge par petite-vue",
    "`v-memo`, `v-slot:header` et la directive personnalisée `v-my-directive` sont absents de la liste des directives prises en charge par petite-vue. Le script petite-vue désigne ce HTML comme relevant de ce dialecte.",
    "Le remplacement utilise les syntaxes prises en charge `v-scope`, `v-effect`, `v-if`, `v-bind` et `v-on` au lieu de dépendre de directives non prises en charge.",
  ],
  "petite-vue/valid-v-effect": [
    "Exiger une expression non vide pour v-effect",
    "Chaque `v-effect` ne possède aucune expression exécutable : sa valeur est absente, vide ou composée uniquement d’espaces.",
    "Les deux valeurs de `v-effect` contiennent une expression : l’une met à jour `el.textContent` et l’autre incrémente `count`. Cette règle vérifie que l’expression n’est pas vide, et non la logique métier de l’effet.",
  ],
} satisfies Record<string, readonly [string, string, string]>;
