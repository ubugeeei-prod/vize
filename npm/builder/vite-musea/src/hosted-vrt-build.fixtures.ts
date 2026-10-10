import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { cp, mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "vite";
import vize from "../../vite/src/index.ts";
import { musea } from "./plugin/index.ts";

export const repository = fileURLToPath(new URL("../../../../", import.meta.url));
export const sha256 = (bytes: string | Uint8Array) =>
  createHash("sha256").update(bytes).digest("hex");

/** No native/compiler/preview substitution: use the same authored corpus in the actual build. */
export async function buildServiceGallery(root: string, output: string, changed = false) {
  await stat(path.join(repository, "npm/builder/vite-musea/dist/gallery/index.html"));
  await mkdir(path.join(root, "src"), { recursive: true });
  const artPath = path.join(root, "src/Button.art.vue");
  await cp(
    path.join(repository, "tests/tooling/fixtures/musea/snapshot-collision/left/Button.art.vue"),
    artPath,
  );
  const original = await readFile(artPath, "utf8");
  const authored = changed ? original.replace("#0000ff", "#00ff00") : original;
  assert.notEqual(original, original.replace("#0000ff", "#00ff00"));
  await writeFile(artPath, authored);
  await writeFile(
    path.join(output, changed ? "changed-input.art.vue" : "original-input.art.vue"),
    authored,
  );
  const directory = path.join(root, "dist");
  const setup =
    "export default function() { const target = new URL(location.href).searchParams.get('navigation'); if (target) location.replace(target); }\n";
  await writeFile(path.join(root, "preview.setup.ts"), setup);
  await writeFile(path.join(output, "preview.setup.ts"), setup);
  await build({
    root,
    configFile: false,
    base: "/built/",
    logLevel: "warn",
    resolve: {
      alias: { vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")) },
    },
    plugins: [
      vize(),
      musea({
        include: ["src/**/*.art.vue"],
        basePath: "/gallery/",
        previewSetup: "preview.setup.ts",
      }),
    ],
    build: { outDir: directory, emptyOutDir: true, minify: true },
  });
  const manifest = JSON.parse(
    await readFile(path.join(directory, "gallery/api/static.json"), "utf8"),
  ) as {
    arts: { path: string }[];
    previews: Record<string, Record<string, string>>;
    snapshotIdentities: Record<string, string>;
    snapshotIdentityVersion: number;
  };
  assert.equal(manifest.snapshotIdentityVersion, 1);
  assert.equal(manifest.arts.length, 1);
  assert.equal(manifest.snapshotIdentities[artPath], "src/Button.art.vue");
  await rm(path.join(root, "src"), { recursive: true });
  await rm(path.join(root, "preview.setup.ts"));
  await assert.rejects(stat(artPath), { code: "ENOENT" });
  return { directory, artPath, manifest, inputSha256: sha256(authored) };
}
