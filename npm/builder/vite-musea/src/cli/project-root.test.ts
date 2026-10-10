import assert from "node:assert/strict";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import test from "node:test";
import { loadMuseaFileSet } from "./config.ts";

void test("CLI resolves explicit Musea projectRoot against the configured Vite root", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-project-root-"));
  const optionsUrl = pathToFileURL(path.resolve("src/plugin/options.ts")).href;
  try {
    await writeFile(
      path.join(root, "vize.config.mjs"),
      'export default { musea: { projectRoot: ".." } };',
    );
    await writeFile(path.join(root, "vite.config.mjs"), 'export default { root: "gallery" };');
    assert.equal(
      (await loadMuseaFileSet("vite.config.mjs", root)).projectRoot,
      path.join(root, "gallery"),
    );
    await writeFile(
      path.join(root, "vite.config.mjs"),
      `import { attachMuseaOptions } from ${JSON.stringify(optionsUrl)}; export default { root:"gallery", plugins:[attachMuseaOptions({name:"vite-plugin-musea"},{include:["custom/**/*.art.vue"]})] };`,
    );
    assert.equal(
      (await loadMuseaFileSet("vite.config.mjs", root)).projectRoot,
      path.join(root, "gallery"),
    );
    await writeFile(
      path.join(root, "vite.config.mjs"),
      `import { attachMuseaOptions } from ${JSON.stringify(optionsUrl)}; export default { root:"gallery", plugins:[attachMuseaOptions({name:"vite-plugin-musea"},{projectRoot:"../explicit"})] };`,
    );
    assert.equal(
      (await loadMuseaFileSet("vite.config.mjs", root)).projectRoot,
      path.join(root, "explicit"),
    );
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
