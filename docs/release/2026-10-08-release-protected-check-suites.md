# Protected release check suites

Paired issues: [#6239](https://github.com/ubugeeei-prod/vize/issues/6239) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Actual promotion failure

The v0.437.0 immutable source receipt is draft PR #8314:
C=`04ec4c43d9c84c46892fc33f013917d2561bfe78`,
H=`b303befe31132cd559dbe02eb3d45be11f63fbf8`,
pin=`56fb6bdb873ca5a4c1bf373dd4de9f8e576340bd`, and
[Release R=37774663155](https://github.com/ubugeeei-prod/vize/actions/runs/37774663155).
All original exact-H Check, Fuzz, Miri, Docs and Real Project Matrix gates,
release preflight, complete builds and candidate readiness succeeded.

Metadata PR #8315 actually merged at 14:08:39 UTC as signed, valid
G=`9c977e1e38a2ace211a0b4ac7821f9183825fd59`, with sole parent
`81aa0c449cdeb2daf2959d3c9a272ff880bf4765` and tree
`71efaac266cd77e70067766b58f14edc08d9062e`. All five protected workflows
succeeded. Its exact-G catalog reported `catalogEqual=true` and
`metadataExact=true` for the same C/H and integration head
`f54fc98c345b5f00ee4973e2885068e96360c49f`.

The official promotion command then refused:
`Ambiguous required PR check check-js.` One supported same-R resume reproduced
that refusal. Both PR required-check rollups remained uniquely green; the
failure was in the actual-G protected-check producer.

The unmodified commit check-runs API returned two genuine `check-js` rows:

| Event                              | Run         | Check suite  | Check        | Result    |
| ---------------------------------- | ----------- | ------------ | ------------ | --------- |
| Exact protected merge group        | 37786681534 | 102371818744 | 113343114681 | success   |
| Subsequent main push at the same G | 37790055431 | 102381502901 | 113354750642 | cancelled |

The producer authenticated the five protected workflow runs, then mixed all
same-G check suites when evaluating required names. Each configured context
therefore appeared twice. A later push must not replace protected evidence.

## Bounded correction

Keep the latest exact-G merge-group workflow selection, repository, own queue
branch/parent, signed actual delivery, complete catalog and every configured
required-check law. Bind the required check rows to those authenticated
workflow `check_suite_id` values before evaluating the unchanged name, app,
head, link and terminal-success requirements. Missing, foreign or ambiguous
queue suites and missing, duplicate or failed queue checks still refuse.

The regression preserves the whole primary API objects for the six relevant
workflow runs and eight required check rows. It reproduces the old ambiguity
and accepts only the four successful protected rows. Controls retain missing,
failed and duplicated required contexts, foreign apps, wrong heads, missing
or foreign suites, ambiguous suites and failed/pending newer queue runs.
The driver source pin advances to invalidate compiled support-module caches.

## Recovery decision

The hosted promotion verifier also comes from immutable H, so a local-only
fix cannot safely publish the existing R. Retire this unpublished candidate
without changing H, pin, artifacts, historical checks or merged metadata.
Cancel only R, preserve the source and gate evidence, retain H under the own
archive ref `release-archive/v0.437.0-pr8314`, and close the owned draft source
receipt after terminal cancellation and external absence checks. Never create
or delete a tag, delete historical checks, or revert the delivered version.

After this narrow correction actually merges through the protected queue,
start the official next minor, v0.438.0, from genuine fresh main immediately.
Its C/H/R are not assigned in advance. Require its own original exact-H five
gates, native builds, semver/preflight and terminal public publication; the
retired v0.437.0 receipts supply no publication credit. Unrelated product and
migration lanes continue independently.

## Terminal retirement receipt

R attempt 1 is terminal `cancelled`, with terminal update 14:20:27 UTC.
All 21 publication dependents are cancelled with zero executed steps; all
41 build/evidence artifacts are retained. The archive ref points to exact H
and the durable pin is unchanged. Source #8314 is actually closed unmerged,
still draft, with its body, title, base and H unchanged.

No v0.437.0 tag or public GitHub Release exists. All 26 planned npm and 26
crates.io versions returned HTTP 404; Marketplace has no matching version and
Open VSX returned HTTP 404. The latest public version remains v0.435.0.
Metadata G remains delivered on main. Neither retirement nor the source
regression grants publication or issue-completion credit.
