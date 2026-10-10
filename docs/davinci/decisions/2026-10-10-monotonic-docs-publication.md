# Monotonic docs publication

Status: source proposal; review, exact-source Actions, protected merge and actual
Pages/browser delivery remain pending.

## Observed delivery failure

The Docs build already has a SHA-specific concurrency group. Healthy complete
builds take about forty minutes because their browser proof captures the complete
catalogues. The deployment workflow compares the originating source with the
current `main` tip, so unrelated merges make completed builds ineligible.

Publisher run `38034771162` reports success at source
`cb9afd9298bc74a852b0ea6a14e45d5b723db054`, but its actual Pages step is skipped.
GitHub still creates a successful generic `github-pages` environment deployment
(`6977792459`). That status cannot establish the published source. Public HTTP
responses can also be cached, so they supply browser acceptance after delivery,
not the authoritative rollback fence.

The initially observed actual Pages step succeeded in publisher run `38013719817`,
attempt 1, job `114099286630`, from Docs run `38010819888`, attempt 1, source
`26031a4fbb30a1e86511919bafc7035b08440ef3`. Its primary artifact metadata and
original publisher log identify the source archives and Pages archive
`11655422443`. The original workflow bytes have SHA-256
`a2ba0528b06e317a8740b1c324a84a3de3c75c456a983d12d559f54b14e0e100`.
The first complete primary rehearsal discovered a later actual publication at
`042f24370b08031f0e281d0b7b5530ae8265b5e7`: publisher `38031579779`, attempt 1,
job `114153455581`, original Docs run `38029167488`, actual Pages archive
`11661699668`. Both original native archive manifests and whole digests agree.
This historical draft rehearsal made 135 read requests in 12m06s locally; it does
not qualify the corrected source or supply deployed feature acceptance.
The journal collector's second read-only rehearsal reached the same primary
publication in 107 requests and 470.658 seconds, with no outstanding publisher
attempts or environment IDs. This measures initial collection, not authenticated
steady-state publication: that requires the first actual immutable custody
receipt. The source controls separately prove that a complete newer receipt
does not request obsolete legacy metadata or re-download historical archives.

## Publication conditions

Acquire the existing `pages-main` job lock before fetching trusted current-main
validator code. Retain `queue: max`, `cancel-in-progress: false`, the original
successful Docs trigger, and every existing build, Pages and release gate.

The primary GitHub API must prove all of the following:

- The original Docs run and attempt are complete and successful, belong to this
  repository, use `build-docs.yml`, and originate on `main` through the existing
  push, schedule or manual triggers.
- Its whole source SHA is the same commit or an ancestor of actual protected
  `main`. A detached source, another branch or a fork is refused.
- Exactly one available archive exists for each original `docs`,
  `docs-render-evidence`, `playground` and `musea-examples` artifact. IDs, complete
  digests, originating run/repository/source and attempt timestamps agree.
- The native `_og/manifest.json` inside the original Docs archive names that
  whole source and preserves its complete bytes and asset fingerprint. The
  original archive's SHA-256 must match primary artifact metadata.
- The incoming source is newer than the latest authoritative actual Pages
  publication in the same protected lineage. Equal or older builds skip cleanly.

Download only the verified archive IDs. Copy the trusted-main validators into the
runner's temporary directory before checking out an older eligible source; an
older checkout cannot remove or replace the validator entrypoints.

## Durable source custody

After constructing the original site and uploading its immutable Pages artifact,
compare both native manifest bytes and the actual uploaded archive digest with
the original Docs build. Create a repository Deployment with task
`vize-docs-pages-v1`, separate environment `vize-docs-source-custody`, and immutable
payload containing the complete originating run/attempt/source, four archive
identities, native manifest identity, actual publisher run/attempt/job/context
commit and Pages archive identity.
It also retains the primary validation step's actual start timestamp and exact
older outstanding publisher attempts and Pages environment IDs.

Creation uses the exact publisher commit, `auto_merge: false` and an empty required
context list; it does not change a Git ref. Before the actual Pages step the
custody status is only in progress, with `auto_inactive: false`. Log its exact
Deployment ID and canonical whole-payload hash in the real custody step.

The pinned Pages action and its original artifact name remain unchanged. Its
context commit can differ from the original built source, so both identities are
retained. Only the actual successful `Deploy to GitHub Pages` step can qualify
the candidate receipt. The final custody status becomes successful afterwards.

A later cleanup or status-write failure cannot erase a real Pages effect: the
next publisher independently validates the actual primary step, exact bot-owned
immutable receipt and its original job-log hash. Generic environment success,
skipped steps, unexecuted candidate receipts and a cached public manifest never
become publication floors. An uncertain later Pages effect is refused rather
than permitting an older publication.

## Bootstrap and retained authority

