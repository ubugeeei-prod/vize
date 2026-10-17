## Drop-in scope

`@vizejs/vite-plugin` is a drop-in replacement for `@vitejs/plugin-vue` on **Vue 3 SFCs**
(`<script setup>` and Options API). Vue 2 / 2.7 (`vue.version: "2"` / `"2.7"`) is
incubating and opt-in, webpack / rollup / esbuild / Rspack are outside the drop-in claim
(`@vizejs/unplugin` and `@vizejs/rspack-plugin` are experimental), and plugin-option
parity with `@vitejs/plugin-vue` is still incomplete.

See [Drop-in Scope](https://vizejs.dev/guide/vite-plugin#drop-in-scope) and
[#3227](https://github.com/ubugeeei-prod/vize/issues/3227).
