import assert from "node:assert/strict";
import {
  compilerRows,
  location,
  sha,
  type Packet,
} from "./content-mapper-instance-globals-10617.ts";
import type { ReceiverProgram } from "./content-mapper-instance-globals-receiver-symbols-10617.ts";

// These two immutable receiver-allocation counterfactuals intentionally keep
// TS2552 in the handwritten oracle and raw mapper. The native CLI presents a
// missing authored template name as TS2551. See template_instance_names.rs,
// a_lexical_lookup_failure_becomes_an_instance_one, and projection/assemble.rs,
// template_lookup_failures_become_instance_diagnostics_once. This is a finite
// native presentation expectation, not a diagnostic normalizer.
const expectations = {
  "unresolved-value": {
    sourceSha256: "7ed2755b7f687cd8d4167683f0241c05c8dee1a307a24a7b26e27ca98d8e97d2",
    anchor: "{{ __vize_template_instance }}",
    role: "value",
    line: 2,
    column: 51,
    unresolvedIndex: 1,
    errors: [
      { anchor: "$ghost.value", token: "$ghost", kind: "undeclared" },
      {
        anchor: "{{ __vize_template_instance }}",
        token: "__vize_template_instance",
        kind: "unresolved",
      },
    ],
  },
  "unresolved-type-query": {
    sourceSha256: "22546360ed4a9d7f5961e2817e74937a19160cd5c981109a039392e99fe482c8",
    anchor: "typeof __vize_template_instance",
    role: "type-query",
    line: 5,
    column: 46,
    unresolvedIndex: 0,
    errors: [
      {
        anchor: "typeof __vize_template_instance",
        token: "__vize_template_instance",
        kind: "unresolved",
      },
      { anchor: "$ghost.value", token: "$ghost", kind: "undeclared" },
    ],
  },
} as const;

export function hasReceiverNativeExpectation(id: string) {
  return Object.hasOwn(expectations, id);
}

export function assertReceiverNativeCase(
  content: string,
  entry: ReceiverProgram,
  context: Packet,
  cli: Packet,
  compilerOptions: unknown,
  label: string,
) {
  assert.ok(hasReceiverNativeExpectation(entry.id), "only two pinned receiver controls qualify");
  const expected = expectations[entry.id as keyof typeof expectations];
  assert.equal(content, entry.source, "the native actor must use the same authored source");
  assert.equal(sha(content), expected.sourceSha256, "exact counterfactual source bytes");
  assert.deepEqual(entry.errors, expected.errors, "exact counterfactual authored anchors");
  assert.equal(entry.receiver, "fresh");
  assert.deepEqual(entry.candidate, { name: "__vize_template_instance", role: expected.role });
  assert.equal(entry.binding, undefined);
  assert.equal(entry.unusedDirective, undefined);
  assert.equal(content.split(expected.anchor).length, 2, "one unique unresolved anchor");
  const at = location(content, expected.anchor, "__vize_template_instance");
  assert.equal(at.line, expected.line);
  assert.equal(at.column, expected.column);
  const rows = compilerRows(context.stdout);
  assert.equal(rows.length, 2);
  assert.deepEqual(
    {
      status: context.status,
      signal: context.signal,
      error: context.error,
      stderr: context.stderr,
    },
    { status: 2, signal: null, error: null, stderr: "" },
    label,
  );
  assert.ok(rows.every((row) => row.file === "oracle/Context.ts"));
  const lexical = rows[expected.unresolvedIndex];
  assert.equal(lexical.code, 2552, "the raw handwritten oracle keeps its lexical diagnostic");
  assert.equal(
    lexical.message,
    "Cannot find name '__vize_template_instance'. Did you mean '__vize_template_instance_'?",
  );
  const diagnostics = entry.errors.map((error, index) => {
    const sourceAt = location(content, error.anchor, error.token);
    const code = index === expected.unresolvedIndex ? 2551 : rows[index].code;
    const message =
      index === expected.unresolvedIndex
        ? "Property '__vize_template_instance' does not exist on the component instance. Did you mean '__vize_template_instance_'?"
        : rows[index].message;
    const sourceLine = content.split("\n")[sourceAt.line - 1].trim();
    return `error:${sourceAt.line}:${sourceAt.column} [TS${code}] ${message} (source: ${sourceLine})`;
  });
  assert.deepEqual(
    { status: cli.status, signal: cli.signal, error: cli.error, stderr: cli.stderr },
    { status: 1, signal: null, error: null, stderr: "" },
    label,
  );
  assert.deepEqual(
    JSON.parse(cli.stdout),
    {
      files: [
        { file: "src/Globals.vue", diagnostics },
        { file: "src/globals.d.ts", diagnostics: [] },
      ],
      programs: [
        {
          root: ".",
          tsconfig: "tsconfig.json",
          compilerOptions,
          files: ["src/Globals.vue", "src/globals.d.ts"],
        },
      ],
      errorCount: 2,
      warningCount: 0,
      fileCount: 2,
    },
    `whole native receiver report ${label}`,
  );
}
