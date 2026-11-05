import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import {
  devCacheReporterFilename,
  devCacheReporterSource,
  parcelHostConfig,
} from "./reporter-config.ts";

const parcelRequire = createRequire(
  new URL("../../../../npm/builder/parcel/package.json", import.meta.url),
);
const coreManifest = parcelRequire.resolve("@parcel/core/package.json");
const schema: {
  validatePackageName(name: string, pluginType: string, key: string): void;
} = parcelRequire(join(dirname(coreManifest), "lib/ParcelConfig.schema.js"));

void test("the consumer reporters satisfy the pinned Parcel schema in their required order", async () => {
  assert.equal(JSON.parse(await readFile(coreManifest, "utf8")).version, "2.16.4");
  assert.throws(
    () =>
      schema.validatePackageName("@vizejs/parcel-transformer/dev-cache", "reporter", "reporters"),
    /Scoped parcel reporter packages must be named/,
  );
  const config = parcelHostConfig();
  for (const name of config.reporters.filter((name) => name !== "...")) {
    assert.doesNotThrow(() => schema.validatePackageName(name, "reporter", "reporters"));
  }
  assert.deepEqual(config.reporters, [
    `./${devCacheReporterFilename}`,
    "./native-trace.cjs",
    "...",
  ]);
});

void test("the consumer CJS adapter forwards the public export without changing default interop", async () => {
  const directory = await mkdtemp(join(tmpdir(), "vize-parcel reporter 😺 "));
  try {
    const manifest = await readFile(
      new URL("../../../../npm/builder/parcel/package.json", import.meta.url),
      "utf8",
    );
    const packageRoot = join(directory, "node_modules/@vizejs/parcel-transformer");
    await mkdir(join(packageRoot, "dist"), { recursive: true });
    await writeFile(join(packageRoot, "package.json"), manifest);
    // A literal consumer input, independent of the native transformer runtime.
    await writeFile(
      join(packageRoot, "dist/dev-cache.cjs"),
      'exports.default = { reporter: "identity" };\n',
    );
    await writeFile(join(directory, "package.json"), '{"type":"module"}\n');
    await writeFile(join(directory, devCacheReporterFilename), devCacheReporterSource);
    const require = createRequire(join(directory, "package.json"));
    const direct: unknown = require("@vizejs/parcel-transformer/dev-cache");
    assert.equal(require(`./${devCacheReporterFilename}`), direct);
    assert.ok(direct !== null && typeof direct === "object" && "default" in direct);
    assert.deepEqual(direct.default, { reporter: "identity" });
    assert.throws(() => require("@vizejs/parcel-transformer/dist/dev-cache.cjs"), {
      code: "ERR_PACKAGE_PATH_NOT_EXPORTED",
    });
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
