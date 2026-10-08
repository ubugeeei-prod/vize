# Accept a byte-zero UTF-8 BOM in the two existing config readers

Issue: [#3984](https://github.com/ubugeeei-prod/vize/issues/3984).

A valid tsconfig that differs only by a leading UTF-8 BOM fails
`vize check --quiet --format json` before a complete report, in default and
explicit modes. A BOM on the root or its extended parent produces exit 1,
empty stdout and `Error: JSON parse error: expected value at line 1 column 1`.
The same complete plain inputs select three authored roots, retain the
selected ambient library and alias, report the intended TS2322 in the broken
phase, and recover after repair. Official TypeScript 6.0.3 and 7.0.2 accept
all corresponding config variants with identical whole packets.

The reproduction uses the authenticated macOS ARM CLI from unpublished,
later superseded .2 source candidate H212692/R37715578811, actual source cut
6d26. Both real JSONC entry files are byte-exact to actual main a538. This is
source defect evidence; it is not .2/.436 publication or installed acceptance.
The original full inputs, stock showConfig/check outputs and all original
public CLI packets remain in the differential corpus unchanged.

The production change borrows the input after at most one byte-zero U+FEFF,
then runs the existing comment and trailing-comma transformations unchanged.
It applies to the CLI input/declaration/type selection reader and Canon's
virtual-project config reader. It adds no allocation for the prefix, file
read, pipeline stage, native-option/error transport, cache or watch behavior.
BOM characters inside string/key data and all nested/unknown fields remain
complete. Empty/BOM-only and malformed contents remain failures.

The complete authored DTO compares all files, diagnostics, program/options,
counts and selected declaration graph entries. Its sole environment binding
is the known canonical fixture root plus the authored `src/model.ts` alias
path; the current producer never supplies expected fields or paths. The
reported graph is Vize's authored program report, not an assertion about all
internal TypeScript standard-library files. Original plain malformed CLI
exit/stdout/stderr packets are frozen and compared whole; a BOM on malformed
input must not turn that failure into success.

Required native source Actions and the current protected candidate must
execute the new two valid public tests (30 Vize default/explicit calls,
15 official native project calls, two separate version probes) and the eight
public malformed failure controls. Malformed configs fail in the existing
Rust reader before backend checking; their eight CLI calls are separate from
native backend project invocations. Both existing APIs run three pure laws
for complete values/string data, malformed failure, and empty/BOM-only
failure. The existing native qualifier gains only the new test target, in
the same job and stage with every prior argv, target, field, timeout and cap
retained. Optional complete receipts reuse the existing capture directory;
all original receipt locations remain intact. Ordinary source native skips
receive no actual-body credit. Original differential/native phases and the
100+4 protected instruction gates remain mandatory, without budget changes.

This is a narrow config lexical acceptance repair. Complete tsconfig
semantics, project/build/watch parity, negative template prop diagnostics,
installed acceptance and the general 10x target remain unfinished under
#3984; this change does not close that issue or replace a legacy path.
