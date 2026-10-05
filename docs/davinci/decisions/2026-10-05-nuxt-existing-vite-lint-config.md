# Nuxt auto-init and existing Vite configuration ownership

Paired issue: [#7829](https://github.com/ubugeeei-prod/vize/issues/7829).

## Decision

Keep generating Nuxt's build-directory lint artifact and its regeneration
hooks. Before automatically creating a root `oxlint.config.mts`, preserve
existing regular or symbolic `vite.config.js/mjs/ts/cjs/mts/cts` files in the
project or an ancestor, alongside the existing standalone Oxlint discovery.
The published `ROOT_OXLINT_CONFIG_NAMES` list and exclusive-create protection
remain unchanged.

This is a conservative file-ownership rule. Vite supports
[function and async configuration](https://vite.dev/config/#conditional-config),
so auto-init does not evaluate user configuration or infer a literal lint
block with a text heuristic. A Vite config without a literal lint block also
suppresses auto-init. This does not assert that every Vite config enables lint.
The project owner composes the generated Nuxt plan in its chosen toolchain
explicitly; this change does not automatically merge it into Vite+.

The reported bug is unwanted creation of a competing root file. Existing
standalone Oxlint files were already protected against overwrite. Recognition
of `linter` in other `vize.config.*` formats is separate unfinished work.
The existing default parent-relative ignore compatibility limit recorded in
[the Nuxt-major decision](./2026-10-05-nuxt2-lint-runtime-rules.md) remains open.

## Retained regression

The issue's rendered Vite+ snippet is retained unchanged at
`npm/framework/nuxt/src/lint/fixtures/auto-init-vite-lint/vite.config.ts`.
Its 238 bytes hash to
`347c41a34729a136b6d3e75eefd53dfdd67c46f5bfb7f000c4f12036d9abcc83`.
An independently authored missing-key Vue input exercises the configured rule.

Seventeen filesystem laws cover all six filename forms at root and ancestor,
byte/inode/mtime preservation, dynamic nonexecution, conservative plain-Vite
ownership, symbolic configuration, no-config auto-init and explicit disabling.
The unmodified old generator fails fifteen ownership laws while both existing
default/disabled controls pass; the corrected generator passes all seventeen
locally using the actual current shared preset source.

The existing locked Nuxt 2 webpack/SSR fixture and route assertions are
preserved. A separate child rooted in that real project exclusively installs
the retained Vite config and temporary Vue input, executes actual `vp lint`
before and after genuine lint-enabled Nuxt loading, and compares complete
diagnostic witnesses without filtering. It verifies the entire installed
candidate Nuxt dist inventory, retains raw outputs and the generated artifact,
and preserves original project/config bytes. Published plugin/native 0.429.1
remain a qualified configuration oracle; no current native build is claimed.
Hosted proof, protected checks, signed delivery and publication remain gates.

## Delivery checkpoint for the preceding Nuxt fix

[#7840](https://github.com/ubugeeei-prod/vize/pull/7840) actually merged as signed
`5d879b2724735749a0d045f11fa5cfcd77542922`, parent
`688da7cc620af5a9aec82152b54d8052c64c52bc`, after explicit root technical
clearance of this narrow P0 delivery during the earlier publication hold.
Protected Check 37244499170, Nuxt scoped 37244498880 and Musea 37244498853
succeeded. Check has 24 successful jobs and 15 declared skips; its delegated
four Rust shards authenticate 15,823 unique cases with zero failures/errors/skips.
The separate instruction receipts reconcile 100 + 4 benchmarks and all 312 raw
windows across three identical runs with unchanged budgets and passing ratchets.
The actual signed commit ends with the verified reporter Co-author footer
`ubugeeei <71201308+ubugeeei@users.noreply.github.com>`. These facts close only
#7828's unsupported-rule generation; publication remains coordinated separately.
They do not complete Davinci/default products, every dialect or the 10x target.
