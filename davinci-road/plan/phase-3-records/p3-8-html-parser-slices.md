# P3-8 — HTML parser parity slices

## Eleventh slice: implicit `<tbody>` (2026-09-22)

A `<tr>` placed directly under `<table>` emits the `tbody` both parsers
insert. The synthesized open tag is anchored one byte past the triggering
child's tag name, matching the legacy walker's zero-width loc.

## Twelfth slice: implicit `<tr>` (2026-09-22)

A direct cell (`<td>` / `<th>`) emits the `tr` both parsers insert, under
an authored row group or the implicit `tbody` of a cell placed in
`<table>`. The synthesized name uses the same open-tag anchor.

## Thirteenth slice: title content (2026-09-22)

`<title>` now emits from the string plan. The shared text facts already
preserve its raw markup text, entities, interpolation and whitespace, so
it no longer needs the defensive element refusal. Script/style remain
refused. Five admitted fixtures run under all four option sets, including
source maps, requiring exact output and a plan-emitted verdict. A title SFC
also participates in the production adapter corpus battery.

Validation: the SSR library and snapshot suites (144 tests) and the
90-file construct-matrix corpus on both the template and SFC adapter paths.
