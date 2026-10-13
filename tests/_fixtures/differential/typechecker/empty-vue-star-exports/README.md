# Empty Vue star exports: additive native supplement

This corpus supplements the historical type-checker fixtures. It does not change
the original 32 entries or goldens, qualify a legacy adapter, or close #6879.

The production change concerns an unaliased top-level `export *` whose original
specifier is explicitly relative and ends in `.vue`. Erasure requires the actual
registered SFC's original script blocks to prove that it has no named module
exports. A default export is eligible. Unknown, named, indirect, recovered, or
external script sources remain conservative. Alias, package, and extensionless
routes receive no erasure credit.

## Discovery and execution

Cargo automatically discovers
`crates/vize_canon/tests/empty_vue_star_exports.rs`. It loads every case listed in
`supplement.json`; the test also checks the complete ordered case inventory.
No existing fixture manifest or workflow must be rewritten to discover it.

The hosted native gate can enroll it with:

```sh
VIZE_TEST_REQUIRE_TSGO=1 cargo test -p vize_canon --test empty_vue_star_exports -- --nocapture
```

`CORSA_PATH` selects an installed genuine native runtime when necessary. The
runtime must belong to the official host-platform TypeScript 7.0.2 package,
resolve to its actual `lib/tsc` executable, and return exact version output with
exit zero and empty stderr. Package bytes and the whole executable SHA are
retained independently. No LSP/API/PID/options custody is implied by CLI evidence.
Vue and
Vite must come from installed workspace providers, with whole manifest and entry
declaration bytes matching the retained authentic qualification. No handwritten
Vue declarations, wildcard shim, or canned diagnostic transport is used.

Required native mode fails if the runtime or providers are missing or disabled.
Default-mode absence prints an explicit `nativeAcceptance: unobserved` receipt.
Such a return provides no native acceptance credit.

Set `VIZE_TEST_FIX_HISTORY_CAPTURE_DIR` or `VIZE_NATIVE_SYNTAX_CAPTURE_DIR` to
retain all current packets. The capture includes authored inputs, current
generated source and source maps, generated config, provider bytes, complete
production results, same-config native CLI output, native `--showConfig`, and
native `--listFilesOnly` dependency bytes. Every loaded dependency is read with
its full body, byte length and SHA. Actual loaded Vue import/require declarations
and Vite client must match their own exact declared provider roles. A source read
of an entry is distinct from proof that native loaded that entry.
Registered producer roots and the
native dependency closure are separate fields.

## Contracts and credit

| Cases | Assertion |
| --- | --- |
| Original positive, wrong prop, plain authored `Shared` duplicate | Complete exact production and raw native vectors taken from authentic counterfactual output: zero, two TS2322, and one authored TS2308 respectively. |
| Direct types, classic default only, namespace | Complete empty vectors, concrete generated Props/Emits/Slots/default types, preserved namespace and named default exports. |
| Authored named and reserved `Props` | The named-export provider's star remains; its genuine authored types remain importable and concrete. |
| Strict and permissive inherited config | The selected native config controls the full null-assignment diagnostic vector; a decoy config cannot supply the options. |
| Malformed, script `src`, indirect, alias, extensionless, missing | The specified star refuses erasure. Every native result is retained as an observation; no unrecorded boundary semantic golden is claimed. |

All exact production laws compare complete file, authored UTF16 start, severity,
code, and message vectors. Ordinary barrel, consumer and supporting script
projections equal independent complete literal bodies. Whole generated Vue
modules are parsed and compared against complete exported-name lists; substring
checks cannot prove preservation.

Raw native diagnostics retain their separate generated-coordinate authority.
The original three controls compare complete unfiltered raw vectors with their
authentic generated-coordinate oracle, normalizing filenames only, after
proving complete diagnostic-file projected-body and authored diagnostic-line
identity. Additive valid cases assert an empty raw vector. The inherited strict
configuration case retains its nonempty raw vector as an explicitly unrecorded
generated-coordinate observation; its complete authored production oracle is
asserted separately. No raw position is converted into an invented mapped
oracle. Generated direct types and default constructors remain in the SFC output;
only proved-empty importer stars change.

The malformed/source-boundary observations deliberately carry narrower credit.
Their unchanged stars are assertions, while their other native diagnostics are
not reconstructed into invented goldens. These observations do not substitute
for exact semantic laws or broader real-project acceptance.

Pending full fixture/native/provider/config custody is written before the normal
Batch call. Every real subprocess output, including raw non-UTF8 bytes, is
persisted before interpretation or validation. Completed source, config,
dependency and diagnostic packets survive genuine RED assertions. Pending and
accepted projection/native statuses are separate.

## Authentic evidence custody

`authentic/receipt.json` binds the exact original `whole-packets.json` bytes,
native binary hash, complete command arguments, outputs, provider qualification,
and six complete historical packets. SHA256 of the original packet is:

```text
a5e32916175feeb304fc88744cb5fcd94ca5f39ee385dad5ed573022d893129e
```

Every input and provider body is stored losslessly in `blobs/<sha256>.json`.
Decoding `wholeUtf8` produces the exact bytes and byte length recorded in the
original packet. The custody test reconstructs every complete packet and
compares it with the byte-retained original; output and input hashes are checked
independently. The authentic Panel source is 233 bytes and is shared unchanged
by the original production controls.

Those historical generated `.vue.ts` and helper bytes prove the native
counterfactual class: helper star ambiguity disappears while real consumer
errors and plain authored ambiguity survive. They are never installed as
current production inputs or granted current producer credit. Production
controls use authored `.vue` sources and the ordinary explicitly configured
`BatchTypeChecker::scan_project` / `check_project` path.

Current-source native qualification, full Nuxt/Reka acceptance, performance
measurement, merge, release, and fix-history closure require separate evidence.
