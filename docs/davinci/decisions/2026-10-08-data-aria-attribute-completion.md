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
