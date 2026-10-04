# Configured Router diagnostic rendering

Tracking: external [#7817](https://github.com/ubugeeei-prod/vize/issues/7817),
[Draft #7845](https://github.com/ubugeeei-prod/vize/pull/7845).
This is fixture comparison preparation; source/runtime adoption remains pending.

## Actual failure

Exact source `cfcd10baa44e5ca2e09db04d0dffd674f31550c3` passes source
Check37241040235 (39 Router Rust bodies) and independently audited Native
Phase37241039921 (45 ordered output pairs /105 native graphs).
The necessary [full Check37241714582 / test-scripts111551584412](https://github.com/ubugeeei-prod/vize/actions/runs/37241714582/job/111551584412)
fails the first primary case's complete native-versus-JS message comparison.
The native repair now emits the same TS2353, file, line16 and column7 as the
original official reference. The two renderers order and shorten the parser
union differently. Nine other primary subtests and18 secondary cases pass;
the `definePage` LSP phase never executes because CLI comparison fails first.
Do not call this complete runtime parity.

The original4,566,643-byte ZIP/artifact11316968172 SHA256 is
`6bce853f4d05b56977579aae17ecc5c679870b6abd4895396fe463095cab0cf5`.
All348 members,27 paired CLI reports,54 original raw process results, four
full LSP streams, provider files and source-built binary are retained.
Failed-review receipt SHA256 is
`02a1e0eaff202397d0f2784c0bfc582061ce5a5dbbde7b7dd9a96d7b20cb1ee5`.
The actual source-built binary SHA256 is
`ad3a8db5dfb0775279ca1b319e86c6911990a272ba2dd4fc044a4c7aa35ae19a`.
The fixture resolves the frozen native7.0.2 runtime and Vue TSC3.3.11 /
JavaScript TypeScript6.0.3 from existing locked dependencies; no versions or
production dependencies change. Original argv/configs/streams remain evidence;
no new compiler-version process was executed for this offline review.

## Closed fixture contract

Prepare one explicitly selected `define-page-unknown-id` render case. Require
its exact authored source SHA256
`fc7bc0530fc13ee5fcc67cba6f3e54b1a86280fdf80caf679e1fedbf58199844`,
original generated-map SHA256
`1a19bf3a6f143d7b7ee5c931d0f8da9a4e0a06f378278fc2615f99db746d95c8`
and complete provider-archive SHA256
`5c0bf884438b9c58b1e926663e07572c80c5dcc8d0ddd57a32943a0161d8ee5f`, plus the
exact page path and one full diagnostic vector at16:7 /2353. Both complete
message strings are frozen separately from their original observations, and
each engine's full row vector must equal its own expected vector. File,
line, column, code, count and order remain equal across engines. Unknown cases,
other contexts or changed full messages must fail; there is no generic sorting,
ellipsis expansion, regex-only acceptance or replacement of raw output.
Before reference execution, require byte-identical source, complete tsconfig
and actual current generated-map bytes between the two sides of this case.
The map hash comes from current captured input bytes, not cached original
authority metadata. Every other fixture retains its complete comparison. The
existing parser joins diagnostic whitespace as before; it receives no new
normalizer, and original raw streams remain unchanged and retained.

Return the strictly validated native rows to the unchanged editor comparison,
so the actual full LSP message, source, severity and UTF16 range must equal the
native expected vector. Preserve complete original native and JS streams before
assertion. This qualifies two exact engine outputs for one fixture context;
it does not establish general cross-engine message equality.

Whole provider member `dist/index-BQLwgiyK.d.ts` SHA256
`79f82780d617a96c00b0d5015e1e8455b674d0b0e06585b88d730bfcc49a2aae`
independently ties `definePage<FilePath>` to
`TypesConfig._RouteFileInfoMap`, its `pathParamNames` and optional parser-key
properties. The unchanged generated map permits only `userId` for this entry.
Its13 custom parser keys and the native `int`, `bool`, `string` parser keys
explain the full native union. The JS renderer prints the first nine custom
keys and omits exactly `test-num`, `test-set`, `test-set-shape`, `version-range`.
This supports the same unknown-key type error; it grants no general soundness.
A same-native official-projection oracle would require a separate materialized
Volar projection/map/helper/program proof and remains unimplemented/unmeasured.

## Required delivery

Pure controls must reject corrupted/truncated/ reordered messages, extra or
missing diagnostics, changed code/range/path/source/provider/map authority,
unknown cases and missing pins. Then require fresh source Actions and complete
original-provider CLI/editor replay, including the previously unexecuted page
macro invalid/repair phase. Production, provider declarations, compiler flags,
full raw capture, source mapping, caps, locks and instruction budgets remain
unchanged. Ready/adoption review, protected checks, actual merge and release
remain pending. The whole-cold500-SFC425.5ms→42.55ms target and opt-in fallback
cost remain unfinished; this fixture work supplies no speed or CPU claim.

Local pure controls pass47/0 (Node25.8.1); the unchanged raw-first stub
controls pass8/0 (Node26.10.0). Configured bound format/lint passes all five
changed TS files with zero warnings/errors. These checks execute no real native
CLI/provider and do not qualify the new source; fresh hosted gates remain.
