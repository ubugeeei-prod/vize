# Retained bindings and fallible compatibility inspection

Decision for #6838 and #6832, 2026-10-02. Paired comments follow publication.

An actual OXC `FormalParameter` is a binding, not an expression. Native For
aliases therefore use `BindingRef::Js` and a neutral `JsBinding` carrying the
original parameter pointer, checked authored range and shared decode/wrapper
coordinates. Compatibility aliases use `BindingRef::Expr` with their existing
expression pointer. No parameter is cast into an expression or reconstructed
from source text. The producer remains responsible for its L1 admission.

`NativeForAliases::checked` admits one to three dense formals from the same
checked alias block, in actual AST order. An index requires a key. Class
constructor modifiers, reversed/overlapping roots and mixed contexts are
refused. The native carrier keeps destructuring, rest inside a pattern,
defaults and TS annotations as actual syntax; it does not legalize them for a
backend. A dependent checked For factory will construct semantic binding facts.

The actual 64-bit footprints remain `BindingRef` 16 bytes, optional binding
16 bytes, `ForBinding` 64 bytes and `ForOp` 96 bytes. Existing `ExprRef` and
the old owned dump grammar keep their representations. The protected 100-probe
instruction gate must still establish compatibility; no ceiling is raised.

The existing dump conversion now returns `Result<Page, NativeDumpError>`.
Its same recursive conversion propagates `JsBindingUnsupported` with the
actual authored span instead of inventing expression syntax or a partial
page. There is no preliminary scan or extra stage. Normal conversion drops
partial owned documents on refusal. Existing expression aliases produce the
same canonical bytes and remain readable through their current parser.
The legacy emitter likewise returns its existing unsupported outcome for a
native alias until a real binding-emission contract is implemented.

The existing transitional L2-to-L3 operand conversion retains a native binding
as `ValueKind::Opaque` with qualifier `native-binding-unsupported`, its actual
authored range and parameter text. It constructs no expression AST and never
turns a binding into `Absent`. This is an unsupported fact, without native
resolution or emission semantics. DOM alias entry points and SSR's string and
VNode alias entry points return their existing typed refusal; Vapor refuses
the actual lowered opaque operand before loop execution. MoonBit records its
existing typed unsupported projection rather than manufacturing an expression.
All exposed execution routes must refuse this fact until a real binding
writing contract is reviewed; an opaque spelling is never emission authority.

All existing readers adapt at their current boundary: test fixtures establish
their expression-only premise; optional owners return `None`; guest callers
return their existing typed trap. Observing SSR/Vapor paths use L0's lazy
`try_page`, including DOM's captured L2 boundary. Debug pass inspection maps
the actual dump refusal into the existing `PassFailure` channel, not a panic
or skipped verification. The observer records failure, the after-pass hook
does not run, and the existing lowering diagnostic retains the alias span.
Successful verifier/table checks and ordinary product bytes remain unchanged.

Curator's product feed retains failed inspections separately as optional
`unavailable` rows of level, step and full reason. Empty success feeds omit
that field byte-for-byte; nonaccepted selection strips it even if a producer
forgot to finish. The independent ladder records path, stage, pass and reason
at its actual L2 boundary or pass. Failure never enters its `Collector` as
page text. The optional schema-1 host field is distinct from product evidence.
WASM forwards these same-run facts, and frontend protocol negotiation keeps
them separate from valid pages and rejects malformed or fake-page records.

Scoped actual-source evidence covers complete L2 compilation, 47 L2 unit laws
and seven current-main expression-resolution integration laws, with strict
production Clippy; complete L3 and L2-to-L3 compilation plus strict
edge Clippy; nine native/decision/expression edge laws; eight real
compatibility dump/replay laws; 17 capture and
host/schema laws, six existing byte-exact stage-feed laws, and nine real
frontend protocol tests. Seven new refusal laws use actual retained formals,
including public DOM and SSR rejection boundaries and Vapor's real lowered
operand admission and both debug transform entry points. The two edge laws run
locally; the five product laws await
exact-head full Actions because the coherent cache lacks complete product
semantic dependencies. Test-only pinned OXC AST edges in SSR and L2-to-L3
support genuine fixtures and add no normal dependency.
Selected host registration borrows real cached L0
primitives and the actual new capture module. These results do not prove a
whole current workspace or production SSR/Vapor build. Exact-head Actions,
frontend/full compatibility suites and unchanged strict instructions remain
required before actual merge.

## SSR emitter source policy

The SSR emitter policy inventories every nested Rust source and follows
ordinary module registrations from `emit.rs`. Only actual attached
`#[cfg(test)]` module edges and their exclusively reachable descendants
establish a test-only fixture allowance. Any production route overrides it;
unregistered files remain checked regardless of file or directory names.
The existing token and item masker also handles inline test-only items.
Path metadata, including `cfg_attr`, includes and symbolic entry or
descendant routing fail closed. The original forbidden legacy-AST pattern,
bridge fixture assertions, Rust implementation and budgets remain unchanged.

Two exact-source local policy laws pass: the actual emitter and a filesystem
fixture that accepts a genuine test-only oracle subtree and rejects nested
production reads, spoofed or removed test gates, mixed production conditions,
additional production routes, unregistered files and unknown routing. The
unchanged compile-bridge and counter laws passed on hosted
`64570ac854076a8835e3aef9f3e26cae9ac12979`. The local runner extracts only
the two policy-law bodies to avoid the original module's unrelated eager
Cargo metadata import; it uses no mock metadata or Cargo build. The whole
four-law module, full compatibility and Fuzz suites, and unchanged 100-probe
instruction gates require fresh exact-head Actions before protected merge.
This CI correction supplies no native admission or completion credit.

## Strict fixture correction and current-main replay

The 2026-10-03 replay retains the four owned bridge/policy changes on signed
main `e648610c51f79aeeefe08a20b936c0ae1ed39ad0`. The original registered
retained-formal fixture reproduces six strict unwrap/panic findings. Its helper
now returns the actual parse/root/parameter/span/coordinate/binding failures;
the original test asserts that preparation succeeds before checking the same
unsupported operand, retained span and compatibility ordering. No fixture
input, expected diagnostic, assertion or lint scope is weakened. The existing
dump-error display assertion uses `cstr!` with the same expected bytes.

Complete replayed L2, L3 and L2-to-L3 production source compiles with strict
`clippy::all` and the thirteen workspace panic/scope denies. All 47 L2 unit
laws, seven expression-resolution laws and the original two binding-refusal
laws pass with those same strict flags. The registered edge unit target also
compiles strictly and contains zero tests; it supplies no additional law count.
These finite checks use cached historical L0 and pinned stock OXC primitives,
so current whole-workspace ABI, product/frontend suites and unchanged full
instruction gates still require fresh exact-head Actions. Prior hosted results
are evidence for their original heads, not acceptance of this replay. The
canonical consumption shards are regenerated from the actual replayed source.

## Remaining work

- Native binding emission and a real native-binding dump grammar remain
  explicitly unsupported; no product replacement is enabled here.
- Native For/If factories and real ComponentParse consumers are dependent
  source slices, with independent ownership and tests.
- Resolve references/defaults through actual lexical facts and implement the
  backend contracts before admitting native control flow to a product.
- A frontend view of separate unavailable inspection rows remains UI work;
  the protocol preserves them without manufacturing a page or rung.
- #6838 and #6832 remain open through their remaining scope and merge gates.
