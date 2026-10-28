import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve, join } from "node:path";

interface Position {
  column: number;
  line: number;
  offset: number;
}
interface Diagnostic {
  help: string;
  location: { start: Position; end: Position };
  message: string;
  rule: string;
  severity: string;
}
interface LintReport {
  diagnostics: Diagnostic[];
  errorCount: number;
  filename: string;
  warningCount: number;
}
interface Before {
  version: string;
  source: string;
  options: Record<string, unknown>;
  result: LintReport;
}
const workspaceArg = process.argv[2];
const version = process.argv[3];
const out = process.argv[4];
assert.ok(workspaceArg, "WORKSPACE is required");
assert.ok(version, "VERSION is required");
assert.ok(out, "OUTPUT is required");
const require = createRequire(join(resolve(workspaceArg), "package.json"));
const manifest = require("@vizejs/native/package.json") as { version: string };
assert.equal(manifest.version, version);
const beforeBytes = readFileSync(new URL("./before-public-0.599.0.json.txt", import.meta.url));
assert.equal(
  createHash("sha256").update(beforeBytes).digest("hex"),
  "c205b3deee856416cbf26181b729a1143df51c111600670ec395fa2f8542aebf",
);
const before = JSON.parse(beforeBytes.toString("utf8")) as Before;
assert.equal(before.version, "0.599.0");
assert.equal(before.result.diagnostics.length, 6);
const expected = structuredClone(before.result);
const originalDiagnostic = before.result.diagnostics[0];
assert.ok(originalDiagnostic);
for (const tag of ['<i class="mdi mdi-close" />', '<i class="ri-home-line" />']) {
  const start = before.source.indexOf(tag);
  assert.ok(start >= 0);
  const end = start + tag.length;
  const position = (offset: number): Position => {
    const lines = before.source.slice(0, offset).split("\n");
    const last = lines.at(-1);
    assert.ok(last !== undefined);
    return { column: last.length + 1, line: lines.length, offset };
  };
  expected.diagnostics.push({
    ...structuredClone(originalDiagnostic),
    location: { start: position(start), end: position(end) },
  });
}
expected.diagnostics.sort((a, b) => a.location.start.offset - b.location.start.offset);
expected.warningCount = 8;
const native = require("@vizejs/native") as {
  lintPatinaSfc(source: string, options: Record<string, unknown>): LintReport;
};
const actual = native.lintPatinaSfc(before.source, before.options);
writeFileSync(
  out,
  JSON.stringify(
    {
      version,
      source: before.source,
      options: before.options,
      actual,
      expected,
      before: before.result,
    },
    null,
    2,
  ) + "\n",
);
assert.deepEqual(actual, expected);
console.log(
  JSON.stringify({
    version,
    exactWarnings: actual.warningCount,
    oldPacketsPreserved: before.result.diagnostics.length,
  }),
);
