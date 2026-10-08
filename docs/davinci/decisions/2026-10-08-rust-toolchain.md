# Rust 1.99.0 development and production toolchain

The maintainer requests the latest stable Rust. The official [1.99.0 announcement](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/) dates the release to October 1, 2026. The official channel manifest and checksum agree on SHA256 `ce6dddc886364f8d786514771212cebe9b731ba82d6b859951c6b0ccc516b6a2`, with rustc `1.99.0 (b940084d7 2026-09-28)`.

Pin repository development and production CI builds to 1.99.0. Move the actual archive producer/consumer envelope and its active harness fixtures together; reject archives claiming the previous compiler. Cache identity already includes the toolchain file and compiler metadata. Keep supported workspace MSRV 1.95.0, its semver and Nix lanes, historical receipts and frozen differential oracles unchanged.

The #6189 historical SFC parse replay and the isolated allocation-ownership experiment retain their explicit 1.98.0 contracts, workflows and archive validators. They authenticate old compiler/source experiments and grant no current 1.99.0 performance or production credit. Generic synthetic cache, typechecker and instruction-count samples remain historical or version-opaque controls where their contracts do not select the current compiler.

Install 1.99.0 alongside existing local toolchains; change no global rustup default. This isolated branch genuinely starts from signed main `eed471b4424b922b878bc35cac765dc7274e5736`. The initial toolchain change preserves dependencies, frozen inputs, expected compiler output, instruction ceilings and allocation budgets. Current-source corrections keep those contracts unchanged.

Actual 1.99.0 formatting, compilation, Clippy, source/runtime/corpus and protected instruction-count gates must pass before delivery. If compiler-generated code exceeds an existing ceiling, optimize the affected implementation with controls; never raise the ceiling. Prior 1.98.0 successes grant no changed-toolchain acceptance. Release machinery and installed artifacts need fresh qualification after the actual protected merge.

Tracking issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830). Exact-head Actions, protected merge and release qualification remain pending.

The first actual 1.99.0 source `9515b781ffb466720fd23e1d5270bc08cf02bf2a` stopped broad/native Clippy on redundant iterator `must_use` attributes. Remove those two attributes and the two remaining identical opaque-iterator attributes found in downstream source: the returned iterator trait already carries the same requirement. A minimal 1.99.0 Clippy reproduction confirms this applies to opaque iterator returns; concrete custom iterator returns remain unchanged. The actual preview-producer workflow control also moves its two escaped version assertions to 1.99.0 without changing its build/publication checks. The protected instruction recipe measured the same four failures in all three repetitions: medium parser 243,391 versus 243,340; Vapor interpolation lowering 520,095 versus 518,158; unobserved fact query 11,455 versus 11,290; observed fact query 11,306 versus 11,141. Preserve every ceiling and original benchmark input.

Fact demand iteration now visits its present identities directly instead of scanning all 64 positions for every closure, pending-demand check and invalidation. Allocation storage, producer counts, observation policy and ascending identity order remain unchanged. Independent enumeration over empty, full, alternating, all single-bit and 1,024 generated masks plus exhaustion controls checks the complete identity space. A small Rust 1.99.0 wrapper compiled these actual foundation modules and passed all 14 controls and Clippy with warnings denied; whole repository and protected measurements require fresh Actions and receive no local qualification credit.

Vapor placeholder planning removes an intermediate flattened list while preserving existing node classifications, transparent-template flattening, rendered-sibling visibility and final authored order. Five nested/template/comment/text/trailing-block controls accompany the source; the [Vapor lowering record](./2026-10-08-rust199-vapor-lowering.md) retains actual old hotspots and the fresh instruction/allocation qualification obligation. No new pipeline stage, input or budget waiver is added.

The medium-parser correction classifies whitespace-only text using the exact five Vue ASCII bytes. Empty text, vertical-tab and excluded Unicode behavior stay the same; original Unicode borrowing/spans and whitespace between elements are frozen for both Condense/Preserve and both legacy-line-break modes. Existing mixed-text, preformatted and textarea processing is unchanged. The foundation controls also compile and pass on supported Rust 1.95.0. Whole source, compiler-output parity and measured instruction savings remain pending fresh 1.99.0 Actions.

