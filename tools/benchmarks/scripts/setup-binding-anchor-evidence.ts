/** Compare full source generation packets and allocation pressure. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { isAbsolute, join } from "node:path";

const directory = process.argv[2];
assert.ok(directory);
const read = (name: string) => JSON.parse(readFileSync(join(directory, name), "utf8"));
const digest = (text: string) => createHash("sha256").update(text).digest("hex");
const median = (values: number[]) => [...values].sort((a, b) => a - b)[1];
const binaries = read("binaries.json");
assert.notEqual(binaries.base.sha256, binaries.head.sha256, "role-isolated binaries must differ");
const pinnedSha256 = "73d98e1625eac89e544e822565ea41e066bec3b3f38f747ed3263dda9b02358a";
let pinnedInput: unknown;

function completeDocuments(row: {
  source: string;
  documents: Record<string, unknown>[];
  pinnedInput: { path: string; source: string };
}) {
  assert.ok(isAbsolute(row.pinnedInput.path));
  assert.deepEqual(Object.keys(row.pinnedInput).sort(), ["path", "source"]);
  assert.equal(digest(row.pinnedInput.source), pinnedSha256, "exact pinned input bytes");
  pinnedInput ??= row.pinnedInput;
  assert.deepEqual(row.pinnedInput, pinnedInput, "one physical input in every role and cell");
  assert.equal(row.documents.length, 4);
  for (const [index, document] of row.documents.entries()) {
    const kind = index < 2 ? "measured" : "pinned";
    assert.deepEqual(Object.keys(document).sort(), [
      "code", "hoist", "importSourceMapDebug", "kind", "mappingDebug", "preRewriteCode",
      "source", "sourceTypeDebug", "virtualFile", "virtualSuffix",
    ]);
    assert.equal(document.kind, kind);
    assert.equal(document.hoist, index % 2 === 1);
    assert.equal(document.virtualFile, kind === "measured" ? "src/Bindings.vue" : "src/Imports.vue");
    assert.equal(document.source, kind === "measured" ? row.source : row.pinnedInput.source);
    for (const field of ["code", "preRewriteCode", "mappingDebug", "importSourceMapDebug", "sourceTypeDebug", "virtualSuffix"])
      assert.equal(typeof document[field], "string", field);
  }
  return { documents: row.documents, pinnedInput: row.pinnedInput };
}

const summary = [];
for (const shape of ["locals", "imports"]) {
  for (const count of [32, 128, 512]) {
    let reference: { source: string; packet: unknown; documents: unknown; pinnedInput: unknown } | undefined;
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
          const packet = { source: row.source, packet: row.packet, ...completeDocuments(row) };
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
    assert.ok(
      median(head.bytes) <= median(base.bytes),
      "whole-generation requested bytes must not regress",
    );
    summary.push({
      shape,
      count,
      sourceSha256: digest(reference!.source),
      packetSha256: digest(JSON.stringify(reference!.packet)),
      documentsSha256: digest(JSON.stringify(reference!.documents)),
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
  completeDocumentObservations: 288,
  pinnedInputSha256: pinnedSha256,
  summary,
  scope:
    "actual single-source generation and allocations; runtime-off timing is a profiling build, not normal release cold-check latency",
};
writeFileSync(join(directory, "comparison.json"), JSON.stringify(receipt, null, 2) + "\n");
console.log(JSON.stringify(receipt));
