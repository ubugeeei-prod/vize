# Rule-parity run artifact

The current all-rule inventory and whole-repository counts are generated from
`crates/vize_patina/src` during full CI and release validation. They are published
in the `davinci-generated-ledgers` Actions artifact as `rule-parity.md` for that
run's checkout. Generated tables are not committed.

Generate and verify the same bundle locally with:

```sh
node tools/support/compat/davinci/generated-ledgers.ts --write
node tools/support/compat/davinci/generated-ledgers.ts --check
```

The output lives in `artifacts/davinci-ledgers/`, which Git ignores. For an
isolated individual report:

```sh
node tools/support/compat/davinci/rule-parity.ts --write --out-dir /tmp/vize-rule-parity
node tools/support/compat/davinci/rule-parity.ts --check --out-dir /tmp/vize-rule-parity
```

The generator derives rule identity, registration surfaces, SFC/JSX dispatch,
Croquis use and precision tiers from sources. It still rejects duplicate
rule names, missing identities, changed dispatch anchors, inconsistent tier
declarations and invalid authored overrides.
The JSX/TSX report includes the conditional native plain-module type lane. Its
rule identities, admitted filename extensions, declaration exclusions and
configuration guards come from the actual preparation and diagnostic branches.
The report distinguishes admitted paths from default opt-out, query-free input,
and project or runtime failures; membership does not certify a successful type
query. The markup partitions and native type completion are checked separately.
[Rule classification overrides](./rule-parity-overrides.toml) remain reviewed,
committed input. Classification remains heuristic and dispatch membership does
not prove a runtime effect or production fact adoption.

Counts quoted in dated phase records are historical evidence. Use the artifact
from the validated checkout for current counts; do not update those historical
records from a new scan. The [generated-ledger decision](../decisions/2026-09-27-generated-ledgers.md)
records the boundary between run reports and retained compatibility evidence.
