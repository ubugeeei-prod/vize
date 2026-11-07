# Project files

Repository-wide policy lives here unless a hosting platform requires a fixed
location. GitHub community files are in `.github/`; upstream-derived source is
in `upstream/`; executable repository tooling is in `tools/`.

The root is reserved for entry points and files discovered there by Cargo,
Node, pnpm, Nix, Vite, or repository services. Project-specific policy should
not add another root file.

- [`redundancy.md`](./redundancy.md) records the continuity and engineering
  principles used when maintainers cannot supply implicit context.
- [`.github/CONTRIBUTING.md`](../../.github/CONTRIBUTING.md) describes the
  contribution and verification workflow.
- [`.github/GOVERNANCE.md`](../../.github/GOVERNANCE.md) and
  [`.github/SUPPORT.md`](../../.github/SUPPORT.md) describe project decisions
  and support channels.
- [`upstream/README.md`](../../upstream/README.md) distinguishes pristine
  checkouts from maintained forks.
