# Compatible stable patch cuts

The user's 2026-10-12 direction permits each compatible fix to justify a patch
release and supersedes the earlier policy that every 0.x release is a minor.
Breaking public API changes, including the approved nominal CSS migration,
remain on the next minor. A patch does not remove any qualification or channel.

## First concrete selection

The fresh GitHub read identifies the last published release as v0.443.0,
Release 409241946, published 2026-10-10T22:24:22Z. Its annotated tag object is
fd0d797b6a006489e96617768e47f8d744c5a544 and its immutable source H is
15e116fa258c7aaba5c6fdd2456442535605abe8; the original successful Release run is
38081199501 and source receipt is #8541. v0.443.1 and a `release/0.443` branch
returned typed GitHub 404 during this preparation; absence must be rechecked at
actual start. Retired unpublished minors 444–447 do not change that patch line.

The smallest independent urgent fix is #8627, delivered by #8637 as
c47cbd01c203d7ee40c2c5ff7ff3d5a1359a9b79. Its sole production change is the typed
handler predicate in `crates/vize_atelier_vapor/src/generate/operations/events.rs`.
The complete original preimage is byte-identical to that file in published H,
and the TypeScript-aware predicate already exists in published core. This fix
needs none of the later SVG, slot, root, CSS, SDK or config changes. Preserve the
complete original SFC unit/snapshot and typed-handler App, Child, controls and
reporter input corpus when preparing `backport/0.443.1-typed-handlers` from H.
The verified reporter trailer is `Heb <xsh4k3@gmail.com>` (public ID 13395944).

## Existing protocol reuse

The original official operator is main-only and starts a minor; even its local
`patch --pin` command fetches moving main. It cannot select an old stable cut.
The new explicit `patch` operation keeps the same maintainer user, main workflow,
protected environment, cooperative lease, immutable source PR, Release workflow,
artifact builders, publishers and signed main metadata integration.

A reviewed main policy commit A supplies a separate complete
`tools/support/release/stable_patch_plan.json`. Its immutable Git blob and SHA256
are bound in the source receipt, durable pin and annotated tag. The manifest is
outside C, avoiding a self-referential expected commit/tree. It names the exact
public annotated tag/H/tree/R, canonical next patch, ordered original merged fix
SHA/parent/reporter records, ordered real backport parents, exact C/tree, complete
raw diff and each whole changed file/mode. Original fix/corpus bodies must match
actual merged originals. Qualification transport bodies must match reviewed A;
unknown, omitted, duplicate, forged or unscoped changes refuse before install.
No CSS nominal migration, new crate registration or unmerged SDK graph belongs
to this initial product backport.

C remains the sole parent of prepared H. The existing complete C→H generated
version transformation authorizes stable shipping manifests, dependencies,
exports and publication graph. Stable shipping does not borrow moving main's
catalog. Main metadata delivery separately retains its exact actual-parent
version-only transformation, signed one-parent actual merge, main ancestry and
all five protected workflows. Normal minor catalog equality is unchanged.

The root operator may use `operation=patch` with the exact reviewed `source_cut`
only after this implementation and its finite plan actually land. The old
start/resume/retire inputs remain separate; mixed inputs, stale policy, changed
public tag/R, non-patch versions and unknown plans refuse. Source checks, exact-H
full Check/Fuzz/Miri/Docs/enforced Real Project Matrix, every platform build,
installation smoke, semver, final publication and installed reporter replay all
remain required. No historical green is reused as a new source measurement.

## Current status and remaining work

The source prototype passes all 75 inherited release facade laws plus six new
stable laws using real private Git objects and inert API responses. These include
a genuine detached stable chain, full-body/path/parent mutations, changed public
R/tag/fix authority and a real replacement-ref refusal. This is source evidence,
not product, hosted or release qualification.

TODO: finish the complete gate-source preservation review before freezing C;
include only genuinely necessary qualification/transport dependencies, preserving
all existing gate modes and budgets. Commit the full backport C and acyclic
manifest, run ordinary exact-head Actions, obtain independent source review and
carry the conventional implementation PR through protected actual merge. Extend
retirement's retained stable source reconstruction if a failed patch needs that
operation; do not reinterpret old minor reservations or reuse a failed version.
Then root alone starts one official patch. After genuine public H/R/tag/npm
custody, replay the original typed-handler physical consumer law using published
bytes; keep every other external issue's acceptance scope separate and pending.
