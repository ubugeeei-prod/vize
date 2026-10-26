# eslint-plugin-vize

Vue lint rules powered by Vize's native engine, with complete source locations and native fixes.

Not yet published to npm: it is an experimental workspace package until it joins
the release.

```sh
npm install --save-dev eslint eslint-plugin-vize typescript
```

```js
// eslint.config.mjs
import vize from "eslint-plugin-vize";

export default [vize.configs.recommended];
```

The flat presets include the Vue parser and TypeScript script parser. They apply to `**/*.vue`, including scriptless components and JavaScript, TypeScript and TSX script blocks. ESLint 10 and Node.js 22.13+ or 24+ are supported.

## Select rules and options

```js
import vize, { createVizeConfig } from "eslint-plugin-vize";

export default [
  createVizeConfig({ presets: ["essential", "ecosystem"] }),
  {
    files: ["**/*.vue"],
    rules: {
      "vize/vue/component-name-in-template-casing": ["error", "PascalCase"],
      "vize/a11y/alt-text": "error",
    },
  },
];
```

Presets: `essential`, `recommended`, `happyPath`, `ecosystem`, `nuxt`, `opinionated` and `all`. The corresponding `recommendedWithTypeAware`, `happyPathWithTypeAware`, `ecosystemWithTypeAware` and `opinionatedWithTypeAware` presets opt into type-aware rules. `ruleConfigs` and `createVizeRuleConfig` expose rule entries for an existing flat config.

Rules run over the parser's complete `SourceCode.text`, including unsaved editor text. Enabled rules share one native lint call per file and ESLint pass. Autofix passes run again against the updated text. The shared bridge converts native UTF-8 spans to ESLint's UTF-16 indices and refuses inconsistent metadata or overlapping edits.

## Type-aware rules

Type-aware rules require Vize's native TypeScript runtime. Set `settings.vize.corsaPath` to its executable when it is not discoverable:

```js
import vize from "eslint-plugin-vize";

export default [
  vize.configs.recommendedWithTypeAware,
  { settings: { vize: { corsaPath: "/absolute/path/to/tsc" } } },
];
```

An unavailable runtime produces a visible diagnostic. These presets do not request a TypeScript parser project or turn type-aware lint on implicitly in other presets.

## Scope

This integration exposes the native per-file Vue rules. Project graph analysis and cross-file linting remain separate Vize operations; linting several files with ESLint does not enable them. The bundled configurations do not claim HTML, JSX, Svelte, Pug or other input-language support. No JavaScript fallback substitutes for an unavailable native binding.

The existing `oxlint-plugin-vize` and `oxlint-plugin-vize/eslint` exports remain available. This package uses their shared rule implementation and native binding.
