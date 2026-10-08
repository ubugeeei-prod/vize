# Timing observer host ownership

The [paired #6834 decision](https://github.com/ubugeeei-prod/vize/issues/6834#issuecomment-6051794831) moves the existing `TimingObserver` into Carton because it owns
`Instant`, host error text and the legacy global profiler adapter. L0 retains
`WalkTiming`, the portable pass protocol and the complete six walk laws.

The move-only commit `3261ff2cc36e3f3e63346f781e2f3f0676cb585f` relocates the
complete 114-line implementation and original 135-line law without changing a
byte. Integration `f6f1e56ba2e6c7822b9df62bcc3f1c618d5f6792` changes only their
imports, exports and two existing normal CLI/Curator consumers. Both consumers
already depend on Carton. L0's dump and zero-cost laws use the same dev-only host
owner. No manifest, dependency, provider, pipeline or profiler behavior changes.
The implementation from `pub struct TimingObserver` onward remains byte-exact;
the original timing law changes only its owner import. The 79-line portable
implementation and complete six-law source remain byte-exact.

The original 400-input warm qualification admits only the authenticated complete
11-body move, including the removed old law, and both unchanged portable bodies.
The preceding path-host mapping, event clauses, two-sided fresh L0/Carton rebuild,
RPC observations, inputs and numeric ceilings remain unchanged. Unknown product
changes and partial mappings still refuse qualification.

Existing profile/export and allocator replay snapshots remain frozen. Checks
project only the authenticated whole Curator import and complete moved law onto
their original snapshots; the current law's physical path is used for mutation
controls. Changed bodies, duplicate laws and non-ENOENT read errors fail closed.
The allocator facade accepts only the complete reviewed Carton declaration and
recovers its unchanged original source hash. No replay artifact is rewritten.

Local controls pass; actual hosted Rust timing, zero-cost, dump, Curator, CLI,
whole corpus and protected instruction qualification remain pending. This is an
ownership change with no numerical performance claim. Whole #6834 platform and
no-std isolation, broader profiler migration and native graduation remain open.
The current 0.436 release source stays fixed; a future structural API release
uses the separately authorized 0.437 minor boundary.
