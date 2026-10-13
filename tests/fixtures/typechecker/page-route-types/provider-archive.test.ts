// Offline transport controls; these do not execute or fetch a provider.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { gzipSync } from "node:zlib";

import {
  type ProviderFile,
  ProviderRefusal,
  members,
  stage,
  validatePackage,
} from "./provider-archive.ts";

const REGULAR = "0";
const HARD_LINK = "1";
const SYMBOLIC_LINK = "2";
const CHARACTER_DEVICE = "3";
const PAX_HEADER = "x";

interface Entry {
  readonly name: string;
  readonly body: Buffer;
  readonly type: string;
  /** Store the name in a pax extended header and leave an ASCII stand-in. */
  readonly pax?: boolean;
}

function header(name: string, size: number, type: string): Buffer {
  const block = Buffer.alloc(512);
  const nameBytes = Buffer.from(name, "utf8");
  assert.ok(nameBytes.length <= 100, "control names fit the classic name field");
  nameBytes.copy(block, 0);
  block.write("0000644\0", 100, "ascii");
  block.write("0000000\0", 108, "ascii");
  block.write("0000000\0", 116, "ascii");
  block.write(`${size.toString(8).padStart(11, "0")}\0`, 124, "ascii");
  block.write("00000000000\0", 136, "ascii");
  block.write("        ", 148, "ascii");
  block.write(type, 156, "ascii");
  if (type === SYMBOLIC_LINK || type === HARD_LINK) block.write("outside", 157, "ascii");
  block.write("ustar\0", 257, "ascii");
  block.write("00", 263, "ascii");
  let sum = 0;
  for (const byte of block) sum += byte;
  block.write(`${sum.toString(8).padStart(6, "0")}\0 `, 148, "ascii");
  return block;
}

function padded(body: Buffer): Buffer {
  return Buffer.concat([body, Buffer.alloc((512 - (body.length % 512)) % 512)]);
}

function paxRecord(key: string, value: string): Buffer {
  const payload = Buffer.from(` ${key}=${value}\n`, "utf8");
  let length = payload.length + 1;
  while (String(length).length + payload.length !== length) {
    length = String(length).length + payload.length;
  }
  return Buffer.concat([Buffer.from(String(length), "ascii"), payload]);
}

function archive(entries: readonly Entry[]): Buffer {
  const blocks: Buffer[] = [];
  for (const entry of entries) {
    if (entry.pax === true) {
      const record = paxRecord("path", entry.name);
      blocks.push(header("PaxHeaders/member", record.length, PAX_HEADER), padded(record));
      blocks.push(header("package/stand-in", entry.body.length, entry.type));
    } else {
      blocks.push(header(entry.name, entry.body.length, entry.type));
    }
    blocks.push(padded(entry.body));
  }
  blocks.push(Buffer.alloc(1024));
  return gzipSync(Buffer.concat(blocks));
}

const integrityOf = (raw: Buffer): string => createHash("sha512").update(raw).digest("base64");

function decode(entries: readonly Entry[]): ProviderFile[] {
  const raw = archive(entries);
  const total = entries.reduce((sum, entry) => sum + entry.body.length, 0);
  return members(raw, integrityOf(raw), entries.length, total);
}

const regular = (name: string, body: string): Entry => ({
  name,
  body: Buffer.from(body),
  type: REGULAR,
});

void test("regular files preserve complete original bytes", () => {
  const license = Buffer.from("original license\n");
  const declaration = Buffer.from("export type T = '\u{1f3a8}';\r\n", "utf8");
  for (const pax of [false, true]) {
    const files = decode([
      { name: "package/LICENSE", body: license, type: REGULAR },
      { name: "package/dist/日本語.d.ts", body: declaration, type: REGULAR, pax },
    ]);
    assert.deepEqual(
      files.map((file) => [file.path, file.body]),
      [
        ["LICENSE", license],
        ["dist/日本語.d.ts", declaration],
      ],
    );
    assert.deepEqual(
      files.map((file) => file.mode),
      [0o644, 0o644],
    );
  }
});

void test("digest refuses a modified complete archive", () => {
  const raw = archive([regular("package/LICENSE", "license")]);
  const changed = Buffer.concat([raw, Buffer.from("changed")]);
  assert.throws(() => members(changed, integrityOf(raw), 1, 7), ProviderRefusal);
});

