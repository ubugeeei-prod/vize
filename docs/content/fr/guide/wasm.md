---
title: Reliures WASM
---

<!-- Generated translation; source: guide/wasm.md -->

# Reliures WASM

> **⚠️ Travaux en cours :** Vize est en développement actif et n’est pas encore prêt pour une utilisation en production. Les API WASM peuvent changer sans préavis.

`@vizejs/wasm` fournit des liaisons WebAssembly pour exécuter directement le compilateur Vue dans le navigateur. Cela permet la compilation, le linting et la mise en forme SFC en temps réel sans serveur — idéal pour les terrains de jeux, la documentation et les outils éducatifs.

Les liaisons WASM sont compilées à partir de la même base de code Rust que les liaisons CLI et NAPI (`vize_vitrine`), garantissant une sortie de compilation identique sur toutes les plateformes.

## Installation

Installez `vp` une fois depuis le [Vite+ install guide](https://viteplus.dev/guide/install), puis ajoutez le package :

```bash
vp install @vizejs/wasm
```

## API

### Compatibilité des options du compilateur

Le type de `CompilerOptions` est l’inventaire des options prises en charge pour `compile`, `compileVapor`,
`parseTemplate`et `compileSfc`. Les clés d’objet inconnues sont ignorées à la frontière JavaScript et
ne sont pas des promesses de compatibilité. `vueParserQuirks` reste un alias déprécié pour
`templateSyntax: "quirks"`; Un `templateSyntax` explicite passe toujours en priorité. Le champ
partagé Rust `experimentalServerScript` est réservé et n’est exposé qu’à l’étape du compilateur WASM
l’implémente. Chaque façade ignore les champs pris en charge qui ne s’appliquent pas à son stade de compilateur :
`bindingMetadata` ne s’applique qu’à la compilation directe de modèles. Les noms d’exécution s’appliquent aux modules VDOM
générés et aux sorties client SFC (VDOM ou Vapor) ; les cartes sources s’appliquent à la sortie VDOM, y compris le résultat
modèle retourné par `compileSfc`. `outputMode` et `scriptExt` ne s’appliquent qu’aux compilations SFC.

### Compiler SFC

Compiler un composant Vue en fichier unique en JavaScript :

```javascript
import init, { compileSfc } from "@vizejs/wasm";

await init();

const result = compileSfc(
  `<template>
    <div>{{ msg }}</div>
  </template>

  <script setup lang="ts">
  const msg = ref('Hello Vize!')
  </script>`,
  { filename: "App.vue" },
);

console.log(result.script.code); // compiled <script> / <script setup>
console.log(result.template?.code); // compiled render function, when a template exists
console.log(result.css); // compiled styles, when styles exist
```

### SFC de peluches

Règles spécifiques à la peluche de Vue sur un SFC :

```javascript
import init, { lintSfc } from "@vizejs/wasm";

await init();

const result = lintSfc(source, {
  filename: "App.vue",
  locale: "en", // 'en' | 'ja' | 'zh'
});

for (const diagnostic of result.diagnostics) {
  console.log(
    `${diagnostic.severity}: ${diagnostic.message} (line ${diagnostic.location.start.line})`,
  );
}
```

### Format SFC

Formatez un Vue SFC :

```javascript
import init, { formatSfc } from "@vizejs/wasm";

await init();

const formatted = formatSfc(source, { printWidth: 80 });

console.log(formatted.code);
```

## Initialisation

La fonction `init()` doit être appelée une fois avant d’utiliser toute autre API. Il charge et instance le module WebAssembly :

```javascript
import init from "@vizejs/wasm";

// Basic initialization
await init();

// With custom WASM URL (useful for CDN or bundler setups)
await init("https://cdn.example.com/vize_vitrine_bg.wasm");
```

## Cas d’utilisation

### Aires de jeux

Construisez des terrains de compilation interactifs Vue qui tournent entièrement dans le navigateur. Le [Vize Playground](https://vizejs.dev/play) officiel utilise les liaisons WASM pour la compilation en temps réel :

```javascript
// React to editor changes and compile in real-time
editor.onChange((source) => {
  const result = compileSfc(source, {
    filename: "Playground.vue",
  });

  if (result.errors.length === 0) {
    preview.update({
      script: result.script.code,
      template: result.template?.code,
      css: result.css,
    });
  } else {
    diagnostics.show(result.errors);
  }
});
```

### Documentation

Intégrez des exemples Vue en direct et modifiables dans votre documentation :

```javascript
// Compile documentation examples on the fly
const examples = document.querySelectorAll("[data-vue-example]");
for (const el of examples) {
  const result = compileSfc(el.textContent, {
    filename: `example-${el.id}.vue`,
  });
  // Use result.script.code, result.template?.code, and result.css to mount it.
}
```

### Éducation

Créez des outils interactifs d’exploration des compilateurs qui montrent la sortie de la compilation en temps réel, aidant ainsi les développeurs à comprendre comment les modèles Vue sont transformés.

### CI/CD

Utilisez des liaisons WASM pour une compilation légère dans des environnements où les binaires natifs ne sont pas disponibles (par exemple, Cloudflare Workers, Deno Deploy, CI basé sur navigateur).

## Construire à partir de la source

```bash
# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli

# Build WASM
cargo build --release -p vize_vitrine \
  --no-default-features \
  --features wasm \
  --target wasm32-unknown-unknown

# Generate JS bindings
wasm-bindgen \
  target/wasm32-unknown-unknown/release/vize_vitrine.wasm \
  --out-dir npm/wasm \
  --target web
```

## Internationalisation

Toutes les API WASM qui produisent des diagnostics (lint, erreurs de compilation) prennent en charge des messages localisés :

| Code | Langue               |
| ---- | -------------------- |
| `en` | Anglais (par défaut) |
| `ja` | Japonais (日本語)    |
| `zh` | Chinois (中文)       |

Passez l’option `locale` à toute API qui produit des diagnostics :

```javascript
const result = lintSfc(source, {
  filename: "App.vue",
  locale: "ja", // Lint messages in Japanese
});

console.log(result.diagnostics);
```

## Taille du faisceau

Le module WASM inclut le pipeline complet du compilateur Vue (analyseur syntaxique, analyseur sémantique, générateur de code) compilé en WebAssembly. La taille du bundle compressé est d’environ **1,5 Mo**, ce qui convient au chargement non critique (par exemple, chargé après l’interactivité de la page).

Pour une utilisation en production, considérons le chargement paresseux du module WASM :

```javascript
// Lazy-load the compiler only when needed
const compiler = await import("@vizejs/wasm");
await compiler.default(); // init()
const result = compiler.compileSfc(source, opts);
console.log(result.script.code, result.template?.code, result.css);
```
