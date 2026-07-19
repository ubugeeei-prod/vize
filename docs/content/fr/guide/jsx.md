---
title: JSX & TSX
---

<!-- Generated translation; source: guide/jsx.md -->

# JSX & TSX

> **Statut :** JSX/TSX est couvert par le compilateur, le linter, le vérificateur de type, le LSP et le formateur.
> Les vérifications sensibles au type restent volontaires, donc les fichiers React `.tsx` ne sont jamais traités comme Vue JSX par accident.
> HMR pour les modules `.jsx`/`.tsx` autonomes reste le principal manque d’intégration restant.

Vize compile `.jsx` et `.tsx` composants Vue via les **mêmes caisses de compilation** que `.vue`
composants à fichier unique — les backends VDOM et Vapor, l’analyse sémantique de Croquis, la vérification de type
Canon, la peluche Patina et le serveur de langage Maestro. Il n’y a pas de pipeline Babel séparé ni de cale d’usine JSX
runtime : un composant JSX est directement rétrogradé vers une fonction de rendu Vue (ou un modèle Vapor
) par le compilateur natif.

Cela signifie qu’un composant Vue `.tsx` bénéficie de la même compilation Rust-native, du même contrôle de type et
la même expérience d’éditeur qu’un SFC — mais créé comme une fonction typée au lieu d’une `<template>`.

## Activation de JSX/TSX

`.jsx` et `.tsx` fichiers sont automatiquement acheminés via les plugins bundler Vize — il n’y a pas de
drapeau d’adhésion pour les compiler. Tout projet utilisant déjà une intégration Vize bundler bénéficie du support JSX/TSX
:

- `@vizejs/vite-plugin`
- `@vizejs/unplugin` (enroulement / webpack / esbuild)
- `@vizejs/rspack-plugin`
- `@vizejs/nuxt`

```ts
// vite.config.ts — nothing JSX-specific is required
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
});
```

Sous le capot, les plugins appellent le point d’entrée natif/WASM `compileJsx` (exposé à partir de
`@vizejs/native` et `@vizejs/wasm`), ce qui réduit le code source et renvoie le code de rendu ainsi que tout CSS
extrait avec scope.

## API d’auteur

Un composant Vize JSX/TSX est une **fonction simple avec des paramètres typés**. Il n’y a ni macros ni enveloppe
`defineComponent` dans le cas courant — les types sont lus directement depuis la fonction
signature et effacés de la sortie à l’exécution (coût zéro).

- **Les accessoires** sont le **premier paramètre typé**.
- **Les émets et les emplacements** sont le **second paramètre typé**, un `Ctx<Emits, Slots>` fourni par Vize
  contexte (avec `emit`, `slots`et `attrs`, en miroir du contexte de configuration de Vue).
- **Les valeurs de prop par défaut** proviennent de **la déstructuration des défauts** dans le motif de paramètres — le
  compilateur les extrait de la déstructuration.

```tsx
import { computed, ref } from "vue";

type CounterProps = {
  label: string;
  start?: number;
};

type CounterEmits = {
  change: [value: number];
};

const Counter = ({ label, start = 0 }: CounterProps, { emit }: Ctx<CounterEmits>) => {
  const count = ref(start);
  const doubled = computed(() => count.value * 2);

  const increment = () => {
    count.value += 1;
    emit("change", count.value);
  };

  return (
    <section class="counter">
      <p>
        {label}: {count.value}
      </p>
      <p>Double: {doubled.value}</p>
      <button type="button" onClick={increment}>
        Increment
      </button>
    </section>
  );
};
```

Les composantes uniquement à accessoires peuvent omettre complètement le second paramètre :

```tsx
const Hello = ({ name }: { name: string }) => <h1>Hello, {name}!</h1>;
```

Les valeurs par défaut sont écrites comme des valeurs par défaut déstructurantes ; Aucune option `props` séparée n’est nécessaire :

```tsx
const Badge = ({ count = 0 }: { count?: number }) => <span class="badge">{count}</span>;
```

