# Complete SSR fix-history capture

Tracks [#6880](https://github.com/ubugeeei-prod/vize/issues/6880), following the
[target witness audit](./2026-09-27-compiler-target-history-audit.md).

Nine original raw-template inputs cover three fixes: select `v-model` #990
(one input), duplicate component handlers #3701 (five inputs), and keyed
slot iteration/fallback #2487 (three inputs). Each source is copied exactly
from its existing executable witness. The input provenance preserves symbols,
source paths, fix SHAs and byte hashes. No synthetic outer wrapper, imported
module, dialect substitution or altered options are added.

The test-only example uses the original `compile_ssr` public entrypoint and
actual defaults. Its observation retains every public `SsrCodegenResult`
field: raw `preamble`, `code`, and nullable raw source-map string. Returned
diagnostics preserve code as the actual enum's `u16` representation, complete
message and complete nullable source location. Exhaustive result/diagnostic
patterns require an explicit contract change if their public fields expand.
The returned arena AST is not a serialized compiler output artifact and is
not part of this output contract.

Public options are serialized directly. Internal dialect, binding metadata,
Croquis absence and the wrapper's Standard/default matcher/slotted context
are explicit. Experimental option values are read from the actual defaults,
not inferred from generated code. The fixture reader rejects default drift.

## Actual build and capture

Source `22decfc11` was committed and tracked-clean before the narrow build:

```sh
CARGO_TARGET_DIR=/tmp/vize-ssr-fix-history-target-20260927 \
  cargo build --locked --profile ci -p vize_atelier_ssr \
  --example ssr_fixture_observer --message-format=json-render-diagnostics
```

It used a new exclusive target directory, never another agent's warm target.
The actual Cargo build finished successfully in 30.47 seconds. Its selected
example artifact has opt-level 0, no debug info and test=false. The executable
was copied off target and hashed before use:
`ef8c6d1ef3fa89cd3a6ed4af1e628fe378f81d4a80b3eea0a83a077a614664ff`.

The receipt binds source commit/tree, clean state, exact argv, lock hash,
toolchain versions, selected Cargo event, successful process and raw build-log
hashes. Raw stdout/stderr and process records are retained for both captures.
All nine complete outputs repeated equally. All nine actual maps are null and
diagnostic arrays empty; this workload does not certify non-null maps or
populated diagnostic handling.

Expected values come only from those actual outputs. The integration test
compares every complete output and option value against the immutable raw
capture. JSON object-key order alone is immaterial; raw strings, null/presence,
scalar types and array order remain exact. No Insta normalization, trimming,
CRLF conversion or import/body projection occurs. Separate expected JSON files
are convenient per-case views of the same actual payload, not independent
measurements. Capture metadata files use JSON/text extensions; authored inputs
use `.input.txt` and are explicitly compiled by this target.

## Remaining completion gates

This is local macOS arm64 profile-ci evidence, not GitHub Actions execution.
The nine-case ordinary Rust test and strict Clippy for the example/test pass
locally. The earlier three SFC complete Result tests and their strict Clippy
also pass on this exclusive target. Assertion-based tests use ordinary test
entries, and the observer returns a validation error on default-context drift.
No golden or raw capture was changed to resolve lint. Formatting, source-length
and archive integrity checks also pass. Fresh Actions must pass before merge.
Preserve the original substring, shape and runtime witnesses.

The public selected route is captured; no zero-fallback native route is
asserted. Native acceptance stays zero, and no shared compiler adapter or
whole-history/dialect certification is added. The issue's historical 609-fix
denominator remains unreconciled. These nine cases protect three concrete
fixes without closing #6880. Vapor exact-output captures remain next work.
