# Restore an accidentally advanced immutable source receipt

An immutable release source PR must keep its original source head while ordinary
PRs continue merging into main. Updating that source branch from main invalidates
the receipt; successful checks on the newer branch do not qualify the frozen
release. The source selector now checks the pinned markers, own repository,
recorded head, draft state and main base before ordinary source qualification.
The official release driver still authenticates the complete pin, source parent,
metadata delta, required checks, catalog, artifacts and publication identity.

## Recovery observed for v0.457.0

Source PR [#9196](https://github.com/ubugeeei-prod/vize/pull/9196) records:

- Cut C: `014bd42cacdf3d026fb75162cfa80dce76d78315`.
- Source H: `03882089eff75664abe99ba71868e33739496b01`.
- Durable pin: `b5ec1bf30d8da37c50c4100804b5b6f33c18ead7`, with H as its parent
  and the identical source tree.
- Annotated tag object: `46536212e8e1a16492f9e196d84908286170d57b`, targeting H
  and identifying Release run `38805261385` and metadata integration #9197.

The source branch had advanced to `008cc10364c09a57c1fd56acbac1d72761fbdc8d`
by merging main. After confirming there was no running release operator, that
complete commit was preserved on
`archive/release-v0.457.0-drift-008cc103`. Only the source branch was restored to
H, with a force-with-lease naming the exact accidental head. GitHub's PR head
subsequently matched H. The tag, pin, source body, integration and 42 unexpired
Release artifacts were preserved.

This restores the original receipt; it does not authorize another source or
substitute newer checks. Before any such recovery, confirm the current branch,
PR, pin and tag identities and coordinate with any active writer. Preserve an
unexpected concurrent change instead of retrying an unconditional force push.

## Remaining publication gate

The latest observed Release attempt failed only the Deno JSR consumer: Deno's
24-hour dependency-age policy refused newly published `@vizejs/vize@0.457.0`.
The registry reports creation at `2026-10-15T21:32:59.134270Z`; the frozen
consumer becomes eligible after `2026-10-16T21:32:59.134270Z` (October 17,
06:32:59 JST). Re-run that failed job on the original Release run after the
threshold, then resume the official operator with `--resume 9196 --pin`.
Successful public availability alone does not close the source receipt: retain
it until the original release and external publication verification succeed.
The next minor must wait for that source receipt's verified closure.

The urgent Vapor HMR #9180 and workspace-server resolution #9201 fixes were
merged after this cut. They require the next minor and fresh installed-package
replays; this recovery grants neither fix public acceptance.
