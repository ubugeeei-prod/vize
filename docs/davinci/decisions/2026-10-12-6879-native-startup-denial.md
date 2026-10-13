# Deterministic native startup denial

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).
Companion: [native syntax/semantic completion](./2026-10-11-6879-native-syntax-semantic-completion.md).

The original unavailable-transport fixture exited 41 immediately after an
actual `--api` or `--lsp` invocation. That races the SDK's JSON-RPC writer and
reader. The complete native run on
`d7a33c60acc089b6d9545f07048317b7cab4a9a2`
[failed its original refusal assertion](https://github.com/ubugeeei-prod/vize/actions/runs/38157426440/job/114523185378):
it retained the real CLI TS1128 response and a combined semantic-completion
failure ending in `process is closed: jsonrpc writer`, whereas its existing
oracle admitted `jsonrpc connection`. Keep that failed packet as evidence;
neither closed-channel variant describes one deterministic startup denial.

## Bounded test fixture

Keep the wrapper under its existing `.bin` path, preserving the SDK's actual
asynchronous JSON-RPC transport selection. Ordinary CLI calls still execute
the identical real native binary with every original argument. Only actual
`--api` and `--lsp` calls enter the intentionally unavailable test peer.

The TypeScript peer reads the complete real `Content-Length` request frame
before closing any pipe. It requires `jsonrpc: "2.0"`, `method: "initialize"`
and the actual request ID. It retains the whole argv and both framed byte
vectors, sends exactly error code `-32041` and message
`native diagnostic transport intentionally unavailable` for that ID with
synchronous writes, and exits 41. It emits no successful native answer or
replacement semantic diagnostic. The SDK registers this pending request
before writing; its reader consumes the complete error frame before EOF.

Retain the actual startup runtime's Node version, executable SHA-256 and whole
peer source. Replay the captured real bootstrap, including its original ID
and argv, through the identical wrapper. Require byte-exact whole stdout,
whole stderr and process exit 41, plus identical runtime/source identity.
Retain every original and replay packet before asserting the verdict.
Bind every whole original transport-trace record to the captured argv by
exact count and contents; PID filenames grant no invocation-order claim.
Require the replay packet's entire response frame to equal the original too.

The unavailable-transport law still requires the whole genuine CLI response
`post-request.d.cts(1294,1): error TS1128: Declaration or statement expected.`,
native CLI exit 1 and empty stderr. Its complete combined failure must end in
the exact typed RPC startup denial. There is no prefix-only match, list of
closed-channel alternatives, benign nonzero outcome or empty clean result.
The ordinary semantic fast-path law still requires no diagnostic transport.

## Validation and delivery

This isolated slice starts from authenticated main
`67e3d0e6775cc3a1f29bfb9309d991252f9e75ff`. Only the two existing integration
test files, the new test peer and this decision change. The explicitly
reserved existing foundation row 43 gains only this document link; all other
349 rows and all incoming text remain intact. No production, SDK, fixture
source, diagnostic golden or performance ceiling changes.
Publication replays this fixture on freshly authenticated main
`ce5f407ebb10517a0797e15b2e07845fd52003b5`, preserving its incoming foundation
rows. The local observation below remains evidence from the initial base;
the refreshed source requires its own hosted qualification.

The locked targeted integration law passes locally on Rust 1.99's default
test profile: one passed, none failed or ignored, three filtered. It uses the
genuine locked TypeScript 7.0.2 and Vue 3.5.35 packages and actual Node
24.14.0. The real SDK sends `--api --async --cwd` and an 82-byte initialize
frame with ID 1; the denial response is 137 bytes. Direct replay returns 41
with identical complete framed output, stderr and runtime/source identity.
Both the complete generated-project packet and original/replay refusal packet
are retained; the original whole TS1128 and combined typed failure remain
exact. This observation contains one actual API startup denial and grants no
LSP invocation credit.

TODO: obtain exact-source Linux full native qualification. A focused local
success does not replace the
complete hosted gate, prove provider/public CLI parity, complete #6879 or
prove merge/release acceptance. The provider Stack's independent Vue Fes and
ReKa reference laws and original full App failures remain separate.
