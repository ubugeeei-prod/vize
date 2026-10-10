---
title: "Règles petite-vue"
---

# Règles petite-vue

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.


| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](#petite-vue-no-unsupported-directive) | [Mauvais](#petite-vue-no-unsupported-directive-bad) · [Bon](#petite-vue-no-unsupported-directive-good) | Interdire les directives non prises en charge par petite-vue |
| [`petite-vue/valid-v-effect`](#petite-vue-valid-v-effect) | [Mauvais](#petite-vue-valid-v-effect-bad) · [Bon](#petite-vue-valid-v-effect-good) | Exiger une expression non vide pour v-effect |
| [`petite-vue/valid-v-scope`](#petite-vue-valid-v-scope) | [Mauvais](#petite-vue-valid-v-scope-bad) · [Bon](#petite-vue-valid-v-scope-good) | Exiger que v-scope reçoive un objet littéral |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `petite-vue/no-unsupported-directive`

Interdire les directives non prises en charge par petite-vue

[Mauvais](#petite-vue-no-unsupported-directive-bad) · [Bon](#petite-vue-no-unsupported-directive-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Documents HTML détectés comme petite-vue ; les SFC Vue ordinaires sont hors du champ de cette règle.  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-no-unsupported-directive-bad"></span>

**Mauvais**

`v-memo`, `v-slot:header` et la directive personnalisée `v-my-directive` sont absents de la liste des directives prises en charge par petite-vue. Le script petite-vue désigne ce HTML comme relevant de ce dialecte.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-no-unsupported-directive-good"></span>

**Bon**

Le remplacement utilise les syntaxes prises en charge `v-scope`, `v-effect`, `v-if`, `v-bind` et `v-on` au lieu de dépendre de directives non prises en charge.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) · [Toutes les règles](all.md)

### `petite-vue/valid-v-effect`

Exiger une expression non vide pour v-effect

[Mauvais](#petite-vue-valid-v-effect-bad) · [Bon](#petite-vue-valid-v-effect-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Documents HTML détectés comme petite-vue ; les SFC Vue ordinaires sont hors du champ de cette règle.  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-effect": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-effect-bad"></span>

**Mauvais**

Chaque `v-effect` ne possède aucune expression exécutable : sa valeur est absente, vide ou composée uniquement d’espaces.

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-effect-good"></span>

**Bon**

Les deux valeurs de `v-effect` contiennent une expression : l’une met à jour `el.textContent` et l’autre incrémente `count`. Cette règle vérifie que l’expression n’est pas vide, et non la logique métier de l’effet.

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) · [Toutes les règles](all.md)

### `petite-vue/valid-v-scope`

Exiger que v-scope reçoive un objet littéral

[Mauvais](#petite-vue-valid-v-scope-bad) · [Bon](#petite-vue-valid-v-scope-good)

Gravité par défaut: `error`  
Préréglages: _none_  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Documents HTML détectés comme petite-vue ; les SFC Vue ordinaires sont hors du champ de cette règle.  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-scope": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-scope-bad"></span>

**Mauvais**

Les quatre valeurs non vides de `v-scope` sont un identifiant, un appel, une expression arithmétique et un nombre ; aucune n’est analysée comme un objet littéral.

```html annotate="remove:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-scope-good"></span>

**Bon**

Un `v-scope` sans valeur utilise la portée racine. Les autres valeurs sont des objets littéraux, y compris l’objet entre parenthèses, que la règle accepte.

```html annotate="add:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) · [Toutes les règles](all.md)
