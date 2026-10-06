## Area

`vize lint` configuration (`vize.config.*`)

## Version

`vize` 0.432.0

## Minimal reproduction

`MyText.vue`

```vue
<template>
  <p style="color: red">x</p>
</template>
```

`vize.config.json` (`vue/no-inline-styles` is a typo for `vue/no-inline-style`)

```json
{
  "linter": {
    "preset": "incremental",
    "rules": { "vue/no-inline-styles": "warn", "vue/no-such-rule": "error" }
  },
  "entries": [{ "files": ["**/*.vue"], "linter": { "rules": { "a11y/no-autofocuss": "off" } } }]
}
```

```sh
vize lint -f plain --help-level none MyText.vue; echo "exit=$?"
```

## Actual

```
Patina lint report: No problems found in 1 file(s)
exit=0
```

No warning about the three unknown names. With the name fixed (`vue/no-inline-style`) the file is
reported (`MyText.vue:2:6 warning vue/no-inline-style …`), so the typo silently turned the rule
off. In a long config (dozens of opt-in rules and per-directory `entries`) there is no way to
notice this, and a misspelled `"off"` in `entries` silently keeps a rule on.

## Expected

An error (or at least a warning) naming the unknown rule ids, as Oxlint does:

```
$ oxlint a.ts   # .oxlintrc.json: { "rules": { "no-debuggerr": "error" } }
Failed to parse oxlint configuration file.

  x Rule 'no-debuggerr' not found in plugin 'eslint'
```

The CLI already knows the full list (`LINT_RULE_NAMES` in `vize/src/types/rules.ts`, the
registry behind `docs/content/rules/all.md`). A "did you mean `vue/no-inline-style`?" hint would
be a bonus.

Small related inconsistency found while checking names: `script/no-next-tick` is documented in
`docs/content/rules/all.md` and works, but is missing from `LINT_RULE_NAMES`;
`vue/no-unused-setup-bindings` is in `LINT_RULE_NAMES` and works, but is missing from `all.md`.
