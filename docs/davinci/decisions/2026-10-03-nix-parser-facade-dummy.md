# Nix parser facade dependency preparation (2026-10-03)

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The parser publication change in
[#7501](https://github.com/ubugeeei-prod/vize/pull/7501) gives the maintained
implementation the package identity `vize_oxc_parser`. Upstream OXC crates
consume its local `oxc_parser` compatibility facade. The facade re-exports
the same implementation; it contains no second parser.

Crane's manifest-only dependency build already restored the implementation
source after generating local dummy crates. It also needs the real facade.
Otherwise the facade's empty dummy library has no `Parser`, `ParseOptions`
or `ParserReturn`, and `oxc_formatter` fails before the Vize package builds.
This was observed in the full Checks for
[#7512](https://github.com/ubugeeei-prod/vize/actions/runs/37110618933) and
[#7507](https://github.com/ubugeeei-prod/vize/actions/runs/37110637936).

Restore both vendored parser roots in the existing `extraDummyScript`.
Keep the remainder of the workspace manifest-only. No parser code,
dependency versions, features, product routes or instruction budgets change.
Fresh Actions `nix flake check` must prove the dependency and package builds;
source-head PR checks and protected-queue validation remain separate gates.
