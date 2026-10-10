import assert from "node:assert/strict";
import { cp, mkdir, readFile, readdir, realpath, symlink, writeFile } from "node:fs/promises";
import path from "node:path";
import { build, resolveConfig } from "vite";
import { repository, sha256 } from "./hosted-vrt-build.fixtures.ts";
import { emitGalleryShell } from "./static-gallery-shell.ts";
import { publicBasePathFromViteBase } from "./static-base.ts";
import type { StaticGalleryPayload } from "./static-data.ts";
import type { StaticEmitContext } from "./static-export.ts";

const oldHead = "83ee93d21ddc451dc0c2282e6127e4489643fc81";
const oldPanelGitBlob = "140dd11be23d846810991d866c255d21ec00e8ed";
const panelPath = "npm/builder/vite-musea/gallery/components/VrtPanel.vue";
const oldPanelSha256 = "beccb418fcad51f11e94d865537999239b7ce4daf25a71371f399604377781f8";

async function inventory(directory: string): Promise<Record<string, string>> {
  const files: Record<string, string> = {};
  async function visit(relative: string) {
    for (const item of await readdir(path.join(directory, relative), { withFileTypes: true })) {
      const next = path.join(relative, item.name);
      if (item.isDirectory()) await visit(next);
      else if (item.isFile()) files[next] = sha256(await readFile(path.join(directory, next)));
    }
  }
  await visit("");
  return files;
}

/** Compile authentic old Panel bytes in an isolated copy with the real production gallery config. */
export async function installOldRenderer(root: string, directory: string, output: string) {
  const source = path.join(repository, "npm/builder/vite-musea");
  const isolated = path.join(root, "old-renderer-module");
  await mkdir(isolated, { recursive: true });
  for (const name of ["gallery", "src", "package.json", "gallery-vite.config.ts"])
    await cp(path.join(source, name), path.join(isolated, name), { recursive: true });
  const sourceGallery = await inventory(path.join(source, "gallery"));
  const copiedGallery = await inventory(path.join(isolated, "gallery"));
  assert.deepEqual(copiedGallery, sourceGallery);
  const sourceModules = await inventory(path.join(source, "src"));
  assert.deepEqual(await inventory(path.join(isolated, "src")), sourceModules);
  const config = await readFile(path.join(source, "gallery-vite.config.ts"));
  assert.deepEqual(await readFile(path.join(isolated, "gallery-vite.config.ts")), config);
  const oldPanel = await readFile(
    path.join(repository, "tests/tooling/fixtures/musea/hosted-vrt-reconnect/VrtPanel.before.vue"),
  );
  assert.equal(sha256(oldPanel), oldPanelSha256);
  await writeFile(path.join(isolated, "gallery/components/VrtPanel.vue"), oldPanel);
  const withOldPanel = await inventory(path.join(isolated, "gallery"));
  assert.deepEqual(
    { ...withOldPanel, "components/VrtPanel.vue": sourceGallery["components/VrtPanel.vue"] },
    sourceGallery,
  );
  await mkdir(path.join(isolated, "node_modules"));
  const dependencyTargets: Record<string, string> = {};
  for (const entry of await readdir(path.join(source, "node_modules"))) {
    // Vite's config-loader cache is local to this owned copy, not the real dependency directory.
    if ([".vite-temp", ".vite", ".cache"].includes(entry)) continue;
    const target = await realpath(path.join(source, "node_modules", entry));
    dependencyTargets[entry] = target;
    await symlink(target, path.join(isolated, "node_modules", entry));
  }
  await build({ configFile: path.join(isolated, "gallery-vite.config.ts") });
  const oldCompiled = path.join(isolated, "dist/gallery");
  const compiledFiles = await inventory(oldCompiled);
  assert.ok(Object.hasOwn(compiledFiles, "index.html"));
  const before = await inventory(directory);
  const indexPath = path.join(directory, "gallery/index.html");
  const globals = (html: string) =>
    [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)]
      .map((match) => match[1])
      .filter((script) => script.includes("window.__MUSEA_STATIC__=true;"));
  const originalGlobals = globals(await readFile(indexPath, "utf8"));
  assert.equal(originalGlobals.length, 1);
  const manifestPath = path.join(directory, "gallery/api/static.json");
  const manifestBytes = await readFile(manifestPath);
  const payload = JSON.parse(manifestBytes.toString()) as StaticGalleryPayload;
  const ctx: StaticEmitContext = {
    config: await resolveConfig({ configFile: false, root, base: "/built/" }, "build"),
    artFiles: new Map(payload.arts.map((art) => [art.path, art])),
    scanRoots: [path.join(root, "src")],
    tokensPath: undefined,
    basePath: publicBasePathFromViteBase("/built/", "/gallery/"),
    resolvedPreviewCss: [],
    resolvedPreviewSetup: null,
    devSessionToken: "",
    themeConfig: undefined,
  };
  const assets: Array<{ type: "asset"; fileName: string; source: string | Uint8Array }> = [];
  await emitGalleryShell(
    (asset) => {
      assert.ok(asset.fileName.startsWith("gallery/"));
      assert.equal(asset.fileName.startsWith("gallery/api/"), false);
      assert.equal(asset.fileName.startsWith("gallery/preview/"), false);
      assets.push(asset);
    },
    "gallery",
    ctx,
    payload,
    oldCompiled,
  );
  for (const asset of assets) {
    const target = path.join(directory, asset.fileName);
    await mkdir(path.dirname(target), { recursive: true });
    await writeFile(target, asset.source);
  }
  const emitted = assets.map((asset) => asset.fileName);
  const after = await inventory(directory);
  assert.deepEqual(await readFile(manifestPath), manifestBytes);
  assert.deepEqual(globals(await readFile(indexPath, "utf8")), originalGlobals);
  for (const [file, digest] of Object.entries(before))
    if (!emitted.includes(file)) assert.equal(after[file], digest, file);
  const receipt = {
    sourceHead: oldHead,
    panelPath,
    panelGitBlob: oldPanelGitBlob,
    panelSha256: sha256(oldPanel),
    productionConfigSha256: sha256(config),
    sourceGallery,
    copiedGalleryWithOnlyOldPanel: withOldPanel,
    unchangedSourceModules: sourceModules,
    dependencyTargets,
    compiledFiles,
    manifestSha256: sha256(manifestBytes),
    originalStaticGlobals: originalGlobals,
    nativeFixtureBefore: before,
    nativeFixtureAfter: after,
    emitted,
    untouchedNativeManifestPreviewsAndRuntimeWhole: true,
  };
  await mkdir(output, { recursive: true });
  await writeFile(
    path.join(output, "old-renderer-source-custody.json"),
    JSON.stringify(receipt, null, 2),
  );
  await writeFile(path.join(output, "old-VrtPanel.vue"), oldPanel);
  await writeFile(path.join(output, "production-gallery-vite.config.ts"), config);
  await cp(oldCompiled, path.join(output, "old-compiled-gallery"), { recursive: true });
  return receipt;
}
