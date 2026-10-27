import assert from "node:assert/strict";

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

export function publicInstanceReference(packets) {
  const out = structuredClone(packets);
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
