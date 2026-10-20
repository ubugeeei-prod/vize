# Canon library filter scope

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The original outside-import editor and monorepo-alias commands select library
tests. Add `--lib` to those two commands in
`.github/workflows/typechecker-native-phases.yml`, retaining their complete
filters, `--locked`, `ci-opt` release inheritance, thin LTO, sixteen codegen
units and `--nocapture`. Every other native command and receipt stays exact.

## Authenticated current inventory

The existing protected [Check 38195691568](https://github.com/ubugeeei-prod/vize/actions/runs/38195691568)
at `5a9b8df91801ef550d65cf681e366c3c5494967d` succeeded. Its existing full
workspace archive producer, job `114637844226`, recorded the complete
`workspace-tests.json` in artifact `11723342452`. Downloading that 495,262-byte
timing artifact avoided downloading the 4.9 GB executable archive or running
another Cargo builder/list command.

- ZIP SHA256: `1681beb6622d5b5c7e5af971a9191d59ec2fa24fdf68731a1452ebbfcd999cfc`.
- Inventory SHA256: `c7c9af075a4d7754390420e65bdc86d65b9dd4a02628d40dba14732cac626c2d`.
- Checkout, event and archive receipts identify the same source SHA and tree
  `4c91772b647b107f2a047cc2d8cc92c73859595a`.
- Rust 1.99.0, nextest 0.9.146, Linux x64, workspace `ci` profile,
  `VIZE_TEST_REQUIRE_TSGO=1`, Nuxt iterations 100.
- Canon has one library target with 1,809 cases and 55 integration targets
  with 246 cases. Both filters select exactly one nonignored library case;
  all 55 integration targets have zero matches.
- The recorded Cargo timing units for Canon all enable `default/native`,
  matching the two native commands' default feature declaration. This
  corroborates registration, without borrowing `ci` binaries for `ci-opt`.

The selected full names are:

```text
corsa_bridge::outside_import_editor_tests::original_outside_import_preserves_types_in_native_editor
corsa_bridge::vue_document_package_config_tests::native_editor_resolves_package_alias_from_monorepo_root
```

Implementation base `0d63bc7e392b952351c9f2ebdc2d1725954f9606` has identical
Canon, Cargo, native-workflow and Rust-lexer bytes to that observed source.
Fresh integration base `dd188515f8ec3c51662b4f4d69adcd39c86a0907` retains
those same authority bytes and every incoming foundation clause.
Walking the 55 current integration roots follows 52 authored module edges
through 95 files and reproduces all 246 compiled case names, without missing
or extra names. The guard additionally reads the currently feature-gated
`davinci_projection_differential` target: conservative union becomes 56 roots,
96 files and 248 cases. Counts describe these sources; future benign additions
do not need to preserve these historical counts.

## Durable admission law

The existing native Node test glob and the ordinary tooling campaign run the
same guard. It discovers real Cargo test/bin and explicitly tested example
targets, follows literal external and inline Rust modules, and reads all
conditional branches. Known `test`, Unix, Windows and Linux conditions are
unioned, avoiding reliance on `ci` versus `ci-opt` debug assertions.
The current manifest inherits workspace edition2024, leaves automatic test/bin
discovery enabled and has no library override or build script. Automatic
examples/benches have no selected test harness unless explicitly enabled;
explicit tested targets enter the same discovery closure.

Any outside-library test whose qualified name contains either original filter
refuses admission. This includes matching module prefixes, suffixed function
names, new file/directory targets and explicit target paths. Unknown item
macros, procedural attributes, `cfg_attr`, includes, generators, custom
harnesses, escaped/cyclic/ambiguous modules and physical symlinks also refuse.
Qualified attribute heads and unsupported Unicode outside-library item names
refuse instead of being truncated by the existing ASCII lexer. Default non-mod
files retain their owned directory for inline paths; explicit `#[path]` files
use Rust's mod-rs sibling semantics, with both real/decoy paths covered.
Unknown proof makes Check red; restore the original broad commands or review
and extend the admission law before delivery.

Only compiler builtin derives and the actual serde provider are admitted.
The seven-crate serde derive lock closure binds version, registry, checksum
and dependency edges. Canon dependency shadowing, root patch/replace aliases,
their physical package manifests and Cargo source replacement cannot silently
change that provider. An intentional provider upgrade requires explicit
review of this finite provider tuple; product bodies are not hash-frozen.
Cargo configuration metadata includes the invocation directory, its ancestors
and effective Cargo home. Paths/patches/includes, lockfile redirection,
compiler/wrapper/runner and executable environment overrides refuse. Existing
platform linkers and the exact linker flag written by the unchanged Wild0.9.0
action remain admitted. Configuration contents are never executed or logged.
The successful original [native run38201692388](https://github.com/ubugeeei-prod/vize/actions/runs/38201692388)
at `0d50f9c2a7dbd44d32a92da95e2047f08967f78c`, job114656287259, records
`CARGO_HOME=/home/runner/.cargo` and the actual Wild target linker flag, with
no `RUSTFLAGS`, `RUSTC` or Rust compiler-wrapper environment override.
Its complete3,670,913-byte job log has SHA256
`e92ae5ab4c6b674c2dc995238f1486d907385fc525ad80a38d267d31082cd1f4`.
The unchanged [Wild action](https://github.com/wild-linker/action/blob/0bbbfa5df4380cab8e63cb8505a1ce65e1d10203/action.yml)
writes GNU/musl target configuration with clang and its installed linker path;
the positive configuration law admits that exact recipe and rejects changed
flags. These are existing environment observations, not optimized execution.

Rust's [test harness collector](https://github.com/rust-lang/rust/blob/master/compiler/rustc_builtin_macros/src/test_harness.rs)
recurses through module items when collecting named tests. The admission
parser handles authored item boundaries and never executes Rust source.
Normal function-body edits and additional matching library cases are allowed:
the unchanged substring filter continues to select every matching library
case. Both original library functions, their direct module paths, default
native feature and active test attributes remain mandatory.
The four original ancestry files also bind builtin test/cfg/path/std/core
namespace authority: foreign imports, aliases, reexports and opaque ancestral
items/attributes cannot replace literal `#[test]` with a procedural attribute.

## Complete native obligations

The source CLI outside-import campaign, original Program process-reuse and
editor process laws, native project retirement and all other original native
campaigns remain unchanged. Keep the five-file outside-import fixture, complete
diagnostic vectors, `editor-original-script` and `editor-monorepo-alias` files,
source/fixture/native package hashes, exact physical native binary and runtime
path checks. Keep ordinary full Check, protected workspace/feature corpora,
doctests and all manual/scheduled/release qualifications. No new pipeline
stage, Cargo builder, automatic list, manual capture or profile alias is added.

The historical native run `38179002901`, job `114587416121`, source
`13cd6c9cd1e5bd709875bcd355e04ce4501a538b`, emitted 56 harnesses for each
original command: one selected library case and 55 zero-match integration
harnesses. No doctest harness was emitted in those command segments; this
does not establish doctest execution. Its first broad compile took 165 seconds
and its second compile 0.09 seconds. These observations are not a measured
speedup, and avoided compilation can change later warm-build costs.

Rust1.99.0's Cargo submodule is
`5f94df4789f005f9a352888e8355ffc645b7ed0e`. Its
[test CLI](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/bin/cargo/commands/test.rs#L94-L99)
changes an unchanged positional `TESTNAME` without `--doc` to
[all test targets](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/ops/cargo_compile/compile_filter.rs#L170-L178),
using the `CompileFilter::Only` branch. Its
[target generator](https://github.com/rust-lang/cargo/blob/5f94df4789f005f9a352888e8355ffc645b7ed0e/src/ops/cargo_compile/unit_generator.rs#L390-L438)
does not add doctests in that branch with test intent. Thus future doc blocks
were already outside these two original filtered commands. Keep their exact
positional names/no `--doc` and the existing full doctest steps; `--lib` does
not remove selected documentation tests or claim they passed here.

## Validation and delivery

Run the existing cheap campaign with:

```sh
vp node --test tests/tooling/typechecker-native-canon-lib-scope.test.ts
```

Source laws must cover genuine body/library additions, both filters' integration
functions/prefixes/suffixes, new/explicit targets, modules, conditional union,
opaque generators, original-case removal/deactivation, derive drift and physical
source refusal. Require independent source review, fresh ordinary Check and
the original native Actions receipts, then full protected merge-group success
and actual merge. Hosted results and matched wall-time gain remain unclaimed
until those terminal receipts exist.

Local preflight passed all53 source/provider laws under Node24.14.0 in9.17s,
with zero skipped cases and zero lint warnings. This is static preparation;
it supplies no native runtime, protected merge or measured speedup credit.

The first [source Check38205223466](https://github.com/ubugeeei-prod/vize/actions/runs/38205223466)
at `19044ac5a88ac0ecd3a5bf5cba43ed0959a5ff23` refused23 floating Node test
registration promises in job114667109870. Await all existing registrations;
retain every53 law body, complete source/provider proof, zero-warning gate and
original native command. Focused type-aware lint and fresh exact-head source,
native and protected qualification are required after this correction.
