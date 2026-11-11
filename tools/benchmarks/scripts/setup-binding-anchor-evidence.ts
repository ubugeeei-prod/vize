/** Compare full source generation packets and allocation pressure. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { isAbsolute, join } from "node:path";
import {
  certifiedTransition,
  currentEmbeddedWithDefaults,
  previousEmbeddedWithDefaults,
} from "../../support/compat/content-mapper/canonical-helper-transition.ts";
import {
  readAssets,
  relocatePacket,
  type Packet,
} from "../../support/compat/content-mapper/constructor-projection-parity.ts";
import {
  range,
  type Edit,
} from "../../support/compat/content-mapper/projection-byte-relocation.ts";
import { certifyFallthroughSourceFields } from "../../support/compat/content-mapper/shared-usage-gates-transition.ts";

const directory = process.argv[2];
assert.ok(directory);
const read = (name: string) => JSON.parse(readFileSync(join(directory, name), "utf8"));
const digest = (text: string) => createHash("sha256").update(text).digest("hex");
const median = (values: number[]) => [...values].sort((a, b) => a - b)[1];
const binaries = read("binaries.json");
assert.notEqual(binaries.base.sha256, binaries.head.sha256, "role-isolated binaries must differ");
const pinnedSha256 = "73d98e1625eac89e544e822565ea41e066bec3b3f38f747ed3263dda9b02358a";
let pinnedInput: unknown;
const projectionAssets = readAssets(process.cwd(), process.cwd());

type CompleteDocument = Record<string, unknown> & {
  code: string;
  mappingDebug: string;
  preRewriteCode: string;
  source: string;
};

function relocateMappingDebug(debug: string, edits: Edit[], generatedBytes: number) {
  return debug.replace(
    /\b(gen_range|source_range|target_range): (\d+)\.\.(\d+)/gu,
    (whole, label: string, startText: string, endText: string) => {
      const start = Number(startText),
        end = Number(endText),
        [relocated, length] = range(start, end - start, edits, generatedBytes);
      return `${label}: ${relocated}..${relocated + length}`;
    },
  );
}

function certifyDocument(before: CompleteDocument, after: CompleteDocument) {
  const withDefaults = certifiedTransition(
    Buffer.from(before.code),
    Buffer.from(after.code),
    previousEmbeddedWithDefaults,
    currentEmbeddedWithDefaults,
    "embedded withDefaults docs return",
  );
  const edits: Edit[] = [
    ...(withDefaults ? [withDefaults] : []),
    ...certifyFallthroughSourceFields(Buffer.from(before.code), Buffer.from(after.code)),
  ].sort((left, right) => left.start - right.start);
  if (edits.length === 0) {
    assert.deepEqual(after, before);
    return;
  }
  const packet = (text: string): Packet => ({ text, mappings: [], diagnostics: [] });
  relocatePacket(packet(before.code), packet(after.code), before.source, projectionAssets);
  relocatePacket(
    packet(before.preRewriteCode),
    packet(after.preRewriteCode),
    before.source,
    projectionAssets,
  );
  const expected = structuredClone(before);
  expected.code = after.code;
  expected.preRewriteCode = after.preRewriteCode;
  expected.mappingDebug = relocateMappingDebug(
    before.mappingDebug,
    edits,
    Buffer.byteLength(before.code),
  );
  assert.deepEqual(
    after,
    expected,
    "only certified withDefaults and fallthrough-source document shifts may differ",
  );
}

function certifyCompletePacket(
  before: {
    source: string;
    packet: Packet;
    documents: CompleteDocument[];
    pinnedInput: unknown;
  },
  after: {
    source: string;
    packet: Packet;
    documents: CompleteDocument[];
    pinnedInput: unknown;
  },
) {
  assert.equal(after.source, before.source);
  assert.deepEqual(after.pinnedInput, before.pinnedInput);
  relocatePacket(before.packet, after.packet, before.source, projectionAssets);
  assert.equal(after.documents.length, before.documents.length);
  for (const [index, document] of before.documents.entries())
    certifyDocument(document, after.documents[index]!);
}

function completeDocuments(row: {
  source: string;
  documents: CompleteDocument[];
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
      "code",
      "hoist",
      "importSourceMapDebug",
      "kind",
      "mappingDebug",
      "preRewriteCode",
      "source",
      "sourceTypeDebug",
      "virtualFile",
      "virtualSuffix",
    ]);
    assert.equal(document.kind, kind);
    assert.equal(document.hoist, index % 2 === 1);
    assert.equal(
      document.virtualFile,
      kind === "measured" ? "src/Bindings.vue" : "src/Imports.vue",
    );
    assert.equal(document.source, kind === "measured" ? row.source : row.pinnedInput.source);
    for (const field of [
      "code",
      "preRewriteCode",
      "mappingDebug",
      "importSourceMapDebug",
      "sourceTypeDebug",
      "virtualSuffix",
    ])
      assert.equal(typeof document[field], "string", field);
  }
  return { documents: row.documents, pinnedInput: row.pinnedInput };
}

const summary = [];
for (const shape of ["locals", "imports"]) {
  for (const count of [32, 128, 512]) {
    let reference:
      | {
          source: string;
          packet: Packet;
          documents: CompleteDocument[];
          pinnedInput: unknown;
        }
      | undefined;
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
          if (reference) certifyCompletePacket(reference, packet);
          else reference = packet;
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
  certifiedProjectionTransition: {
    previousSha256: digest(previousEmbeddedWithDefaults),
    currentSha256: digest(currentEmbeddedWithDefaults),
    fallthroughSourceFields: true,
  },
  summary,
  scope:
    "actual single-source generation and allocations; complete packets differ only by the certified editor-only withDefaults documentation return and raw fallthrough-source fields; runtime-off timing is a profiling build, not normal release cold-check latency",
};
writeFileSync(join(directory, "comparison.json"), JSON.stringify(receipt, null, 2) + "\n");
console.log(JSON.stringify(receipt));
