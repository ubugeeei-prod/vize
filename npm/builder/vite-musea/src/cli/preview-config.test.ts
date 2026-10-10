import assert from "node:assert/strict";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import test from "node:test";
import { loadMuseaPreviewBasePath } from "./config.ts";

void test("VRT resolves Vite base and explicit Musea route ahead of shared config", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-preview-route-"));
  const optionsUrl = pathToFileURL(path.resolve("src/plugin/options.ts")).href;
  try {
    assert.equal(await loadMuseaPreviewBasePath("missing.mjs", root), "/__musea__");
    await writeFile(
      path.join(root, "vize.config.mjs"),
      'export default { musea: { basePath: "/shared" } };',
    );
    await writeFile(path.join(root, "vite.config.mjs"), 'export default { base: "/site/" };');
    assert.equal(await loadMuseaPreviewBasePath("vite.config.mjs", root), "/site/shared");
    await writeFile(
      path.join(root, "vite.config.mjs"),
      `import { attachMuseaOptions } from ${JSON.stringify(optionsUrl)};
      export default { base: "/site/", plugins: [attachMuseaOptions({name:"vite-plugin-musea"}, { basePath: "/explicit" })] };`,
    );
    assert.equal(await loadMuseaPreviewBasePath("vite.config.mjs", root), "/site/explicit");
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
