# Default L1 markup providers

Tracked in [#6835](https://github.com/ubugeeei-prod/vize/issues/6835).

## Decision

Expose the existing profile-generic `markup::Lexer<P, S>` and complete
`markup::entity::decode_one` implementation in ordinary L1 builds. Delete
their two panicking skeleton implementations and lower the L1 skeleton
ratchet from four modules to two. An independently compiled integration test
calls the public APIs without `cfg(test)` or `native-markup-lex` on L1; unit
tests alone previously concealed the default-build skeletons.

Keep the existing `native-markup-lex` feature spelling for the compatibility
callback adapter and its differential test consumers. It no longer selects
between native providers and skeletons. L1 is experimental and has no users,
so the complete decoded-value return type is the canonical provider contract;
the obsolete first-character skeleton signature is removed.

The compatibility tokenizer remains unchanged. Both `vize_l1::parse` and
Armature's production parser continue to call that implementation. This
change neither replaces a product route nor claims native product acceptance;
the compiler fix-history gate [#6880](https://github.com/ubugeeei-prod/vize/issues/6880)
still applies to a production parser switch.

## Provider contracts and validation

- `Lexer<P, S>` statically dispatches `Component` and `Document` profiles,
  pushes source byte ranges into the caller's sink, and ends with `on_end`.
- `Document` tolerates markup declarations; `Component` reports the existing
  recoverable lexical error. The remaining document tree rules are outside
  this lexer step.
- Entity decoding retains every decoded scalar and the authored reference
  span, with attribute-context rules and numeric corrections. The preserved
  tokenizer keeps its explicit first-scalar legacy behavior.
- Default-library integration laws cover both profiles, full `&fjlig;`
  expansion and spans, context-sensitive `&timesX`, numeric corrections,
  custom delimiters and Vue 1 raw interpolation. Both profiles also run over
  every UTF-8 prefix and suffix of the 42 committed surface fixtures.
- Existing Armature feature-enabled event, recovery, AST and diagnostic
  parity remains required. Frozen corpus bytes and instruction ceilings are
  unchanged.

The [earlier opt-in decision](./2026-09-28-l1-markup-skeleton.md) recorded
instruction-count regressions caused by making unused native code available
to thin LTO. Therefore this default-provider slice must pass a manual exact
head measurement of all 100 unchanged instruction ceilings before entering
the protected merge queue. A regression keeps the slice out of the queue;
it does not justify a budget, harness or tolerance change. PR Actions and
the final cumulative merge-group measurements remain required.

## Remaining work

#6835 remains open. Implement document-profile tree rules and integrate the
generic provider into native L1 surface construction under the recorded
parity contracts. Replacing the compatibility production tokenizer additionally
requires #6880 and full legacy-output parity. The two remaining L1 skeletons
are directive decomposition and typed embed source preparation (#6836);
this provider exposure does not complete them.
