import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";

import { loadMuseaConfiguration, loadMuseaFileSet, loadMuseaVrtOptions } from "./config.ts";

void test("loads Musea VRT options from vite config plugin options", async () => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-config-"));
  const pluginOptionsUrl = pathToFileURL(path.resolve("src/plugin/options.ts")).href;
  const configPath = path.join(workspace, "vite.config.ts");

  await fs.promises.writeFile(
    configPath,
    `
      import { attachMuseaOptions } from ${JSON.stringify(pluginOptionsUrl)};

      export default {
        plugins: [
          attachMuseaOptions(
            { name: "vite-plugin-musea" },
            {
              vrt: {
                threshold: 0,
                viewports: [{ width: 320, height: 240, name: "tiny" }],
                capture: { settleTime: 250, waitForNetwork: false },
                comparison: { antiAliasing: false },
              },
            },
          ),
        ],
      };
    `,
  );

  assert.deepEqual(await loadMuseaVrtOptions(configPath, workspace), {
    threshold: 0,
    viewports: [{ width: 320, height: 240, name: "tiny" }],
    capture: { settleTime: 250, waitForNetwork: false },
    comparison: { antiAliasing: false },
  });
});

void test("ignores vite plugins without Musea VRT options", async () => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-config-"));
  const configPath = path.join(workspace, "vite.config.ts");

  await fs.promises.writeFile(
    configPath,
    `
      export default {
        plugins: [
          {
            name: "other-plugin",
            vrt: {
              threshold: 0,
            },
          },
        ],
      };
    `,
  );

  assert.equal(await loadMuseaVrtOptions(configPath, workspace), undefined);
});

void test("loads Musea include and exclude for the VRT file scan", async () => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-files-"));
  const pluginOptionsUrl = pathToFileURL(path.resolve("src/plugin/options.ts")).href;
  const configPath = path.join(workspace, "vite.config.ts");

  await fs.promises.writeFile(
    configPath,
    `
      import { attachMuseaOptions } from ${JSON.stringify(pluginOptionsUrl)};

      export default {
        root: "gallery",
        plugins: [
          attachMuseaOptions(
            { name: "vite-plugin-musea" },
            {
              include: ["../*/src/**/*.art.vue"],
              exclude: ["**/legacy/**"],
            },
          ),
        ],
      };
    `,
  );

  assert.deepEqual(await loadMuseaFileSet(configPath, workspace), {
    root: path.join(workspace, "gallery"),
    projectRoot: path.join(workspace, "gallery"),
    include: ["../*/src/**/*.art.vue"],
    exclude: ["**/legacy/**"],
  });
});

void test("missing vite config scans the working directory with the gallery defaults", async () => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-files-"));

  assert.deepEqual(await loadMuseaFileSet("vite.config.ts", workspace), {
    root: workspace,
    projectRoot: workspace,
    include: ["**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
  });
});

void test("missing vite config has no Musea VRT options", async () => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-config-"));

  assert.equal(await loadMuseaVrtOptions("vite.config.ts", workspace), undefined);
});

void test("VRT and scanning use top-level vize.musea without a dedicated config", async (t) => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vite-top-level-"));
  t.after(() => fs.promises.rm(workspace, { force: true, recursive: true }));
  await fs.promises.writeFile(
    path.join(workspace, "vite.config.mjs"),
    `export default {
      root: "gallery",
      vize: { musea: {
        include: ["../stories/**/*.art.vue"],
        exclude: ["**/legacy/**"],
        vrt: { threshold: 0, outDir: ".gallery-snapshots", viewports: [{ width: 320, height: 240 }] },
      } },
    };`,
  );

  assert.deepEqual(await loadMuseaFileSet("vite.config.mjs", workspace), {
    root: path.join(workspace, "gallery"),
    include: ["../stories/**/*.art.vue"],
    exclude: ["**/legacy/**"],
  });
  assert.deepEqual(await loadMuseaVrtOptions("vite.config.mjs", workspace), {
    threshold: 0,
    snapshotDir: ".gallery-snapshots",
    viewports: [{ width: 320, height: 240 }],
  });
  assert.equal(
    (await fs.promises.readdir(workspace)).some((name) => name.startsWith("vize.config.")),
    false,
  );
});

