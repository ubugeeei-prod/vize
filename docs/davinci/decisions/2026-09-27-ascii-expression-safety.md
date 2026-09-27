# Complete ASCII word safety (#6868)

This private, unpushed candidate starts at frozen
`5e40274da99c9a25cd0b6c11649d57debf31ec45`. It adds a positive safety path for
one complete ASCII identifier-shaped token. Every expression still goes through
the existing OXC parser and whole-coverage admission rule. Runtime and numeric
acceptance remain pending root review and Actions.

## Source decisions

The first, extraction-only commit `56d9f5366` moves the exact existing public
`expression_is_safe_to_parse` predicate into `expression_guard/safety.rs`, with
the same public path re-exported by its parent. An inverse check reconstructs
the original parent byte-exactly. Its 359 lines decrease to 350. Module placement
follows ordinary Rust module rules, without a path attribute. Feature gates,
compiler callers and dependencies remain unchanged; the new branch uses only
existing byte operations and introduces no allocation or std-only operation.

The second commit tests the first byte for `[A-Za-z_$]`, then calls the existing
`skip_identifier(bytes, 1)`. It returns true only if that scanner reaches EOF.
The original full safety predicate is unchanged after this branch, verified by
an exact inverse against the extraction commit. No inline annotation is added.

The positive path is a semantic subset of the existing guard. The nesting
scanner consumes those same bytes as one identifier, with no delimiter, escape,
numeric-token start, angle, string, template marker or operator. Its analysis
is balanced with zero depths and no oversized numeric token. The prefix scan
also consumes one token: ordinary words contribute no prefix operator; a prefix
keyword contributes one, below the unchanged depth cap of 31. Both original
checks therefore return true on every positive-path input.

Empty, digit-leading, Unicode, escaped, punctuation-bearing and whitespace-bearing
text runs the original guard. Keyword safety does not admit a keyword expression:
OXC still rejects `for`, `return`, `class` and standalone `typeof`, and returns its
actual BooleanLiteral/NullLiteral ASTs for `true`/`null`. Authored source bytes,
file-absolute payload spans and expression-relative AST spans remain intact.

## Exact controls and local evidence

- Carton controls cover identifier starts/continuations, an identifier followed
  by 4097 digits, and the numeric-token boundary of 4096 accepted/4097 refused.
- The depth boundary of 31 remains safe; a word followed by 32 nested calls,
  32 symbolic or keyword prefixes, unbalanced suffixes and an escaped unbalanced
  suffix remain refused. Member expressions, closed trailing comments, Unicode
  and escaped names continue through the fallback.
- L2 controls compare exact source, authored span, complete AST span, identifier
  name (including escaped `a`), literal kind/value and refusal class. Reserved
  keywords, empty text, a second word and statement tails remain ParseRejected;
  numeric/deep/prefix controls retain NestingRefused before OXC is called.
- Rustfmt and the actual assertion scanner pass; both touched integration tests
  have zero weak-assertion findings. All touched source/docs files meet source350.
- All 41 protected raw paths match frozen5e: existing scanners/operators, OXC
  admission, stack guard, every benchmark source, harness/driver/library,
  workflows, budget registries, Cargo manifests and lockfile. The 100 IDs,
  raw input digests, measured windows, protocol, caps and captured fixtures remain
  unchanged. Existing v-for provenance literals, spell/desc behavior and ScopeTag
  formatting are untouched by this candidate.

## Actual status and remaining work

Actual frozen5e instruction run `36320906189` failed 13 ceilings. The v-for
window is `149089` versus `148047`: the fixed provenance changes saved 1251,
leaving 1042 excess. Its initial maps scan still takes 57 iterations. The separate
emitter codegen regression is owned by the buffer lane and is outside this patch.
Full Check `36320903163` did not provide Rust-test acceptance: its exact allocation
probe rejected the improvement from 12 to 11 allocations and 1511 to 1499 peak
bytes, so Rust tests were skipped. No runtime claim follows from the static controls.

The five unchanged safety calls currently cost 1779 inclusive Ir. The candidate
replaces them with first-byte tests and scanning 16 remaining bytes across
`items`, `item`, `key`, `index`, `item`. Even a hypothetical zero-cost classifier
would give 147310 (737 below the cap); that is a cost bound, not a prediction.
Real classifier/fallback costs, inlining and binary-layout effects are unknown.

TODO after concrete root review: compose only approved producer/buffer/allocation
changes on one verification ref; run unchanged full Rust/differential coverage
and the actual 100-probe, three-execution instruction gate. Preserve ceilings,
inputs, window and stack safety. Existing adversarial guard tests and the small
thread-stack recursion control must pass. No local Cargo build/test, install,
push, PR, workflow dispatch or new source acceptance was performed here.

The roadmap owner records the final issue comment and central Performance link
with the reviewed composition. This companion is the private source evidence;
it does not close #6868 or accept the earlier public stack.
