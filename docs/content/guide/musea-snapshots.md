# Snapshot identity and migration

Musea keeps each baseline associated with a project-relative Art path,
variant, viewport dimensions and device scale. Different components named
`Button.art.vue` can have independent baselines. A rebuilt hosted gallery can
reuse them even when its build machine's absolute source directory changes.

Commit or transfer the snapshot directory's **`identities.json` and baseline
PNGs together**. The index records ownership and preserves reservations when
an Art is removed. Ordinary noncolliding PNG names stay familiar; colliding
captures receive deterministic `snapshot-<hash>.png` names.

## Migrating existing baselines

Existing PNGs created before the ownership index have no recorded owner.
Review their Art, variant and viewport, then adopt unambiguous baselines once:

```sh
vp exec musea-vrt --adopt-legacy-snapshots --json
```

For a hosted gallery, include its URL:

```sh
vp exec musea-vrt --gallery-url https://example.com/components/ \
  --adopt-legacy-snapshots --json
```

An old hosted gallery must be rebuilt with a Musea version that emits
snapshot identity version 1. The CLI does not guess a remote build root from
your local directory.

If multiple captures share an old filename, Musea cannot determine which
component owns that PNG. Explicit adoption refuses it. Capture new qualified
baselines and review them; preserve the old PNG until that review is complete.

## Reviewing one component

Use the path relative to the configured project root, omitting `.art.vue`:

```sh
vp exec musea-vrt approve 'right/Button/*' --gallery-url https://example.com/components/
```

`approve 'Button/*'` refuses when multiple Art files match. `approve` without
a pattern approves all currently failing captures. Clean keeps the ownership
index, including reservations for removed components and variant-specific
viewport overrides.

## Project roots and concurrent runs

The default identity root is Vite's `root`. If a gallery includes Art files
from other packages, set Musea's `projectRoot` to their common project root:

```ts
musea({ projectRoot: '../..' })
```

The CLI and static build use that same root. Moving the whole project keeps
relative identities unchanged. Changing the root or moving an Art within
the project changes its identity and requires a reviewed baseline migration.

One runner owns a snapshot directory at a time. Concurrent runs fail with a
lock error. If a process crashes, confirm no VRT process is still using the
directory before removing its stale `identities.lock`; retain
`identities.json`. Corrupt ownership data must be repaired or restored before
capturing, approving or cleaning PNGs.
