import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { sourceProvenance, validateProductPacket } from "./sfc-parse-attributes-source.mjs";
import {
  attributesPacketReference,
  exposureReferenceCustody,
} from "./sfc-parse-attributes-reference.ts";

const directory = path.resolve(process.argv[2]);
const base = JSON.parse(await fs.readFile(path.join(directory, "base/packets.json"), "utf8"));
const head = JSON.parse(await fs.readFile(path.join(directory, "head/packets.json"), "utf8"));
assert.equal(base.source, "49a98d1e72707083354b8ff11e1cec800ded28b3");
const observerActor = execFileSync(
  "git",
  ["-C", fileURLToPath(new URL("../../", import.meta.url)), "rev-parse", "HEAD"],
  { encoding: "utf8" },
).trim();
const provenance = sourceProvenance(observerActor);
assert.equal(head.source, process.env.VIZE_SFC_ATTRS_ACTOR_SHA);
for (const { role, packet } of [
  { role: "base", packet: base },
  { role: "head", packet: head },
]) {
  const image = JSON.parse(await fs.readFile(path.join(directory, role, "image.json"), "utf8"));
  validateProductPacket(packet, image, role, observerActor);
}
const fixtures = JSON.parse(
  await fs.readFile(
    new URL("../fixtures/sfc/parsed-attributes/cases.json", import.meta.url),
    "utf8",
  ),
);
function original(packet) {
  return packet.kind === "return"
    ? packet
    : { kind: packet.kind, code: packet.code, message: packet.message };
}
function blocks(descriptor) {
  return [
    descriptor.template,
    descriptor.script,
    descriptor.scriptSetup,
    ...descriptor.styles,
    ...descriptor.customBlocks,
  ].filter(Boolean);
}
for (const fixture of fixtures) {
  const before = base.packets.find(({ id }) => id === fixture.id);
  const after = head.packets.find(({ id }) => id === fixture.id);
  for (const mode of ["omitted", "filename", "disabled", "compile"])
    assert.deepEqual(
      original(after[mode]),
      original(attributesPacketReference(before[mode], fixture.source, mode)),
      `${fixture.id}/${mode}: complete existing API packet stays identical`,
    );
  if (fixture.error) {
    assert.equal(after.enabled.kind, "throw");
    assert.deepEqual(original(after.enabled), original(after.filename));
    continue;
  }
  assert.equal(after.enabled.kind, "return", `${fixture.id}: ${JSON.stringify(after.enabled)}`);
  const enabled = JSON.parse(after.enabled.json);
  const disabled = JSON.parse(after.disabled.json);
  for (const block of blocks(disabled)) assert.equal(Object.hasOwn(block, "attrs"), false);
  for (const block of blocks(enabled)) assert.ok(Array.isArray(block.attrs));
  for (const [key, expected] of Object.entries(fixture.attrs)) {
    const actual = ["styles", "customBlocks"].includes(key)
      ? enabled[key].map((block) => block.attrs.map(({ name, value }) => [name, value]))
      : enabled[key].attrs.map(({ name, value }) => [name, value]);
    assert.deepEqual(
      actual,
      expected,
      `${fixture.id}/${key}: real parser order, names and effective values`,
    );
  }
  const marker = [enabled.script, enabled.scriptSetup].some((script) =>
    script?.attrs.some(({ name }) => name === "vapor"),
  );
  assert.equal(
    marker,
    fixture.vapor ?? false,
    `${fixture.id}: the real compiler's exact attribute-presence predicate`,
  );
  for (const block of blocks(enabled)) delete block.attrs;
  assert.equal(
    JSON.stringify(enabled),
    after.disabled.json,
    `${fixture.id}: opt-in adds only attributes`,
  );
}
await fs.writeFile(
  path.join(directory, "qualification.json"),
  JSON.stringify(
    {
      complete: true,
      base: base.source,
      head: head.source,
      provenance,
      cases: fixtures.length,
      defaultPackets: fixtures.length * 4,
      originalErrorStacksRetained: true,
      historicalExposureReference: exposureReferenceCustody,
    },
    null,
    2,
  ) + "\n",
);
