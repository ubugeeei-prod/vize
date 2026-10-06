# Authored member hover after abandoned native IPC

`SlotAuthoring.vue.txt` is the complete unmodified real VS Code host fixture
whose dynamic slot `#[names.current]` must expose the property JSDoc. Its
SHA256 is `7d22c92c9cb0d55a58d91d76c33710a6939fba3e1b4164d1332188b6e9594342`.
Issue #8085 records the actual full-host failure and complete wrong Hover.
The original strict Bundler config is retained alongside the source.

The Rust stdio test requires the actual workspace TypeScript native runtime
and frozen Vue dependency. A test-only transparent process proxy holds one
complete real `textDocument/hover` response after the native process answers;
it never replaces, filters or synthesizes protocol values. After cancelling
the owning public request, the next original hover must wait on the same
worker, syntax-only folding must answer while the native reply is held, and
release must yield the complete independently authored property Hover.
LF/CRLF and unsaved astral-prefixed revisions cover current source/ranges;
a stale waiting request must return complete ContentModified, and a second
cancelled waiter must never poison the next current request. Shutdown/exit
remain strict with stdin open. Existing #8012 startup/lifecycle controls stay
unchanged. Native successful empty is governed by unchanged #7916 controls.

Expected Markdown follows the literal property type and full authored JSDoc,
not an observed Vize response. The proxy retains both original native frames
for diagnosis. This prepared corpus does not by itself grant runtime, full
host, protected merge, public release or complete LSP history credit.
