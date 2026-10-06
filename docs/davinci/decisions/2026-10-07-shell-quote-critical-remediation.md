# Critical shell-quote dependency remediation

Issue: [#8136](https://github.com/ubugeeei-prod/vize/issues/8136).

The reviewed [GHSA-pqg4-j6r4-53mv advisory](https://github.com/advisories/GHSA-pqg4-j6r4-53mv)
affects shell-quote `>=1.8.4 <1.11.0`. The first patched version is 1.11.0.
Its [upstream fix](https://github.com/ljharb/shell-quote/commit/6002b2ed90c6b83095eb272b6b0adaf3a172b0bc)
rejects any later string token containing LF, CR, U+2028 or U+2029 after a
comment token instead of allowing that token to terminate the shell comment.

Update the existing workspace override from 1.9.0 to 1.11.0 and its one
launch-editor 2.14.1 dependency edge. A real PNPM 12.1.0 lockfile-only solve
supplied the target metadata; restore unrelated solver changes to the signed
064ffb3b base. Frozen lockfile-only validation passes. The official registry
tarball's complete SHA512 integrity and regular members were authenticated.

The current production audit resolves
`npm__framework__nuxt>nuxt>@nuxt/devtools>launch-editor>shell-quote`.
After the update, its complete report has zero critical/moderate findings and
only the two existing source-attested high Forge/Braces entries. Keep their
patches, validators, advisory policy and printed reports unchanged.

The hosted consumer law resolves Nuxt's actual devtools/editor dependency,
checks every installed launch-editor instance uses 1.11.0, calls its genuine
explicit-editor parser, and retains whole ordinary token/quote results plus
all four line-terminator refusals. It does not launch an editor or shell.
Launch-editor's reviewed source calls `parse`, not `quote`; dependency
reachability does not establish an exploitable Vize comment/quote path.
This workspace override also does not govern independently installed users'
peer dependency graphs.

The publisher stopped the vulnerable 061/064 release attempt. Do not publish
that cut or add this advisory to an allowance. Resume once the actual signed
security merge and current strict installed audit are qualified. TODO: retain
fresh hosted consumer/security/full protected results and the signed merge
identity before handing the patched cut to the sole release publisher.
