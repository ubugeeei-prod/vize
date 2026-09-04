# P2-11 Installment 85 - Prefix Identifiers

> Part of the [P2-11 series record](../p2-11.md), split per installment.
> PR: [#5617](https://github.com/ubugeeei-prod/vize/pull/5617), merged into
> the installment-84 branch and carried to `origin/main` by
> [#5615](https://github.com/ubugeeei-prod/vize/pull/5615) at `e9923c0f8`.

`prefix_identifiers` without binding metadata: every free identifier in a
template expression becomes a `_ctx.` member. `emit::prefix` ports the shipped
transform - `process_expression` and `process_inline_handler`, with the
retained-AST splice under the JS-module dialect gate and the legacy re-parse
chain behind it, scope hygiene included - and the codegen consumption that
follows it: the prefix strips, the dynamic-argument visitor and
`prefix_slot_defaults`, replayed in one walk over two stacks.

The content each site rewrites is the text the shipped node held, not the
trimmed source: the padded quoted attribute value, with bind values
entity-decoded, interpolations trimmed, `modelValue` trimmed and the native
`v-model` entry padded.

The allocation gate caught the first shape at 73 against a budget of 60: the
scope stacks and `prefix_slot_defaults` ran unconditionally. The prefix-only
bookkeeping is gated on the option, `RawJs::Borrowed` carries text nothing
rewrote, and the default lane's remaining membership questions are answered
without allocating (`destructure_params_contain`).
