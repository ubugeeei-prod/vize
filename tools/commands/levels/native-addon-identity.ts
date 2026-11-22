// Verify ordinary Node artifacts and an independent candidate rebuild with the base cache identity.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { differences } from "./codegen-equivalence.ts";

interface Side {
  revision: string;
  pluginHostBuildId: string;
  members: { key: string; sha256: string; bytes: number }[];
}
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

export function verifyNativeAddons(file: string) {
  const report = JSON.parse(fs.readFileSync(file, "utf8"));
  assert.equal(report.package, "vize_vitrine");
  assert.equal(report.product, "node-addon");
  assert.deepEqual(report.differences, ["node-addon"]);
  assert.deepEqual(differences(report.base, report.head), ["node-addon"]);
  const read = (side: Side, relative: string) => {
    assert.match(side.pluginHostBuildId, /^[a-f0-9]{40}:[a-f0-9]{64}$/u);
    assert.equal(side.pluginHostBuildId.slice(0, 40), side.revision);
    const name = path.resolve(path.dirname(file), relative);
    const image = fs.readFileSync(name);
    assert.equal(side.members.length, 1);
    assert.equal(side.members[0].key, "node-addon");
    assert.equal(image.length, side.members[0].bytes);
    assert.equal(sha256(image), side.members[0].sha256);
    assert.ok(
      image.includes(Buffer.from(side.pluginHostBuildId, "ascii")),
      "missing native cache identity",
    );
    return name;
  };
  const base = read(report.base, report.executables.base);
  const head = read(report.head, report.executables.head);
  assert.notEqual(report.base.revision, report.head.revision);
  const control = report.pluginIdentityControl;
  assert.equal(control.evidence.revision, report.head.revision);
  assert.equal(control.evidence.pluginHostBuildId, report.head.pluginHostBuildId);
  assert.equal(control.produced, report.head.pluginHostBuildId);
  assert.equal(control.compiled, report.base.pluginHostBuildId);
  assert.deepEqual(control.differences, []);
  assert.deepEqual(
    differences(report.base, control.evidence),
    [],
    "cache-controlled native addon differs from the base",
  );
  return {
    base,
    head,
    raw: [report.base.members[0], report.head.members[0]],
    controlled: control.evidence.members[0],
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  console.log(JSON.stringify(verifyNativeAddons(process.argv[2])));
