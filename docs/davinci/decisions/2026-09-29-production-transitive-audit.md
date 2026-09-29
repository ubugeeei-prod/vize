# Production transitive audit repair (2026-09-29)

Tracking issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The ordinary Check security audit became red on unrelated Davinci PRs after
production dependency advisories appeared for `fast-uri@3.1.6`,
`ip-address@10.3.1` and `undici@8.10.0`. The affected importers are the MCP
server, unplugin and Nuxt. Keep the required audit and patch only their shared
transitive resolutions: `fast-uri@3.1.7`, `ip-address@10.5.1` and
`undici@8.10.2`. Preserve the other lockfile resolutions and the existing
ordinary and merge-queue checks.

The upstream advisories are [fast-uri URI validation](https://github.com/advisories/GHSA-qw65-cvwx-89v3),
[fast-uri denial of service](https://github.com/advisories/GHSA-58mr-gqgx-xq4g),
[undici](https://github.com/advisories/GHSA-3wwx-pv8p-q78v), and
[ip-address advisory one](https://github.com/advisories/GHSA-rpw4-54j3-4h4q)
and [two](https://github.com/advisories/GHSA-2vr4-cq9g-pvrc).

Acceptance requires a frozen lockfile check, a clean production npm audit,
the exact-head ordinary Check including Rust cargo-audit, and the protected
merge queue's full suites before actual merge. A separate change to Node
support is not part of this dependency repair: the prior `undici@8.10.0`
already declared `node >=22.19.0` in the lockfile, and the 8.10.2 patch
retains that requirement.
