#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import { builtinModules } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import zlib from "node:zlib";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const vsixPath = path.resolve(
  process.cwd(),
  process.argv[2] ?? path.join(root, "npm/vscode-vize/dist/vize.vsix"),
);
const builtins = new Set([
  ...builtinModules,
  ...builtinModules.map((moduleName) => `node:${moduleName}`),
]);

assert.ok(fs.existsSync(vsixPath), `VSIX does not exist: ${vsixPath}`);

const archive = readZip(vsixPath);
const entryNames = archive.entries.map((entry) => entry.name).sort();
const entries = new Set(entryNames);
const vsixSize = fs.statSync(vsixPath).size;

assert.ok(vsixSize > 20_000, `VSIX is suspiciously small: ${vsixSize} bytes`);
assert.ok(vsixSize < 5_000_000, `VSIX is unexpectedly large: ${vsixSize} bytes`);
assert.deepEqual(entryNames, Array.from(new Set(entryNames)), "VSIX contains duplicate entries");

for (const name of entryNames) {
  assert.ok(!name.includes("\\"), `VSIX entry must use POSIX separators: ${name}`);
  assert.ok(!name.includes("\0"), `VSIX entry contains a NUL byte: ${name}`);
  assert.ok(!name.startsWith("/"), `VSIX entry must be relative: ${name}`);
  assert.ok(!name.split("/").includes(".."), `VSIX entry must not traverse: ${name}`);
  assert.match(
    name,
    /^(?:extension\/|extension\.vsixmanifest$|\[Content_Types\]\.xml$)/,
    `VSIX entry has an unexpected top-level path: ${name}`,
  );
}

const requiredFiles = [
  "[Content_Types].xml",
  "extension.vsixmanifest",
  "extension/LICENSE.txt",
  "extension/changelog.md",
  "extension/dist/extension.cjs",
  "extension/icons/logo.png",
  "extension/icons/vue.svg",
  "extension/language-configuration.json",
  "extension/package.json",
  "extension/readme.md",
  "extension/syntaxes/art-vue.tmLanguage.json",
  "extension/syntaxes/vue.tmLanguage.json",
];

for (const name of requiredFiles) {
  assert.ok(entries.has(name), `VSIX is missing required file: ${name}`);
  assert.ok(readEntryBuffer(archive, name).byteLength > 0, `VSIX file is empty: ${name}`);
}

const allowedExtensionEntries = [
  /^extension\/LICENSE\.txt$/,
  /^extension\/changelog\.md$/,
  /^extension\/dist\/extension\.cjs$/,
  /^extension\/icons\/(?:logo\.png|vue\.svg)$/,
  /^extension\/language-configuration\.json$/,
  /^extension\/package\.json$/,
  /^extension\/readme\.md$/,
  /^extension\/syntaxes\/(?:art-vue|vue)\.tmLanguage\.json$/,
];

for (const name of entryNames.filter((entry) => entry.startsWith("extension/"))) {
  assert.ok(
    allowedExtensionEntries.some((pattern) => pattern.test(name)),
    `VSIX ships an unexpected extension file: ${name}`,
  );
}

const forbiddenEntries = [
  /^extension\/\.github\//,
  /^extension\/\.vscode-test\//,
  /^extension\/\.vscode\//,
  /^extension\/dist\/.*\.map$/,
  /^extension\/node_modules\//,
  /^extension\/package-lock\.json$/,
  /^extension\/pnpm-lock\.yaml$/,
  /^extension\/src\//,
  /^extension\/test(?:s)?\//,
  /^extension\/test-fixtures\//,
  /^extension\/tsconfig\.json$/,
  /^extension\/vite\.config\.ts$/,
  /\.vsix$/,
];

for (const name of entryNames) {
  for (const pattern of forbiddenEntries) {
    assert.ok(!pattern.test(name), `VSIX must not ship ${name}`);
  }
}

const packageJson = readJsonEntry(archive, "extension/package.json");
const workspaceVersion = readWorkspaceVersion();

assert.equal(packageJson.name, "vize");
assert.equal(packageJson.displayName, "Vize");
assert.equal(packageJson.publisher, "ubugeeei");
assert.equal(packageJson.version, workspaceVersion);
assert.equal(packageJson.main, "./dist/extension.cjs");
assert.equal(packageJson.engines?.vscode, "^1.75.0");
assert.equal(packageJson.dependencies?.["vscode-languageclient"], "9.0.1");

assertUniqueStrings(packageJson.activationEvents, "activationEvents");
assertUniqueStrings(
  packageJson.contributes?.commands?.map((command) => command.command),
  "contributes.commands",
);

