# Native L1 component surface construction

Tracked in [#6835](https://github.com/ubugeeei-prod/vize/issues/6835).

## Decision

Expose ordinary-library `vize_l1::markup::parse_component`,
`parse_component_with_options` and `parse_component_with_authored`.
They instantiate `Lexer<Component, Recorder>` directly. The recorder's native
`Sink` records authored syntax events without a compatibility callback adapter
or a legacy parser. Both lexer entries share the existing event-to-tree builder,
source-length guard, hole recovery, fidelity verifier and optional authored
projection; there is no additional pipeline stage or serialization.

Full entity-value callbacks record their authored span once, including named
references with multiple decoded scalars. Surface tokens remain borrowed source
slices. The event/builder quote vocabulary is native; preserved callback quotes
convert exhaustively at their entry. All 21 native lexical errors convert
exhaustively to the existing L0 surface diagnostic contract.

Admission is deliberately narrow: Component lexing, default `{{`/`}}`
delimiters, raw interpolation disabled, and the existing in-tag-comment bool.
No unrestricted `LexOptions` or generic profile surface entry is exposed while
the builder's delimiter framing and Document tree rules are unfinished.

The root parse APIs and every compiler product route keep the preserved
tokenizer. An actual standalone CLI consumer follows as the next native GitHub
Stack layer, using this public provider for L1 roundtrip only. Product route
admission and full native parity remain separate requirements.

## Validation

Ordinary-build integration laws compare all tree fields, token leading/status,
ordered diagnostic codes/offsets, hole counts and byte rendering with the
preserved constructor. They cover the 42 committed fixtures and every UTF-8
prefix/suffix, pinned fixture hole counts, complete entity references, malformed
quotes, in-tag comments, and normal/authored interactive-tag recovery.
An internal event oracle also compares exact ordered event records and verifies
one record per authored multi-scalar entity span. Admission laws explicitly pin
the current default delimiters, Component declarations and unfinished `v-pre`.

Before queueing, require exact-head PR Actions, unchanged all-100 manual
instruction measurements, review, then protected cumulative merge-group checks.
Only actual merge and fresh-main ancestry establish delivery. Validation results
and exact heads are recorded on the issue; no budget or tolerance changes apply.

## Remaining work

`v-pre` suppression is unfinished and intentionally follows the current surface
contract: children containing mustaches remain interpolation nodes. It requires
dialect-owned directive decomposition and a scope controller sharing the tree's
namespace and exact implicit-close recovery. The unpublished `5909a8244` recorder
does not establish that contract: it matches only raw `v-pre` and truncates scopes
without the builder's implicitly-closed-tag precedence. Preserve this regression
law for the implementation:

```html
<section>
  <a
    ><span><a></a><span v-pre>{{ inside }}</span>{{ tail }}</span></a
  >
</section>
```

Document tree rules, custom/raw delimiter framing, typed embeds, surface module
scoping and preserved tokenizer retirement remain unfinished. This bounded
native provider does not close #6835 or establish a single production lexer.
