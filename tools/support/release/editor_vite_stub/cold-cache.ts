import assert from "node:assert/strict";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { asArray, asNumber, asRecord, asString, parseJson } from "../../json-value.ts";
import { Recorder } from "../../config/windows_consumer/common.ts";
import { digest, inside, inventory } from "./filesystem.ts";

function exactKeys(value: unknown, keys: string[]) {
  const record = asRecord(value);
  assert.deepEqual(Object.keys(record).sort(), [...keys].sort());
  return record;
}
export function validateCacheBytes(raw: Buffer, root: string, version: string): void {
  const entries = asArray(parseJson(raw.toString("utf8")));
  assert.ok(
    entries.length >= 1 && entries.length <= 4,
    "C2 format1 stores one to four environment entries",
  );
  for (const value of entries) {
    const entry = exactKeys(value, ["key", "env", "env_names", "inputs", "evaluation"]);
    const key = exactKeys(entry.key, ["format", "version", "runtime", "config", "explicit"]);
    assert.equal(key.format, 1);
    assert.equal(key.version, version);
    assert.equal(key.config, join(root, "vite.config.ts"));
    assert.equal(key.explicit, false);
    assert.ok(Number.isInteger(asNumber(key.runtime)) && asNumber(key.runtime) >= 0);
    for (const value of asArray(entry.env)) {
      const row = asArray(value);
      assert.equal(row.length, 2);
      asString(row[0]);
      const read = asRecord(row[1]);
      assert.equal(Object.keys(read).length, 1);
      if (Object.hasOwn(read, "Value")) {
        if (read.Value !== null) assert.match(asString(read.Value), /^[0-9a-f]{64}$/);
      } else {
        exactKeys(read, ["Presence"]);
        assert.equal(typeof read.Presence, "boolean");
      }
    }
    if (entry.env_names !== null) assert.match(asString(entry.env_names), /^[0-9a-f]{64}$/);
    const inputs = asArray(entry.inputs);
    assert.ok(
      inputs.some((value) => asArray(value)[0] === key.config),
      "cache must observe the actual authored config",
    );
    for (const value of inputs) {
      const row = asArray(value);
      assert.equal(row.length, 3);
      asString(row[0]);
      assert.equal(typeof row[1], "boolean");
      const stamp = exactKeys(row[2], ["kind", "len", "modified", "changed", "inode", "link"]);
      // Native u64 timestamps/inodes can exceed JS's safe-integer range. Raw bytes
      // retain numeric custody; these checks validate shape without reserializing it.
      for (const field of ["kind", "len", "modified", "changed", "inode"])
        assert.ok(Number.isInteger(asNumber(stamp[field])) && asNumber(stamp[field]) >= 0);
      assert.ok(asNumber(stamp.kind) <= 3);
      assert.ok(stamp.link === null || typeof stamp.link === "string");
    }
    const evaluation = exactKeys(entry.evaluation, [
      "config",
      "complete",
      "watchDependencies",
      "dependencies",
    ]);
    asRecord(evaluation.config);
    assert.equal(typeof evaluation.complete, "boolean");
    for (const value of [
      ...asArray(evaluation.watchDependencies),
      ...asArray(evaluation.dependencies),
    ])
      asString(value);
  }
}
const cacheFile = /^\.vize\/config-cache\/[0-9a-f]{16}\.json$/;
export function cacheSnapshot(
  root: string,
  version: string,
  record?: Recorder,
  label = "snapshot",
) {
  const entries = inventory(join(root, "node_modules"));
  record?.json(label + ".inventory.json", entries);
  const files: Record<string, { rawBase64: string; bytes: number; sha256: string }> = {};
  for (const row of entries.filter(
    (row) => row.path === ".vize" || row.path === ".vize/config-cache",
  )) {
    assert.equal(row.kind, "directory", "cache ownership must never follow a link");
    const directory = join(root, "node_modules", row.path);
    assert.equal(realpathSync(directory), directory);
  }
  for (const row of entries.filter((row) => cacheFile.test(row.path))) {
    assert.equal(row.kind, "file");
    const file = join(root, "node_modules", row.path),
      owner = join(root, "node_modules/.vize/config-cache");
    assert.equal(realpathSync(owner), owner);
    assert.ok(inside(owner, realpathSync(file)) && lstatSync(file).isFile());
    const raw = readFileSync(file),
      observed = digest(file);
    assert.equal(observed.bytes, row.bytes);
    assert.equal(observed.sha256, row.sha256);
    files[row.path] = { rawBase64: raw.toString("base64"), ...observed };
    record?.json(label + ".cache-bytes.json", { entries, files });
    validateCacheBytes(raw, root, version);
  }
  assert.ok(
    Object.keys(files).length <= 1,
    "one root config with explicit=false declares exactly one cache filename",
  );
  return { entries, files };
}
/** Each CLI actor accounts for its declared cache transition; all package/Vite payloads are exact. */
export function auditConfigCache(
  root: string,
  before: ReturnType<typeof cacheSnapshot>,
  after: ReturnType<typeof cacheSnapshot>,
  record: Recorder,
  label: string,
): void {
  const previous = new Map(before.entries.map((row) => [row.path, row]));
  const current = new Map(after.entries.map((row) => [row.path, row]));
  assert.equal(previous.size, before.entries.length);
  assert.equal(current.size, after.entries.length);
  const transitions: unknown[] = [];
  for (const row of before.entries) {
    const next = current.get(row.path);
    assert.ok(next, "CLI removed an existing installation entry: " + row.path);
    if (cacheFile.test(row.path)) {
      assert.equal(next.kind, "file");
      assert.equal(next.mode, row.mode);
      assert.ok(before.files[row.path] && after.files[row.path]);
      if (row.sha256 !== next.sha256) transitions.push({ before: row, after: next });
    } else assert.deepEqual(next, row, "CLI changed an existing package/Vite entry: " + row.path);
  }
  const additions = after.entries.filter((row) => !previous.has(row.path));
  for (const row of additions) {
    if (row.path === ".vize" || row.path === ".vize/config-cache") {
      assert.equal(row.kind, "directory");
      const directory = join(root, "node_modules", row.path);
      assert.equal(realpathSync(directory), directory);
      continue;
    }
    assert.match(row.path, cacheFile);
    assert.equal(row.kind, "file");
    assert.ok(after.files[row.path]);
  }
  record.json(label + ".cache-actor.json", {
    producerSourceBlob: "b1893cdb920ff48ea060e1c32b31342e76beb159",
    root,
    before,
    after,
    additions,
    transitions,
    assertion:
      "every package/Vite byte/link exact; each source-owned cache creation/update has full validated pre/post bytes; no deletion or ignored row",
  });
}
