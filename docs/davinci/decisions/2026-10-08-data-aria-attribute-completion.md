# Contextual data and ARIA attribute completion

Issue: [#8015](https://github.com/ubugeeei-prod/vize/issues/8015).
Based on actual signed main `486c390d81643c16b865754239a987cfe4c9861e`.

This implements the data/ARIA follow-up already recorded in the
[common component attribute decision](./2026-10-06-component-global-attribute-completion.md)
and linked from the canonical decision record. Existing common attributes,
expression globals and declared generic-prop hover fixes are actually merged.

Keep the existing nine complete common candidates unchanged at empty opening-tag
and binding prefixes. Matching typed data/ARIA prefixes receive the same contextual
family on native and component tags. Explicit declared props retain their type,
documentation, native resolve payload and camel/kebab authority.

Use the complete authored data name when present; otherwise offer
`data-${1:name}="$2"`. At the original `dat|a-role="x"` position, insert only the
typed prefix so the existing suffix and value remain intact. Known assigned ARIA
names use the same edit protection, including whitespace before `=`. Retain the
existing `aria-label` payload and add the other 52 names from the repository's
accepted 53-name ARIA vocabulary, including its retained deprecated/draft names.
Normalize each declared prop name once and scan the authored name once per
matching request. Existing parse, native request, cache and stage boundaries stay
in place; measured performance targets require separate Actions evidence.

The corpus retains the full original issue/four SFCs through the merged parent
fixtures, the pinned accepted vocabulary and literal complete data/ARIA payloads.
Five Rust laws cover original assignment edits, whole native/component family
banks, declared aliases, unchanged defaults and spaced ARIA assignments. Actual
source-built stdio controls cover the original case, both native/component tags,
static and bound prefixes, complete declared/native prop authority, same-line
Unicode/CRLF dirty buffers, unchanged disk and full restoration. The existing
native-phase Actions cell explicitly includes the new CLI target alongside every
incoming original target; its workflow remains 350 lines.

Source/native Actions, protected full/instruction gates, actual signed merge and
installed/public release replay are required before delivery is complete.
Preserve all earlier original/global/generic oracles. Keep #8015 open until the
remaining full acceptance and delivery are actually qualified; unrelated P0
issues and Content Mapper combined-diagnostic limits remain unfinished.

## Optional native resolve carrier

The [native run on `76cfe84d`](https://github.com/ubugeeei-prod/vize/actions/runs/37645309688/job/112874418271)
retains three failed complete optional-prop comparisons; the original assignment
and disk/hash controls passed, while the compared visible declared
type/documentation fields match. Both authored
`Declared.vue` props use `?`, and the runtime remains pinned to TypeScript 7.0.2.
The prior test carrier incorrectly reused required-member metadata. The independent
[backend optional-member contract](https://github.com/microsoft/typescript-go/blob/89d5d5b2849a0db0957065889ca58536fa6d2e4a/internal/ls/completions.go#L5069-L5080)
decorates the raw label with `?` and preserves the undecorated filter/insertion
name; its optional sort priority is `12`. Pin all those complete fields while
keeping the undecorated resolve-data name, independently witnessed file/query
UTF-16 offsets and source revisions unchanged. That backend source is corroborating
primary evidence, not a claimed source checkout of the pinned binary. No response
fields are removed or learned from the response. Retain the red run; fresh exact-head
source/native and protected qualification remain required without a waiver.
