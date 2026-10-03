# Braces depth source remediation

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830), with the
compiler's unfinished product acceptance tracked in
[#6880](https://github.com/ubugeeei-prod/vize/issues/6880).

The production audit found
[GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm),
which currently affects `braces <=3.0.3` and has no published patched version.
This is a distinct new finding. The existing Node Forge repair, audit threshold,
retry policy, Wasmtime closure, guest locks and native providers stay unchanged.
The native DOM source campaign remains unqueued while its security audit is red.

The proposed registry-package patch backports only the depth hunks from the
**open, unmerged** upstream
[PR #72](https://github.com/micromatch/braces/pull/72), at immutable head
`d0d575e55e74a4e0218e5248fafb79efc3e54ebb`.
All six original library files match the official `3.0.3` tag blobs.
The parser counts real brace and parenthesis blocks, respecting the existing
escape, quote and bracket parsing. Parsing and the three public AST walkers
refuse depth above 100. A caller can lower that limit but cannot raise it.

The backport preserves registry `3.0.3` quote/comma behavior already changed on
the upstream base. It also preserves the original stringify parent semantics
and reads `maxDepth` once to prevent a getter from changing the cap to `NaN`.
The package version remains truthfully `3.0.3`; this is not an upstream release.

Controlled actual-source tests pass 59 rejection and 18 positive/boundary
checks, and 1,320 complete output/AST comparisons across 33 ordinary patterns,
eight option sets and five public APIs. On macOS Node 24, under-limit strings
through depth 4,998 did not exhaust the default stack. The original package's
under-limit string exhaustion is reproduced with an explicit 512 KiB stack;
its public AST walkers exhaust the default stack at depth 20,000. The corrected
package instead produces the bounded depth diagnostic. These observations do
not claim this platform reproduces the upstream Node 26 default-stack threshold.

The current root pnpm graph resolves one `braces@3.0.3` package through
`chokidar@3.6.0` and `micromatch@4.0.8`, both from the Nuxt framework importer.
Its complete package source, registry integrity, patch bytes and every installed
resolved instance must be attested before recognizing this finding as repaired.
Independent Nuxt fixture npm lockfiles are outside this root pnpm repair,
including their historical `braces@2.3.2` records; no independent-install or
browser/CDN remediation is claimed.

The actual pnpm-generated patch lock, frozen Linux installation, strict compound
Forge/Braces audit report proof, unknown-finding refusal laws and fresh exact-head
Actions remain pending. The temporary resolver capture is evidence only and
must not appear in the production PR. No audit exception or version claim
substitutes for the actual installed-source proof.
