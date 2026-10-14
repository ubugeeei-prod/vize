# TypeScript release and publishing commands

This [#8840](https://github.com/ubugeeei-prod/vize/issues/8840) decision moves
the release and publishing commands from MoonBit packages under
`tools/moon/cmd/` to strictly typed TypeScript that Node runs directly
through native type stripping. It follows the maintainer decision to stop
using MoonBit for repository tooling. The MoonBit dialect product fixtures are
product tests and stay untouched.

## Scope

Each command keeps its name as a directory under `tools/support/release/`,
with a `main.ts` entry point. `tools/support` is where the repository keeps
TypeScript support tools. `tools/commands` stays reserved for Rust Script
entry points, which `tools/commands/ci/verify-tool-layout.rs` enforces.

| MoonBit package                               | TypeScript entry point                                       |
| --------------------------------------------- | ------------------------------------------------------------ |
| `tools/moon/cmd/publish_crates`               | `tools/support/release/publish_crates/main.ts`               |
| `tools/moon/cmd/publish_npm_package`          | `tools/support/release/publish_npm_package/main.ts`          |
| `tools/moon/cmd/publish_npm_package_dirs`     | `tools/support/release/publish_npm_package_dirs/main.ts`     |
| `tools/moon/cmd/publish_vscode_extension`     | `tools/support/release/publish_vscode_extension/main.ts`     |
| `tools/moon/cmd/publish_open_vsx_extension`   | `tools/support/release/publish_open_vsx_extension/main.ts`   |
| `tools/moon/cmd/prepare_npm_publish_manifest` | `tools/support/release/prepare_npm_publish_manifest/main.ts` |
| `tools/moon/cmd/release`                      | `tools/support/release/release/main.ts`                      |

Workflows, the Vite+ release tasks and the Rust release driver now invoke
`node tools/support/release/<command>/main.ts`. The shared `command_runtime.ts`,
`command_io.ts` and `ordered_json.ts` modules replace the MoonBit `@tool`
helper and the parts of the MoonBit runtime the commands observably relied on.

## Behavior that stays identical

The port keeps arguments, exit codes, spawned commands and their order, and
written bytes. It reproduces these MoonBit runtime details:

- `@string.parse_int`: signs, `0x`/`0o`/`0b` prefixes, underscores and 32-bit
  range, plus 32-bit wrapping in version bumps;
- `trim`, which removes only tab, newline, carriage return and space;
- raw `readdir` order, and the `opendir: <error>` line a failed directory read
  printed;
- `@json` parsing and serialization: member order including integer-like
  keys, a nesting limit of 1024, escapes, number formatting, and unsafe
  integers and overflowing numbers written back literally;
- the `OSError("@process.run(): ...")` text for a child that cannot start.

Side-by-side runs of the original MoonBit binaries and the TypeScript commands
compared exit status, standard output lines, standard error, every faked child
invocation and every resulting file across 439 scenarios. Only the macOS
signal case below differed.

## Deliberate differences

- **Registry freshness.** With `VIZE_RELEASE_REGISTRY_REFRESH=1`, the
  MoonBit `release` command ran `moon update` before acting. That only kept
  MoonBit's own package registry fresh, so the TypeScript command and the
  `vp run release` task drop it, together with its tests.
- **Output order.** MoonBit `println` was block-buffered on pipes, while
  direct writes and child output were not. The TypeScript commands write in
  program order. Line content is unchanged.
- **Signals.** A child killed by a signal now reports the signal number, as
  the original runtime did on Linux runners. On macOS the original reported 0.
- **Uncaught errors.** Failures other than a failed child start print a
  JavaScript stack on standard error instead of MoonBit's error value on
  standard output. Both exit with status 1.
- **Test seam.** The node runtime for child publishers is `NODE_BIN`
  (default: the running Node) instead of `MOON_BIN`.

## Older sources

Some paths still run sources from before this change: published tags in the
Open VSX workflow and the crate handoff job, stable patch cuts prepared by
`tools/support/release/pr_start.rs`, the public acceptance plan for older
release heads, and pinned catalogs of older revisions. Each uses the
TypeScript command when it exists in that source and the original MoonBit
package otherwise. These fallbacks need the shared MoonBit setup until the
final MoonBit removal decides whether such older sources remain supported.

The frozen warm type-backed release bridge recipe still names the MoonBit
release files, so it continues to authenticate only the source cuts it
reviewed. A new release bridge needs a new recipe review.
