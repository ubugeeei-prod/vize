#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import zlib from "node:zlib";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const archivePath = path.resolve(
  process.cwd(),
  process.argv[2] ?? path.join(root, "zed-vize-extension.tar.gz"),
);

assert.ok(fs.existsSync(archivePath), `Zed extension archive does not exist: ${archivePath}`);

const archiveSize = fs.statSync(archivePath).size;
assert.ok(archiveSize > 5_000, `Zed extension archive is suspiciously small: ${archiveSize} bytes`);
assert.ok(
  archiveSize < 2_000_000,
  `Zed extension archive is unexpectedly large: ${archiveSize} bytes`,
);

const entries = readTarGz(archivePath);
const entryNames = entries.map((entry) => entry.name).sort((a, b) => a.localeCompare(b));
const entryMap = new Map(entries.map((entry) => [entry.name, entry]));

assert.deepEqual(
  entryNames,
  Array.from(new Set(entryNames)),
  "Zed archive contains duplicate entries",
);

for (const name of entryNames) {
  assert.ok(!name.includes("\\"), `Zed archive entry must use POSIX separators: ${name}`);
  assert.ok(!name.includes("\0"), `Zed archive entry contains a NUL byte: ${name}`);
  assert.ok(!name.startsWith("/"), `Zed archive entry must be relative: ${name}`);
  assert.ok(!name.split("/").includes(".."), `Zed archive entry must not traverse: ${name}`);
  assert.ok(name === "zed/" || name.startsWith("zed/"), `unexpected root: ${name}`);
}

const requiredFiles = [
  "zed/Cargo.lock",
  "zed/Cargo.toml",
  "zed/LICENSE",
  "zed/README.md",
  "zed/extension.toml",
  "zed/languages/art-vue/brackets.scm",
  "zed/languages/art-vue/config.toml",
  "zed/languages/art-vue/highlights.scm",
  "zed/languages/art-vue/indents.scm",
  "zed/languages/art-vue/injections.scm",
  "zed/languages/art-vue/outline.scm",
  "zed/languages/art-vue/overrides.scm",
  "zed/src/lib.rs",
];

for (const name of requiredFiles) {
  assert.ok(entryMap.has(name), `Zed archive is missing required file: ${name}`);
  assert.ok(readTextEntry(entryMap, name).trim().length > 0, `Zed archive file is empty: ${name}`);
}

const allowedEntries = [
  /^zed\/$/,
  /^zed\/Cargo\.lock$/,
  /^zed\/Cargo\.toml$/,
  /^zed\/LICENSE$/,
  /^zed\/README\.md$/,
  /^zed\/extension\.toml$/,
  /^zed\/languages\/$/,
  /^zed\/languages\/art-vue\/$/,
  /^zed\/languages\/art-vue\/(?:brackets|config|highlights|indents|injections|outline|overrides)\.scm$/,
  /^zed\/languages\/art-vue\/config\.toml$/,
  /^zed\/src\/$/,
  /^zed\/src\/lib\.rs$/,
];

for (const name of entryNames) {
  assert.ok(
    allowedEntries.some((pattern) => pattern.test(name)),
    `Zed archive ships an unexpected file: ${name}`,
  );
}

const forbiddenEntries = [
  /^zed\/\.git/,
  /^zed\/\.github\//,
  /^zed\/\.zed\//,
  /^zed\/node_modules\//,
  /^zed\/target\//,
  /\/\.DS_Store$/,
  /~$/,
];

for (const name of entryNames) {
  for (const pattern of forbiddenEntries) {
    assert.ok(!pattern.test(name), `Zed archive must not ship ${name}`);
  }
}

const workspaceVersion = readWorkspaceVersion();
const extensionToml = readTextEntry(entryMap, "zed/extension.toml");
const cargoToml = readTextEntry(entryMap, "zed/Cargo.toml");
const libRs = readTextEntry(entryMap, "zed/src/lib.rs");
const artVueConfig = readTextEntry(entryMap, "zed/languages/art-vue/config.toml");
const injections = readTextEntry(entryMap, "zed/languages/art-vue/injections.scm");
const highlights = readTextEntry(entryMap, "zed/languages/art-vue/highlights.scm");

assertTomlString(extensionToml, "id", "vize");
assertTomlString(extensionToml, "name", "Vize");
assertTomlString(extensionToml, "version", workspaceVersion);
assertTomlString(extensionToml, "repository", "https://github.com/ubugeeei-prod/vize");
assert.match(extensionToml, /^\[language_servers\.vize\]$/m);
assert.match(extensionToml, /^languages = \["Vue", "Art Vue"\]$/m);
assert.match(extensionToml, /^\[language_servers\.vize\.language_ids\]$/m);
assert.match(extensionToml, /^"Vue" = "vue"$/m);
assert.match(extensionToml, /^"Art Vue" = "art-vue"$/m);
assert.match(extensionToml, /^\[grammars\.art-vue\]$/m);
assert.match(extensionToml, /^commit = "[0-9a-f]{40}"$/m);

assertTomlString(cargoToml, "name", "vize-zed-extension");
assertTomlString(cargoToml, "version", workspaceVersion);
assertTomlString(cargoToml, "edition", "2024");
assertTomlString(cargoToml, "license", "MIT");
assert.match(cargoToml, /^publish = false$/m);
assert.match(cargoToml, /^crate-type = \["cdylib"\]$/m);
assert.match(cargoToml, /^zed_extension_api = "=0\.7\.0"$/m);

assert.match(libRs, /const SERVER_NAME: &'static str = "vize";/);
assert.match(libRs, /const SERVER_BINARY: &'static str = "vize";/);
assert.match(libRs, /worktree\.which\(Self::SERVER_BINARY\)/);
assert.match(libRs, /unwrap_or_else\(\|\| vec!\["lsp"\.to_string\(\)\]\)/);
assert.match(libRs, /language_server_initialization_options/);
assert.match(libRs, /language_server_workspace_configuration/);
assert.match(libRs, /zed::register_extension!\(VizeExtension\);/);

assertTomlString(artVueConfig, "name", "Art Vue");
assertTomlString(artVueConfig, "grammar", "art-vue");
assert.match(artVueConfig, /^path_suffixes = \["art\.vue"\]$/m);
assertTomlString(artVueConfig, "prettier_parser_name", "vue");

for (const queryFile of [
  "brackets",
  "highlights",
  "indents",
  "injections",
  "outline",
  "overrides",
]) {
  const contents = readTextEntry(entryMap, `zed/languages/art-vue/${queryFile}.scm`);
  assert.ok(contents.trim().length > 0, `${queryFile}.scm must not be empty`);
}

assert.match(injections, /directive_attribute/);
assert.match(injections, /style_element/);
assert.match(injections, /template_element/);
assert.match(highlights, /@tag\.component\.type\.constructor/);

console.log(
  `Zed package smoke passed: ${path.relative(root, archivePath)} (${entryNames.length} entries)`,
);

function assertTomlString(source, key, expected) {
  assert.match(source, new RegExp(`^${escapeRegExp(key)} = "${escapeRegExp(expected)}"$`, "m"));
}

function readWorkspaceVersion() {
  const cargoToml = fs.readFileSync(path.join(root, "Cargo.toml"), "utf-8");
  const version = cargoToml.match(/^version = "(.+)"$/m)?.[1];
  assert.ok(version, "workspace version is missing from Cargo.toml");
  return version;
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

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
