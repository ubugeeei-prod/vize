// Verify the two metadata fields expected to change between packaged Linux Node builds.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

interface Side {
  revision: string;
  pluginHostBuildId: string;
  members: { key: string; sha256: string; bytes: number }[];
}
const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");

function buildNote(file: string): Buffer | undefined {
  const notes = execFileSync("readelf", ["--notes", "--wide", file], {
    encoding: "utf8",
    env: { ...process.env, LC_ALL: "C" },
  });
  const ids = [...notes.matchAll(/Build ID: ([0-9a-f]+)/gu)];
  if (!ids.length) return undefined;
  assert.equal(ids.length, 1, "ambiguous GNU build-id notes");
  assert.equal(ids[0][1].length % 2, 0);
  return Buffer.from(ids[0][1], "hex");
}

// Replace only the exact producer-supplied member at its identical, unique offset.
export function matchIdentity(base: Buffer, candidate: Buffer, before: Buffer, after: Buffer) {
  assert.ok(before.length && before.length === after.length);
  const offset = base.indexOf(before);
  assert.ok(offset >= 0, "base identity missing");
  assert.equal(base.indexOf(before, offset + 1), -1, "ambiguous base identity");
  assert.equal(candidate.indexOf(after), offset, "candidate identity moved or missing");
  assert.equal(candidate.indexOf(after, offset + 1), -1, "ambiguous candidate identity");
  before.copy(candidate, offset);
  return { offset, bytes: before.length };
}

function main(file: string) {
  const report = JSON.parse(fs.readFileSync(file, "utf8"));
  assert.equal(report.package, "vize_vitrine");
  assert.equal(report.product, "node-addon");
  assert.deepEqual(report.differences, ["node-addon"]);
  const read = (side: Side, relative: string) => {
    assert.match(side.pluginHostBuildId, /^[a-f0-9]{40}:[a-f0-9]{64}$/u);
    assert.equal(side.pluginHostBuildId.slice(0, 40), side.revision);
    const name = path.resolve(path.dirname(file), relative);
    const image = fs.readFileSync(name);
    assert.equal(side.members.length, 1);
    assert.equal(side.members[0].key, "node-addon");
    assert.equal(image.length, side.members[0].bytes);
    assert.equal(sha256(image), side.members[0].sha256);
    return { image, note: buildNote(name), id: Buffer.from(side.pluginHostBuildId, "ascii") };
  };
  const base = read(report.base, report.executables.base);
  const head = read(report.head, report.executables.head);
  assert.equal(base.image.length, head.image.length);
  assert.notEqual(report.base.revision, report.head.revision);
  const host = matchIdentity(base.image, head.image, base.id, head.id);
  assert.equal(Boolean(base.note), Boolean(head.note));
  const note =
    base.note && head.note
      ? matchIdentity(base.image, head.image, base.note, head.note)
      : undefined;
  assert.ok(
    base.image.equals(head.image),
    "native addon contains differences outside its recorded plugin cache identity and GNU build ID",
  );
  console.log(
    JSON.stringify({
      bytes: base.image.length,
      host,
      note,
      base: report.base.members[0].sha256,
      head: report.head.members[0].sha256,
    }),
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  main(process.argv[2]);
