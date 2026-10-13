# Selected next-minor CSS AST provider migration

Status: the maintainer explicitly selected the documented next-minor type
migration on2026-10-12. The source is now prepared; genuine12-control compilation,
full/protected qualification and publication remain required. The historical
unselected proposal and its full source/receipt bodies remain unchanged.

The checked CSS engine fork in #8709 changes the nominal Rust AST provider.
Existing code that passes upstream `lightningcss::stylesheet::StyleSheet` to
Patina, uses explicitly typed upstream visitors, or implements `CssRule::check`
with upstream types no longer compiles (E0308, E0631, E0053). The authenticated
original nine packets retain these three failures and six working controls.

## Selected Rust migration

Re-export Patina's actual provider without another parse, conversion, or runtime
adapter:

```rust
/// CSS AST provider used by [`ParsedStyleSheet`] and [`rules::css::CssRule`].
pub use lightningcss as css_ast;
```

External rules should import the AST through the Patina version they implement:

```diff
-use lightningcss::stylesheet::{ParserOptions, StyleSheet};
+use vize_patina::css_ast::stylesheet::{ParserOptions, StyleSheet};

-use lightningcss::rules::{font_face::FontFaceRule, style::StyleRule};
+use vize_patina::css_ast::rules::{font_face::FontFaceRule, style::StyleRule};

-use lightningcss::stylesheet::StyleSheet;
+use vize_patina::css_ast::stylesheet::StyleSheet;
```

The three complete migrated examples change only these import prefixes. Their
source inputs, explicit visitor types, custom trait body, call counts, ranges,
and runtime assertions remain byte-exact after reversing that namespace change.
The new consumer manifests retain the original upstream dependency, proving
that selecting Patina's namespace is deliberate rather than package unification.
Applications can remove that dependency separately when nothing else uses it;
that dependency cleanup is outside these source-only comparison controls.

This is an intentional public Rust API migration in a next minor release.
It does not make the upstream and fork types interchangeable or repair existing
imports automatically. The next minor release must state this Rust type migration
in its release notes and link this complete import example. The current urgent
dd188 third-party release is separate and receives no CSS source or API credit.

## Complete compilation controls

Each row runs all three whole contracts: new/raw, typed visitors, and external
trait implementation. Keep Cargo.toml, Cargo.lock, full metadata, stdout,
stderr, exit status, source hashes, and actual source/compiler identities.

| Plane                      | Body                 | Required exit              | Evidence                           |
| -------------------------- | -------------------- | -------------------------- | ---------------------------------- |
| Baseline upstream          | Original             | 0 x 3                      | Original observed packets retained |
| Candidate upstream         | Original             | 101 x 3; E0308/E0631/E0053 | Original failures retained         |
| Candidate fork alias       | Original             | 0 x 3                      | Original observed packets retained |
| Candidate Patina namespace | Import-only migrated | 0 x 3                      | Observed on `177d9f1` / `5b2622f`  |

The selected migration requires all twelve controls on the exact
new source through the existing Actions producer. Expected original failures
are evidence of the documented break, not compatibility acceptance. The new
three successes must execute every unchanged runtime assertion; Cargo check
alone cannot establish the custom-rule call or complete lint result.

Keep all original 99 dev/release consumer observations and 66 + 6 whole Atelier
parse/compile/print and healthy-recovery routes, including all nine repaired
markerless-list results. Keep the import-order obligations and full gates.
The selected migration retains every check and performance budget, introduces
no duplicate parse or AST projection, and preserves all original corpus bytes.
Source-only controls supply no NAPI or registry qualification.

The Vize release graph still needs a fresh full dependency/feature closure and
unwaived audit with the checked rkyv 0.8 provider. The frozen original external
consumers enable their own upstream defaults and rkyv 0.7; that separate graph
cannot serve as Vize's release dependency qualification.

## Recorded decision and remaining checks

The maintainer selected the next-minor migration using this additive namespace.
A transparent full AST adapter is not selected; no extra parse, AST copy or
projection is added. Original upstream users still observe the three documented
compile failures, while the three namespace-only bodies must compile and execute
all unchanged assertions. This is a selected API migration, never transparent
compatibility acceptance.

The existing issue and foundation record carry the selected policy in the same
change. Draft status remains until exact-source checks pass; protected queue,
actual merge, fork publication/bootstrap and the later minor release still need
terminal proof. Root remains the sole release operator. No acceptance transfers
from an earlier source, the current third-party cut, or source-only controls.

The first combined source `177d9f1` reached the real focused compiler jobs,
while normal Check [38206418573](https://github.com/ubugeeei-prod/vize/actions/runs/38206418573)
refused the misplaced re-export under rustfmt and two stale generated consumer
inventory rows. The successor moves the same documented re-export into its
formatter-selected position, pins the complete resulting library body, and
regenerates those two rows from the unchanged inventory generator. The original
failure remains evidence; all contracts, fixtures and checks are preserved.

Focused [38206418446](https://github.com/ubugeeei-prod/vize/actions/runs/38206418446)
completed all five active jobs successfully on physical source `5b2622f`
for proposal `177d9f1`. The authenticated 102-member public-types archive retains
all twelve full bodies, manifests, locks, metadata, compiler and raw process
outputs: six original and three namespace-only bodies actually ran successfully,
and the three original nominal failures remain. The unchanged SemVer producer
returned zero. This is source evidence for that exact proposal; the formatting
successor, ordinary full checks, native/registry consumers and actual protected
merge still require their own proof.
