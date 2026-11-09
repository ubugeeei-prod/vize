import assert from "node:assert/strict";
import { test } from "node:test";
import {
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

// The component constructor picks the public instance inline and keeps the
// bound instance behind `NoInfer` (#12523): two exact replacements per constructor.
const REPLACEMENTS = [
  [
    "...args: any[]) => $ComponentInstanceFor<$ComponentInstance, ",
    "...args: any[]) => ($IsAny<$AuthoredProps> extends true ? true : $IsUnknown<$AuthoredProps> extends true ? $UsePublicInstance<$JsxProps> : false) extends true ? $ComponentInstance : NoInfer<$ComponentInstanceFor<$ComponentInstance, ",
  ],
  [", $AuthoredProps, $JsxProps>;", ", $AuthoredProps, $JsxProps>>;"],
] as const;

type Row = number[];
type Link = { sourceStart: number; targetStart: number };
type Result = Transform & {
  diagnosticDirectives: { directives: Row[] };
  semanticLinks?: Link[];
};
type Edit = { start: number; end: number; replacement: Buffer };

function constructorEdits(text: Buffer): Edit[] {
  const edits: Edit[] = [];
  const [[head, nextHead], [tail, nextTail]] = REPLACEMENTS;
  for (let at = text.indexOf(head); at >= 0; at = text.indexOf(head, at + 1)) {
    const end = text.indexOf(tail, at);
    assert.ok(end > at, "every constructor has its instance tail");
    edits.push({
      start: at,
      end: at + Buffer.byteLength(head),
      replacement: Buffer.from(nextHead),
    });
    edits.push({
      start: end,
      end: end + Buffer.byteLength(tail),
      replacement: Buffer.from(nextTail),
    });
  }
  return edits.sort((left, right) => left.start - right.start);
}

function point(at: number, edits: readonly Edit[]): number {
  let shift = 0;
  for (const edit of edits) {
    if (edit.end <= at) shift += edit.replacement.length - (edit.end - edit.start);
    else assert.ok(edit.start >= at, "no generated coordinate falls inside an edit");
  }
  return at + shift;
}

test("the NoInfer constructor population only rewrites each constructor return", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const name = "source-packets-current-e28630387b44.json";
  const golden = selectPacketGoldens({ ...authority, currentSourcePackets: name }, files, cases)!;
  const provenance = JSON.parse(files.get(golden.provenance as string)!);
  assert.equal(provenance.pullRequest, 12523);
  assert.equal(
    name,
    `source-packets-current-${provenance.producer.sourceRevision.slice(0, 12)}.json`,
  );
  const previousName = "source-packets-current-d48ea4e08e0e.json";
  assert.deepEqual(provenance.previousPopulation, {
    path: previousName,
    bytes: Buffer.byteLength(files.get(previousName)!),
    sha256: sha(files.get(previousName)!),
  });
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: previousName },
    files,
    cases,
  )!;
  assert.equal(golden.packets.length, 80);
  const changed: string[] = [];
  for (const [index, packet] of golden.packets.entries()) {
    const before = previous.packets[index]!;
    assert.equal(packet.label, before.label);
    assert.equal(packet.sourceSha256, before.sourceSha256);
    assert.equal(packet.globalsSha256, before.globalsSha256);
    const old = before.result as Result;
    const edits = constructorEdits(Buffer.from(old.text));
    if (edits.length === 0) {
      assert.equal(JSON.stringify(packet.result), JSON.stringify(old), packet.label);
      continue;
    }
    changed.push(packet.label);
    const text = Buffer.from(old.text);
    const parts: Buffer[] = [];
    let cursor = 0;
    for (const edit of edits) {
      parts.push(text.subarray(cursor, edit.start), edit.replacement);
      cursor = edit.end;
    }
    parts.push(text.subarray(cursor));
    const mappings = old.mappings.map((row) => {
      const start = point(row[0]!, edits),
        end = point(row[0]! + row[1]!, edits);
      return [start, end - start, ...row.slice(2)];
    });
    const directives = old.diagnosticDirectives.directives.map(([a, b, start, stop, c, d]) => [
      a!,
      b!,
      point(start!, edits),
      point(stop!, edits),
      c!,
      d!,
    ]);
    assert.equal(
      JSON.stringify({
        ...old,
        text: Buffer.concat(parts).toString(),
        mappings,
        diagnosticDirectives: { ...old.diagnosticDirectives, directives },
        ...(old.semanticLinks
          ? {
              semanticLinks: old.semanticLinks.map((link) => ({
                ...link,
                sourceStart: point(link.sourceStart, edits),
                targetStart: point(link.targetStart, edits),
              })),
            }
          : {}),
      }),
      JSON.stringify(packet.result),
      `${packet.label}: every diagnostic, mapping and opaque field is unchanged apart from shifted coordinates`,
    );
  }
  // Every packet holds one component constructor.
  assert.deepEqual(
    changed,
    golden.packets.map((packet) => packet.label),
  );
});
