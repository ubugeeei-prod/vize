# Vue 3 removal checks: corpus false positives and the data-object check

Issue [#9059](https://github.com/ubugeeei-prod/vize/issues/9059). The maintainer asked
for more rules on by default in each preset.
[#6097](./2026-10-13-6097-essential-vue3-removal-defaults.md) enabled nine Vue 3
removal checks in `essential` and kept `script/no-deprecated-data-object-declaration`
opt-in until plain modules stop counting as components. This record covers the
work that remained.

## Corpus run

Every single-file rule ran over 34 `tests/_fixtures/_git` projects, covering `.vue`,
`.js`, `.ts`, `.jsx` and `.tsx` files. 31 projects are Vue 3. The others are Vue 2
code: `element`, `vue-element-admin`, and the Vue 2.7 site and theme generator in
`tdesign`. The run found two false positives in rules that are now on by default.

- `vue/no-deprecated-filter` read TypeScript unions in TSX expressions as filters
  in `naive-ui` (6 findings). JSX never had filters, so the rule now skips `.jsx`
  and `.tsx`.
- The Vue 2 migration script checks ran on every plain module. `vant` vendors
  vue-lazyload, which has its own `this.$on`, and that call was reported. A
  `site.config.ts` object with `data: {}` would also have been reported. As in
  eslint-plugin-vue, these checks now run only on `.vue`, `.jsx` and `.tsx` files,
  and on other modules that call `defineComponent`.

With both fixes, the ten removal checks report nothing in the Vue 3 projects. All
129 findings are in the Vue 2 code.

## Decision

The new gate meets the condition that #6097 set, so `essential`, and every preset
built on it, now also enables `script/no-deprecated-data-object-declaration`. It
keeps its Error severity, and `compiler.compatibility.vueVersion` set to `2` or
`2.7` turns it off, as it does for the other removal checks.

## Not affected

n8n's configuration uses `preset: "incremental"` with an explicit rule list. The list
includes `vue/no-deprecated-filter` and the `$listeners` and `$scopedSlots` checks. The
n8n checkout has no findings for them in JSX or plain modules, so its results stay
the same.
