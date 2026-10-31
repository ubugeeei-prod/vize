import assert from "node:assert/strict";
import { relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asRecord,
  asString,
  readJsonRecord,
} from "../../json-value.ts";
import { packet } from "../../../benchmarks/scripts/canon-instance-directives-model.ts";

export const sourceReference = "b7592b0211060bd63de0e98cb1684dd4ad8c52fe";
export function reference() {
  return readJsonRecord(
    fileURLToPath(new URL("./source-reference.json.txt", import.meta.url)),
  );
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
  for (const line of text.trimEnd().split("\n").filter(Boolean)) {
    const match = /^(.*?)\((\d+),(\d+)\): error TS(\d+): (.*)$/u.exec(line);
    if (match)
      rows.push({
        file: relative(project, resolve(project, match[1])).replaceAll(
          "\\",
          "/",
        ),
        line: +match[2],
        column: +match[3],
        code: +match[4],
        message: match[5],
        continuation: [],
      });
    else {
      assert.ok(
        rows.length && /^\s/u.test(line),
        "unclassified compiler output: " + line,
      );
      rows.at(-1)!.continuation.push(line);
    }
  }
  return rows;
}
/** Only the known mapper package label changes; generated positions/messages stay exact. */
export function expectedMapper(version: string) {
  const before = "vize@0.0.0-source-" + sourceReference;
  return asArray(reference().mapper).map((value) => {
    const row = structuredClone(asRecord(value));
    row.continuation = asArray(row.continuation).map((line) =>
      asString(line).replace(before, "vize@" + version),
    );
    return row;
  });
}
export function assertOriginalVectors(
  version: string,
  mapper: unknown[],
  cli: unknown[],
  repaired = false,
) {
  const keep = (row: unknown) =>
    !repaired || asRecord(row).file !== "wrong-prop/src/Tiny.vue";
  assert.deepEqual(
    mapper,
    expectedMapper(version).filter(keep),
    "complete mapper vector",
  );
  assert.deepEqual(
    cli,
    asArray(reference().cli).filter(keep),
    "complete CLI vector",
  );
}
export function cliPacket(
  stdout: Buffer,
  status: number | null,
  project: string,
  ids: string[],
) {
  const result = packet(stdout, status, project);
  const files = result.result.files.map((file) =>
    relative(project, resolve(project, file.file)).replaceAll("\\", "/"),
  );
  assert.equal(
    new Set(files).size,
    54,
    "complete original file population required",
  );
  assert.deepEqual(
    [...files].sort(),
    ids.flatMap((id) => [id + "/component.ts", id + "/src/Tiny.vue"]).sort(),
  );
  for (const file of result.result.files)
    assert.equal(
      typeof file.virtualTs,
      "string",
      "retain every complete virtual TS packet",
    );
  return result;
}
