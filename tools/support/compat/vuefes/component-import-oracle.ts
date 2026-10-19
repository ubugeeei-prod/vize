import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import type { NuxtComponentDescriptor } from "../../../../npm/framework/nuxt/src/components.ts";
import type { RawTransformPlugin } from "./oracle-support.ts";

interface NuxtTransformModule {
  readonly TransformPlugin: (
    nuxt: { options: { buildDir: string; sourcemap: { client: boolean; server: boolean } } },
    options: {
      mode: "all";
      getComponents: () => readonly NuxtComponentDescriptor[];
      serverComponentRuntime: string;
    },
  ) => { raw(meta: object): RawTransformPlugin };
}

const oracle = fileURLToPath(new URL("./fixtures/app-component-order/", import.meta.url));
const fromOracle = createRequire(path.join(oracle, "package.json"));
const source = fs.readFileSync(path.join(oracle, "nuxt-transform.ts.txt"));
assert.equal(source.length, 4846);
assert.equal(
  createHash("sha256").update(source).digest("hex"),
  "229b33a703af61a63604e1623a207b3a802728a754b368dac8e20dfb983087b6",
);
const materialized = path.join(oracle, ".generated/transform.ts");
fs.mkdirSync(path.dirname(materialized), { recursive: true });
fs.writeFileSync(materialized, source);
assert(fs.readFileSync(materialized).equals(source));
const { TransformPlugin }: NuxtTransformModule = await import(pathToFileURL(materialized).href);
for (const [name, version] of [
  ["nuxt", "4.0.3"],
  ["@nuxt/kit", "4.0.3"],
  ["unimport", "5.2.0"],
] as const) {
  const manifest: unknown = JSON.parse(
    fs.readFileSync(fromOracle.resolve(`${name}/package.json`), "utf8"),
  );
  assert.ok(typeof manifest === "object" && manifest !== null && "version" in manifest);
  assert.equal(manifest.version, version);
}

export function createNuxtImportOracle(
  components: readonly NuxtComponentDescriptor[],
): RawTransformPlugin {
  return TransformPlugin(
    {
      options: {
        buildDir: path.join(oracle, ".nuxt"),
        sourcemap: { client: false, server: false },
      },
    },
    {
      mode: "all",
      getComponents: () => components,
      serverComponentRuntime: "#app/components/server-component",
    },
  ).raw({});
}

export function originalNuxtComponents(): NuxtComponentDescriptor[] {
  const packageDir = path.dirname(fromOracle.resolve("nuxt/package.json"));
  const entries = [
    ["NuxtPage", "dist/pages/runtime/page.js"],
    ["NuxtRouteAnnouncer", "dist/app/components/nuxt-route-announcer.js"],
    ["NuxtLayout", "dist/app/components/nuxt-layout.js"],
  ] as const;
  return entries.map(([pascalName, relative]) => {
    const filePath = path.join(packageDir, relative);
    assert(fs.existsSync(filePath), "actual pinned Nuxt runtime entry must exist");
    return { pascalName, filePath };
  });
}

/** The transformed code; the pinned Nuxt transform always rewrites these inputs. */
export async function nuxtTransform(
  plugin: RawTransformPlugin,
  code: string,
  id: string,
): Promise<string> {
  const result = await plugin.transform(code, id);
  assert.ok(result, `pinned Nuxt transform returned no result for ${id}`);
  return result.code;
}
