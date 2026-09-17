import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

test("pattern declarations are checked and preserve structural coverage", () => {
  const declarations = fileURLToPath(
    new URL(
      "../../crates/vize_canon/src/virtual_ts/helpers/pattern_matching.d.ts",
      import.meta.url,
    ),
  );
  const fixture = fileURLToPath(new URL("../_fixtures/canon-pattern-types.ts", import.meta.url));
  const license = readFileSync(declarations.replace(/\.d\.ts$/, ".LICENSE"), "utf8").trimEnd();
  assert.ok(
    readFileSync(declarations, "utf8").includes(license),
    "embedded helpers retain the complete MIT notice",
  );
  for (const exactOptionalPropertyTypes of [false, true]) {
    const program = ts.createProgram([declarations, fixture], {
      noEmit: true,
      strict: true,
      skipLibCheck: false,
      types: [],
      target: ts.ScriptTarget.ESNext,
      exactOptionalPropertyTypes,
    });
    assert.deepEqual(
      ts.getPreEmitDiagnostics(program).map((diagnostic) => ({
        file: diagnostic.file?.fileName,
        start: diagnostic.start,
        message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
      })),
      [],
      `exactOptionalPropertyTypes=${exactOptionalPropertyTypes}`,
    );
  }
});
