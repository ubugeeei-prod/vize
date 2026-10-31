import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  historicalPublicInstanceReference,
  publicInstanceReference,
} from "../../npm/volar/test/public-instance-reference.js";

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
  assert.deepEqual(historicalPublicInstanceReference(expected), complete.packets);
});

test("the current helper display is a finite successor of the unchanged native archives", () => {
  const certificateBytes = readFileSync(
    new URL("reference-transitions/public-instance-helper-display-20261031.json", root),
  );
  const sha256 = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
  assert.equal(
    sha256(certificateBytes),
    "24b4390aa1b7636e4bc444527c01ca685e29bd12389e94a65f3461839e070ea4",
  );
  const certificate = JSON.parse(certificateBytes.toString("utf8"));
  assert.equal(certificate.kind, "source-owned-helper-display-successor");
  assert.equal(certificate.nativeAcceptanceCredit, false);
  for (const [file, digest] of [
    ["public-instance-10933-complete.json", certificate.historicalCompleteFileSha256],
    ["public-instance-10933.json", certificate.historicalPartialFileSha256],
  ]) {
    assert.equal(sha256(readFileSync(new URL("reference-transitions/" + file, root))), digest);
  }
  assert.equal(complete.artifactPacketSha256, certificate.historicalArtifactPacketSha256);
  const historical = historicalPublicInstanceReference(expected);
  const historicalBytes = JSON.stringify(historical, null, 2) + "\n";
  assert.equal(Buffer.byteLength(historicalBytes), certificate.historicalReferencePacketBytes);
  assert.equal(sha256(historicalBytes), certificate.historicalReferencePacketSha256);
  const successor = structuredClone(historical);
  assert.equal(certificate.changes.length, 9);
  for (const change of certificate.changes) {
    const packet = successor.find((row: { name: string }) => row.name === change.packet);
    assert.ok(packet);
    const diagnostic = packet.result[change.diagnostic];
    assert.equal(diagnostic.code, 2339);
    assert.equal(diagnostic.start, change.start);
    assert.equal(diagnostic.length, change.name.length);
    assert.equal(
      diagnostic.messageText,
      `Property '${change.name}' does not exist on type '__VizeTemplateInstance'.`,
    );
    diagnostic.messageText = `Property '${change.name}' does not exist on type '$TemplateInstance'.`;
  }
  assert.deepEqual(publicInstanceReference(expected), successor);
  const currentBytes = JSON.stringify(successor, null, 2) + "\n";
  assert.equal(Buffer.byteLength(currentBytes), certificate.currentReferencePacketBytes);
  assert.equal(sha256(currentBytes), certificate.currentReferencePacketSha256);
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
  assert.deepEqual(historicalPublicInstanceReference(expected).slice(0, 26), observed.packets);
});

test("unrelated code, coordinate, file and type-information changes are rejected", () => {
  const reference = historicalPublicInstanceReference(expected).slice(0, 26);
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
