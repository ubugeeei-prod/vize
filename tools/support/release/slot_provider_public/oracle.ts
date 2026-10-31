import assert from "node:assert/strict";
import { relative, resolve } from "node:path";
import { packet } from "../../../benchmarks/scripts/canon-instance-directives-model.ts";
import type { Stage, Row } from "./fixtures.ts";

/** Preserve every line, continuation and final space; only the terminal LF is structural. */
export function diagnostics(stdout: Buffer, project: string): Row[] {
  const text = new TextDecoder("utf-8", { fatal: true }).decode(stdout);
  if (!text) return [];
  assert.ok(text.endsWith("\n"), "complete terminal LF required");
  const rows: Row[] = [];
  for (const line of text.slice(0, -1).split("\n")) {
    const match = /^(.*?)\((\d+),(\d+)\): error TS(\d+): (.*)$/u.exec(line);
    if (match)
      rows.push({
        file: relative(project, resolve(project, match[1])).replaceAll("\\", "/"),
        line: Number(match[2]),
        column: Number(match[3]),
        code: Number(match[4]),
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

/** The literal quiet C8 command dumps its shared preamble and every declared VTS body. */
export function cliObservation(
  stdout: Buffer,
  stderr: Buffer,
  status: number | null,
  project: string,
  stage: Stage,
  helperText: string,
) {
  const observed = packet(stdout, status, project),
    result = observed.result;
  assert.equal(result.fileCount, 1);
  assert.equal(result.files.length, 1);
  assert.equal(result.warningCount, 0);
  assert.deepEqual(
    result.files.map((file) =>
      relative(project, resolve(project, file.file)).replaceAll("\\", "/"),
    ),
    ["src/Tiny.vue"],
  );
  for (const file of result.files) assert.ok(typeof file.virtualTs === "string" && file.virtualTs);
  const expectedStderr =
    "\n=== __vize_helpers.d.ts ===\n" +
    helperText +
    "\n" +
    result.files
      .map((file) => "\n=== " + resolve(project, file.file) + " ===\n" + file.virtualTs + "\n")
      .join("");
  assert.ok(
    stderr.equals(Buffer.from(expectedStderr)),
    "complete quiet CLI stderr must equal source-pinned helper and declared VTS bodies",
  );
  const source = stage.files.get("src/Tiny.vue")!;
  const expected = stage.expected.map((row) => ({
    file: row.file,
    line: row.line,
    column: row.column,
    code: row.code,
    message: row.message + " (source: " + source.split("\n")[row.line - 1]!.trim() + ")",
    severity: "error",
  }));
  assert.deepEqual(
    observed.diagnostics.map(({ raw: _raw, ...row }) => row),
    expected,
    "whole ordered authored CLI vector",
  );
  return observed;
}

export function assertCompilerVectors(
  stage: Stage,
  mapper: Buffer,
  framework: Buffer,
  project: string,
) {
  const mapped = diagnostics(mapper, project),
    official = diagnostics(framework, project);
  assert.deepEqual(mapped, stage.expected, "whole mapper vector");
  assert.deepEqual(official, stage.expected, "whole independent VueTSC vector");
  if (stage.frameworkStdout !== undefined)
    assert.ok(
      framework.equals(Buffer.from(stage.frameworkStdout)),
      "complete frozen independent collision framework stream",
    );
  return { mapper: mapped, framework: official };
}
