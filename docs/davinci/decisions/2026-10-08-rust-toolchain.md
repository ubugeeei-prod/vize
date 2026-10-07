# Rust 1.99.0 development and production toolchain

The maintainer requests the latest stable Rust. The official [1.99.0 announcement](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/) dates the release to October 1, 2026. The official channel manifest and checksum agree on SHA256 `ce6dddc886364f8d786514771212cebe9b731ba82d6b859951c6b0ccc516b6a2`, with rustc `1.99.0 (b940084d7 2026-09-28)`.

Pin repository development and production CI builds to 1.99.0. Move the actual archive producer/consumer envelope and its active harness fixtures together; reject archives claiming the previous compiler. Cache identity already includes the toolchain file and compiler metadata. Keep supported workspace MSRV 1.95.0, its semver and Nix lanes, historical receipts and frozen differential oracles unchanged.

The #6189 historical SFC parse replay and the isolated allocation-ownership experiment retain their explicit 1.98.0 contracts, workflows and archive validators. They authenticate old compiler/source experiments and grant no current 1.99.0 performance or production credit. Generic synthetic cache, typechecker and instruction-count samples remain historical or version-opaque controls where their contracts do not select the current compiler.

Install 1.99.0 alongside existing local toolchains; change no global rustup default. This isolated branch genuinely starts from signed main `eed471b4424b922b878bc35cac765dc7274e5736`. No dependency, production implementation, frozen input, expected compiler output, instruction ceiling or allocation budget is changed.

Actual 1.99.0 formatting, compilation, Clippy, source/runtime/corpus and protected instruction-count gates must pass before delivery. If compiler-generated code exceeds an existing ceiling, optimize the affected implementation with controls; never raise the ceiling. Prior 1.98.0 successes grant no changed-toolchain acceptance. Release machinery and installed artifacts need fresh qualification after the actual protected merge.

Tracking issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830). Exact-head Actions, protected merge and release qualification remain pending.
