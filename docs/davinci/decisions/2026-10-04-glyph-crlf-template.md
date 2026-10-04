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

The Auto newline TODO is handled separately by
[#7704](https://github.com/ubugeeei-prod/vize/issues/7704) and its
[decision](./2026-10-04-glyph-auto-line-endings.md), with separate corpus and
Actions/merge-queue evidence. The broader #6098 profiles/configuration issue
remains open.

The first protected queue candidate (`b50cafb2`, Check 37171010927) exceeded
only the original SFC/reuse ceilings by 74 instructions (293649/274797).
Remove the candidate immediately. Keep the CRLF helper cold and non-inlined
so its owned mask/line storage cannot enlarge the ordinary LF call frame;
rerun exact-head Actions and all unchanged ceilings before requeueing.

Cold storage alone saved 20 instructions but still exceeded the two SFC caps
by 54 (`91bb1323`, preflight 37171322672). Specialize ordinary LF line writes
to the known one-byte LF slice; non-LF dispatch stays cold. This removes
dynamic newline copy work on each LF output line while keeping the raw lexer
and every retained law unchanged. Remeasure the same immutable budgets.
