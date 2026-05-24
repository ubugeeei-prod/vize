#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import zlib from "node:zlib";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const archivePath = path.resolve(
  process.cwd(),
  process.argv[2] ?? path.join(root, "nvim-vize-extension.tar.gz"),
);

assert.ok(fs.existsSync(archivePath), `Neovim extension archive does not exist: ${archivePath}`);

const archiveSize = fs.statSync(archivePath).size;
assert.ok(archiveSize > 2_000, `Neovim archive is suspiciously small: ${archiveSize} bytes`);
assert.ok(archiveSize < 200_000, `Neovim archive is unexpectedly large: ${archiveSize} bytes`);

const entries = readTarGz(archivePath);
const entryNames = entries.map((entry) => entry.name).sort((a, b) => a.localeCompare(b));
const entryMap = new Map(entries.map((entry) => [entry.name, entry]));

assert.deepEqual(entryNames, Array.from(new Set(entryNames)), "Neovim archive has duplicates");

for (const name of entryNames) {
  assert.ok(!name.includes("\\"), `Neovim archive entry must use POSIX separators: ${name}`);
  assert.ok(!name.includes("\0"), `Neovim archive entry contains a NUL byte: ${name}`);
  assert.ok(!name.startsWith("/"), `Neovim archive entry must be relative: ${name}`);
  assert.ok(!name.split("/").includes(".."), `Neovim archive entry must not traverse: ${name}`);
  assert.ok(name === "nvim-vize/" || name.startsWith("nvim-vize/"), `unexpected root: ${name}`);
}

const requiredFiles = [
  "nvim-vize/LICENSE",
  "nvim-vize/README.md",
  "nvim-vize/ftdetect/vize.lua",
  "nvim-vize/lua/vize/config.lua",
  "nvim-vize/lua/vize/init.lua",
  "nvim-vize/plugin/vize.lua",
  "nvim-vize/test/vize_spec.lua",
];

for (const name of requiredFiles) {
  assert.ok(entryMap.has(name), `Neovim archive is missing required file: ${name}`);
  assert.ok(readTextEntry(entryMap, name).trim().length > 0, `Neovim file is empty: ${name}`);
}

const allowedEntries = [
  /^nvim-vize\/$/,
  /^nvim-vize\/LICENSE$/,
  /^nvim-vize\/README\.md$/,
  /^nvim-vize\/ftdetect\/$/,
  /^nvim-vize\/ftdetect\/vize\.lua$/,
  /^nvim-vize\/lua\/$/,
  /^nvim-vize\/lua\/vize\/$/,
  /^nvim-vize\/lua\/vize\/(?:config|init)\.lua$/,
  /^nvim-vize\/plugin\/$/,
  /^nvim-vize\/plugin\/vize\.lua$/,
  /^nvim-vize\/test\/$/,
  /^nvim-vize\/test\/vize_spec\.lua$/,
];

for (const name of entryNames) {
  assert.ok(
    allowedEntries.some((pattern) => pattern.test(name)),
    `Neovim archive ships an unexpected file: ${name}`,
  );
}

const forbiddenEntries = [
  /^nvim-vize\/\.git/,
  /^nvim-vize\/\.github\//,
  /^nvim-vize\/node_modules\//,
  /^nvim-vize\/target\//,
  /\/\.DS_Store$/,
  /\.tar\.gz$/,
  /~$/,
];

for (const name of entryNames) {
  for (const pattern of forbiddenEntries) {
    assert.ok(!pattern.test(name), `Neovim archive must not ship ${name}`);
  }
}

const configLua = readTextEntry(entryMap, "nvim-vize/lua/vize/config.lua");
const initLua = readTextEntry(entryMap, "nvim-vize/lua/vize/init.lua");
const ftdetectLua = readTextEntry(entryMap, "nvim-vize/ftdetect/vize.lua");
const specLua = readTextEntry(entryMap, "nvim-vize/test/vize_spec.lua");

assert.match(configLua, /cmd = \{ "vize", "lsp" \}/);
assert.match(configLua, /filetypes = \{ "vue", "art-vue" \}/);
assert.match(
  configLua,
  /root_markers = \{ "vize\.config\.pkl", "vize\.config\.json", "package\.json", "\.git" \}/,
);
assert.match(configLua, /lint = true/);
assert.match(configLua, /recommended = \{/);
assert.match(configLua, /assert_list\("cmd"/);
assert.match(initLua, /vim\.lsp\.config\("vize", resolved\)/);
assert.match(initLua, /vim\.lsp\.enable\("vize"\)/);
assert.match(ftdetectLua, /pattern = "\*\.vue"/);
assert.match(ftdetectLua, /pattern = "\*\.art\.vue"/);
assert.match(ftdetectLua, /filetype = "art-vue"/);
assert.match(specLua, /config\.normalize/);
assert.match(specLua, /vim\.lsp\.config\.vize/);

console.log(
  `Neovim package smoke passed: ${path.relative(root, archivePath)} (${entryNames.length} entries)`,
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
