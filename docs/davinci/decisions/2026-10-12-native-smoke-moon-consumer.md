# Native smoke MoonBit consumer selection

Decision for [#6830](https://github.com/ubugeeei-prod/vize/issues/6830), 2026-10-12.

The default Native Smoke matrix runs six host checks and twelve fresh-install
checks. Their actual Rust/N-API, CLI, glibc, package-install and runtime recipes
do not consume MoonBit. Keep all eighteen checks and their commands, inputs,
pins, outputs, deadlines and platform vectors. Run the existing MoonBit setup
only for the selected producer that executes its actual Moon command.

## Source and consumer boundaries

The original failed source `5f8d486ff7707936cb196603ecfb1c8d343a3c22` had no
`host_only` selector or selected Moon consumer. Current main
`4ae56daecf4cc31d2d2558b92451b07d448bc259` shares its host-step anchor with
`selected-host-native-producer`. That producer executes
`moon test --target native --deny-warn cmd/github/build_napi_package` in
`tools/moon`, including all three actual native-token laws. Its exact condition
is `workflow_dispatch && host_only && target == 'darwin-arm64'`. Use that
existing condition for setup as well; the default six-plus-twelve matrix stays
active for every other context. Native Smoke retains only its existing
dispatch and schedule triggers; the authored push-context guard does not add
a push trigger.

The authored law reverses only the setup predicate and authenticates the whole
incoming workflow against SHA256
`25d6dfff5052a85bdbd673af0872ee65ed5f72d5dc3dccb5d792dc78ab138ce5`.
It checks all 72 selector contexts and the actual shared anchor, preserves the
three Moon law bodies, and rejects removed CLI, glibc, runtime or Moon commands.
The same five-law driver rejects the actual original workflow and accepts the
predicate change, with no skipped laws or exempted stale source object.

The real exported NAPI producer executes under a controlled spawn callback;
its complete command and Node synchronizer remain unchanged. Inspect its
seven-file recursive local JavaScript closure, actual Rust-script support
bodies and local setup actions. A conservative manifest traversal includes
optional dependencies, all target-specific normal/build dependencies and
all root local Cargo patch candidates, including `vendor/oxc_parser_compat`.
Its 34 repository packages exclude the Moon dialect; the only local build
scripts are Vitrine and Fresco, with no Moon commands. This is a repository
source census, not an audit of every external registry build script.
The dialect's optional `moonc` feature remains off by default. Dedicated
Level MoonBit qualification, the setup helper, `.moonbit-version`, Nix archive
hashes and selected producer source remain unchanged.

## Failure and environment boundaries

[Original run 38187396126](https://github.com/ubugeeei-prod/vize/actions/runs/38187396126)
remains historical. Linux ARM64 job `114612592131` missed its Moon cache and
received HTTP 403 from the authenticated installer for pinned
`0.10.7+bc794d341`. Windows ARM64 failed in outer PowerShell with CLR
`0x80131506` before the installer. Darwin x64's Vite+ DNS timeout was another
failure. Do not attribute those three distinct failures to one provider cause.

Prior Linux ARM64 success used a warm cache and does not prove archive
availability. The original run's Linux x64 job successfully downloaded the
pinned binary/core through a cold installer; local HEAD failures do not prove
a universal GET outage. Provider availability and its ARM64 failure cause
remain unknown. No archive, pin, mirror, retry or checksum policy changes.

Default Windows rows now omit the setup helper's MSVC `amd64` override. The
caller's matrix-selected `arm64` toolchain remains authoritative for Windows
ARM64. This is an intended environment change; original and current runs are
not claimed to be byte-equivalent. Current main's selected dispatch and native
token environment also differ from the original failed source.

## Remaining qualification

Initial source `a69685f3d9f7c56bb8dda11b2d55e21affcd0493` genuinely failed
[tooling shard two](https://github.com/ubugeeei-prod/vize/actions/runs/38189757116/job/114620319467):
the existing native-image guard compared the new setup condition with the
original matrix condition. Its full original fixture/hash stays unchanged.
The narrow test successor asserts the actual approved Moon selector and all
72 contexts before reversing only that predicate for the original whole-job
comparison. Every other field remains present; the same validator rejects an
unknown setup field and a removed CLI command. The seven-law suite has six
passes and one failure before this correction, then seven passes and zero
failures after it, with no skipped laws.
Keep the original failure; requalify the actual successor rather than rerun or
borrow the first source's passing Moon laws.

Source `9057aadee2ef1677c27e3b8c4321b67fd0724c2c` passed its source Check,
including all seven actual laws, but its
[native phase](https://github.com/ubugeeei-prod/vize/actions/runs/38191166695/job/114624034609)
failed with status 127: the current workflow called the lexical dollar helper
absent from that older source checkout. The helper had genuinely merged with
the corresponding product fix in #8716. Preserve the complete original failure
and replay the owned slice onto actual current main
`47b204f26839afb27af4a2f789eafc86295f0f81`, including that whole producer
ancestry. Keep all owned workflow/test bytes and every incoming canonical row;
do not splice a foreign helper, skip its checks or retry the old head. Both
fresh source Check and actual native-phase qualification remain required.

- Obtain fresh exact-head Actions and protected-queue qualification, then
  verify actual merge of this prerequisite.
- Run one complete eighteen-check Native Smoke qualification on the genuine
  combined initializer source with this prerequisite. A selected-only producer
  run cannot supply full-matrix credit; do not duplicate that expensive run.
- Keep every original failure and require terminal platform results. Local
  source laws, queue admission and a published release grant no full native,
  installed initializer or third-party issue completion credit by themselves.
