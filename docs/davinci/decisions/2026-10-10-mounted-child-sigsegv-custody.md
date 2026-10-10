# Mounted child SIGSEGV custody

Issue: [#7951](https://github.com/ubugeeei-prod/vize/issues/7951).

The metadata Check [38027451698, Rust shard 2](https://github.com/ubugeeei-prod/vize/actions/runs/38027451698/job/114142745936) ran 4,183 tests, with 4,182 passed and one failed: `object_positional_matrix_matches_reference_or_pinned_upstream_gap`. The original VDOM Node child returned SIGSEGV 11 and printed a complete ten-snapshot JSON trace, with empty printed stderr. The original status assertion at `davinci_mounted_behavior/runtime.rs:155` correctly rejected the child before comparing the oracle.

The worker checked out synthetic source `1d0f18e51a73f627922690bc4592689c927e080e`. Its whole tree equals frozen metadata `510d72772ebfc0348e0392e66e425481a790eccf` and release source `bf2cd911c268605588b2ceedbc011e36c79d113c`: `2a6355da2e5ddeb9024d2784d51a75709c0e6365`. The original raw failure log has SHA-256 `97c5a03a755528de4da1b4e3d1a48af57be4ef7254a2229c28a6a87877fc2ec7`.

The panic's complete serialized stdin can be reconstructed without an added newline: 1,994 bytes, SHA-256 `6419fea8a049f0c000d0a637380d2e601160dc404a3a758ae5be2f27b78b6722`. The whole observed trace equals the original `object-positional-toggle-open` reference. This is source/log reconstruction, not an independently captured original stdin stream, and cannot turn the fatal exit into success.

The job records setup/doctor Node 24.14.0, but the actual `Command::new("node")` executable/version was not captured. Node 22.23.2 from a separate older job does not identify this child. The crash's native cause and its relationship to the earlier missing-status #7951 failure remain unknown.

Use an isolated diagnostic branch and the existing manually dispatched mounted-runtime workflow. Retain every original Rust/Node runner, compiler option, matrix input, full reference, pinned gap and strict exit assertion. An observational preload captures the actual Node executable, versions, process maps and native addon images, without consuming stdin, writing protocol streams or installing signal handlers. Enable core capture and preserve complete original outcomes and backtraces. This observational replay does not qualify a repair or close #7951, even if it passes; the independent exact failed-shard replay retains its original full gate.

TODO: inspect genuine process/core evidence, make only a source-bound necessary correction, then complete exact-source Actions and protected/public delivery. Never ignore a nonzero child, add a retry-based success rule, or change original oracles.
