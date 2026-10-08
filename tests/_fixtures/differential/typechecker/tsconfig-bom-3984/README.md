# UTF-8 BOM tsconfig acceptance (#3984)

These whole inputs were authored before the Vize reproduction. `input.json`
is byte-exact to that original: direct and inherited configs, with a UTF-8
BOM inserted only at byte zero of the root or parent. The corpus keeps the
complete Vue declaration stubs, selected ambient declaration library, alias
import, authored source roots, and deliberately broken excluded files.

`official-whole-observations.json` retains the complete TS 6.0.3 and 7.0.2
showConfig and check packets for every plain/BOM clean/broken/repair variant.
`before-cli-whole-observations.json` retains all original public Vize packets
and their complete CLI artifact/source provenance. They came from the
unpublished, later superseded .2 candidate H212692/R37715578811, actual
source cut 6d26; this is source defect evidence, not installed release credit.
All valid BOM variants failed with an empty stdout and a complete JSON parse
failure before the fix; healthy plain variants checked all three intended
roots and reported the authored TS2322 in the broken phase.

`expected-cli.json` is the independently authored complete public DTO.
The only environment binding is the known fixture-root alias destination
`__FIXTURE_ROOT__/src/model.ts`; no current response supplies expected values.
Every other options, files, program and diagnostics field is compared whole.
The reported program file list is Vize's authored graph report, not a claim
about every internal TypeScript standard-library file.

`parser-controls.json` keeps complete nested/unknown fields, empty arrays,
comment-like string data and BOM string/key data, plus malformed/empty
controls. `malformed-original-whole-controls.json` retains the complete
original CLI failure packets; plain and BOM malformed configs must keep the
whole authored EOF failure rather than become successful configurations.

The two valid public tests execute 30 Vize invocations (default and explicit)
and 15 official native project invocations across clean/broken/repair phases.
Two native version probes are separate. A third public test executes eight
whole malformed CLI failure controls. Both real parser APIs execute the same
three pure laws. Ordinary source tests may skip native bodies; required
native Actions and the current protected candidate must execute them.

The change strips at most one byte-zero BOM in the two existing readers.
It adds no generic JSONC recovery, native error/option transport, cache,
watch/build phase, negative template-prop acceptance or performance claim.
#3984 remains open for its complete original acceptance criteria.

The [controlled color observation decision](../../../../../docs/davinci/decisions/2026-10-08-tsconfig-bom-color-observer.md)
archives the full inherited source-coverage failure in
`malformed-color-before-d5fa.log.gz` with complete packet/hash custody.
`malformed-color-original-authority.json` retains 24 actual authenticated
original CLI controls. The Vize test child now uses the existing plain-output
policy (`NO_COLOR=1`, clear both force variables), and a fourth test compares
24 complete forced-color packets and 24 subsequent controlled plain packets.
The original malformed eight and every original input/gold remain unchanged;
current packets are captured before their assertions. Fresh Actions and
protected delivery are still required.
