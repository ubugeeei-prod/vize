# Storage summary run artifact

Storage totals by category, scope and owned-storage type are generated from the
reviewed [per-file storage inventory](./storage-inventory.tsv). Full CI and
release validation publish them as `storage-summary.md` in the
`davinci-generated-ledgers` Actions artifact for the validated checkout.
Generated aggregate tables are not committed.

Generate and verify the bundle locally with:

```sh
node tools/support/compat/davinci/generated-ledgers.mjs --write
node tools/support/compat/davinci/generated-ledgers.mjs --check
```

Outputs live in the Git-ignored `artifacts/davinci-ledgers/` directory. To compare
an isolated individual report, pass `--out-dir <dir>` to
`node tools/support/compat/davinci/storage-summary.mjs --write` and `--check`.
The check rejects absent output and any byte difference from the current TSV.

The [storage boundary](./storage-boundary.md) remains the policy. Its reviewed
per-file TSV stays committed and must exactly match the production source scan;
moving aggregate tables into artifacts does not change that ratchet. Dated
measurements in phase records remain historical evidence, not current totals.
See the [generated-ledger decision](../decisions/2026-09-27-generated-ledgers.md)
for the audit of retained evidence.
