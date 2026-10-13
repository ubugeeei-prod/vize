# Authenticated native shutdown and EOF

This #6879/#3984 decision pairs the complete native semantic laws with an
explicit close capability for two authenticated TypeScript-go 7.0.2 binaries.

The pinned [native server source](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/lsp/server.go)
handles exit by canceling the dispatch loop's local context. Its next queue read
returns `context.Canceled`; `Run` reports whichever grouped error wins first.
An input EOF can instead win and return exit 0. The source explains the observed
race; the exact source does not make a nonzero native process successful.

Initial SDK source `041d5df` Actions had two existing genuine owner laws fail
with acknowledged shutdown, complete/reaped native exit 1 and full
`context canceled\n`. Every packet remained a refusal. Retained genuine local
protocol experiments independently include three configured shutdown-response
then stdin-EOF runs, all native exit 0, no kill and complete empty stderr.
Keeping stdin open after the standard exit produced native exit 1, so delaying
or waiting for standard exit is not the repair.

Canon requires both a whole actual executable SHA256 and equality of the entire
genuine initialize result: all 29 capability fields and the complete
`{name: "typescript-go", version: "7.0.2"}` server identity. All 14 retained
Linux results and the configured Darwin protocol response agree on that whole
value. Missing/changed capabilities, including a foreign position encoding,
cannot select the capability. Reviewed binaries are:

| Platform     | Complete binary SHA256                                             |
| ------------ | ------------------------------------------------------------------ |
| Linux x64    | `4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4` |
| Darwin arm64 | `a82f731365ad69d5c4c15f5e18fba4584bf3b7b839960172a76c3462b5114bf2` |

The Linux executable is the complete 24,101,026-byte ELF x64 member of the
official `@typescript/typescript-linux-x64@7.0.2` package. Its binary and
1,042-byte package manifest match the authenticated successful native source
artifact. The registry SHA1 and SHA512 tarball integrity were independently
verified. The Darwin executable is the complete 23,653,616-byte arm64 member
used in the three physical EOF experiments. Version output alone is not an
identity authority.

The authenticated successful Linux native qualification for source `b32ae8b`
also contains 14 genuine stock-native terminal transcripts. Its source-bound
session helper receives the complete shutdown result, closes its owned stdin,
then polls the physical child and joins both readers. All 14 finish exit 0
with empty stderr; deadline/error cleanup cannot count as success. This is
acknowledged shutdown followed by stdin EOF, not a claim that standard Exit
notifications succeed or that attached API-only processes share this contract.

The Linux executable's embedded Go build information identifies revision
`2bd066d87f5bafd315be9f40889d0a60b9e58e0b` and `vcs.modified=true`.
This is published exact-binary custody; no clean source reconstruction is
claimed from registry metadata or the embedded revision.

A missing/changed/unknown binary, foreign or absent initialize identity and
other versions do not acquire the capability. The capability requests the
params-less shutdown response, then closes stdin and drains/reaps the actual
owned process. A failed shutdown request still retires the process and remains
the full original failure. The diagnosing API attachment closes after reaping;
its reader cannot wait on a still-live owner. Default SDK shutdown/exit behavior
remains unchanged, including other native platforms and unverified user servers.

The native same-owner laws retain complete initialize/config/source/diagnostic
packets, the actual PID, loaded 55-byte leaf declaration and ordinary-import
repair. Only complete exit 0 with no kill, timeout or reader/process error can
accept a per-call result. `context canceled`, exit 1/17, panic and any anomalous
stderr retain their complete packets. No nonzero receipt becomes benign.

Negative controls use a separately authenticated test adapter because a wrapper
does not match the official binary digest. It keeps standard parent SDK protocol,
forwards complete frames to the real child, and translates only the retained
exact exit notification after native shutdown acknowledgment into child EOF.
It records both processes and all stream bytes, drains and reaps the child,
then emits controlled extra bytes or wrapper exit 17. The wrapper's EOF bytes
and status have explicit wrapper provenance. Timeout/kill receives no credit.

Source review and local locked compilation give no hosted qualification. Fresh
exact-head Actions, all original 400 requests/diagnostics/budgets and protected
full delivery remain required. Unknown-platform lifecycle behavior, API-only
owners, broader performance targets and release acceptance remain unfinished.
