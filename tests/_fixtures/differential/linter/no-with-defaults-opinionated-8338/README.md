# Opinionated withDefaults preset regression

Paired issue: [#8338](https://github.com/ubugeeei-prod/vize/issues/8338).
Actual baseline: `f46c40071c78b191f382956a88aef1bef7380b16`.

The baseline registers `script/no-with-defaults` with no preset. The source-built
legacy CLI must now emit its existing warning through `opinionated` without a
rule override. The same macro remains accepted by the generic default and the
other five presets. `strict` and `all` retain their existing opinionated aliases.
Reactive destructuring, comments, strings and ordinary same-name helpers stay
accepted. Every complete source is byte/hash pinned in `cases.json`.

`tests/tooling/lint-no-with-defaults-opinionated.test.ts` compares complete JSON
reports and process results, including the existing `--max-warnings 0` failure.
It requires the normal source-built CLI receipt and records every run. This is
legacy CLI corpus coverage; it does not claim admission to the separate shared
44-case API/native linter pack. The original #7962 macro ownership corpus and
compiler compatibility fixtures remain unchanged.
