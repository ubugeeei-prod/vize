import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { stageSourceNativeEntrypoint } from "../../tools/support/compat/nuxt/source-native-entry.ts";

const published =
  'const native = require("./native-binding"); module.exports = native; module.exports.compileSfc = native.compileSfc;\n';
const current = `${published}module.exports.typeProjectInputs = native.typeProjectInputs;\n`;
const binding =
  "module.exports = { compileSfc() {}, typeProjectInputs(root, revision) { return [root, revision]; } };\n";

function fixture() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "source-native-entry-"));
  const source = path.join(root, "npm/native");
  const app = path.join(root, "fixture");
  const installed = path.join(app, "node_modules/@vizejs/native");
  const plugin = path.join(app, "node_modules/@vizejs/vite-plugin");
  const artifacts = path.join(root, "artifacts");
  for (const directory of [source, installed, plugin, artifacts])
    fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(path.join(plugin, "package.json"), "{}\n");
  fs.writeFileSync(path.join(installed, "package.json"), '{"main":"index.js"}\n');
  fs.writeFileSync(path.join(source, "index.js"), current);
  fs.writeFileSync(path.join(installed, "index.js"), published);
  for (const directory of [source, installed]) {
    fs.writeFileSync(path.join(directory, "native-binding.js"), binding);
    fs.writeFileSync(path.join(directory, "native-binding-identity.js"), "module.exports = {};\n");
    fs.writeFileSync(path.join(directory, "native-targets.js"), "module.exports = [];\n");
  }
  return { root, app, installed, artifacts };
}

function importShape(entry) {
  const script = `import * as api from ${JSON.stringify(pathToFileURL(entry).href)};
    console.log(JSON.stringify({ cjs: typeof api.default.typeProjectInputs,
      esm: typeof api.typeProjectInputs,
      result: api.typeProjectInputs?.("/project/tsconfig.json", "7") ?? null }));`;
  const result = spawnSync(process.execPath, ["--input-type=module", "--eval", script], {
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}

await test("a source-only API reaches ESM through the paired entrypoint and restores the original", () => {
  const f = fixture();
  try {
    const entry = path.join(f.installed, "index.js");
    assert.deepEqual(importShape(entry), { cjs: "function", esm: "undefined", result: null });
    const staged = stageSourceNativeEntrypoint(f.root, f.app, f.artifacts);
    assert.deepEqual(importShape(entry), {
      cjs: "function",
      esm: "function",
      result: ["/project/tsconfig.json", "7"],
    });
    staged.verify();
    assert.equal(
      fs.readFileSync(path.join(f.artifacts, "published-native-index.js"), "utf8"),
      published,
    );
    staged.restore();
    assert.equal(fs.readFileSync(entry, "utf8"), published);
    assert.deepEqual(importShape(entry), { cjs: "function", esm: "undefined", result: null });
  } finally {
    fs.rmSync(f.root, { recursive: true, force: true });
  }
});

await test("an unrelated loader difference refuses pairing before the entrypoint changes", () => {
  const f = fixture();
  try {
    fs.writeFileSync(path.join(f.installed, "native-targets.js"), "module.exports = [1];\n");
    assert.throws(() => stageSourceNativeEntrypoint(f.root, f.app, f.artifacts));
    assert.equal(fs.readFileSync(path.join(f.installed, "index.js"), "utf8"), published);
    assert.deepEqual(fs.readdirSync(f.artifacts), []);
  } finally {
    fs.rmSync(f.root, { recursive: true, force: true });
  }
});

await test("entrypoint drift is reported while the published bytes are still restored", () => {
  const f = fixture();
  try {
    const staged = stageSourceNativeEntrypoint(f.root, f.app, f.artifacts);
    const entry = path.join(f.installed, "index.js");
    fs.writeFileSync(entry, "unexpected entrypoint bytes\n");
    assert.throws(() => staged.restore());
    assert.equal(fs.readFileSync(entry, "utf8"), published);
    staged.restore();
  } finally {
    fs.rmSync(f.root, { recursive: true, force: true });
  }
});

await test("an external linked native package is refused without changing it or the artifacts", () => {
  const f = fixture();
  try {
    const external = path.join(f.root, "shared-native");
    fs.renameSync(f.installed, external);
    fs.symlinkSync(external, f.installed, "dir");
    assert.throws(() => stageSourceNativeEntrypoint(f.root, f.app, f.artifacts));
    assert.equal(fs.readFileSync(path.join(external, "index.js"), "utf8"), published);
    assert.deepEqual(fs.readdirSync(f.artifacts), []);
  } finally {
    fs.rmSync(f.root, { recursive: true, force: true });
  }
});
