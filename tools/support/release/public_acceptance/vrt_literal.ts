import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { cp, mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Evidence } from "./vrt_artifacts.ts";
import { galleryPath, sha256 } from "./vrt_fixtures.ts";
import { hostedPath } from "./vrt_host.ts";
import { rejectOverrides } from "./installed.ts";

const prefix = "tests/tooling/fixtures/musea/literal-variant-keys/";
export const literalVrtSources = [
  `${prefix}Controls.art.vue`,
  `${prefix}Keys.art.vue`,
  "tests/_fixtures/differential/musea/literal-variant-keys.json",
] as const;
export const literalVariants = {
  Controls: [
    ["Default", "Ordinary"],
    ["Custom theme", "Custom"],
  ],
  Keys: [
    ["__proto__", "Proto"],
    ["constructor", "Constructor"],
    ["hasOwnProperty", "Own property"],
  ],
} as const;
export type LiteralTitle = keyof typeof literalVariants;

/** Additive exact H inputs; no old two-side fixture/materializer behavior is changed. */
export function materializeLiteralVrtInputs(consumer: string, read: (file: string) => Buffer) {
  return literalVrtSources.map((sourceFile) => {
    const file = `trusted-vrt-inputs/${path.basename(sourceFile)}`;
    const bytes = read(sourceFile);
    const target = path.join(consumer, file);
    mkdirSync(path.dirname(target), { recursive: true });
    writeFileSync(target, bytes, { flag: "wx" });
    return { file, sourceFile, bytes: bytes.length, sha256: sha256(bytes) };
  });
}

/** Fail closed on complete vector and authored byte identity before resolving any input path. */
export function readLiteralVrtInputs(consumer: string) {
  const inputs = path.join(consumer, "trusted-vrt-inputs");
  const raw = readFileSync(path.join(inputs, "literal-variant-keys.json"));
  assert.equal(
    sha256(raw),
    "e83575aa334ed2071602df9595650b1116721a752379371324ac10292908fcf9",
    "Frozen literal vector changed",
  );
  const corpus = JSON.parse(raw.toString()) as {
    variants: Record<LiteralTitle, string[]>;
    sources: Array<{ path: string; sha256: string }>;
  };
  assert.deepEqual(corpus.variants, {
    Controls: ["Default", "Custom theme"],
    Keys: ["__proto__", "constructor", "hasOwnProperty"],
  });
  assert.deepEqual(corpus.sources, [
    {
      path: literalVrtSources[0],
      sha256: "d4bb54f6be66646f344026afa72addfef86f237ef703c9b1f024fd815de1fb57",
    },
    {
      path: literalVrtSources[1],
      sha256: "3f121d5b7e959bf18c368eb060656db8cbe61cb4b45ccd003c1d9e583206001e",
    },
  ]);
  return corpus.sources.map((source) => {
    const bytes = readFileSync(path.join(inputs, path.basename(source.path)));
    assert.equal(sha256(bytes), source.sha256, "Frozen literal authored bytes changed");
    return { ...source, bytes };
  });
}

interface LiteralManifest {
  snapshotIdentityVersion: number;
  snapshotIdentities: Record<string, string>;
  arts: Array<{
    path: string;
    metadata: { title: LiteralTitle };
    variants: Array<{ name: string }>;
  }>;
  previews: Record<string, Record<string, string>>;
}

