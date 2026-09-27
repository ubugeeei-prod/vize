# Rule-parity run artifact

The current all-rule inventory and whole-repository counts are generated from
`crates/vize_patina/src` during full CI and release validation. They are published
in the `davinci-generated-ledgers` Actions artifact as `rule-parity.md` for that
run's checkout. Generated tables are not committed.

Generate and verify the same bundle locally with:

```sh
node tools/support/compat/davinci/generated-ledgers.mjs --write
node tools/support/compat/davinci/generated-ledgers.mjs --check
```

The output lives in `artifacts/davinci-ledgers/`, which Git ignores. For an
isolated individual report:

```sh
node tools/support/compat/davinci/rule-parity.mjs --write --out-dir /tmp/vize-rule-parity
node tools/support/compat/davinci/rule-parity.mjs --check --out-dir /tmp/vize-rule-parity
```

The generator derives rule identity, registration surfaces, SFC/JSX dispatch,
Croquis use and precision tiers from sources. It still rejects duplicate
rule names, missing identities, changed dispatch anchors, inconsistent tier
declarations and invalid authored overrides.
[Rule classification overrides](./rule-parity-overrides.toml) remain reviewed,
committed input. Classification remains heuristic and dispatch membership does
not prove a runtime effect or production fact adoption.

Counts quoted in dated phase records are historical evidence. Use the artifact
from the validated checkout for current counts; do not update those historical
records from a new scan. The [generated-ledger decision](../decisions/2026-09-27-generated-ledgers.md)
records the boundary between run reports and retained compatibility evidence.
