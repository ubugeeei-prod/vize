# Whole-repository generated ledgers

Issue: [#6867](https://github.com/ubugeeei-prod/vize/issues/6867).

## Decision

Current whole-repository count tables are run artifacts. CI inventory validation
generates the bundle from the checkout being validated and publishes it as
`davinci-generated-ledgers`. Source contracts and reviewed compatibility inputs
remain blocking checks. PRs no longer rewrite the generated rule inventory or
storage aggregate tables, so independent changes do not conflict on their totals.

`node tools/support/compat/davinci/generated-ledgers.mjs --write` emits the bundle
under the ignored `artifacts/davinci-ledgers/` directory. `--check` regenerates
from sources and rejects missing, edited or unexpected output. Both modes accept
`--out-dir <dir>` for isolated validation. Individual rule and storage generators
have the same byte-exact modes. Staleness tests generate a scratch copy and prove
that an injected edit fails; they do not skip the source derivation.

## Report audit

| Report or input                                                                                      | Treatment and reason                                                                                                                                                                                                                                                |
| ---------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `plan/rule-parity.md`                                                                                | Authored, count-free guide. Current all-rule tables and aggregates are emitted as `rule-parity.md`; authored classification overrides remain in `rule-parity-overrides.toml`.                                                                                       |
| `plan/storage-summary.md`                                                                            | Authored, count-free guide. Category and scope aggregates are emitted as `storage-summary.md`; reviewed `storage-inventory.tsv` still exactly matches production source storage.                                                                                    |
| Croquis cross-crate totals                                                                           | Emitted as `analysis-consumption-summary.md`. The committed `croquis-consumption.md` method/product set and per-crate shards contain source-specific evidence, already omit whole-repository totals, and retain strict shard-set staleness checks and demand gates. |
| Consumer migration totals                                                                            | Emitted as `consumer-migration-summary.md`. Committed configuration/method and per-consumer/per-crate TSVs are independent source-specific migration evidence, with no cross-file totals.                                                                           |
| Loc-shaped span read totals                                                                          | Emitted as `sourcelocation-summary.md`. `sourcelocation-inventory.md` retains the zero-read regression contract and historical migration explanations; regeneration rejects deleted carriers or member reads returning.                                             |
| `plan/corpus-coverage.md`                                                                            | Retained corpus-specific measured scope witness. Its hydrated-project footer and expansion proofs are consumed by phase-exit tests; replacing it with an arbitrary current checkout scan would destroy that evidence.                                               |
| `plan/v-on-corpus/*.tsv`                                                                             | Retained per-source/per-area fixture evidence for measured inline storage bounds; no aggregate totals are committed. Its exact spelling inventory remains a deliberate-update tripwire.                                                                             |
| Fixture planes, pinned HTML data, machine JSON, storage/witness TSVs, budget/override/allowlist TOML | Retained executable reference data, reviewed ratchets or compatibility inputs. These are not whole-repository prose count reports.                                                                                                                                  |
| Dated phase records, FP/FN/compile waivers and decision records                                      | Retained authored decisions, triage, fixture proof and historical measurements. A new scan must not overwrite their recorded evidence.                                                                                                                              |

This issue changes where observational count reports live. It does not establish
production fact adoption, change compatibility waivers or complete a product
migration. The generated artifact always describes its own validated checkout;
dated counts in records remain dated evidence.

The `check-level-inventories` composite action keeps all source witness
commands in the owning Check job, generates and checks the bundle, and uploads
it even when a later inventory command fails. The inventory step retains its
current pull-request, push and merge-group condition. Moving generation to the
full merge-queue, nightly and release tiers is tracked separately in #6864;
this artifact change does not remove current PR inventory coverage.

The inventory action directory is `check-level-inventories`, following the
level-only naming rule. The move is isolated from its caller updates. Existing
check display names and artifact paths retain their published contract.