Le nom du composant est tiré de la liaison (`const Counter = …`) ou de la déclaration de fonction
(`function Card() { … }`), exactement comme on s’y attendrait. Tout le reste est JSX similaire à React — imbriquage d’éléments
, fragments (`<>…</>`), enfants d’expression, et props d’événements tels que `onClick`. La seule addition spécifique
Vue est l’élément `<style scoped>` décrit [below](#scoped-styles).

> Le formulaire d’auteur uniquement par type ci-dessus est le cas courant supporté. Synthèse de l’exécution `props`
> Les métadonnées, ainsi que le formulaire de `defineComponent(() => () => vnode)` d’installation, sont des suivis prévus.

## Surface JSX prise en charge

Le compilateur abaisse JSX au même IR de soulagement utilisé par les modèles SFC, puis envoie cet IR au
VDOM ou au backend Vapor. Ces formulaires sont couverts par la matrice de test JSX/TSX :

- Fragments et éléments imbriqués
- balises composante, balises d’expression membre et balises intrinsèques HTML/SVG
- attributs statiques, liaisons `prop={expr}` dynamiques, accessoires abcourcis booléens et supports spread
- gestionnaires d’événements, y compris des modificateurs d’options de type Vue encodés dans le nom du prop
- `v-if`, `v-else-if`, `v-else`, `v-show`, directives `v-*` personnalisées, et `v-model`
- enfants d’expression, branches JSX logiques, branches JSX ternaires, et rendu `.map(...)` liste
- emplacements écrits comme enfants objets ou enfants de rendu
- Syntaxe TSX : paramètres typés, annotations de retour, appels JSX génériques, distributions et assertions non nulles
- `<style scoped>` extraction ; L’interpolation `${expr}` de type type littéral est prise en charge pour les avancés
  cas, mais les classes statiques et les variables CSS sont généralement plus claires

La forme de liste canonique est idiomatique JSX :

```tsx
import { computed, ref } from "vue";

type Todo = {
  id: string;
  title: string;
  done: boolean;
};

type TodoListProps = {
  todos: Todo[];
  initialActiveId?: string;
};

const TodoList = ({ todos, initialActiveId }: TodoListProps) => {
  const activeId = ref(initialActiveId ?? todos[0]?.id);
  const activeTodo = computed(() => todos.find((todo) => todo.id === activeId.value));

  return (
    <section class="todo-panel">
      <header>
        <h2>{activeTodo.value?.title ?? "Select a todo"}</h2>
      </header>

      <ul class="todo-list">
        {todos.map((todo, index) => (
          <li
            key={todo.id}
            class={{ done: todo.done, active: todo.id === activeId.value }}
            data-index={index}
          >
            <button type="button" onClick={() => (activeId.value = todo.id)}>
              <span>{todo.title}</span>
              {todo.id === activeId.value ? <strong>Active</strong> : <em>{index + 1}</em>}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
};
```

Les alias de rappel `.map(...)` (`todo`, `index`) sont conservés dans la portée du vérificateur de type généré et
TypeScript virtuel LSP, donc le survol, la complétude, le diagnostic et le renommage fonctionnent sur les mêmes liaisons
que vous avez créées.

## Mode de sortie : VDOM vs Vapor

Chaque composant compile soit en sortie **Virtual DOM** (le moteur de rendu par défaut de Vue), soit en sortie
[**Vapor**](https://blog.vuejs.org/posts/vue-vapor). Le défaut est choisi par la configuration ;
composants individuels peuvent le surpasser.

### Configuration par défaut

`compiler.jsxMode` définit le backend global par défaut pour `.jsx`/`.tsx` composants. Il accepte `"vdom"`
ou `"vapor"` et passe par défaut à `"vdom"`.

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  compiler: {
    // Default every .jsx/.tsx component to Vapor output.
    jsxMode: "vapor",
  },
});
```

`jsxMode` est indépendant de `compiler.vapor`: `vapor` bascule Vapor pour `.vue` SFC, tandis que `jsxMode`
contrôle le backend par défaut pour JSX/TSX. Un projet peut garder les SFC sur VDOM tout en mettant par défaut JSX sur
Vapor, ou inversement. Le plugin Vite accepte aussi `jsxMode` directement comme option plugin, ce qui
remplace la configuration partagée.

### Directives par composant

Un composant individuel remplace le défaut par un prologue directif, reflétant `"use strict"`:

```tsx
// Compiled to Vapor regardless of the configured default.
const Fast = () => {
  "use vue:vapor";
  return <div class="fast" />;
};

