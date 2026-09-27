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

## Tokenizer move (#6835)

- The armature tokenizer now lives in `vize_l1::markup::lex` as
  `Lexer<P, S>`; entity decoding lives in `markup::entity`. `vize_armature`
  and `vize_relief` depend on `vize_l1`; `vize_l1` has no legacy dependency,
  and its two allowlist entries are removed.
- `LexErrorCode` owns its messages. Relief converts with
  `From<LexErrorCode> for ErrorCode`, and a relief test pins that both
  messages stay identical, so legacy output is unchanged.
- Armature's document mode instantiates `Lexer<Document, _>`; the SFC path
  instantiates `Lexer<Component, _>`. Vue 1 `{{{` is the plain
  `LexOptions::raw_interpolation` switch.
- The never-entered `InSFCRootTagName` and `InSpecialComment` states are
  deleted.
- Armature's public `Tokenizer`/`Callbacks` API is replaced by re-exports of
  the L1 names; the next release needs the matching version bump.

TODO: implement `VueDirectives` (#6836) and the SFC split (#6837). The
`v-pre` switch is still answered by armature's parser; the L1 surface tree
does not use it yet.
