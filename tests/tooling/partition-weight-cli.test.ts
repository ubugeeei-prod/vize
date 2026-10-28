import assert from "node:assert/strict";
import { test } from "node:test";
import { spawnSync } from "node:child_process";
import {
  partitionWeightInputs,
  verifyPartitionWeightPacket,
  partitionBackendObserver,
} from "../../tools/benchmarks/scripts/partition-weight-cli.ts";
void test("whole shard packets require every authored file and independent negative count", () => {
  const inputs = partitionWeightInputs("/public");
  assert.equal(inputs.length, 26);
  assert.equal(inputs.filter((v) => v.errorLine !== null).length, 24);
  const packet = {
    fileCount: 26,
    errorCount: 24,
    warningCount: 0,
    files: inputs.map((v) => ({
      file: v.path,
      diagnostics:
        v.errorLine === null
          ? []
          : [
              "error:" +
                v.errorLine +
                ":7 [TS2322] Type 'string' is not assignable to type 'number'.",
            ],
    })),
  };
  assert.equal(
    verifyPartitionWeightPacket(Buffer.from(JSON.stringify(packet)), "/public", inputs, true)
      .errorCount,
    24,
  );
  for (const invalid of [
    { ...packet, files: packet.files.slice(1) },
    { ...packet, errorCount: 0 },
    { ...packet, files: packet.files.map((v, i) => (i === 0 ? { ...v, diagnostics: [] } : v)) },
    { ...packet, files: [packet.files[0], ...packet.files.slice(0, -1)] },
  ])
    assert.throws(() =>
      verifyPartitionWeightPacket(Buffer.from(JSON.stringify(invalid)), "/public", inputs, true),
    );
});
void test("backend observer remains executable JS without adding checker or stats flags", () => {
  const child = spawnSync(process.execPath, ["--check", "--input-type=module"], {
    input: partitionBackendObserver("/public/tsc", "/public/trace"),
    encoding: "utf8",
  });
  assert.equal(child.status, 0, child.stderr);
  assert.ok(!partitionBackendObserver("b", "t").includes("--extendedDiagnostics"));
});
