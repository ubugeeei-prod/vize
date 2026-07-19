---
title: Tests et retour d’information
---

<!-- Generated translation; source: guide/testing.md -->

# Tests et retour d’information

Vize en est dans sa phase **de Test en Monde Réel** : l’accent est mis sur la justesse, et les projets réels sont
suite de tests. Cette page s’adresse aux testeurs — comment inspecter ce que fait Vize, où consulter, comment
les résultats des rapports, comment mesurer la performance, et comment proposer votre projet comme banc d’essai.

## Inspectez avec le terrain de jeu

Le playground propose un **inspecteur** qui affiche, côte à côte, la sortie officielle du compilateur SFC de Vue
, la sortie du compilateur Vize, le Virtual TS généré, le VIR, et un graphe croisé pour les lots locaux de
. C’est le moyen le plus rapide de voir exactement où Vize est d’accord ou en désaccord avec Vue pour un fichier
`.vue` donné.

- Ouvrez-le à <https://vizejs.dev/play/?tab=inspector>.
- Consultez le guide [Compiler Inspector](./compiler-inspector.md) pour savoir ce que chaque surface signifie.

Un lien d’inspecteur de terrain de jeu est une excellente reproduction de réparation.

## Lisez les cas de test

Vize est testé de manière intensive et de multiples manières — les fixatures du compilateur comparées au compilateur officiel
Vue, la parité des vérifications de type avec `vue-tsc`, des instantanés de lint et de formateur, des instantanés de code SSR
, des faisceaux de fuzz et des éléments d’application réels. Avant de déposer un rapport, il est souvent
utile de survoler les dossiers existants :

- Compilateur et SFC parité et snapshots sous `tests/` et chaque caisse `src/snapshots/`.
- Éléments d’applications réelles sous `tests/_fixtures/` (par exemple Elk, Misskey, Nuxt UI,
  Reka UI et VOICEVOX) qui pilotent E2E et VRT.

Si un cas manque ou un résultat semble erroné, c’est exactement le type de retour que cette phase recherche.

## Conclusions du rapport

- **Le texte brut est correct.** Une description claire de ce que tu as fait, de ce que tu attendais, et de ce qui s’est passé
  est déjà précieux.
- **Si possible, attachez une reproduction minimale** au suivi GitHub - le plus petit fichier `.vue` (ou
  petit projet) qui montre toujours le problème. Un lien Playground Inspector fonctionne très bien.
- Les rapports de correction, les reproductions, les résultats de benchmark et les résultats de compatibilité aident tous. Voir le
  [Contributing](../contributing.md) guide et
  [Support](https://github.com/ubugeeei-prod/vize/blob/main/SUPPORT.md).

## Mesure de la performance

Vize a un **mode profilage** intégré, ce qui permet de mesurer où va le temps au lieu de deviner.

- Dans le développement local, la chaîne d’outils expose le profilage à travers l’analyseur, le compilateur, l’analyse, et
  phases de contrôle de type.
- Le CLI l’a aussi : `vize check --profile` passe la vérification par **vize_conservateur** et imprime un
  rapport de profilage par phase. Utilisez-le pour capturer et partager des chiffres de performance à partir de votre propre base de code
  .

## Proposez votre projet comme banc d’essai

De vraies bases de code importantes trouvent les échecs que les exemples synthétiques ne trouveront jamais. **Lorsque la licence
le permet, un projet peut être ajouté aux installations de Vize et devenir un**cible E2E / VRT, afin que les régressions futures de
soient automatiquement détectées.

Si vous maintenez (ou connaissez) une application, une bibliothèque, un framework ou un outil Vue pouvant être utilisé de cette
manière, merci de nous en informer - ouvrez une demande de correction ou contactez-nous. Plus la base de code est grande et réelle, plus le signal
plus utile.
