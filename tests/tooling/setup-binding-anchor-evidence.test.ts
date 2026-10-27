import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

const observer = new URL(
  "../../tools/benchmarks/scripts/setup-binding-anchor-evidence.ts",
  import.meta.url,
);

function fixture(root: string) {
  const binaries = {
    base: { source: "a".repeat(40), sha256: "a".repeat(64) },
    head: { source: "b".repeat(40), sha256: "b".repeat(64) },
  };
  writeFileSync(join(root, "binaries.json"), JSON.stringify(binaries));
  for (const role of ["base", "head"] as const)
    for (const shape of ["locals", "imports"]) {
      for (const count of [32, 128, 512])
        for (const round of [1, 2, 3]) {
          for (const mode of ["runtime-off", "allocation"]) {
            const calls = role === "base" ? count * count : count;
            const row = {
              sourceRevision: binaries[role].source,
              shape,
              count,
              mode,
              source: `${shape} ${count} 日本 🧭`,
              packet: { text: "whole packet", mappings: [[0, 1, 0, 1]], diagnostics: [] },
              cold: { generationCalls: 1, warmups: 0, persistentCache: false },
              elapsedNanoseconds: 100,
              allocation: {
                calls: mode === "runtime-off" ? 0 : calls,
                requestedBytes: calls * 8,
                failures: 0,
              },
              spans:
                mode === "runtime-off"
                  ? []
                  : [
                      {
                        name: "canon.virtual_ts.emit_setup_binding_anchors",
                        calls: 1,
                        allocationCalls: calls,
                        requestedBytes: calls * 8,
                      },
                    ],
            };
            writeFileSync(
              join(root, `${role}-${shape}-${count}-${mode}-${round}.json`),
              JSON.stringify(row),
            );
          }
        }
    }
}
const run = (root: string) =>
  spawnSync(process.execPath, [observer.pathname, root], { encoding: "utf8", timeout: 10000 });

test("comparison retains all72 full observations and independent binary identities", () => {
  const root = mkdtempSync(join(tmpdir(), "anchor-evidence-"));
  try {
    fixture(root);
    const result = run(root);
    assert.equal(result.status, 0, result.stderr);
    const receipt = JSON.parse(readFileSync(join(root, "comparison.json"), "utf8"));
    assert.equal(receipt.completeObservations, 72);
    assert.equal(receipt.summary.length, 6);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("same binary, modified packet, skipped size and allocation increases are rejected", () => {
  for (const control of ["same-binary", "packet", "missing", "regression"]) {
    const root = mkdtempSync(join(tmpdir(), "anchor-evidence-invalid-"));
    try {
      fixture(root);
      const row = join(root, "head-imports-128-allocation-2.json");
      if (control === "same-binary") {
        const binaries = JSON.parse(readFileSync(join(root, "binaries.json"), "utf8"));
        binaries.head.sha256 = binaries.base.sha256;
        writeFileSync(join(root, "binaries.json"), JSON.stringify(binaries));
      } else if (control === "missing") rmSync(row);
      else if (control === "packet") {
        const packet = JSON.parse(readFileSync(row, "utf8"));
        packet.packet.diagnostics.push({ code: 2322, start: 0 });
        writeFileSync(row, JSON.stringify(packet));
      } else
        for (const round of [1, 2, 3]) {
          const path = join(root, `head-imports-128-allocation-${round}.json`);
          const packet = JSON.parse(readFileSync(path, "utf8"));
          packet.spans[0].allocationCalls = 128 * 128 + 1;
          writeFileSync(path, JSON.stringify(packet));
        }
      assert.notEqual(run(root).status, 0, control);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  }
});

test("unchanged allocations remain admissible for future optimized baselines", () => {
  const root = mkdtempSync(join(tmpdir(), "anchor-evidence-equal-"));
  try {
    fixture(root);
    for (const shape of ["locals", "imports"])
      for (const count of [32, 128, 512]) {
        for (const round of [1, 2, 3]) {
          const base = join(root, `base-${shape}-${count}-allocation-${round}.json`);
          const row = JSON.parse(readFileSync(base, "utf8"));
          const head = JSON.parse(
            readFileSync(join(root, `head-${shape}-${count}-allocation-${round}.json`), "utf8"),
          );
          row.allocation = head.allocation;
          row.spans = head.spans;
          writeFileSync(base, JSON.stringify(row));
        }
      }
    const result = run(root);
    assert.equal(result.status, 0, result.stderr);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
