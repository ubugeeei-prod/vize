# @vizejs/parcel-transformer

Compile Vue single-file components with Vize's native compiler in Parcel 2.
The host matrix exercises Parcel 2.16.4. This integration is a compatibility preview.

```sh
npm install --save-dev parcel @vizejs/parcel-transformer
npm install vue
```

Register the transformer and named component pipeline in `.parcelrc`:

```json
{
  "extends": "@parcel/config-default",
  "transformers": {
    "*.vue": ["@vizejs/parcel-transformer"],
    "vize-component:*.vue": ["@vizejs/parcel-transformer"],
    "*.css": ["@vizejs/parcel-transformer/style", "..."]
  }
}
```

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

Production, SSR, CSS/Sass dependency updates and ordinary component HMR are
qualified separately from post-error recovery. Parcel 2.16.4 can reload a stale
HTTP-cached document after rapid compilation-error recovery
([tracked issue](https://github.com/ubugeeei-prod/vize/issues/10736)). This remains
an explicit failing frontier; content hashing alone is not a qualified workaround.
The `test:recovery-frontier` script retains strict latest-source assertions
with browser caching enabled.

Parcel tracing (`shouldTrace: true`) records individual native compiler, parser,
asset and CSS operations. These spans measure each NAPI call's inclusive wall
time; internal Rust phases are not inferred from JavaScript samples.
