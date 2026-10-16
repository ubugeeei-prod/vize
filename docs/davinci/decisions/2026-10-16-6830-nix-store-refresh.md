# Refresh the qualified Nix store without losing build dependencies

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Companion: [CI caches and distributed builds](./2026-10-16-6830-distributed-builds.md).

## Evidence

The old `nix-flake` job keyed its entire store only by platform and `flake.lock`.
On [October 14, job 115760157213](https://github.com/ubugeeei-prod/vize/actions/runs/38536381434/job/115760157213),
that exact key hit and the cache action explicitly skipped saving. New source
outputs from that run could therefore never update the immutable cache entry.

On [October 15, job 116318820085](https://github.com/ubugeeei-prod/vize/actions/runs/38704015046/job/116318820085),
the Nix job passed in 6m42s, including 6m18s for `nix flake check` and 16s for
saving the store. The log reports an 8,479,237,000-byte store and a
2,737,695,962-byte compressed upload. It also confirms that no garbage
collection limit was configured. These are baseline observations, not timings
for the changed workflow.

## Decision

Keep the existing pinned installer, cache action, runner, and full flake check.
The Nix job calls the [check-nix composite](../../../.github/actions/check-nix/action.yml)
after checkout, keeping its scheduled/manual trigger and timeout. Change its
cache lifecycle:

- The primary key includes a `nix-v2` namespace, OS/architecture, the hash of
  locked dependencies and Nix recipes, and the source SHA. A successful source
  revision can now save the outputs it actually qualified.
- Restore exactly one archive: first prefer the same dependency/recipe hash,
  then another v2 cache on the same platform, and finally the old exact
  flake-lock namespace for migration. Do not merge multiple whole-store
  archives. Nix still determines whether each restored derivation is reusable.
- Only a checkout of the repository default branch may save. Manual checks on
  other refs restore existing entries and run the same checks without writing
  a candidate cache.
- Before a new save, ask the existing cache action to collect garbage toward
  an 8 GiB store target. Exact-key reruns neither collect nor upload again.

The chosen inputs and post-save behavior are supported by the
[pinned cache action](https://github.com/nix-community/cache-nix-action/blob/7df957e333c1e5da7721f60227dbba6d06080569/action.yml)
and its
[save implementation](https://github.com/nix-community/cache-nix-action/blob/7df957e333c1e5da7721f60227dbba6d06080569/src/saveImpl.ts).

Nix garbage collection does not implement least-recently-used eviction. After
`nix flake check` succeeds, `nix build --out-link` registers the current CLI,
Crane `cargoArtifacts`, and checked shell inputs as GC roots. Those outputs
were already built by the check; this step retains them rather than introducing
another product build. Enabling `keep-derivations` and `keep-outputs` preserves
their build dependencies too. Older unrooted outputs can then be collected.
[Nix build links](https://nix.dev/manual/nix/2.34/command-ref/new-cli/nix3-build.html),
[Nix GC configuration](https://nix.dev/manual/nix/2.35/command-ref/conf-file.html#conf-keep-outputs).

Eight GiB is a GC target, not permission to remove live dependencies. The store
may exceed it if the current rooted closure itself is larger. The existing
cache action logs the size before and after collection, making that condition
visible. This preserves useful compilation artifacts instead of meeting a
nominal archive size by deleting the cache's most valuable contents.

This change still transfers one compressed store archive to an ephemeral
runner. It does not claim to provide lazy per-object restoration. Mounting an
entire live `/nix` store on a sticky disk would also require handling the daemon,
store database, installation, and trust lifecycle; that migration is not part
of this change.

## Validation

The workflow and composite YAML parse. Nix evaluated the Linux Crane dependency derivation,
and an offline `nix build --dry-run` accepted all three GC-root installables.
The full `nix flake check` command and scheduled/manual qualification remain
unchanged. No Nix recipe or lockfile was modified. To exercise the composite on
Actions, run `gh workflow run check.yml --ref <candidate-branch>`; candidate
branches restore only, while the first cache save requires the default branch.

Actions must still demonstrate the first v2 save, a later matching-prefix
restore, retained dependency reuse, and GC sizes. Do not report a measured
speedup until those runs complete.
