# Declared JSX routing hint in the editor conformance oracle

Issue: [#6883](https://github.com/ubugeeei-prod/vize/issues/6883).
Paired [issue decision](https://github.com/ubugeeei-prod/vize/issues/6883#issuecomment-5982952895).

The scheduled [TS-45 run 37196335410](https://github.com/ubugeeei-prod/vize/actions/runs/37196335410)
at historical source `d71d8398187dfbee63c26f1e4f2946fbe9fe38f9` built the real
server successfully. Its authenticated four client artifacts show one failing
step each: the complete initialize expectation lacks
`experimental: { vize: { jsxTypecheck: false } }`. All other fifteen steps pass;
each client rejects all sixteen corrupted expectations and has no driver error.
Neovim, Helix and VS Code use actual editors; Zed uses its declared replay mode.

## Independent contract authority

The resolved client routing hint was introduced by
[`7f728953`](https://github.com/ubugeeei-prod/vize/commit/7f728953ffc1a9fdaf0a35b2bc9824f5266d61bf).
`server/client_capabilities.rs` explicitly supplies it from the resolved
workspace JSX opt-in. The scenario has no JSX opt-in, so the declared value is
false. The separate source-built `lsp-capabilities.test.ts` already expects this
complete property. Both this production helper and the TS-45 initialize golden
are unchanged between the failing historical source and current `61c975f8`.

Add only this independently declared property to the authored initialize
golden. Preserve every other capability, scenario input, diagnostic, completion,
edit, URI and shutdown expectation. The exact comparator remains unchanged.
Focused negative controls reject an absent hint, true opt-in and a string
instead of the boolean. The existing sixteen-step corruption gate remains.
This is an authored current-contract correction, not an original historical
oracle capture or native product admission.

## Authenticated historical observations

API ZIP digest verification passed for `ts45-zed` artifact `11301337278`,
`ts45-helix` `11301312212`, `ts45-neovim` `11301162749` and `ts45-vscode`
`11300973250`. Whole-object comparison found only the missing experimental
property in every result; their complete transcripts remain historical evidence.
Re-judging retained bytes is pure comparison and does not execute a new editor
or server. Fresh source Actions and an actual four-client workflow at the new
source are required before current editor conformance is accepted.

The current-main protected run `37198344364` separately authenticated all twelve
legacy shared LSP fixtures and twenty complete responses with source `61c975f8`,
successful shutdowns and zero baseline drift. That stdio corpus remains unchanged
and grants zero native credit; it does not stand in for the editor workflow.

## Remaining delivery

Source Actions, the new-source editor workflow, the protected full suite and
104 probes, actual merge and release are still required. Queue admission stays
held through coordinated v0.433 publication and explicit thaw. Full LSP fix
history, default replacement and native admission remain unfinished in #6883.
The issue originator is the maintainer `ubugeeei`; source attribution supplies
their verified public GitHub identity without claiming a distinct external
reporter or a final squash-normalization cause.
