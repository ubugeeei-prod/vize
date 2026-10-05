# Static non-focusable aria-hidden controls

Paired issue: [#7977](https://github.com/ubugeeei-prod/vize/issues/7977), with
[the implementation decision](https://github.com/ubugeeei-prod/vize/issues/7977#issuecomment-5996699409).
The inspected baseline is actual main
`8a8521d6897bbe3fd0af0cbfaebd83f4fc933933`.

## Correction and compatibility

The existing predicate checks tabindex and native element names before
any static inert/disabled/hidden exclusion. Add exclusions only to
`a11y/no-aria-hidden-on-focusable`; its shared focusability predicate and
other rules retain their original contracts. Static Boolean inertness on
an element or its physical ancestors, disabled button/input/select/textarea,
and a static hidden input type suppress this rule's focusability finding.
Same-name, object or dynamic-argument bindings remain conservative.
Boolean attribute presence includes `disabled="false"`; a bound
`:disabled="false"` stays dynamic and does not suppress a finding.

This follows the existing [HTML inert contract](https://html.spec.whatwg.org/multipage/interaction.html#the-inert-attribute),
[disabled form-control contract](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#attr-fe-disabled)
and [hidden-input state](<https://html.spec.whatwg.org/multipage/input.html#hidden-state-(type=hidden)>).
Component, slot and potentially modal dialog boundaries stop unproven
inherited inertness. A dialog's own static inertness still applies to its
children. Runtime placement, disabled-fieldset/first-legend semantics and
JSX spread-overwrite analysis are outside this bounded slice.

SFC/JSX markup callbacks inspect the existing original ancestor views.
The public bare-template entry uses the retained visitor, whose original
context holds tags but not attributes. One crate-private Boolean carries
current inertness in that existing recursion; a once-per-dispatch rule
flag gates its calculation. Save/restore preserves sibling state. No new
parse, tree walk, pipeline, serialization or public ElementContext field
is introduced. This is traversal metadata, not native callback authority.

The existing #5943 programmatic-focus policy stays: tabindex=-1 reports
on an otherwise focusable element. The original whole SFC should have
two findings, its tabindex=-1 input and final enabled button. That input
retains the old full help; other findings reuse the existing localized
actionable sentence through HelpLevel::Short. All three catalogs remain
byte-exact. Diagnostics retain their original whole-element byte spans,
severity, message, labels, fixes and ordering.

## Whole original inputs and execution requirements

`crates/vize_patina/tests/fixtures/issue-7977/` pins the full original issue,
758-byte SFC and config fences, an exact LF-to-CRLF copy and thirty authored
Boolean/dynamic/positive/carrier/component/dialog/sibling controls.
The original SFC SHA256 is
`bba56b373bfaedc1c8a13fd3e5e6b0c1512a2ad957c8d1f5e19b448c5352a36d`.

Five Rust laws compare complete results: original LF/CRLF in all three
locales and help modes; public bare/SFC controls; genuine Relief and L1/L2
facade ancestor views; public JSX literal-versus-expression controls; and
an exact native UnprovidedRule refusal. The native callback stays
unprovided, with no fallback clean/handled credit. Existing fixtures,
expected vectors and history census stay unchanged. Only two genuine
new test/dev inventory rows are generated in the owned Patina shard.

The normal tooling test requires the exact source-build receipt and
compares three whole public JSON results twice in fresh processes, keeping
original inputs, hashes, raw streams, exit codes and complete expected
messages/columns/help. CLI columns retain their existing Unicode scalar
contract. Qualification advances only after both complete replies and
unchanged input bytes pass. nativeHandled remains zero.

Authored laws and independent source review grant no execution credit.
Fresh exact-head automatic Actions must compile/run these laws and the
public CLI corpus. The first-v0.433 publication hold keeps the conventional
PR Draft and off queue. After the hold, protected full suites and unchanged
104 instruction ceilings, actual signed merge with the literal verified
reporter trailer, and released public payloads remain separate delivery
requirements. Native/history #6881 and the 10x target remain unfinished.
The verified primary reporter is ubugeeei, GitHub ID 71201308; the meaningful
source commit includes the verified noreply Co-Author footer.
