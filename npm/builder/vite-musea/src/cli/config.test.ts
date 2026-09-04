import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";

import { loadMuseaVrtOptions } from "./config.ts";

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

void test("missing vite config has no Musea VRT options", async () => {
  const workspace = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-vrt-config-"));

  assert.equal(await loadMuseaVrtOptions("vite.config.ts", workspace), undefined);
});
