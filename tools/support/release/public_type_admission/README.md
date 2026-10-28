Run the full entry only against the first published tag containing merge `83e3ff318fb772340663b5688624dac2ad37e2b1` (#10950). Versions 0.602.0 and 0.603.0 are ineligible.

Full entry: `node tools/support/release/public_type_admission/run.ts WORKSPACE VERSION OUTPUT ELIGIBILITY_JSON`. Its first-containing schema and all ancestry/publication checks remain required. Add `mapperHost: { route: "hosted-content-mapper", sdkVersion: "7.1.0-beta", workspace: HOST_ROOT, setupReceipt: HOST_PROOF/setup.json.txt }` to eligibility.

The existing CPP graph retains Vue 3.5.42, native TypeScript 7.0.2, the published Vize packages and its one authenticated physical NAPI image. CLI and LSP use that graph. The additional Linux x64 hosted-mapper column uses an isolated TypeScript 7.1.0-beta graph. Its actual binary, lock rows, tarball integrities, version and help are pinned separately. The 7.1 host checks the CPP fixture config and must load the same published Vize/NAPI image from CPP; no source overlay or silent backend replacement is allowed.

Native 7.0.2 rejects `--runExternalCode`. Tree `55116fa7a8215a904c0e6ccd6339d55bda2cbeeb` preserved an unexecuted driver that incorrectly assigned this route to 7.0.2. This revision replaces that driver contract while preserving every original fixture byte, expected vector, shared-helper pin and immutable source admission. It does not claim that the old driver ran successfully.

Fresh full acceptance requires 32 CLI packets on 7.0.2, 32 hosted-mapper packets on 7.1, all 117 LSP diagnostic responses and four typed hovers. Full authored inputs, outputs, messages, multiplicity, process argv/status, both locks and physical image identities are retained. Expected diagnostic comparisons use file, UTF-16 line/column and code; original full messages remain available.

Capability entry: `node tools/support/release/public_type_admission/capability.ts WORKSPACE 0.603.0 OUTPUT CAPABILITY_JSON`. Its separate `vize.typeAdmission.sourceCapabilityEligibility.v1` schema requires `scope: "source-capability-only"`, actual known publication/tag/source and the same mapperHost plan. It authenticates published 0.603, then checks a numeric template through valid, invalid and repaired stages using the same process/observer mechanics. This proves route/provider mechanics only; its receipt always denies #10950 and first-containing-release qualification.

The historical source admission (40523513452 at 4834900) contains 32 CLI and 32 independent Vue oracle packets plus LSP/hover checks. The original public mapper counterexample used 7.1. Later three positive mapper packet identities are not currently requalified and remain null. Neither historical source proof nor capability-only 0.603 replaces fresh first-containing public acceptance.

The actual final merge-queue source admission for #10950 remains Check 40603113935 / job 122536275919: source83e3, native7.0.2, VIZE_TEST_REQUIRE_TSGO=1, 3359 passed and 612 cohort-skipped. Global-origin #10980 remains open and unqualified. No new publication or consumer dispatch is implied by this capsule.

Shared helpers remain blobs `8c58f64e4d6ff081a00cfa9da9f947c359be3869` and `4197bd7ccc74051bdb1015370b6e434ba45d6f29`. Bootstrap installs occur only in fresh remote consumer jobs; the capsule does not install packages.
