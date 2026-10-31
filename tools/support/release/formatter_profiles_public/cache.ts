import assert from "node:assert/strict";
import { lstatSync, readFileSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { asArray, asRecord, asString } from "../../json-value.ts";
import { type Recorder, sha256 } from "../../config/windows_consumer/common.ts";
import type { Entry } from "../editor_vite_stub/filesystem.ts";

export const cacheWriterBlob = "b1893cdb920ff48ea060e1c32b31342e76beb159";
const cacheRawByteLimit = 4 * 1024 ** 2;
const cacheFile = /^\.vize\/config-cache\/[0-9a-f]{16}\.json$/u;
const exact = (value: unknown, keys: string[]) => {
  const row = asRecord(value);
  assert.deepEqual(Object.keys(row).sort(), [...keys].sort());
  return row;
};
export type CacheKey = { explicit: boolean; runtime: string; config: string };

/** R18's exact format1 writer; raw bytes and u64 tokens remain unmodified. */
export function validateCache(raw: Buffer, root: string, version: string): CacheKey {
  const numbers = new WeakMap<object, Map<string, string>>();
  const parsed: unknown = JSON.parse(
    new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(raw),
    function (this: unknown, key: string, value: unknown, context?: { source?: string }) {
      if (
        this !== null &&
        typeof this === "object" &&
        typeof value === "number" &&
        context?.source !== undefined
      ) {
        let fields = numbers.get(this);
        if (!fields) numbers.set(this, (fields = new Map()));
        fields.set(key, context.source);
      }
      return value;
    },
  );
  const u64 = (parent: Record<string, unknown>, key: string, max = 18446744073709551615n) => {
    const token = numbers.get(parent)?.get(key);
    assert(token !== undefined);
    assert.match(token, /^(0|[1-9][0-9]{0,19})$/u);
    assert(BigInt(token) <= max);
    return token;
  };
  const entries = asArray(parsed);
  assert(entries.length >= 1 && entries.length <= 4);
  let identity: CacheKey | undefined;
  for (const value of entries) {
    const entry = exact(value, ["key", "env", "env_names", "inputs", "evaluation"]);
    const key = exact(entry.key, ["format", "version", "runtime", "config", "explicit"]);
    assert.equal(key.format, 1);
    assert.equal(key.version, version);
    assert.equal(key.config, join(root, "vite.config.mjs"));
    assert.equal(typeof key.explicit, "boolean");
    const current = {
      explicit: key.explicit as boolean,
      runtime: u64(key, "runtime"),
      config: asString(key.config),
    };
    if (identity) assert.deepEqual(current, identity, "one cache filename owns one exact key");
    else identity = current;
    for (const value of asArray(entry.env)) {
      const pair = asArray(value);
      assert.equal(pair.length, 2);
      asString(pair[0]);
      const read = asRecord(pair[1]);
      if (Object.hasOwn(read, "Value")) {
        exact(read, ["Value"]);
        if (read.Value !== null) assert.match(asString(read.Value), /^[0-9a-f]{64}$/u);
      } else {
        exact(read, ["Presence"]);
        assert.equal(typeof read.Presence, "boolean");
      }
    }
    if (entry.env_names !== null) assert.match(asString(entry.env_names), /^[0-9a-f]{64}$/u);
    const inputs = asArray(entry.inputs);
    assert(inputs.some((value) => asArray(value)[0] === current.config));
    for (const value of inputs) {
      const input = asArray(value);
      assert.equal(input.length, 3);
      asString(input[0]);
      assert.equal(typeof input[1], "boolean");
      const stamp = exact(input[2], ["kind", "len", "modified", "changed", "inode", "link"]);
      u64(stamp, "kind", 3n);
      for (const field of ["len", "modified", "changed", "inode"]) u64(stamp, field);
      assert(stamp.link === null || typeof stamp.link === "string");
    }
    const evaluation = exact(entry.evaluation, [
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
  assert(identity);
  return identity;
}

type CachedFile = { rawBase64: string; bytes: number; sha256: string; key?: CacheKey };
export type CacheSnapshot = { entries: Entry[]; files: Record<string, CachedFile> };
export function captureCache(
  root: string,
  version: string,
  entries: Entry[],
  record: Recorder,
  label: string,
): CacheSnapshot {
  const files: Record<string, CachedFile> = {};
  const observed = { entries, files };
  record.json(label + ".cache-raw.json", observed);
  for (const row of entries) {
    if (row.path === ".vize" || row.path === ".vize/config-cache") {
      assert.equal(row.kind, "directory");
      assert.equal(
        realpathSync(join(root, "node_modules", row.path)),
        join(root, "node_modules", row.path),
      );
    }
    if (!cacheFile.test(row.path)) continue;
    assert.equal(row.kind, "file");
    const file = join(root, "node_modules", row.path);
    assert(lstatSync(file).isFile() && !lstatSync(file).isSymbolicLink());
    assert.equal(realpathSync(file), file);
    if ((row.bytes ?? Infinity) > cacheRawByteLimit) {
      record.json(label + ".cache-raw.json", {
        ...observed,
        omitted: {
          row,
          reason:
            "cache raw evidence exceeds four MiB; whole row/hash retained; cache transition unqualified",
        },
      });
      assert.fail("cache raw evidence exceeds its closed proof budget");
    }
    const raw = readFileSync(file);
    assert(raw.length <= cacheRawByteLimit);
    files[row.path] = { rawBase64: raw.toString("base64"), bytes: raw.length, sha256: sha256(raw) };
    record.json(label + ".cache-raw.json", observed);
    assert.equal(raw.length, row.bytes);
    assert.equal(sha256(raw), row.sha256);
    files[row.path]!.key = validateCache(raw, root, version);
    record.json(label + ".cache-raw.json", observed);
  }
  assert(
    Object.keys(files).length <= 2,
    "one authored MJS config permits only its explicit and discovered slots",
  );
  const explicit = Object.values(files).map((file) => file.key?.explicit);
  assert.equal(
    new Set(explicit).size,
    explicit.length,
    "no second valid-looking filename for the same explicit slot",
  );
  return observed;
}

/** Every module row is exact except a validated source-owned cache transition. */
export function auditCache(
  before: CacheSnapshot,
  after: CacheSnapshot,
  explicit: boolean | null,
  record: Recorder,
  label: string,
): void {
  const old = new Map(before.entries.map((row) => [row.path, row]));
  const current = new Map(after.entries.map((row) => [row.path, row]));
  assert.equal(old.size, before.entries.length);
  assert.equal(current.size, after.entries.length);
  const transitions: unknown[] = [];
  for (const row of before.entries) {
    const next = current.get(row.path);
    assert(next, "CLI removed an installation entry: " + row.path);
    if (cacheFile.test(row.path) && row.sha256 !== next.sha256) {
      assert.equal(next.kind, "file");
      assert.equal(next.mode, row.mode);
      assert(before.files[row.path]?.key && after.files[row.path]?.key);
      assert.deepEqual(after.files[row.path]!.key, before.files[row.path]!.key);
      assert.equal(after.files[row.path]!.key!.explicit, explicit);
      transitions.push({ before: row, after: next });
    } else assert.deepEqual(next, row, "CLI changed a non-cache installation entry: " + row.path);
  }
  const additions = after.entries.filter((row) => !old.has(row.path));
  for (const row of additions) {
    assert(explicit !== null, "this argv route cannot evaluate a JS config cache");
    if (row.path === ".vize" || row.path === ".vize/config-cache")
      assert.equal(row.kind, "directory");
    else {
      assert.match(row.path, cacheFile);
      assert.equal(row.kind, "file");
      assert.equal(after.files[row.path]?.key?.explicit, explicit);
    }
  }
  record.json(label + ".cache-actor.json", {
    cacheWriterBlob,
    before,
    after,
    explicit,
    additions,
    transitions,
    assertion:
      "all module rows enforced; only exact owned format1 cache additions/updates, no deletion, ignored path, link or unrelated dependency mutation",
  });
}
