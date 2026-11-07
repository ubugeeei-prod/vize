# @vizejs/parcel-transformer-native

Compile Vue single-file components in Parcel 3 through Parcel's native plugin ABI.
The transformer loads as a Rust dynamic library and calls the Vize compiler directly;
Node.js and N-API are not involved in the transform path.

```sh
pnpm add -D parcel@3.0.0-alpha.2 @vizejs/parcel-transformer-native
```

```json
{
  "extends": "@parcel/config-default",
  "transformers": {
    "*.vue": ["@vizejs/parcel-transformer-native"]
  }
}
```

The transformer supports JavaScript and TypeScript script blocks, templates, scoped
styles, source maps, and static template asset URLs. Plain and scoped CSS is injected
once at runtime because the native transformer ABI returns one asset. External SFC
blocks, custom blocks, CSS modules, preprocessors, and CSS `@import` fail explicitly
instead of producing partial output.

An object entry may configure the renderer and style policy:

```json
{
  "plugin": "@vizejs/parcel-transformer-native",
  "config": {
    "ssr": false,
    "vapor": false,
    "styles": "inject"
  }
}
```

`styles` accepts `"inject"`, `"ignore"`, or `"error"`. `"ignore"` deliberately drops
compiled CSS; `"error"` prevents accidental style loss. `transformAssetUrls` defaults to
`true` and can be disabled for hosts that manage template URLs separately. During
repository development, `parcel.devLibrary` points Parcel at the workspace debug build.
Published packages use per-platform optional artifacts instead.
