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

The registry-package patch backports only the depth hunks from the
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

Actions run `37087472637` produced the genuine pnpm 12.1.0 patch resolution.
Its full output also pruned unrelated native catalog/importer/package entries
and rewrote Forge's existing lock representation; the subsequent frozen install
failed. That original output and failure are retained. The production lock uses
only its Braces patch hash, patched snapshot and two parent edges, conserving
every unrelated original byte. This is a resolver-derived projection, not the
verbatim full resolver output or a hand-written integrity value.

Actions run `37088402443` then passed the inherited
`vp install --frozen-lockfile --ignore-scripts --prefer-offline` on the exact
projected lock. Its complete Linux census contains ten published Braces files,
both actual parent consumers and the three range dependencies. All 34 file
contents match their recorded hashes; the package and both linked parents pass
the 59 rejection and 18 positive laws. Forge's installed-source proof passes
unchanged. The Braces digest is
`74958865c440ebdccbe18322a4b9a2d6468368e5a3c975a761c0752d8ff2c0cc`.
An earlier private digest accidentally included patch-construction Git metadata;
that receipt is retained with an additive correction. Unexpected installed
metadata, files or symlinks still fail the full-package digest.

The original moderate-threshold audit JSON remains visible and unmodified:
two HIGH findings (Forge and Braces), no MODERATE/CRITICAL, and one LOW count.
The strict report composer accounts for the complete actionable set before
recognizing precisely those two independently source-attested Node repairs.
Unknown findings, count/identity/path changes, new consumers, aliases, missing
patches and changed sources remain fatal. The LOW count remains visible under
the original threshold. No audit exception or upstream patched-version claim
substitutes for the actual installed-source proof.

The temporary source and workflow are evidence only and excluded from the PR.
The first ordinary source campaign passed its security audit but strict
type-aware lint refused an implicit sort of unknown advisory IDs. Its failure
is retained; the follow-up supplies an explicit comparator while preserving
the original raw identity comparison and unchanged audit conditions.
Fresh ordinary exact-head required/full/Contracts/Fuzz/all-100 Actions and actual
protected-queue merge remain pending; the native children stay unqueued until
the security repair is accepted. This does not finish compiler fix-history
acceptance or switch any product's legacy default.
