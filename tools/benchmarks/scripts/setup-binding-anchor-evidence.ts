/** Compare full source generation packets and allocation pressure. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const directory = process.argv[2];
assert.ok(directory);
const read = (name: string) => JSON.parse(readFileSync(join(directory, name), "utf8"));
const digest = (text: string) => createHash("sha256").update(text).digest("hex");
const median = (values: number[]) => [...values].sort((a, b) => a - b)[1];
const binaries = read("binaries.json");
assert.notEqual(binaries.base.sha256, binaries.head.sha256, "role-isolated binaries must differ");
const summary = [];
for (const shape of ["locals", "imports"]) {
  for (const count of [32, 128, 512]) {
    let reference: { source: string; packet: unknown } | undefined;
    const roles = new Map<
      string,
      {
        wall: number[];
        allocations: number[];
        bytes: number[];
        anchorCalls: number[];
        anchorBytes: number[];
      }
    >();
    for (const role of ["base", "head"]) {
      const result = {
        wall: [] as number[],
        allocations: [] as number[],
        bytes: [] as number[],
        anchorCalls: [] as number[],
        anchorBytes: [] as number[],
      };
      for (const round of [1, 2, 3]) {
        for (const mode of ["runtime-off", "allocation"]) {
          const row = read(`${role}-${shape}-${count}-${mode}-${round}.json`);
          assert.equal(row.sourceRevision, binaries[role].source);
          assert.deepEqual([row.shape, row.count, row.mode], [shape, count, mode]);
          assert.deepEqual(row.cold, { generationCalls: 1, warmups: 0, persistentCache: false });
          const packet = { source: row.source, packet: row.packet };
          reference ??= packet;
          assert.deepEqual(
            packet,
            reference,
            "complete packet bytes and fields must match across sources, modes and repeats",
          );
          if (mode === "runtime-off") {
            assert.equal(row.allocation.calls, 0);
            assert.deepEqual(row.spans, []);
            result.wall.push(row.elapsedNanoseconds);
          } else {
            assert.equal(row.allocation.failures, 0);
            const anchor = row.spans.find(
              (span: { name: string }) =>
                span.name === "canon.virtual_ts.emit_setup_binding_anchors",
            );
            assert.equal(anchor.calls, 1);
            result.allocations.push(row.allocation.calls);
            result.bytes.push(row.allocation.requestedBytes);
            result.anchorCalls.push(anchor.allocationCalls);
            result.anchorBytes.push(anchor.requestedBytes);
          }
        }
      }
      roles.set(role, result);
    }
    const base = roles.get("base")!;
    const head = roles.get("head")!;
    assert.ok(
      median(head.anchorCalls) <= median(base.anchorCalls),
      "actual anchor allocation count must not regress",
    );
    assert.ok(
      median(head.anchorBytes) <= median(base.anchorBytes),
      "actual anchor requested bytes must not regress",
    );
    assert.ok(
      median(head.allocations) <= median(base.allocations),
      "whole-generation allocation count must not regress",
    );
    summary.push({
      shape,
      count,
      sourceSha256: digest(reference!.source),
      packetSha256: digest(JSON.stringify(reference!.packet)),
      base,
      head,
      median: {
        baseRuntimeOffNs: median(base.wall),
        headRuntimeOffNs: median(head.wall),
        baseAnchorCalls: median(base.anchorCalls),
        headAnchorCalls: median(head.anchorCalls),
        baseAnchorBytes: median(base.anchorBytes),
        headAnchorBytes: median(head.anchorBytes),
      },
    });
  }
}
const receipt = {
  schema: "vize.setup-binding-anchor-comparison",
  binaries,
  completeObservations: 72,
  summary,
  scope:
    "actual single-source generation and allocations; runtime-off timing is a profiling build, not normal release cold-check latency",
};
writeFileSync(join(directory, "comparison.json"), JSON.stringify(receipt, null, 2) + "\n");
console.log(JSON.stringify(receipt));
