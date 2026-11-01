/** Whole historical source inputs for pure tests, independent of fetched Git refs. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

export const frozenColdSourcePins = {
  base: "e57fd021bf08b197956807eb122941c9cd13b056",
  head: "d440bf04becabbb71d916915c640d39c8d8c6450",
  workflow: "a193c99c824b54fbe7e900bb88a19f549cd25ea7",
} as const;
type Role = keyof typeof frozenColdSourcePins;
type Digest = { bytes: number; sha256: string };
type Source = Digest & { fixture: string; gitBlob: string };
type Manifest = {
  version: number;
  roles: Record<Role, { commit: string; files: Record<string, Source> }>;
  overlays: {
    role: "base" | "head";
    productSource: string;
    files: (Digest & { file: string })[];
  }[];
};
const directory = new URL(
  "../../_fixtures/tooling/cold-typechecker-frozen-source/",
  import.meta.url,
);
const manifestBytes = fs.readFileSync(new URL("manifest.json", directory));
assert.equal(
  createHash("sha256").update(manifestBytes).digest("hex"),
  "56a9643ba7086bfbfd23bd0512b4b2cc8eed394a353cd7fc1e5fe6eea5d4d014",
  "whole frozen-source manifest",
);
const manifest = JSON.parse(manifestBytes.toString()) as Manifest;
assert.equal(manifest.version, 1);
assert.deepEqual(Object.keys(manifest.roles).sort(), Object.keys(frozenColdSourcePins).sort());
for (const role of Object.keys(frozenColdSourcePins) as Role[])
  assert.equal(manifest.roles[role].commit, frozenColdSourcePins[role]);

function verify(body: Buffer, expected: Digest) {
  assert.equal(body.length, expected.bytes, "whole frozen source byte length");
  assert.equal(createHash("sha256").update(body).digest("hex"), expected.sha256);
}

export function verifyFrozenColdSource(role: Role, file: string, body: Buffer) {
  const expected = manifest.roles[role].files[file];
  assert(expected, "only the closed historical source inventory");
  verify(body, expected);
}

export function frozenColdSource(role: Role, file: string): Buffer {
  const expected = manifest.roles[role].files[file];
  assert(expected, "only the closed historical source inventory");
  assert.match(expected.fixture, /^(base|head|workflow)\/[A-Za-z0-9_./-]+\.txt$/u);
  assert(!expected.fixture.split("/").includes(".."));
  const body = fs.readFileSync(new URL(expected.fixture, directory));
  verifyFrozenColdSource(role, file, body);
  return body;
}

export function verifyFrozenReadOverlay(
  role: "base" | "head",
  bodies: Record<string, string | Buffer>,
) {
  const matches = manifest.overlays.filter((row) => row.role === role);
  assert.equal(matches.length, 1);
  const expected = matches[0];
  assert.equal(expected.productSource, frozenColdSourcePins[role]);
  assert.deepEqual(Object.keys(bodies).sort(), expected.files.map((row) => row.file).sort());
  for (const row of expected.files) verify(Buffer.from(bodies[row.file]), row);
}
