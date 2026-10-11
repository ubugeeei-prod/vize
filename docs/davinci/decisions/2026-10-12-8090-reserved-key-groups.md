# Match reserved-key spellings and Options API groups

Issue: [#8090](https://github.com/ubugeeei-prod/vize/issues/8090).
Parent: [#8668](https://github.com/ubugeeei-prod/vize/pull/8668).

After the parent's bare-name correction, real-project Matrix
[38147644073](https://github.com/ubugeeei-prod/vize/actions/runs/38147644073)
still contains native underscore findings in computed and methods members.
The [pinned official default rule](https://github.com/vuejs/eslint-plugin-vue/blob/9aa463a4aa4ad3c37d7facacda2e40ad7880ea92/lib/rules/no-reserved-keys.js)
checks the exact [24 reserved public spellings](https://github.com/vuejs/eslint-plugin-vue/blob/9aa463a4aa4ad3c37d7facacda2e40ad7880ea92/lib/utils/vue-reserved.json)
in props, computed, data, methods, setup and asyncData. Its special underscore
restriction applies only to data and asyncData; inject is outside the default
groups. Arbitrary `$` prefixes and `_` members in other groups are permitted.

Correct the predicate over the existing Options descriptor: exact reserved
public names in props/computed/methods/data/setup, underscore names only in
data, and no inject findings. Preserve the existing metadata and complete
positive diagnostic packets for every still-valid finding. Update the source
examples and public rule description/domain to name the actual bounded groups.
No preset addition, options producer extension, second parse, semantic query
or pipeline stage is introduced.

## Frozen inputs and complete controls

The new `reserved-prefix-8090` corpus retains twenty authored controls covering
all 24 reserved names in five groups and array props (144 exact-name positive
findings), arbitrary prefixes, lookalikes, inject object/array exclusions,
mixed data, escaped/quoted names, Unicode/CRLF, setup locals and other groups.
Two unsupported controls name asyncData and script-setup defineProps gaps.
Core parent source expectations contain 227 findings; authenticated public
0.443.0 contains 230, including the parent's separate bare-name repairs.
After expectations retain 152 whole native diagnostics. The official core
contains 155 findings, including three explicit unsupported findings.

The original parent sixteen inputs, 137 public-before and 17 historical prefix
after vectors, baseline packets, source hashes and reference tests remain
unchanged. The child supplies independent literal successor vectors with nine
whole findings for those same input bytes. Existing API/CLI consumers read
that declared successor instead of filtering observed diagnostics or weakening
budgets. Seven original routes, including default presets, explicit disabling,
incremental and the unchanged n8n projection, remain exercised.

The unchanged `script/no-reserved-props` remains active beside this rule.
Separate source-authored literal companions contain 59 complete findings
across five core props carriers, so full default packets contain 211 findings
rather than target-only 152. Explicit target-off retains these companions;
incremental and n8n remain empty. The inherited sixteen carriers retain the
parent's complete target 17/default 19 vectors as historical authorities, while
separate current successor vectors contain target nine/default eleven findings.
No observed packet is filtered, normalized or used to replace an old vector.
The initial API companions incorrectly assumed an interleaved start/end order.
The template-free SFC source path appends registry diagnostics and returns
before merge sorting, so a separate literal API successor preserves all target
findings followed by all companion findings. Initial vectors remain frozen;
CLI identity order stays defined by the existing aggregate producer. No
production diagnostic ordering changes. Exact source qualification is required.

Twenty-one immutable complete original Vue files from six projects retain
37 historical underscore findings: Element 19, uploader 5, Element Admin 7,
Habitica 3, Bootstrap 2 and Vue Select 1. Six entire original Matrix divergence
packets remain frozen, with source revisions, raw URLs, complete file bytes,
hashes and every finding's original location. Corrected policy removes the
36 computed/method findings. Vue Select's data `_value` is still a correct
native positive; its `eslint-disable-next-line vue/no-reserved-keys` explains
the official suppressed message. That existing suppression-compatibility gap
is retained explicitly, and no original-file expectation is blanket empty.

The new API and source-qualified CLI consumers each execute 140 core routes
plus 21 original-file target-rule routes (161 observations). The parent's 112
routes remain separately retained. Public-before passes all 41 complete target
packets (230 core +39 original findings =269). Source-parent and original
public results remain distinct authorities. Official provider packets retain
all messages, suppressed messages, warnings and fixes, with pinned package,
lockfile, source and observer custody. The initial provider replay matched 37/41; its omitted complete Property-node
metadata for defineProps and seven unused-disable warnings/fixes were then
authored from pinned source contracts in a separate provider supplement.
All initial vectors, 82 reference packets, primary manifest and exact helper
remain frozen. Final provider replay matches 41/41, including 155 reserved
findings, seven warnings/fixes and one suppressed Vue Select data finding.
No directive checks were disabled and no native after vectors changed.

The original computed `_foo` unit snapshot was a false positive under the
pinned rule. Its exact Git blob, original source/byte span and complete snapshot
move into the historical corpus in a move-only commit. The active test becomes
a negative; inject and mixed props counts follow the same independent source
basis. The positive `$el` snapshot and every still-valid diagnostic's whole
metadata remain fixed. New expectations are authored from source, not captured
from the candidate.

## Remaining work and delivery

The parent actually merged at `19e8fa700d3c79631166bcaed4e3cfe65dda1903`
after all five current protected workflows succeeded. Rebase onto main
`67e3d0e6775cc3a1f29bfb9309d991252f9e75ff` preserves all 64 historical
fixture files and the reserved-key predicate byte for byte. The incoming
canonical record remains complete at 350 lines with only owned row 209 added.
The already merged setup-emits change
`2284cb1d47009c5d19a4c626e7029d5c22d6bdda` relocated the SFC dispatcher;
the old full-file source hash correctly refused that new main. Its complete
failure log and the original API-order authority remain frozen. A separate
`api-order-main-source-custody.json` pins the five current producer files,
retains the previous source records, and proves the active rule iterator is
byte-exact. The moved loop still appends each rule's whole normal/setup results
before the next rule. Every output vector and its order remains unchanged;
fresh source and protected qualification are required after this rebase.

The shared descriptor does not expose asyncData and this rule has no
compiler-owned defineProps provider. Those named false negatives remain
unfinished; this slice does not add macro shadowing or declaration semantics.
ESLint inline suppression compatibility remains unfinished, including Vue
Select's retained data positive. The old help wording remains byte-exact for
valid diagnostics; the rule domain and source examples clarify the corrected
policy without changing these legacy packets.

Full Matrix parity, broad #8090 completion, exact-head native 161/parent 112
packets, all configured Actions, protected full/performance checks, actual
merge and a successful official successor release with installed acceptance
remain required. No historical provider/before run qualifies the candidate,
no failed 0.446.0 publication is credited, and no budget or corpus requirement
is reduced. Use a native Stack above #8668 and merge only an exact-green
contiguous prefix so the ready parent can proceed independently. The root
operator owns release triggers.
