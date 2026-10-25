# @vizejs/parcel-transformer

Compile Vue single-file components with Vize's native compiler in Parcel 2.16.4
and later 2.x versions. This integration is a compatibility preview.

```sh
npm install --save-dev parcel @vizejs/parcel-transformer
npm install vue
```

Add the SFC transformer and the CSS continuation to `.parcelrc`:

```json
{
  "extends": "@parcel/config-default",
  "transformers": {
    "*.vue": ["@vizejs/parcel-transformer"],
    "*.css": ["@vizejs/parcel-transformer/style", "..."]
  }
}
```

The CSS continuation scopes styles after Parcel's preprocessor pipeline. Keep
`"..."` so Parcel can resolve CSS imports, asset URLs, targets, and optimization.
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
reloads a changed component; local component state can reset. CSS-only
dependencies are handled by Parcel's CSS HMR runtime.

Vapor output is opt-in with `"compiler": { "vapor": true }` and needs a Vue
runtime that provides the corresponding helpers. The preview uses the default
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
