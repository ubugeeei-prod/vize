# Authored formatter tag forms (#6846)

Issue decision: [#6846 comment](https://github.com/ubugeeei-prod/vize/issues/6846#issuecomment-5854644576).

## Decision

The audited legacy formatter does not replace a paired component tag with a
self-closing tag. Keep its source unchanged. Component-resolution-dependent
rewrites belong to lint autofixes; the formatter does not need L2.

The public template formatter's `parse_opening_tag` sets `is_self_closing`
only when the authored bytes contain `/>`. Its writer's self-closing branch
consumes that flag. For an immediate empty paired tag, the other branch writes
both `>` and `</name>` explicitly. The closing-tag branch retains authored
closing tags. See `crates/vize_glyph/src/template/formatter.rs`.

`is_void_element_str` in `template/helpers.rs` uses the fixed HTML void-tag
vocabulary and an uppercase-name guard. That affects indentation; it neither
resolves components nor chooses a self-closing rewrite. Native SFC templates
use this same writer through `formatter/template_block.rs`; other template
languages keep opaque source content. Raw-region masks classify authored
`/>` to find region boundaries and do not rewrite tag forms.

## Executable contract

The strict JSON corpus covers eleven authored forms through both actual
public template and SFC APIs: paired component/native/custom/member names,
an uppercase void-tag namesake, slot and dynamic component, three authored
self-closing forms and an authored HTML void tag. Compare complete expected
bytes, then repeat formatting and require an unchanged SFC verdict. These
are authored expectations pending fresh Actions, not historical captures.

Corpus parsing uses the existing workspace `serde_json` as a dev dependency.
Formatter behavior, options, production dependencies and stages are unchanged.
Native acceptance and shared differential-runner registration remain
unsupported/pending. #6882 still owns complete historical formatter coverage.

## Completion evidence

Before closing #6846, require fresh source-built Actions for this corpus and
the actual main merge queue. Record their run and merge identities in the
issue. Preserve #6836's native L1 embed work and #6875's native formatter work
as separate requirements.
