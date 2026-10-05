# Zed workspace profile — 2026-10-05

Issue: [#8007](https://github.com/ubugeeei-prod/vize/issues/8007).
Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8007#issuecomment-5992284213).
Packaging follow-up: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8007#issuecomment-5992554915).
Current-wire follow-up: [issue comment](https://github.com/ubugeeei-prod/vize/issues/8007#issuecomment-5992831553).
Prior VS Code rule: [#7196](https://github.com/ubugeeei-prod/vize/issues/7196),
commit `6496dd1ea4134f1a8a36dcef593bcaa7efcf5812`.

The Zed extension always supplied the recommended profile when explicit
initialization options were absent. The server correctly gives those options
per-key precedence over the project's config, so that invented profile
reenabled the reporter's disabled hover and other project choices.

Preserve any explicit `lsp.vize.initialization_options` value verbatim, including
an empty object, without reading files. Otherwise use the pinned Zed 0.7.0 SDK's
root-relative `Worktree::read_text_file` for the exact CLI/VS Code filenames:
`vize.config.pkl`, `.ts`, `.js`, `.mjs`, `.json`. A readable file returns `{}`;
no readable file keeps the existing recommended defaults. Config contents and
error strings are not parsed. There is no parent search, path heuristic,
server precedence change, project config rewrite, or additional pipeline stage.
The SDK exposes a string error rather than a typed existence probe; this
detection is qualified for readable worktree config files.

Extract the unchanged recommended helper in a separate move-only commit. The
extension callback and the existing Zed real-server launcher then share the
exact selector. Four native unit controls retain all five names, untouched
explicit values without reads, the no-file default, and unrelated paths.

The independent source peer found that the strict archive contract allowed only
`src/lib.rs` and expected the moved profile literals there. Require and allow
the new module, read its unchanged profile literals there, and preserve every
existing archive safety, version, grammar and API check. The recursive tar
producer already includes it. The adapter test's original recommended-profile
assertions now read the same module; all other editor routing laws stay exact.
Packaged delivery must qualify on current Actions.

The corpus retains both original issue configs byte-for-byte. Neither report
specifies a full SFC, so the complete Vue file is explicitly an authored control.
Six real stdio cases pin actual initialization options, hover/formatting
capabilities and complete hover replies for project defaults, partial explicit
options, explicit empty options, explicit recommended options, no config, and
the original VS Code report. The fixture uses a real Vue/TypeScript workspace
and discovered runtime link without inserting a runtime key into either config.
The existing complete editor contract remains intact. Complete observed input,
initialization and hover packets are printed for current-source Actions custody.

First actual Actions executed the 18 extension units, strict archive contract,
official WASM validation and original whole editor contract successfully. The
new observer failed because it expected the report's `hoverProvider: false`,
while the current unchanged `capabilities.rs` uses `then_some(true)` and omits
disabled providers. Pin exact key absence separately from JSON null, retain
enabled providers as present/true, and keep every complete hover response and
all six cases unchanged. This is a current-wire observation correction, not a
server change or a false/null union allowance. Apply the current formatter's
chain layout to the input-custody Node test. Fresh successor execution remains
required; an incoming non-native `stable_revision` Clippy failure is coordinated
with its backend owner and is not waived or patched in this Zed-only change.

The opaque Zed `Worktree` is not fabricated in the native process fixture:
filesystem reads exercise the shared selector; official pinned Zed CLI WASM
validation checks the real SDK callback separately. This is not Zed GUI-host
execution or new native-stage eligibility. The README explains project-owned
defaults and deliberate explicit overrides.

TODO: frozen source peer, current exact-head ordinary Check, existing editor
packaging/unit and official CLI/real-server Actions, protected full Rust/all104,
actual signed reporter-credited merge and the next supported release remain
required. Local native builds and large installs are not used. Preserve all
incoming canonical decisions and the independently completed #7960 proofs.
The rejected logo #7850 remains closed and unchanged.
