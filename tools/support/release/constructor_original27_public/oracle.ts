import assert from "node:assert/strict";
import { relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { asArray, asRecord, asString, readJsonRecord } from "../../json-value.ts";
import { packet } from "../../../benchmarks/scripts/canon-instance-directives-model.ts";
import { sha256 } from "../computed_event_custody.ts";

export const sourceReference = "b7592b0211060bd63de0e98cb1684dd4ad8c52fe";
export function reference() {
  return readJsonRecord(fileURLToPath(new URL("./source-reference.json.txt", import.meta.url)));
}
export type Diagnostic = {
  file: string;
  line: number;
  column: number;
  code: number;
  message: string;
  continuation: string[];
};
export function diagnostics(stdout: Buffer, project: string): Diagnostic[] {
  const text = new TextDecoder("utf-8", { fatal: true }).decode(stdout);
  const rows: Diagnostic[] = [];
  if (!text) return rows;
  assert.ok(text.endsWith("\n"), "compiler output must retain its terminal newline");
  for (const line of text.slice(0, -1).split("\n")) {
    const match = /^(.*?)\((\d+),(\d+)\): error TS(\d+): (.*)$/u.exec(line);
    if (match)
      rows.push({
        file: relative(project, resolve(project, match[1])).replaceAll("\\", "/"),
        line: +match[2],
        column: +match[3],
        code: +match[4],
        message: match[5],
        continuation: [],
      });
    else {
      assert.ok(rows.length && /^\s/u.test(line), "unclassified compiler output: " + line);
      rows.at(-1)!.continuation.push(line);
    }
  }
  return rows;
}
/** Only the known mapper package label changes; generated positions/messages stay exact. */
export function expectedMapper(version: string, input = asArray(reference().mapper)) {
  const rows = input.map((value) => structuredClone(asRecord(value)));
  assert.equal(rows.length, 16);
  const debts = rows.filter((row) => row.code === 2741);
  assert.deepEqual(
    debts.map((row) => [row.file, row.line, row.column]),
    [
      ["constructor-public-props-discrepancy/src/Tiny.vue", 57, 495],
      ["constructor-public-props-discrepancy/src/Tiny.vue", 108, 501],
    ],
  );
  const line = (label: string) =>
    `  This location is in virtual code produced by the content mapper 'vize@${label}' and has no corresponding location in the original file.`;
  for (const row of rows) {
    if (debts.includes(row)) {
      assert.deepEqual(row.continuation, [line("0.0.0-source-" + sourceReference)]);
      row.continuation = [line(version)];
    } else
      for (const value of asArray(row.continuation))
        assert.ok(
          !asString(value).includes("vize@0.0.0-source-"),
          "unexpected source-version continuation",
        );
  }
  return rows;
}
/** Derive one finite repaired expectation; never drop a file-wide error population. */
export function repairExpected(rows: unknown[], role: "mapper" | "cli") {
  const matches = rows
    .map((value, index) => ({ row: asRecord(value), index }))
    .filter(({ row }) => row.file === "wrong-prop/src/Tiny.vue");
  assert.equal(matches.length, 1, "exactly one original wrong-prop row required");
  const pins = {
    mapper: "6604b1308bdcc2a583c26edcf33977e00574c9aebaa390abcb8db82e62f1b886",
    cli: "8817a6487ce4fc3e05d87ab4a79fb4452183ab4384eee8699487c549fc29996d",
  };
  assert.equal(
    sha256(JSON.stringify(matches[0].row)),
    pins[role],
    "complete original repair row differs",
  );
  return rows.filter((_value, index) => index !== matches[0].index);
}
export function assertOriginalVectors(
  version: string,
  mapper: unknown[],
  cli: unknown[],
  repaired = false,
) {
  const expected = expectedMapper(version),
    displayed = asArray(reference().cli);
  assert.deepEqual(
    mapper,
    repaired ? repairExpected(expected, "mapper") : expected,
    "complete mapper vector",
  );
  assert.deepEqual(
    cli,
    repaired ? repairExpected(displayed, "cli") : displayed,
    "complete CLI vector",
  );
}
export function cliPacket(stdout: Buffer, status: number | null, project: string, ids: string[]) {
  const result = packet(stdout, status, project);
  const files = result.result.files.map((file) =>
    relative(project, resolve(project, file.file)).replaceAll("\\", "/"),
  );
  assert.equal(new Set(files).size, 54, "complete original file population required");
  assert.deepEqual(
    [...files].sort(),
    ids.flatMap((id) => [id + "/component.ts", id + "/src/Tiny.vue"]).sort(),
  );
  for (const file of result.result.files)
    assert.equal(typeof file.virtualTs, "string", "retain every complete virtual TS packet");
  return result;
}
