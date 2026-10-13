import assert from "node:assert/strict";
import fs from "node:fs";
import { mkdtemp, mkdir, readFile, realpath, rm, symlink, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join, toNamespacedPath } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import {
  configModuleImportPath,
  importConfigModule,
} from "../../crates/vize_carton/src/config/loader/module-runtime.mjs";

const cases = JSON.parse(
  fs.readFileSync(
    new URL("./fixtures/native-config-module-import-paths.json", import.meta.url),
    "utf8",
  ),
);

for (const control of cases) {
  test(`the complete config import path preserves ${control.name}`, () => {
    const originalPath = control.input;
    const configURL = pathToFileURL(originalPath, { windows: control.windows });
    const originalURL = configURL.href;
    const importPath = configModuleImportPath(configURL, control.windows);
    assert.equal(importPath, control.expected);
    assert.equal(JSON.stringify(importPath), control.expectedSpecifier);
    assert.equal(configURL.href, originalURL);
  });
}

for (const [url, windows] of [
  ["file:///C:/project/vite%2Fconfig.ts", true],
  ["file:///C:/project/vite%5Cconfig.ts", true],
  ["file:///project/vite%2Fconfig.ts", false],
] as const) {
  test(`config import paths refuse encoded filesystem separators: ${url}`, () => {
    assert.throws(() => configModuleImportPath(new URL(url), windows), {
      code: "ERR_INVALID_FILE_URL_PATH",
    });
  });
}

test("the real Vite bundle graph loads a spaced Unicode quoted path without changing its source identity", async () => {
  const directory = await realpath(await mkdtemp(join(tmpdir(), "vize-config-import-path-")));
  const project = join(directory, "space 日本語 'quoted' #100%");
  const selected = join(project, "vite.config.mts");
  const original = "export default {value: 7, directory: __dirname, sourceURL: import.meta.url};";
  try {
    await mkdir(join(project, "node_modules"), { recursive: true });
    await writeFile(join(project, "package.json"), '{"type":"module"}');
    const require = createRequire(new URL("../../package.json", import.meta.url));
    await symlink(
      dirname(require.resolve("vite/package.json")),
      join(project, "node_modules", "vite"),
      "junction",
    );
    await writeFile(selected, original);
    const namespace = await importConfigModule(toNamespacedPath(selected));
    assert.deepEqual(namespace.default, {
      value: 7,
      directory: project,
      sourceURL: pathToFileURL(selected).href,
    });
    assert.equal(await readFile(selected, "utf8"), original);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