The native-feature 1.99.0 build identifies the newly deprecated atomic
`fetch_update` in navigation capacity reservation. Retain MSRV 1.95.0 with an
equivalent `compare_exchange_weak` loop: initial/failed observations use Acquire,
success uses AcqRel, capacity refusal remains at 16, and the owned slot releases
on the existing worker exit. No newer alias or deprecation suppression is used.
Existing owner-exit/capacity/native navigation controls remain mandatory.

Compose genuine signed main `8f01ff9c320f4b474b410efe7d66baed462df7aa`, including
the delivered licensed n8n compiler fixture and existing Vapor fixes. Keep all
33 immutable compiler capture drivers and their baseline/repair pins unchanged.
The n8n custody job explicitly selects 1.99.0 for both genuine isolated source
phases, including the frozen baseline whose source file still records 1.98.0.
Within its existing capture step, require complete actual rustc/Cargo receipts
to agree, pin the genuine 1.99.0 compiler identity and require distinct producer
binary hashes. This changes the common execution compiler, never the archived
baseline source or observed metadata. Fresh paired output/runtime and the entire
unchanged corpus must pass on this source and compiler.

The maintainer explicitly authorizes the necessary active instruction-methodology
transition from 1.98.0 to 1.99.0. Preserve all 100 ordinary and four formatter
numeric ceilings, original fixture paths/SHA256/window, Callgrind/libc/ci-opt
settings, historical provenance and every other methodology field. Permit only
this exact forward compiler transition aligned with the actual source toolchain;
reject unrelated, backward, input, inventory, source-version or ceiling changes.
Historical e24 synthetic source `1a3996963af208c13cd732420bdee586df88bf7a` has
all 104 cases below their original ceilings in all three authenticated repeats;
this is measured numerical evidence, not successful migration or new-source
qualification. Fresh source/methodology/instruction gates remain required.

Fresh source `3c8ab916` passed the strict instruction gate and the complete
same-compiler n8n custody/runtime job. Its real hotspot CLI test correctly failed
because the synthetic report claimed an unrelated source revision. Derive only
that test report's source from the actual checkout, explicitly reject its old
synthetic revision first, and retain the original over-ceiling, malformed-dump
and under-ceiling laws. This corrects test context; no source guard or ceiling
changes. The separate authored Vapor raw-template premise needs its own literal
AST/original-algorithm proof, and all fresh source gates remain required.

Fresh `1a3a590a` paired n8n passes, but native job `112934206269` refuses
the missing `lsp_data_aria_attributes_cli` target before execution: merged PR
workflow configuration includes the newly delivered target while its exact
source checkout still descends from older main. Genuinely merge signed current
main `cd7adbdd1bcf6dd8bc92734a9a4108da6eb3284f`, preserving all incoming product,
formatter/vendor/dependency and original fixture bytes alongside these owned
changes. Keep the mandatory target and full source/native/corpus/instruction
gates; adding no missing-target skip or test waiver.

Protected candidate `ad413976a3bb1de21852174e0bd2fb0416ab737b` authentically
passes n8n compiler/runtime/authored navigation, all 104 original ceilings and
the full zero-divergence corpus. Its four Rust workers still fail the incoming
typechecker observation verifier: a hardcoded 1.98 compiler assertion rejects
the genuine 1.99 producer. Preserve that failed candidate as historical proof;
neither its passing subset nor source `fbf3671a34` completes protected delivery.

Genuinely incorporate signed current main
`f3ed2ee49cd50619830b3db6a2ae2d946f18ec5e`, preserving all incoming source,
original fixtures and the delivered authored n8n workflow alongside the owned
Rust changes. Worker and aggregate receipt validation now derives its compiler
from `rust-toolchain.toml` at the exact committed source revision. Only reviewed
source pins 1.98 and 1.99 are supported; actual compiler banners must match that
source pin. Missing, stale, malformed and unsupported source/compiler stamps
fail, and an uncommitted working-tree pin cannot change the authority. Keep all
other receipt/source/tree/executable/JUnit/observation/four-worker guards exact.
The current synthetic accounting producer follows its committed source pin;
all stored historical 1.98 fixtures and metadata stay unchanged. Original plus
new transport controls pass locally, without runtime acceptance credit. Fresh
whole-source, all four workers, original 104 ceilings, native, n8n and protected
corpus gates remain mandatory before actual delivery.

Before publication, also incorporate signed actual main
`c544ca63ffb2dd14fe58b796ccea79b1d518a156`. Its incoming changes deliver
documentation navigation; preserve all of them and every incoming canonical
clause. The reviewed observer compiler correction remains unchanged.