// Compiled to Virtual DOM regardless of the configured default.
const Classic = () => {
  "use vue:vdom";
  return <div class="classic" />;
};
```

Comme chaque composant est routé indépendamment, un **seul fichier peut mélanger les deux backends** :

```tsx
// vize.config: { compiler: { jsxMode: "vapor" } }

// No directive -> takes the configured default (Vapor here).
export const Dashboard = () => <main>{/* ... */}</main>;

// Opts back into Virtual DOM just for this component.
export const LegacyWidget = () => {
  "use vue:vdom";
  return <aside>{/* ... */}</aside>;
};
```

### Préséance

Le mode de sortie d’un composant se résout dans cet ordre :

1. Une directive `"use vue:vapor"` / `"use vue:vdom"` par composant.
2. Le `compiler.jsxMode` par défaut depuis la configuration (ou l’option `jsxMode` du plugin).
3. Le plan B intégré, `"vdom"`.

### Diagnostic

Les directives mal formées ou contradictoires sont signalées plutôt que silencieusement ignorées :

- Une directive qui commence par `"use vue:"` mais ne nomme pas de mode connu (une faute de frappe telle que
  `"use vue:vdomx"`) est une erreur de compilation.
  - Deux directives de mode conflictuelles dans un composant (`"use vue:vapor"` suivies de `"use vue:vdom"`)
    sont diagnostiqués ; La première directive l’emporte toujours pour le mode Résolu.
- Des prologues sans lien comme `"use strict"` restent intacts.

## Styles à portée

Un élément `<style scoped>` **à l’intérieur du composant** est l’équivalent JSX du bloc
`<style scoped>` d’un SFC. Il est extrait au moment de la compilation — jamais rendu en temps d’exécution `<style>`
vnode — son CSS est réécrit par scope avec un identifiant de `data-v-<hash>` généré, cet attribut de portée
est injecté sur les autres éléments du composant, et le CSS réécrit est émis via le pipeline CSS du plugin
bundler. Cela fonctionne à la fois dans les backends VDOM et Vapor, et les deux obtiennent le même id de portée
pour un composant donné.

Idiomatiquement, l’élément `<style scoped>` passe **en dernier**, après le balisage — correspondant à l’ordre
`<template>` → `<style>` d’un SFC — mais le compilateur l’extrait là où il apparaît.

```tsx
type CardProps = {
  title: string;
};

const Card = ({ title }: CardProps) => (
  <article class="card">
    <h2>{title}</h2>

    <style scoped>{`
      .card {
        border: 1px solid currentColor;
        padding: 12px;
      }
    `}</style>
  </article>
);
```

### Valeurs de style dynamiques

Privilégiez les liaisons de classes normales, les objets de style en ligne ou les propriétés personnalisées CSS pour le style dynamique dans
JSX/TSX. Les interpolations au sens propre du modèle `${expr}` à l’intérieur des `<style scoped>` sont prises en charge et
vérifiées par type, mais elles sont une échappatoire plutôt qu’un style principal d’écriture :

```tsx
type BoxProps = {
  color: string;
  gap: number;
};

const Box = ({ color, gap }: BoxProps) => (
  <section
    class="box"
    style={{
      "--box-color": color,
      "--box-gap": `${gap}px`,
    }}
  >
    <p>content</p>

    <style scoped>{`
      .box {
        color: var(--box-color);
        gap: var(--box-gap);
      }
    `}</style>
  </section>
);
```

Un élément `<style>` **sans** `scoped` est traité comme un élément normal et rendu tel quel — il n’est
pas extrait.

`<style scoped>{`.box { color : ${color} ; }`}</style>` fonctionne aussi et est couvert par le vérificateur de type,
mais conservez-le pour les cas où une feuille de style à portée doit vraiment référencer une expression composante.
La syntaxe CSS `v-bind(...)` fonction littérale utilisée dans un bloc SFC `<style>` n’est pas un formulaire d’auteur
supporté dans un bloc de style JSX.

## Mise en forme

Glyph formate le contenu des scripts JSX/TSX avec l’analyseur et le formateur OXC. Dans `.vue` fichiers,
`<script lang="jsx">`, `<script lang="tsx">``<script setup lang="tsx">` et sont analysés en JSX/TSX
au lieu de revenir à un simple TypeScript, donc les enfants JSX et les annotations TSX sont formatés comme
syntaxe réelle :

```vue
<script setup lang="tsx">
type CardProps = {
  title: string;
  items: string[];
};

