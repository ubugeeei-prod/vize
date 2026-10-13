import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { cp, mkdir, readFile, readdir, realpath, symlink, writeFile } from "node:fs/promises";
import path from "node:path";
import { build, resolveConfig } from "vite";
import { repository } from "./literal-variant-browser.fixture.ts";
import { emitGalleryShell } from "./static-gallery-shell.ts";
import { publicBasePathFromViteBase } from "./static-base.ts";
import type { StaticGalleryPayload } from "./static-data.ts";
import type { StaticEmitContext } from "./static-export.ts";

export const digest = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");

export async function inventory(directory: string): Promise<Record<string, string>> {
  const files: Record<string, string> = {};
  async function visit(relative: string) {
    for (const item of await readdir(path.join(directory, relative), { withFileTypes: true })) {
      const next = path.join(relative, item.name);
      if (item.isDirectory()) await visit(next);
      else if (item.isFile()) files[next] = digest(await readFile(path.join(directory, next)));
    }
  }
  await visit("");
  return files;
}

// Compile the real complete gallery with one explicit producer perturbation. Original
// native preview modules, payload and source checkout remain byte-exact in isolated sites.
export async function mutantSite(root: string, repairedSite: string, kind: "callback" | "smooth") {
  const source = path.join(repository, "npm/builder/vite-musea");
  const isolated = path.join(root, `mutation-${kind}`);
  await mkdir(isolated);
  for (const name of ["gallery", "src", "package.json", "gallery-vite.config.ts"])
    await cp(path.join(source, name), path.join(isolated, name), { recursive: true });
  const originalGallery = await inventory(path.join(source, "gallery"));
  assert.deepEqual(await inventory(path.join(isolated, "gallery")), originalGallery);
  const modules = await inventory(path.join(source, "src"));
  assert.deepEqual(await inventory(path.join(isolated, "src")), modules);
  const producerRelative = "utils/variantNavigation.ts";
  const producerPath = path.join(isolated, "gallery", producerRelative);
  const before = await readFile(producerPath, "utf8");
  const needle =
    kind === "callback"
      ? "      if (generation !== variantObserverGeneration) return;\n\n"
      : 'behavior: "instant"';
  assert.equal(before.split(needle).length - 1, kind === "callback" ? 1 : 2);
  const after = before.replaceAll(needle, kind === "callback" ? "" : 'behavior: "smooth"');
  await writeFile(producerPath, after);
  const mutatedGallery = await inventory(path.join(isolated, "gallery"));
  assert.deepEqual(
    { ...mutatedGallery, [producerRelative]: originalGallery[producerRelative] },
    originalGallery,
  );
  const config = await readFile(path.join(source, "gallery-vite.config.ts"));
  assert.deepEqual(await readFile(path.join(isolated, "gallery-vite.config.ts")), config);
  await mkdir(path.join(isolated, "node_modules"));
  const dependencies: Record<string, string> = {};
  for (const entry of await readdir(path.join(source, "node_modules"))) {
    if ([".vite-temp", ".vite", ".cache"].includes(entry)) continue;
    const target = await realpath(path.join(source, "node_modules", entry));
    dependencies[entry] = target;
    await symlink(target, path.join(isolated, "node_modules", entry));
  }
  await build({ configFile: path.join(isolated, "gallery-vite.config.ts") });
  const site = path.join(root, `site-${kind}`);
  await cp(repairedSite, site, { recursive: true });
  const siteBefore = await inventory(site);
  const payloadBytes = await readFile(path.join(site, "__musea__/api/static.json"));
  const payload = JSON.parse(payloadBytes.toString()) as StaticGalleryPayload;
  const ctx: StaticEmitContext = {
    config: await resolveConfig({ configFile: false, root }, "build"),
    artFiles: new Map(payload.arts.map((art) => [art.path, art])),
    scanRoots: [path.join(root, "src")],
    tokensPath: undefined,
    basePath: publicBasePathFromViteBase("/", "/__musea__/"),
    resolvedPreviewCss: [],
    resolvedPreviewSetup: null,
    devSessionToken: "",
    themeConfig: undefined,
  };
  const assets: Array<{ fileName: string; source: string | Uint8Array }> = [];
  const compiled = path.join(isolated, "dist/gallery");
  await emitGalleryShell(
    (asset) => {
      assert.ok(asset.fileName.startsWith("__musea__/"));
      assert.ok(!asset.fileName.startsWith("__musea__/api/"));
      assert.ok(!asset.fileName.startsWith("__musea__/preview/"));
      assets.push(asset);
    },
    "__musea__",
    ctx,
    payload,
    compiled,
  );
  for (const asset of assets) {
    await mkdir(path.dirname(path.join(site, asset.fileName)), { recursive: true });
    await writeFile(path.join(site, asset.fileName), asset.source);
  }
  const emitted = assets.map((asset) => asset.fileName);
  const siteAfter = await inventory(site);
  assert.deepEqual(await readFile(path.join(site, "__musea__/api/static.json")), payloadBytes);
  for (const [name, hash] of Object.entries(siteBefore))
    if (!emitted.includes(name)) assert.equal(siteAfter[name], hash, name);
  return {
    site,
    receipt: {
      kind,
      producerRelative,
      originalGallery,
      mutatedGallery,
      sourceModules: modules,
      dependencies,
      configSha256: digest(config),
      producerBefore: digest(before),
      producerAfter: digest(after),
      compiled: await inventory(compiled),
      siteBefore,
      siteAfter,
      emitted,
      payloadSha256: digest(payloadBytes),
    },
  };
}
