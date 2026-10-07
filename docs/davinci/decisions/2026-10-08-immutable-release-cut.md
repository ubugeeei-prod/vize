# Immutable release cuts during ordinary merges

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830), paired with
the [coordinator decision](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6044521219).

The existing release protocol requires the source version commit to be one
commit on the current main head. Every ordinary main advance cancels the
unpromoted workflow and rebuilds that commit. Continuous healthy merges can
therefore prevent a release from completing even when each source cut is valid.

Add an opt-in `--pin` protocol while retaining the existing default. The release
command fixes the original source base C, generated version commit H and its
distinct `Pinned Release` workflow R. Every existing exact-H required check,
all five full Check, Fuzz, Miri, Matrix and Docs gates, platform build, install
smoke and artifact remains required. Parent C supplies no full-gate credit.
Ordinary merges never grant H credit and never replace its source or artifacts.

The command prepares a separate version-only integration PR I from current
main. The coordinator admits I through the ordinary protected merge queue;
the release command never admits it or writes main. Its actual signed merge V
must satisfy the unchanged current required checks and instruction ceilings.
I may complete source Actions in parallel with H, but its queue admission waits
for H/R's full qualification and H's own required checks to succeed. This keeps
a failed source cut from first advancing main's version. Ordinary source PRs
continue while those proofs run.
The verifier derives the version update from V's actual first parent P, which
can differ from I's original base after preceding queue entries merge. The
complete generated tree must equal V, preserving every non-version byte of P.
C must be on V's and current main's first-parent history. The workspace,
package versions and published package catalog must agree between H, V and
current main, including complete shipment manifests, entry points, dependency
requirements, features and targets. Missing, stale, ambiguous or changed
custody fails closed.

Only after both the original H qualification and the actual integration proof
pass does the command create the immutable annotated tag at H. This operation
pushes the tag alone and never moves main. Its receipt binds C, H, I, V and R;
resume authenticates an existing receipt before treating a tag as success.
The hosted waiter then publishes the artifacts already built at H. No old
candidate artifacts, substitute commits, gate waivers or duplicate publication
are authorized by this protocol.

A cooperative local and remote operator lease prevents competing operators.
The durable pin marker revokes legacy refresh and promotion authority. During
an existing release's mode switch, the coordinator first records and stops its
owned old CLI process after authenticating its source identity, ownership and
absence of a tag. The new source implementation then acquires the lease and
installs the marker. Only the official command may retire an authenticated,
still-active legacy R for that unchanged source PR/H, repository, branch and
workflow. It never cancels a pinned R, a full-H gate or another source's run.
Global release workflow concurrency remains unchanged.
The marker transaction checks remote source H immediately before and after
publication; an unchanged H-to-H refspec is not a server compare-and-swap.
A concurrent source change leaves the marker for inspection and stops body
updates, legacy retirement and promotion without rewinding the source branch.

The original source PR remains draft and is an immutable source record. It is
closed after terminal successful publication; it is not represented as merged.
The actual integration PR supplies the merge receipt. Same-version main
descendants with the same package catalog may continue while registries,
GitHub assets, Marketplace and fresh eligible Pages deployment are verified.
The next version promotion waits for that external verification to complete.

The existing manual npm bootstrap recovery contract remains unchanged and
rejects a pinned tag outside main's first-parent history. A future receipt-aware
bootstrap change requires its own reviewed verification; no fallback weakens
that exceptional recovery gate. Normal existing-package publication is intact.

Validation must cover ordinary main advances, actual queue-parent changes,
unrelated source edits disguised as version updates, catalog drift, missing
or red checks, invalid signatures, concurrent version changes, tag races,
interrupted pushes and repeated resume. Source implementation and local
controls alone do not constitute publication or external consumer proof.
