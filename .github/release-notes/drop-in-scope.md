## Drop-in scope

`@vizejs/vite-plugin` is a drop-in replacement for `@vitejs/plugin-vue` on **Vue 3 SFCs**
(`<script setup>` and Options API). Vue 2 / 2.7 (`vue.version: "2"` / `"2.7"`) is
incubating and opt-in, webpack / rollup / esbuild / Rspack are outside the drop-in claim
(`@vizejs/unplugin` and `@vizejs/rspack-plugin` are experimental), and plugin-option
parity with `@vitejs/plugin-vue` is still incomplete.

See [Drop-in Scope](https://vizejs.dev/guide/vite-plugin#drop-in-scope) and
[#3227](https://github.com/ubugeeei-prod/vize/issues/3227).

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
