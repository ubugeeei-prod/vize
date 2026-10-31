import assert from "node:assert/strict";
import { relative, resolve } from "node:path";
import { packet } from "../../../benchmarks/scripts/canon-instance-directives-model.ts";
import { asArray, asRecord, asString, parseJson } from "../../json-value.ts";
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

/** C8 reports the pinned tsconfig inputs, including plain TS and declaration sources. */
export function cliSourceFiles(stage: Stage) {
  const config = asRecord(parseJson(stage.files.get("tsconfig.json")!));
  assert.equal(config.files, undefined);
  assert.equal(config.exclude, undefined);
  assert.equal(config.extends, undefined);
  const names = asArray(config.include).flatMap((value) => {
    const pattern = asString(value);
    // These immutable populations use one SFC glob plus explicit local TS files.
    if (pattern === "src/**/*.vue") return ["src/Tiny.vue"];
    assert.ok(["component.ts", "globals.d.ts", "types.ts"].includes(pattern));
    return [pattern];
  });
  assert.equal(new Set(names).size, names.length, "duplicate configured source");
  const order = (left: string, right: string) => (left < right ? -1 : left > right ? 1 : 0);
  names.sort(order);
  assert.deepEqual(
    [...stage.files.keys()].filter((name) => name !== "tsconfig.json").sort(order),
    names,
    "every pinned source must belong to the literal CLI program",
  );
  return names;
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
  const sourceFiles = cliSourceFiles(stage);
  assert.equal(result.fileCount, sourceFiles.length);
  assert.equal(result.files.length, sourceFiles.length);
  assert.equal(result.warningCount, 0);
  assert.deepEqual(
    result.files.map((file) =>
      relative(project, resolve(project, file.file)).replaceAll("\\", "/"),
    ),
    sourceFiles,
  );
  for (const [index, file] of result.files.entries()) {
    assert.ok(typeof file.virtualTs === "string" && file.virtualTs);
    if (sourceFiles[index] !== "src/Tiny.vue")
      assert.equal(file.virtualTs, stage.files.get(sourceFiles[index]), "whole plain TS body");
  }
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
