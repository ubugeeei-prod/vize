# Inventory shards are run artifacts

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

The per-crate shards of the consumer migration surface inventory
(`docs/davinci/plan/consumer-migration-surfaces/<consumer>/<crate>.tsv`) and
of the Croquis consumption matrix
(`docs/davinci/plan/croquis-consumption/<crate>.md`) are no longer committed.
Both generators write them on every `--write` and `--check` under
`artifacts/davinci-ledgers/`, and the source preflight uploads that directory
in the `davinci-generated-ledgers` artifact it already publishes.

The two index pages stay committed and byte-compared. A shard committed under
either plan directory is stale, and `--write` deletes it. The Croquis gate on
products with no consumer and no demand gate is unchanged.

## Reason

The shards record file paths and the first line of every matching `use`. A
pull request that moved one import line in a linter, compiler, Canon or
Croquis source had to regenerate a shard, and two such pull requests touching
the same crate conflicted on it; #8861 failed its source checks that way. The
[2026-10-03 sharding](./2026-10-03-croquis-census-reports.md) reduced the
conflicts to one crate at a time but kept the line numbers in the tree.

A shard is an observation of the current source, not a decision, so it belongs
to the run that observed it. The index pages carry the method, the scopes and
the product set, which change only when the inventory itself changes.

## Scope

Readers who used the committed shards download the `davinci-generated-ledgers`
artifact of a run or run
`rust-script tools/commands/davinci/consumer-migration-surfaces.rs --write`
(or `croquis-consumers.rs --write`) and read `artifacts/davinci-ledgers/`.
Earlier records that cite a committed shard path describe the tree at their
own commit.
