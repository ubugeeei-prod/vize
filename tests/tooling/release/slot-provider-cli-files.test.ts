import assert from "node:assert/strict";
import { test } from "node:test";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  loadFixtures,
  type Stage,
} from "../../../tools/support/release/slot_provider_public/fixtures.ts";
import {
  cliObservation,
  cliSourceFiles,
} from "../../../tools/support/release/slot_provider_public/oracle.ts";

const fixtures = loadFixtures(fileURLToPath(new URL("../../../", import.meta.url)));
const stage = (population: string, id: string) => {
  const row = fixtures.stages.find((row) => row.population === population && row.id === id);
  assert.ok(row);
  return row;
};
type File = { file: string; virtualTs: string; diagnostics: string[] };
function syntheticCli(row: Stage) {
  const source = row.files.get("src/Tiny.vue")!;
  const result = {
    fileCount: cliSourceFiles(row).length,
    errorCount: row.expected.length,
    warningCount: 0,
    files: cliSourceFiles(row).map((file): File => ({
      file,
      virtualTs:
        file === "src/Tiny.vue"
          ? "// explicitly synthetic parser control\nexport {};\n"
          : row.files.get(file)!,
      diagnostics:
        file !== "src/Tiny.vue"
          ? []
          : row.expected.map(
              (diagnostic) =>
                "error:" +
                diagnostic.line +
                ":" +
                diagnostic.column +
                " [TS" +
                diagnostic.code +
                "] " +
                diagnostic.message +
                " (source: " +
                source.split("\n")[diagnostic.line - 1]!.trim() +
                ")",
            ),
    })),
  };
  return result;
}
function stderrFor(files: File[]) {
  return Buffer.from(
    "\n=== __vize_helpers.d.ts ===\n" +
      fixtures.helperText +
      "\n" +
      files
        .map((file) => "\n=== " + resolve("/project", file.file) + " ===\n" + file.virtualTs + "\n")
        .join(""),
  );
}
function observe(row: Stage, result = syntheticCli(row), stderr = stderrFor(result.files)) {
  return cliObservation(
    Buffer.from(JSON.stringify(result)),
    stderr,
    result.errorCount ? 1 : 0,
    "/project",
    row,
    fixtures.helperText,
  );
}

test("all48 frozen configurations independently declare2/3/4 ordered CLI files", () => {
  for (const row of fixtures.stages) {
    const expected =
      row.population === "registered15"
        ? ["component.ts", "globals.d.ts", "src/Tiny.vue"]
        : row.population === "collision16"
          ? ["component.ts", "globals.d.ts", "src/Tiny.vue", "types.ts"]
          : ["component.ts", "src/Tiny.vue"];
    assert.deepEqual(cliSourceFiles(row), expected);
  }
});
test("the original valid two-file CLI shape passes without dropping its plain TS source", () => {
  const row = stage("original10", "valid"),
    result = syntheticCli(row);
  assert.equal(result.fileCount, 2);
  assert.equal(result.files[0].virtualTs, row.files.get("component.ts"));
  assert.deepEqual(observe(row, result).diagnostics, []);
});
test("all48 synthetic decoders retain every independent whole authored vector", () => {
  for (const row of fixtures.stages) observe(row);
});
for (const mutation of ["omitted", "duplicate", "extra", "reordered"] as const) {
  test("CLI refuses " + mutation + " plain TS file even with coherent count and stderr", () => {
    const row = stage("original10", "valid"),
      result = syntheticCli(row);
    if (mutation === "omitted") result.files.shift();
    if (mutation === "duplicate") result.files.unshift(structuredClone(result.files[0]));
    if (mutation === "extra")
      result.files.push({ file: "extra.ts", virtualTs: "export {};\n", diagnostics: [] });
    if (mutation === "reordered") result.files.reverse();
    result.fileCount = result.files.length;
    assert.throws(() => observe(row, result));
  });
}
test("plain TS body mutation refuses matching stdout/stderr instead of trusting observed bodies", () => {
  for (const population of ["original10", "registered15", "collision16"]) {
    const row = fixtures.stages.find((row) => row.population === population)!;
    for (const name of cliSourceFiles(row).filter((name) => name !== "src/Tiny.vue")) {
      const result = syntheticCli(row);
      result.files.find((file) => file.file === name)!.virtualTs += "\n";
      assert.throws(() => observe(row, result), /whole plain TS body/u);
    }
  }
});
test("fileCount cannot omit a plain TS file while the files array retains it", () => {
  const row = stage("original10", "valid"),
    result = syntheticCli(row);
  result.fileCount = 1;
  assert.throws(() => observe(row, result));
});
test("pinned config/source mutations cannot be inferred away from observed CLI results", () => {
  const original = stage("original10", "valid");
  for (const include of [
    ["src/**/*.vue"],
    ["src/**/*.vue", "component.ts", "component.ts"],
    ["src/**/*.vue", "component.ts", "extra.ts"],
  ]) {
    const changed = { ...original, files: new Map(original.files) };
    changed.files.set("tsconfig.json", JSON.stringify({ include }));
    assert.throws(() => cliSourceFiles(changed));
  }
  const changed = { ...original, files: new Map(original.files) };
  changed.files.set("component.ts", changed.files.get("component.ts")! + "\n");
  assert.throws(() => observe(changed, syntheticCli(original)), /whole plain TS body/u);
});
test("quiet CLI decoder conserves whole helper and all declared VTS bodies with no remainder", () => {
  const row = stage("imported7", "declared-any"),
    result = syntheticCli(row),
    stderr = stderrFor(result.files);
  observe(row, result, stderr);
  for (const mutation of [
    Buffer.concat([stderr, Buffer.from("warning\n")]),
    Buffer.from(stderr.toString().replace("synthetic parser", "changed parser")),
    Buffer.from(stderr.toString().replace("__vize_helpers", "__other_helpers")),
    stderr.subarray(0, stderr.length - 1),
    stderrFor(result.files.slice(1)),
  ])
    assert.throws(() => observe(row, result, mutation));
});
test("plain TS diagnostics remain in the complete ordered CLI vector", () => {
  const row = stage("original10", "valid"),
    result = syntheticCli(row);
  result.errorCount = 1;
  result.files[0].diagnostics.push("error:1:1 [TS2322] unexpected plain TS diagnostic");
  assert.throws(() => observe(row, result), /whole ordered authored CLI vector/u);
});
