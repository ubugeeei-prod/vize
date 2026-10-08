# Reactive props with named declarations

These supplemental sources extend the reactive destructure contract in
[#7996](https://github.com/ubugeeei-prod/vize/issues/7996). They preserve the
original inline and checked-boolean report fixtures.

The native CLI test covers local type aliases and interfaces, each with a
shorthand binding, a default, and an explicit local alias. Public declaration,
parent argument, local declaration, and local template origins run with LF and
CRLF: 48 complete transactions.

Public rename changes the type and destructure property key, preserves the
child local value and parent declaration, and expands the parent shorthand.
Local rename preserves the public type and parent bytes while changing only
the local binding and its template use.

Every complete references response and WorkspaceEdit is compared without
filtering or deduplication. The existing authored driver applies actual edits,
checks all saved files and version-2 diagnostics, then independently installs
the authored repaired project and checks version-3 diagnostics. The separate
goldens are authored before native queries; passing execution must be observed
before these supplemental variants receive coverage credit.
