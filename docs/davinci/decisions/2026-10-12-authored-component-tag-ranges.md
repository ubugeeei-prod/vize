# Component tag anchors retain their authored extent

Owner: [#3952](https://github.com/ubugeeei-prod/vize/issues/3952).

The actual native Options API definition oracle at source 17bb018 failed:
the first byte of `sectionComponent` at authored offset 29 mapped to generated
position 202:7, the closing brace of an empty component props check. The
native definition answer was empty. Its original failed Linux receipt is
Check run 38152433486, job 114508465496; the complete pinned Canvas Matrix
control succeeded separately and does not qualify the minimal native case.

The expression has a correct copied mapping, source 29..64. Three component
tag producers instead claimed source 14..33, using the 19-byte generated
`__vize_dynamic_is_3` binding for the 9-byte authored `component` tag. The
narrowest generated row won over the expression and addressed the empty
props object. Navigation, whole-props and required-slot rows share this bug.

Use the retained template bytes at the parser-owned element start/end to
bound each authored opening tag. All three producers share that range helper;
the source selector, emitted TypeScript and semantic links retain their
existing behavior. Missing or invalid authored source cannot authorize a
guessed alias range. Work is bounded by the tag length and adds no AST walk,
pipeline stage or serialization.

The original minimal Canvas bytes, query offset, complete scalar target,
Airi source and original event authority remain unchanged. Separate corpus
cases cover expression, named and dotted dynamic values, static tags,
reserved-looking opaque text, nesting, UTF-8 and CRLF. Their actual projection
controls supplement the native nonempty-answer guard and complete scalar
definition oracle. Required exact-source Linux and whole Matrix receipts
remain necessary; the original failed qualification is retained as failed.

The bounded local actual-runtime rerun changes the original empty-answer
failure to a pass with the unchanged method source, first-byte query and
three complete scalar responses. The four canonical mapping controls,
including all seven new source cases, also pass. These local controls do not
replace the pending exact-source Linux/full Matrix qualification.

Source 8dc5238 full Check 38156814430 stopped before native execution because
the new projection test used the forbidden `format!` macro; check-js also
identified the new helper's unregenerated Croquis consumer row. Adopt `cstr!`
and regenerate that exact row. Those original failed gate receipts remain
failed; they provide no Linux native success evidence.
