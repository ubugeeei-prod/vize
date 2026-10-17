# Preserve Rust coverage while narrowing proven default targets

The Typechecker job in Check `38992862530`, source merge
`1e626a96260a243c551055737e2bf239274de672`, spent 249 seconds compiling and
29.590 seconds running 2,732 tests. The earlier `ccc1b260` run spent 52.74
seconds compiling and 30.802 seconds running 2,726 tests. The six additions
are the main #9314 import-fact tests. Both mounted the same read-only sticky
namespace; that does not prove identical cache snapshots.

The new owned Cargo output shows 1,222 test binaries; nextest excluded 1,048
from this cohort. Its two `sha2` units took only 0.47/0.50 seconds themselves.
The changed dependency fingerprints and intervening Rust sources accompanied
broad recompilation. This is not evidence that hashing took 196 extra seconds,
nor a controlled attribution of the cross-run difference to one change.

## Bounded default-feature proof

Keep every selected package root, shared unit target, test filter, runtime
profile, cancellation path, and Cargo stdout capture. Only the full-workspace
ordinary PR invocation with the existing exact default-feature option envelope
can use the additional proof. Subsets, feature flags, unknown options, different
events/profiles, unavailable offline metadata, and inconsistent metadata retain
the existing conservative selection.

Cargo metadata resolves all workspace roots and dev dependencies. Its resolved
feature set precedes the target/host narrowing pass. Therefore absence can prove
an integration target inactive; presence alone cannot prove it active. A positive
proof requires each gate to be a direct default feature of its selected root.
There is no transitive feature resolver. Same-name targets must all be proven
active or all inactive; mixed/unknown states retain Cargo's default selection.
This follows the pinned Cargo 1.99.1
[metadata implementation](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/ops/cargo_metadata.rs),
[feature narrowing](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/resolver/features.rs),
and [documented metadata contract](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html).

Exact original/candidate projection against the same 44-root metadata:

| Cohort               | Before                                      | After                                        |
| -------------------- | ------------------------------------------- | -------------------------------------------- |
| Compiler             | Cargo defaults                              | Cargo defaults                               |
| Typechecker          | Cargo defaults                              | 172 named integration targets + shared units |
| Language server      | 54 named integration targets + shared units | unchanged                                    |
| Lint and format      | Cargo defaults                              | 235 named integration targets + shared units |
| CLI and integrations | Cargo defaults                              | Cargo defaults                               |

Canon's `empty_vue_star_exports` requires its direct default `native` feature;
`davinci_projection_differential` requires inactive `legacy-differential`.
Patina's `davinci_markup_differential` gate is also inactive. Compiler retains its
fallback for DOM's `text_shape_matrix` and `v_pre_freezes_its_subtree`; CLI retains
its fallback for Marquette's `contract_compat`. Their active transitive gates are
intentionally not inferred. No full build moves onto another cohort.

The required Clippy/doctest job additionally owns a normal `cargo build --examples`
with the same package roots, default features and `ci` profile, after doctests.
Cargo treats this as a dev-dependency build, matching default test compilation
of ordinary examples without turning them into test harnesses
([target modes](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/ops/cargo_compile/unit_generator.rs),
[dev dependencies](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/ops/cargo_compile/compile_filter.rs)).
This is an explicit coverage owner, not exactly-once compilation: the remaining
conservative defaults can also compile examples. Native Cargo JSON, fresh timing
HTML and fingerprints are retained through the existing build-evidence helper.
Merge-queue archives, scheduled/full Rust recipes, and trusted cache writers do
not change.

## Evidence and acceptance boundary

The static replay maps all 1,222 observed binaries and all 18,560 JUnit case
identities to their existing cohort, with no missing identity, and preserves all
30 observed normal example targets. The original/candidate argument vectors and
all 39 feature-gated targets are in the review receipt. A dependency-free native
Cargo probe also compares complete test identities and example modes/features,
including a dev-dependency feature and an inactive target that fails compilation
if accidentally selected. Negative controls cover ambiguous positives, duplicate
names, feature flags, subsets, malformed metadata and missing metadata.

These checks prove selection and recipe properties, not a wall-clock saving.
The next actual-head Actions run must compare all case identities, retained
example artifacts and timings before claiming the five-minute PR goal.
