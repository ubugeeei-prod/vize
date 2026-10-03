# Original Document lexical owner

Issue: [#6835](https://github.com/ubugeeei-prod/vize/issues/6835), Stage 1.
The dependent petite-vue integration remains
[#6843](https://github.com/ubugeeei-prod/vize/issues/6843), after #6835/#6841.

## Actual provider boundary

`markup::document::NativeDocument::lex_in` takes a checked complete
`SourceRoot` and runs the actual `Lexer<Document>` once. Private arena-owned
events and diagnostics retain that exact original source. Readonly token
views borrow the original owner; no Component, raw surface tree, caller-built
event vector or replacement source can mint these views.

The private sink receives the driver's actual final `on_end` exactly once.
`normal_completion` only produces a nonClone, owner-bound lexical completion
when that full run has no recoverable diagnostics, unfinished declarations,
declaration recovery or silent pending lexer states. The intrinsic actual
terminal-state receipt must be Text or raw-text content; reaching `on_end`
alone is insufficient. Constant-time sticky sink facts preserve refusals
without an additional event walk.
Moving and reborrowing the owner retain its original run. This lexical
capability has no conversion to a browser tree, descriptor, dialect, L2 or
semantic File admission.

The shared lexer now reports complete declaration byte ranges, including
`<!` and the observed terminating `>`, to an optional default-no-op callback.
EOF declarations retain their complete original frame and refuse normal
completion. The original opening offset survives bogus comment and CDATA
prefix recovery. Complete comments and CDATA retain their separate callbacks.
The inherited driver can swallow a `>` while entering declaration mode for
`<!>` and `<!->`. A separate default-no-op callback records that original
skipped recovery window, which may include following bytes, instead of
calling it an exact declaration frame. It always refuses normal completion,
and a later normal declaration cannot erase that refusal. Existing sinks
retain declaration skipping and all old errors; no legacy or Component
parsing/recovery rule changes.

Callback coordinates are not fabricated tree spans. In particular, an
inferred EOF tag-end coordinate may lie inside a UTF-8 character. The readonly
source accessor checks both boundaries and returns `None` for that original
coordinate instead of snapping it or allocating a replacement buffer.
Decoded multi-scalar entities retain one original entity event, without
duplicating source coverage.

## Tree and dialect policies remain unfinished

Only `TOLERATE_DECLARATIONS` is consumed by the existing profile-generic lexer.
`FOLD_NAME_CASE`, `SELF_CLOSING_ANY_ELEMENT`, `IMPLIED_END_TAGS` and
`TABLE_CONTENT_MODEL` remain declarations of future tree policies. Every
Document owner exposes all four unfinished policies, including after normal
lexical completion. Authored names remain raw; self-closing callbacks remain
lexical facts; this provider does not construct implied or foster-parented
children.

The frozen original Armature capture proves byte-identical legacy AST and
diagnostics, not browser Document semantics. Its Document constructor enables
declaration tolerance over the existing shared parser. The ordinary native
Component builder also cannot become a Document producer by changing the
profile marker: skipped declaration bytes are otherwise recovered holes,
and its tree policies differ from an HTML browser tree.

The [HTML tree-construction rules](https://html.spec.whatwg.org/multipage/parsing.html#tree-construction)
remain the separate future oracle. The repository-pinned
[petite-vue walker](https://github.com/vuejs/petite-vue/blob/9ff3b98b844033b935143c221176c81244aae32e/src/walk.ts)
consumes actual DOM nodes and attributes; lexical callbacks cannot claim that
DOM authority or petite-vue scope meaning.

The existing L0 petite-vue detection is real. Native petite-vue dialect
admission, `v-scope`/`v-effect` L2 meaning, nested scope ownership and genuine
embed handoffs remain unfinished under #6843. Legacy Croquis, Patina and
Maestro routes stay on their original implementations. Lint/LSP replacement
still requires the complete #6881/#6883 fix-history gates. No L3/L4 or product
completion is inferred.

## Source laws and acceptance

Twelve functional laws cover original declaration custody, unchanged Component
errors, bogus declaration prefixes, EOF refusal, comment/CDATA distinction,
multi-scalar entity source identity, UTF-8 recovery, all four unsupported tree
policies and move/reborrow custody, plus silent EOF states, skipped
declaration recovery and sticky refusal. The two independent review
counterexamples (`{{` and `<!>tail<div>`) failed their new laws before the
receipt correction; all current laws require fresh execution.

The complete, unmodified pinned petite-vue SVG example is committed with its
original license and exact source/blob/hash identities in
[`tests/fixtures/document`](../../../davinci/vize_l1/tests/fixtures/document/README.md).
Its real once-run law checks all seventeen authored opening tags, all four
interpolations, the original `v-scope` head and untouched script/style source.
It provides lexical evidence only, without runtime or semantic credit.
Compile-fail laws deny cloning, owner field construction and lexical-completion
cloning. Original callback and parser fingerprints remain unchanged.

The two arena event/error buffers are source-sized Drop-free observations;
their direct and bound uses, plus test-owned lists, are recorded in the strict
storage inventory. The generated L1 census and actual source dependency gate
must stay current. Focused local laws do not replace exact-head Actions,
all one hundred unchanged instruction ceilings, protected queue validation
and actual merge confirmation. #6835 and #6843 remain open.
