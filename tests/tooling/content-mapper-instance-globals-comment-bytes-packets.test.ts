import assert from "node:assert/strict";
import { test } from "node:test";
import { generatedCommentBytesCut } from "../../tools/support/compat/content-mapper/generated-comment-bytes-cut.ts";
import {
  range,
  rangeIncludingInsertions,
} from "../../tools/support/compat/content-mapper/projection-byte-relocation.ts";
import { withoutProducerUnmappedIgnores } from "../../tools/support/compat/content-mapper/producer-unmapped-ignores.ts";
import {
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

type Result = Transform & {
  semanticLinks?: {
    sourceStart: number;
    sourceLength: number;
    targetStart: number;
    targetLength: number;
  }[];
  diagnosticDirectives?: { directives: number[][]; unusedExpectDirectiveDiagnostics: unknown[] };
};

// The generated comment cut (#12621) removes debug-only `@vize-map` lines and
// the reasons after generated `@ts-ignore`; nothing else may move.
test("the comment-bytes population changes each packet only by the generated comment cut", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const name = "source-packets-current-1ca72e5ed642.json";
  const previousName = "source-packets-current-f29a8ee58999.json";
  assert.equal(authority.currentSourcePackets, name);
  const golden = selectPacketGoldens(authority, files, cases)!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: previousName },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(provenance.pullRequest, 12621);
  assert.equal(provenance.producer.sourceRevision, "1ca72e5ed642e45f993abf0eef07e3acc97e427a");
  assert.deepEqual(provenance.previousPopulation, {
    path: previousName,
    bytes: Buffer.byteLength(files.get(previousName)!),
    sha256: sha(files.get(previousName)!),
  });
  assert.equal(golden.packets.length, 80);
  let cutBytes = 0;
  for (const [index, packet] of golden.packets.entries()) {
    const old = previous.packets[index]!;
    assert.equal(packet.label, old.label);
    assert.equal(packet.sourceSha256, old.sourceSha256);
    assert.equal(packet.globalsSha256, old.globalsSha256);
    const base = withoutProducerUnmappedIgnores(old.result as Result);
    const head = withoutProducerUnmappedIgnores(packet.result as Result, { requireComplete: true });
    const cut = generatedCommentBytesCut(base.text);
    const before = Buffer.from(base.text);
    const edits = cut.edits;
    assert.ok(edits.length > 0, `${packet.label}: every packet carries a cut comment`);
    cutBytes += before.length - Buffer.byteLength(cut.text);
    const expected = structuredClone(base);
    expected.text = cut.text;
    expected.mappings = base.mappings.map(([g, length, ...rest]) => [
      ...range(g!, length!, edits, before.length),
      ...rest,
    ]);
    if (expected.semanticLinks)
      expected.semanticLinks = expected.semanticLinks.map((link) => {
        const [sourceStart, sourceLength] = range(
          link.sourceStart,
          link.sourceLength,
          edits,
          before.length,
        );
        const [targetStart, targetLength] = range(
          link.targetStart,
          link.targetLength,
          edits,
          before.length,
        );
        return { ...link, sourceStart, sourceLength, targetStart, targetLength };
      });
    if (expected.diagnosticDirectives)
      expected.diagnosticDirectives.directives = expected.diagnosticDirectives.directives.map(
        (tuple) => {
          const [g, length] = rangeIncludingInsertions(
            tuple[2]!,
            tuple[3]! - tuple[2]!,
            edits,
            before.length,
          );
          return [tuple[0]!, tuple[1]!, g, g + length, ...tuple.slice(4)];
        },
      );
    assert.deepEqual(
      head,
      expected,
      `${packet.label}: only the generated comment cut moves any byte or coordinate`,
    );
  }
  assert.ok(cutBytes > 0);
});
