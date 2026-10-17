# Requalifying a published immutable consumer fixture

Tracking issue: [#9309](https://github.com/ubugeeei-prod/vize/issues/9309).

An immutable Release run can publish its packages successfully and then fail because its own
consumer fixture is defective. Retrying executes the same frozen workflow. Rewriting that source,
changing its tag, declaring its failed run successful, or using unpublished retirement would lose
the original evidence. This procedure preserves all of it.

This is an explicitly selected, digest-bound migration for one reviewed fixture shape. Ordinary
releases and ordinary `resume` still require their original Release run to succeed. No automatic
fallback selects replacement evidence.

## Admission

The original C, H, source PR, annotated tag, integration PR and Release run must authenticate
together. The integration must retain its actual signed merge and exact delivery Check success.
The original frozen policy records the actual signed delivery as its protected candidate; it does
not claim a merge-group workflow ran on that SHA. Recovery preserves that original contract.
Every original artifact must remain available with its recorded archive digest.

The original release and JSR workflow bytes and the unchanged consumer bytes must match the
reviewed hashes in `consumer_recovery/policy.ts`. The complete effective job inventory must match
the original 56-job graph: 54 successes, the conditional crates.io handoff skip, and exactly one
failed Deno consumer. Missing, pending, failed, cancelled or newly skipped build/publication jobs
refuse recovery. The failed job must belong to the latest attempt, contain exactly the known
`npm:vize@VERSION` / `node_modules` exception, and have failed only its original Deno invocation.

Admission retains the exact Deno configuration source and digest qualified in
[#9257](https://github.com/ubugeeei-prod/vize/pull/9257), including its existing scoped exclusions.
The separate **effective recovery configuration** contains only `nodeModulesDir: "none"`, with an
empty Deno cache and Deno 2.9.7's default dependency age. Every Deno packet records that effective
configuration's digest, and finalization/verifiers check its retained `deno.json` bytes. No age
exclusion or override executes. The existing additional checks that every observed Vize npm
package and JSR version is at least 24 hours old remain required.

## Official sequence

1. Review and merge this policy normally. Dispatch **Public release consumer recovery** from
   `main`, supplying exact C/H/tag/source-PR/original-R/latest-attempt identities. This workflow
   reads registries; it cannot publish, change a tag, rerun R or close a PR.
   Its admission contracts run in pull requests and must pass again before live qualification.
2. Admission retains the complete original failure log, job inventory, source bytes, artifact
   identities, signed integration evidence and fresh public registry archive observations.
   Original log transport uses `--allow-escape-sequences` and retains complete stdout/stderr bytes
   and status before semantic gates. UTF-8 decoding preserves BOM and requires an exact byte
   round-trip. For the original 0.457.0 / R4 tuple, the entire 24,145-byte failed log must retain
   SHA256 `df3ec6ea97cd47a655a0fd46528a4e28f24ae3aac6fa256ff8b7e23a8e778860`.
3. All six original contracts execute in fresh projects outside the checkout: Node 22 and 24 on
   Linux, Node 24 on macOS and Windows, Deno 2.9.7 on Linux, and Bun 1.3.14 on Linux. Each uses the
   byte-for-byte original consumer, its exact terminal completion output, actual public JSR/npm
   packages and a loaded native provider. A zero exit without the complete consumer witness fails.
   Native/public commands receive an explicit runtime environment allowlist, not arbitrary job
   variables. Credentials exist only in trusted metadata steps; no native/public parent step has
   a job-wide credential. Complete raw command bytes are retained separately from decoded text.
   Node/Bun retain the Vite assertion; the original Deno contract does not import that entry point.
   Complete commands, stdout/stderr, exit status, installed paths, archive hashes, manifests,
   lockfiles and publication times remain in the packet.
   The actual installed manifest and every shipped regular package file must match the entries
   of the SRI-verified public tarball, including the loaded native provider's exact bytes. The
   original consumer, Deno configuration and custody probe are hashed around every command;
   their retained files are verified again before signing and during signed verification.
4. Finalization requires all six packets from the same current run and attempt, rechecks the
   original identities and all public channels, authenticates the closed source/import/config and
   materialized harness policy, and signs a complete file manifest with the
   existing pinned Cosign/OIDC toolchain. The receipt says `originalConclusion: "failure"` and
   `status: "consumer-requalified"`; it never represents R as successful.
   The finalizer independently fetches trusted exact-version npm metadata and re-inspects each
   public archive; the later signed verifier does the same. An archive index supplied by a consumer
   cannot authorize itself. Downloads are deduplicated by public URL and SRI, and the signed packet
   retains the recomputed index and archive digest rather than copies of large native archives.
5. Dispatch the protected main-only **Release Operator**, operation `recover-consumer`, with
   `source_cut`, `source_pr`, `source_head`, `source_tag`, `release_run`, and
   `consumer_recovery_run`. It verifies the exact signer workflow/ref/repository/commit, signed
   receipt, all raw evidence, current original attempt, integration and public registries.
   This operation verifies evidence and writes its own Actions artifact only.
6. Run **Public release acceptance** with the original C/H/tag/source-PR/R and the explicit
   `consumer_recovery_run`. Signature verification is mandatory; the workflow preserves the
   original failure in its publication packet and runs its complete installed native, LSP,
   browser/Musea and VRT acceptance. Existing product checks remain required. After all installed
   processes finish, its metadata-only seal revalidates the signed recovery and current original
   attempt again before accepting the installed results.
7. Only after that acceptance succeeds may the maintainer close the original unmerged source
   receipt and start the next official release. Retain original R as failed and leave H, the
   annotated tag, original workflow and public versions unchanged.

A replacement expires after 24 hours, binds one exact original run attempt and one reviewed policy
commit, and requires every recovery job to succeed in its own exact attempt. Rerunning either run,
changing policy bytes, substituting a consumer, missing raw files or invalid signatures refuses
verification. Produce a new complete proof instead of reusing an old packet. Signatures verify
through the normal public Sigstore trust chain and transparency evidence; no ignore-log or
insecure verification flags are supported.

## Original incident

The initial request is C `014bd42cacdf3d026fb75162cfa80dce76d78315`, H
`03882089eff75664abe99ba71868e33739496b01`, tag `v0.457.0`, source PR #9196, original R
[`38805261385`](https://github.com/ubugeeei-prod/vize/actions/runs/38805261385), attempt 4.
Its failed job is `117254411511`. Those operational identities are retained in the primary test
fixture and supplied explicitly at dispatch; production admission is based on authenticated
identities and the reviewed frozen bytes and job graph.

The implementation and unit tests do not establish actual hosted requalification or release
completion. Record the resulting signed recovery run and full installed acceptance run before
claiming this incident complete.

The pull-request admission-contract job additionally typechecks the complete recovery authority
and runs one read-only Linux Deno probe of the exact historical H consumer using only the host
configuration correction. Its commands, whole raw streams, effective configuration digest and
actual runtime outcome are retained. This probe never admits, signs or substitutes release
evidence, closes a source, or launches the six-platform live recovery. Record its hosted result
before requesting approval for this authority change.
