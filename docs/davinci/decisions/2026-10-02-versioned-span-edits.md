# Versioned L1 span edits

Issue: [#6876](https://github.com/ubugeeei-prod/vize/issues/6876).

The existing embedded source retains authored offsets and decoded bytes, but
neither its decode map nor its text identifies the complete preparation input.
Checking equal decoded text or equal numeric spans would admit unrelated input.
`EmbedSource` now retains a private borrowed `authored_root`, populated only
from the actual preparation input. Native attribute decoding, authored sources,
trimmed Vue interpolation and checked slices retain that same reference.
The readonly accessor performs no parse, decode, traversal or allocation.

This is physical buffer provenance: compare the original pointer and length.
It provides no semantic document, language, grammar, File or producer authority.
Documents can share borrowed bytes, and empty buffers need not have distinct
pointers. A real caller must retain its own document identity and version.
Maestro already exposes URI, client version and a changing document revision;
its separately obtained `text()` values are distinct owned snapshots. A future
adapter must capture those host identities with its retained parsed source,
and must not substitute an independently copied equal-text buffer.

| Retained 64-bit payload |   Before |    After |                     Change |
| ----------------------- | -------: | -------: | -------------------------: |
| `EmbedSource`           | 40 bytes | 56 bytes | one 16-byte borrowed `str` |

The source remains Copy and has no Drop. The reviewed production storage row
stays exactly one L0 String import/two bound uses and one arena Vec import/two
bound uses; the getter adds no owned storage. The new origin laws have their
own reviewed test-only L0 String row for distinct live allocated buffers.
Plain preparation and identity slicing still use zero arena bytes in the law.
That observation does not establish unchanged instructions or whole-process
allocation performance, especially for arrays of the larger retained payload.

Local validation compiles the actual complete L1 library under strict Clippy
against the retained official dependency cache and runs five laws against that
real library: separate equal allocated roots, Unicode/entity exact boundaries,
identity/entity slices, trimmed interpolation, shared/empty limitations and
the payload size/no-Drop/plain-allocation observation. It is a scoped cache
proof, not a fresh whole-Cargo or hosted acceptance result.

`vize_l1::edit` now provides `VersionedSource`, `SpanEdit` and `EditSet`.
The host supplies its genuine snapshot key, client version and `SourceRoot`;
the key must distinguish both document and revision when versions reset or
bytes are shared. A borrowed actual URI plus actual host revision is one
possible carrier. The provider creates no global brand, URI or identity hash.
It cannot query current host state; adapters must capture genuine identities
and serialize validation/application with their ordinary document updates.

Checked construction rejects reversed/out-of-bounds/non-UTF-8 authored spans.
Projection consumes the retained native `EmbedSource` and its existing exact
`authored_span` map, after original pointer/length validation. It never uses
diagnostic covering, compares decoded-text equality, or parses/decodes again.
The immutable edit set borrows source-ordered checked edits and replacements;
mixed snapshots, stale versions, foreign buffers, unsorted ranges, interior
overlaps and duplicate insertion points are explicit errors. Adjacent ranges
and boundary insertions follow supplied order; insertion at a replacement's
start must precede it, while later insertion can use its end. Construction validates once
and records checked output length; application rechecks the current snapshot,
reserves one L0 String and emits changed/unchanged authored bytes in one pass.
Every error leaves the original source untouched. Private fields prevent
unchecked construction or changing a validated span, owner or replacement.

Eight additional laws run against the real complete L1 artifact, including a
single native Expr parse whose retained OXC right-hand span selects `&fjlig;`
in an actual authored directive. Replacing it produces the exact expected
authored output without reparsing. Other laws cover valid Unicode/application,
stale versions, reopened/foreign keys, equal separately allocated buffers,
overlaps/order, entity-interior refusal despite valid diagnostic coverage,
identity slices, interpolation and shared/empty document keys. Six deliberately
broken provider guards are caught by these actual laws. Five independent
external compilations reject private field mutation/forgery. Both proof sets
retain their actual source/dependency identities and scope; they provide no
product or hosted performance credit. Full per-file storage-policy laws pass;
the new production row reviews one L0 String import/two bound uses for output,
with no alloc String/Vec, arena storage, refcounts or new dependencies.

Complete diagnostic/fix/code-action consumers remain unfinished. The current
Patina fixes and Maestro actions lack this retained native source carrier;
their default behavior is unchanged. Product integration must retain actual
host snapshot identities and full native diagnostic/fix mappings, close each
product's fix-history gate and reuse the shared provider rather than inventing
a second authority or legacy-backed native path. Fresh exact-head Actions,
all 100 unchanged instruction ceilings, full native observations and protected
merge are required before acceptance. #6876 remains open for those consumers.

The first published origin head (`237aeb7`, PR #7434) failed its full source
inventory check: the committed L1 Span grep count remained 310, while the
actual source generator reported 327. Regenerating the authoritative Croquis
inventory changes only that L1 shard. The production source, scanner, gates
and budgets remain unchanged. This correction needs fresh exact-head required,
full and instruction Actions; the failed original run is retained as evidence.

The first published checked-edit head (`4341acd`, PR #7435) had the same
source-inventory omission: its actual complete L1 Span grep count is 375.
The child records that actual count after replaying only its reviewed source
and documentation over the corrected origin layer. Both failed original full
runs remain preserved, and both new Stack heads require fresh acceptance.

Actual merged Stock/Shared and formatter main 2118713 is the new replay
base. The original 6d1e91c and 81382c1 source checks and all 100 unchanged
instruction ceilings passed. Their genuine ordinary workers ran five origin
and thirteen combined laws, with eight original same-attempt artifact ZIPs,
complete JUnit, service hashes and every entry CRC preserved. These are
historical observations and do not grant acceptance to the replayed heads.
The first cumulative queue entries were unmergeable before any candidate test
run; both were removed immediately. Actual three-way source comparisons
located conflicts in the central record and the L1 Croquis shard, while the
selected library root, embedded source and storage rows merged cleanly.
This replay preserves every incoming parser-authority and formatter change,
regenerates only the actual affected L1 inventory and keeps all source tests,
performance ceilings and product-history gates. Fresh required/full/instruction
Actions, named runtime laws and the protected queue are still pending.
