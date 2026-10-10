// Filesystem-only locator law. The authored package is inert and grants no registry/native credit.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import { installedBarePlugin } from "./support/oxlint-installed-html-plugin.ts";

test("the actual Node package locator admits an import-only ancestor without requiring or loading it", (t) => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "html-esm-locator-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const directory = path.join(root, "node_modules/oxlint-plugin-vize");
  const workspace = path.join(root, "inputs/repo");
  const entry = path.join(directory, "dist/index.mjs");
  const marker = path.join(root, "package-loaded");
  fs.mkdirSync(path.dirname(entry), { recursive: true });
  fs.mkdirSync(workspace, { recursive: true });
  fs.writeFileSync(path.join(workspace, "package.json"), '{"private":true,"type":"module"}\n');
  fs.writeFileSync(
    path.join(directory, "package.json"),
    JSON.stringify({
      name: "oxlint-plugin-vize",
      type: "module",
      exports: { ".": { import: "./dist/index.mjs", types: "./dist/index.d.mts" } },
    }),
  );
  fs.writeFileSync(
    entry,
    `import fs from 'node:fs'; fs.writeFileSync(${JSON.stringify(marker)}, 'unwanted startup'); throw Error('inert package cannot execute');\n`,
  );
  assert.throws(
    () => createRequire(path.join(workspace, "package.json")).resolve("oxlint-plugin-vize"),
    { code: "ERR_PACKAGE_PATH_NOT_EXPORTED" },
  );
  assert.equal(installedBarePlugin(workspace, root, directory, entry), entry);
  assert.equal(fs.existsSync(marker), false);
  fs.mkdirSync(path.join(workspace, "node_modules/oxlint-plugin-vize"), { recursive: true });
  fs.writeFileSync(
    path.join(workspace, "node_modules/oxlint-plugin-vize/package.json"),
    fs.readFileSync(path.join(directory, "package.json")),
  );
  assert.throws(() => installedBarePlugin(workspace, root, directory, entry));
  assert.equal(fs.existsSync(marker), false);
});
