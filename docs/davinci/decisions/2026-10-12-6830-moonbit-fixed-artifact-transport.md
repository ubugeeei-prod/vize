# Fixed MoonBit artifact transport

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Cold setup of the exact `0.10.7+bc794d341` compiler returned HTTP 403 before
compilation in the original 447 retirement job and later ordinary/protected
jobs. The complete original log and source identities are retained in
[`moonbit-pinned-transport`](../../../tests/_fixtures/tooling/moonbit-pinned-transport/).
Fresh encoded/literal-plus requests, cache-busting queries and no-cache headers
also returned 403 without redirects. The administrative cause remains unknown.
Retirement already omits its unused MoonBit dependency; start/resume, Docs and
the existing native suites still genuinely require the pinned compiler.

Use the immutable
[`v0.10.7+bc794d341` mirror](https://github.com/chawyehsu/moonbit-binaries/releases/tag/v0.10.7%2Bbc794d341)
with fixed full-archive sizes and SHA-256 checks before executing the installer.
All three Unix hashes equal the existing Nix pins. The newly explicit Windows
ZIP hash `c659625f5c3a9fca5d17866bff3de07b6328c59628d74df5d5b4d79b78524880`
matches its immutable release metadata, original checksum asset and downloaded
84,486,115-byte ZIP. All 232 regular member CRCs and file hashes were verified
without executing the compiler. The pinned mirror source downloads the official
ZIP, reads its actual `moonc -v`, hashes it and copies it without repacking.
This provenance does not substitute for real Windows setup and native smoke.

Keep the complete original core tar in the checked action source. Its existing
Nix SHA-256 `06922d35dd94dc0baea7682295fe6084f0d854eed3b497cd6b527f77d899a7f5`
authenticates all 1,449,345 original bytes, including the Apache-2.0 license and
attribution notices. It was retained from the fixed-output Nix archive. No tar
member, permission, header or gzip byte changes. The mirror core ZIP is a distinct
artifact with one additional README; it is not treated as the original tar.

The official Unix and PowerShell installer hashes stay unchanged. After both
payloads are qualified, Unix replaces exactly two URI assignments with local
file URIs. PowerShell copies the authenticated compiler ZIP and extracts the
authenticated original core tar with Windows' existing `tar`, refusing a failed
extract. Every other official installer byte, compiler-version check, registry
retry/refusal, Darwin header adjustment, native async smoke, shim and export
remains. The cache key includes the complete action source and core asset.
Unknown versions, platforms, development archives, source drift, HTTP failures,
missing payloads and size/hash mismatches refuse; there is no latest fallback.

Five added source laws cover fixed identities, reversible whole installer
boundaries, ten staging/refusal scenarios, the original full-shell 403 followed
by controlled fixed transport, and the complete Node caller including cached
reuse. The compiler in the shell/caller controls is explicitly a tiny fixture.
All 65 inherited setup/workflow/registry laws also pass; their original bodies
are preserved, with only the necessary module-copy/transport-isolation fixture
adapters. The real transport helper is exercised independently and through the
complete caller. The official script's existing unquoted executable invocation
requires an ordinary runner-home path; unusual artifact URI paths are tested.

These are source-only observations. Genuine exact-head Actions, protected
integration, next release gates and installed consumers remain required.
There is no historical Docs publisher replay or new recovery dispatch. A future
necessary automatic main Docs run uses the repaired current action and retains
its own source/build/deployment identity; the failed frozen-775 run stays failed.
