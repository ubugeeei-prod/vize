Four rules found the same problems as eslint-plugin-vue 10.12.0 but reported a different range. Every finding therefore split into a false positive and a false negative in the Real Project Matrix (#12336):

| rule | Vize reported | eslint-plugin-vue reports |
| --- | --- | --- |
| `vue/no-console` | `console.error` (the member expression) | the `console` object |
| `vue/require-default-export` | one byte at `</script>` | the program's last token, the whole `</script>` end tag |
| `vue/no-literals-in-template` | the whole attribute value, including whitespace and parentheses | the literal node (`() => { ... }`, `{ ... }`, `[ ... ]`) |
| `vue/require-prop-comment` | the prop key | the prop node: `key: value` for runtime props; for TypeScript members, the member including its `;`/`,` delimiter, as in typescript-estree |

A local `lint-divergence-report.ts --all-mapped-rules` run over airi, mavon-editor, vux, scalar, dho-web-client and inertia shows 0 divergences for `no-console` and `require-default-export`. One `require-prop-comment` pair is left in scalar, in a file where the reference's own parser misplaces lines (fixed separately).

Tests:
- `no-literals-in-template`: a multi-line arrow function value, a parenthesized object and a padded array.
- `no-console`: static, computed and parenthesized `console` members; `logger.console` stays quiet.
- `require-default-export`: `</script  >` with trailing space.
- `require-prop-comment`: type-literal members with `;`, `,` and a method signature; runtime object, shorthand and computed keys; array props.

The input-only differential case `linter/eslint-vue-report-nodes-12336` pins the shapes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
