# Direct Node setup for the dependency direction gate

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The dependency direction gate needs Node built-ins and Cargo metadata. Its
five-minute deadline expired before any dependency checks on main
`b45d789b317933d48dadbba4841c8e353231547b`:
[job 115117177368](https://github.com/ubugeeei-prod/vize/actions/runs/38342328555/job/115117177368)
stalled at Vite+ Node installation from 03:40:15 UTC until cancellation at
03:44:41 UTC on 2026-10-13. The same gate on
[PR #8805](https://github.com/ubugeeei-prod/vize/actions/runs/38341624281/job/115113670215)
expired during Vite+ runtime preparation.

Use SHA-pinned `actions/setup-node` v6 directly with the Node version from
`package.json`. The pinned action reads `devEngines.runtime.version`, keeping
the existing runtime selection. Disable automatic package-manager caching:
this gate neither installs dependencies nor invokes a package manager.

Keep both complete mutation test files, comparison-base fetch, Cargo metadata,
allowlist ratchet, mandatory gate aggregation, runner and five-minute deadline.
The workflow regression requires direct Node setup before both existing checks
and refuses a package-manager bootstrap. Exact-head Actions and protected merge
queue success remain required; this change makes no claim about full CI speed
or broader runtime-download availability.
