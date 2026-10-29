# Packed Babel host contracts

These isolated consumers use actual Babel 7.29.7 and 8.0.9, their matching major plugin releases, and an actual `npm pack --ignore-scripts` artifact. They never import the plugin's source files or resolve a registry native fallback.

Install each fixture with `npm ci --ignore-scripts --prefix npm/babel/test/hosts/fixtures/babel7` and the corresponding `babel8` path. The lockfiles pin the actual host/runtime graph. Run `host-controls.ts` to check only the real host API/parser/ordinary-plugin infrastructure; that command grants no packed-plugin or native credit.

The full command is `node --test npm/babel/test/packed-hosts.test.ts` with:

- `VIZE_BABEL_PACKED_PLUGIN`: the real built plugin `.tgz` containing both public runtime and declaration entrypoints.
- `VIZE_BABEL_NATIVE_ROOT`: the explicit source `npm/native` directory, with the matching package version and a genuinely built local addon.
- `SOURCE_SHA`: the full source-native Git revision.
- `VIZE_BABEL_HOST_OUTPUT` (optional): a fresh output directory for whole results and raw process streams.

Run the full command while the repository's actual native-preparation owner is active. Each positive worker validates the preparation's source head, tree/diff lease and image hash, then verifies the actual loaded `.node` against that same image. Preparing a receipt around an unrelated existing binary is not source qualification; the owning source Actions build supplies the build provenance.

The five top-level tests are strict. Missing artifacts, hosts or preparation fail rather than skip. Each host is installed once per test process; run `npm ci` again before another run, so stale extracted packages cannot become an implicit input. Both actual Node conditional exports are exercised from real `.mjs`/`.cjs` consumer entrypoints.

The cold suite blocks any native module/addon resolution and compares every Babel output field against the same host without the plugin. Both raw results retain Babel's self-describing input options; those options contain deliberately different plugin lists and hook functions and are excluded from output equality. Parser options are fresh for each actual host invocation. Positive workers retain every case's whole code/map/AST/metadata/error, continue collecting after individual failures, and fail the worker if any contract fails. Source-native positive cases currently cover VDOM; they make no Vapor/SSR-target admission or runtime speed claim. Vue server rendering is an independent whole-HTML oracle for the VDOM host result.

The parent lane owns implementation, map precision, profiling and source-native Actions. Preserve failing packets when a source feature is not ready; do not substitute a public binary or bless a narrower oracle to obtain a green source result.
