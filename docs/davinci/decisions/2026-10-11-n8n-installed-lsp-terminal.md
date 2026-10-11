# Installed n8n LSP terminal custody

The installed slot-policy adapter must validate the actual child result after
normal shutdown. The shared native-journal validator requires both an observed
exit status and signal. The original LSP caller supplied a literal status zero
and omitted the signal, so it rejected a successful public process.

Use the complete existing `LspWire` observation before journal validation. Pass
its actual `exitStatus`, `signal` and retained `processError`; refuse an absent
exit status. The existing shutdown, cleanup, raw wire capture, native identity,
public payload and collector checks remain unchanged. No new launch or provider
stage is added.

## Whole public control

`tests/tooling/support/n8n-installed-lsp-terminal-control.ts` accepts a new output
directory and an independently reviewed installed-campaign plan. It initializes
the actual public LSP with editor, lint and typecheck disabled, then requests
normal shutdown. Complete responses, raw wire, native journal, terminal result
and any failure are retained before the desired law is judged.

The unchanged control was executed before and after the adapter correction with
the existing single official Darwin ARM64 0.441.0 install. Both real processes
exited with status zero, no signal and no transport error. Before the correction,
the native validator rejected the omitted signal; after it, normal terminal
custody passed. The install receipt remained
`d9a29f1029bbd386b722d7ddbcc5fe876649050fdf78c19bd913b193eb29557b`;
all 202 public payload files and the lock were rehashed unchanged.

The launch records authenticate the installed process and native journal. They
do not embed a per-launch adapter/control source hash; the unchanged-control
claim is recorded by the paired receipt and final source snapshot.

This is an adapter control, not slot-policy, editor or complete adoption
qualification. No installation, collector or local Rust build was repeated.
The 75 CLI/50 JSON-RPC slot-option campaign still requires its own genuinely
including public release and official receipt. The retained parameter producer
and default-binding child remain separately qualified source work. Refs #8142.
