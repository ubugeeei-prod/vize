---
title: "type/unbound-method"
description: "Interdire une référence à une méthode sans son receveur (`this`)."
---

# `type/unbound-method`

Interdire une référence à une méthode sans son receveur (`this`).

| Gravité par défaut | Préréglages | Correction automatique | Options |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | Non | [Options des règles](/rules/options.md) |

Champ d’application : Informations de type dans les scripts et templates des SFC Vue, pour les constructions présentées ci-dessous.

Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.

## Mauvais

counter.increment prend la méthode sans son receveur, donc this serait perdu à l’appel.

```vue annotate="remove:9"
<script setup lang="ts">
class Counter {
  value = 0;
  increment(): void {
    this.value += 1;
  }
}
const counter = new Counter();
const unbound = counter.increment;
</script>
```

## Bon

counter.increment() appelle la méthode sur counter, donc this reste lié.

```vue annotate="add:9"
<script setup lang="ts">
class Counter {
  value = 0;
  increment(): void {
    this.value += 1;
  }
}
const counter = new Counter();
counter.increment();
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/unbound-method": "warn"
    },
    "typeAware": true
  },
});
```

Lancez ensuite `vp run lint`.

## Différences avec typescript-eslint

Les méthodes de la bibliothèque lues sur l’objet global ne sont pas signalées. Cela concerne un receveur typé `Window & typeof globalThis` ou `typeof globalThis`, y compris `window.setTimeout`, `const { matchMedia } = window`, `self.requestAnimationFrame` et `document.defaultView?.setTimeout`. Une opération de la plateforme appelée sans receveur utilise l’objet global, donc ces références fonctionnent telles quelles. Une méthode que votre code ajoute à `Window` reste signalée. Il en va de même pour le `Window` d’un autre cadre (`parent.setTimeout`) et pour la méthode d’un objet atteint par l’objet global (`window.localStorage.getItem`). Avec `checkGlobalObjectMethods: true`, ces références sont signalées comme dans typescript-eslint.

Un objet natif auquel une déclaration est fusionnée (`declare global { interface Math {} }`) garde son exemption, et `RegExp.escape` est exempté par tout chemin vers le constructeur `RegExp`. L’exemption suit les déclarations de la bibliothèque TypeScript, et non la version de Node.js qui exécute le linter.

## Voir aussi

- [Règles de type et de script](../type-and-script.md)
- [Options des règles](/rules/options.md)
- [Toutes les règles](../all.md)
- [Code source](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/unbound_method.rs#L5)
