# Explicit content providers for palpable-content lint

Issue: [#8018](https://github.com/ubugeeei-prod/vize/issues/8018).

The original `vSafeHtml` definition assigns `innerHTML` in `mounted` and `updated`.
Its exact 209-byte Notice SFC, 287-byte directive definition, and 98-byte config are
retained with the original issue body hash and verified reporter GitHub ID 71201308.
Custom directives can also only focus or style an element. The existing HTML rule
therefore cannot infer visible content from every arbitrary directive name.

Choose the report's explicit configuration alternative:
`linter.ruleOptions["html/no-empty-palpable-content"].contentDirectives` lists exact
bare template directive names, for example `["safe-html"]`. Arguments and modifiers
retain the same name. Default absence and an empty list retain existing findings;
scoped entries replace the complete option object, including an empty reset.
The setting neither enables the rule nor proves that HTML is sanitized.

Preserve the original public unit rule and its default callback byte-for-byte.
A configured rule instance only bypasses explicitly named providers before delegating
to that callback. The existing CLI and Maestro config adapters install this instance
only when the typed option exists, retaining disabled-rule/severity precedence.
The stable Rust `LintRuleOptions` construction shape and absent-option serialization
remain unchanged; Pkl, JSON Schema, and generated configuration types agree.

An authored eleven-scenario corpus binds complete public Rust results and JSON reports:
original default/configured Notice, unrelated focus/lookalike/ordinary attributes,
three sanitizer names, argument/modifier names, empty reset, scoped replacement/reset, option-only inactivity, and explicit rule off.
Source-built CLI repeats each whole report three times and retains the literal reported
plain command. Genuine source-built LSP sessions compare all eleven complete versioned
publications across open, same-width unsaved edit, and restored original (33 total).
Original and authored input/reference hashes are retained; no runtime oracle recording.

The first source run at `6909818ab8` rejected two authored paired-tag range ends.
The unchanged parser constructs `element.loc` from the opening tag and does not
extend it on closing; the unchanged rule reports that location. Independent source
review therefore corrects only those expected CLI/LSP ends and Rust paired-tag
spans, preserving the full original inputs, all other reference fields, and the
historical failed reference hash/run. No production range policy is changed.
The same run rejected hand-authored schema indentation; regeneration from the
reviewed Pkl source corrects only the array item object whitespace (30 bytes).

Exact source Actions, unchanged protected 104 instruction ceilings/ratchets, full Rust,
original differential corpus, actual signed merge, and later publication are required.
These observations confer no native product replacement or #6881 history acceptance.
Direct Node/WASM lint options and sanitizer inference are outside this config slice.

Co-authored-by: ubugeeei <71201308+ubugeeei@users.noreply.github.com>

## Post-publication delivery qualification

The [paired current-main decision](https://github.com/ubugeeei-prod/vize/issues/8018#issuecomment-5998346899)
records the explicit root thaw after supported v0.433 publication. A genuine
replay onto actual `8a8521d689` preserves all26 noncanonical owned source/config/
original/reference/test blobs from reviewed `73bf0b9f`; its retained canonical
clause reproduces every incoming main byte/LF at350 lines when removed. This
record supplies no new production behavior or historical runtime transfer.
Fresh exact source/native Actions, prospective whole-prefix compatibility,
protected full104 suites and signed actual merge remain mandatory. Root alone
selects and publishes the next finite0.434 release; original failed/successful
proof and explicit source reporter trailers remain retained separately.
