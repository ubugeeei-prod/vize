# Legacy Glyph CRLF template terminators (#7697)

Issue: [#7697](https://github.com/ubugeeei-prod/vize/issues/7697).
Related configuration scope: [#6098](https://github.com/ubugeeei-prod/vize/issues/6098).

SFC template indentation splits the formatted body on LF. The residual CR from
a CRLF line was copied before the configured CRLF, yielding `CRCRLF`. The Vue 2
filter regression intentionally preserved this documented preexisting defect.

The CRLF path now consumes one residual CR before appending one CRLF. Keep the
ordinary LF loop intact rather than adding CR normalization inside every hot
loop iteration. The raw-line lexer still owns the decision to skip indentation
inside native raw elements, v-pre, literal attributes and template literals.
No new pipeline stage, dependency, serialization or relaxed budget is added.

The old Vue 2 reference changes solely by `CRCRLF` → `CRLF`. Add a separate
configured `sfc-crlf-template-layout` corpus case with independently authored
complete expected bytes. CRLF input is already a fixed point; LF input reaches
the same exact CRLF output. Public tests cover three passes, changed flags and
raw regions. The six-case CLI corpus retains the original historical capture
artifacts and grants zero native formatter acceptance or paired-comparison credit.

Local RED: compiling the unmodified real `template_indent::write_line` in an
isolated rustc reproducer produced final bytes `[13, 13, 10]` versus independently
required `[13, 10]` (exit 101). This is helper evidence only; actual current-source
public API, CLI, full suites and unchanged instruction budgets must pass in
Actions before protected merge completion is reported.

TODO #6098: `EndOfLine::Auto` currently resolves to LF in the newline helpers and
OXC conversion. Detecting the source's line-ending style and consistently applying
it to every public formatter surface needs its own exact corpus/config coverage.
The broader profiles/configuration issue remains open.
