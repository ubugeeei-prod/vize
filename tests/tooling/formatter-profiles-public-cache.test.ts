import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import {
  auditCache,
  cacheWriterBlob,
  captureCache,
  validateCache,
} from "../../tools/support/release/formatter_profiles_public/cache.ts";
import {
  originalBytes,
  originalContract,
} from "../../tools/support/release/formatter_profiles_public/authority.ts";
import { Recorder } from "../../tools/support/config/windows_consumer/common.ts";
import { inventory } from "../../tools/support/release/editor_vite_stub/filesystem.ts";

// Artificial format1 validator controls, never claimed as a product capture.
function cacheBytes(root: string, explicit = false, integer = "1"): Buffer {
  const file = join(root, "vite.config.mjs");
  return Buffer.from(
    JSON.stringify([
      {
        key: { format: 1, version: "0.626.0", runtime: "__NATIVE_U64__", config: file, explicit },
        env: [["PATH", { Value: "a".repeat(64) }]],
        env_names: null,
        inputs: [
          [
            file,
            false,
            { kind: 1, len: 7, modified: "__NATIVE_U64__", changed: 0, inode: 1, link: null },
          ],
        ],
        evaluation: {
          config: { formatter: { semi: false } },
          complete: true,
          watchDependencies: [],
          dependencies: [],
        },
      },
    ]).replaceAll('"__NATIVE_U64__"', integer),
  );
}

test("the exact R18 cache writer and raw u64/object boundaries stay source bound", () => {
  const original = originalContract();
  assert.equal(
    original.references.find(
      (row) => row.path === "crates/vize_carton/src/config/loader/js_cache.rs",
    )?.gitBlob,
    cacheWriterBlob,
  );
  const writer = originalBytes(
    original,
    "crates/vize_carton/src/config/loader/js_cache.rs",
  ).toString("utf8");
  assert(writer.includes("const FORMAT: u32 = 1;"));
  assert(writer.includes("const ENTRIES: usize = 4;"));
  assert(writer.includes("(&key.config, explicit).hash(&mut name)"));
  const root = "/owned-original",
    max = "18446744073709551615";
  for (const explicit of [false, true]) {
    const raw = cacheBytes(root, explicit, max),
      before = Buffer.from(raw);
    assert.deepEqual(validateCache(raw, root, "0.626.0"), {
      explicit,
      runtime: max,
      config: join(root, "vite.config.mjs"),
    });
    assert.deepEqual(raw, before);
  }
  for (const integer of ["18446744073709551616", "1e100", "-1", "1.5"]) {
    assert.throws(() => validateCache(cacheBytes(root, false, integer), root, "0.626.0"), integer);
  }
  assert.throws(() => validateCache(cacheBytes(root), "/foreign", "0.626.0"));
  assert.throws(() => validateCache(cacheBytes(root), root, "0.627.0"));
  assert.throws(() =>
    validateCache(
      Buffer.concat([Buffer.from([0xef, 0xbb, 0xbf]), cacheBytes(root)]),
      root,
      "0.626.0",
    ),
  );
  const raw = JSON.parse(cacheBytes(root).toString("utf8")) as Record<string, unknown>[];
  raw[0]!.unknown = true;
  assert.throws(() => validateCache(Buffer.from(JSON.stringify(raw)), root, "0.626.0"));
});

test("only the two owned explicit/discovered slots can change, while unrelated dependencies stay exact", () => {
  const directory = realpathSync(mkdtempSync(join(tmpdir(), "vize-formatter-cache-")));
  try {
    const root = join(directory, "consumer"),
      modules = join(root, "node_modules"),
      cache = join(modules, ".vize/config-cache");
    mkdirSync(join(modules, "unrelated"), { recursive: true });
    writeFileSync(join(modules, "unrelated/entry.js"), "before");
    const record = new Recorder(join(directory, "proof"));
    const before = captureCache(root, "0.626.0", inventory(modules), record, "before");
    mkdirSync(cache, { recursive: true });
    writeFileSync(join(cache, "0000000000000001.json"), cacheBytes(root));
    const discovered = captureCache(root, "0.626.0", inventory(modules), record, "discovered");
    auditCache(before, discovered, false, record, "allowed-discovered");
    assert.throws(() => auditCache(before, discovered, null, record, "no-config-route"));
    assert.throws(() => auditCache(before, discovered, true, record, "wrong-explicit-route"));
    writeFileSync(join(cache, "0000000000000002.json"), cacheBytes(root, true));
    const both = captureCache(root, "0.626.0", inventory(modules), record, "both");
    auditCache(discovered, both, true, record, "allowed-explicit");
    writeFileSync(join(modules, "unrelated/entry.js"), "after!");
    const changed = captureCache(root, "0.626.0", inventory(modules), record, "changed");
    assert.throws(() => auditCache(both, changed, false, record, "foreign-dependency-write"));
    writeFileSync(join(modules, "foreign.json"), "{}");
    const added = captureCache(root, "0.626.0", inventory(modules), record, "added");
    assert.throws(() => auditCache(changed, added, false, record, "foreign-addition"));
    assert.throws(() =>
      captureCache(
        root,
        "0.626.0",
        inventory(modules).map((row) =>
          row.path === ".vize" ? { ...row, kind: "link", target: "foreign" } : row,
        ),
        record,
        "foreign-cache-owner",
      ),
    );
    rmSync(join(cache, "0000000000000002.json"));
    writeFileSync(join(cache, "0000000000000002.json"), cacheBytes(root));
    assert.throws(() =>
      captureCache(root, "0.626.0", inventory(modules), record, "duplicate-valid-slot"),
    );
    const retained: unknown = JSON.parse(
      readFileSync(join(record.root, "duplicate-valid-slot.cache-raw.json"), "utf8"),
    );
    assert(retained && typeof retained === "object" && "files" in retained);
    assert.equal(Object.keys(retained.files as object).length, 2);
  } finally {
    rmSync(directory, { recursive: true });
  }
});

test("malformed cache bytes are retained before refusal and no removed module row is accepted", () => {
  const directory = realpathSync(mkdtempSync(join(tmpdir(), "vize-formatter-cache-refusal-")));
  try {
    const root = join(directory, "consumer"),
      modules = join(root, "node_modules"),
      cache = join(modules, ".vize/config-cache");
    mkdirSync(cache, { recursive: true });
    const record = new Recorder(join(directory, "proof"));
    writeFileSync(join(cache, "0000000000000001.json"), "{broken");
    assert.throws(() => captureCache(root, "0.626.0", inventory(modules), record, "malformed"));
    assert(
      readFileSync(join(record.root, "malformed.cache-raw.json"), "utf8").includes(
        Buffer.from("{broken").toString("base64"),
      ),
    );
    writeFileSync(join(cache, "0000000000000001.json"), cacheBytes(root));
    const before = captureCache(root, "0.626.0", inventory(modules), record, "valid");
    rmSync(join(cache, "0000000000000001.json"));
    const after = captureCache(root, "0.626.0", inventory(modules), record, "removed");
    assert.throws(() => auditCache(before, after, false, record, "removed-cache-file"));
  } finally {
    rmSync(directory, { recursive: true });
  }
});
