import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

const fixture = new URL("./reference-fixtures/listener-wrapper.vue.txt", import.meta.url);
const value = "(value: number) => undefined";
const digest = (input) => createHash("sha256").update(input).digest("hex");
const before = "f1b55943869cfcc03d92cfad08d93857fdf5738280832f32f00fd32aa95413ac";
const after = "f9c8ec10cb0a54edad788184496cefe6d55447afd86b55b585e2dbde113e107b";

// Preserve the archived38 packets. Only #11675's authored listener span moves;
// the37 observed source packets do not qualify the final, unexecuted rename.
export function listenerWrapperReference(packets, source = fs.readFileSync(fixture, "utf8")) {
  assert.equal(
    digest(source),
    "a6c3cee7449f0b4f97930c1382561b946814d9544c3bc326a4ec6d72c5518e6e",
    "closed authored listener fixture",
  );
  const start = source.indexOf(value);
  assert.equal(start, source.lastIndexOf(value), "one literal authored listener");
  assert.equal(start, 107);
  assert.equal(value.length, 28);
  const matches = packets.filter((row) => row.name === "feature-boundary-diagnostics");
  assert.equal(matches.length, 1, "one closed feature-boundary packet");
  const diagnostic = matches[0].result[0];
  assert.deepEqual(
    Object.keys(diagnostic).sort(),
    ["category", "code", "file", "length", "messageText", "relatedInformation", "start"],
    "closed listener diagnostic fields",
  );
  assert.equal(diagnostic.code, 2322);
  assert.equal(diagnostic.file.text, source, "closed diagnostic source");
  assert.equal(diagnostic.start, start - 2, "closed historical listener start");
  assert.equal(diagnostic.length, value.length + 2, "closed historical listener length");
  assert.equal(source.slice(diagnostic.start, diagnostic.start + diagnostic.length), `="${value}`);
  assert.equal(diagnostic.relatedInformation.length, 1, "one closed related diagnostic");
  const related = diagnostic.relatedInformation[0];
  assert.deepEqual(
    Object.keys(related).sort(),
    ["category", "code", "file", "length", "messageText", "start"],
    "closed related diagnostic fields",
  );
  assert.equal(related.file.text, source);
  assert.equal(related.code, 6212);
  assert.equal(related.start, start - 2, "closed historical related start");
  assert.equal(related.length, value.length + 2, "closed historical related length");
  assert.equal(digest(JSON.stringify(packets)), before, "closed historical packet image");
  const out = structuredClone(packets);
  const updated = out.find((row) => row.name === "feature-boundary-diagnostics").result[0];
  for (const target of [updated, updated.relatedInformation[0]]) {
    target.start = start;
    target.length = value.length;
  }
  assert.equal(digest(JSON.stringify(out)), after, "closed current packet image");
  return out;
}

export function assertListenerStarts(errors, eventParent) {
  // The wrong listener parameter and the wrong prop both survive (#11657).
  const anchors = [
    eventParent.indexOf("(value: number) => undefined"),
    eventParent.indexOf("foo-bar"),
  ];
  assert.deepEqual(
    errors.map((diagnostic) => [diagnostic.code, diagnostic.start]),
    anchors.map((start) => [2322, start]),
  );
}
