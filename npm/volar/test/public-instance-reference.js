import assert from "node:assert/strict";
import { createHash } from "node:crypto";

const historicalListenerMessage =
  "Type '(value: number) => undefined' is not assignable to type '(value: string) => any'.";
const currentListenerMessage =
  "Type '(value: number) => undefined' is not assignable to type '(value: string) => void'.";
const storedHistorical = "cc4105b72d75ee50cf42b7612551c0f7843539df51ae26b7cf7d3568dbb4dccf";
const storedCurrent = "4b1dddccbe26f62f74e0c8eb3aca9d572b2e3713b44ff5cac220bf7ce78065fc";
const wrappedHistorical = "f9c8ec10cb0a54edad788184496cefe6d55447afd86b55b585e2dbde113e107b";
const wrappedCurrent = "4c779bc23d2efabbeae04fe4d455ce17622376002c99b85165d180d13c191c96";

const digest = (packets) => createHash("sha256").update(JSON.stringify(packets)).digest("hex");

function replaceListenerMessage(packets, before, after, beforeDigest, afterDigest) {
  assert.equal(digest(packets), beforeDigest, "closed listener return packet image");
  const out = structuredClone(packets);
  const matches = out.filter((row) => row.name === "feature-boundary-diagnostics");
  assert.equal(matches.length, 1, "one closed feature-boundary packet");
  const diagnostic = matches[0].result[0];
  assert.equal(diagnostic.code, 2322);
  assert.equal(diagnostic.messageText.messageText, before);
  diagnostic.messageText.messageText = after;
  assert.equal(digest(out), afterDigest, "closed listener return successor image");
  return out;
}

/** Restore the native packet image before the source-owned return parity successor. */
export function beforeListenerReturnReference(packets) {
  return replaceListenerMessage(
    packets,
    currentListenerMessage,
    historicalListenerMessage,
    storedCurrent,
    storedHistorical,
  );
}

/** Reapply the source-owned `void` return after the historical span transitions. */
export function listenerReturnReference(packets) {
  return replaceListenerMessage(
    packets,
    historicalListenerMessage,
    currentListenerMessage,
    wrappedHistorical,
    wrappedCurrent,
  );
}

// The archived packets remain byte-authenticated by their existing provenance.
// Advance only the nine reviewed missing-template-name diagnostics in #10933.
// Source ranges, SourceFile packets and every other field remain unchanged.
const changes = [
  ["basic-diagnostics", 1, "absent", 298],
  ["unsaved-update-diagnostics", 3, "absent", 299],
  ["delete-import-diagnostics", 4, "absent", 299],
  ["recreate-import-diagnostics", 3, "absent", 299],
  ["directive-diagnostics", 0, "realMissing", 197],
  ["directive-diagnostics", 1, "childMissing", 254],
  ["musea-variant-diagnostics", 0, "unknownStoryBinding", 250],
  ["musea-unsaved-update-diagnostics", 1, "unknownStoryBinding", 247],
  ["musea-delete-import-diagnostics", 2, "unknownStoryBinding", 247],
];

export function historicalPublicInstanceReference(packets) {
  const out = beforeListenerReturnReference(packets);
  for (const [packetName, index, name, start] of changes) {
    const packet = out.find((row) => row.name === packetName);
    assert.ok(packet, `missing archived packet: ${packetName}`);
    const diagnostic = packet.result[index];
    assert.equal(diagnostic.code, 2304);
    assert.equal(diagnostic.messageText, `Cannot find name '${name}'.`);
    assert.equal(diagnostic.start, start);
    assert.equal(diagnostic.length, name.length);
    assert.equal(diagnostic.file.text.slice(start, start + name.length), name);
    diagnostic.code = 2339;
    diagnostic.messageText = `Property '${name}' does not exist on type '__VizeTemplateInstance'.`;
  }
  return out;
}

// This source-owned display successor has no new native-capture credit. The
// original26/38 native packet archives retain their complete historical bytes.
export function publicInstanceReference(packets) {
  const out = historicalPublicInstanceReference(packets);
  for (const [packetName, index, name] of changes) {
    const diagnostic = out.find((row) => row.name === packetName).result[index];
    assert.equal(
      diagnostic.messageText,
      `Property '${name}' does not exist on type '__VizeTemplateInstance'.`,
    );
    diagnostic.messageText = `Property '${name}' does not exist on type '$TemplateInstance'.`;
  }
  return out;
}
