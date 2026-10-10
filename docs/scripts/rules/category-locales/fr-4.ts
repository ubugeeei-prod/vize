export const frenchRules4 = {
  "type/require-typed-emits": [
    "Exiger une définition de type pour defineEmits",
    'La déclaration uniquement sous forme de tableau `defineEmits(["save"])` déclare le nom de l’événement sans contrat typé pour ses arguments.',
    "`defineEmits<{ save: [] }>()` déclare l’événement typé `save` avec un tuple d’arguments vide, indiquant explicitement qu’il n’accepte aucun argument.",
  ],
  "type/require-typed-props": [
    "Exiger une définition de type pour defineProps",
    'La déclaration uniquement sous forme de tableau `defineProps(["title"])` déclare `title` par son nom sans lui donner de type.',
    "`defineProps<{ title: string }>()` donne à `title` un type chaîne explicite au lieu d’une déclaration à l’exécution limitée à son nom.",
  ],
  "type/strict-boolean-expressions": [
    "Exiger des expressions booléennes sûres dans les conditions des scripts et des templates",
    "`if (count)` repose sur la conversion implicite en booléen d’une liaison numérique nullable plutôt que sur un test booléen explicite ; cela confond aussi zéro avec l’absence de valeur.",
    "`count !== undefined && count > 0` teste séparément la présence et la positivité, produisant une condition booléenne explicite après avoir affiné le type de la valeur optionnelle.",
  ],
  "vapor/no-inline-template": [
    "Interdire l’attribut obsolète inline-template",
    "LegacyCard utilise l’attribut inline-template pour le balisage de son enfant.",
    "Le balisage est transmis par le slot par défaut au lieu d’un template en ligne.",
  ],
  "vapor/no-vue-lifecycle-events": [
    "Interdire les événements de cycle de vie @vue:xxx par élément (non pris en charge dans Vapor)",
    "Le champ de saisie utilise l’événement de cycle de vie de template @vue:mounted.",
    "onMounted accède à la référence de template nommée et donne le focus au champ de saisie au moyen du hook de cycle de vie de script pris en charge.",
  ],
  "vapor/prefer-static-class": [
    "Préférer une classe statique à une liaison de classe dynamique pour les chaînes littérales",
    "La liaison de classe évalue une chaîne constante alors que la classe ne change pas.",
    "Un attribut class statique exprime les mêmes classes du panneau sans liaison.",
  ],
  "vapor/require-vapor-attribute": [
    "Suggérer l’ajout de l’attribut vapor à script setup",
    "Le bloc script setup ne possède pas l’attribut de compilation Vapor. Il s’agit d’une convention prévue : la fonction de rappel actuellement vide de la règle ne le signale pas.",
    "L’ajout de vapor sélectionne la compilation Vapor. Il illustre la correction prévue et ne signifie pas que le linter actuel émet cette règle du catalogue.",
  ],
  "ecosystem/vue-router-extra-param": [
    "La route ne déclare pas tab ; Vue Router l’ignore.",
    "Le chemin `user-post` déclare `userId` et `postId`, mais la navigation fournit également `tab`, non déclaré, comme paramètre de chemin.",
    "Supprimer `tab` de params et conserver uniquement les clés présentes dans le chemin de la route. Utiliser query séparément si l’application a besoin de sélectionner un onglet.",
  ],
  "ecosystem/vue-router-missing-param": [
    "Le paramètre obligatoire postId manque ; dépendre de la route actuelle est fragile.",
    "La navigation omet le paramètre obligatoire `postId` du chemin `user-post`. Il s’agit d’un avertissement, car Vue Router peut hériter d’une valeur de la route actuelle.",
    "Passer explicitement `userId` et `postId` pour que la navigation ne dépende pas de l’état des paramètres de la route actuelle.",
  ],
  "ecosystem/vue-router-param-type": [
    "postId n’est pas répétable ; un tableau est donc invalide.",
    '`postId` est un paramètre de chemin scalaire, mais la navigation lui donne le tableau `["2"]`.',
    'Passer la valeur scalaire `"2"` pour le segment `postId` non répétable.',
  ],
  "ecosystem/vue-router-unknown-route": [
    "Le nom est absent du routeur installé complet.",
    "Le routeur installé accessible déclare `user-post`, mais la navigation utilise le nom mal orthographié `user-posts`.",
    "Utiliser le nom enregistré `user-post` tout en conservant les deux paramètres de chemin déclarés.",
  ],
  "html/cross-component-nesting": [
    "Vérifier l’imbrication HTML réelle après la composition des composants importés.",
    "Le `<p>` du parent contient un enfant résolu dont la racine est `<div>`, ce qui produit une imbrication paragraphe/bloc invalide après composition.",
    "Utiliser un conteneur `<section>` pouvant contenir l’élément de bloc de l’enfant ; l’enfant reste inchangé.",
  ],
  "vize:croquis/cf/array-mutation": [
    "Un tableau est modifié par index, ce qu’un tableau réactif ne suit pas.",
    "Dans ce projet historique Vue 2.7, `items[0] = next` modifie le tableau sans avertir l’observateur de tableaux de Vue 2 ; le premier élément affiché peut donc ne pas être mis à jour.",
    "`splice(0, 1, next)` utilise la méthode de modification de tableau observée par Vue 2, permettant au même remplacement de mettre à jour la vue.",
  ],
  "vize:croquis/cf/async-boundary": [
    "Un état réactif traverse une frontière asynchrone et peut être observé dans un état périmé.",
    "Une ancienne requête plus lente peut se terminer après une requête plus récente et écraser `result`, car l’observateur n’effectue aucun nettoyage lors de l’invalidation.",
    "Enregistrer le nettoyage avant d’attendre : annuler l’ancienne requête et invalider son indicateur `active`, puis affecter uniquement une réponse encore active.",
  ],
  "vize:croquis/cf/async-no-suspense": [
    "Un composant asynchrone est rendu sans frontière Suspense.",
    "L’enfant contient un await au niveau supérieur, mais son parent ne fournit aucune frontière `<Suspense>`. L’analyse actuelle du code source ne fournit pas l’information de macro nécessaire pour émettre ce code de diagnostic.",
    "Le parent enveloppe le même enfant asynchrone dans `<Suspense>` avec un contenu de secours pendant le chargement. Cela illustre la convention ; aucune des deux variantes de code source ne produit ce diagnostic dans la passe actuelle.",
  ],
  "vize:croquis/cf/browser-api-ssr": [
    "Une API réservée au navigateur est utilisée alors que le composant peut être rendu sur le serveur.",
    "`window.innerWidth` s’exécute pendant setup, alors qu’un environnement SSR ne dispose pas du `window` du navigateur.",
    "Initialiser une ref avec une valeur sûre pour le serveur et lire `window` dans `onMounted`, qui s’exécute après le montage côté client.",
  ],
  "vize:croquis/cf/circular-dep": [
    "Des composants s’importent mutuellement en formant un cycle.",
    "`a.ts` importe `b.ts`, qui importe à son tour `a.ts`. Tous deux initialisent immédiatement une constante à partir de la constante encore non initialisée de l’autre module, provoquant une erreur de zone morte temporelle.",
    "Les deux modules lisent des préfixes initialisés dans le module indépendant `labels.ts`, supprimant le cycle et la lecture croisée immédiate.",
  ],
  "vize:croquis/cf/circular-reactive-dependency": [
    "Des calculs réactifs dépendent les uns des autres en formant un cycle.",
    "App possède et fournit count (A). CycleView en dérive nextCount (B), puis réécrit immédiatement chaque valeur dérivée dans le même count injecté. Chaque écriture modifie à nouveau l’entrée du calcul, créant une boucle de mises à jour A → B → A. Les identités du graphe conservé ci-dessous représentent ces deux références, et non des liaisons sans rapport portant les mêmes noms.",
    "Supprimer l’observateur qui réécrit B dans A. App conserve la propriété de count et ne le modifie que par son action explicite Increment ; CycleView lit la valeur dérivée nextCount sans réinjecter le résultat. Les mêmes références ne conservent que la dépendance A → B.",
  ],
  "vize:croquis/cf/closure-captures-reactive": [
    "Une fermeture capture une valeur réactive et ne verra pas les mises à jour ultérieures.",
    "`makeReader` copie `count.value` avant de créer la fermeture. Le lecteur calculé renvoie ensuite ce nombre initial sans lire de dépendance réactive.",
    "La fermeture lit `count.value` lorsqu’elle est appelée ; l’accesseur calculé peut donc suivre la ref et mettre à jour `shown` après les incréments.",
  ],
  "vize:croquis/cf/composable-outside-setup": [
    "Un composable est appelé en dehors de `setup`.",
    "L’import de `use-title.ts` enregistre `onMounted` avant qu’un setup de composant soit actif. Appeler sa fonction exportée plus tard ne fait que renvoyer cette ref au niveau du module ; cela ne peut pas rétablir l’appartenance manquée au cycle de vie.",
    "La création de l’état et l’enregistrement du hook sont tous deux déplacés dans `useTitle`, qu’App appelle de manière synchrone dans setup. Le hook de montage appartient désormais à cette instance d’App.",
  ],
  "vize:croquis/cf/computed-side-effects": [
    "L’accesseur d’une propriété calculée écrit dans l’état ou produit un autre effet de bord.",
    "L’évaluation de `doubled` écrit dans `lastCalculated` ; lire une valeur calculée modifie donc aussi un état distinct. Cela lie l’effet de bord au moment où l’accesseur paresseux est lu.",
    "L’accesseur renvoie uniquement le nombre dérivé. Un observateur distinct se charge d’écrire dans `lastCalculated` lorsque `count` change, y compris pour sa valeur initiale.",
  ],
  "vize:croquis/cf/deep-import": [
    "Une chaîne d’imports est plus profonde que ce que le projet autorise.",
    "Le point d’entrée fait transiter une valeur simple par `level-one`, `level-two` et `level-three`, créant une chaîne d’imports inutilement profonde pour un projet qui souhaite une interface publique peu profonde.",
    "Le point d’entrée utilise `public-api.ts`, qui réexporte directement la valeur. Le consommateur conserve le même nom importé tandis que la chaîne se raccourcit.",
  ],
  "vize:croquis/cf/destructuring-breaks-reactivity": [
    "La déstructuration d’un objet réactif copie ses champs et supprime leur suivi.",
    "La déstructuration ordinaire de l’objet `props` copie sa valeur actuelle de `item` ; elle est distincte de la déstructuration directe de `defineProps()` dans Vue 3.5.",
    '`toRef(props, "item")` conserve le lien avec la propriété de `props`.',
  ],
  "vize:croquis/cf/di-outside-setup": [
    "`provide` ou `inject` est appelé en dehors de `setup`.",
    "`main.ts` appelle le `provide` de composant sans instance de composant active. Le `inject` de l’enfant ne peut donc pas recevoir cette valeur prévue de l’ancêtre et utilise `light`.",
    "App appelle le fournisseur depuis son setup avant de rendre l’enfant. L’enfant hérite désormais de la valeur `dark` de son ancêtre composant.",
  ],
  "vize:croquis/cf/dom-access-without-next-tick": [
    "Le DOM est lu avant que Vue ait appliqué la mise à jour.",
    "Le gestionnaire de clic incrémente `count` et lit immédiatement le paragraphe rendu, avant que Vue applique la mise à jour du DOM planifiée. `sampled` peut contenir le décompte précédent.",
    "Attendre `nextTick()` après l’écriture de l’état permet à Vue de mettre à jour le paragraphe avant que `readLabel` relève son texte.",
  ],
  "vize:croquis/cf/duplicate-id": [
    "Le même identifiant d’élément est utilisé dans plusieurs composants.",
    'Les composants de livraison et de facturation accessibles rendent tous deux `id="postal-code"` ; leurs libellés partagent donc une cible ambiguë dans le document.',
    "Chaque composant appelle `useId()` et lie sa propre valeur au libellé et au champ de saisie, conservant leur association sans répéter un identifiant littéral.",
  ],
  "vize:croquis/cf/event-listener-leak": [
    "Un écouteur d’événement est enregistré sans jamais être supprimé.",
    "Le montage ajoute un écouteur de redimensionnement de window qui capture la ref de largeur du composant, mais le démontage ne le supprime jamais. Des montages répétés peuvent conserver des écouteurs et un état inutilisés.",
    "`onUnmounted` supprime exactement la même fonction `resize` que celle enregistrée au montage, mettant fin à la durée de vie de l’écouteur externe de cette instance.",
  ],
  "vize:croquis/cf/event-modifier": [
    "Un écouteur d’événement utilise un modificateur que l’événement émis ne prend pas en charge.",
    "`.stop` suppose que l’événement personnalisé `save` de l’enfant possède la méthode de propagation d’un événement natif, alors que son argument n’est pas nécessairement un DOM Event.",
    "Supprimer `.stop` de l’écouteur d’événement personnalisé ; gérer la propagation native au niveau de l’écouteur DOM réel lorsque nécessaire.",
  ],
  "vize:croquis/cf/hydration-risk": [
    "Ce code regroupe plusieurs diagnostics de réactivité, dont une prop copiée dans une ref. Il ne signifie pas que chaque expression Date.now() est détectée par la passe inter-fichiers.",
    "L’enfant initialise `ref(props.count)` une seule fois ; son décompte local ne suit donc plus les changements ultérieurs de la prop du parent. Il s’agit du producteur actuel pour la copie d’une prop dans une ref, et non d’un exemple général de SSR non déterministe.",
    '`toRef(props, "count")` pointe vers la prop au lieu de copier sa valeur initiale dans un état indépendant.',
  ],
  "vize:croquis/cf/inherit-attrs-unused": [
    "`inheritAttrs: false` est défini et le composant ne lit jamais les attributs.",
    'L’enfant définit `inheritAttrs: false`, mais ne transmet jamais l’attribut `class="notice"` du parent.',
    "Conserver le contrôle explicite de l’héritage et lier `$attrs` à la cible `<main>` prévue.",
  ],
  "vize:croquis/cf/inject-without-symbol": [
    "`inject` utilise une clé ordinaire au lieu d’un symbole `InjectionKey`.",
    'Le consommateur injecte la clé chaîne non typée `"theme"`, qui ne fournit aucune identité de symbole partagée avec le fournisseur.',
    "Le consommateur et le fournisseur importent le même `ThemeKey` au lieu de dupliquer des noms sous forme de chaînes.",
  ],
  "vize:croquis/cf/injected-async-mutation-race": [
    "Une valeur injectée est modifiée par une tâche asynchrone susceptible de provoquer une condition de concurrence.",
    "`CountLoader.vue` écrit directement le résultat obtenu après attente dans le store injecté partagé avec `CountSummary.vue`, permettant à un travail périmé d’affecter les deux consommateurs.",
    "Le chargeur annule le travail invalidé et n’émet qu’un résultat actif. Le fournisseur se charge de modifier le store via `applyLoadedCount`.",
  ],
  "vize:croquis/cf/lifecycle-outside-setup": [
    "Un hook de cycle de vie est enregistré en dehors de `setup`.",
    "Le point d’entrée appelle `installTitle()` avant de monter une application ; `onMounted` est donc enregistré sans contexte setup de composant actif.",
    "Appeler le même utilitaire de manière synchrone depuis le setup d’App associe la fonction de rappel de cycle de vie au montage de cette instance.",
  ],
  "vize:croquis/cf/lifecycle-without-cleanup": [
    "Un hook de cycle de vie lance un travail sans jamais le nettoyer.",
    "Le montage enregistre un écouteur de redimensionnement de window, mais le démontage ne supprime jamais cette même fonction de rappel.",
    "`onUnmounted` supprime l’écouteur avec le même nom d’événement et la même identité de fonction que ceux utilisés par `addEventListener`.",
  ],
  "vize:croquis/cf/missing-required-prop": [
    "Une prop obligatoire n’est pas transmise.",
    "Le parent rend `<Child />` sans la prop obligatoire `title: string` de l’enfant.",
    '`title="Hello"` fournit la prop obligatoire déclarée par l’enfant résolu.',
  ],
} satisfies Record<string, readonly [string, string, string]>;
