#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import zlib from "node:zlib";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const archivePath = path.resolve(
  process.cwd(),
  process.argv[2] ?? path.join(root, "emacs-vize-extension.tar.gz"),
);

assert.ok(fs.existsSync(archivePath), `Emacs extension archive does not exist: ${archivePath}`);

const archiveSize = fs.statSync(archivePath).size;
assert.ok(archiveSize > 2_000, `Emacs archive is suspiciously small: ${archiveSize} bytes`);
assert.ok(archiveSize < 200_000, `Emacs archive is unexpectedly large: ${archiveSize} bytes`);

const entries = readTarGz(archivePath);
const entryNames = entries.map((entry) => entry.name).sort((a, b) => a.localeCompare(b));
const entryMap = new Map(entries.map((entry) => [entry.name, entry]));

assert.deepEqual(entryNames, Array.from(new Set(entryNames)), "Emacs archive has duplicates");

for (const name of entryNames) {
  assert.ok(!name.includes("\\"), `Emacs archive entry must use POSIX separators: ${name}`);
  assert.ok(!name.includes("\0"), `Emacs archive entry contains a NUL byte: ${name}`);
  assert.ok(!name.startsWith("/"), `Emacs archive entry must be relative: ${name}`);
  assert.ok(!name.split("/").includes(".."), `Emacs archive entry must not traverse: ${name}`);
  assert.ok(name === "emacs-vize/" || name.startsWith("emacs-vize/"), `unexpected root: ${name}`);
}

const requiredFiles = [
  "emacs-vize/LICENSE",
  "emacs-vize/README.md",
  "emacs-vize/test/vize-test.el",
  "emacs-vize/vize.el",
];

for (const name of requiredFiles) {
  assert.ok(entryMap.has(name), `Emacs archive is missing required file: ${name}`);
  assert.ok(readTextEntry(entryMap, name).trim().length > 0, `Emacs file is empty: ${name}`);
}

const allowedEntries = [
  /^emacs-vize\/$/,
  /^emacs-vize\/LICENSE$/,
  /^emacs-vize\/README\.md$/,
  /^emacs-vize\/test\/$/,
  /^emacs-vize\/test\/vize-test\.el$/,
  /^emacs-vize\/vize\.el$/,
];

for (const name of entryNames) {
  assert.ok(
    allowedEntries.some((pattern) => pattern.test(name)),
    `Emacs archive ships an unexpected file: ${name}`,
  );
}

const forbiddenEntries = [
  /^emacs-vize\/\.git/,
  /^emacs-vize\/\.github\//,
  /^emacs-vize\/node_modules\//,
  /^emacs-vize\/target\//,
  /\/\.DS_Store$/,
  /\.elc$/,
  /\.tar\.gz$/,
  /~$/,
];

for (const name of entryNames) {
  for (const pattern of forbiddenEntries) {
    assert.ok(!pattern.test(name), `Emacs archive must not ship ${name}`);
  }
}

const vizeEl = readTextEntry(entryMap, "emacs-vize/vize.el");
const testEl = readTextEntry(entryMap, "emacs-vize/test/vize-test.el");

assert.match(vizeEl, /lexical-binding: t/);
assert.match(vizeEl, /defcustom vize-eglot-command '\("vize" "lsp"\)/);
assert.match(vizeEl, /defcustom vize-eglot-profile 'lint/);
assert.match(vizeEl, /recommended \. \(:editor t :ecosystem t :lint t :typecheck t\)/);
assert.match(vizeEl, /define-derived-mode vize-vue-mode/);
assert.match(vizeEl, /define-derived-mode vize-art-vue-mode/);
assert.ok(vizeEl.includes("(add-to-list 'auto-mode-alist '(\"\\\\.vue\\\\'\" . vize-vue-mode))"));
assert.ok(
  vizeEl.includes(
    "(add-to-list 'auto-mode-alist '(\"\\\\.art\\\\.vue\\\\'\" . vize-art-vue-mode))",
  ),
);
assert.match(vizeEl, /:initializationOptions options/);
assert.match(vizeEl, /eglot-server-programs/);
assert.match(vizeEl, /provide 'vize/);
assert.match(testEl, /ert-deftest vize-eglot-default-program/);
assert.match(testEl, /:initializationOptions \(:lint t\)/);
assert.match(testEl, /ert-deftest vize-eglot-off-program/);

console.log(
  `Emacs package smoke passed: ${path.relative(root, archivePath)} (${entryNames.length} entries)`,
);

function readTextEntry(entryMap, name) {
  const entry = entryMap.get(name);
  assert.ok(entry, `missing tar entry: ${name}`);
  return entry.data.toString("utf-8");
}

function readTarGz(filePath) {
  const buffer = zlib.gunzipSync(fs.readFileSync(filePath));
  const entries = [];
  let offset = 0;

  while (offset + 512 <= buffer.byteLength) {
    const header = buffer.subarray(offset, offset + 512);
    if (header.every((byte) => byte === 0)) {
      break;
    }

    const name = readTarString(header, 0, 100);
    const prefix = readTarString(header, 345, 155);
    const fullName = prefix ? `${prefix}/${name}` : name;
    const size = parseOctal(readTarString(header, 124, 12));
    const typeflag = readTarString(header, 156, 1) || "0";
    const dataOffset = offset + 512;
    const data = buffer.subarray(dataOffset, dataOffset + size);

    assert.ok(fullName, "tar entry name must not be empty");
    if (typeflag === "x" || typeflag === "g") {
      offset = dataOffset + Math.ceil(size / 512) * 512;
      continue;
    }

    assert.ok(
      typeflag === "0" || typeflag === "5",
      `unsupported tar entry type ${typeflag}: ${fullName}`,
    );

    entries.push({ data, name: fullName, size, typeflag });
    offset = dataOffset + Math.ceil(size / 512) * 512;
  }

  return entries;
}

function readTarString(buffer, offset, length) {
  const raw = buffer.subarray(offset, offset + length);
  const end = raw.indexOf(0);
  return raw.subarray(0, end === -1 ? raw.byteLength : end).toString("utf-8");
}

function parseOctal(value) {
  const trimmed = value.trim();
  return trimmed ? Number.parseInt(trimmed, 8) : 0;
}
