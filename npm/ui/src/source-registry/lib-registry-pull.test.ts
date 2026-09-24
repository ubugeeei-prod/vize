import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";

import { test } from "vite-plus/test";

import { composableLibRegistryOptions } from "../../../compose/core/scripts/build-source-registry.ts";
import { uiLibRegistryOptions } from "../../scripts/build-source-registry.ts";
import { buildLibRegistry, sha256Hex } from "../../scripts/source-registry-bundle/bundle.ts";
import type {
  LibRegistryBundle,
  LibRegistryItem,
} from "../../scripts/source-registry-bundle/types.ts";
import {
  createPackageLibRegistryInput,
  writeLibRegistry,
} from "../../scripts/source-registry-bundle/write.ts";

const packageRoot = path.resolve(import.meta.dirname, "../..");

/**
 * Copy an item and its registry closure out of a *written* registry exactly
 * as `vize lib pull` lays it out: `<dir>/<files[].path>`, digests verified.
 */
function pull(registryRoot: string, bundle: LibRegistryBundle, name: string, dir: string): void {
  const items = new Map(bundle.manifest.items.map((item) => [item.name, item]));
  const root = items.get(name);
  assert.ok(root, `${name} is published`);
  const closure: LibRegistryItem[] = [root];
  for (const dependency of root.registryDependencies) {
    const item = items.get(dependency);
    assert.ok(item, `${name} depends on published ${dependency}`);
    closure.push(item);
  }
  for (const item of closure) {
    for (const file of item.files) {
      const bytes = readFileSync(path.join(registryRoot, "registry/files", file.path));
      assert.equal(sha256Hex(bytes), file.sha256);
      const target = path.join(dir, file.path);
      mkdirSync(path.dirname(target), { recursive: true });
      writeFileSync(target, bytes);
    }
  }
}

test(
  "pulled switch and use-storage type-check on their own with vue-tsc",
  { timeout: 120_000 },
  () => {
    // Inside the package so `vue` resolves from its node_modules like it
    // would in a consumer project; removed afterwards.
    const project = mkdtempSync(path.join(packageRoot, ".lib-pull-e2e-"));
    try {
      const ui = buildLibRegistry(createPackageLibRegistryInput(uiLibRegistryOptions()));
      const composable = buildLibRegistry(
        createPackageLibRegistryInput(composableLibRegistryOptions()),
      );
      writeLibRegistry(path.join(project, "ui-package"), ui);
      writeLibRegistry(path.join(project, "composable-package"), composable);

      pull(
        path.join(project, "ui-package"),
        ui,
        "switch",
        path.join(project, "src/components/vize"),
      );
      pull(
        path.join(project, "composable-package"),
        composable,
        "use-storage",
        path.join(project, "src/composables/vize"),
      );
      writeFileSync(
        path.join(project, "src/app.ts"),
        [
          'import { Switch } from "./components/vize/families/selection/switch/switch.ts";',
          'import { useStorage } from "./composables/vize/use-storage.ts";',
          "export const pulled = [Switch, useStorage] as const;",
          "",
        ].join("\n"),
      );
      writeFileSync(
        path.join(project, "tsconfig.json"),
        JSON.stringify({
          compilerOptions: {
            target: "ES2022",
            module: "ESNext",
            moduleResolution: "Bundler",
            allowImportingTsExtensions: true,
            lib: ["ES2022", "ESNext.Intl", "DOM", "DOM.Iterable"],
            types: [],
            strict: true,
            noUncheckedIndexedAccess: true,
            exactOptionalPropertyTypes: true,
            skipLibCheck: true,
            noEmit: true,
          },
          include: ["src/**/*.ts", "src/**/*.vue"],
        }),
      );

      const result = spawnSync(
        path.join(packageRoot, "node_modules/.bin/vue-tsc"),
        ["--noEmit", "-p", path.join(project, "tsconfig.json")],
        { cwd: project, encoding: "utf8" },
      );
      assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    } finally {
      rmSync(project, { recursive: true, force: true });
    }
  },
);
