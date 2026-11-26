import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test, { type TestContext } from "node:test";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const pluginSource = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

interface Provider {
  loadNative(projectRoot: string, nativePath?: string): Record<string, unknown>;
}

/** The plugin as tsserver loads it from a location shared by every project. */
const sharedPlugin = require(path.join(pluginSource, "provider.cjs")) as Provider;

function project(t: TestContext): string {
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-native-path-")));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  return directory;
}

/** Where the plugin sits decides which projects it serves, so tests place real copies. */
function pluginCopy(directory: string): string {
  fs.mkdirSync(directory, { recursive: true });
  for (const name of ["index.cjs", "provider.cjs", "package.json"])
    fs.copyFileSync(path.join(pluginSource, name), path.join(directory, name));
  return directory;
}

/** The plugin as a project's own dependency. */
function installedPlugin(root: string): Provider {
  const directory = pluginCopy(path.join(root, "node_modules", "@vizejs", "typescript-vue-plugin"));
  return require(path.join(directory, "provider.cjs")) as Provider;
}

/** A module directory whose entry records that it ran. */
function recordingModule(directory: string, name: string | undefined): string {
  fs.mkdirSync(directory, { recursive: true });
  const marker = path.join(directory, "loaded.txt");
  fs.writeFileSync(
    path.join(directory, "index.js"),
    `require("node:fs").writeFileSync(${JSON.stringify(marker)}, "loaded");\n` +
      "exports.generateVueLanguageCode = () => '{}';\n",
  );
  fs.writeFileSync(
    path.join(directory, "language-projection.js"),
    "exports.toUtf16LanguageProjection = exports.applyDiagnosticDirectives =\n" +
      "  exports.mapLanguageDiagnosticRange = () => undefined;\n",
  );
  if (name !== undefined)
    fs.writeFileSync(
      path.join(directory, "package.json"),
      JSON.stringify({ name, version: "0.0.0", main: "index.js" }),
    );
  return marker;
}

const outsideProject =
  "nativePath requires the project to resolve this @vizejs/typescript-vue-plugin";

void test("a plugin the project does not resolve does not load the project's nativePath", (t) => {
  const root = project(t);
  const marker = recordingModule(path.join(root, "native"), "@vizejs/native");
  assert.deepEqual(sharedPlugin.loadNative(root, "./native"), { error: outsideProject });
  assert.deepEqual(sharedPlugin.loadNative(root, path.join(root, "native", "index.js")), {
    error: outsideProject,
  });
  assert.equal(fs.existsSync(marker), false);
});

void test("a plugin installed inside the project loads the project's nativePath", (t) => {
  const root = project(t);
  const marker = recordingModule(path.join(root, "native"), "@vizejs/native");
  const provider = installedPlugin(root).loadNative(root, "./native");
  assert.deepEqual(
    { origin: provider.origin, path: provider.path, version: provider.version },
    { origin: "explicit", path: path.join(root, "native", "index.js"), version: "0.0.0" },
  );
  assert.equal(provider.error, undefined);
  assert.equal(fs.readFileSync(marker, "utf8"), "loaded");
});

void test("a plugin hoisted to the workspace root or linked into it is the project's own", (t) => {
  const root = project(t);
  const app = path.join(root, "packages", "app");
  const marker = recordingModule(path.join(app, "native"), "@vizejs/native");
  const hoisted = installedPlugin(root).loadNative(app, "./native");
  assert.deepEqual(
    { origin: hoisted.origin, error: hoisted.error },
    { origin: "explicit", error: undefined },
  );
  assert.equal(fs.readFileSync(marker, "utf8"), "loaded");

  const linkedRoot = project(t);
  const store = pluginCopy(path.join(project(t), "typescript-vue-plugin"));
  fs.mkdirSync(path.join(linkedRoot, "node_modules", "@vizejs"), { recursive: true });
  fs.symlinkSync(store, path.join(linkedRoot, "node_modules", "@vizejs", "typescript-vue-plugin"));
  recordingModule(path.join(linkedRoot, "native"), "@vizejs/native");
  const linkedPlugin = require(path.join(store, "provider.cjs")) as Provider;
  assert.equal(linkedPlugin.loadNative(linkedRoot, "./native").origin, "explicit");
  // The same copy, opened for a project that does not resolve it.
  assert.deepEqual(linkedPlugin.loadNative(root, "./packages/app/native"), {
    error: outsideProject,
  });
});

void test("a nativePath that is not the native package is never loaded", (t) => {
  const root = project(t);
  const plugin = installedPlugin(root);
  const other = recordingModule(path.join(root, "other"), "another-package");
  assert.deepEqual(plugin.loadNative(root, "./other"), {
    error: "nativePath must identify @vizejs/native",
  });
  assert.equal(fs.existsSync(other), false);

  const bare = recordingModule(path.join(root, "bare"), undefined);
  assert.equal(typeof plugin.loadNative(root, "./bare/index.js").error, "string");
  assert.equal(fs.existsSync(bare), false);

  // A directory named like the manifest is not a manifest.
  const disguised = recordingModule(path.join(root, "disguised"), undefined);
  const manifestMarker = recordingModule(path.join(root, "disguised", "package.json"), undefined);
  assert.equal(typeof plugin.loadNative(root, "./disguised/index.js").error, "string");
  assert.equal(fs.existsSync(disguised), false);
  assert.equal(fs.existsSync(manifestMarker), false);
});
