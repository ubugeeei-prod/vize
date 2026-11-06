import assert from "node:assert/strict";
import { asArray, asRecord } from "../../json-value.ts";

/** Admit the historical or already-migrated packet and return the current path. */
export function currentNoDupeKeysPacket(historical: unknown): unknown[] {
  const packet = asArray(structuredClone(historical), "historical lint packet");
  assert.equal(packet.length, 1);
  const messages = asArray(asRecord(packet[0]).messages, "historical lint messages");
  assert.equal(messages.length, 1);
  const diagnostic = asRecord(messages[0]);
  assert.equal(diagnostic.ruleId, "script/no-dupe-keys");
  const previous = "docs/content/rules/type-and-script.md";
  const current = "docs/content/rules/reference/script-no-dupe-keys.md";
  assert.ok(diagnostic.ruleDocsPath === previous || diagnostic.ruleDocsPath === current);
  diagnostic.ruleDocsPath = current;
  return packet;
}
