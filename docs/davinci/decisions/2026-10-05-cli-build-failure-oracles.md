# CLI build failure oracle custody

Issue: [#7879](https://github.com/ubugeeei-prod/vize/issues/7879).
Release tracking: [#6239](https://github.com/ubugeeei-prod/vize/issues/6239).
Date: 2026-10-05.
Paired decisions: [compiler issue](https://github.com/ubugeeei-prod/vize/issues/7879#issuecomment-5989030962) and [release issue](https://github.com/ubugeeei-prod/vize/issues/6239#issuecomment-5989032122).

## Original release failure

The actual merged #7958 rejects returned SFC compiler errors before a build
counts success or emits a default artifact. The public SFC API still returns
its original partial code and complete errors. Manual Check
`37268136202` at `6f56261d79d2587c128f7b16aa27ebbac00a0a6d` failed the two
original create-vue and Vue Router compiler patch oracles because their CLI
contracts still required exit 0 and a partial JSON artifact.

The first raw Vue-parity failure is retained at
`/tmp/vize-release-6f-first-vue-parity-failure/job-111629203402.log`:
211670 bytes, SHA-256
`545981e131de50e325aa3b8df865e54898587165a6554b08993a72b0e7c9f7b2`.
The release run `37267966250` was cancelled; v0.433.0 was not published.

## Decision

Retain both original pinned projects, authored sources, patch anchors, full
diagnostic messages and locations, code checks, source modes, repeated runs,
repair controls, and all seven original compiler output fields. Observe those
fields through a small test-only Rust example calling the exact public SFC
entrypoint and default DOM options used by the original CLI. The original
basename, relative script source ID, field order, two-space pretty JSON and
absence of a trailing newline remain intact. Vue Router's original broken
output and code SHA-256 values remain unchanged:

- output: `0914134eb2d13c322402dcf94b608ac904f9ff2ed021c69cd414d5eed84f5654`;
- code: `eed24c03d97029fca5d3d13f81ac416099d3876e7f36572d113acc0768872679`.

Assert the default CLI separately: exit 1, empty stdout, no output directory
or artifact, and the complete original stderr, including ANSI bytes and every
diagnostic. The variable elapsed value must match the existing four-decimal
grammar and is inserted verbatim into the otherwise fixed whole expectation.
No diagnostic text, path, range or code is normalized or filtered.

Build the example in the same existing CLI build steps for Vue parity, PR/queue
tooling and full Check release scripts. All three tooling entrypoints retain
their source-build receipt; the full Check bootstrap also hydrates Vue Router. A thin tooling entry imports both complete original oracle files,
validates the existing source-build receipt, and binds both CLI and LSP to that
exact built executable. The existing tooling fixture hydration additionally
fetches the pinned Vue Router gitlink. Every actual API request retains its
whole authored input and stdout/stderr/status/signal, current source revision
and actual example executable SHA-256. Failed CLI observations also retain
whole stdio and artifact state. These passive records live under
`target/differential/cli-build-error-oracles/` in the existing automatically
uploaded differential/raw-protocol artifact; fresh CI build logs remain
required authority for the example, and a CLI receipt alone is insufficient. The normal automatic four-shard source
planner runs these laws on PRs and in the protected queue; full scheduled/manual
Vue parity keeps its original fixture and cycle controls.

This changes test authority only. Product code, APIs, native stage claims,
legacy corpus, output goldens, instruction ceilings and release gates are
unchanged. There is no local native build, install or manual campaign.

## Required follow-through

Fresh exact-source Actions must execute both whole compiler patch laws and
their retained check/lint/format/LSP controls. Source review, protected full
suites and all 104 instruction gates precede actual signed merge. Release
publication and installed-artifact proof remain owned by the separate release
lane; this repair does not accept the failed or cancelled candidate.
