---
title: Contributing
---

# Contributing

Thanks for helping make Vize sharper. The project is in its **Real World Testing** phase and moving
toward v1 alpha, so small, focused changes with clear verification are the easiest to review. If you
are here to report findings rather than open a PR, start with the
[Testing & Feedback](./guide/testing.md) guide.

## Setup

Use the Node.js version from `.node-version` and the Rust version from `rust-toolchain.toml`. The
workspace declares a minimum supported Rust version (MSRV) of `1.95.0` in `Cargo.toml`
(`[workspace.package].rust-version`); contributions must compile under that version.

The default Nix shell contains the reproducible local toolchain. Blacksmith Testbox support is
optional and lives in a separate shell with the pinned Blacksmith CLI, `rsync`, and GitHub CLI:

```sh
nix develop             # local development
nix develop .#testbox   # hosted Testbox workflows
```

Install dependencies from the workspace root:

```sh
vp install --frozen-lockfile --prefer-offline
```

If `vp` is not available yet, install [Vite+](https://viteplus.dev/guide/install) first.

## Common Checks

Run the narrowest check that covers your change, then broaden when you touch shared behavior.

```sh
vp check <changed-files>
node --test tests/tooling/<test-file>.test.ts
cargo fmt --all -- --check
cargo test -p <crate>
```

Before opening a PR that changes shared tooling, release automation, native bindings, or compiler
behavior, run the relevant workspace task from CI locally when practical.

The root build, test, and lint workflows are local by default and need no hosted credentials:

```sh
vp run --workspace-root build
vp run --workspace-root test
vp run --workspace-root lint
```

Inside the Nix development shell, `vp build`, `vp test`, and `vp lint` are shorthand for these
workspace tasks.

For one-command Linux CI parity, enter the dedicated Testbox shell. The default `nix develop` shell
intentionally omits Blacksmith and does not need its hosted artifact or credentials:

```sh
nix develop .#testbox
```

Then run the guarded lifecycle below. It clears any old box ID before warmup, skips remote tasks if
authentication, push, or warmup fails, and always attempts to stop a successfully warmed box even
when a task fails:

```sh
run_testbox_checks() {
  unset BLACKSMITH_TESTBOX_ID testbox_output
  "$VIZE_BLACKSMITH_BIN" auth login || return
  git push --set-upstream origin "$(git branch --show-current)" || return

  if testbox_output="$(vp run --workspace-root testbox:warmup)"; then
    BLACKSMITH_TESTBOX_ID="$(printf '%s\n' "$testbox_output" | tail -n1)"
  else
    warmup_status=$?
    unset testbox_output
    return "$warmup_status"
  fi
  if [ -z "$BLACKSMITH_TESTBOX_ID" ]; then
    printf '%s\n' "Testbox warmup returned no box id." >&2
    unset BLACKSMITH_TESTBOX_ID testbox_output
    return 1
  fi
  export BLACKSMITH_TESTBOX_ID

  if vp run --workspace-root build:testbox &&
    vp run --workspace-root test:testbox &&
    vp run --workspace-root lint:testbox; then
    testbox_status=0
  else
    testbox_status=$?
  fi
  if vp run --workspace-root testbox:stop; then
    stop_status=0
  else
    stop_status=$?
  fi
  unset BLACKSMITH_TESTBOX_ID testbox_output

  if [ "$testbox_status" -ne 0 ]; then
    return "$testbox_status"
  fi
  return "$stop_status"
}
run_testbox_checks
```

For Blacksmith Testbox job changes, also validate the workflow shape with
`node --test tests/tooling/github-workflows.test.ts`.

## Language Processor Change Discipline

Vize follows compiler-project practice from rustc, TypeScript, TypeScript-Go, and Flow: classify the
change, add the smallest meaningful fixture, review generated output as a contract, then broaden to
parity, performance, or release gates when the touched surface needs it. See
[Language Engineering Practices](./architecture/language-engineering-practices.md) for the full
matrix.

Use one of these change classes in PRs when applicable:

- Parser or AST
- Compiler and codegen
- Semantic analysis, lint, and cross-file analysis
- Virtual TypeScript and type checking
- Formatter and LSP
- Runtime packaging, release, or docs

For language-facing changes, include the fixture or snapshot diff that proves the behavior. For
snapshot refreshes, explain why the new output is correct and avoid broad baseline churn unless the
PR is specifically about that output family.

When a compiler mismatch starts from an external repro or a local project file, use the playground
[Compiler Inspector](./guide/compiler-inspector.md) to inspect the official Vue output, Vize output,
Virtual TS, VIR, and cross-file graph. Add the inspector permalink to the PR body, then land the
minimized fixture or full snapshot that turns the output into a reviewed contract. Local batches can
be packaged with `vize inspector <file-or-glob>`, and agent handoff can use
`vize inspector --format agent`.

## Pull Requests

- Use Conventional Commits for commit messages and PR titles, such as
  `fix(vite-plugin): surface SFC compile errors`.
- Keep PRs focused on one behavioral change or one documentation/governance change.
- Include verification commands in the PR body.
- Do not refresh large snapshot baselines unless the PR is specifically about those outputs.
- Do not include secrets, registry tokens, private vulnerability details, or machine-local paths in
  reports, commits, or PRs.

## Fix Requests

Use the fix report template for regressions, crashes, incorrect diagnostics, package installation
problems, and release failures. Use the feature request template for new integrations, API changes,
or workflow improvements. A minimal reproduction — ideally a playground inspector link — makes a
report much faster to act on.

Security reports should follow
[`SECURITY.md`](https://github.com/ubugeeei-prod/vize/blob/main/SECURITY.md) instead of the public
fix templates.

## Code of Conduct and Governance

By participating, you agree to abide by the
[Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). The
governance model and decision-making process are documented in
[`GOVERNANCE.md`](https://github.com/ubugeeei-prod/vize/blob/main/GOVERNANCE.md). For help finding
the right channel, see [`SUPPORT.md`](https://github.com/ubugeeei-prod/vize/blob/main/SUPPORT.md).