void test("member count and total refuse missing or added data", () => {
  const raw = archive([regular("package/LICENSE", "license")]);
  assert.equal(members(raw, integrityOf(raw), 1, 7).length, 1);
  for (const [count, total] of [
    [0, 7],
    [2, 7],
    [1, 6],
    [1, 8],
  ] as const) {
    assert.throws(() => members(raw, integrityOf(raw), count, total), ProviderRefusal);
  }
});

void test("unsafe members never reach extraction", () => {
  for (const [name, type] of [
    ["/package/file", REGULAR],
    ["elsewhere/file", REGULAR],
    ["package/../outside", REGULAR],
    ["package/./file", REGULAR],
    ["package//file", REGULAR],
    ["package/file/", REGULAR],
    ["package\\outside", REGULAR],
    ["package", REGULAR],
    ["package/link", SYMBOLIC_LINK],
    ["package/hardlink", HARD_LINK],
    ["package/device", CHARACTER_DEVICE],
  ] as const) {
    assert.throws(() => decode([{ name, body: Buffer.alloc(0), type }]), ProviderRefusal, name);
  }
  assert.throws(
    () =>
      decode([{ name: "package/../pax-outside", body: Buffer.alloc(0), type: REGULAR, pax: true }]),
    ProviderRefusal,
    "a pax path is checked like a classic name",
  );
});

void test("duplicate members are refused", () => {
  assert.throws(
    () => decode([regular("package/file", "first"), regular("package/file", "second")]),
    ProviderRefusal,
  );
  for (const names of [
    ["package/file", "package/file/child"],
    ["package/file/child", "package/file"],
  ]) {
    assert.throws(() => decode(names.map((name) => regular(name, "original"))), ProviderRefusal);
  }
});

void test("a truncated or corrupted archive is refused", () => {
  const blocks = Buffer.concat([header("package/LICENSE", 1024, REGULAR), Buffer.alloc(512)]);
  const truncated = gzipSync(blocks);
  assert.throws(() => members(truncated, integrityOf(truncated), 1, 1024), ProviderRefusal);
  const corrupt = header("package/LICENSE", 0, REGULAR);
  corrupt[0] = (corrupt[0] ?? 0) ^ 0xff;
  const raw = gzipSync(Buffer.concat([corrupt, Buffer.alloc(1024)]));
  assert.throws(() => members(raw, integrityOf(raw), 1, 0), ProviderRefusal);
});

void test("an existing or symlink destination is refused before the network", async () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "router-page-provider-"));
  try {
    const existing = path.join(temporary, "existing");
    fs.mkdirSync(existing);
    const link = path.join(temporary, "link");
    fs.symlinkSync(existing, link, "dir");
    const dangling = path.join(temporary, "dangling");
    fs.symlinkSync(path.join(temporary, "nowhere"), dangling);
    for (const target of [existing, link, dangling, path.join(temporary, "missing", "child")]) {
      await assert.rejects(stage(target), ProviderRefusal, target);
    }
    assert.deepEqual(fs.readdirSync(temporary).sort(), ["dangling", "existing", "link"]);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

void test("required exports and the license are authoritative", () => {
  const exports = {
    ".": { types: "./core.d.ts" },
    "./experimental": { types: "./experimental.d.ts" },
    "./auto-routes": { types: "./auto-routes.d.mts" },
    "./auto": { types: "./auto.d.ts" },
    "./volar/sfc-typed-router": { default: "./plugin.cjs", types: "./plugin.d.cts" },
  };
  const manifest = { name: "vue-router", version: "5.1.0", exports };
  const names = [
    "core.d.ts",
    "experimental.d.ts",
    "auto-routes.d.mts",
    "auto.d.ts",
    "plugin.cjs",
    "plugin.d.cts",
    "LICENSE",
  ];
  const file = (name: string, body: string): ProviderFile => ({
    path: name,
    body: Buffer.from(body),
    mode: 0o644,
  });
  const files = [
    file("package.json", JSON.stringify(manifest)),
    ...names.map((name) => file(name, "original")),
  ];
  assert.deepEqual(validatePackage(files), manifest);
  for (const missing of names) {
    assert.throws(
      () => validatePackage(files.filter((entry) => entry.path !== missing)),
      ProviderRefusal,
      missing,
    );
  }
  const other = file("package.json", JSON.stringify({ ...manifest, version: "4.5.1" }));
  assert.throws(() => validatePackage([other, ...files.slice(1)]), ProviderRefusal);
  const escaping = { ...exports, ".": { types: "../core.d.ts" } };
  const outside = file("package.json", JSON.stringify({ ...manifest, exports: escaping }));
  assert.throws(() => validatePackage([outside, ...files.slice(1)]), ProviderRefusal);
});
