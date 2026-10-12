# ESLint over original Vue source

Tracking: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Expose `oxlint-plugin-vize/eslint` in the existing package. Its independent rules
use the configured parser's full `SourceCode.text`, including unsaved buffers;
the default Oxlint entry retains its existing `createOnce`, batching and mappings.
Each verification batches only its participating rules and their options once.
SourceCode identity, filename, settings and native response identity remain bound
through reporting, including a reused AST and an aborted verification.

The existing `lintPatinaSfc` API gains optional `includeFixes`. Absent or false
retains its prior JSON exactly. True adds each existing native fix's message and
edits (`start`/`end` as UTF-8 byte offsets, `newText` as replacement text). The
adapter checks UTF-8 boundaries, source bounds, edit ordering and overlap before
converting to ESLint UTF-16 ranges. Native locations must match the same source.
Unpaired UTF-16 surrogates are refused before native conversion can normalize them.
No suggestions are fabricated from help text. Lone CR and Unicode line separators
with a differing native location convention are visible refusals; general
newline-convention alignment remains unfinished.

Required controls use the project's installed ESLint and Vue parser, with actual
native diagnostics and fixes: template/script/style and multiple script blocks,
unsaved buffers, astral text, CRLF, positive/clean/off inputs, parser refusal,
same-Program filename/preset/options changes, whole fixed bytes, default JSON,
invalid spans and unchanged default rules. The existing package test command
retains every prior check and adds these controls. Fresh Actions native artifacts,
protected actual merge and a matching installed package release are required.
An older local native package is a refusal receipt, not qualification.

This is a standard ESLint consumer contract. Other runtime CLI hosts need their
own released command, plugin-loading and installed-consumer evidence. Their
source API or current documentation alone cannot establish support. No formatter
plugin ABI or formatter support is added by this decision.

The explicit `qualification:js-plugin-host` label selects one bounded #8769
PR qualification in the same package test job. Trusted repository/PR input,
current native preparation, fixed artifact bytes and actual executable revision
are required before real CLI loading, unsaved stdin diagnosis, clean control and
whole fixed bytes. Any selected failure is a failed gate. Unselected runs claim
no target qualification. Rolling prerelease labels or metadata supply no stable
support or installed-public acceptance; the asset ID and digest preserve the
specific observed binary without making it a permanent default dependency.
