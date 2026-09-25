import assert from "node:assert/strict";
import test from "node:test";
import type { ModuleNode } from "vite";

import type { ArtFileInfo } from "../types/art.ts";
import { createHandleHotUpdate, createLoad, type VirtualModuleState } from "./virtual.ts";

void test("art changes refresh loaded art, variant, shared setup and preview modules", async () => {
  const file = "/repo/components/MyCard.art.vue";
  const art: ArtFileInfo = {
    path: file,
    metadata: { title: "MyCard", component: "./MyCard.vue", tags: [], status: "ready" },
    variants: [{ name: "Default", template: "<MyCard />", isDefault: true, skipVrt: false }],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
  const artFiles = new Map([[file, art]]);
  const sourceModule = {} as ModuleNode;
  const moduleIds = [
    `\0musea:${file}?musea-virtual`,
    `\0musea-art:${file}?musea-virtual`,
    `\0musea-variant:${file}:Default?musea-virtual`,
    `\0musea-shared:${file}?musea-virtual`,
    `\0musea-preview:${file}:Default`,
  ];
  const moduleGraph = new Map(moduleIds.map((id) => [id, { id } as ModuleNode]));
  const unrelatedModule = {
    id: "\0musea-art:/repo/components/Other.art.vue?musea-virtual",
  } as ModuleNode;
  moduleGraph.set(unrelatedModule.id!, unrelatedModule);
  const state: VirtualModuleState = {
    basePath: "/__musea__",
    inlineArt: false,
    artFiles,
    resolvedPreviewCss: [],
    resolvedPreviewSetup: null,
    getConfigRoot: () => "/repo",
    getScanRoots: () => ["/repo"],
    getVueVersion: () => 3,
    getServer: () => ({ moduleGraph: { idToModuleMap: moduleGraph } }),
    processArtFile: async () => {
      art.variants = [
        {
          name: "Wide",
          template: "<MyCard style='width: 400px' />",
          isDefault: true,
          skipVrt: false,
        },
      ];
    },
  };

  const modules = await createHandleHotUpdate(state)({ file, modules: [sourceModule] });

  assert.deepEqual(modules, [sourceModule, ...moduleIds.map((id) => moduleGraph.get(id))]);
  assert.equal(modules?.includes(unrelatedModule), false);
  const updatedArtModule = createLoad(state)(`\0musea-art:${file}?musea-virtual`);
  assert.match(updatedArtModule!, /virtual:musea-variant:[^"\n]*:Wide/);
  assert.doesNotMatch(updatedArtModule!, /virtual:musea-variant:[^"\n]*:Default/);
});
