# Legacy Glyph automatic line endings (#7704)

Issue: [#7704](https://github.com/ubugeeei-prod/vize/issues/7704).
Configuration roadmap: [#6098](https://github.com/ubugeeei-prod/vize/issues/6098).
Depends on the [CRLF template repair](./2026-10-04-glyph-crlf-template.md) (#7697).

`EndOfLine::Auto` was accepted by schema/config but every printer treated it as
LF. Resolve its layout style from the first source CR or LF terminator: adjacent
CRLF selects CRLF, a lone CR selects CR, and LF or no terminator selects LF.
The caller's options stay immutable. The complete SFC resolves before parsing
or dispatch, so mixed embedded languages use the document's style. Standalone
script (including source-type/import-sort variants), version-aware template,
style and JSON/JSONC surfaces resolve their own complete input first.

Only Auto enters the cold resolution helper and clones options. The default
and explicit paths keep their existing parse/stabilization stages; no second
parse, serialization, dependency or relaxed instruction budget is introduced.
CSS's existing indentation pass also applies explicit CR/CRLF layout. SFC
block terminator checks and indentation recognize lone CR. JSONC line comments
stop at CR as well as LF instead of swallowing the rest of a CR document.
Authored raw body/literal ownership remains with the existing printers/lexers.

Two additive CLI corpus cases select Auto through explicit config and author
complete independent CRLF/CR references from mixed layout inputs. They run
three passes with immutable input/config/output pins and retain the original
five corpus obligations. Public API assertions cover all entrypoints, each
terminator, fixed points, no-terminator fallback and explicit-option precedence.
The actual current-source API/CLI execution and unchanged instruction counts
must pass in Actions before protected native Stack merge is reported.

Historical capture receipts keep their original hashes and source identities.
Complete original `style.rs` and `block_indent.rs` blobs are included as frozen
assertion witnesses; exact current-owner pins and unchanged raw function hashes
retain all original laws. These assets give no fresh execution, native provider
acceptance or paired-comparison credit.

This completes the newline TODO identified in #7697, subject to its exact-head
Actions and queue gates. #6098 remains open for its broader configuration and
profiles acceptance; this scoped defect does not finish that roadmap.