const Card = ({ title, items }: CardProps) => (
  <section class="card">
    <h2>{title}</h2>
    {items.map((item) => (
      <span key={item}>{item}</span>
    ))}
  </section>
);
</script>
```

Les modules `.jsx`/`.tsx` autonomes sont découverts par `vize fmt` aux côtés de `.vue` fichiers et
formatés avec la même gestion de type source JSX/TSX :

```bash
# Formats .vue, .jsx, and .tsx files by default
vize fmt src --write
```

## Vérification de type

La vérification de type JSX/TSX se fait **par option** via `typeChecker.jsxTypecheck`, qui par défaut ** est`false`**.
Il est désactivé par défaut volontairement : un dépôt peut contenir des fichiers React `.tsx` qui ne doivent pas être
vérifiés en tant que JSX Vue.

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  typeChecker: {
    enabled: true,
    jsxTypecheck: true,
  },
});
```

Lorsqu’il est activé, `vize check` vérifie le type `.jsx`/`.tsx` composants Vue via Canon. Le fichier virtuel
généré est un simple TypeScript, et non TSX, et il préserve le contrat composant créé :

- le premier paramètre typé reste le type props ;
- `Ctx<Emits, Slots>` reste visible pour le corps de configuration et les expressions JSX ;
- Gestionnaires d’événements, props liés, `v-if`/`v-show`, directives personnalisées et interpolation de type scope
  expressions, lorsqu’elles sont utilisées, sont réémises comme des lectures normales en TypeScript ;
- `v-model` cibles sont réémises en auto-affectations écrivables, donc liaisons lecture seule ou non l-value
  sont diagnostiqués au niveau de la liaison ;
- `.map(...)` corps de liste sont réémis à l’intérieur du callback généré, donc les alias valeur/index sont conservés
  leurs types d’éléments inférés.

Les diagnostics sont rapportés aux **emplacements sources d’origine** (à la fois en JSON pour le CLI et via
LSP), car chaque plage virtuelle TS significative correspond à la plage source que vous avez écrite.

```tsx
type FieldProps = {
  model: {
    readonly value: string;
  };
};

const Field = ({ model }: FieldProps) => <input v-model={model.value} />;
```

Dans l’exemple ci-dessus, `model.value` est coché comme cible d’attribution. Si c’est en lecture seule, le diagnostic
se retrouve sur `model.value` dans le code source TSX, et non dans le code généré.

```bash
# Type-check a project including its .jsx/.tsx Vue components.
# .jsx/.tsx files are collected only when typeChecker.jsxTypecheck is enabled.
vize check src
```

Composants JSX/TSX autonomes inférieurs à un simple TypeScript virtuel pour vérification. Les SFC contenant
`<script lang="jsx">`, `<script lang="tsx">`ou `script setup` correspondants sont matérialisés sous forme de fichiers virtuels
`.vue.tsx`, de sorte que TypeScript analyse la syntaxe JSX dans le bloc de script. Le LSP et la CLI partagent
la même baisse, donc un diagnostic Corsa se retrouve à la même plage source dans l’éditeur et sur la ligne de commande
.

## Éditeur / LSP

L’ouverture d’un composant Vue `.jsx`/`.tsx` dans un éditeur appuyé par `vize lsp` donne le même langage
fonctionnalités qu’un SFC — **pas besoin d’un wrapper SFC** :

- Diagnostic
- Vol stationnaire
- Achèvement
- Référence à la définition
- Références
- Renom
- Symboles de documents
- Jetons sémantiques
- Actions du code
- Diagnostic CSS intégré pour `<style scoped>` blocs

