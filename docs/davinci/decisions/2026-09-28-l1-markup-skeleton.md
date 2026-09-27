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
  token array. `v-pre` is `LexMode::Verbatim`, answered by the L1 tree builder
  through `Sink::mode`.
- Lex errors are an L1 enum (`LexErrorCode`). Legacy maps them onto its own
  `ErrorCode` at the armature boundary, so no level crate depends on relief.
- Directive-name decomposition is the dialect hook `DirectiveSyntax`. It
  returns spans only (`modifiers` is one span over the `.a.b` run), so it
  allocates nothing.
- `vize_l1::container` holds `ContainerFormat` and the lossless block records;
  `container::vue` is the first format.

TODO: move the armature tokenizer and entity decoder into `markup` and invert
the dependency (#6835), implement `VueDirectives` (#6836) and the SFC split
(#6837). Unfinished bodies are `todo!()` under module-level expectations; no
product path reaches them.
