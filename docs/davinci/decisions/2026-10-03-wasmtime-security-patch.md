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
Actions-only resolver capture used Cargo 1.98.0 and exact source `2576a9b5`,
running only `cargo update -p wasmtime --precise 48.0.4`. Actual
[run 37064301182](https://github.com/ubugeeei-prod/vize/actions/runs/37064301182)
and job `111028167782` succeeded with original inputs preserved and only
`Cargo.lock` dirty. The complete raw/gzip-hashed lock, package blocks and
original resolver log are retained; the final source excludes the temporary
workflow and courier.

The generated lock is 134,299 bytes, with SHA-256
`b7abcb06a463b6866617170cfc7ef77801632cf253184da571043031da66f41a`.
Strict TOML and complete-block review retain all 524 packages: 14
Wasmtime/Pulley records resolve to `48.0.4`, 13 Cranelift records to compatible
`0.135.4`/`0.135.5`, and six wasm-tools records to `0.254.2`. These are the
existing dependency families required by the
[upstream patch manifest](https://github.com/bytecodealliance/wasmtime/blob/v48.0.4/Cargo.toml).
Every other package block, including the separate `0.259.0` wasm-tools
records, stays byte-exact. The only dependency-reference changes are 16
`0.254.0` references to `0.254.2`; no package, dependency or feature edge is
added or removed. All checksums come from actual Cargo output. Eight
checksum/version/edge/unrelated-package/extra-package/TOML controls reject
malformed closures without a build.

Product `3e31` passed its four corrected tooling shards, runtime laws and all
100 probes, but its required/full checks ended security-red; it stays unqueued. Preserve
both failed audit logs and the separate earlier standalone guest lock failure.
The patch gives no new runtime, compiler history or product-switch credit.
Fresh exact-head audit with `--deny warnings`, real both-mode Contracts,
required/full checks, Fuzz, all 100 probes and protected queue/actual merge
remain mandatory before acceptance.