for (const command of packageJson.contributes.commands) {
  assert.equal(command.category, "Vize", `${command.command} must stay in the Vize category`);
  assert.ok(command.title, `${command.command} must have a title`);
  assert.ok(
    packageJson.activationEvents.includes(`onCommand:${command.command}`),
    `${command.command} must activate the extension when invoked directly`,
  );
}

assert.ok(packageJson.activationEvents.includes("onLanguage:vue"));
assert.ok(packageJson.activationEvents.includes("onLanguage:art-vue"));

const languages = new Map(
  packageJson.contributes.languages.map((language) => [language.id, language]),
);
assertLanguage(languages, "vue", ".vue");
assertLanguage(languages, "art-vue", ".art.vue");

const grammars = new Map(
  packageJson.contributes.grammars.map((grammar) => [grammar.language, grammar]),
);
assertGrammar(grammars, "vue", "source.vue", "./syntaxes/vue.tmLanguage.json");
assertGrammar(grammars, "art-vue", "source.art-vue", "./syntaxes/art-vue.tmLanguage.json");

const configurationProperties = packageJson.contributes.configuration.properties;
assert.equal(configurationProperties["vize.enable"].default, true);
assert.equal(configurationProperties["vize.serverPath"].default, "");
assert.equal(configurationProperties["vize.trace.server"].default, "off");

for (const [key, property] of Object.entries(configurationProperties)) {
  if (key === "vize.serverPath" || key === "vize.trace.server") {
    continue;
  }
  const expectedDefault =
    key === "vize.diagnostics.enable" ||
    key === "vize.formatting.enable" ||
    key === "vize.legacyVue2.enable"
      ? false
      : true;
  assert.equal(property.default, expectedDefault, `${key} has an unexpected default`);
}

const extensionBundle = readTextEntry(archive, "extension/dist/extension.cjs");
assert.match(extensionBundle, /exports\.activate=/);
assert.match(extensionBundle, /exports\.deactivate=/);
assert.doesNotMatch(extensionBundle, /sourceMappingURL=/);
assert.doesNotMatch(extensionBundle, /\bvscode-languageclient\/node(?:\.js)?\b/);

for (const dependency of findStaticRequires(extensionBundle)) {
  assert.ok(
    isAllowedRuntimeRequire(dependency),
    `extension.cjs has an unpackaged runtime require: ${dependency}`,
  );
}

const vsixManifest = readTextEntry(archive, "extension.vsixmanifest");
assert.match(vsixManifest, /<Identity\b[^>]*\bId="vize"/);
assert.match(
  vsixManifest,
  new RegExp(`<Identity\\b[^>]*\\bVersion="${escapeRegExp(workspaceVersion)}"`),
);
assert.match(vsixManifest, /<Identity\b[^>]*\bPublisher="ubugeeei"/);
assert.match(vsixManifest, /Microsoft\.VisualStudio\.Code/);

console.log(`VSIX smoke passed: ${path.relative(root, vsixPath)} (${entryNames.length} files)`);

function assertLanguage(languages, id, extension) {
  const language = languages.get(id);
  assert.ok(language, `missing language contribution: ${id}`);
  assert.ok(language.aliases.includes(id === "vue" ? "Vue" : "Art Vue"));
  assert.ok(language.extensions.includes(extension));
  assert.equal(language.configuration, "./language-configuration.json");
  assert.equal(language.icon.light, "./icons/vue.svg");
  assert.equal(language.icon.dark, "./icons/vue.svg");
}

function assertGrammar(grammars, language, scopeName, grammarPath) {
  const grammar = grammars.get(language);
  assert.ok(grammar, `missing grammar contribution: ${language}`);
  assert.equal(grammar.scopeName, scopeName);
  assert.equal(grammar.path, grammarPath);
  assert.ok(
    entries.has(`extension/${grammarPath.slice(2)}`),
    `missing grammar file: ${grammarPath}`,
  );

  for (const [scope, embeddedLanguage] of Object.entries({
    "source.css": "css",
    "source.css.less": "less",
    "source.css.scss": "scss",
    "source.js": "javascript",
    "source.json": "json",
    "source.ts": "typescript",
    "text.html.basic": "html",
  })) {
    assert.equal(grammar.embeddedLanguages?.[scope], embeddedLanguage);
  }
}

