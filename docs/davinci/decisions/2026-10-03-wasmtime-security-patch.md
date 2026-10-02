# Wasmtime host security patch

Issues: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) and
[#6880](https://github.com/ubugeeei-prod/vize/issues/6880).

Actual main `b2d0`, its signed successor `688d` and product #7459 source
`3e31` inherit Wasmtime `48.0.3`
through `vize_extension_host` and `vize_dialect_moonbit`. Required and full
security audits reject three advisories published on 2026-10-02:
[RUSTSEC-2026-0325](https://github.com/RustSec/advisory-db/blob/main/crates/wasmtime/RUSTSEC-2026-0325.md),
[RUSTSEC-2026-0326](https://github.com/RustSec/advisory-db/blob/main/crates/wasmtime/RUSTSEC-2026-0326.md)
and [RUSTSEC-2026-0327](https://github.com/RustSec/advisory-db/blob/main/crates/wasmtime/RUSTSEC-2026-0327.md).
All three list `>=48.0.4,<49.0.0` as patched. The
[upstream 48.0.4 release](https://github.com/bytecodealliance/wasmtime/releases/tag/v48.0.4)
provides the compatible patch; a major-version migration is unnecessary.

Pin the existing workspace dependency to exactly `48.0.4`, preserving its
disabled defaults, explicit feature list and sole optional Host edge. The
three standalone guest lock corrections remain byte-exact to reviewed
`feb5c2`; the combined source retains every incoming Source29 path from `688d`. Guest
SDK/WIT APIs, goldens, released fixtures, normal dependency direction,
workflow commands and all instruction ceilings are unchanged.

The local read-only registry metadata lacks the new patch. A temporary
Actions-only resolver capture uses pinned Cargo 1.98.0 and the exact prepared
source, runs only `cargo update -p wasmtime --precise 48.0.4`, and retains the
complete generated lock, package delta and source/output hashes. No guest,
host or product build belongs to this transport. The final source excludes
the temporary workflow. Genuine lock generation and strict review of every
changed package/version/checksum/dependency edge are still pending.

Product `3e31` passed its four corrected tooling shards and all 100 probes,
but its required/full checks remain security-red; it stays unqueued. Preserve
both failed audit logs and the separate earlier standalone guest lock failure.
The patch gives no new runtime, compiler history or product-switch credit.
Fresh exact-head audit with `--deny warnings`, real both-mode Contracts,
required/full checks, Fuzz, all 100 probes and protected queue/actual merge
remain mandatory before acceptance.
