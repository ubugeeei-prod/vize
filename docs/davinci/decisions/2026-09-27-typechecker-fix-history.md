# Type-checker fix-history diagnostic fixtures

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

## Scope and decisions

The directory query at `9aaa1fe458a09e0d0c6604dc8835ccf7c737d943`
returns 355 commits and exactly the issue's 271 `fix` subjects. The
[requirement ledger](../plan/typechecker-fix-history.tsv) preserves all 271
immutable commit identities and their focused requirements. The requirements
were previously reviewed against `9e203108c7b77b8a0bf6243d5db1f2c09c878eac`;
Canon has no source changes between that pin and this slice's main revision.
Every row retains the explicit gap for the whole bundled commit and native
diagnostic equivalence. A reviewed requirement is not fixture acceptance.

The first executable slice pins the authored inputs of
`613ce3a5a31d22ec0dfd42feab25772e63fc54bf` (#4996). It checks three existing
event-assignment cases through the unchanged production `BatchTypeChecker`:

| Case                                               | Expected complete batch diagnostic list                                           |
| -------------------------------------------------- | --------------------------------------------------------------------------------- |
| Component handler assignments and sibling bindings | Empty                                                                             |
| Native handler assignment and sibling bindings     | Empty                                                                             |
| Invalid assignment in a component handler          | One TS2322 at `src/parent.vue:7:33`, with the exact authored message and severity |

The previous valid tests filtered only TS2367. The previous invalid test
compared only parent-file rows. The new fixture test compares every returned
diagnostic, including other files, in the returned order. It does not sort,
normalize messages, or filter unexpected codes. It fails if project creation,
Vue installation, checker construction, scanning or checking fails. A missing
runtime cannot turn this test into a skipped success.

The fixture pack holds exact source bytes in `.txt` carriers and declares their
authored `.vue` and `tsconfig.json` paths. The test materializes those exact
paths in one temporary project per case and links the installed real Vue
package. The inputs already exist inline in `event_handler_narrowing.rs`; these
carriers make them independently consumable without introducing a second
standalone Vue application into repository-wide fixture discovery.

This is a batch start-position contract. The public batch diagnostic type has
no end range, related-information payload, or raw backend response. The pack
records those missing fields explicitly. It claims no complete LSP capture,
historical backend-payload parity, common differential execution, native adapter,
or native acceptance. The prepared complete observer archives remain separate
work and must be reviewed and freshly verified before admission.

## Verification and remaining work

The actual production checker passed all three cases locally at this source
revision with TypeScript 7.0.2 and Vue 3.6.0-beta.10. The test runtime was 2.50
seconds after a 79-second cached build; these are fixture timings, not CI
performance measurements. Actions and merge-queue execution remain pending.

Two actual runs passed. Changing the expected code, line, column, message or
severity failed each run; adding an unexpected diagnostic to the child SFC
also failed. Every mutation was restored before the repeat passed.

- Fresh Actions execution and merge-queue verification are required before
  admitting this fixture slice.
- The complete #4996 patch also changes dynamic/custom and disabled-check event
  handlers and guard behavior. These three inputs do not close that whole fix.
- The other 270 directory fix rows have no admitted fixture coverage in this
  ledger. Convert each live diagnostic requirement into exact input and
  diagnostic fixtures, and record exact successor evidence for retired rules.
- Include historical flat `src/virtual_ts.rs` changes before declaring history
  complete. Earlier focused preparation identified eight additional `fix`
  subjects outside the directory query; title counts are discovery aids.
- Integrate the pack through the shared #6891 result/accounting contract when
  the type-checker adapter is reviewed. End/related payload and native L4 parity
  remain open. #6879 stays open; #6849 replacement is still blocked.