/** Unexecuted preparation: real installed Vite build and public plugin, never source fallback. */
export async function buildLiteralHostedVrt(consumer: string, e: Evidence) {
  rejectOverrides();
  const sources = readLiteralVrtInputs(consumer);
  const root = path.join(consumer, "trusted-vrt-workspace");
  const directory = path.join(root, "dist");
  const config = path.join(root, "vite.config.mjs");
  await mkdir(root);
  await mkdir(path.join(root, "src"));
  for (const source of sources) {
    const basename = path.basename(source.path);
    await writeFile(path.join(root, "src", basename), source.bytes, { flag: "wx" });
    await e.save(`trusted-literal/inputs/${basename}`, source.bytes);
  }
  await writeFile(path.join(root, "index.html"), "<!doctype html><html><body></body></html>\n", {
    flag: "wx",
  });
  const options = {
    include: ["src/**/*.art.vue"],
    basePath: galleryPath,
    vrt: {
      viewports: [{ name: "authored-compact", width: 320, height: 180 }],
      snapshotDir: "trusted-baselines",
      threshold: 0,
      workers: 2,
      capture: { settleTime: 0, waitForNetwork: false },
      comparison: { antiAliasing: false },
    },
  };
  const configBytes = `import vize from '@vizejs/vite-plugin';
import { musea } from '@vizejs/vite-plugin-musea';
import { fileURLToPath } from 'node:url';
export default {
  root: ${JSON.stringify(root)}, base: '/built/',
  resolve: { alias: { vue: fileURLToPath(import.meta.resolve('vue/dist/vue.runtime.esm-bundler.js')) } },
  plugins: [vize(), musea(${JSON.stringify(options)})],
};\n`;
  await writeFile(config, configBytes, { flag: "wx" });
  await e.save("trusted-literal/inputs/vite.config.mjs", configBytes);
  const { build } = await import("vite");
  await build({
    root,
    configFile: config,
    logLevel: "warn",
    build: { outDir: directory, emptyOutDir: true },
  });
  const raw = await readFile(path.join(directory, "gallery/vrt/api/static.json"));
  await e.save("trusted-literal/static.json", raw);
  const manifest = JSON.parse(raw.toString()) as LiteralManifest;
  assert.equal(manifest.snapshotIdentityVersion, 1);
  assert.equal(manifest.arts.length, 2);
  assert.deepEqual(manifest.arts.map((art) => art.metadata.title).sort(), ["Controls", "Keys"]);
  const urls = new Set<string>();
  for (const title of ["Controls", "Keys"] as const) {
    const artPath = path.join(root, "src", `${title}.art.vue`);
    const art = manifest.arts.find((item) => item.path === artPath);
    assert.ok(art);
    assert.equal(manifest.snapshotIdentities[artPath], `src/${title}.art.vue`);
    const names = literalVariants[title].map(([name]) => name);
    assert.deepEqual(
      art.variants.map((variant) => variant.name),
      names,
    );
    assert.equal(Object.hasOwn(manifest.previews, artPath), true);
    const previews = manifest.previews[artPath];
    assert.deepEqual(Object.keys(previews), names);
    for (const name of names) {
      assert.equal(Object.hasOwn(previews, name), true);
      const value = previews[name];
      assert.equal(typeof value, "string");
      assert.ok(value.startsWith(`${hostedPath}preview/`));
      const url = new URL(value, "https://manifest.invalid");
      assert.equal(url.origin, "https://manifest.invalid");
      assert.equal(url.search + url.hash, "");
      assert.equal(urls.has(value), false);
      urls.add(value);
      const physical = path.resolve(directory, url.pathname.slice("/built/".length));
      assert.ok(physical.startsWith(directory + path.sep));
      assert.equal((await stat(physical)).isFile(), true);
    }
  }
  assert.equal(urls.size, 5);
  await cp(directory, path.join(e.output, "trusted-literal/dist"), { recursive: true });
  await rm(path.join(root, "src"), { recursive: true });
  for (const title of ["Controls", "Keys"])
    await assert.rejects(stat(path.join(root, "src", `${title}.art.vue`)), { code: "ENOENT" });
  e.records.push({
    phase: "trusted-literal-native-build",
    manifestSha256: sha256(raw),
    authoredSourcesDeleted: true,
    ownPreviewURLs: 5,
    sources: sources.map(({ bytes, ...source }) => ({ ...source, bytes: bytes.length })),
  });
  return { root, config, directory, manifest, raw, sources };
}
