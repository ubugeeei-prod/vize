# Configuring CLI project findings

The CLI applies the same per-file resolved `linter.rules` and `entries[].linter.rules`
configuration to findings emitted with `--cross-file`. Route-typing rules use their
reported `ecosystem/vue-router-*` IDs. The `cross-file` group controls the Croquis
findings, and a complete diagnostic code overrides the group for one finding:

```json
{
  "linter": {
    "rules": {
      "ecosystem/vue-router-unknown-route": "warn",
      "cross-file": "warn",
      "vize:croquis/cf/unmatched-inject": "off"
    }
  }
}
```

All three values `off`, `warn` and `error` are supported. Categories can also set
`cross-file` or `ecosystem`; route findings additionally follow the existing
`suspicious` ecosystem category. A disabled category suppresses its findings;
otherwise an explicit rule (then the cross-file group) overrides a category's
severity. Defaults and existing diagnostic IDs remain unchanged. A warning-only
pass exits successfully unless the configured warning limit is exceeded. These
settings configure the CLI project pass; they do not turn route typing into a
single-file rule or add the project rules to a preset.
