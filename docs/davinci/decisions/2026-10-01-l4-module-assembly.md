# Prepared-fragment L4 module assembly (#6840)

## Decision

Implement the neutral module assembler independently of the unfinished L2/L3
target providers, as permitted by the maintainer's parallel-work order. Assembly
joins existing `Writer<L>` fragments directly: hoists/cache prelude, rewritten
script, render output, component attachment and default export. It builds the
helper import preamble last and uses `finish_with_preamble`; it adds no pipeline
stage, JavaScript parse, serialization or mid-string insertion.

The previous experimental signature could not represent linked script rewrites,
cache declarations, a named client/server render binding or an inline setup
insertion point. Replace it with prepared writers, an explicit function binding
and component field, or script prefix/suffix surrounding an inline expression.
The script provider owns its rewritten component declaration, default-export
removal, `setup()` and return punctuation. The assembler appends inline fragments
without adding whitespace, so it cannot insert an ASI-breaking newline after
`return`. Script-only/template-only assembly declares the empty component only
when the caller supplies no script.

Complete statement fragments have an explicit separator on its own line. A
newline alone would let a script beginning with `(` continue a prelude's previous
initializer; placing the separator after the newline also preserves a trailing
line comment. This separator is never inserted within inline expressions.

All fragments use one caller-supplied `Vocabulary`, whose helper indices and
`_name` aliases are shared; the vocabulary provides distinct JavaScript names.
Imports contain each used helper once, in complete
body first-use order. Unknown helper indices and inconsistent placement return
typed errors instead of calling the unfinished runtime table provider. Module
specifiers are JSON-quoted JavaScript string literals. Generated identifiers and
fragment syntax remain the responsibility of their providers.

Recording and non-recording assembly produce identical bytes. Recorded hoist,
script and template links are rebased onto their final positions, including the
late import preamble, and the finished document can emit one Source Map v3.

## Verification

Nine module laws cover script-only and empty components, complete fragment link
offsets and recording parity, used-helper deduplication, server render attachment,
inline return semantics, safe statement boundaries, quoted imports and malformed
contracts. All 35 crate tests and strict all-target Clippy pass locally; the
existing writer and source-map laws remain in the same suite. Source-derived
consumer and source-location inventories are unchanged and pass their checks.
A read-only review reproduced the ambiguous join in Node 24 and verified the
separator in real ES modules, including a trailing line comment. The skeleton
ratchet lowers L4 from seven unfinished modules to six. Actions and the protected
merge queue remain the exact-head and instruction-count acceptance gates; no
numeric budget, production route or differential fixture is changed.

## Remaining scope

This is a neutral provider slice, not closure of #6840. Expression rewriting,
runtime vocabulary tables, DOM/SSR/Vapor/type-check targets, mixed-runtime helper
imports and production SFC integration remain unfinished. Their consuming slices
wait for their actual provider contracts. Compiler product-route replacement also
waits for #6880 fix-history fixtures and native parity. Prepared fragments and a
passing module law do not count as native product acceptance.
