import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  prepareAdmissionScale,
  scaleDefaults,
} from "../../tools/benchmarks/scripts/type-admission-scale.ts";
import { scaleDiagnostics } from "../../tools/benchmarks/scripts/type-admission-scale-runner.ts";
import { scaleSfc } from "../../tools/benchmarks/scripts/type-admission-scale-sfc.ts";

// Every negative has its own original physical owner and independently located
// UTF-16 token. A diagnostic count alone cannot prove graph/file visitation.
test("scale corpus assigns every graph node and SFC three boundary probes", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-scale-custody-"));
  try {
    for (const shape of ["chain", "diamond", "scc", "crosspkg"] as const) {
      for (const stage of ["invalid", "poisoned"] as const) {
        const corpus = prepareAdmissionScale(path.join(temporary, `${shape}-${stage}`), temporary, {
          ...scaleDefaults,
          sfcs: 2,
          graphNodes: 8,
          shape,
          stage,
        });
        assert.equal(corpus.expected.length, 14);
        assert.equal(corpus.sfcs.length, 2);
        for (const entry of corpus.sfcs) assert.ok(entry.bytes >= scaleDefaults.sfcBytes);
        const files = new Map<string, number>();
        for (const diagnostic of corpus.expected) {
          files.set(diagnostic.file, (files.get(diagnostic.file) ?? 0) + 1);
          const source = fs.readFileSync(
            path.join(temporary, `${shape}-${stage}`, diagnostic.file),
            "utf8",
          );
          const line = source.split("\n")[diagnostic.line - 1];
          assert.ok(line && diagnostic.column >= 1 && diagnostic.column <= line.length);
          const token = line.slice(diagnostic.column - 1);
          if (stage === "poisoned") assert.match(token, /^false/);
          else if (diagnostic.file.endsWith(".vue")) assert.match(token, /^missingF\d+R\d+/);
          else assert.match(token, /^graphWitness:/);
        }
        assert.equal(files.size, 10);
        for (const [file, count] of files) assert.equal(count, file.endsWith(".vue") ? 3 : 1);
        assert.deepEqual(
          corpus.sfcs.map((entry) => entry.lineEndings),
          ["CRLF", "LF"],
        );
      }
    }
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

test("large single-file stress contains actual code and Unicode across boundary probes", () => {
  for (const bytes of [128 * 1024, 1024 * 1024, 10 * 1024 * 1024]) {
    const result = scaleSfc(0, { ...scaleDefaults, sfcBytes: bytes, stage: "poisoned" });
    assert.ok(Buffer.byteLength(result.source) >= bytes);
    assert.ok(result.rows > bytes / 512, "size must come from expressions, not padding");
    assert.equal(result.probes.length, 3);
    assert.ok(result.source.includes("🌱日本語"));
    assert.ok(result.source.includes("\r\n"));
    for (const probe of result.probes) assert.equal(result.source.split(probe.before).length, 2);
  }
});

test("scale diagnostic admission retains duplicates, warnings, and unknown native errors", () => {
  const row = "src/Scale00000.vue(5,8): error TS2339: Property 'bad' does not exist.\n";
  assert.equal(scaleDiagnostics(row + row, "mapper", "/project").length, 2);
  assert.throws(() =>
    scaleDiagnostics(
      "error TS2589: Type instantiation is excessively deep.\n",
      "mapper",
      "/project",
    ),
  );
  assert.throws(() =>
    scaleDiagnostics(
      JSON.stringify({
        files: [{ file: "x.vue", diagnostics: ["warning:1:1 [TS1234] owned warning"] }],
        errorCount: 0,
        warningCount: 1,
      }),
      "cli",
      "/project",
    ),
  );
});
