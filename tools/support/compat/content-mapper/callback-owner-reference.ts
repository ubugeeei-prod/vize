/** Complete independent callback reference vectors and raw process states. */
import assert from "node:assert/strict";
import type { SourceSpan } from "./callback-owner-projects.ts";

export type Observation = {
  argv: string[];
  cwd: string;
  status: number | null;
  signal: string | null;
  error: unknown;
  stdout: string;
  stderr: string;
};
type Diagnostic = {
  file: string;
  line: number;
  column: number;
  code: number;
  message: string;
  continuation: string[];
};

export function processPacket(packet: Observation, statuses: number[]) {
  assert.equal(packet.error, null);
  assert.equal(packet.signal, null);
  assert.equal(packet.stderr, "");
  assert.ok(packet.status !== null && statuses.includes(packet.status));
}
/** Every output line must belong to one diagnostic; no messages are filtered. */
export function diagnostics(output: string): Diagnostic[] {
  const rows: Diagnostic[] = [];
  if (!output) return rows;
  assert.ok(output.endsWith("\n"));
  for (const line of output.slice(0, -1).split("\n")) {
    const match = /^(.*)\((\d+),(\d+)\): error TS(\d+): (.*)$/u.exec(line);
    if (match) {
      rows.push({
        file: match[1],
        line: Number(match[2]),
        column: Number(match[3]),
        code: Number(match[4]),
        message: match[5],
        continuation: [],
      });
    } else {
      const previous = rows.at(-1);
      assert.ok(previous && /^\s+/u.test(line), `unclassified compiler output: ${line}`);
      previous.continuation.push(line);
    }
  }
  return rows;
}

/** #11140 has a handwritten typed owner after the eager delete, never a Vize oracle. */
export function assertDynamicEventArgumentReference(
  reference: Observation,
  nativeReference: Observation,
  span: SourceSpan,
) {
  processPacket(reference, [2]);
  processPacket(nativeReference, [1]);
  const expected = [
    {
      file: span.file,
      line: span.line,
      column: span.column,
      code: 2322,
      message:
        "Type '(field: undefined) => false' is not assignable to type '(x: number) => boolean'.",
      continuation: [
        "  Types of parameters 'field' and 'x' are incompatible.",
        "    Type 'number' is not assignable to type 'undefined'.",
      ],
    },
  ];
  assert.deepEqual(diagnostics(reference.stdout), expected, "complete independent typed reference");
  assert.deepEqual(
    diagnostics(nativeReference.stdout),
    expected,
    "complete physical native reference",
  );
}
