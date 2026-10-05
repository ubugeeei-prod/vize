# File-route param parser names in editor diagnostics

Issue: [#7816](https://github.com/ubugeeei-prod/vize/issues/7816).

## Contract and scope

The external report names `src/pages/users/[id=int].vue` and provides a complete
script-setup SFC. The editor warns that `id` is unknown because the file-route
collector retains `id=int` as its name. The
[Vue Router param-parser contract](https://router.vuejs.org/experimental/param-parsers)
defines `id` as the parameter and `int` as its parser; custom parsers and optional
or repeatable parameters retain the same separation.

The fix removes the parser suffix before collecting the file-route parameter
name. It preserves existing optional, repeatable, catch-all, named-view,
deduplication and source-order behavior. It also corrects the editor completion
label through the same existing collector. Parser execution and inferred parser
return types are outside this naming fix; #7817 remains separate.

No Patina CLI rule, native provider, default routing, configuration, shared LSP
history manifest, existing golden or instruction budget changes. The original
CLI distinction in the report remains a regression control.

## Authored regression evidence

`tests/_fixtures/differential/lsp/router-param-parsers/Reported.vue.txt` retains
the complete original report; `references.json` pins its bytes and contains seven
authored complete sources and diagnostic/completion vectors. The expected names
follow the public grammar, and completion details retain Vize's existing string,
array and optional display policy. These references were authored independently
of runtime output.

`lsp-route-parser-params.test.ts` requires the existing source-build receipt and
spawns real `vize lsp` over stdio. Each case checks the full versioned diagnostic
publication on open, the full completion array, an unknown-param warning with
complete identity/range, and disappearance after restoring the source. It also
executes source-built `vize lint` on all seven on-disk inputs and checks their
complete empty diagnostic results. Existing passive wire capture retains the
actual session and source custody. Bounded Rust tests cover named views, nested
deduplication and empty names as well.

This legacy regression corpus stays separate from the historical shared-response
manifest, which supports requests rather than server diagnostic notifications.
No historical or native admission credit is claimed, and #6883 remains open.
The new test is deliberately included in PR tooling as well as the protected
full suite. Compiled Actions execution, protected 104 probes, actual merge and
release delivery remain required; local source review is not runtime evidence.

The reporter's public GitHub identity is `naitokosuke`, ID `102337893`; the profile
exposes no email. The meaningful source commit and PR supply
`Co-authored-by: naitokosuke <102337893+naitokosuke@users.noreply.github.com>`.
Final merge attribution must be checked independently. Queue admission remains
held until the coordinated v0.433 publication and explicit thaw.

## Initial source execution and gate repair

Source `53a67865` ran Check `37222722727` on actual checkout `f6cd662b`, whose
tree equals the source tree. The receipted CLI is `vize 0.432.0`, binary SHA-256
`4a9f55cb39622f6348a1751f2e2abd7df12710421fe7f75f5dee9349b7a366cb`.
Tooling job `111496265865` passed all seven complete real LSP and CLI controls;
artifact `11311505035` retains the actual client/server bytes and build receipt.
Rust source checks also passed, but the whole Check failed two source guards.

The redundant standalone Rust diagnostic test added three `parse_sfc` calls,
raising the crate's lexical count from its immutable ceiling of 15 to 18. Remove
that duplicate test while keeping the stronger complete real RPC regression and
the two modifier/deduplication unit controls. Register the latter's one genuine
test/dev L0 helper reference in the generated Maestro surface census. No ceiling,
reference, production repair or RPC input/output changes. The original failed
campaign remains historical evidence; the corrected head requires fresh Actions
and protected terminal acceptance before actual merge and release.

## Actual protected delivery

Corrected source Check [37223617054](https://github.com/ubugeeei-prod/vize/actions/runs/37223617054)
and protected Check [37227719900](https://github.com/ubugeeei-prod/vize/actions/runs/37227719900)
completed successfully. The protected instruction job `111510724760` verified
both unchanged 100/4 registries and ratchets and passed all 104 ceilings; all
four full Rust workers and tooling shards passed. #7833 actually merged on
2026-10-04 at 19:34:19 UTC as signed
`688da7cc620af5a9aec82152b54d8052c64c52bc`. Its final commit message retains the
literal distinct-reporter footer
`Co-authored-by: naitokosuke <102337893+naitokosuke@users.noreply.github.com>`.
Actual main delivery is complete; publication of this repair remains pending.
Earlier failed source evidence and the unmodified original references remain
separate; no native/history completion credit is added.
