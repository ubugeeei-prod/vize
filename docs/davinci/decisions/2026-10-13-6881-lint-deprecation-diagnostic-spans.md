# Lint deprecation diagnostic targets

Date: 2026-10-13. Related issue: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881).

The original v0.448.0 Matrix run [38331707741](https://github.com/ubugeeei-prod/vize/actions/runs/38331707741)
reports paired findings for three implemented migration rules. The native and
eslint-plugin-vue 10.9.2 severities agree, but their ranges identify different
pieces of the same deprecated syntax. The first six original shards contain
136 slot-scope pairs, 54 `.sync` pairs and six functional-template pairs.

Report `.sync` on the complete binding directive, `slot-scope` on the attribute
name, and a functional SFC template on its complete `functional` attribute.
These are the actual upstream diagnostic nodes. Recover the functional attribute
only within the descriptor's already parsed opening tag, respecting quoted
values; do not add a parsing stage. Keep the existing rule detection, messages,
help, severity, fixes and dialect gates.

The corpus retains the two complete Vue Element Admin files at revision
`6858a9ad67483025f6a9432a926beb9327037be3`, their original Matrix observations,
and eighteen native API cases. Independent pinned-provider execution covers
short and long bindings, dynamic arguments, multiple modifiers, multiline and
Unicode values, boolean and valued attributes, misleading quoted text,
duplicate attributes, healthy replacements and neighbouring names. The native
corpus compares complete result packets, including messages, help, labels and
fixes. A petite-vue document remains a foreign-dialect control.

The original failed Matrix observations remain failed evidence. No comparator,
preset, rule map, documented divergence or zero ceiling changes. Actions must
qualify the new native implementation; a new Matrix is owned by the release
coordinator after actual merge. Other inheritance, props, slot, component and
baseline-parser failures remain separate work.
