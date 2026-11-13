import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import {
  readGlobalsFixture,
  sha,
  type Transform,
} from "./support/content-mapper-instance-globals-10617.ts";
import { selectPacketGoldens } from "./support/content-mapper-instance-globals-packets-10617.ts";

type SemanticLink = {
  kind: string;
  sourceStart: number;
  sourceLength: number;
  targetStart: number;
  targetLength: number;
};
type Result = Transform & { semanticLinks?: SemanticLink[] };

const previousName = "source-packets-current-c6a6c571b090.json";
const observedName = "source-packets-current-a0fe19e9a57e.json";
const changedLabels = [
  "lf/slot-shadow",
  "lf/mixed-slot-scope",
  "crlf-unicode/slot-shadow",
  "crlf-unicode/mixed-slot-scope",
];

test("the mapper wire omits only four wire-only template-ref links", () => {
  const { authority, files, cases } = readGlobalsFixture();
  const observed = selectPacketGoldens(
    { ...authority, currentSourcePackets: observedName },
    files,
    cases,
  )!;
  const previous = selectPacketGoldens(
    { ...authority, currentSourcePackets: previousName },
    files,
    cases,
  )!;
  const provenance = JSON.parse(files.get(observed.provenance as string)!);
  assert.equal(provenance.pullRequest, 12604);
  assert.equal(provenance.producer.sourceRevision, "a0fe19e9a57e31b1a2f785273477ea10c5d2c3ae");
  assert.equal(provenance.producer.pullRequestHead, "393142e7e6e77c9dde476131485011bfef8dfe59");
  assert.deepEqual(provenance.sourceReceipt, {
    path: "crates/vize/src/commands/content_mapper/mode.rs",
    sha256: sha(
      readFileSync(
        new URL("../../crates/vize/src/commands/content_mapper/mode.rs", import.meta.url),
      ),
    ),
  });
  assert.deepEqual(provenance.previousPopulation, {
    path: previousName,
    bytes: Buffer.byteLength(files.get(previousName)!),
    sha256: sha(files.get(previousName)!),
  });
  assert.deepEqual(provenance.successorPopulation, {
    path: observedName,
    bytes: Buffer.byteLength(files.get(observedName)!),
    sha256: sha(files.get(observedName)!),
  });

  const changed: string[] = [];
  let identical = 0;
  for (const [index, packet] of observed.packets.entries()) {
    const oldPacket = previous.packets[index]!;
    assert.equal(packet.label, oldPacket.label);
    assert.equal(packet.sourceSha256, oldPacket.sourceSha256);
    assert.equal(packet.globalsSha256, oldPacket.globalsSha256);
    if (JSON.stringify(packet.result) === JSON.stringify(oldPacket.result)) {
      identical += 1;
      continue;
    }

    changed.push(packet.label);
    const before = structuredClone(oldPacket.result) as Result;
    const links = before.semanticLinks;
    assert.deepEqual(links, [
      {
        kind: "vueSetupTemplateRefUnwrap",
        sourceStart: 1411,
        sourceLength: 5,
        targetStart: 1500,
        targetLength: 5,
      },
    ]);
    const text = Buffer.from(before.text);
    for (const link of links!) {
      assert.equal(
        text.subarray(link.sourceStart, link.sourceStart + link.sourceLength).toString(),
        "Child",
      );
      assert.equal(
        text.subarray(link.targetStart, link.targetStart + link.targetLength).toString(),
        "Child",
      );
    }
    delete before.semanticLinks;
    assert.equal(Object.hasOwn(packet.result, "semanticLinks"), false);
    assert.deepEqual(
      packet.result,
      before,
      "text, mappings, directives, diagnostics, and opaque fields stay exact",
    );
  }
  assert.equal(identical, 76);
  assert.deepEqual(changed, changedLabels);
});