Les caractéristiques structurelles (symboles de documents, jetons sémantiques, diagnostics de type scoped, actions de code) fonctionnent
du document analysé et sont toujours disponibles. Les fonctionnalités sensibles aux types (diagnostic, survolation, complétion
, aller à la définition, références, renommage) ne sont atteintes que lorsque `typeChecker.jsxTypecheck` est
activé, donc les fichiers React `.tsx` ne sont jamais traités comme Vue JSX dans l’éditeur non plus.

## Linting

Les règles de peluches Patina de Vize fonctionnent sur JSX/TSX via une règle IR **à coût zéro projetée directement à partir de l’OXC
AST**. Les règles orientées balisage ne reconstruisent pas un modèle SFC synthétique ; ils lisent directement les éléments JSX et
attributs. Les règles qui nécessitent la forme du modèle Vue, comme `.map(...)` liste des clés de vérification, passent
sur l’arbre de relief abaissé. Les règles sémantiques sont soutenues par Croquis, la même couche d’analyse utilisée pour
SFC.

Cela signifie que le linting JSX/TSX détecte les mêmes classes de problèmes sans dépendre d’un
de correspondance partielle de chaînes :

```tsx
const BrokenMedia = () => (
  <article>
    <img src="/avatar.png" />
    <button accessKey="s" autoFocus>
      Save
    </button>
  </article>
);
```

L’exemple ci-dessus est indiqué comme source JSX :

- `a11y/img-alt` signale la disparition `alt`;
- `a11y/no-access-key` rapports `accessKey`;
- `a11y/no-autofocus` rapporte `autoFocus`.

Les règles clés de la liste comprennent la forme idiomatique JSX `.map(...)` :

```tsx
const KeyedList = ({ rows }: { rows: Array<{ id: string; label: string }> }) => (
  <ul>
    {rows.map((row) => (
      <li key={row.id}>{row.label}</li>
    ))}
  </ul>
);
```

Les diagnostics et corrections correspondent aux plages source JSX, donc la sortie CLI et les décorations de l’éditeur pointent vers l’élément
ou le prop qui devrait changer.

```bash
# Lint .vue, .html, .jsx, and .tsx files
vize lint src
```

Voir [Static Analysis](./static-analysis.md) pour le modèle de peluches et de contrôle de type, et
[Rules](../rules/index.md) pour la sortie des règles concrètes.

## Limitations

Soyez conscient des arêtes actuelles :

- **La vérification de la typographie se fait volontairement.** `typeChecker.jsxTypecheck` est `false` par défaut, donc un mélange Vue/React
  dépôts ne routent pas accidentellement React TSX via le vérificateur JSX de Vue.
- **HMR n’est pas encore câblé pour les modules `.jsx`/`.tsx` .** Le compilateur JSX émet actuellement un
  module de fonction de rendu plutôt qu’un module composant-objet complet, il n’y a donc pas de
  frontière Vue HMR à attacher. Une sortie complète du module composant plus un HMR préservant l’état est un suivi prévu ; Jusqu’à
  ce moment-là, les modifications d’un composant `.jsx`/`.tsx` reviennent à un rechargement normal.
- **Le `v-bind(...)` CSS littéral à l’intérieur d’un bloc JSX `<style scoped>` n’est pas pris en charge.** Utilisez `${expr}`
  interpolation modèle-littéral, qui est le formulaire pris en charge et vérifié par type.

## Voir aussi

- [Configuration](./configuration.md) — les `compiler.jsxMode` et `typeChecker.jsxTypecheck` clés,
  plus la configuration partagée complète.
- [Vite Plugin](./vite-plugin.md) — l’intégration recommandée pour le bundler.
- [Static Analysis](./static-analysis.md) — comment la vérification de la peluchon et des types partagent le pipeline du compilateur.
- [`examples/jsx-tsx`](https://github.com/ubugeeei-prod/vize/tree/main/examples/jsx-tsx) —
  ciblé sur les exemples de sources JSX/TSX pour la couverture du compilateur, linter, vérificateur de types, LSP et formateur.
