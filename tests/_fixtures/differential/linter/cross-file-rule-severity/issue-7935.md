## Summary

The route-typing diagnostics that `--cross-file` adds (for example `ecosystem/vue-router-unknown-route`) cannot be configured through `linter.rules`. Setting the rule to `"warn"` or `"off"` has no effect: the finding is always reported as an error and makes `vize lint` exit non-zero.

`ecosystem/vue-router-unknown-route` is documented in `docs/content/rules/ecosystem.md`, but it is not in `getPatinaRules()` (only `ecosystem/vue-router-prefer-named-link`, `ecosystem/vue-router-prefer-named-push` and `ecosystem/router-link-require-to` are), and `crates/vize/src/commands/lint/routes.rs` maps the diagnostic's own severity straight to `LintDiagnostic::error` / `warn` without looking at the config.

## Reproduction

Use the files from #7932 (any project where route typing reports `ecosystem/vue-router-unknown-route`) and this config:

```json
{ "linter": { "preset": "ecosystem", "rules": { "ecosystem/vue-router-unknown-route": "off" } } }
```

```sh
vize lint -c vize.config.json --cross-file "src/**/*.vue" "src/**/*.ts"
```

## Actual (0.432.0)

```text
× [vize:ecosystem/vue-router-unknown-route] unknown route name `users-id`
...
1 error in 4 files
```

Same with `"warn"`. The `[vize:cross-file] vize:croquis/cf/*` findings are not configurable either: their severity is fixed per finding (for example `unmatched-inject` without a default and `async-boundary` are always errors, regardless of `linter.rules` / `categories`), so a project that wants `--cross-file` as an advisory pass cannot keep it from failing.

## Expected

- Rules that `--cross-file` emits under a rule name (`ecosystem/vue-router-unknown-route` and its siblings) honor `linter.rules` (`off` / `warn` / `error`), like other rules.
- Ideally the `cross-file` findings can be adjusted too (per finding or as a group), or at least follow `categories`.

## Environment

- vize / @vizejs/native 0.432.0
- macOS arm64, Node 26
