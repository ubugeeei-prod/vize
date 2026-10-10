# Quoted nested dynamic arguments in selected SSR

Issue: [#8480](https://github.com/ubugeeei-prod/vize/issues/8480).

The original full Check run
[38035331735](https://github.com/ubugeeei-prod/vize/actions/runs/38035331735/job/114164470517)
reported four selected-versus-legacy SSR output differences. Two distinct
parent sources use valid `names]` and `events]` quoted keys followed by nested
index expressions. The lower directive classifier counted the quoted `]` as
syntax and truncated `state.keys['names]'][state.indices[state.index]]` to
`state.keys['names]'`. A new regression against the original classifier
fails with that exact argument while its seven existing tests pass.

The retained whole parent sources are 347 bytes (longhand) and 337 bytes
(shorthand), with the same original 1005-byte Child. Their lossless copies and
SHA-256 identities live in
`tests/_fixtures/differential/compiler/ssr-nested-dynamic-arguments/custody.json`.
The original mounted DOM/Vapor children completed successfully; their four
whole stdout packets were 9498 bytes with SHA-256
`3ad76a48e60a9595118ec9190333be1ed1cadc37803b9644fe6e64db50bc11a2`.
Those observations do not explain or close the older opaque fatal-child
failure in #7951. Its cause and actual engine remain unknown.
`before.full-ssr.log.txt` preserves the literal original four whole-module
selected/legacy RED records with their timestamps. That compiler baseline
has no actual SSR Node execution credit. Both new compiler routes print
their complete public results before assertions; the runtime input retains
their exact pair. When those entire results agree, executing the current
module executes the same emitted bytes as the explicit legacy result.

## Boundary decision

The lower classifier shares the existing total L1 lexical scanner through a
small borrowed-text API. The scanner's body, original lexer caller, quote,
escape, nested delimiter, template interpolation, and HTML recovery rules
remain unchanged. The API takes the text immediately after the authored `[`
and returns the original byte offset and closed flag. No second scanner,
pipeline stage, serialization, or legacy dependency is introduced.

For a complete argument, lower splits at the reported closing `]` and keeps
the original modifier splitting. For an incomplete name, it retains the
whole remaining authored text and no modifiers, including empty and missing
bracket controls. The shared v-pre and element-identity consumers retain
their complete selected/legacy result checks.

The existing fallible `VueDirectives::decompose` API is not substituted into
the total classifier. Its nesting/offset errors would require a separate
typed lower failure contract. This change neither hides such failures nor
adds an implicit legacy route. The scanner visibility change composes with
the separately owned adjacent-arrow change in #8471; no unmerged source
ancestry or frozen-head edit is used. Existing `=>` inside quoted attribute
names retains the current scanner's HTML-boundary recovery.

## Required evidence

- All three original complete SFCs must actually select `s4`.
- Selected and explicit legacy compilation must agree on every serialized
  public result field, including bindings, macro artifacts, diagnostics,
  CSS, whole code, and map.
- Source maps must be additive: disabling the map changes no other field.
- Both official and current whole map graphs must decode and round-trip,
  retain exact source content, and use valid generated/original UTF-16
  coordinates. Current maps retain their script-provenance-only scope;
  whole official template-map parity and argument anchor coverage are not
  claimed.
- Pinned Vue/compiler/server `3.6.0-rc.9` and plugin `6.0.7` must execute
  complete original Child and parent modules. Three selected-key states
  cover first, second, and reordered indices for both parent spellings.
  Twelve official/current renders must match independently written whole
  HTML expectations with no diagnostics or executed event handler.
- Raw runtime input, whole stdout/stderr, actual status/code/signal, and
  observed checkout source/tree/parents are retained before assertions.
  The complete packet is also printed in the original full feature recipe's
  raw job log. Actual Node metadata comes only from the executing helper.
  Checkout metadata does not identify the Rust executable or its build.

## Delivery

The original 31 mounted laws, compiler and canonical populations, SSR
10-case battery and 11 builtin controls, native gates, full recipes,
historical differences, and performance caps are unchanged. The n8n lane's
51-rule inventory is a rule identity witness, not 51 SSR compiler cases;
its full accuracy remains independently unfinished. No upstream change,
publisher edit, release graph change, or v0.441 inclusion is part of this
source repair.

Exact `2b192` full Check `38041695808` preserves zero-divergence original
SSR smoke results but fails to compile the new integration helper with
`E0583` at the crate-root module declaration. The helper moves byte-exactly
to `tests/nested_dynamic_arguments/mod.rs`, the ordinary crate-root lookup
location; fixture path depth, assertions, production, and recipes remain
unchanged. No new whole SSR runtime packet executed on that failed head.
Its separate Misskey server RSS result is 130.51 MiB over the unchanged
128 MiB ceiling; fresh full proof and the real LSP prerequisite remain.

Exact `ca0742` full Check `38045442545` executes all 24 complete compiler
packets, and the original-source selected/legacy result and additive map
checks pass. It preserves two failures: the shared v-pre/identity control
still differs on its original ancestry, and the actual Node child exits 1
with no signal before any of the twelve renders. The pinned official Vite
SSR module uses `__ssrInlineRender: true` and a `setup` function returning
its renderer, so the evaluator's separate `ssrRender` assertion rejects
that valid official Child. The evaluator validates the genuine shape per
route without invoking or replacing setup: official requires inline SSR
and setup; current still requires its separate SSR renderer. All authored
modules, complete map graphs, twelve HTML/state/diagnostic oracles and
shared lower controls remain unchanged. Fresh full execution on genuine
current main is still required; this failed run has zero completed renders.
A local replay of those exact source-built compiler modules and maps with
pinned packages reproduces exit 1 before the correction and exit 0 with all
twelve original outcomes after it. The local host is Node `26.11.1`, distinct
from the runner's `24.14.0`; this diagnostic replay is not new Actions or
current-main qualification. Earlier local missing-dependency failures stay
separate from the actual runner's renderer-shape failure.

TODO: obtain fresh exact-source full Actions, complete runtime/map packets,
unchanged performance and protected qualification, and actual merged main.
Local classifier failure and fixture custody do not replace those witnesses.
