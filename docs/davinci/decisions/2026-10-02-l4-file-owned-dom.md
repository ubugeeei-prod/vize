# File-owned native DOM emission (2026-10-02)

Issue: [#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

## Decision

`targets::dom::emit_file` accepts the sole `NativeFileAnalysis`. It derives the
source, canonical artifact, semantic tables, DOM facts and file owner from that
analysis. Callers cannot replace its expression tables, scope, binding policy
or file with a bare `NativeAnalysis`. The existing bare-artifact entry remains
distinct for its declared-context development fixtures.

Both entries use the same append encoder through a private, statically
dispatched expression backend. This adds no pipeline stage, serialization,
expression parse, AST construction, declaration search or second tree walk.
L3 retains semantic grouping, ordering, block eligibility and dependency roles;
L4 encodes checked runtime helpers, property syntax and numeric patch flags.
The same Writer deduplicates helper uses, and late template-module assembly
validates the complete import preamble against the selected runtime vocabulary.

At each genuine canonical expression node, the file backend obtains only its
factory-created `DomFileExpression` row. It checks the file pointer, node,
captured lexical scope record, retained AST/source pointers, authored span and
coordinate capability pointer. Only an actual literal AST with a complete
empty reference table is emitted. The checked expression writer still validates
the authored source/projection before either link sink writes text.

Referenced/FileDependent expressions that reach the file backend return
`RuntimeAccessUnavailable` at their original authored expression span.
Unsupported zero-reference nonliterals retain L3's earlier typed refusal.
A private rejecting access adapter
supplies no context, ref, prop, setup or global fallback. `FileDependent`,
script identity, setup shadowing and literal const initializers establish no
Vue exposure or runtime access spelling. No partial Writer is returned after
any refusal; the analysis and retained source/observations remain available.

## Real output and ownership laws

Eight genuine Program/Vue Component/File/L3 pipelines generate complete
template modules: empty elements, static props/text, mixed text/elements/comments,
root fragments, entity-decoded and Unicode literals at whole-file offsets,
literal class/style, retained block comments, and the actual literal AST family.
Every byte, including import order, matches the pinned Vue 3.5.35 references.
Recorded and NoLinks output and helper lists are identical.

The explicit reference options select module mode, prefix identifiers,
comments, no hoisting/cache handlers, source maps and `bindingMetadata: {}`.
That last option selects the six-parameter render surface of this prepared
target; these laws do not claim byte parity with compiler options that omit it.
No caller binding classes are synthesized for the native file entry.

The generated modules execute in the actual pinned runtime with empty and
throwing-access contexts. The latter also supplies throwing props/setup/data/
options objects. Authored entity/Unicode/comment semantics remain intact and
literal rendering performs no runtime binding access.

Real owner laws retain each factory scope and original AST. Equal numeric node
or binding IDs in separate files never authenticate a foreign owner. Foreign
ASTs and referenced expressions refuse before expression-writer mutation,
preserving existing text, indentation, helpers and links. A genuine setup
declaration shadowing an ordinary declaration is resolved to its actual unit,
then refused for missing runtime access even after an earlier literal binding.
Zero-reference arrays and special attributes retain the existing L3 refusal
kind, original binding/expression span and whole source.

Native maps retain complete whole-file sourcesContent and actual anonymous
literal links, including entity spelling and nonzero authored offsets. The
upstream raw template-relative maps are retained separately and regenerated
unchanged. Whole upstream map equivalence is unfinished; no map filtering or
generic Verbatim named-link policy changes occur here.

## Source and delivery scope

The independent source change is based on genuine file-owned L3 provider
`fbb183528926f3cd468c7bc12540d6f30d01dec2`, whose own source closure is on
accepted main `4cabe83f7d0826907b6035b58d76798e92634927`. Previously frozen owned
DOM/conditional encoder and template assembly source is reproduced without
copying research provider ancestry into this change. Its private bare If path
does not grant file If/For admission.

Local proof compiles complete actual L3 and L4 source against hash-authenticated
ordinary File provider `52dd94a44d14e1f17933020c087e237764b44030` and its
coherent cached L0/OXC/selected stock L1/upper libraries. Fifty-one L4 unit
laws, four genuine file pipeline laws and strict production Clippy pass. Three
template assembly laws cover rebased named/anonymous links, identical NoLinks
bytes, invalid helpers and an actual foreign-vocabulary index absent from the
selected DOM table. The
new DOM body removes one skeleton marker and lowers the L4 ratchet from four
to three. This is bounded source/API evidence, not current whole-workspace,
hosted Actions, no_std matrix or instruction-count acceptance.

The genuine selected Component producer's supported result is checked by
these fixtures. File-family completeness does not certify an entire SFC:
carrying all descriptor/native holes through the authentic whole driver into
the final file owner is an upstream prerequisite. No public boolean or receipt
is introduced to imitate that capability.

## TODO

- Deliver and revalidate the genuine File/Program/component/L3 providers and
  this source on accepted main through exact-head Actions and the protected
  native Stack/merge queue; keep all instruction ceilings unchanged.
- Consume genuine same-file Vue runtime exposure/access, rather than infer it
  from declarations, initialization syntax or FileDependent semantics.
- Connect the real whole-component driver and its complete typed refusals;
  retain unsupported descriptor, control and dialect provenance.
- Admit file If/For only after their authentic control scopes, Dense context/
  source authority, Template-origin binding IDs and formal/access writer exist.
- Complete hoists/cache, remaining DOM features, Vue dialects, JSX/TSX, maps and
  same-semantics performance acceptance before product migration.

No product route changes. #6840 and compiler fix-history gate
[#6880](https://github.com/ubugeeei-prod/vize/issues/6880) remain open.
