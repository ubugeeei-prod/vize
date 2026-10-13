# Production CSS variables in delegated Vite styles

Issue: [#8664](https://github.com/ubugeeei-prod/vize/issues/8664).
This is a separate native Stack child of
[#8609](https://github.com/ubugeeei-prod/vize/pull/8609).

The original VueFes full-38 run
[38146955685](https://github.com/ubugeeei-prod/vize/actions/runs/38146955685)
at source `37b5f5d0b82dbab29ee5556965f20bb070b2cc1e` passed the previously
failing development speaker/panel/detail routes. It then failed
`preview-timetable`: 2,960,914 of 17,130,240 pixels differed (17.2847%),
against the unchanged 0.4% ceiling. Layout, content and images remained while
card/header colors disappeared. This failed run is retained as historical
evidence; it supplies no full-suite acceptance.

## Reproduction and independent authority

The complete original `TimetableHead.vue` is retained in the legacy fix-history
fixture `production-extracted-css-vars.input.txt`. Its upstream source is
`app/pages/timetable/_components/TimetableHead.vue` at pinned VueFes revision
`8eff3cd`, SHA-256
`fd03e2177c4780f607f99ea303278bc205fc462b452364b61e013dbed0ed28ba`.
Its two declarations use `textColor` and `backgroundColor`.

Actual public 0.443.0 and clean source
`511d0c0fa3e48fe91b05c69be82c23c0d90dbd37` reproduce the same mismatch.
The source NAPI was built from tree
`b1f7737e79688c0b3451bd63fae3a652a1287464`; its signed local binary SHA-256 is
`2898f1a249463c4120d71a7b296e78b914278e211cfc97b2530faf191e7e78d4`.

With actual filename
`/project/app/pages/timetable/_components/TimetableHead.vue`
and scope `86318631`, the complete vectors in declaration order are:

| Mode        | Runtime/aggregate CSS                            | Delegated CSS before this change                 |
| ----------- | ------------------------------------------------ | ------------------------------------------------ |
| Development | `86318631-textColor`, `86318631-backgroundColor` | Same                                             |
| Production  | `v5112640d`, `v7f8f2f6c`                         | `86318631-textColor`, `86318631-backgroundColor` |

The independent official Vue compiler produces the same production pair when
given that filename as its production identifier. The failure is in the
delegated raw-style transform: aggregate styles and JavaScript already receive
the production filename, while the Vite style loader supplied only the scope.

## Decision

Add an optional production filename to the existing Rust CSS `v-bind()`
scanner and NAPI transform. The production Vite loader passes its resolved
owning SFC path, matching the compiler's filename authority. Development and
all existing two-argument public calls retain their previous names and bytes.
Raw and Vite `/@fs/` requests use the same resolved filename.

The change uses the existing single scan. It transforms only actual authored
`v-bind()` calls, retaining comments, strings, hand-written CSS variable
references and malformed tails. A global replacement of development variable
names would corrupt an authored `var(--scope-color)` and is not used.
Extract the existing helper bodies in a separate move-only commit to keep
over-limit files from growing; do not raise any source or runtime budget.

Do not change SSR's `:--name` object keys. Actual Vue server rendering removes
that internal prefix and produces valid `--name:value` inline styles. The
stock compiler and DOM style parser independently confirm that behavior.
The adjacent SSR hypothesis is falsified, so it receives no product change.

## Durable controls and delivery

The original SFC fixture runs through the real native SFC compiler and Vite
style loader in development and production. Compare every delegated/aggregate
name against the independent official compiler, for raw and filesystem
requests, while preserving the exact two-argument development vector.

`production-css-var-scanner.input.txt` and whole-output Rust assertions cover
quoted identifiers, nested expressions, Unicode, comments, strings, authored
variable collisions, scope-prefix equivalence and malformed tails. Independent
Vue 3.5.40 production names own all four transformed declarations; existing
comment/string behavior stays byte-exact, including its deliberate difference
from stock Vue's quoted CSS-content transformation.

The existing six real-runtime expression/mode controls now compare complete
delegated names in both modes, the full official production CSS vector and
JavaScript variable map, and retain their client values and real SSR HTML
checks. Focused local Rust tests pass (three tests, including these six runtime
controls). The actual source-built Vite loader regression is red on the
unchanged 511d binary and green with the candidate transform.

The first source Check 38151702928 at `5cd97cc2` rejected two unformatted
Rust re-export declarations. Correct only their formatting and retain this
failed head. The superseded full-38 dispatch 38151728085 is canceled before
qualification; a fresh dispatch must authenticate the corrected final head.

Corrected source `7ef568265c28e6090376407f1e797defd1dbe070` passes source
Check 38151949747, actual CLI phases 38151949697 and the complete unchanged
VueFes qualifier 38151964793. All 19 development and 19 production-preview
routes pass. Retained receipts authenticate the clean source before building,
the single native artifact, full SSR/DOM/hydration equality and the negative
missing-style-edge control.

The first protected prefix then exposes a genuine composition failure:
candidate `ef240c6755fcd889148aa5affe797ff4687ddb55` fails the original Photo
compiler packet after the independently qualified Vapor root repair. The
parent owns an explicit successor for two nested-component flags in four
Vapor/Nuxt/map coordinates; the original packet and every other field remain
fixed. The child also conflicts in this foundation and the generated Croquis
consumer table. Dequeuing the parent clears both native Stack entries, avoiding
a stalled prefix. Replay onto actual merged `main`, preserve every CSS product
and original corpus byte, union the complete incoming foundation within its
350-row ceiling, regenerate canonical references from the composed source,
and require new exact-head source Actions and the unchanged full-38 qualifier.
The earlier successful source and failed queue candidates remain historical
evidence; neither qualifies the successor.

Exact final-head Actions, the combined original full-38 qualifier with every
existing case/retry/ceiling unchanged, the protected Stack prefix, actual merge
and a later included official release remain required. No failed parent is
queued independently and no published consumer acceptance is inferred from
these source controls. Third-party #8631 and #8623 delivery remains pending
until their separate fresh installed-public oracles pass on an authenticated
included cut. GitHub already closed #8623 through its source merge lifecycle;
that closure does not establish installed release acceptance.
