---
title: Compatibilité Babel JSX
---

<!-- Generated translation; source: guide/jsx-babel-compat.md -->

# Compatibilité Babel JSX

> **Statut :** option d’adhésion et désactivation par défaut. `compiler.jsxCompat` est lu par le chargeur de configuration et
> honoré par les `compileJsx` liaisons ; Les plugins bundler ne le transmettent pas encore au compilateur.
> La section « Activation » ci-dessous explique ce qui fonctionne aujourd’hui.

Vize compile `.jsx` et `.tsx` via ses propres caisses de compilation, donc la sortie est
forme de compilateur modèle : un arbre de blocs, `v-if` / `v-for` supprimés du JavaScript, et patch
drapeaux sur chaque nœud. [`@vue/babel-plugin-jsx`](https://github.com/vuejs/babel-plugin-jsx) ne fait rien de tout cela — il émet
appels `createVNode` nus, n’ouvre jamais de bloc, laisse `&&`, `?:` et `.map()` comme
JavaScript pur, et par défaut n’émet aucun drapeau de correctif.

La plupart de cette différence est invisible à l’exécution. Le reste, c’est à quoi sert ce switch : un projet
migre du plugin Babel a besoin d’un moyen de demander la sémantique du plugin au lieu de celle de Vize.
`compiler.jsxCompat: "babel"` est cet interrupteur.

Cette page porte sur **la sémantique de compatibilité**. Pour l’API d’auteur, la surface de type et le sélecteur de sortie
Vapor/VDOM, voir le [JSX & TSX guide](./jsx.md).

## La facilitation

```json
{
  "compiler": {
    "jsxCompat": "babel"
  }
}
```

La clé accepte `"native"` (le défaut) et `"babel"`. Toute autre valeur revient à `"native"`
plutôt qu’à échouer dans la compilation, correspondant à la façon dont un `jsxMode` non reconnu est géré : une valeur
de configuration errante ne doit jamais bloquer la compilation.

La même valeur est acceptée directement par les liaisons `compileJsx` , c’est là que le mode prend
effet aujourd’hui :

```js
import { compileJsx } from "@vizejs/native";

const result = compileJsx(source, {
  filename: "App.tsx",
  lang: "tsx",
  jsxCompat: "babel",
});
```

`@vizejs/wasm` expose la même option `jsxCompat`. Les plugins bundler
(`@vizejs/vite-plugin`, `@vizejs/unplugin`, `@vizejs/rspack-plugin`, `@vizejs/nuxt`) passent actuellement
`jsxMode` et `vapor` à `compileJsx` mais pas `jsxCompat`, donc le simple réglage de la clé de configuration
ne change pas encore ce qu’un bundler émet. Ce câblage est suivi sur
[#3391](https://github.com/ubugeeei-prod/vize/issues/3391).

## Pourquoi c’est un consentement volontaire et au niveau projet

**Désactivé par défaut.** `"native"` est le défaut et doit rester le par défaut. Le retourner modifiait
silencieusement la sortie émise pour chaque projet Vize existant, aucun ne demandant babel
sémantique.

**au niveau du projet, sans forme par composant.** `jsxMode` peuvent être sélectionnés par composant avec un prologue
`"use vue:vapor"` / `"use vue:vdom"`, car les composants VDOM et Vapor coexistent parfaitement dans
seul module — chacun est une fonction de rendu indépendante. Le mode de compatibilité n’est pas comme ça. Il modifie
la forme de **sortie au niveau du module** : le plugin babel réécrit l’expression JSX en place,
`const A = () => <div />` reste un `const A = …`, tandis que Vize émet un `render` autonome à exporter. Un module
compilé à moitié en mode compat et moitié hors de celui-ci émettrait deux formes de module
mutuellement incompatibles à partir d’un seul fichier. Le compat est donc configuré une seule fois pour le projet et n’a délibérément
aucun prologue directif.

## Mappage des options de plugin

Les options propres au plugin babel n’ont pas d’orthographe de fichier de configuration dans Vize. Chacune est un paramètre d’un point d’entrée
`compile_jsx_with_babel_*` sur la caisse
[`vize_atelier_jsx`](https://github.com/ubugeeei-prod/vize/tree/main/crates/vize_atelier_jsx),
et chacune d’elles est inerte sauf si `jsxCompat` est `"babel"`.

| `@vue/babel-plugin-jsx`      | Point d’entrée Vize                         |
| ---------------------------- | ------------------------------------------- |
| `transformOn`                | `BabelJsxOptions::transform_on`             |
| `pragma`                     | `compile_jsx_with_babel_pragma`             |
| `mergeProps`                 | `compile_jsx_with_babel_merge_props`        |
| `isCustomElement`            | `BabelJsxCustomizations::is_custom_element` |
| `enableObjectSlots`          | `compile_jsx_with_babel_object_slots`       |
| n’importe quelle combinaison | `compile_jsx_with_babel_customizations`     |

Deux options de plugins ne figurent pas dans ce tableau :

- **`optimize`** n’a pas d’équivalent Vize, car la sortie de Vize est toujours optimisée — ce qui est ce que
  le `optimize: true` du plugin produit. Le plugin par défaut est `optimize: false`, et son propre
  README avertit que l’activer « peut sauter certains rerendus », donc le mode gap compat doit
  combler est la direction _non optimisée_ : émission de sortie sans drapeau de patch.
- **`resolveType`** n’est pas mis en œuvre ; voir « Ce qui est différé » ci-dessous.

`enableObjectSlots` est par défaut `true` dans le plugin et dans la voie de compat de Vize : un identifiant unique ou une expression d’appel
passée comme enfant unique d’un composant peut déjà être un objet slot, donc il est vérifié
à l’exécution. Passer `false` considère toujours cette valeur comme l’enfant brut de l’emplacement par défaut.

## Où le mode ne s’applique pas

**sortie Vapor.** `@vue/babel-plugin-jsx` est un plugin de l’ère vdom : chaque forme de sortie qu’il définit est un arbre
`createVNode`, et il n’a pas d’équivalent Vapor. `jsxCompat: "babel"` combiné avec
`jsxMode: "vapor"` n’a donc pas de signification définie, et est rejeté par un diagnostic plutôt que
ignoré silencieusement :

```text
compiler.jsxCompat: "babel" is not supported with Vapor output: @vue/babel-plugin-jsx has no
Vapor equivalent. Use jsxMode "vdom" for babel compatibility, or drop jsxCompat to use Vize's own
Vapor semantics.
```

**sortie SSR.** Les options du plugin décrivent les arbres vnode clients. La compilation SSR
n’applique donc pas du tout la voie de Babel — ni les aides `transformOn` et `enableObjectSlots`, ni
le prédicat `isCustomElement`, ni `mergeProps: false`, ni aucun abaissement propre à Babel — et
utilise la sémantique SSR propre de Vize au lieu d’émettre un mélange à moitié appliqué.

Les deux sont des réponses délibérées, enregistrées dans la caisse pour ne pas être recontestées.

## Qu’est-ce qui est différé

Deux lignes de corpus sont enregistrées comme `deferred` plutôt que divergentes, car chacune attend
travail de compilateur non lié plutôt que le mode compat lui-même :

| Rangée                    | Ce que fait Babel                         | Ce qu’il attend                                                                                                                                                                                                            |
| ------------------------- | ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `options/resolve_type_on` | ajoute `{ props: { … }, name: "A" }`      | l’inférence pilotée par type propage/émet, qui nécessite que la résolution du type soit suivie sur [#1497](https://github.com/ubugeeei-prod/vize/issues/1497) / [#1502](https://github.com/ubugeeei-prod/vize/issues/1502) |
| `slots/dynamic_slot_name` | émet une clé calculée, `{ [n]: () => … }` | abaissement dynamique des slots ; Vize avertit actuellement et abandonne le slot                                                                                                                                           |

## Comment la compatibilité est mesurée

La compatibilité est mesurée par rapport au **plugin réel**, pas à partir de la mémoire. Le corpus est compilé par un
épinglé `@vue/babel-plugin-jsx`, sa sortie est enregistrée comme vérité terrestre engagée, et la suite Rust
des instantanés de cet enregistrement à côté de la sortie de Vize avec un verdict explicite par ligne.

| Artefact                                                          | Rôle                                                                         |
| ----------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| `crates/vize_atelier_jsx/tests/babel_compat/fixtures/corpus.json` | les entrées et les options de plugins sont chacune compilées avec            |
| `crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs`           | Fait passer le corpus via le plugin réel                                     |
| `crates/vize_atelier_jsx/tests/babel_compat_oracle.rs`            | capture instantanés de la sortie de Babel à côté de celle de Vize, par ligne |
| `crates/vize_atelier_jsx/tests/BABEL_COMPAT_INVENTORY.md`         | la forme en prose du tableau des verdicts, et les totaux                     |

Les verdicts ligne par ligne, les divergences globales qui s’appliquent à presque chaque ligne (forme du module, arbre
bloc, drapeaux de patch, flux de contrôle non abaissé), et les totaux de courant vivent tous en
[`BABEL_COMPAT_INVENTORY.md`](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_atelier_jsx/tests/BABEL_COMPAT_INVENTORY.md).
Ces totaux sont fixés par le test de `babel_compat_verdict_totals`, donc ils ne peuvent pas dériver du
corpus — c’est pourquoi cette page ne cite aucun d’eux. Lisez-les à la source.

Pour régénérer ou vérifier l’enregistrement localement :

```bash
node crates/vize_atelier_jsx/tests/babel_compat/oracle.mjs --check
cargo test -p vize_atelier_jsx --test babel_compat_oracle
node --test tests/tooling/babel-jsx-oracle.test.ts
```

## Voir aussi

- [JSX & TSX](./jsx.md) — l’API de création, les props et emits typés, les styles de portée et `jsxMode`.
- [Configuration](./configuration.md) — chaque clé `compiler.*` et l’ordre de recherche des fichiers de configuration.
- [`examples/jsx-tsx`](https://github.com/ubugeeei-prod/vize/tree/main/examples/jsx-tsx) — un projet JSX/TSX exécutable.
