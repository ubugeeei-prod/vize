# Complete same-page rule category examples

Issue: [#8334](https://github.com/ubugeeei-prod/vize/issues/8334)

## Decision

The original remaining-catalogue slice was sealed as
`18baa158a260b8767d6083716bc40b23246a9f86` on Vue parent
`a40a7abdd3c37a8d292a4b687f81b4108ef87925`. The local refresh now genuinely
descends from actual main `26031a4fbb30a1e86511919bafc7035b08440ef3`, through
Vue parent `f10f75f80ed83fadd380a29eb2c6e176c4e8e570`, preserving both original
sealed worktrees. Third-party issues #8328 and #8329 retain publication priority.
This preparation does not publish or close an issue.

Each existing category page now includes the rule's purpose, prerequisites,
configuration, support limits, and complete Bad/Good examples. All five locales
cover the current 251 source rules and 66 project packets. Existing English
subgroup pages remain complete reader pages. The three previously absent foreign
Petite Vue category routes are added; foreign individual reference routes are
not invented. Prerequisites use existing translated routes where available and
existing English routes otherwise.

Use the existing build-time reference composition and content materialization.
Read the authoritative references once and cache each localized packet within
the generation pass. Retain original source blocks, shared project files,
reactive graphs, and all existing rule routes and fragments. Match project
Bad/Good blocks by their actual file paths before computing native Ox Content
`annotate="remove:..."` / `annotate="add:..."` fence metadata. Shared files are
unchanged and copyable source contains no decorative diff prefixes. Native
headings own rule fragments; explicit aliases preserve older category fragments.

No additional runtime requests, generation stages, dependency upgrade, custom
renderer, or handwritten `.mjs` are introduced. There is no new UI component in
this slice; the existing Vue SFC theme renders the native output.

Project pages state their real support boundary: 60 analyzer codes comprise 19
CLI codes (18 qualified source pairs and one illustrative project with its
retained reactive graph), 16 experimental Rust analyzer codes not individually
emitted by that CLI pass, and 25 contracts with no current producer. Six further
packets are project-specific lint IDs. All five cross-file overview tables keep
those statuses and the public CLI/configuration instructions. Configuring a
diagnostic ID does not activate an unavailable producer.

## Local evidence

- Deterministic generation and the five existing focused docs suites pass:
  30 tests, no failures. Strict type-aware lint passes for the 51 changed/new
  JavaScript and TypeScript modules.
- The installed `@ox-content/napi@2.81.0` renderer plus HappyDOM validates all
  66 materialized reader pages: 3,281 complete packets and 14,974 code blocks,
  including shared files and retained graph blocks. Actual native IDs are unique,
  existing fragments resolve, links stay on the current page, and rendered copied
  source matches the independent references byte for byte.
- Local rendered HTML gzip size peaks at 119,440 bytes for the Japanese all-rules
  page. Native line annotations increase the existing English/Japanese all-page
  gzip sizes from 98,697/108,466 to 109,035/119,440 bytes. These are local payload
  measurements, not browser latency or a performance benchmark.

The genuine main26031 refresh passes the same 30 focused tests and deterministic
generation, plus strict type-aware lint for all 66 changed/new modules across
both slices. Every one of the 66 native page source/HTML hashes, copied code
blocks, retained fragments and payload sizes equals the original sealed child.
The initial refresh needed no additional executable docs change.

## Hosted type-check correction

The original #8355 source job
[114104968371](https://github.com/ubugeeei-prod/vize/actions/runs/38015576041/job/114104968371)
reports four TS7053 errors before repository checks: foreign dictionary
aggregates infer a union of literal-key objects and indexed records. Give the
shared locale aggregate an explicit catalogue translation contract, retaining
checked dynamic rule/support lookups and fail-closed missing entries. This adds
erasable types only; all translation text, examples and native page outputs stay
unchanged. The original failed run remains evidence. Fresh successor exact-head
Actions are required before any protected admission.

The successor passes the actual local TypeScript 7.0.2 docs project check,
strict type-aware lint, deterministic generation and all 30 focused tests.
All 66 native page source/HTML hashes and payload sizes remain exact; hosted
successor qualification is still pending.

## Remaining acceptance

- After successful third-party publication, check fresh `main` and retain the
  genuine Vue parent relationship when publishing the native Stack. Regenerate
  again if the authoritative rule registry has changed.
- Pair this decision with a concise issue comment during root integration. The
  local child appends its link to canonical row 350; all incoming clauses,
  including current v12 release/retirement row 330, remain exact. No remote issue
  comment has been made.
- Run exact-head Actions, the full docs SSG build, and the expanded real Chromium
  navigation/render checks. Verify deployed category and cross-file pages,
  fragment links, copyable source and page payloads in all five locales.
- Keep #8334 open until its full original acceptance is met, including successful
  merge and deployed verification. Local native rendering does not establish
  Chromium, SSG, Actions, deployment, release or publication success.

Proposed paired issue comment after the publication gate:

> Keep complete Bad/Good packets on every existing category and cross-file page
> in all five locales, using native Ox Content line annotations and unchanged
> copyable source. Preserve existing routes, fragments and explicit producer
> limits. The local decision and pending hosted acceptance are recorded in
> `docs/davinci/decisions/2026-10-10-rule-category-examples.md`.
