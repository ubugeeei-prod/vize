import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { resolveComponentSourcePath } from "./component-source.ts";
import { generateArtModule } from "./art-module.ts";
import { resolveArtComponent } from "./art-component.ts";
import { buildVariantSfcSource } from "./art-variant-sfc.ts";
import { HttpError } from "./http-error.ts";
import { processMuseaArtFile } from "./plugin/art-processing.ts";
import type { ArtFileInfo } from "./types/index.ts";

function createArt(pathname: string, component: string): ArtFileInfo {
  return {
    path: pathname,
    metadata: { title: "Card", component, tags: [], status: "ready" },
    variants: [],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
}

void test("resolveComponentSourcePath rejects a .vue symlink whose realpath is not a Vue file", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-component-source-"));
  const artPath = path.join(tempDir, "Card.art.vue");
  const decoy = path.join(tempDir, "Evil.vue");
  const secret = path.join(tempDir, ".env");

  try {
    await fs.promises.writeFile(secret, "SECRET=1\n");
    await fs.promises.symlink(secret, decoy);
    await fs.promises.writeFile(artPath, "");

    assert.throws(
      () => resolveComponentSourcePath(createArt(artPath, "./Evil.vue"), artPath, [tempDir]),
      (error: unknown) =>
        error instanceof HttpError &&
        error.status === 400 &&
        error.message === "component path must be a .vue, .ts, .tsx, .js, or .jsx file",
    );
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("resolveComponentSourcePath rejects script paths whose realpath is not a component file", async () => {
  const root = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-script-source-"));
  const artPath = path.join(root, "Card.art.vue");
  const decoy = path.join(root, "Secret.ts");
  const secret = path.join(root, ".env");

  try {
    await fs.promises.writeFile(artPath, "");
    await fs.promises.writeFile(secret, "SECRET=1\n");
    await fs.promises.symlink(secret, decoy);
    assert.throws(
      () => resolveComponentSourcePath(createArt(artPath, "./Secret.ts"), artPath, [root]),
      (error: unknown) => error instanceof HttpError && error.status === 400,
    );
  } finally {
    await fs.promises.rm(root, { recursive: true, force: true });
  }
});

void test("defineArt accepts a render-function module and wires it into the art and variant", async () => {
  const root = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-ts-art-"));
  const artPath = path.join(root, "my-button.art.vue");
  const componentPath = path.join(root, "my-button.ts");
  try {
    await fs.promises.writeFile(
      artPath,
      `<script setup lang="ts">defineArt("./my-button.ts", { title: "MyButton" });</script>
<art><variant name="Default" default><MyButton label="Hello" /></variant></art>`,
    );
    await fs.promises.writeFile(componentPath, "export default {};");
    const art = await processMuseaArtFile(artPath, { root, command: "build" });
    assert.ok(art);
    assert.equal(art.metadata.component, "./my-button.ts");
    assert.equal(art.variants.length, 1);
    assert.equal(resolveComponentSourcePath(art, artPath, [root]), componentPath);

    const component = resolveArtComponent(
      art,
      artPath,
      { defineArtComponentName: "MyButton", defineArtComponentSource: "./my-button.ts" },
      { root },
    );
    assert.equal(component.componentImportPath, componentPath);
    const artModule = generateArtModule(art, artPath, { root });
    assert.match(artModule, /import MyButton from .*my-button\.ts/);
    assert.match(artModule, /export const __component__ = MyButton/);
    const variant = buildVariantSfcSource(art, art.variants[0].template, "Default", {
      artFilePath: artPath,
      componentImportPath: component.componentImportPath,
      componentBindingName: component.componentBindingName,
    });
    assert.match(variant, /import MyButton from .*my-button\.ts/);
    assert.match(variant, /<MyButton label="Hello"/);
  } finally {
    await fs.promises.rm(root, { recursive: true, force: true });
  }
});