Select the newest fully authenticated immutable receipt and actual successful
Pages step before requesting any legacy metadata. Once that floor exists, missing
obsolete migration metadata cannot block delivery. Inspect new publisher and
environment records since that receipt's primary validation start, stopping
pagination at the cutoff, and explicitly recheck every older outstanding ID.
Creation time alone cannot retire a writer queued before the anchor: its Pages
effect can happen later. Resolve generic records to their primary jobs and refuse
unqualified newer or uncertain effects and unknown writers.
Also query all five active workflow statuses without a creation cutoff: a rerun
keeps its old run creation time. Authenticate and retain each discovered current
attempt, without replaying its obsolete completed attempts. Primary count and
pagination completeness requirements also apply to these bounded status queries.

Before allowing publication, authenticate every pending writer's exact workflow
hash. The preserved legacy writer locks the whole job and checks fresh main
after acquiring that lock; the current guarded writer locks the whole job and
checks authenticated actual publication custody. Both may remain queued without
blocking delivery. A pending unknown environment without primary job identity,
or a publisher whose exact lock and guard cannot be proved, refuses publication
on every floor. Retaining a journal alone cannot prevent an unguarded later write.

Only a real skipped Pages step or a primary terminal effect superseded by a later
authenticated actual publication retires an outstanding writer. Generic status,
age and overall cleanup conclusion cannot discard it. The next immutable receipt
preserves this complete outstanding journal with its own actual primary start,
so steady reads cover new work and unresolved writers without retaining a
permanent full-history dependency.
The same skipped or unstarted effect filter applies to candidate custody records;
an unexecuted newer candidate retains the older floor and still proves its guard
when pending. Terminal metadata in the current writer's final custody step may
refresh for at most ten seconds before applying the unchanged strict assertion.
Canonical receipt hashes use code-unit key ordering, independent of runner locale.

Workflow searches include the primary `total_count` on every page. GitHub limits
[filtered workflow-run searches](https://docs.github.com/en/rest/actions/workflow-runs?apiVersion=2022-11-28#list-workflow-runs-for-a-workflow)
to 1000 results. A larger, missing or changing count, or a collected result count
that does not equal it, refuses publication; a short page cannot silently retire
omitted pending writers. Artifact and job pagination retain their existing scope.

A receipt-less historical bootstrap is allowed only when no authenticated current
receipt exists, and only for the exact original workflow
hash above. Its original log must bind the source run and every artifact ID and
digest; the native Docs and actual Pages archives must retain identical manifest
bytes. Unknown replacement workflows without their immutable custody are
refused. A configured site whose actual publication cannot be proved is refused.
Only a truly unconfigured Pages API response permits first-publication bootstrap.
That absence still audits prior and pending publisher attempts and Pages
environment writers and proves their guards, without requiring a nonexistent
legacy floor. Queued attempts without environment records are included. Unknown pending
writers and uncertain actual effects refuse this first publication as well.

Historical artifact availability can expire without changing identity. Custody
qualification still requires retained primary artifact IDs/digests/source/run
metadata, the exact immutable Deployment payload and the real publisher log and
step. Missing primary metadata or logs blocks new publication; their absence
does not authorize discarding the floor. This initial migration scans publisher
history and downloads the two original archives, so its API and archive cost
must be measured in the read-only rehearsal and exact Actions before admission.

For operations, enable GitHub failure notifications for `Deploy docs` and triage
failed `Whole immutable receipt` or publisher-log assertions against the exact
floor Deployment, publisher job and source artifact IDs. Before a long publication
gap, increase repository Actions artifact/log retention and the original Docs
artifact upload retention to cover it. Retention changes cannot recover already
expired data. If primary log or identity metadata is already unavailable, retain
the floor, report those exact IDs in an issue, and require a maintainer-reviewed
recovery that restores equivalent primary custody before publishing. Never delete
the floor or promote generic environment success to bypass the assertion.

## Required controls and delivery

The source tests exercise a real Git DAG with a completed build that remains an
ancestor after `main` advances, older and equal candidates, divergent sources,
foreign branches/forks, wrong source/run/attempt, incomplete or forged artifacts,
whole native ZIP/tar manifest custody, absent or forged receipts, unknown receipt
writers, skipped Pages, first bootstrap, configured unprovable sites and cleanup
failure after actual successful Pages. Journal controls cover a missing legacy
anchor beside a complete newer receipt, older queued publishers and unknown
environments crossing the cutoff, new unknown writers, late uncertain effects,
actual skip retirement and refusal when current custody metadata disappears.
Primary-page controls also reject a truncated 1001-run workflow search and an
incomplete smaller search, while accepting complete multipage and empty results.
Review regressions reproduce an older unretained rerun in every active status,
unknown guards without a Pages environment, newer skipped or unstarted candidate
custody, locale-dependent JSON key ordering and stale terminal metadata. Original
guard, count completeness and uncertain-effect controls remain in place.

The existing strict erasable TypeScript CI gate includes every new entrypoint
and control. Existing whole Docs build and release-preflight obligations stay
unchanged. After actual merge, require a genuine successful Pages step bound to
the originating successful main Docs run and receipt, then replay the affected
public browser routes against that exact deployed manifest source and asset
fingerprint. Source preparation and generic workflow success are not delivery.
