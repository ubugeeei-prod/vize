#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import zlib from "node:zlib";
import toml from "@iarna/toml";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const archivePath = path.resolve(
  process.cwd(),
  process.argv[2] ?? path.join(root, "helix-vize-extension.tar.gz"),
);

assert.ok(fs.existsSync(archivePath), `Helix extension archive does not exist: ${archivePath}`);

const archiveSize = fs.statSync(archivePath).size;
assert.ok(archiveSize > 1_000, `Helix archive is suspiciously small: ${archiveSize} bytes`);
assert.ok(archiveSize < 100_000, `Helix archive is unexpectedly large: ${archiveSize} bytes`);

const entries = readTarGz(archivePath);
const entryNames = entries.map((entry) => entry.name).sort((a, b) => a.localeCompare(b));
const entryMap = new Map(entries.map((entry) => [entry.name, entry]));

assert.deepEqual(entryNames, Array.from(new Set(entryNames)), "Helix archive has duplicates");

for (const name of entryNames) {
  assert.ok(!name.includes("\\"), `Helix archive entry must use POSIX separators: ${name}`);
  assert.ok(!name.includes("\0"), `Helix archive entry contains a NUL byte: ${name}`);
  assert.ok(!name.startsWith("/"), `Helix archive entry must be relative: ${name}`);
  assert.ok(!name.split("/").includes(".."), `Helix archive entry must not traverse: ${name}`);
  assert.ok(name === "helix-vize/" || name.startsWith("helix-vize/"), `unexpected root: ${name}`);
}

const requiredFiles = ["helix-vize/LICENSE", "helix-vize/README.md", "helix-vize/languages.toml"];

for (const name of requiredFiles) {
  assert.ok(entryMap.has(name), `Helix archive is missing required file: ${name}`);
  assert.ok(readTextEntry(entryMap, name).trim().length > 0, `Helix file is empty: ${name}`);
}

const allowedEntries = [
  /^helix-vize\/$/,
  /^helix-vize\/LICENSE$/,
  /^helix-vize\/README\.md$/,
  /^helix-vize\/languages\.toml$/,
];

for (const name of entryNames) {
  assert.ok(
    allowedEntries.some((pattern) => pattern.test(name)),
    `Helix archive ships an unexpected file: ${name}`,
  );
}

const forbiddenEntries = [
  /^helix-vize\/\.git/,
  /^helix-vize\/\.github\//,
  /^helix-vize\/node_modules\//,
  /^helix-vize\/target\//,
  /\/\.DS_Store$/,
  /\.tar\.gz$/,
  /~$/,
];

for (const name of entryNames) {
  for (const pattern of forbiddenEntries) {
    assert.ok(!pattern.test(name), `Helix archive must not ship ${name}`);
  }
}

const languagesToml = readTextEntry(entryMap, "helix-vize/languages.toml");
const parsed = toml.parse(languagesToml);

assert.equal(parsed["language-server"].vize.command, "vize");
assert.deepEqual(parsed["language-server"].vize.args, ["lsp"]);
assert.deepEqual(parsed["language-server"].vize.config, { lint: true });

const languages = new Map(parsed.language.map((language) => [language.name, language]));
assertLanguage(languages.get("vue"), {
  fileTypes: ["vue"],
  languageId: undefined,
  scope: "source.vue",
});
assertLanguage(languages.get("art-vue"), {
  fileTypes: [{ glob: "*.art.vue" }],
  languageId: "art-vue",
  scope: "source.art-vue",
});

assert.match(languagesToml, /^\[language-server\.vize\]$/m);
assert.match(languagesToml, /^command = "vize"$/m);
assert.match(languagesToml, /^args = \["lsp"\]$/m);
assert.match(languagesToml, /^\[language-server\.vize\.config\]$/m);
assert.match(languagesToml, /^lint = true$/m);
assert.match(languagesToml, /^file-types = \[\{ glob = "\*\.art\.vue" \}\]$/m);

console.log(
  `Helix package smoke passed: ${path.relative(root, archivePath)} (${entryNames.length} entries)`,
);

function assertLanguage(language, expected) {
  assert.ok(language, `missing language: ${expected.scope}`);
  assert.equal(language.scope, expected.scope);
  assert.equal(language["language-id"], expected.languageId);
  assert.deepEqual(language["file-types"], expected.fileTypes);
  assert.deepEqual(language.roots, ["vize.config.pkl", "vize.config.json", "package.json", ".git"]);
  assert.deepEqual(language["language-servers"], ["vize"]);
}

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
