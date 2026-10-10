# Keep CLI collection inside the selected path alias

Owning issue: [#3984](https://github.com/ubugeeei-prod/vize/issues/3984).
Reproduction baseline: actual main `77ed369860b19a44f2716dfad9d537d968fba018`.
The test-first source is composed on actual signed main
`f9a9bc554eab90158edbfcdffcb479114bb6c62c`; production collector, Canon
resolver and the original eight-case input/oracle bytes remain unchanged.

CLI import discovery still probes every matching `paths` pattern when an earlier
pattern's targets are missing. Its input collector can therefore add an unrelated
wildcard source to the program even though Canon's corrected resolver retains the
authored unresolved import. A seeded type error in that unrelated source becomes
an extra TS2322 alongside the required App TS2307. The selected project report
already lists only App as its authored root; this is a collector defect.

## Decision and preserved boundaries

Both `PathAliasResolver::resolve` and `resolve_with_inputs` must select one pattern
before probing: an exact key first, otherwise the matching wildcard with the
longest prefix. Equal prefixes retain the existing incoming order. Only the
selected pattern's ordered targets may be consulted; failed targets remain
invalidation inputs. The existing baseUrl fallback stays after those targets.

Keep target substitution, source extensions, path anchoring, package ownership,
configuration loading and the existing import-specifier scanner unchanged. Add no
parser, provider or compiler stage. This slice does not establish authored JSON
key order: the existing serde map can reorder equal-prefix patterns. That remains
the independently recorded ordered-loader TODO. The legacy baseUrl fallback law
does not qualify the retired compiler option against native TypeScript 7.

## Required proof

The additive corpus keeps every byte of the seven-input historical exact-missing
reproduction. It adds complete clean, broken and repaired cold CLI controls,
ordered same-pattern fallback, a missing longer wildcard with a broken unselected
decoy, and an independently selected wildcard error. Expected packets follow
tsconfig selection semantics. Preserve complete stdout, stderr, exit status,
options, authored inputs, program roots, file membership and diagnostic spans
before assertions. Same-root repair means another fresh CLI invocation; it grants
no persistent editor invalidation credit.

The existing native qualification command requires this target and captures its
whole raw results in the existing artifact. Every previous required target, env
value, command, job and stage remains, and the workflow retains its 350-line cap.
Pure resolver laws exercise both APIs, incoming ties and complete consulted-input
vectors without invoking another compiler.

The original eight-case Canon corpus deliberately froze extra empty decoy files
in its two missing-selected cases. Its hash gate covers Canon's resolver, not this
CLI collector. The original inputs and oracle bytes are preserved in the
[historical archive](../../../tests/_fixtures/differential/typecheck/selected-alias-collector-3984/historical-eight-case-manifest.json).
A correct successor must bind an exact collector source identity and require its
own complete before/after packets. Accepting either membership, silently rewriting
the old oracle, or treating every later source hash as qualified is forbidden.

Imported historical RED reports are expressly labeled portable derivatives,
with each original and derived digest plus whole-byte inverse correspondence.
Original physical-root execution records remain outside Git. These historical
records grant no qualification to the fresh source build.

At this test-first checkpoint, production collector bytes remain unchanged.
Actual source-bound Actions RED, the reviewed producer repair, all complete native
controls, the original unchanged workload measurements, protected qualification
and actual delivery remain required. No speedup, 10x target, public release,
default JSX, full project/declaration parity or P0 completion is claimed.
