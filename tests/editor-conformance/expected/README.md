# Current protocol expectations

`scenario.json` selects `initialize-current.json` and `completion-current.json`.
The original `initialize.json` and `completion.json` remain byte-for-byte unchanged
as the prior complete vectors. Every other scenario expectation is unchanged.

The current initialization vector derives from the original vector by adding the
three source-action kinds required by
`crates/vize_maestro/src/server/capabilities/code_action_tests.rs`, the moniker
provider advertised in `crates/vize_maestro/src/server/capabilities.rs`, and the
enabled JSX routing hint from `projectConfigDefaults` in
`crates/vize_carton/src/config/loader/vite-runtime.mjs`. The shared real Vue
workspace selects this default through its canonical `vite.config.mjs`.

The current completion vector retains both original authored candidates first,
then appends the complete independent vector from
`tests/_fixtures/differential/lsp/template-expression-globals-8015/globals.expected.json`.
`crates/vize_maestro/src/ide/completion/template/globals.rs` defines the Vue
instance members and Vue expression global allowlist. No candidates or fields
are removed from the comparison.

All fields in these canonical Vite vectors matched the recorded Neovim, Zed, and
VS Code responses in [Editor conformance run 38936651085](https://github.com/ubugeeei-prod/vize/actions/runs/38936651085)
at commit `37be818f1699a83675a79a6d35056f70ce9c13f8`. This records equality for
that fixture and source; it does not qualify a later combined commit.

The prior main fixture retained a disabled JSX routing hint. Its
[complete initialization vector](https://github.com/ubugeeei-prod/vize/blob/ee7f8575c7e0476ac70a545bffa372b6b65270f6/tests/editor-conformance/expected/initialize-current.json)
and completion vector matched the complete actual Neovim, Zed, and Helix responses
in [Editor conformance run 38999187468](https://github.com/ubugeeei-prod/vize/actions/runs/38999187468)
at main-based commit `d8f46f463efb0de5b1bbf0de653d855e5b26331d`. The original
expectations failed initialization and completion in that run. Rejudging its
original 58-entry Helix transcript with those vectors passed all 16 steps and
rejected all 16 corrupted expectations; that replay is not a new native editor run.

The judge still compares complete responses and rejects every corrupted scenario
expectation, including a missing, false, or incorrectly typed JSX routing hint
for the current canonical Vite fixture.

Code-action diagnostics use the scenario's explicit `diagnosticsFromRequest`
contract. Each complete canonical diagnostic must first match a server-published
diagnostic. Its source, rule code, and range then identify the client diagnostic
that the action must echo in full. This preserves optional fields exactly as the
client sent them, including an omitted documentation URL or supplied metadata.
The judge does not alter observed responses, remove fields, or branch by client.
Published diagnostics and all other code-action fields retain their complete
static expectations.
