# Native CI builds retain line tables

Issue: #6830. The Linux Actions JS package tests build the source addon once
through `build-local.mjs --no-js`, then retain and authenticate the complete
artifact throughout the original CLI and NAPI formatter observations.

The two observed builders jobs in runs [38935551261](https://github.com/ubugeeei-prod/vize/actions/runs/38935551261/job/117090136978)
and [38936385902](https://github.com/ubugeeei-prod/vize/actions/runs/38936385902/job/117092960110)
used dev opt0 with full debug information. Each frozen addon was 991,546,723
bytes; its compressed ZIP member was about 196 MB. Cargo took 62 and 47.38
seconds, respectively, and each formatter artifact upload took 13 seconds.
The 90–95 second CLI and 70–74 second NAPI cases are whole-test durations,
not measurements of hash CPU time.

The existing Linux Actions/non-release gate now passes
`--config 'profile.dev.debug="line-tables-only"'` directly to Cargo. This keeps
debug assertions, overflow checks, opt0, incremental settings, features,
fresh build tokens and the committed JS/type entrypoints. Local builds and
the release profile are unchanged. The existing separate `build:ci` route
still owns its generated-entrypoint custody and debug0 profile contract.

[Cargo documents line tables](https://doc.rust-lang.org/cargo/reference/profiles.html#debug)
as retaining filename/line backtraces without variable/parameter data.
The existing fault collector requests backtraces, shared libraries and
registers while suppressing frame arguments. Complete artifact hashes,
source/toolchain identity, raw output and failure evidence remain mandatory.
Capture and replay both reject a changed command or an optimized,
assertion-disabled, overflow-unchecked or unexpected debug profile.

The `test-js-packages` cache namespace and trusted-write policy stay the same.
Cargo fingerprints profile settings; the new setting initially rebuilds
incompatible objects until the trusted native seed includes it. Cache
freshness and toolchain changes must be reported alongside timing results.

The controlled [Actions run 38986404760](https://github.com/ubugeeei-prod/vize/actions/runs/38986404760)
used the same source `c22d1f3eabf3e2e85f955da0e003588eb3d8e021`, based on
`ee0d93020bbc3a25acee7a2fd0dba45f3ea947c7`, for both variants. The separate
duplicate-hash optimization was absent from both. Both used Blacksmith 32,
Rust 1.99.1, Node 24.14.0, the same restored registry/git/JS cache keys and a
missing Cargo target cache; all 435 Cargo artifacts were rebuilt. Non-main
push policy did not mount or write the trusted sticky disks.

| Emitted artifact                 |  Full debug |          Line tables |
| -------------------------------- | ----------: | -------------------: |
| Cargo `debuginfo`                |         `2` | `"line-tables-only"` |
| Addon bytes                      | 991,707,977 |          422,179,756 |
| Compressed addon ZIP entry bytes | 195,921,909 |           77,408,510 |
| Observed Cargo seconds           |       50.59 |                47.75 |

The addon shrank by 569,528,221 bytes (57.4%); its ZIP entry shrank 60.5%.
The byte comparison uses the addon entry because the complete archives
contain different report sets. These are single cold-build observations,
not a claim about warm PR latency or a measured full-workload speedup.

The [line-tables job](https://github.com/ubugeeei-prod/vize/actions/runs/38986404760/job/117258795632)
passed the unchanged builders workload: all original NAPI 9 cases/25 passes
and CLI 9 cases/27 passes matched their references. GDB 15.1 stopped in the
actual frozen addon at `build_token.rs:13`, retained source frames at lines
13 and 11, and exited normally; its artifact hash remained unchanged.

The [full-debug job](https://github.com/ubugeeei-prod/vize/actions/runs/38986404760/job/117258795965)
emitted the correct full-debug addon, then failed because cached package
tasks omitted the diagnostic-only environment selector. Strict receipt
replay correctly rejected the full-debug command against the default
line-tables contract. Its original failure and raw output remain retained;
it produced neither complete formatter reports nor a GDB observation.
There was no retry, so this run cannot establish paired test parity or
end-to-end timing. The landing implementation has a literal Cargo setting
and no selector propagation requirement. Temporary comparison workflow,
observer and selector are excluded from landing; the combined final PR
still owns actual development-loop latency qualification.
