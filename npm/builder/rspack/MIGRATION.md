# Migrating Rspack configuration

Compilation options belong to the SFC or JSX loader. File selection belongs to
Rspack rules. `VizePlugin` configures CSS integration, automatic rules, TypeScript
post-processing, and infrastructure debug messages.

## Move compilation options to the loader

Earlier plugin types exposed compilation options without forwarding them to the
loader. Those fields remain accepted and produce a migration warning. Their old
behavior is preserved: setting them on the plugin does not change compilation.
There is no plugin-versus-loader compilation precedence to resolve.

Before:

```javascript
{
  module: {
    rules: [{ test: /\.vue$/, loader: "@vizejs/rspack-plugin/loader" }],
  },
  plugins: [new VizePlugin({
    vapor: true,
    compilerOptions: { experimentalSelfComponent: true },
  })],
}
```

After:

```javascript
{
  module: {
    rules: [{
      test: /\.vue$/,
      loader: "@vizejs/rspack-plugin/loader",
      options: {
        vapor: true,
        compilerOptions: { experimentalSelfComponent: true },
      },
    }],
  },
  plugins: [new VizePlugin()],
}
```

The second configuration actually enables those compilation options. Moving a
previously ineffective field can change output; verify that this is the intended
behavior. If both locations contain a value, preserve the existing loader value
to preserve compilation behavior, then remove the plugin field.

## Option mapping

| Previous location                                         | Recommended location                        | Compatibility behavior                                                                             |
| --------------------------------------------------------- | ------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| Plugin `ssr`, `vapor`, `sourceMap`, `compilerOptions`     | SFC loader options                          | Plugin fields do not configure compilation; warn when set                                          |
| Plugin `jsxMode`, `jsxCompat`                             | JSX loader options                          | Plugin fields remain ineffective and warn                                                          |
| Plugin `root`                                             | SFC loader `root`, or Rspack `context`      | Plugin field remains ineffective and warns                                                         |
| Plugin `isProduction`                                     | Rspack `mode`                               | Legacy field still controls plugin flags and logging, not SFC compilation; warns                   |
| Plugin `include`, `exclude`                               | Rspack rule conditions                      | Legacy fields still filter plugin watch messages only; warn                                        |
| Loader `include`, `exclude`                               | Rspack rule conditions                      | Legacy filters still pass unmatched input through with a warning                                   |
| SFC loader `css.native`                                   | Plugin default, or a manual loader override | With automatic rules, must match the plugin mode; with manual rules, explicit values are preserved |
| Plugin `css.native`, `autoRules`, `typescript`, `debug`   | Same location                               | Supported integration options                                                                      |
| Loader `customElement`, `hotReload`, `transformAssetUrls` | Same location                               | Remain SFC loader options                                                                          |

`VizeSfcLoaderOptions` and `VizeJsxLoaderOptions` describe the separate entry
points. `VizeLoaderOptions` remains exported as a compatibility type. No removal
version for deprecated fields is scheduled by this change.

Set `ssr`, `vapor`, and `sourceMap` at loader top level. The nested fields in
`compilerOptions` are deprecated fallbacks. Explicit top-level values, including
`false`, take precedence. Previously, the SFC loader's defaults shadowed nested
values even when the top-level field was omitted. A nested setting can now take
effect; move it to loader top level to make the selected behavior explicit.
The JSX loader does not consume `compilerOptions`.

## File selection and environment

Use `test`, `include`, and `exclude` on the rule. Automatic style-rule cloning
preserves the Vue rule's conditions. Legacy loader filters return uncompiled
source when they exclude a file; they do not select another Vue compiler.

Set `mode` and `context` on Rspack. For SFC-specific overrides, loader
`isProduction` controls production output and loader `root` now controls scope ID
and development file-path calculations. Relative roots resolve against Rspack
context. Previously, loader `root` was declared but not used; an existing explicit
value can now change scope IDs and `__file`.

For separate client and server configurations, use loader `ssr: false` and
`ssr: true` respectively. The server configuration still needs its normal Rspack
target, externals, and output settings. Setting a Node target alone does not
enable SFC SSR compilation.

`sourceMap` controls the compilation request and forwarding of available native
maps. Its precedence is loader `sourceMap`, deprecated
`compilerOptions.sourceMap`, then Rspack's loader context. If all are omitted,
development enables maps and production disables them. Configure Rspack's
`devtool` to emit final bundle maps.

The SFC loader now preserves line and column mappings during module assembly,
including inserted imports, default-export rewrites, and template asset URL
rewrites. External script mappings refer to the external file. The current
native SFC map covers script code; forwarding does not add mappings for template
regions that native leaves unmapped.

## CSS mode

The plugin resolves a default CSS integration mode. With `autoRules: true`, its
generated style rules and SFC loaders must use that mode. Conflicting loader
values fail with an actionable error; matching values are accepted.

With `autoRules: false`, each SFC loader can explicitly select its own
`css.native` value. Both `true` and `false` override the plugin default without
deprecation warnings. The default is supplied only when the loader leaves the
mode unspecified. This applies to static entries in nested `rules`, `oneOf`,
loader shorthand, and `use` arrays.

| Rspack configuration                         | Plugin configuration                                           |
| -------------------------------------------- | -------------------------------------------------------------- |
| 1.x Native CSS: `experiments: { css: true }` | Auto-detected, or `css: { native: true }`                      |
| 2.x Native CSS                               | Auto-detected unless the CSS experiment is explicitly disabled |
| CssExtract / style-loader                    | `css: { native: false }`, with matching CSS rules              |

In automatic mode, configure the shared mode on the plugin. In manual mode,
retain loader overrides and configure the corresponding style sub-request
branches to match each loader's mode. Ordinary CSS rules can use a different
pipeline from SFC styles. Native CSS still requires the corresponding Rspack
capability, including `experiments.css` on Rspack 1.x.

For example, with `new VizePlugin({ autoRules: false, css: { native: true } })`,
a manually routed SFC main branch with `options: { css: { native: false } }`
retains its JS-based CSS mode. Its style branches must supply the matching
css-loader and extraction or injection chain. Other SFC loaders without an
override inherit `true`.

Automatic style-rule cloning
targets the first matching root-level Vue rule without `oneOf`. Function-valued `use`
entries are not rewritten; static Vize entries provide deterministic CSS mode
propagation.

## Logging and validation

Vize debug messages require `debug: true` and Rspack infrastructure logging that
enables the `VizePlugin` logger, for example:

```javascript
{
  infrastructureLogging: { level: "verbose", debug: /VizePlugin/ },
  plugins: [new VizePlugin({ debug: true })],
}
```

Warnings, including migration warnings, are independent of the debug option.

After migration, check client and server builds independently, verify rule
selection for excluded directories, and exercise the configured CSS pipeline.
Projects using several SFC rules can configure different compilation options on
each rule without a global plugin override.

This package remains a Vue 3 integration. This change adds no Vue 2 fallback,
Nuxt-specific adapter, or new framework support guarantee.
