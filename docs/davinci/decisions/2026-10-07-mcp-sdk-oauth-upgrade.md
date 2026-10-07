# MCP SDK OAuth advisory upgrade (2026-10-07)

Issue: [#8145](https://github.com/ubugeeei-prod/vize/issues/8145).

The existing security audit reports GHSA-6qxp-vccf-f47h for the integration
catalog SDK 1.30.0. Upgrade the catalog and its lockfile identity to 1.31.0,
the first patched 1.x release, without a waiver. Registry metadata confirms
unchanged dependency, peer, and engine ranges. Preserve every resolved
transitive dependency instead of broadly upgrading unrelated packages.

Musea imports the server and stdio transport APIs. The advisory concerns
OAuth clients; the package upgrade still keeps the repository audit gate
clear without changing application behavior. Existing Musea tool tests and
package build run in Actions.

Exact-head source Actions, protected merge-group Actions and actual main
merge remain required. Refresh stale P0 candidates after this merges.
Public release and installed acceptance are separate follow-up evidence.
