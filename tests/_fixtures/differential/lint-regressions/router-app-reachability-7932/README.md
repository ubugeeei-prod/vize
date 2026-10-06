# Router application reachability (#7932)

The original public issue is retained in `original/issue.json`, its authenticated
capture in `original/authenticated-issue.json`, and its literal body in
`original/issue.md`. The reporter is GitHub user 71201308. The two component/test
sources and `original/command.sh` are exact fenced payloads, including their LF.
`nuxt.config.ts` contains the exact inline payload followed by one LF.

The reporter specified page paths but no page content. The two files under
`supplemental/pages/` are minimal authored pages, staged at those original paths.
Every other file under `supplemental/` is an authored coverage control, not a
reporter-provided source. Each staged input has its provenance and SHA-256 in
`cases.json`; no original fixture is rewritten for a control.

Every invocation preserves the original `lint --cross-file src/**/*.vue
src/**/*.ts` arguments and adds only `--format json` to observe the complete
public report. The test checks the whole stdout bytes/JSON vector, empty stderr,
exit status, every reported source row and unchanged whole authored inputs. Raw
streams, before/after inputs, whole expected/current values and status persist
under `target/nextest/$NEXTEST_PROFILE/router-app-reachability-7932/` (default
`full`); every case runs before one aggregate terminal assertion.
Expectations are authored from the pinned source in `source-authority.json`,
including exact messages, primary ranges, severities and plain-text help. They
have no update/recapture option. This author performed no checker execution.

The original Nuxt case uses the issue's allowed conservative outcome: without
an established application installation, application router injection stays
unknown. This does not claim page-route inference. Installed Vue/SSR applications
are checked only through their own static root/component/runtime-import and
route-component edges. Test routers, orphan files and type-only imports do not
supply application routing. Directly known router calls retain all four original
route diagnostics independently of installation. Uninstalled, dynamic,
conditional, logical, factory, ambiguous and shadowed bindings remain unknown.
Distinct roots and separately invoked projects stay independent. A shared
component reached by two independently proven applications accepts both routers
and retains each table’s parameter diagnostics.