function assertUniqueStrings(values, label) {
  assert.ok(Array.isArray(values), `${label} must be an array`);
  for (const value of values) {
    assert.equal(typeof value, "string", `${label} must only contain strings`);
  }
  assert.deepEqual(values, Array.from(new Set(values)), `${label} must not contain duplicates`);
}

function findStaticRequires(source) {
  return Array.from(
    source.matchAll(/\brequire\(\s*["'`]([^"'`]+)["'`]\s*\)/g),
    (match) => match[1],
  );
}

function isAllowedRuntimeRequire(specifier) {
  return specifier === "vscode" || builtins.has(specifier);
}

function readWorkspaceVersion() {
  const cargoToml = fs.readFileSync(path.join(root, "Cargo.toml"), "utf-8");
  const version = cargoToml.match(/^version = "(.+)"$/m)?.[1];
  assert.ok(version, "workspace version is missing from Cargo.toml");
  return version;
}

function readJsonEntry(archive, name) {
  return JSON.parse(readTextEntry(archive, name));
}

function readTextEntry(archive, name) {
  return readEntryBuffer(archive, name).toString("utf-8");
}

function readEntryBuffer(archive, name) {
  const entry = archive.entriesByName.get(name);
  assert.ok(entry, `missing zip entry: ${name}`);

  const localHeaderOffset = entry.localHeaderOffset;
  assert.equal(archive.buffer.readUInt32LE(localHeaderOffset), 0x04034b50);

  const fileNameLength = archive.buffer.readUInt16LE(localHeaderOffset + 26);
  const extraLength = archive.buffer.readUInt16LE(localHeaderOffset + 28);
  const dataOffset = localHeaderOffset + 30 + fileNameLength + extraLength;
  const compressed = archive.buffer.subarray(dataOffset, dataOffset + entry.compressedSize);

  if (entry.compressionMethod === 0) {
    assert.equal(compressed.byteLength, entry.uncompressedSize);
    return compressed;
  }

  if (entry.compressionMethod === 8) {
    const inflated = zlib.inflateRawSync(compressed);
    assert.equal(inflated.byteLength, entry.uncompressedSize);
    return inflated;
  }

  assert.fail(`unsupported compression method ${entry.compressionMethod} for ${name}`);
}

function readZip(filePath) {
  const buffer = fs.readFileSync(filePath);
  const eocdOffset = findEndOfCentralDirectory(buffer);

  const diskNumber = buffer.readUInt16LE(eocdOffset + 4);
  const centralDirectoryDisk = buffer.readUInt16LE(eocdOffset + 6);
  const totalEntries = buffer.readUInt16LE(eocdOffset + 10);
  const centralDirectoryOffset = buffer.readUInt32LE(eocdOffset + 16);

  assert.equal(diskNumber, 0, "multi-disk ZIP files are not supported");
  assert.equal(centralDirectoryDisk, 0, "multi-disk ZIP files are not supported");
  assert.notEqual(totalEntries, 0xffff, "ZIP64 VSIX files are not supported by this smoke test");
  assert.notEqual(
    centralDirectoryOffset,
    0xffffffff,
    "ZIP64 VSIX files are not supported by this smoke test",
  );

  const entries = [];
  let offset = centralDirectoryOffset;
  for (let index = 0; index < totalEntries; index++) {
    assert.equal(buffer.readUInt32LE(offset), 0x02014b50);

    const compressionMethod = buffer.readUInt16LE(offset + 10);
    const compressedSize = buffer.readUInt32LE(offset + 20);
    const uncompressedSize = buffer.readUInt32LE(offset + 24);
    const fileNameLength = buffer.readUInt16LE(offset + 28);
    const extraLength = buffer.readUInt16LE(offset + 30);
    const commentLength = buffer.readUInt16LE(offset + 32);
    const localHeaderOffset = buffer.readUInt32LE(offset + 42);
    const name = buffer.subarray(offset + 46, offset + 46 + fileNameLength).toString("utf-8");

    entries.push({
      compressedSize,
      compressionMethod,
      localHeaderOffset,
      name,
      uncompressedSize,
    });
    offset += 46 + fileNameLength + extraLength + commentLength;
  }

  return {
    buffer,
    entries,
    entriesByName: new Map(entries.map((entry) => [entry.name, entry])),
  };
}

function findEndOfCentralDirectory(buffer) {
  const minOffset = Math.max(0, buffer.byteLength - 65_557);
  for (let offset = buffer.byteLength - 22; offset >= minOffset; offset--) {
    if (buffer.readUInt32LE(offset) === 0x06054b50) {
      return offset;
    }
  }
  assert.fail("could not find ZIP end of central directory");
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
