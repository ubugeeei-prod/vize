export const frenchRules0 = {
  "a11y/alt-text": [
    "Exiger un texte alternatif pour les éléments multimédias",
    "Le contrôle de soumission sous forme d’image ne fournit que l’URL de son image ; il n’a pas de texte `alt` décrivant l’action.",
    '`alt="Submit search"` donne au contrôle image un nom accessible décrivant la soumission de la recherche.',
  ],
  "a11y/anchor-has-content": [
    "Exiger un contenu accessible pour les éléments de lien",
    "Le lien `/settings` n’a ni texte ni autre contenu servant à le nommer ; sa destination n’a donc aucune description accessible.",
    "Le texte visible `Settings` fournit le contenu du lien vers la même destination.",
  ],
  "a11y/anchor-is-valid": [
    "Imposer un href valide sur les éléments de lien",
    "Le premier lien utilise `#` pour une action ; le second utilise une URL JavaScript. Aucun ne fournit une destination de navigation ordinaire.",
    "Un bouton natif exécute `openPanel`, tandis que le lien restant possède la destination réelle `/docs/javascript-urls`.",
  ],
  "a11y/aria-props": [
    "Interdire les attributs ARIA invalides",
    "`aria-lable` est mal orthographié et n’est pas un attribut ARIA pris en charge.",
    "L’attribut pris en charge `aria-label` fournit le nom du bouton.",
  ],
  "a11y/aria-role": [
    "Exiger un rôle ARIA valide et non abstrait pour les éléments possédant un rôle ARIA",
    "`datepicker` n’est pas un rôle ARIA reconnu pour cette section.",
    "La section utilise le rôle reconnu `dialog` et un libellé décrivant la sélection de la date.",
  ],
  "a11y/aria-unsupported-elements": [
    "Interdire les attributs ARIA sur les éléments qui ne les prennent pas en charge",
    "L’élément de métadonnées porte `aria-hidden`, alors que `meta` ne prend pas en charge les attributs ARIA.",
    "Supprimer l’attribut ARIA conserve intacte la déclaration de jeu de caractères.",
  ],
  "a11y/click-events-have-key-events": [
    "Exiger des gestionnaires de clavier avec les événements de clic",
    "Le `div` non interactif possède un gestionnaire de clic, mais aucune gestion des événements de clavier.",
    "Un `button` natif permet une activation au clavier avec le même gestionnaire `activate`.",
  ],
  "a11y/form-control-has-label": [
    "Exiger des libellés associés aux contrôles de formulaire",
    "Le champ de recherche n’a pas de libellé indiquant ce que l’utilisateur doit saisir.",
    "Envelopper le champ dans un label associe le texte visible `Search` au contrôle.",
  ],
  "a11y/heading-has-content": [
    "Exiger un contenu accessible pour les éléments de titre",
    "Le `h2` apporte un niveau de titre, mais ne contient aucun texte de titre.",
    "`Billing settings` fournit le contenu du titre de niveau deux existant.",
  ],
  "a11y/heading-levels": [
    "Interdire de sauter des niveaux de titre",
    "La séquence des titres passe directement de `h1` à `h3`, sans niveau deux.",
    "Remplacer le titre de facturation par `h2` conserve une hiérarchie de titres consécutive.",
  ],
  "a11y/iframe-has-title": [
    "Exiger un attribut title sur les éléments iframe",
    "Le cadre de paiement possède une URL source, mais aucun `title` décrivant le contenu intégré.",
    '`title="Checkout preview"` nomme le contenu de ce cadre.',
  ],
  "a11y/img-alt": [
    "Exiger un attribut alt sur les images pour les rendre accessibles",
    "L’image de l’avatar ne possède pas d’attribut `alt`.",
    '`alt="User avatar"` fournit une alternative textuelle à l’avatar.',
  ],
  "a11y/interactive-supports-focus": [
    "Exiger que les éléments possédant un rôle interactif puissent recevoir le focus",
    "Donner à un `span` le rôle button et un gestionnaire de clic ne rend pas l’élément accessible au focus clavier.",
    "Le bouton natif peut recevoir le focus et conserve la même action `open`.",
  ],
  "a11y/label-has-for": [
    "Exiger des contrôles de formulaire associés aux libellés",
    "Le label séparé n’est ni associé au moyen de `for` ni placé autour du champ.",
    '`for="email"` correspond à l’identifiant du champ et associe explicitement les deux éléments.',
  ],
  "a11y/landmark-roles": [
    "Valider l’emplacement et l’unicité des rôles de zones de repère",
    "Deux éléments `main` déclarent des zones principales en double dans le même template.",
    "Le tableau de bord reste la zone principale ; la zone des paramètres devient une zone de navigation nommée.",
  ],
  "a11y/media-has-caption": [
    "Exiger des sous-titres pour les éléments multimédias",
    "La vidéo possède des contrôles de lecture, mais aucune piste de sous-titres.",
    'Un `track` avec `kind="captions"` fournit les sous-titres anglais de la même vidéo.',
  ],
  "a11y/mouse-events-have-key-events": [
    "Exiger des événements de focus et de perte du focus avec les événements de souris",
    "La visibilité de l’aperçu change uniquement avec les gestionnaires d’entrée et de sortie de la souris.",
    "Les mêmes actions d’aperçu s’exécutent lors du focus et de sa perte, et le bouton peut recevoir le focus clavier.",
  ],
  "a11y/no-access-key": [
    "Interdire l’utilisation de l’attribut accesskey",
    'Le raccourci `accesskey="s"` peut entrer en conflit avec les raccourcis du navigateur ou des technologies d’assistance.',
    "Supprimer `accesskey` conserve le bouton Save ordinaire.",
  ],
  "a11y/no-aria-hidden-on-focusable": [
    'Interdire aria-hidden="true" sur les éléments pouvant recevoir le focus',
    'Le bouton Close pouvant recevoir le focus est masqué dans l’arbre d’accessibilité avec `aria-hidden="true"`.',
    "Le bouton reste exposé et reçoit un libellé `Close` au lieu d’être masqué.",
  ],
  "a11y/no-autofocus": [
    "Interdire l’utilisation de l’attribut autofocus",
    "Le champ demande à recevoir automatiquement le focus lorsqu’il apparaît.",
    "Supprimer `autofocus` évite cette demande de focus automatique tout en conservant le champ de requête.",
  ],
  "a11y/no-distracting-elements": [
    "Interdire les éléments distrayants tels que &lt;marquee&gt; et &lt;blink&gt;",
    "L’élément `marquee` introduit un texte en mouvement automatique.",
    "Un paragraphe affiche la même offre sans l’élément marquee distrayant.",
  ],
  "a11y/no-i-for-icon": [
    "Interdire l’élément &lt;i&gt; pour les icônes",
    "L’icône est affichée avec `i`, dont la sémantique textuelle ne décrit pas une action représentée uniquement par une icône.",
    "Un span décoratif masque le glyphe de l’icône, tandis que le texte séparé `Delete item` nomme l’action du bouton.",
  ],
  "a11y/no-redundant-roles": [
    "Interdire les rôles ARIA redondants",
    'Le bouton natif possède déjà le rôle button ; `role="button"` répète donc sa sémantique implicite.',
    "Supprimer le rôle répété conserve la sémantique du bouton fournie par HTML.",
  ],
  "a11y/no-refer-to-non-existent-id": [
    "Interdire les références à des identifiants inexistants",
    "`aria-labelledby` pointe vers `save-label`, mais aucun élément ne déclare cet identifiant.",
    "Ajouter le span correspondant résout la référence et fournit le libellé du bouton.",
  ],
  "a11y/no-role-presentation-on-focusable": [
    'Interdire role="presentation" ou role="none" sur les éléments pouvant recevoir le focus',
    "Le lien de facturation pouvant recevoir le focus demande role=presentation, ce qui entre en conflit avec son rôle de lien interactif ; les navigateurs doivent ignorer cette demande de présentation.",
    "Supprimez la demande de présentation contradictoire et utilisez le rôle de lien natif ainsi que la destination de facturation.",
  ],
  "a11y/no-static-element-interactions": [
    "Interdire les gestionnaires d’événements sur les éléments statiques",
    "Une section statique reçoit une action liée à la touche Entrée sans posséder de rôle interactif.",
    "Un bouton natif porte la même action dans un élément interactif approprié.",
  ],
  "a11y/placeholder-label-option": [
    "Exiger disabled ou hidden sur l’option d’invite d’un select",
    "L’invite à valeur vide reste sélectionnable comme s’il s’agissait d’une valeur de pays.",
    "Ajouter `disabled` distingue l’invite de l’option Japan sélectionnable.",
  ],
  "a11y/role-has-required-aria-props": [
    "Exiger les propriétés obligatoires des rôles ARIA",
    "Le rôle checkbox omet `aria-checked`, qui communique l’état de la case à cocher.",
    '`aria-checked="false"` fournit l’état exigé par le rôle checkbox.',
  ],
  "a11y/tabindex-no-positive": [
    "Interdire les valeurs positives de tabindex",
    "Un tabindex positif de 3 crée un ordre de focus personnalisé avant les contrôles ordinaires.",
    "Le bouton utilise son ordre de focus natif sans tabindex positif.",
  ],
  "a11y/use-list": [
    "Suggérer des éléments de liste pour les textes ressemblant à des listes à puces",
    "Les tâches sont des paragraphes séparés précédés de tirets saisis, plutôt que des éléments de liste.",
    "Une liste non ordonnée et ses éléments expriment les mêmes tâches avec une sémantique de liste.",
  ],
  "css/no-display-none": [
    "Suggérer v-show plutôt que display: none",
    "La déclaration `.message` masque le paragraphe local au moyen du CSS plutôt que d’une condition de visibilité dans le template.",
    '`v-show="isSaved"` rend la condition de visibilité explicite sur le paragraphe local et supprime `display: none`.',
  ],
  "css/no-hardcoded-values": [
    "Suggérer des variables CSS plutôt que des valeurs codées en dur",
    "Le bouton place directement des nombres d’espacement et une couleur hexadécimale dans les déclarations.",
    "Les déclarations référencent des propriétés personnalisées d’espacement et de couleur nommées, afin que ces valeurs puissent être maintenues sous forme de tokens.",
  ],
  "css/no-id-selectors": [
    "Déconseiller les sélecteurs d’identifiant en CSS",
    "`#submit` lie la règle de style à un sélecteur d’identifiant.",
    "La classe `.submit` fournit un point d’accroche de style réutilisable sans sélecteur d’identifiant.",
  ],
  "css/no-important": [
    "Déconseiller !important en CSS",
    "La déclaration de couleur remplace la priorité normale de la cascade avec `!important`.",
    "La couleur provient d’une propriété personnalisée sans déclaration important.",
  ],
  "css/no-utility-classes": [
    "Déconseiller l’implémentation de classes utilitaires dans les styles des composants",
    "Les sélecteurs écrits utilisent des noms de forme utilitaire, tels que `.flex`, `.mt-4` et `.text-center`.",
    "Un sélecteur `.my-component` propre au composant rassemble ses styles sous un nom sémantique unique.",
  ],
} satisfies Record<string, readonly [string, string, string]>;
