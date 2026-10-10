export const frenchShared = {
  "Shared project files": "Fichiers communs du projet",
  "Example qualification": "Qualification de l’exemple",
  "Public explanation": "Explication publique",
  Producer: "Producteur de diagnostics",
  "Cross-file index": "Index des règles inter-fichiers",
  "Experimental Rust analyzer; not an individual CLI code":
    "Analyseur Rust expérimental ; code non émis individuellement par le CLI",
  "Contract; no current producer": "Contrat ; aucun producteur actuel",
  "Project-specific lint ID": "Identifiant de lint propre au projet",
  "The public CLI exposes the same pass with `vize lint --cross-file`. Displayed `vize:croquis/cf/*` codes use `croquis/cf/*` in `lint.vize.rules` (omit `vize:`). Information/hint diagnostics become CLI warnings. Related locations explain the source/consumer relationship.":
    "Le CLI public expose la même passe avec `vize lint --cross-file`. Les codes affichés `vize:croquis/cf/*` utilisent `croquis/cf/*` dans `lint.vize.rules` (sans `vize:`). Les diagnostics de niveau information/hint deviennent des avertissements CLI. Les emplacements associés expliquent la relation entre la source et le consommateur.",
  "Not emitted": "Non émis",
  "context-dependent": "Selon le contexte",
  None: "Aucune",
  "error / warning (with default)": "error / warning (avec valeur par défaut)",
  "None; review related files and apply the repair":
    "Aucune ; examinez les fichiers concernés et appliquez la correction",
  "Analyzed component graph and the supported facts described below":
    "Graphe de composants analysé et faits pris en charge décrits ci-dessous",
  "CSS inside SFC style blocks": "CSS dans les blocs style des SFC",
  "HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.":
    "Documents HTML détectés comme petite-vue ; les SFC Vue ordinaires sont hors du champ de cette règle.",
  "JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form":
    "Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup",
  "Musea .art.vue art, variant, and style blocks":
    "Blocs art, variant et style des fichiers .art.vue de Musea",
  "Nuxt configuration files (nuxt.config.ts)": "Fichiers de configuration Nuxt (nuxt.config.ts)",
  "Reachable project declarations and imported components":
    "Déclarations du projet accessibles et composants importés",
  "Type information in Vue SFC scripts and templates, for the constructs shown below":
    "Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous",
  "Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts":
    "Contrôles de script destinés à Vapor ; une activation explicite applique aussi la restriction aux scripts ordinaires",
  "No per-code options; supported CLI findings accept severity overrides":
    "Aucune option propre à un code ; les diagnostics CLI pris en charge acceptent des changements de gravité",
  "crossFile; rule severity (off/warn/error)": "crossFile ; gravité de la règle (off/warn/error)",
  "A single SFC root link may inherit its target from parent attributes. This example uses a nested link, whose target must be explicit.":
    "Un lien constituant l’unique racine d’un SFC peut hériter de sa destination via les attributs du parent. Cet exemple utilise un lien imbriqué, dont la destination doit être explicite.",
  "Checks non-interactive elements without an interactive role. Native buttons and elements with an interactive ARIA role are outside this rule's finding.":
    "Contrôle les éléments non interactifs sans rôle interactif. Les boutons natifs et les éléments possédant un rôle ARIA interactif sont hors du champ du diagnostic de cette règle.",
  "Enable typeAware and this rule explicitly. The default disallows nullable numbers, while non-null numbers are allowed.":
    "Activez explicitement typeAware et cette règle. Par défaut, les nombres pouvant être nuls sont interdits, tandis que les nombres non nuls sont autorisés.",
  "Historical Vue 2.7 only: use matching Vue 2.7 and SFC compiler dependencies for this scenario. Vue 3 proxies track array index assignment, so `items[0] = next` is reactive in Vue 3 and is not a Vue 3 defect. This published code has no current producer.":
    "Uniquement pour l’ancien Vue 2.7 : utilisez des dépendances Vue 2.7 et de compilateur SFC correspondantes pour ce scénario. Les proxies de Vue 3 suivent les affectations aux indices des tableaux ; `items[0] = next` est donc réactif en Vue 3 et ne constitue pas un défaut de Vue 3. Ce code publié n’a actuellement aucun producteur de diagnostics.",
  "Intentional shared application stores may export reactive state. This scenario requires isolated state and does not claim every reactive export is invalid. No current producer emits this contract.":
    "Les stores applicatifs volontairement partagés peuvent exporter un état réactif. Ce scénario exige un état isolé et ne prétend pas que tout export réactif est invalide. Aucun producteur actuel n’émet ce contrat.",
  "Module-scope reactive state is legal for intentional application stores. This example assumes component/request isolation; the published contract currently has no producer.":
    "Un état réactif à la portée du module est autorisé pour des stores applicatifs conçus à cet effet. Cet exemple suppose une isolation par composant ou requête ; le contrat publié n’a actuellement aucun producteur de diagnostics.",
  "Module-scope watchers are valid when their owner keeps and calls a stop handle or intentionally gives them application lifetime. This example requires component-owned lifetimes; the contract has no current producer.":
    "Les watchers à la portée du module sont valides lorsque leur propriétaire conserve et appelle une fonction d’arrêt, ou leur donne volontairement la durée de vie de l’application. Cet exemple exige des durées de vie appartenant au composant ; le contrat n’a actuellement aucun producteur de diagnostics.",
  "Pinia must be installed, and main.ts installs its plugin before mounting. Reading `store.doubled` directly inside a tracked computation or template is valid; the defect here is taking a plain snapshot. This contract currently has no producer.":
    "Pinia doit être installé, et main.ts installe son plugin avant le montage. Lire directement `store.doubled` dans un calcul suivi ou un template est valide ; le défaut consiste ici à prendre un simple instantané. Ce contrat n’a actuellement aucun producteur de diagnostics.",
  "Refs may legitimately be returned from composables or shared across scopes. This example explicitly requires a snapshot cache; it does not claim that unmount invalidates a ref. No current producer emits this contract.":
    "Les refs peuvent légitimement être renvoyées par des composables ou partagées entre des portées. Cet exemple exige explicitement un cache d’instantanés ; il ne prétend pas que le démontage invalide une ref. Aucun producteur actuel n’émet ce contrat.",
  "Requires an .art.vue file and the token inventory shown below. It does not infer a token from an arbitrary color.":
    "Nécessite un fichier .art.vue et l’inventaire des tokens présenté ci-dessous. Aucun token n’est déduit d’une couleur arbitraire.",
  "Suspense without a fallback is valid Vue syntax. This is a chosen loading-UI convention, not a compiler error; the published contract has no current producer.":
    "Suspense sans contenu de repli est une syntaxe Vue valide. Il s’agit d’une convention choisie pour l’interface de chargement, et non d’une erreur de compilation ; le contrat publié n’a actuellement aucun producteur de diagnostics.",
  "The Good files demonstrate the change described above; other diagnostics can still apply to the complete project.":
    "Les fichiers de l’exemple Bon montrent la modification décrite ci-dessus ; d’autres diagnostics peuvent encore s’appliquer au projet complet.",
  "The boundary producer reads macros.is_async(), but source parsing currently records top-level await on the script-setup scope instead. The complete Bad/Good source pair below therefore produces no async-no-suspense finding through the current CLI. It explains the Suspense convention; supplying the missing macro fact is implementation follow-up work.":
    "Le producteur de diagnostics de cette frontière lit macros.is_async(), mais l’analyse du code source enregistre actuellement le await de premier niveau dans la portée script-setup à la place. La paire complète de sources Mauvais/Bon ci-dessous ne produit donc aucun diagnostic async-no-suspense via le CLI actuel. Elle explique la convention Suspense ; fournir le fait de macro manquant reste un travail d’implémentation à réaliser.",
  "The complete Vue project below illustrates update feedback and its repair. It is not a qualified CLI finding witness: the diagnostic producer requires retained reactive-flow reference identities and edges, as shown by the accompanying graph. These sources do not establish that the current source path will emit this exact code. Dedicated tracked-ID graph finding controls remain separate from source grammar checks.":
    "Le projet Vue complet ci-dessous illustre une boucle de rétroaction de mises à jour et sa correction. Il ne constitue pas un témoin qualifié de diagnostic CLI : le producteur de diagnostics exige des identités de références et des arêtes de flux réactif conservées, comme le montre le graphe joint. Ces sources ne prouvent pas que le traitement actuel du code source émettra ce code précis. Les contrôles dédiés de diagnostics sur les graphes à identifiants suivis restent distincts des contrôles de grammaire du code source.",
  "The complete installed router must be reachable from the application's createApp(...).use(router). Unknown/dynamic route tables do not prove unknown-name findings. Missing params are warnings because navigation may inherit a value from the current route.":
    "Le routeur installé complet doit être accessible depuis createApp(...).use(router) dans l’application. Des tables de routes inconnues ou dynamiques ne prouvent pas les diagnostics de noms inconnus. Les paramètres manquants sont des avertissements, car la navigation peut hériter d’une valeur de la route actuelle.",
  "The concern is this lifecycle-dependent composable, not a blanket ban on ordinary utility functions or all Composition API calls outside setup. This contract has no current producer.":
    "Le problème concerne ce composable dépendant du cycle de vie ; il ne s’agit pas d’une interdiction générale des fonctions utilitaires ordinaires ou de tous les appels à la Composition API en dehors du setup. Ce contrat n’a actuellement aucun producteur de diagnostics.",
  "The current producer scans template expressions such as JSON.parse(input). It does not report a throw statement that exists only in the script block.":
    "Le producteur actuel analyse les expressions de template telles que JSON.parse(input). Il ne signale pas une instruction throw présente uniquement dans le bloc script.",
  "The example assumes IDs uniquely identify records. Comparing two references to the same reactive proxy remains valid; this contract has no current producer.":
    "L’exemple suppose que les identifiants distinguent les enregistrements de façon unique. Comparer deux références au même proxy réactif reste valide ; ce contrat n’a actuellement aucun producteur de diagnostics.",
  "The experimental Rust CrossFileAnalyzer has a producer for this code. The CLI pass does not emit this individual code; configuring its ID does not enable that Rust pass. These scenarios describe the analyzer's supported graph/facts, not a Vite+ promise.":
    "L’analyseur Rust expérimental CrossFileAnalyzer possède un producteur pour ce code. La passe CLI n’émet pas ce code individuel ; configurer son identifiant n’active pas cette passe Rust. Ces scénarios décrivent le graphe et les faits pris en charge par l’analyseur, sans constituer une promesse de Vite+.",
  "The watcher must only derive the destination. Editable copies and callbacks with other side effects are allowed.":
    "Le watcher doit uniquement dériver la destination. Les copies modifiables et les callbacks ayant d’autres effets de bord sont autorisés.",
  "This check compares explicit provider/consumer type annotations, not inferred literal value types. Keep the provider's `as string` annotation in this example.":
    "Ce contrôle compare les annotations de type explicites du fournisseur et du consommateur, et non les types de valeurs littérales inférés. Conservez l’annotation `as string` du fournisseur dans cet exemple.",
  "This example configures window.localStorage. The rule has no default deny list; enabling it alone does not report a member.":
    "Cet exemple configure window.localStorage. La règle n’a aucune liste d’interdiction par défaut ; son activation seule ne signale pas de membre.",
  "This example uses component provide/inject. `app.provide` and supported `app.runWithContext` injection are different valid ownership surfaces, not prohibited by this scenario. No current producer emits this contract code.":
    "Cet exemple utilise provide/inject dans les composants. `app.provide` et l’injection prise en charge avec `app.runWithContext` sont d’autres contextes de propriété valides, que ce scénario n’interdit pas. Aucun producteur actuel n’émet ce code de contrat.",
  "This illustrates a concrete eager-initialization cycle. A recursive Vue component or every circular import is not automatically erroneous. No current producer emits this contract code.":
    "Cela illustre un cycle concret d’initialisation immédiate. Un composant Vue récursif ou un import circulaire n’est pas automatiquement erroné. Aucun producteur actuel n’émet ce code de contrat.",
  "This illustrates the published preference for purely derived state. Watchers remain appropriate for external effects or independently writable state; no current producer emits this contract.":
    "Cela illustre la préférence publiée pour un état purement dérivé. Les watchers restent appropriés pour les effets externes ou les états modifiables indépendamment ; aucun producteur actuel n’émet ce contrat.",
  "This is a published diagnostic contract without a current producer. The Bad/Good scenario below explains the risk and repair; no flag currently makes this code trigger.":
    "Il s’agit d’un contrat de diagnostic publié sans producteur actuel. Le scénario Mauvais/Bon ci-dessous explique le risque et la correction ; aucune option ne permet actuellement de déclencher ce code.",
  "This is an explicit immutable-history ownership policy, not a general prohibition on passing or later mutating reactive objects. No current producer emits this contract.":
    "Il s’agit d’une politique explicite de propriété d’un historique immuable, et non d’une interdiction générale de transmettre des objets réactifs ou de les modifier ensuite. Aucun producteur actuel n’émet ce contrat.",
  "This is an explicitly chosen project layout policy; it does not invent a supported depth threshold or option. There is no current diagnostic producer for this contract.":
    "Il s’agit d’une politique d’organisation du projet choisie explicitement ; elle n’invente ni seuil de profondeur ni option pris en charge. Aucun producteur de diagnostics actuel n’existe pour ce contrat.",
  "This rule is a placeholder with an empty callback. Adding vapor selects Vapor compilation; the current linter does not report this catalog ID for its absence.":
    "Cette règle est un emplacement réservé doté d’un callback vide. Ajouter vapor sélectionne la compilation Vapor ; le linter actuel ne signale pas cet identifiant du catalogue lorsque vapor est absent.",
  "Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.":
    "Les contrôles utilisant les types s’appuient sur le runtime natif Corsa et le projet TypeScript. `typeAware` seul n’active pas une règle nécessitant une activation explicite.",
  "Use a block-body arrow for the currently supported SFC filter. The underlying validator also handles method shorthand, but the current SFC prefilter does not reliably dispatch that shape.":
    "Utilisez une fonction fléchée à corps de bloc pour le filtre SFC actuellement pris en charge. Le validateur sous-jacent gère aussi la syntaxe abrégée des méthodes, mais le préfiltre SFC actuel ne transmet pas cette forme de façon fiable.",
  "Use these unchanged files in both Bad and Good. Install the imported packages in the project: Vue, plus vue-router or Pinia where shown. Follow any version-specific support note. The entry root makes the component relationship explicit.":
    "Utilisez ces fichiers inchangés dans les exemples Mauvais et Bon. Installez les packages importés dans le projet : Vue, ainsi que vue-router ou Pinia lorsqu’ils sont indiqués. Respectez toute note de prise en charge propre à une version. Le point d’entrée racine rend explicite la relation entre les composants.",
  "Vue permits ref/reactive/computed outside component setup. The risk here is unwanted ownership/sharing under an explicit instance-isolation policy, not API illegality. This contract has no current producer.":
    "Vue autorise ref/reactive/computed en dehors du setup des composants. Le risque concerne ici une propriété ou un partage indésirable dans le cadre d’une politique explicite d’isolation des instances, et non un usage interdit de l’API. Ce contrat n’a actuellement aucun producteur de diagnostics.",
} satisfies Record<string, string>;
