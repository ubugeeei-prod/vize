import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { publicInstanceReference } from "../../npm/volar/test/public-instance-reference.js";

const root = new URL("../../npm/volar/test/", import.meta.url);
const expected = [
  "basic.json",
  "updates-and-language.json",
  "art.json",
  "feature-boundaries.json",
].flatMap((file) => JSON.parse(readFileSync(new URL("expected-packets/" + file, root), "utf8")));
const provenance = JSON.parse(
  readFileSync(new URL("expected-packets.provenance.json", root), "utf8"),
);
const observed = JSON.parse(
  readFileSync(new URL("reference-transitions/public-instance-10933.json", root), "utf8"),
);

const complete = JSON.parse(
  readFileSync(new URL("reference-transitions/public-instance-10933-complete.json", root), "utf8"),
);

test("all38 captured native packets match the complete explicit transition", () => {
  assert.equal(complete.sourceReceipt.source, complete.sourceCandidate);
  assert.equal(complete.sourceReceipt.identity.kind, "native");
  assert.equal(complete.sourceReceipt.run, String(complete.actionsRun));
  const captured = JSON.stringify(complete.packets, null, 2) + "\n";
  assert.equal(Buffer.byteLength(captured), complete.artifactPacketBytes);
  assert.equal(createHash("sha256").update(captured).digest("hex"), complete.artifactPacketSha256);
  assert.equal(complete.packets.length, 38);
  assert.deepEqual(publicInstanceReference(expected), complete.packets);
});

test("the archived complete packets keep their original authentication", () => {
  assert.equal(
    createHash("sha256").update(JSON.stringify(expected)).digest("hex"),
    provenance.expectedPacketDataSha256,
  );
  assert.equal(expected.length, provenance.expectedPackets);
  const unchanged = structuredClone(expected);
  publicInstanceReference(expected);
  assert.deepEqual(expected, unchanged);
});

test("the explicit transition matches all26 complete observed native packets", () => {
  assert.equal(observed.packets.length, 26);
  assert.deepEqual(publicInstanceReference(expected).slice(0, 26), observed.packets);
});

test("unrelated code, coordinate, file and type-information changes are rejected", () => {
  const reference = publicInstanceReference(expected).slice(0, 26);
  for (const change of [
    (rows: typeof reference) => {
      rows[0].result[1].code = 2304;
    },
    (rows: typeof reference) => {
      rows[0].result[1].start++;
    },
    (rows: typeof reference) => {
      rows[0].result[1].messageText += " changed";
    },
    (rows: typeof reference) => {
      rows[0].result[1].file.text += "\nvoid 0;";
    },
    (rows: typeof reference) => {
      rows[1].result.displayParts[0].text += "changed";
    },
  ]) {
    const changed = structuredClone(observed.packets);
    change(changed);
    assert.throws(() => assert.deepEqual(changed, reference), { code: "ERR_ASSERTION" });
  }
});
