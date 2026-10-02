# Linter history inventory (#6881)

The [fix-history issue](https://github.com/ubugeeei-prod/vize/issues/6881)
must remain open until every historical behavior requirement has executable
input, exact diagnostics and offered/applied autofix evidence in the registered
differential corpus. Inventory rows and snapshot filenames are candidates only.
This change neither runs the linter nor measures native acceptance.

## Reproducible enumeration

From a complete repository checkout, run:

```sh
node tools/support/compat/fixtures/linter-fix-history.mjs \
  b4f25fb6511075aa531be80d645bb0db8cc151e0 \
  /tmp/vize-linter-history-inventory
```

The output directory receives `history.json`, `history.tsv` and `merges.tsv`.
Keep generated inventories as artifacts; do not commit them as whole-repository
ledgers. The tool resolves the input to a full commit ID, rejects shallow
history and missing revisions, ignores replacement objects, and reads Git
objects rather than working-tree files. A later revision produces a new
inventory; it does not rewrite an earlier count or witness.

The nonmerge list comes from `git log --full-history --no-merges` restricted
to `crates/vize_patina`, in topological order. Ordinary path simplification can
omit discarded side-branch fixes, so it is not sufficient. Each row contains
the full commit ID, subject, parents, SHA-256 of the raw scoped tree diff,
and every touched file in the whole commit. Raw deltas contain full before/after
blob IDs and modes, including binary changes, without patch context or custom
diff-driver dependence. Rename detection is disabled
for deltas so both removed and added paths remain visible. Fixed diff options
are recorded in the JSON protocol.

Reachable path-touching merges are a separate inventory with full parent IDs
and a scoped diff hash for each parent. Parent diffs preserve integration
topology; they do not classify conflict resolutions or supersession. All merge
rows still require remerge and requirement review.

## Counts and limits

The issue's original **499 touch commits / 255 reported fixes** are historical
counts, retained verbatim in the artifact. Its original counting method was
not specified. At the pinned revision above, full history contains **503
nonmerge touch commits**, **258 title-fix candidates**, **245 other titles**
and **48 merge supplements**. A title candidate matches the conventional
`fix` prefix; it is not a reviewed behavior classification.

The earlier read-only preparation at
`9e203108c7b77b8a0bf6243d5db1f2c09c878eac` enumerated the same 503 / 258
nonmerge rows and 47 merges. The additional merge is
`00a79d884026b551facc65cb1af7b6a874e0c26f` (latest layer naming into the Nuxt
style fix). No prepared requirement classification is automatically imported.
That preparation found behavior fixes with non-fix titles, so the 245 other
titles must also be reviewed.

Snapshot-reference candidates are only `.snap` files actually touched by a
commit inside the scoped crate. Each candidate records the blob before the
commit, at the commit and at the pinned revision; deletion is explicit `null`.
There are 320 such path candidates across 107 nonmerge commits at this pin.
These are exact Git references, not proof that the present snapshot is
executable or still expresses the historical requirement. The tool does not
infer source-test names, map renamed snapshots, import inline/count-only
assertions, or claim that commits without external snapshots lack tests.

Every nonmerge and merge row begins `unreviewed`. The summary always records
zero accepted corpus cases, zero executed tests and zero native passes.
Human-reviewed behavioral requirements need a separate, explicit record and
an executable fixture; they cannot be inferred from titles or file existence.

## Work remaining

- Decompose compound commits and review every non-fix title for behavior.
- Review merge resolutions, predecessor paths and rename/supersession chains.
- Associate each actual requirement with an executable corpus fixture and
  exact ordered diagnostics, offered edits, applied bytes and repeat linting.
- Distinguish historical expectations from changed requirements explicitly.
- Run the registered differential corpus in Actions and retain exact revision
  receipts. Unknown, skipped, unsupported and legacy-backed native results do
  not count as passes.

Five focused tooling tests use real temporary Git repositories to exercise
discarded side-branch history, merge topology, removed snapshot blob identity,
stable artifacts, unusual filenames, rename edges, and rejection of incomplete history.
They verify enumeration only. They are not product fixture acceptance.

## Separately reviewed original filename witnesses

`eaafa5a1f67883277407fcf1c5f3f2b2101ef2ed` (#4447) has a `feat` title.
Its six changed or added tests supply eight exact original filename witnesses
with `<div>Content</div>`, unchanged filenames and the sole rule. The
[bounded semantic record](./2026-10-02-linter-component-name-history.md) binds
two actual fresh-process complete public observations per input to source-built
[Actions capture](https://github.com/ubugeeei-prod/vize/actions/runs/36967214570),
its original artifact, build receipt and authenticated lossless transport.
The eight immutable full output bodies enter the existing shared runner beside
all 36 unchanged inputs/oracles. Current explicit English/preset/help options
remain distinct from the original count-only test helper. Final-head and queue
comparison are still required; enumeration counters stay unchanged. All-locale
coverage, snapshot deltas, whole-commit/history closure and native proof remain
open. Native handled/equivalent/paired comparisons remain zero.
