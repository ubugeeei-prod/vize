import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import { readJsonRecord, type RawTransformPlugin, type TransformResult } from "./oracle-support.ts";

interface PageMetaModule {
  readonly PageMetaPlugin: (options: { dev: boolean; sourcemap: boolean }) => {
    raw(meta: object): RawTransformPlugin;
  };
}

const directory = fileURLToPath(new URL("./fixtures/app-component-order/", import.meta.url));
const fromOracle = createRequire(path.join(directory, "package.json"));
const packageFile = fromOracle.resolve("nuxt/package.json");
assert.equal(readJsonRecord(packageFile).version, "4.0.3");
const original = fs.readFileSync(path.join(path.dirname(packageFile), "dist/index.mjs"));
assert.equal(original.length, 287261);
const hash = (value: Uint8Array): string => createHash("sha256").update(value).digest("hex");
assert.equal(hash(original), "3b279d9e74e3221294a437d610c7cad6ce6971a37f58f79650c0d111421318ee");
// Expose the real selected plugin from a private copy of the complete pinned
// package module. Every original byte and import stays intact; upstream and
// the installed dependency target are never edited.
const access = Buffer.from("\nexport { PageMetaPlugin };\n");
const materialized = path.join(directory, ".generated/page-meta-oracle.mjs");
fs.mkdirSync(path.dirname(materialized), { recursive: true });
fs.writeFileSync(materialized, Buffer.concat([original, access]));
assert(fs.readFileSync(materialized).equals(Buffer.concat([original, access])));
const custody = fileURLToPath(
  new URL(
    "../../../../target/vuefes-ssr-style-dependencies/page-meta-dependencies/oracle-modules/",
    import.meta.url,
  ),
);
fs.mkdirSync(custody, { recursive: true });
fs.writeFileSync(path.join(custody, "nuxt-index.original.mjs"), original);
fs.writeFileSync(
  path.join(custody, "nuxt-index.private-access.mjs"),
  Buffer.concat([original, access]),
);
const { PageMetaPlugin }: PageMetaModule = await import(pathToFileURL(materialized).href);

export const pageMetaOracleIdentity = {
  nuxt: "4.0.3",
  fullOriginalModuleBytes: original.length,
  fullOriginalModuleSha256: hash(original),
  privateAccessAppend: access.toString(),
  privateAccessModuleSha256: hash(Buffer.concat([original, access])),
  boundary: "complete pinned package module plus explicit private plugin export",
};

export async function officialPageMeta(source: string): Promise<TransformResult> {
  const plugin = PageMetaPlugin({ dev: false, sourcemap: true }).raw({});
  const result = await plugin.transform(source, "/original/Photo.vue?macro=true");
  assert(result && typeof result.code === "string");
  return result;
}
