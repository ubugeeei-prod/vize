import assert from "node:assert/strict";
import { mkdir, mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test, { type TestContext } from "node:test";
import { resolveConfig } from "vite";

import { musea } from "./plugin/index.ts";
import { processMuseaArtFile } from "./plugin/art-processing.ts";
import { loadMuseaArtFiles } from "./test-runner.ts";
import { resolveScanRoots } from "./utils.ts";

async function fixture(t: TestContext) {
  const workspace = await realpath(await mkdtemp(path.join(tmpdir(), "musea-external-art-")));
  t.after(() => rm(workspace, { recursive: true, force: true }));
  const root = path.join(workspace, "ui-kit");
  const demo = path.join(workspace, "data-grid/src/__demo__");
  await mkdir(root, { recursive: true });
  await mkdir(demo, { recursive: true });
  const artPath = path.join(demo, "GridDemo.art.vue");
  const source = (component: string) => `<script setup lang="ts">
import GridDemoPage from '${component}'
defineArt(GridDemoPage, { title: 'Grid demo' })
</script>
<art><variant name="Default" default><GridDemoPage /></variant></art>`;
  await writeFile(artPath, source("./GridDemoPage.vue"));
  await writeFile(path.join(demo, "GridDemoPage.vue"), "<template><div>Grid demo</div></template>");
  return { workspace, root, artPath, source, include: ["../data-grid/src/**/*.art.vue"] };
}

void test("art processing resolves components within explicitly included sibling roots", async (t) => {
  const { root, artPath, include } = await fixture(t);
  const art = await processMuseaArtFile(artPath, {
    root,
    scanRoots: resolveScanRoots(root, include),
    command: "build",
  });
  assert.equal(art?.path, artPath);
  assert.equal(art?.metadata.component, "./GridDemoPage.vue");
  assert.equal(art?.metadata.title, "Grid demo");
  assert.deepEqual(
    art?.variants.map((variant) => variant.name),
    ["Default"],
  );
});

void test("the interaction loader shares the include roots with component resolution", async (t) => {
  const { root, artPath, include } = await fixture(t);
  const arts = await loadMuseaArtFiles({ root, include });
  assert.deepEqual(
    arts.map((art) => art.path),
    [artPath],
  );
  assert.equal(arts[0]?.metadata.component, "./GridDemoPage.vue");
});

void test("the Vite build scans sibling arts without dropping their target component", async (t) => {
  const { root, artPath, include } = await fixture(t);
  const config = await resolveConfig(
    {
      root,
      configFile: false,
      logLevel: "silent",
      plugins: [musea({ include, projectRoot: ".." })],
    },
    "build",
  );
  const plugin = config.plugins.find((plugin) => plugin.name === "vite-plugin-musea");
  assert.ok(plugin);
  const hook = plugin.buildStart;
  assert.equal(typeof hook, "function");
  await (hook as () => Promise<void>)();
  const load = plugin.load;
  assert.equal(typeof load, "function");
  const loaded = await (load as (id: string) => Promise<unknown>)(
    `\0musea-art:${artPath}?musea-virtual`,
  );
  assert.equal(typeof loaded, "string");
  assert.match(loaded as string, /GridDemoPage/);
});

void test("including a sibling art does not trust components beyond its scan roots", async (t) => {
  const { workspace, root, artPath, include, source } = await fixture(t);
  await writeFile(path.join(workspace, "Outside.vue"), "<template><div /></template>");
  await writeFile(artPath, source("../../../Outside.vue"));
  await assert.rejects(
    loadMuseaArtFiles({ root, include }),
    /component path escapes the allowed directory/,
  );
});
