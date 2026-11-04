import assert from "node:assert/strict";
import { asArray, asRecord } from "../../json-value.ts";

/** Retain the historical packet while admitting the rule's explicit docs move. */
export function currentNoDupeKeysPacket(historical: unknown): unknown[] {
  const packet = asArray(structuredClone(historical), "historical lint packet");
  assert.equal(packet.length, 1);
  const messages = asArray(asRecord(packet[0]).messages, "historical lint messages");
  assert.equal(messages.length, 1);
  const diagnostic = asRecord(messages[0]);
  assert.equal(diagnostic.ruleId, "script/no-dupe-keys");
  assert.equal(diagnostic.ruleDocsPath, "docs/content/rules/type-and-script.md");
  diagnostic.ruleDocsPath = "docs/content/rules/reference/script-no-dupe-keys.md";
  return packet;
}
