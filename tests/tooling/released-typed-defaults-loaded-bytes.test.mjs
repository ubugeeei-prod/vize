import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { sha256 } from "../../tools/support/release/typed_defaults/custody.mjs";
import { assertLoadedBinary } from "../../tools/support/release/typed_defaults/distribution.mjs";

await test("the complete loaded CLI bytes are revalidated when every filesystem identity field is preserved", (t) => {
  const consumer = fs.mkdtempSync(path.join(os.tmpdir(), "vize-loaded-cli-bytes-"));
  t.after(() => fs.rmSync(consumer, { recursive: true, force: true }));
  fs.mkdirSync(path.join(consumer, "distribution"));
  const binary = path.join(consumer, "distribution", "vize");
  // An inert ELF-shaped file is never executed. Bytes beyond its header still
  // belong to the archive's complete identity and cannot be silently changed.
  const original = Buffer.alloc(128);
  original.set([0x7f, 0x45, 0x4c, 0x46, 2, 1, 1]);
  original.writeUInt16LE(62, 18);
  fs.writeFileSync(binary, original, { mode: 0o755 });
  const realStat = fs.lstatSync;
  const preserved = Object.freeze(realStat(binary, { bigint: true }));
  const identity = [
    preserved.dev,
    preserved.ino,
    preserved.size,
    preserved.mode,
    preserved.mtimeNs,
    preserved.ctimeNs,
  ];
  t.mock.method(fs, "lstatSync", (filename, options) =>
    filename === binary ? preserved : realStat(filename, options),
  );
  const member = { binary, bytes: original.length, mode: 0o755, sha256: sha256(original) };
  assertLoadedBinary(binary, member, consumer);
  assertLoadedBinary(binary, member, consumer);
  for (const offset of [80, original.length - 1]) {
    const changed = Buffer.from(original);
    changed[offset] ^= 1;
    fs.writeFileSync(binary, changed);
    const current = fs.lstatSync(binary, { bigint: true });
    assert.deepEqual(
      [current.dev, current.ino, current.size, current.mode, current.mtimeNs, current.ctimeNs],
      identity,
    );
    assert.deepEqual(fs.readFileSync(binary), changed);
    assert.equal(changed.length, original.length);
    assert.notEqual(sha256(changed), member.sha256);
    assert.throws(() => assertLoadedBinary(binary, member, consumer), /identity changed/);
    fs.writeFileSync(binary, original);
    assertLoadedBinary(binary, member, consumer);
  }
});
