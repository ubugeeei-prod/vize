## Rust native API migration

The 0.431.0 minor release includes native Rust API additions. Existing
`NativeSfcCompileOptions` struct literals need `scope_id: None` to retain the
filename-derived scope identity, or `Some("data-v-...")` for an explicit identity.
Exhaustive matches on `NativeSfcCompileError` and the level lowering
`UnsupportedReason` must handle their new refusal variants.

The native pipeline remains experimental and opt-in. This release does not
claim that native product migration or the product fix-history gates are complete.

## Community lint improvements

- `vue/prefer-props-shorthand` now reports and fixes same-name bindings on native
  elements as well as components, including `:style="style"` and
  `:aria-label="ariaLabel"` ([#7417](https://github.com/ubugeeei-prod/vize/issues/7417)).
- The opt-in `type/strict-boolean-expressions` rule checks non-boolean conditions
  in scripts and templates ([#7377](https://github.com/ubugeeei-prod/vize/issues/7377)).
- Cross-file lint detects attributes passed to a component whose root cannot
  receive fallthrough attributes ([#7273](https://github.com/ubugeeei-prod/vize/issues/7273)).

Thank you [@naitokosuke](https://github.com/naitokosuke) for these reports and proposals.

## Vapor static attributes

`compileVapor` preserves nested character references and quoted values in static
attributes when HTML templates are parsed at runtime or rendered from the IR on
the server ([#7502](https://github.com/ubugeeei-prod/vize/issues/7502)).

Implicit default content beside a named slot template is also retained in the
Vapor IR and generated slot functions ([#7570](https://github.com/ubugeeei-prod/vize/issues/7570)).

Static style values are retained beside `:style` in whitespace-preserve and
parser-recovery compilation, including the public `transform_to_ir` API
([#7600](https://github.com/ubugeeei-prod/vize/issues/7600)).

Thank you [@dannote](https://github.com/dannote) for these reproductions.

## Drop-in scope

`@vizejs/vite-plugin` is a drop-in replacement for `@vitejs/plugin-vue` on **Vue 3 SFCs**
(`<script setup>` and Options API). Vue 2 / 2.7 (`vue.version: "2"` / `"2.7"`) is
incubating and opt-in, webpack / rollup / esbuild / Rspack are outside the drop-in claim
(`@vizejs/unplugin` and `@vizejs/rspack-plugin` are experimental), and plugin-option
parity with `@vitejs/plugin-vue` is still incomplete.

See [Drop-in Scope](https://vizejs.dev/guide/vite-plugin#drop-in-scope) and
[#3227](https://github.com/ubugeeei-prod/vize/issues/3227).

## Declaration formatting

`vize fmt` now accepts ambient `const` declarations in `.d.ts`, `.d.mts`, and
`.d.cts` files.

## Nuxt page metadata

`definePageMeta` imported explicitly from `#imports` now produces route metadata
and is removed from the component setup code, matching Nuxt's auto-import path.

## Reporter acknowledgements

All nine fixes are included in
[v0.429.1](https://github.com/ubugeeei-prod/vize/releases/tag/v0.429.1), whose
[tag commit](https://github.com/ubugeeei-prod/vize/commit/9aaa1fe458a09e0d0c6604dc8835ccf7c737d943)
contains all nine fixes. This release adds explicit reporter credit for those
earlier fixes.

- [@naitokosuke](https://github.com/naitokosuke): Nuxt SSR critical CSS
  [#6897](https://github.com/ubugeeei-prod/vize/issues/6897), SSR slot scopes
  [#6898](https://github.com/ubugeeei-prod/vize/issues/6898) and
  [#6767](https://github.com/ubugeeei-prod/vize/issues/6767), CSS binding expressions
  [#6766](https://github.com/ubugeeei-prod/vize/issues/6766), and the shared Nuxt Vue
  runtime [#6684](https://github.com/ubugeeei-prod/vize/issues/6684).
- [@logica0419](https://github.com/logica0419): repeated Nuxt style resolution
  [#6825](https://github.com/ubugeeei-prod/vize/issues/6825).
- [@dannote](https://github.com/dannote): Vapor hydration order
  [#6728](https://github.com/ubugeeei-prod/vize/issues/6728) and sibling traversal
  [#6727](https://github.com/ubugeeei-prod/vize/issues/6727).
- [@dmnlk](https://github.com/dmnlk): explicit tsconfig files beneath an ancestor
  `node_modules` [#6683](https://github.com/ubugeeei-prod/vize/issues/6683).

Thank you for the concrete reproductions that made these regressions actionable.

## JSX spread children

Native JSX `<div>{...items}</div>` spreads the items into the children, as
`@vue/babel-plugin-jsx` does, instead of rendering `toDisplayString(items)`.
VDOM output supports it. Vapor and SSR report it as unsupported.

## JSX rendering fix

Native JSX `value && <Child/>` preserves falsy numbers such as `0` and `NaN`
instead of rendering nothing. Boolean, nullish and empty-string children stay
empty, and the condition is evaluated once per render or reactive update.
