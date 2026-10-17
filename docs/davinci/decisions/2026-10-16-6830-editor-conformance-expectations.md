# Align TS-45 with the implemented LSP contracts

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The scheduled [Editor conformance run 38879276273](https://github.com/ubugeeei-prod/vize/actions/runs/38879276273)
failed on main `84b21e297cefabf9011c125f3aa08b98656b3368`. The real server
built successfully, all four drivers completed, and every corrupted expectation
was rejected. The previous scheduled run
[38712571661](https://github.com/ubugeeei-prod/vize/actions/runs/38712571661)
has the same differences. These are protocol expectation mismatches, not an
editor installation failure or an effect of the new CI schedule.

## Capabilities and completion

The exact initialize reference omitted the three native import source actions
and `monikerProvider`. These implementations were incorporated in
[#9122](https://github.com/ubugeeei-prod/vize/pull/9122), commit
`8009e15b7cd2b279dfa2a8ad8b9dfa73e628bfd4`.
`server/capabilities/code_action_tests.rs` requires the complete advertised
action-kind array, and `ide/moniker/tests.rs` exercises declaration identities.
The reference now includes those implemented capabilities and retains every
other field.

The completion reference retained only the two authored bindings. Append the
existing independent 41-item
`tests/_fixtures/differential/lsp/template-expression-globals-8015/globals.expected.json`
bank: 14 Vue instance members and 27 allowed JavaScript globals. Preserve both
original binding candidates and every field, order, and rank in that bank.
`ide/completion/template/globals.rs` and `globals_tests.rs` are the existing
source and behavioral witnesses. All four captured complete responses equal
this independently assembled 43-item reference; no response filtering or
runtime-generated expectation is introduced.

## Code-action diagnostic identity

Helix sends the selected lint diagnostic without its optional `codeDescription`.
Neovim, VS Code, and the Zed replay include that field. The published diagnostic
still contains its documentation URL for every client and passes the unchanged
full diagnostic reference.

`server/code_actions.rs` passes the actual request diagnostics to
`CodeActionService::code_actions_for_diagnostics`. The template lint service
matches source, rule code, and range, then clones the selected client diagnostic
without replacing its payload. The existing
`same_range_rules_use_configured_diagnostic_identity_and_full_payload` test
requires this behavior. A static response copied from the published diagnostic
therefore cannot describe both client requests truthfully. Removing the URL
from every response expectation would be wrong for the other three clients.

The code-action reference now names the required request diagnostic by exact
source, code, and range using `$requestDiagnostic`. A small shared resolver
requires exactly one match and substitutes that complete incoming diagnostic.
The judge still compares the entire action response, including every optional
diagnostic field, edit, title, kind, and preference. It never derives an
expectation from the response and has no client-name branch. Missing, ambiguous,
or changed diagnostic identity fails. The published diagnostic expectations,
scenario, editor drivers, server behavior, and workflow are unchanged.

## Validation boundary

The new request-aware transcript fails the previous literal judge. All thirteen
focused judge tests pass with the resolver, including full metadata preservation,
missing/changed/extra response-field rejection, and exact diagnostic identity.
Replaying both retained runs now passes every step for all four clients. All
sixteen corrupted expectations remain rejected for all eight captured sessions.
Fresh execution of the four clients remains an Actions validation step; a
transcript replay alone does not claim a new editor execution.
