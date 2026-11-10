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

const previousName = "source-packets-current-b6b4bbb0a16a.json";
const observedName = "source-packets-current-67b6fffe3bc8.json";
const changedLabels = [
  "lf/slot-shadow",
  "lf/mixed-slot-scope",
  "crlf-unicode/slot-shadow",
  "crlf-unicode/mixed-slot-scope",
];

test("the CLI check projection omits only four wire-only template-ref links", () => {
  const { authority, files, cases } = readGlobalsFixture();
  // The parallel observation never replaces main's newer CURRENT population.
  assert.notEqual(authority.currentSourcePackets, observedName);
  assert.notEqual(authority.currentSourcePackets, previousName);
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
  assert.equal(provenance.producer.sourceRevision, "67b6fffe3bc88922d9d460b15d76fd7ceb35d06c");
  assert.equal(provenance.producer.pullRequestHead, "1b75300829290185308e49e98c2d3b4c3d6c9575");
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
