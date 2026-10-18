# Current protocol expectations

`scenario.json` selects `initialize-current.json` and `completion-current.json`.
The original `initialize.json` and `completion.json` remain byte-for-byte unchanged
as the prior complete vectors. Every other scenario expectation is unchanged.

The current initialization vector derives from the original vector by adding the
three source-action kinds required by
`crates/vize_maestro/src/server/capabilities/code_action_tests.rs` and the moniker
provider advertised in `crates/vize_maestro/src/server/capabilities.rs`.
The existing main fixture retains its disabled JSX routing hint. Unlike the
canonical Vite configuration fixture on the configuration migration branch, this
fixture does not select that branch's enabled JSX checking default.

The current completion vector retains both original authored candidates first,
then appends the complete independent vector from
`tests/_fixtures/differential/lsp/template-expression-globals-8015/globals.expected.json`.
`crates/vize_maestro/src/ide/completion/template/globals.rs` defines the Vue
instance members and Vue expression global allowlist. No candidates or fields
are removed from the comparison.

Both derived vectors matched the complete actual Neovim, Zed, and Helix responses
in [Editor conformance run 38999187468](https://github.com/ubugeeei-prod/vize/actions/runs/38999187468)
at main-based commit `d8f46f463efb0de5b1bbf0de653d855e5b26331d`.
This establishes equality for initialization and completion; the original
expectations failed those two steps in that run. It does not claim that the
updated scenario has already passed a new editor run. The judge continues to
reject corrupted expectations, including a missing, enabled, or incorrectly
typed JSX routing hint for this fixture. Rejudging the original 58-entry Helix
transcript with these vectors passes all 16 steps and rejects all 16 corrupted
expectations; that replay is not a new native editor run.

Code-action diagnostics use the scenario's explicit `diagnosticsFromRequest`
contract. Each complete canonical diagnostic must first match a server-published
diagnostic. Its source, rule code, and range then identify the client diagnostic
that the action must echo in full. This preserves optional fields exactly as the
client sent them, including an omitted documentation URL or supplied metadata.
The judge does not alter observed responses, remove fields, or branch by client.
Published diagnostics and all other code-action fields retain their complete
static expectations.
