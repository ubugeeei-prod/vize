#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import zlib from "node:zlib";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const archivePath = path.resolve(
  process.cwd(),
  process.argv[2] ?? path.join(root, "vim-vize-extension.tar.gz"),
);

assert.ok(fs.existsSync(archivePath), `Vim extension archive does not exist: ${archivePath}`);

const archiveSize = fs.statSync(archivePath).size;
assert.ok(archiveSize > 2_000, `Vim archive is suspiciously small: ${archiveSize} bytes`);
assert.ok(archiveSize < 200_000, `Vim archive is unexpectedly large: ${archiveSize} bytes`);

const entries = readTarGz(archivePath);
const entryNames = entries.map((entry) => entry.name).sort((a, b) => a.localeCompare(b));
const entryMap = new Map(entries.map((entry) => [entry.name, entry]));

assert.deepEqual(entryNames, Array.from(new Set(entryNames)), "Vim archive has duplicates");

for (const name of entryNames) {
  assert.ok(!name.includes("\\"), `Vim archive entry must use POSIX separators: ${name}`);
  assert.ok(!name.includes("\0"), `Vim archive entry contains a NUL byte: ${name}`);
  assert.ok(!name.startsWith("/"), `Vim archive entry must be relative: ${name}`);
  assert.ok(!name.split("/").includes(".."), `Vim archive entry must not traverse: ${name}`);
  assert.ok(name === "vim/" || name.startsWith("vim/"), `unexpected root: ${name}`);
}

const requiredFiles = [
  "vim/LICENSE",
  "vim/README.md",
  "vim/autoload/vize.vim",
  "vim/ftdetect/vize.vim",
  "vim/plugin/vize.vim",
  "vim/test/vize_spec.vim",
];

for (const name of requiredFiles) {
  assert.ok(entryMap.has(name), `Vim archive is missing required file: ${name}`);
  assert.ok(readTextEntry(entryMap, name).trim().length > 0, `Vim file is empty: ${name}`);
}

const allowedEntries = [
  /^vim\/$/,
  /^vim\/LICENSE$/,
  /^vim\/README\.md$/,
  /^vim\/autoload\/$/,
  /^vim\/autoload\/vize\.vim$/,
  /^vim\/ftdetect\/$/,
  /^vim\/ftdetect\/vize\.vim$/,
  /^vim\/plugin\/$/,
  /^vim\/plugin\/vize\.vim$/,
  /^vim\/test\/$/,
  /^vim\/test\/vize_spec\.vim$/,
];

for (const name of entryNames) {
  assert.ok(
    allowedEntries.some((pattern) => pattern.test(name)),
    `Vim archive ships an unexpected file: ${name}`,
  );
}

const forbiddenEntries = [
  /^vim\/\.git/,
  /^vim\/\.github\//,
  /^vim\/node_modules\//,
  /^vim\/target\//,
  /\/\.DS_Store$/,
  /\.tar\.gz$/,
  /~$/,
];

for (const name of entryNames) {
  for (const pattern of forbiddenEntries) {
    assert.ok(!pattern.test(name), `Vim archive must not ship ${name}`);
  }
}

const autoload = readTextEntry(entryMap, "vim/autoload/vize.vim");
const ftdetect = readTextEntry(entryMap, "vim/ftdetect/vize.vim");
const spec = readTextEntry(entryMap, "vim/test/vize_spec.vim");

assert.match(autoload, /'cmd': \['vize', 'lsp'\]/);
assert.match(autoload, /'allowlist': \['vue', 'art-vue'\]/);
assert.match(autoload, /'initialization_options': s:profiles\.lint/);
assert.match(autoload, /function! vize#vim_lsp_config/);
assert.match(autoload, /lsp#register_server/);
assert.match(ftdetect, /\*\.vue setlocal filetype=vue/);
assert.match(ftdetect, /\*\.art\.vue setlocal filetype=art-vue/);
assert.match(spec, /vize#vim_lsp_config/);
assert.match(spec, /assert_equal\(\['vize', 'lsp'\]/);

console.log(
  `Vim package smoke passed: ${path.relative(root, archivePath)} (${entryNames.length} entries)`,
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
