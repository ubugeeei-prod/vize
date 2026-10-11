# Registered reserved-key suppression (#8090)

Status: source change and independently authored fixtures; candidate Actions,
protected queue, actual merge and released acceptance remain pending.

## Problem and bounded decision

The immutable Vue Select `src/components/Select.vue` input contains
`// eslint-disable-next-line vue/no-reserved-keys` before `_value`. The pinned
official ESLint packet suppresses this finding, while the existing Patina rule
still reports it. The prefix/group correction deliberately preserved this one
finding and named the separate suppression gap.

Normalize only the registered `vue/no-reserved-keys` identity to
`script/no-reserved-keys` in the existing directive rule-list parser. Existing
`vize/` prefix normalization runs first. The existing comment lexer, per-block
ownership, directive event ranges, rule dispatch, AST and suppression stage stay
unchanged. No additional pipeline stage, parse or all-rule pass is introduced.
Foreign rule names, case-sensitive identities and markers inside strings retain
their existing behavior. Disable and enable use the same registered identity.

The public rule contract names this exact suppression support. This does not
provide arbitrary ESLint alias compatibility, ESLint unused-directive warnings,
`asyncData`, or `<script setup>` `defineProps` reserved-key producer coverage.
Those limitations and broader #8090/Matrix work remain unfinished.

## Independent whole oracles

`tests/_fixtures/differential/lint/reserved-suppression-8090/` contains 24
source-authored complete diagnostic vectors, frozen before observing any
candidate binary. The controls cover canonical/plugin/registered/foreign rule
identities, next-line and same-line ranges, disable/enable regions, mixed rule
lists and reasons, unqualified disable, Oxlint syntax, string/template-literal
markers, distinct normal/setup/style blocks, UTF-8/CRLF and a positive `$emit`.
The whole target vectors contain 22 diagnostics before and 12 after. The
canonical-disable/alias-enable control corrects an existing missing diagnostic
(zero before, one after), alongside the intended suppression corrections.

The authenticated v0.443.0 binary matched all 24 complete before packets,
including raw stdout, status, stderr, error, signal and input hashes. Its SHA-256
is `9ad7cf956351265a152a9bbfa08b8c50c85dadacef66529474bbb6170267e254`,
Release run `38081199501`, source `15e116fa258c7aaba5c6fdd2456442535605abe8`.
The first 22 cases and complete packets/helper are also preserved unchanged in
`initial-twenty-two-before.json.gz`; the two cross-identity enable controls were
subsequently authored and added before their own public observation.

`original-suppression-oracle.json` preserves the entire immutable Vue Select
provider and public-before packets from the prefix corpus. The pinned official
ESLint 9.39.1 / eslint-plugin-vue 10.9.2 / vue-eslint-parser 10.4.1 result has
**two unused-directive warnings and fixes plus one suppressed message**. All
fields remain exact; it is not an empty provider result. The 35,649 source bytes
have SHA-256 `12f3b27bb0d85e07ab538f2813e7f3ddaaf8708ed4167b95c50a366f4cfd9b68`.
The additional 24 controls have no claimed official-provider execution.

The current original-source successor explicitly changes only Vue Select's one
remaining target finding to zero; all other 20 original inputs/complete vectors
stay exact. The old prefix corpus, its one-finding oracle, complete Matrix
divergences, public packets and official suppression/warning packets are not
rewritten. Existing parent/core API order and diagnostic metadata are preserved.

## Qualification and delivery

The Rust API and configured CLI observers require 24 cases through seven routes
(default, recommended, essential, ecosystem, incremental, explicit rule off and
the unchanged pinned n8n projection), plus the whole original Vue Select input:
**169 packets**. Current native execution must retain the checkout build receipt,
source/input/config custody and whole results. The inherited bare 113 and prefix
161 packet suites remain mandatory with their explicitly declared successor
oracles; no route, full vector, Matrix budget or instruction ceiling is removed.

The source-only authority check validates complete public-before packets, the
initial 22-case archive, original provider warnings/suppression, every pinned
source and the single original successor. It is not candidate execution credit.
The source parent and previous full directive parser are retained in the
authority manifest for review.

Deliver as a native Stack descendant of the prefix correction. A ready parent
prefix enters the protected queue independently of this unfinished descendant.
After actual parent merge, retarget/rebase onto fresh main and requalify the
changed head. Released acceptance requires a successful official release that
actually includes the merge and a separately declared installed corpus. The
failed 0.446 preflight and the older 392-packet installed wrapper confer no
qualification on this source slice.
