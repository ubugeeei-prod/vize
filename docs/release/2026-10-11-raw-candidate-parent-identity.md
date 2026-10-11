# Release candidate object identity

Refs [#6239](https://github.com/ubugeeei-prod/vize/issues/6239).

The metadata gate for v0.443.0 stopped in
[run 38081833766](https://github.com/ubugeeei-prod/vize/actions/runs/38081833766)
with `Candidate parents do not bind the exact PR merge or own queue base.`
Its log does not identify which of the two parent checks failed or record the
observed parent vector. The checked-out event commit and subsequently fetched
GitHub merge commit have the expected raw parents and complete tree in the
retained local evidence. The original worker's graph state and failing call
remain unknown; these observations do not establish its cause.

The gate previously read `git rev-list --parents`. That command reports the
traversal graph, which shallow boundaries, grafts and replacement refs can
change. Authored temporary Git repositories demonstrate both false refusals
of authentic parents and acceptance of virtual parents absent from the actual
commit. A replacement ref can also hide a changed refresh tree from
`rev-parse <sha>^{tree}`.

The gate now reads the exact commit's headers with
`git --no-replace-objects cat-file commit`. An ordinary PR must still have
exactly the ordered parents `[base, integration head]`; its own merge queue
candidate must still have exactly the sole parent `[base]`. A regenerated
PR merge is accepted only for a PR event with the same raw parent vector and
complete raw tree. The queue never uses this refresh allowance. Existing
event, body, source pin, metadata delta and complete catalog checks remain.

Refusals retain the candidate, observed and expected identities and distinguish
the event check from the current PR refresh. The driver source pin advances
with its matching cache-law literal so rust-script does not reuse an executable
compiled from older nested modules.

Eight real Git controls cover shallow, graft and replacement behavior, malicious
virtual-parent substitutions, changed refresh trees, strict parent count/order
and queue shape, and diagnostic custody. All 57 existing laws remain, including
the unchanged 281-line candidate test module. Before the correction, seven new
controls fail; after it, all 65 laws pass. The intermediate cache-law literal
failure is retained separately.

The v0.443.0 shipping source, cut, pin and Release run remain immutable. This
source correction must pass ordinary Actions and actually merge before its
integration PR incorporates the generated version delta onto that genuine
main. Fresh metadata qualification and all release/publication gates remain
required. This decision records neither publication nor installed acceptance.
