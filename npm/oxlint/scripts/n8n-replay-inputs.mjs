import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const repository = fileURLToPath(new URL("../../../", import.meta.url));
export const fixturePath = "tests/_fixtures/_git/n8n";
export const manifest = JSON.parse(
  fs.readFileSync(path.join(repository, "tests/_fixtures/n8n-adoption.json"), "utf8"),
);
const layerPath = path.join(
  repository,
  "tests/_fixtures/differential/lint/oxlint-script-safe-carrier-7903/rules.json",
);
export const layerBytes = fs.readFileSync(layerPath, "utf8");
export const layer = JSON.parse(layerBytes);
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
export const compareBytes = (a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b));
export const editorPrefix = "packages/frontend/editor-ui/";
const designPrefix = "packages/frontend/@n8n/design-system/";
export const disabledFiles = [
  ...manifest.adoption.packageOverrides["editor-ui"][
    "vize/vue/no-multiple-template-root"
  ].files.map((file) => editorPrefix + file),
  ...manifest.adoption.packageOverrides["design-system"]["vize/vue/require-v-for-key"].files.map(
    (file) => designPrefix + file,
  ),
].sort(compareBytes);

assert.deepEqual(
  Object.fromEntries(Object.entries(layer.rules).filter(([name]) => name.startsWith("vize/"))),
  manifest.adoption.rules,
);
assert.equal(Object.keys(manifest.adoption.rules).length, 51);
assert.equal(Object.keys(layer.rules).length, 81);
assert.deepEqual(layer.settings, { vize: { preset: "incremental", helpLevel: "none" } });
assert.equal(disabledFiles.length, 6);

// Oxlint replaces an option-bearing entry when a package specifies a scalar
// severity. In particular editor-ui's "warn" does not retain ["always"].
export function effectiveRules(file) {
  const rules = { ...manifest.adoption.rules };
  if (file.startsWith(editorPrefix)) {
    rules["vize/vue/attribute-hyphenation"] = "warn";
    if (disabledFiles.includes(file)) rules["vize/vue/no-multiple-template-root"] = "off";
  }
  if (file.startsWith(designPrefix) && disabledFiles.includes(file))
    rules["vize/vue/require-v-for-key"] = "off";
  return rules;
}

export function replayConfig(plugin, mode) {
  assert.ok(["baseline", "shared", "effective", "stock"].includes(mode));
  const hinted = mode === "shared" || mode === "effective";
  const settings = (rules) => ({
    vize: { ...manifest.adoption.settings.vize, ...(hinted ? { rules } : {}) },
  });
  const overrides = [
    {
      files: [editorPrefix + "**/*.vue"],
      rules: { "vize/vue/attribute-hyphenation": "warn" },
      ...(mode === "effective" ? { settings: settings(effectiveRules(editorPrefix)) } : {}),
    },
    ...disabledFiles.map((file) => ({
      files: [file],
      rules: {
        [file.startsWith(editorPrefix)
          ? "vize/vue/no-multiple-template-root"
          : "vize/vue/require-v-for-key"]: "off",
      },
      ...(mode === "effective" ? { settings: settings(effectiveRules(file)) } : {}),
    })),
  ];
  const config = {
    // Keep the complete existing native/Vize object layer, including literal
    // option arrays, through a TS/MTS object extends boundary. No JSONC parser
    // or string-extends resolution is substituted for the frozen contract.
    extends: [{ ...layer, jsPlugins: [plugin] }],
    settings: settings(manifest.adoption.rules),
    overrides,
  };
  if (mode === "stock") {
    config.rules = Object.fromEntries(
      Object.keys(manifest.adoption.rules).map((name) => [name, "off"]),
    );
    for (const override of config.overrides)
      override.rules = Object.fromEntries(Object.keys(override.rules).map((name) => [name, "off"]));
  }
  return config;
}

export function verifyPhysicalEntry(fixture, entry) {
  assert.ok(["100644", "100755"].includes(entry.mode));
  assert.equal(entry.kind, "blob");
  const physical = path.join(fs.realpathSync(fixture), entry.file);
  assert.ok(fs.lstatSync(physical).isFile());
  assert.equal(fs.realpathSync(physical), physical);
  const bytes = fs.readFileSync(physical);
  assert.ok(Buffer.from(bytes.toString("utf8")).equals(bytes), "source must be exact UTF-8");
  const blob = createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
  assert.equal(blob, entry.blob, `${entry.file}: physical bytes differ from the pinned Git object`);
  return {
    ...entry,
    sha256: sha256(bytes),
    bytes: bytes.length,
    scriptless: !/^\s*<script\b/m.test(bytes.toString("utf8")),
  };
}

export function verifyCorpus(fixture = path.join(repository, fixturePath)) {
  fixture = fs.realpathSync(fixture);
  const git = (args, cwd = fixture) =>
    execFileSync("git", args, { cwd, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 });
  assert.equal(
    git(["ls-files", "--stage", fixturePath], repository),
    `160000 ${manifest.fixtureRevision} 0\t${fixturePath}\n`,
    "the current Vize source must retain its pinned n8n gitlink",
  );
  assert.equal(git(["rev-parse", "HEAD"]).trim(), manifest.fixtureRevision);
  assert.equal(git(["status", "--porcelain", "--untracked-files=all"]), "");
  const entries = git(["ls-tree", "-rz", "HEAD", "--", "packages"])
    .split("\0")
    .filter(Boolean)
    .map((entry) => {
      const [metadata, file] = entry.split("\t");
      const [mode, kind, blob] = metadata.split(" ");
      return { mode, kind, blob, file };
    })
    .filter(({ file }) => file.endsWith(".vue"))
    .sort((a, b) => Buffer.compare(Buffer.from(a.file), Buffer.from(b.file)));
  assert.equal(entries.length, manifest.corpus.vueFileCount);
  assert.deepEqual(
    fs.globSync("packages/**/*.vue", { cwd: fixture }).sort(compareBytes),
    entries.map(({ file }) => file).sort(compareBytes),
  );
  const files = entries.map((entry) => verifyPhysicalEntry(fixture, entry));
  assert.deepEqual(
    files
      .filter(({ scriptless }) => scriptless)
      .map(({ file }) => file)
      .sort(compareBytes),
    manifest.corpus.scriptlessFiles,
  );
  const packages = Object.fromEntries(
    Object.keys(manifest.corpus.packageVueFileCounts).map((prefix) => [
      prefix,
      files.filter(({ file }) => file.startsWith(prefix + "/")).length,
    ]),
  );
  assert.deepEqual(packages, manifest.corpus.packageVueFileCounts);
  assert.equal(
    Object.values(packages).reduce((a, b) => a + b, 0),
    files.length,
  );
  for (const file of disabledFiles) assert.ok(files.some((entry) => entry.file === file));
  const licenseEntries = git(["ls-tree", "-z", "HEAD", "--", "LICENSE.md", "LICENSE_EE.md"])
    .split("\0")
    .filter(Boolean)
    .map((entry) => {
      const [metadata, file] = entry.split("\t");
      const [mode, kind, blob] = metadata.split(" ");
      return verifyPhysicalEntry(fixture, { mode, kind, blob, file });
    });
  assert.deepEqual(
    licenseEntries.map(({ file }) => file),
    ["LICENSE.md", "LICENSE_EE.md"],
  );
  return {
    fixture,
    revision: manifest.fixtureRevision,
    tree: git(["rev-parse", "HEAD^{tree}"]).trim(),
    files,
    packages,
    disabledFiles,
    licenses: licenseEntries,
  };
}
