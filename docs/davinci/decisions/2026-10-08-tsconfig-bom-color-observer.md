# Keep whole BOM failure observations in a controlled plain-output environment

Issue: [#3984](https://github.com/ubugeeei-prod/vize/issues/3984), paired with the
[observer decision](https://github.com/ubugeeei-prod/vize/issues/3984#issuecomment-6054557391).

The full Glyph source run on `d5fa74e48d01cfe7dfd9a21f41a5d2bc41d42fff`
failed its inherited malformed BOM CLI law in
[run 37739279068, job 113185989482](https://github.com/ubugeeei-prod/vize/actions/runs/37739279068/job/113185989482).
The actual whole packet retained exit 1, empty stdout and the authored EOF
error at line 1 column 20. Only the `Error:` prefix was ANSI red instead of
plain. The corpus archives the complete raw log as a lossless gzip, its raw
and compressed hashes, and the complete failed and expected packets. The
original eight malformed inputs, historical packets and plain expectation
remain byte-exact.

The CLI uses the existing stderr `TextStyle` and Fresco terminal capability
policy. Either `FORCE_COLOR=1` or `CLICOLOR_FORCE=1` overrides `NO_COLOR=1`,
even when stderr is piped. The original BOM command inherited those settings;
the existing `check_text_output_cli` and `check_text_diagnostics_cli` tests
instead set `NO_COLOR=1` and clear both forcing variables on their children.
The failed log does not record which forcing variable reached the child.
Its exact origin remains unqualified; `CARGO_TERM_COLOR=always` in the job
is not a Fresco color input.

Replay the authenticated original unpublished .2 CLI, SHA-256
`4d02d0c14a83e5ce4d4d8239d69c8f221a9f5b0875237e44b17af67d4f95730d`,
with all original common files, config bytes, selected arguments and the
pinned native executable. All eight original whole failure packets match
under the controlled plain policy. Separate `FORCE_COLOR` and
`CLICOLOR_FORCE` controls each produce the entire corresponding colored
packet while `NO_COLOR=1` remains present: 24 actual historical CLI calls.
The complete receipt preserves their environments, native/CLI hashes and
whole output. This proves original color authority; it gives no current
fixed BOM, installed-release or delivery credit.

Apply the existing plain-output policy only to the Vize test child. Preserve
its argv, `CORSA_PATH`, native commands, all production/parser source and
every input and oracle. Add an independent whole colored packet expectation
and execute the same eight malformed configs with each forcing variable and
both together, followed by the plain policy on the same `Command`: 24 forced
and 24 controlled calls. Compare every exit/stdout/stderr field without
stripping escapes or filtering content. Capture each complete packet before
asserting equality so a failed current execution remains available.

Fresh exact-head Actions must execute all four BOM test bodies, including the
original eight malformed controls, 30 valid public CLI calls, 15 official
native project calls, two version probes and the 48 additive color controls.
Keep the historical nine/ten/eleven-target native recipes and every existing
workflow stage, target, field, cap and instruction budget unchanged. Protected
qualification and actual merge remain required. This observer repair changes
no production behavior, performance or P0 acceptance scope; #3984 remains open.
