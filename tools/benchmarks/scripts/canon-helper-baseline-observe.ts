/** Exact authored error multiplicity and complete native/CLI packet capture. */
import assert from "node:assert/strict";
import path from "node:path";
import { TextDecoder } from "node:util";
import { packet } from "./canon-instance-directives-model.ts";
import type { Expected } from "./canon-helper-admission-api.ts";
import { save } from "./canon-instance-directives-io.ts";
type Row = Expected & { message: string };
export function nativeDiagnostics(stdout: Buffer, status: number | null, project: string) {
  const text = new TextDecoder("utf-8", { fatal: true }).decode(stdout);
  const rows: Row[] = [];
  for (const match of text.matchAll(/^(.+?)\((\d+),(\d+)\): error TS(\d+): (.*)$/gmu)) {
    const [_, file, line, column, code, message] = match;
    assert.ok(
      file !== undefined &&
        line !== undefined &&
        column !== undefined &&
        code !== undefined &&
        message !== undefined,
    );
    rows.push({
      file: path.relative(project, path.resolve(project, file)).replaceAll("\\", "/"),
      line: +line,
      column: +column,
      code: +code,
      message,
    });
  }
  assert.equal(
    rows.length,
    (text.match(/error TS\d+:/gu) ?? []).length,
    "all native diagnostic headers must be owned",
  );
  assert.equal(status, rows.length ? 2 : 0);
  return rows;
}
export function assertHelperObservation(
  mode: "cli" | "mapper",
  stdout: Buffer,
  status: number | null,
  project: string,
  expected: Expected[],
  out: string,
) {
  const cli = mode === "cli" ? packet(stdout, status, project) : null;
  if (cli) {
    assert.equal(cli.result.warningCount, 0);
    assert.ok(cli.diagnostics.every((row) => row.severity === "error" && row.code !== null));
  }
  const actual = cli ? cli.diagnostics : nativeDiagnostics(stdout, status, project);
  save(path.join(out, "diagnostics.json"), actual);
  const identity = (
    row: Expected | { file: string; line: number; column: number; code: number | null },
  ) =>
    JSON.stringify({
      file: row.file.replaceAll("\\", "/"),
      line: row.line,
      column: row.column,
      code: row.code,
    });
  assert.deepEqual(
    actual.map(identity).sort(),
    expected.map(identity).sort(),
    "every authored token and duplicate count must survive; output alone is not the oracle",
  );
  assert.ok(actual.every((row) => row.message.length > 0));
  return cli;
}
