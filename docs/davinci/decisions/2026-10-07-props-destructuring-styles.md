### 2026-10-07: props destructuring style parity (#7988)

`script/define-props-destructuring` accepts `destructure: only-when-assigned |
always | never` through `linter.ruleOptions`, consistently in the CLI and LSP.
The default is `only-when-assigned`, matching eslint-plugin-vue and allowing
Vue 3.5 reactive defaults to pass alongside `script/no-with-defaults`. `never`
remains available for explicit props-object style. Diagnostics explain the style
preference rather than claiming Vue 3.5 destructuring loses reactivity.

The checked-in legacy corpus covers each mode, withDefaults, standalone macros,
and unrelated member calls. No Davinci-native completion credit is implied.
