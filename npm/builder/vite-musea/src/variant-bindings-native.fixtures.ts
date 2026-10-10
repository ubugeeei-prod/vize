import { cp, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { inspect } from "node:util";
import { build } from "vite";
import vize from "../../vite/src/index.ts";
import { musea } from "./plugin/index.ts";
import { repository, sha256 } from "./inline-art-static.fixtures.ts";
import type { StaticGalleryPayload } from "./static-data.ts";

export const bindingFixture = path.join(
  repository,
  "tests/tooling/fixtures/musea/variant-binding-collision",
);

/** The original authored red and ordinary control use the actual source-native pipeline. */
export async function buildNativeBindingGallery(output: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-native-bindings-"));
  await mkdir(output, { recursive: true });
  try {
    await stat(path.join(repository, "npm/builder/vite-musea/dist/gallery/index.html"));
    const inputs = [];
    for (const kind of ["collision", "ordinary"]) {
      await mkdir(path.join(root, "src", kind), { recursive: true });
      for (const file of ["Host.vue", "Host.art.vue"]) {
        const original = path.join(bindingFixture, kind === "ordinary" ? "ordinary" : "", file);
        const target = path.join(root, "src", kind, file);
        await cp(original, target);
        const bytes = await readFile(target);
        inputs.push({
          file: `src/${kind}/${file}`,
          sha256: sha256(bytes),
          source: bytes.toString(),
        });
      }
    }
    await writeFile(path.join(output, "inputs.json"), JSON.stringify(inputs, null, 2));
    const directory = path.join(root, "dist");
    await build({
      root,
      configFile: false,
      base: "/built/",
      logLevel: "warn",
      cacheDir: path.join(root, ".vite-cache"),
      resolve: {
        alias: { vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")) },
      },
      plugins: [vize(), musea({ include: ["src/**/*.art.vue"], basePath: "/gallery/" })],
      build: { outDir: directory, emptyOutDir: true, minify: true },
    });
    const bytes = await readFile(path.join(directory, "gallery/api/static.json"));
    const manifest = JSON.parse(bytes.toString()) as StaticGalleryPayload;
    await writeFile(path.join(output, "static.json"), bytes);
    return { root, directory, manifest };
  } catch (error) {
    await writeFile(
      path.join(output, "build-failure.json"),
      JSON.stringify(
        {
          message: String(error),
          stack: error instanceof Error ? error.stack : undefined,
          details: inspect(error, { depth: null, maxArrayLength: null, maxStringLength: null }),
        },
        null,
        2,
      ),
    );
    await rm(root, { recursive: true, force: true });
    throw error;
  }
}
