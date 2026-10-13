# Python, Lean and MoonBit tooling retire behind a ratchet

This [#8840](https://github.com/ubugeeei-prod/vize/issues/8840) record adds
two maintainer decisions of 2026-10-13 to the
[strictly typed TypeScript migration](./2026-10-13-8840-typescript-tooling-migration.md):
repository tooling stops using **Python**, which moves to TypeScript, and
stops using **Lean**, whose proofs move to Rust Kani harnesses. MoonBit
tooling already moves to TypeScript under the same issue.

## The ledger

[`tools/config/retired-tooling-languages.json`](../../../tools/config/retired-tooling-languages.json)
lists every tracked `.py`, `.lean`, `lean-toolchain` and `.mbt` file that
still waits for its port. At this change it holds 47 Python, 41 Lean and 21
MoonBit files.

[`tests/tooling/retired-tooling-languages.test.ts`](../../../tests/tooling/retired-tooling-languages.test.ts)
makes the ledger a ratchet. It runs in the `PR source checks` plan job beside
the other source preflights, so it cannot be skipped by path selection:

- a tracked file in one of these languages that the ledger does not list
  fails the check, so no new Python, Lean or MoonBit tooling can land;
- a listed file that no longer exists fails the check, so the change that
  ports or deletes a file also deletes its entry;
- a listed file that the comparison base does not contain fails the check, so
  the ledger cannot grow and a file cannot be moved instead of ported.

The MoonBit dialect fixtures under `davinci/vize_dialect_moonbit/tests/` test
a product feature. They are the only `permanent` entry and are not tooling.

## Replayed one-shot scripts are deleted, not ported

Seventeen Python scripts existed only to replay a finished move or rename:
fourteen under `tools/support/levels/` with their `l0-runtime-moves.json`
map, and `move-moonbit-lines.py`, `rename-guest-sdk.py` and
`rename-incremental-tier.py` under `tools/commands/davinci/`. Nothing
executes them. Their moves merged between 2026-09-28 and 2026-10-04, and the
source paths they rewrite no longer exist on `main`, so a replay cannot run.

They are deleted rather than ported. The decision records that name them
keep describing the original replay, and the scripts stay readable in git
history at the parent of this change. `rename-l0-dump-tests.ts` no longer
patches the deleted Python replay and still integrates the plan record.

## Order of the remaining ports

1. Python that runs on the pull-request and release paths: the native-fault
   evidence of the JS package tests, the semver allowance, the router page
   type controls, the public-install collectors and the PTY command helper.
2. Python behind on-demand measurement workflows: artifact payloads, Rust
   archive transport, allocation ownership, owned-worker capture and the
   mounted-runtime captures.
3. Lean: the HTML content model proofs first, then the Impeto laws, then the
   Impeto reference interpreter and its fixture differentials. Every pull
   request maps each Lean theorem to the Kani harness or exhaustive Rust
   test that replaces it. A property Kani cannot bound is recorded as such.
4. MoonBit: the release and publish commands, then `tools/moon/lib` and
   every `setup-moonbit` step that only served tooling. The `level-moonbit`
   workflow keeps `setup-moonbit`, because it type-checks the MoonBit dialect
   with the native `moonc` and that is a product feature.

Each group lands as its own pull request and shrinks the ledger.
