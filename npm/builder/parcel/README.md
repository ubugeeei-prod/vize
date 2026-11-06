# @vizejs/parcel-transformer

Compile Vue single-file components with Vize's native compiler in Parcel 2.
The host matrix exercises Parcel 2.16.4.

Not yet published to npm: it is an experimental workspace package until it joins
the release.

```sh
npm install --save-dev parcel @vizejs/parcel-transformer
npm install vue
```

Register the transformer and named component pipeline in `.parcelrc`:

```json
{
  "extends": "@parcel/config-default",
  "reporters": ["./vize-parcel-dev-cache.cjs", "..."],
  "transformers": {
    "*.vue": ["@vizejs/parcel-transformer"],
    "vize-component:*.vue": ["@vizejs/parcel-transformer"],
    "*.css": ["@vizejs/parcel-transformer/style", "..."]
  }
}
```

Create `vize-parcel-dev-cache.cjs` beside `.parcelrc`:

```js
module.exports = require("@vizejs/parcel-transformer/dev-cache");
```

Parcel validates reporter package names before resolving their exports. The
relative CommonJS adapter loads the reporter through its public package subpath
and keeps it ahead of the default development server reporter.

The named pipeline isolates component execution from style dependencies. The CSS
continuation scopes styles after Parcel's preprocessor pipeline and preserves its
normal processing.
Install the corresponding Parcel transformer and preprocessor when using Sass,
Less, or Stylus. The adapter does not download these providers itself.

Inline JavaScript and TypeScript scripts, template asset URLs, scoped CSS,
external style blocks, and host style preprocessors use the same native compiler
and CSS transforms as the other Vize integrations. Virtual blocks stay in
Parcel's asset graph, so relative imports and URLs retain their source owner.
The native addon is an ordinary package dependency; no separate installation is
required. The type checker is not started during a Parcel build.

Configure the adapter with `.vizerc.parcel.json` or the
`"@vizejs/parcel-transformer"` key in `package.json`:

```json
{
  "compiler": {
    "customElements": ["my-element"],
    "templateSyntax": "standard"
  },
  "transformAssetUrls": true
}
```

Parcel tracks configuration changes and invalidates the affected transforms.
The native compiler controls are declared by `VizeParcelOptions`. Source maps,
production mode, SSR targets, filenames, and scope identity come from the host
and cannot be overridden by this configuration. Scope IDs and production CSS
variable names use project-relative filenames.

Node targets use the native SSR backend and omit client CSS imports. Build a
browser target as well to produce its styles. Development HMR conservatively
reloads a changed component; local component state can reset, including edits to
inline SFC styles. CSS and Sass dependency updates preserve the component state
without executing its script module again.

Vapor output is opt-in with `"compiler": { "vapor": true }` and needs a Vue
runtime that provides the corresponding helpers. The transformer uses the default
`vue` runtime module for Vapor; Parcel aliases can select its runtime version.
`runtimeModuleName` can select a separate module for VDOM output.

SFC CSS modules, external script/template blocks, custom blocks, and emitted
macro artifacts currently produce explicit errors. Separate `.module.css`
imports can use Parcel's normal CSS module support. Unsupported input is not
silently discarded.

The source qualification workflow packs the actual source-built native image
and transformer, installs them in a fresh consumer, and checks production,
browser rendering, scoped and preprocessed styles, assets, aliases, source maps,
SSR, and failure cases. Source and public-provider preflight receipts are
recorded separately.

Development bundles keep stable names. Parcel 2.16.4 answers HTTP 304 when
`Last-Modified` stays in the same UTC second, so a reload after rapid
compilation-error recovery can keep the previous bundle
([tracked issue](https://github.com/ubugeeei-prod/vize/issues/10736)). The
`dev-cache` reporter, listed ahead of the dev server, advances that timestamp
to the next second only when the output bytes change. It does not content-hash
HMR assets. The `test:recovery-frontier` script retains strict latest-source
assertions with browser caching enabled.

Parcel tracing (`shouldTrace: true`) records individual native compiler, parser,
asset and CSS operations. These spans measure each NAPI call's inclusive wall
time; internal Rust phases are not inferred from JavaScript samples.
