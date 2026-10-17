# Optimize the CI native image hash

Issue: #6830. The original Vben incremental test failed its unchanged
1,500 ms second-app cold-open budget in full qualification
[38988499789](https://github.com/ubugeeei-prod/vize/actions/runs/38988499789)
at source `e2b883eed6378eb7ada73bf0220d89a185a3d6fd`.

The instrumented [run 38990136719](https://github.com/ubugeeei-prod/vize/actions/runs/38990136719)
at `bd7c444d398baf5abc337fd32b776e5dab2d2e35` reproduced the failure at
1,961.16 ms on the same pinned fixture, Blacksmith 32-vCPU runner class,
`profile.ci` and budget scale 1. The Vben session performed six complete
native-image authentication calls before failing. Its final two SHA-256
phases took 323.03 and 322.17 ms (645.21 ms total); their file reads took
4.39 ms combined and each complete initialize comparison about 0.05 ms.
Every decision accepted the same 24,101,026-byte native image with SHA-256
`4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4`.
Nested phase durations are not added to their enclosing decision durations.

`profile.ci` inherits dev's opt0. A version-scoped package override now
builds the already pinned `sha2:0.11.1` at opt3, explicitly retaining debug
assertions and overflow checks. The SHA-256 compression loop belongs to
this dependency; its existing runtime CPU detection and software fallback
remain intact. The other locked SHA-2 version, local dev profile, product
crate optimization levels and release profiles are unchanged.

Authentication still compares the whole initialized response and reads and
hashes every executable byte for each session, immediately after initialize.
There is no path/stat cache, shortened digest, deferred authentication or
new provider. The original shutdown behavior, test fixtures and budgets are
unchanged. The dependency and its consumers need an initial Cargo rebuild;
normal profile fingerprints govern subsequent reuse.

Workspace qualification accepts only this exact version-scoped CI package
override, with all three fields present and both runtime checks enabled, or
no package override. Other package/version keys, extra fields, global or build
overrides, Cargo config and environment overrides remain refused. The actual
manifest and mutated copies exercise this boundary; source-file digests and
complete inventory/execution receipt checks remain unchanged.

The optimized [run 38990649880](https://github.com/ubugeeei-prod/vize/actions/runs/38990649880)
at `60e3c12b99926916f31359eddfd3593c74f9ec85` passed both original incremental
tests once, including all fourteen Vben and nine Misskey observation stages.
Vben second-app cold open measured 1,365.77 ms against the unchanged 1,500 ms
limit. Its two SHA-256 phases took 9.28 and 8.84 ms (18.12 ms total), with
22.70 ms for both enclosing authentication decisions. All six Vben and three
Misskey decisions accepted the same complete native image recorded above.

Verbose build output confirms opt3 and `debug-assertions=on` for `sha2:0.11.1`;
Canon and Vize remain opt0. Cargo retains explicit `overflow-checks = true`;
rustc omits a redundant flag because overflow checks follow enabled debug
assertions by [default](https://doc.rust-lang.org/rustc/codegen-options/index.html#overflow-checks).
Both server processes exited 0. Full raw streams preserve the original
synthetic-fixture configuration warnings and passive phase traces.

The optimized run retains the baseline's instrumentation and original workload,
with only the reviewed Cargo override and diagnostic guards changed. This is
an instrumented paired observation; combined, uninstrumented Full Check remains
required. Diagnostic code is excluded from this change, and no source-coverage
speedup is claimed because that job uses a different Cargo profile. Local
validation covers the actual manifest, exact dependency pin and narrow profile
refusal controls without a product build.
