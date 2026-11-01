/** Compare only declared file identities across distinct copies of one authored corpus. */
import assert from "node:assert/strict";
import path from "node:path";

const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === "object" && !Array.isArray(value);

export function coldScalePacketView(
  packet: unknown,
  corpus: { root: string; app: string; entries: { path: string }[] },
) {
  assert.ok(record(packet) && Array.isArray(packet.files));
  assert.equal(path.resolve(corpus.root), corpus.root);
  assert.equal(corpus.app, path.join(corpus.root, "apps/web"));
  const names = new Set(corpus.entries.map((entry) => entry.path));
  assert.equal(names.size, corpus.entries.length, "duplicate authored corpus entry");
  for (const name of names) {
    assert.equal(path.isAbsolute(name), false);
    assert.equal(path.normalize(name), name);
    assert.ok(name !== ".." && !name.startsWith("../"));
  }
  assert.equal(packet.fileCount, names.size);
  assert.equal(packet.files.length, names.size, "complete reported corpus required");
  const seen = new Set<string>();
  const files = packet.files.map((file: unknown) => {
    assert.ok(record(file) && typeof file.file === "string");
    assert.equal(path.normalize(file.file), file.file, "noncanonical rendered filename");
    const absolute = path.resolve(corpus.app, file.file);
    assert.ok(absolute.startsWith(corpus.root + path.sep), "foreign rendered filename");
    const logical = path.relative(corpus.root, absolute);
    assert.ok(names.has(logical), "rendered filename missing from authored census");
    assert.equal(seen.has(logical), false, "duplicate rendered filename");
    seen.add(logical);
    return { ...file, file: logical };
  });
  // Diagnostics, programs, order and unknown fields remain exact values.
  return { ...packet, files };
}