The final publication composition also genuinely includes signed latest main
`7557c93594916e797bd6defac459c88235221547` and its delivered caller-retirement
source, fixture, workflow and inventory changes. Every incoming byte and
canonical clause is retained; original 104 registry/cap/input bytes and all 33
compiler custody drivers remain identical to the owned source.

Exact source `55c8925bca` passed all four required checks, complete source,
16,765 Rust tests, original 104 ceilings, native phases, paired n8n custody/runtime
and the full zero-divergence corpus. A pre-admission merge review still finds an
adjacent canonical conflict with queued Art binding history. Relocate only this
complete unmerged Rust suffix from row 324 to the existing Rust cache paragraph
at row 333; preserve both original paragraphs, every prior decision and all
350 rows. Genuinely incorporate signed actual main
`fe94b4d0dfb018e3a10fbde7b083823c3eeb7cc3`, retaining all incoming product and
original fixture bytes. This docs placement and faithful source union require
fresh mandatory Actions and protected delivery; historical 55c success cannot
qualify the successor or installed release. No separate measurement dispatch
or compiler, input, capture driver, recipe, ceiling or gate change follows.

The faithful composition also includes signed actual Art delivery
`67e537cbec83b7971767f7d28d570a34f1a5afcb`; both complete adjacent Art/Page
paragraphs match that main byte for byte. The sole owned canonical suffix now
occupies row 333, unchanged. All 33 capture drivers, four original cap/input
registries and current compiler-observation guards retain exact 55c bytes.

The CI Stack actually delivers signed commits
`86418231fdf6c59752f1239c5bbdd66b22f5f9ba` and
`3e0745b6277c00176178de6e9c15de2ee500a258` before Rust admission. Genuinely
merge that actual main, retain its durable official action pins and every
archive/source/feature/observation guard, and change only the new live
`pr-rust-differential.yml` compiler selection from 1.98.0 to 1.99.0. Existing
producer/source selector pins remain 1.99.0; synthetic and historical 1.98
fixtures plus MSRV 1.95 stay exact. Source 2e fully passed all required gates
but was never admitted with the known stale incoming consumer. The composed
source requires fresh mandatory Actions and actual protected delivery.

Cheap composed controls identify the new sibling harness still simulating
1.98 as its current producer: the unchanged strict 1.99 archive constructor
correctly refuses before either feature law executes. Derive only that active
simulated banner from the declared sibling pin, retain the old 1.98 banner as
an explicit refused envelope, and keep every foreign/corrupt archive and full
feature-failure control intact. No stored historical fixture, archive validator,
production behavior or numerical ceiling changes.

## Nix development and MSRV packaging

The actual 177933 main checkout still pinned rust-overlay revision
`61ec6a4fc56fe0c2b863f7b3eaba07b6664697d9` (2026-05-16), whose stable manifests
stop at 1.95.0. Update only that input's locked metadata to the first official
[1.99 manifest revision](https://github.com/oxalica/rust-overlay/blob/dcee1adabb61484343af863501d2e3d91ef51f72/manifests/stable/1.99.0.nix),
`dcee1adabb61484343af863501d2e3d91ef51f72` (2026-10-01). Keep every other input,
its original reference and the nixpkgs follow edge unchanged.

A private `rustDevToolchain` reads the exact `rust-toolchain.toml` through
rust-overlay's `fromRustupToolchainFile`. Only the development shell and its
`RUST_SRC_PATH` use it. Contributor Rust, its declared components and both
Wasm targets therefore follow the source pin, currently 1.99.0. The existing
package `rustToolchain` stays at MSRV 1.95.0 with the same extensions and target;
Crane and the package build retain that compiler. Preserve all four systems
in `tools/nix/systems.nix`, including x86_64-darwin.

The observed host `origin` Rust launcher separately resolves its own immutable
Nix-store overlay, whose latest stable is 1.98.1. It does not read Vize's
`flake.lock`; this change cannot refresh that external launcher. The already
installed `~/.cargo/bin/rustfmt +1.99.0` reports 1.10.0/b940084d7 and provides a
local formatter route without changing a global profile or Rust default.

Validation uses evaluation only, with import-from-derivation disabled, to
inspect development and package compiler dependencies on every supported
system. No local Nix build, source compiler build or release hold is added.
Fresh exact-source Actions and actual protected delivery remain required;
the ongoing release from the earlier actual main keeps its independent source
pin and qualification.
