export const frenchRules7 = {
  "type/await-thenable": [
    "Interdire d’await une valeur qui n’est pas thenable.",
    "plainNumber renvoie un nombre, donc await attend une valeur qui n’est pas thenable.",
    "Promise.resolve(1) est thenable, donc await attend une vraie Promise.",
  ],
  "type/no-array-delete": [
    "Interdire delete sur un élément d’une valeur de type tableau.",
    "delete numbers[0] retire un élément et laisse un trou au lieu de réindexer le tableau.",
    "splice(0, 1) retire le premier élément et garde le tableau dense.",
  ],
  "type/no-base-to-string": [
    "Interdire la conversion en chaîne des valeurs qui retombent sur Object.prototype.toString().",
    "plain est un objet simple, donc toString() retombe sur Object.prototype.toString().",
    "label est déjà une chaîne, donc toString() utilise la conversion de chaîne.",
  ],
  "type/no-confusing-void-expression": [
    "Interdire une expression de type void là où une valeur est attendue.",
    "logVoid() renvoie void, mais l’initialiseur traite cet appel comme une valeur.",
    "Appeler logVoid() comme instruction n’utilise le résultat void que pour son effet.",
  ],
  "type/no-deprecated": [
    "Interdire l’usage d’une déclaration obsolète.",
    "legacy est marquée @deprecated, et l’appel utilise encore cette déclaration.",
    "current n’a pas de marque d’obsolescence, donc l’appel utilise une déclaration prise en charge.",
  ],
  "type/no-duplicate-type-constituents": [
    "Interdire un constituant dupliqué dans une union ou une intersection.",
    "string apparaît deux fois dans la même union, donc le second constituant n’ajoute rien.",
    "L’alias ne conserve qu’un seul constituant string.",
  ],
  "type/no-for-in-array": [
    "Interdire for-in sur une valeur de type tableau.",
    "for-in parcourt les clés d’un tableau au lieu des valeurs des éléments.",
    "for-of parcourt les éléments numériques du tableau.",
  ],
  "type/no-generated-empty-object-type": [
    "Interdire un type d’objet vide produit par une opération de type.",
    "Omit retire la seule clé, donc l’alias devient un type d’objet vide.",
    "Omit retire a et laisse b, donc l’alias n’est pas un type d’objet vide.",
  ],
  "type/no-implied-eval": [
    "Interdire une API d’exécution dynamique à partir d’une chaîne.",
    "setTimeout reçoit une chaîne, qui est exécutée comme du code dynamique.",
    "setTimeout reçoit une fonction, donc aucune chaîne n’est évaluée.",
  ],
  "type/no-meaningless-void-operator": [
    "Interdire l’opérateur void sur une expression qui est déjà void.",
    "logVoid() est déjà void, donc l’opérateur void ne change pas le résultat.",
    "getNumber() renvoie une Promise, donc void abandonne ce thenable volontairement.",
  ],
  "type/no-misused-promises": [
    "Interdire une Promise là où elle n’est pas prise en charge.",
    "forEach attend un callback void, mais le callback async renvoie une Promise.",
    "Le callback n’est pas async, donc il renvoie void comme forEach l’attend.",
  ],
  "type/no-misused-spread": [
    "Interdire d’étaler une valeur dans une position incompatible.",
    "Étaler une chaîne dans un tableau place une valeur dans une position incompatible.",
    "Étaler un tableau dans un tableau garde le type des éléments compatible.",
  ],
  "type/no-mixed-enums": [
    "Interdire de mélanger des membres chaîne et numériques dans un enum.",
    "L’enum mélange un membre numérique et un membre chaîne.",
    "Les deux membres sont numériques, donc l’enum a un seul domaine de valeurs.",
  ],
  "type/no-redundant-type-constituents": [
    "Interdire un membre d’union ou d’intersection qui n’apporte rien ou écrase l’information de type.",
    "unknown couvre déjà string, donc le constituant string ne restreint pas l’union.",
    "string et number sont tous deux des constituants utiles de l’union.",
  ],
  "type/no-unnecessary-boolean-literal-compare": [
    "Interdire une comparaison inutile avec un littéral booléen.",
    "flag est déjà booléen, donc le comparer à true n’ajoute aucun test.",
    "Renvoyer flag utilise directement la valeur booléenne.",
  ],
  "type/no-unnecessary-condition": [
    "Interdire une condition toujours vraie, toujours fausse, never ou toujours nullish.",
    "value est une chaîne, donc elle n’est jamais nullish et le repli de ?? ne s’exécute jamais.",
    "value peut être undefined, donc ?? fournit une chaîne lorsqu’elle est absente.",
  ],
  "type/no-unnecessary-template-expression": [
    "Interdire un littéral de gabarit qui n’enveloppe qu’une expression.",
    "Le gabarit ne contient que label, donc il ne construit pas une nouvelle chaîne.",
    "Affecter label utilise la chaîne sans enveloppe de gabarit.",
  ],
  "type/no-unnecessary-type-arguments": [
    "Interdire un argument de type égal à la valeur par défaut.",
    "L’argument explicite number est identique au paramètre de type par défaut.",
    "Appeler identity(1) laisse s’appliquer le paramètre de type par défaut.",
  ],
  "type/no-unnecessary-type-assertion": [
    "Interdire une assertion de type qui ne change pas le type de l’expression.",
    "known est déjà number, donc l’assertion number ne change pas son type.",
    "Utiliser known sans assertion conserve le type number annoté.",
  ],
  "type/no-unnecessary-type-conversion": [
    "Interdire une conversion qui ne change ni le type ni la valeur de l’expression.",
    "label est déjà une chaîne, donc String(label) ne change ni son type ni sa valeur.",
    "count est un nombre, donc String(count) le convertit en chaîne.",
  ],
  "type/no-unnecessary-type-parameters": [
    "Interdire un paramètre de type utilisé une seule fois dans sa signature.",
    "T n’apparaît qu’une fois dans la signature de logValue, donc il ne relie rien.",
    "Typer value en unknown accepte les mêmes arguments sans paramètre de type.",
  ],
  "type/no-unsafe-argument": [
    "Interdire de passer une valeur de type `any` à une fonction.",
    "anyValue est any et il est passé à un paramètre qui attend number.",
    "L’appel passe le nombre 1 au paramètre number.",
  ],
  "type/no-unsafe-assignment": [
    "Interdire d’affecter une valeur any à une cible plus précise.",
    "anyValue est any et il est affecté à une liaison annotée number.",
    "La liaison est initialisée avec le nombre 1.",
  ],
  "type/no-unsafe-call": [
    "Interdire d’appeler une valeur typée any.",
    "anyValue a le type any, donc l’appel ne vérifie pas la cible.",
    "plainNumber est une fonction, donc l’appel a une cible connue.",
  ],
  "type/no-unsafe-enum-assignment": [
    "Interdire d’affecter une valeur étrangère à un enum.",
    "La liaison est annotée Color, mais elle est initialisée avec le nombre 1.",
    "Color.Red est un membre de l’enum Color.",
  ],
  "type/no-unsafe-enum-comparison": [
    "Interdire de comparer une valeur d’enum avec une valeur d’un autre domaine.",
    "value est un nombre, donc le comparer au membre Color.Red traverse deux domaines.",
    "value est Color, donc la comparaison reste dans l’enum Color.",
  ],
  "type/no-unsafe-member-access": [
    "Interdire l’accès à un membre d’une valeur typée any.",
    "anyValue est any, donc lire member est un accès de propriété non vérifié.",
    "box a une propriété member connue, donc l’accès est vérifié.",
  ],
  "type/no-unsafe-return": [
    "Interdire de renvoyer une valeur any depuis une fonction.",
    "La fonction promet un number, mais elle renvoie anyValue, qui est any.",
    "La fonction renvoie le nombre 1, qui correspond au type de retour annoté.",
  ],
  "type/no-unsafe-unary-minus": [
    "Interdire le moins unaire sur une valeur qui n’est ni number ni bigint.",
    "label est une chaîne, donc le moins unaire ne nie ni un number ni un bigint.",
    "count est un nombre, donc le moins unaire nie ce nombre.",
  ],
  "type/only-throw-error": [
    "Exiger que la valeur lancée soit de type Error.",
    "La valeur lancée est une chaîne, qui n’est pas une Error.",
    "new Error crée une Error pour le même texte d’échec.",
  ],
  "type/prefer-promise-reject-errors": [
    "Exiger qu’une raison de rejet de Promise soit une Error.",
    "Promise.reject reçoit une chaîne, qui n’est pas une raison Error.",
    "Promise.reject reçoit une Error pour le même texte d’échec.",
  ],
  "type/prefer-reduce-type-parameter": [
    "Préférer le paramètre de type de Array#reduce à une assertion de la valeur initiale.",
    "La valeur initiale est affirmée comme number[] au lieu d’utiliser le paramètre de type de reduce.",
    "reduce<number[]> indique le type de l’accumulateur sans affirmer le tableau initial.",
  ],
  "type/prefer-return-this-type": [
    "Exiger le type `this` lorsqu’une méthode ne renvoie que `this`.",
    "setValue renvoie this, mais le type de retour déclaré est le nom de classe Builder.",
    "Le type de retour this correspond à la valeur renvoyée par la méthode.",
  ],
  "type/related-getter-setter-pairs": [
    "Exiger que les getter et setter appariés utilisent des types compatibles.",
    "Le getter renvoie string alors que le setter accepte number, donc la paire est incompatible.",
    "Le getter et le setter utilisent tous deux string.",
  ],
  "type/require-await": [
    "Interdire une fonction async qui ne contient aucun await.",
    "noAwait est async, mais son corps n’attend aucune valeur.",
    "withAwait attend Promise.resolve(1) avant de le renvoyer.",
  ],
  "type/restrict-plus-operands": [
    "Exiger que les opérandes de + soient explicitement compatibles.",
    "label est une chaîne et numbers est un tableau, donc les deux opérandes de + ne sont pas des primitives compatibles.",
    "label et extra sont deux chaînes, donc les opérandes de + correspondent.",
  ],
  "type/restrict-template-expressions": [
    "Exiger que les expressions d’un littéral de gabarit soient compatibles avec une chaîne.",
    "Le gabarit interpole un tableau, qui n’est pas une expression compatible avec une chaîne.",
    "Le gabarit interpole label, qui est une chaîne.",
  ],
  "type/return-await": [
    "Exiger un usage cohérent de `return await` selon le contexte et la configuration.",
    "Dans un try, renvoyer getNumber() sans await fait perdre le rejet à ce catch.",
    "return await getNumber() garde le rejet dans le même try et le même catch.",
  ],
  "type/unbound-method": [
    "Interdire une référence à une méthode sans son receveur (`this`).",
    "counter.increment prend la méthode sans son receveur, donc this serait perdu à l’appel.",
    "counter.increment() appelle la méthode sur counter, donc this reste lié.",
  ],
  "type/use-unknown-in-catch-callback-variable": [
    "Exiger une annotation unknown explicite pour la variable du callback catch d’une Promise.",
    "Le callback catch nomme error sans annotation unknown explicite.",
    "Le callback catch annote error comme unknown.",
  ],
} satisfies Record<string, readonly [string, string, string]>;
