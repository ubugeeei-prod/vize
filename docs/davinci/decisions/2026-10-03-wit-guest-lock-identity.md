# Standalone WIT guest lock identity

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Actual main `b2d0e510c2541c1252f628f515e7a9546c311541` records workspace
`vize_guest` version `0.430.0`, but the expression, output and typed-expression
standalone guest locks still record the path package as `0.429.2`.
[Davinci Contracts run 37058009569](https://github.com/ubugeeei-prod/vize/actions/runs/37058009569/job/111007372851)
fails all three expression-world cases before executing their guest because
the existing build command correctly uses `--locked`.

The actual pinned Cargo 1.98.0 resolver updates only `vize_guest` in each of
these three standalone locks with `cargo update --offline --manifest-path
<guest>/Cargo.toml -p vize_guest --precise 0.430.0`. Every other lock byte,
registry version, checksum and dependency edge is unchanged. The same command
with `--locked` accepts all three corrected graphs. Restoring the authentic
stale expression lock reproduces exit 101 under the same locked resolver.
The resolver uses isolated
temporary state and existing registry metadata; it does not build a guest or
run the host tests.

Guest manifests, released WIT 0.1.2 fixtures, SDK bytes, goldens, compiler code,
workflow commands and instruction ceilings are unchanged. Fresh exact-head
Davinci Contracts must execute the real guests in both hosting modes, including
typed expressions, legacy SDK, packed SDK, resource limits and contract policy.
Required/full checks, fuzz, all 100 instruction probes and protected queue
acceptance remain separate requirements; no execution or merge credit follows
from the lock resolver proof alone.

## Typed-expression guest after the 0.430.1 release

Fresh main `62d3926a4070356deb48c64f2d37a218d9f3c4b8` inherits workspace
`vize_guest` version `0.430.1` from release commit `a2fc9c9e`, while the typed
expression guest alone still locks that local package at `0.430.0`.
[Actual-main Contracts run 37123702948](https://github.com/ubugeeei-prod/vize/actions/runs/37123702948/job/111204690365)
fails all three typed-expression cases before guest execution under the
unchanged `--locked` build command. This is a separate regression from the
historical 0.430.0 repair and from native Document's protected validation.

The original workspace lock SHA-256 is
`8a5ebbdcdb8b670f796526a8c76b5750a113a18dcac54094fbe5f227a3558d51`
(526 packages). The original typed guest lock is
`2e737da61e194d3edd4ae774ca14834efd14861bacfd76adf6d18efff53f0708`
(39 packages). Its entire Vize path dependency union is the guest's direct
`davinci/vize_guest` dependency, selecting `typed-expression` and the default
runtime. The SDK depends only on the existing exact `dlmalloc 0.2.14` and
`wit-bindgen 0.62.0` registry declarations; no implementation crate is added.

Actual Cargo `1.98.0 (797e8a9bc 2026-08-05)` produces exactly one lock line
with `cargo update --offline --manifest-path
 davinci/vize_extension_host/tests/guests/typed-expression-echo/Cargo.toml
-p vize_guest --precise 0.430.1`: only the local SDK version changes.
The corrected lock SHA-256 is
`fb15b90ee266e91aa08f360272706a2f4811e06562dab34460befdf9730f764f`.
All 37 registry package blocks, versions, checksums and edges, the workspace
lock and both manifests remain byte-identical. The same metadata command with
`--locked` accepts the corrected graph; restoring the authentic original
lock reproduces exit 101, then the corrected bytes are restored. These
metadata-only operations create no target tree and execute no compiler,
Clippy, guest or host tests.

The existing release updater's standalone SDK loop omitted this manifest.
Add just `typed-expression-echo/Cargo.toml` to that loop alongside the
expression, output and Volt guests so the same release drift is prevented.
Preserve its offline update/staging flow, all locked guest build commands,
released SDK/WIT fixtures, source APIs, goldens, pinned upstream versions and
instruction ceilings. Fresh exact-head Actions must execute actual TS-48
Wasm guests in both hosting modes; protected full/instruction validation and
actual merge remain pending. Metadata acceptance supplies no Wasm, native
Document, product or release completion credit.
