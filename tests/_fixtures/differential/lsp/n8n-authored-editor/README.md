# n8n authored editor oracle

Issue: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).

The three complete inputs are unmodified source bytes from licensed n8n master
`e882e8a483f433facb47bab9b407d0ec00a81172`. `provenance.json` pins original paths,
byte lengths and SHA-256 values. They retain the upstream Sustainable Use
License; the accompanying `LICENSE.md` retains its full terms. They are
development/test inputs and are not product code.

The test copies these inputs to a disposable workspace, follows the original
TypeScript component barrel, checks the real template binding's exact definition
and references, and uses the actual typechecker to diagnose an unsaved unknown
property and restore the clean source. An authored consumer drives the unchanged
BlockUi/importer boundary. The checked-in inputs and upstream submodule are never
modified. This small acceptance oracle does not establish whole-monorepo
vue-tsc parity or a performance claim.
