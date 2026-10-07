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
