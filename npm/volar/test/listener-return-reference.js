import assert from "node:assert/strict";
import { createHash } from "node:crypto";

const historicalMessage =
  "Type '(value: number) => undefined' is not assignable to type '(value: string) => any'.";
const currentMessage =
  "Type '(value: number) => undefined' is not assignable to type '(value: string) => void'.";
const storedHistorical = "cc4105b72d75ee50cf42b7612551c0f7843539df51ae26b7cf7d3568dbb4dccf";
const storedCurrent = "4b1dddccbe26f62f74e0c8eb3aca9d572b2e3713b44ff5cac220bf7ce78065fc";
const wrappedHistorical = "f9c8ec10cb0a54edad788184496cefe6d55447afd86b55b585e2dbde113e107b";
const wrappedCurrent = "4c779bc23d2efabbeae04fe4d455ce17622376002c99b85165d180d13c191c96";

const digest = (packets) => createHash("sha256").update(JSON.stringify(packets)).digest("hex");

function replaceMessage(packets, before, after, beforeDigest, afterDigest) {
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
  return replaceMessage(
    packets,
    currentMessage,
    historicalMessage,
    storedCurrent,
    storedHistorical,
  );
}

/** Reapply the source-owned `void` return after the historical span transitions. */
export function listenerReturnReference(packets) {
  return replaceMessage(
    packets,
    historicalMessage,
    currentMessage,
    wrappedHistorical,
    wrappedCurrent,
  );
}
