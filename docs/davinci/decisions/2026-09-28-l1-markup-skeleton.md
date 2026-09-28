# L1 markup and container skeleton

Tracked in [#6835](https://github.com/ubugeeei-prod/vize/issues/6835),
[#6836](https://github.com/ubugeeei-prod/vize/issues/6836) and
[#6837](https://github.com/ubugeeei-prod/vize/issues/6837).

- `vize_l1::markup` holds `Profile` (`Document`, `Component`), the shared
  `Lexer<P: Profile, S: Sink>`, lexer vocabulary (`Sink`, `QuoteType`,
  `Namespace`, `LexMode`, `LexErrorCode`), the entity hook and the `Vue`
  grammar placeholder.
- Profile differences are associated constants on `Profile`, one list for
  the lexer and the tree builder. Only `TOLERATE_DECLARATIONS` is a lexer rule.
- The lexer pushes events into a statically dispatched `Sink`; there is no
  token array. `Sink::mode` exposes `LexMode::Verbatim` for `v-pre` to its
  consumer. Armature's parser currently answers it; the L1 surface tree does
  not yet use the mode.
- Lex errors are an L1 enum (`LexErrorCode`). The native L1 surface tree uses
  them directly; legacy keeps its own `ErrorCode` and tokenizer until #6880.
- Directive-name decomposition is the dialect hook `DirectiveSyntax`. It
  returns spans only (`modifiers` is one span over the `.a.b` run), so it
  allocates nothing.
- `vize_l1::container` holds `ContainerFormat` and the lossless block records;
  `container::vue` is the first format.

## Native tokenizer ownership (#6835, partial)

- L1 owns `markup::lex::Lexer<P, S>` and `markup::entity` for its native
  surface tree. L1 has no legacy dependency; its two dependency allowlist
  entries are removed. The resident tier records native `LexErrorCode`
  findings without converting through relief.
- The existing `vize_armature::tokenizer` and compiler/parser consumers remain
  on their original path. `vize_relief` keeps its own error codes. The native
  lexer and legacy tokenizer temporarily coexist, because compiler fix-history
  issue #6880 is still open. No legacy product parser path changes in this PR.
- `LexErrorCode` owns its messages for native L1. Document and component
  profiles, including the Vue 1 raw-interpolation option, are native lexer
  capabilities; no legacy product uses them yet.
- The native L1 tree still needs its `v-pre` mode wired to the sink, and
  legacy tokenizer replacement needs #6880 to close first. These are
  unfinished parts of #6835, which stays open.

TODO: implement `VueDirectives` (#6836) and the SFC split (#6837). The
`v-pre` switch is still answered by armature's parser; the L1 surface tree
does not use it yet.

## Delivery order for the existing tokenizer PR

The owner explicitly queued #7038 on 2026-09-28 while #6832 remained open.
Deliver the independent native L1 lexer ownership as partial #6835, while
keeping #6832 and #6880 open for their remaining work. This scoped delivery
exception does not change the issue order for new Stage 1 work. The merge
queue's instruction-count budget remains strict and must pass before merge.

The earlier #7043 → #7037 → #7038 PR stack reflected the former live compiler
integration. With that integration deferred, #7038 has no source dependency on
those PRs and can target `main` directly. This supersedes the stack plan in
[#6835](https://github.com/ubugeeei-prod/vize/issues/6835#issuecomment-5867058001).
