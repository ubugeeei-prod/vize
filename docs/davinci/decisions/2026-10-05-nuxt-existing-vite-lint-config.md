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
preserved. A separate child retains all five original tracked project files outside the
repository workspace and links the authentic installed fixture packages plus
pinned Vite+ tooling. It exclusively installs the retained Vite config and
temporary Vue input, executes actual `vp lint`
before and after genuine lint-enabled Nuxt loading, and compares complete
diagnostic witnesses without filtering. It verifies the entire installed
candidate Nuxt dist inventory, retains raw outputs and the generated artifact,
and preserves original project/config bytes. Published plugin/native 0.429.1
remain a qualified configuration oracle; no current native build is claimed.
Hosted proof, protected checks, signed delivery and publication remain gates.

Initial source `db59c54a`, Nuxt 2 run `37246574823`, passed all 135 unit laws,
the original webpack/SSR assertions and the inherited genuine Nuxt 2/3/4 plan
controls. Its new Vite+ before probe returned exit 0 and no diagnostics, so it
failed before new Nuxt loading; that is retained failed acceptance, not a
positive configuration proof. Pinned Vite+ 0.1.24 documents
[workspace-root configuration with overrides](https://github.com/voidzero-dev/vite-plus/blob/v0.1.24/docs/config/lint.md).
The nested fixture selected repository lint authority. An independent tiny
standalone control with the exact original 238-byte config attempts its bare
plugin load, establishing a different configuration path. The correction
copies the complete five-file original project outside workspace discovery,
links the real installed packages and checks every original file hash again
after genuine Nuxt loading. It preserves the positive missing-key expectation,
complete before/after witnesses, original config bytes and all production laws.
Fresh hosted execution remains required; no zero-diagnostic assertion replaces
the failed positive probe.

## Source qualification and fresh-main reconciliation

At source `710d0f8ab2dfcd4763f3b1765fa13df66b49ba83`, actual PR composition
`69ae75b730624d436e067d233e877bb50a5cae62` runs on main `85463bfe`.
Nuxt 2 run `37247495592` passes all 135 laws, original webpack/SSR routes and
inherited complete Nuxt 2/3/4 controls (0/2/2). Artifact `11319771970` is
61,537 bytes, ZIP SHA-256
`06a028a076dcf8bd7d8222aa5ec2e474d7d98ccc2dc6133b29a8f1ec2a569369`;
API hash and all CRCs authenticate the exact original 238-byte Vite config,
all five original project files and complete nine-file candidate dist.
Real Vite+ 0.1.24 emits one complete missing-key error before and after genuine
Nuxt 2.17.3 default lint loading, with identical message/help/causes/related
and offset/length/line/column labels. No competing wrapper is created;
configuration bytes/inode/mtime and original project bytes remain unchanged.
Published plugin/native 0.429.1 remain the stated configuration oracle.

Core Check `37247496031` passes 24 jobs with 15 declared skips. Its four
API-hash/CRC-authenticated source JUnits retain 15,805 unique passing cases,
zero failures/errors/skips; this is not protected instruction execution.
Nuxt 3/4 and CodeQL source checks also pass. Root reviewed and adopted this
bounded ownership behavior and authenticated original-authority packet.

The two meaningful commits are genuinely rebased onto literal main
`d1a25ec1da2efca98534520ff8ebe84d34708e3d`. All ten owned non-canonical blobs
are initially byte-identical to `710d0f8ab2`; only this companion receipt is
then extended. Every incoming canonical native Stack/source-binding clause
is preserved within the unchanged 350-line record. Original project/source,
whole stock lint witnesses, explicit plain-Vite/manual-composition policy
and all budgets remain unchanged. This historical qualification does not
supply current-head CI: fresh automatic Actions must qualify the rebased head.
Queue admission waits for the first release's verified publication and thaw;
then current protected full Rust and unchanged 100 + 4 instruction gates,
actual signed merge/literal verified reporter footer and issue closure remain
required. The delivered fix is handed to the next frequent legacy release.

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