void test("Musea plugin options override individual shared options", async (t) => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-plugin-shared-"));
  t.after(() => fs.promises.rm(workspace, { force: true, recursive: true }));
  const pluginOptionsUrl = pathToFileURL(path.resolve("src/plugin/options.ts")).href;
  await fs.promises.writeFile(
    path.join(workspace, "vite.config.ts"),
    `import { attachMuseaOptions } from ${JSON.stringify(pluginOptionsUrl)};
    export default {
      vize: { musea: { exclude: ["**/legacy/**"], vrt: { threshold: 3, outDir: ".shared" } } },
      plugins: [attachMuseaOptions({ name: "vite-plugin-musea" }, {
        include: ["stories/**/*.art.vue"], vrt: { threshold: 0 },
      })],
    };`,
  );

  assert.deepEqual(await loadMuseaFileSet("vite.config.ts", workspace), {
    root: workspace,
    include: ["stories/**/*.art.vue"],
    exclude: ["**/legacy/**"],
  });
  assert.deepEqual(await loadMuseaVrtOptions("vite.config.ts", workspace), {
    threshold: 0,
    snapshotDir: ".shared",
  });
});

void test("dedicated JSON config remains higher priority than Vite shared settings", async (t) => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-dedicated-priority-"));
  t.after(() => fs.promises.rm(workspace, { force: true, recursive: true }));
  await fs.promises.writeFile(
    path.join(workspace, "vite.config.mjs"),
    'export default { vize: { musea: { include: ["wrong/**/*.art.vue"], vrt: { threshold: 9 } } } };',
  );
  await fs.promises.writeFile(
    path.join(workspace, "vize.config.json"),
    JSON.stringify({ musea: { include: ["stories/**/*.art.vue"], vrt: { threshold: 0 } } }),
  );

  assert.deepEqual(await loadMuseaFileSet("vite.config.mjs", workspace), {
    root: workspace,
    include: ["stories/**/*.art.vue"],
    exclude: ["node_modules/**", "dist/**"],
  });
  assert.deepEqual(await loadMuseaVrtOptions("vite.config.mjs", workspace), { threshold: 0 });
});

void test("a custom Vite config is evaluated once and its vize settings are projected", async (t) => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-custom-vite-config-"));
  t.after(() => fs.promises.rm(workspace, { force: true, recursive: true }));
  const marker = path.join(workspace, "evaluated.txt");
  const configPath = path.join(workspace, "gallery.config.mjs");
  await fs.promises.writeFile(
    configPath,
    `
    import { appendFileSync } from "node:fs";
    appendFileSync(${JSON.stringify(marker)}, "evaluation\\n");
    export default ({ command }) => ({
      vize: { musea: { include: [command + "/**/*.art.vue"] } },
    });
  `,
  );

  assert.deepEqual(await loadMuseaConfiguration(configPath, workspace), {
    fileSet: {
      root: workspace,
      include: ["serve/**/*.art.vue"],
      exclude: ["node_modules/**", "dist/**"],
    },
    vrt: undefined,
    configDir: workspace,
  });
  assert.equal(await fs.promises.readFile(marker, "utf8"), "evaluation\n");
});

void test("automatic Vite config discovery supports mjs and its project root", async (t) => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-auto-vite-config-"));
  t.after(() => fs.promises.rm(workspace, { force: true, recursive: true }));
  await fs.promises.writeFile(
    path.join(workspace, "vite.config.mjs"),
    `
    export default { root: "gallery", vize: { musea: { include: ["stories/**/*.art.vue"] } } };
  `,
  );
  assert.deepEqual(await loadMuseaConfiguration(undefined, workspace), {
    fileSet: {
      root: path.join(workspace, "gallery"),
      include: ["stories/**/*.art.vue"],
      exclude: ["node_modules/**", "dist/**"],
    },
    vrt: undefined,
    configDir: workspace,
  });
});
