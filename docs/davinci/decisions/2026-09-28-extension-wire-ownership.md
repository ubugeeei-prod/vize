# Extension wire ownership

Tracked in [#6833](https://github.com/ubugeeei-prod/vize/issues/6833),
[#6834](https://github.com/ubugeeei-prod/vize/issues/6834) and
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

## Decision

Retire the separately named internal `vize_extension_contract` and
`vize_extension_host` packages by moving their actual responsibilities into the
levels and the product integration owner. The independent Rust guest SDK is the
approved [`vize_guest` external-product exception](./2026-09-28-guest-sdk-crate-axis.md);
do not add another internal package facade.

Begin with a real neutral-code move. The existing L0 workspace alias points to
`vize_carton`; its `extension::wire` module now owns the protocol constants,
capability, byte span, serialized page, severity/stage/part tags, source-block
record, guest errors and resource limits. `extension::handshake` owns the
complete negotiation algorithm. L0 acquires no dependency on a higher level.
The old contract imports the same types and functions for its remaining
acceptance and diagnostic conversion paths. Those imports are transitional;
they do not finish removing the old package names.

The WIT package identity, protocol/page versions, feature spellings, field and
variant order, serde attributes, resource limits, refusal order and messages
remain unchanged. The existing legacy `s1-page@1`/`s2-page@1` wire spellings
are compatibility material, not new internal level names. All guest SDK source,
WIT and released-version files remain byte-identical.

This is an ownership move, not an additional processing stage. Native
first-party producers must use typed level interfaces; this move does not
authorize serializing native IR between levels.

## Why the diagnostic boundary stays separate in this step

The current contract also contains native diagnostic conversion and producer
exemption lookup. Moving that file wholesale would make L0 depend on
`vize_l1_to_l2`, producing L0 → lowering → L0. Moving only the diagnostic DTOs
would also invalidate the existing `From` implementations under Rust's orphan
rule: neither diagnostic type would be owned by the remaining contract crate.

`Diagnostic`, `DiagnosticPart`, `Witness`, `LoweredBlock` and the
`InputDialectGuest` trait therefore remain with those conversions temporarily.
Their actual relocation must move or replace the conversion boundary in the
same change. This does not establish a permanent mixed-ownership contract.

## Feature gate

`extension` is an opt-in `vize_carton` feature, enabled only by the contract
crate. Per-stage instruction ceilings forbid any increase, and the ungated
module shifted legacy parse and compile counts by up to 0.8% in the merge
queue (code layout, not new work). Compiler-only graphs, including the
benchmark harness, therefore build the same L0 source as before.

## Remaining code work

- L1 owns surface-tree acceptance; L1→L2 owns Vue input acceptance and producer
  diagnostics; L2 owns expression facts and scope acceptance.
- L4 owns projection and output acceptance. Host transports move to the
  existing product integration owner rather than another level-named package.
- Physical L0 extraction must make its standalone guest feature genuinely
  defaults-off and `no_std`; current std-based Carton is not that package.
- Keep the five real guest source files, runtime ownership, WIT and version
  sources together in the independent `vize_guest` product. Preserve all
  published guest exchanges and the packed external component check.
- Delete each old package only after its last responsibility and dependent
  imports have moved. These names are still unfinished after this first step.

## Replay and verification

The first commit moves `handshake.rs` without changing its bytes. The second
integrates that module and splits the neutral records mechanically. On a
conflict, run `node tools/support/compat/davinci/move-extension-wire.mjs` on
current main and review its bounded result; the script refuses changed source
boundaries rather than copying an ancient whole file over current work.

Local checks cover exact moved record bodies, unchanged handshake body, all
remaining conversion bodies, unchanged manifests/dependency declarations,
WIT/version bytes, formatting, source limits and dependency metadata. They do
not prove compiled component ABI or performance. Fresh Actions must run the
existing negotiation/refusal, diagnostic conversion, page parity, legacy guest
exchange and feature tests before merging. The merge queue must retain the
unchanged instruction-count gates. Until then, compiled and runtime proof is
pending.
