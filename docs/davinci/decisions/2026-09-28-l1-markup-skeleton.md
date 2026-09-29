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
- Lex errors are an L1 enum (`LexErrorCode`) for the new lexer. Existing
  surface-tree and compiler consumers retain legacy `ErrorCode` until #6880.
- Directive-name decomposition is the dialect hook `DirectiveSyntax`. It
  returns spans only (`modifiers` is one span over the `.a.b` run), so it
  allocates nothing.
- `vize_l1::container` holds `ContainerFormat` and the lossless block records;
  `container::vue` is the first format.

## Native tokenizer ownership (#6835, partial)

- L1 owns implementations of `markup::lex::Lexer<P, S>` and `markup::entity`
  behind the `native-markup-lex` feature. L1 unit tests compile and exercise
  them. Default builds retain the prior skeleton API and do not link the new
  lexer into product compiler binaries. No existing surface tree or product
  parser calls the new implementation yet.
- In the #6835 source-relocation slice, `vize_l1::parse` calls the moved
  L1-owned compatibility tokenizer. Armature re-exports that same code for
  Atelier's production parser. The normal L1→Armature edge is gone, so the
  #6831 dependency allowlist drops its corresponding permission for both
  `vize_l1` and `vize_l1_to_l2`. The L1→Relief vocabulary permission remains.
  No product parser uses the native lexer yet.
- The new lexer uses L1 `LexErrorCode`. Document and component profiles,
  including Vue 1 raw interpolation, are capabilities of the new lexer only.
  Existing products retain their own error codes and parsing behavior.
- The native lexer's `v-pre` mode still needs a consumer in the #6836 child;
  replacing the existing production tokenizer requires compiler fix-history
  issue #6880 to close. These are unfinished parts of #6835, which stays open.
- The opt-in feature is required while thin-LTO layout changes from the
  otherwise-unused implementation exceed current strict instruction ceilings
  in Atelier fused compile benchmarks. Main at `75b87e7a` passes the same-base
  control run, and no instruction budget is raised.

TODO: implement `VueDirectives` (#6836) and the SFC split (#6837). The
`v-pre` switch is still answered by armature's parser on the product route; the
native lexer is not used by the L1 surface tree yet.

## Delivery order for the existing tokenizer PR

The owner explicitly queued #7038 on 2026-09-28 while #6832 remained open.
Deliver the independent, opt-in native L1 lexer as partial #6835, while
keeping #6832 and #6880 open for their remaining work. This scoped delivery
exception does not change the issue order for new Stage 1 work. The merge
queue's instruction-count budget remains strict and must pass before merge.

The earlier #7043 → #7037 → #7038 PR stack reflected the former live compiler
integration. With that integration deferred, #7038 has no source dependency on
those PRs and can target `main` directly. This supersedes the stack plan in
[#6835](https://github.com/ubugeeei-prod/vize/issues/6835#issuecomment-5867058001).

## Tokenizer source relocation (#6835)

The next source-ownership slice moves Armature's existing tokenizer files as a
move-only commit into `vize_l1::markup::lex::compat`. Armature re-exports that
same implementation at its old public path, and L1's surface parser calls the
L1-owned path directly. Thus `vize_l1` loses its normal Armature dependency and
`vize_armature` depends on L1. The compiler still calls its original parser;
neither this move nor the crate edge counts as a native product route switch.

The `compat` module preserves the existing tokenizer's events, diagnostics,
entity behavior and recovery while the profile-generic `Lexer<P, S>` remains
opt-in. A source move is distinct from replacing the parser or claiming native
acceptance. The opt-in `native-lex-parity` test compares every event and error
for 42 surface fixtures, their UTF-8 truncations, and targeted component and
document inputs. The generic lexer now matches the preserved tokenizer for
self-closing whitespace (`<x / >`). A separate `&fjlig;` fixture records that
the preserved tokenizer emits only `f`, while L1 emits the complete `fj`
expansion. L1 `Sink` receives the complete decoded value and authored byte
span in one callback; its scalar callback defaults retain existing native
consumers. The feature-gated `CompatSink` maps L1 lexer events into Armature
callbacks, including the first-scalar legacy entity rule and parser-selected
lexical mode. Differential tests compare 42 fixture ASTs and diagnostics,
event/error parity with UTF-8 truncations, Document mode, Vue 1 raw
interpolation, custom delimiters, and experimental in-tag comments. PR and
merge-queue Actions run these feature-enabled tests; default archives do not.
The #6836 child verifies `v-pre` callbacks. The production parser still uses
the moved compatibility tokenizer. TODO (#6835): run the full legacy output
corpus and instruction-count gate before switching production to the shared
lexer, then remove the duplicate state machine after the #6880 product-route
prerequisite closes. TODO (#6835):
move the remaining `Namespace`/`ErrorCode`
vocabulary out of Relief to remove L1's other direct legacy dependency. The
compiler product route remains gated by #6880.

### Published Armature tokenizer API at v0.429.2

The v0.429.1 `vize_armature` API exposed `Tokenizer`, `Callbacks`, `State`,
`QuoteType`, three character helpers, and 35 `char_codes` constants from both
`vize_armature::tokenizer` and the crate root. The move retains these paths as
re-exports of the same L1 definitions, including the callback trait and all six
public `Tokenizer` methods. Recreating the types in Armature would split their
identity and reverse the L1 ownership decision.

The release check with cargo-semver-checks 0.47.0 and Rust 1.95.0 reports six
`*_missing` lint families for those re-exports, even when they are explicit
named imports with `#[doc(inline)]`. This is the upstream
[cross-crate re-export false positive](https://github.com/obi1kenobi/cargo-semver-checks/issues/355).
The release script accepts only the exact old tokenizer names and duplicate
root/module paths observed in that failure. Any new lint, missing or extra
item, unexpected baseline source path, tool error, or other crate failure still
fails. Its external-consumer witness runs one source against the published
0.429.1 crate and the candidate path crate. That source implements the old
callback trait, calls its default methods and every public tokenizer method,
checks exhaustive matches and discriminants for all 35 state variants and four
quote variants, pins all 35 byte constants to published literals, and assigns
the old module types to the old crate-root types. The witness checks Rust
compatibility that rustdoc JSON cannot represent across crates. It does not
claim a generic L1 lexer product switch or substitute for the full output and
instruction gates.
Exact release Actions and protected validation remain required before release.
