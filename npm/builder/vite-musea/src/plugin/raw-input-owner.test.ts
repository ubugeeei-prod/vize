import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { createRawArtInputOwner } from "./raw-input-owner.ts";
import { loadNative } from "../native-loader.ts";
import { compileAll } from "../../../vite/src/plugin/precompile-run.ts";
import { snapshotPrecompileInputOwners } from "../../../vite/src/plugin/precompile-owner.ts";
import {
  DEFAULT_PRECOMPILE_BATCH_SIZE,
  type VizePluginState,
} from "../../../vite/src/plugin/state.ts";

const raw = '<art title="Owned"><variant name="Default"><div>preview</div></variant></art>';
const normal = "<template><div>ordinary component</div></template>";
const missingBlock = "At least one <template> or <script> is required in a single file component.";

function project() {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-musea-owner-"));
  fs.mkdirSync(path.join(root, "src"));
  const config = {
    root,
    include: ["src/**/*.art.vue"],
    exclude: ["src/excluded/**"],
    scanRoots: [root],
  };
  return { root, config, owner: createRawArtInputOwner(config) };
}

function ownersFor(owner: { ownsInput(file: string, source: string): boolean }) {
  return snapshotPrecompileInputOwners({
    plugins: [{ api: { vizePrecompileInputOwner: { version: 1, forConfig: () => owner } } }],
  });
}

function state(root: string): VizePluginState {
  return {
    root,
    cache: new Map(),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    clientViteBase: "/",
    serverViteBase: "/",
    server: null,
    filter: () => true,
    scanPatterns: ["src/**/*.vue"],
    precompileBatchSize: DEFAULT_PRECOMPILE_BATCH_SIZE,
    ignorePatterns: [],
    mergedOptions: {},
    initialized: true,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: true,
    componentsCssFileName: "assets/vize-components.css",
    clientViteDefine: {},
    serverViteDefine: {},
    logger: { log() {}, info() {}, warn() {}, error() {} } as never,
  };
}

function errorOf(run: () => unknown) {
  try {
    run();
    return null;
  } catch (error) {
    assert.ok(error instanceof Error);
    return { name: error.name, code: Reflect.get(error, "code"), message: error.message };
  }
}

void test("real structural/native ownership requires the selected raw Art role and nonempty variants", () => {
  const { root, config, owner } = project();
  try {
    const file = path.join(root, "src/Owned.art.vue");
    assert.equal(owner.ownsInput(file, raw), true);
    assert.equal(owner.ownsInput(file, normal), false);
    assert.equal(owner.ownsInput(path.join(root, "src/Inline.vue"), raw), false);
    assert.equal(owner.ownsInput(path.join(root, "src/excluded/Owned.art.vue"), raw), false);
    assert.equal(owner.ownsInput(path.join(root, "outside/Owned.art.vue"), raw), false);
    const external = path.resolve(root, "../foreign/Owned.art.vue");
    assert.equal(
      createRawArtInputOwner({ ...config, include: [external] }).ownsInput(external, raw),
      false,
      "a matching pattern cannot invent a trusted source root",
    );
    assert.equal(owner.ownsInput(file, '<art title="Empty"></art>'), false);
    const malformed = '<art title="Broken"><variant>';
    const parserError = errorOf(() => loadNative().parseSfc(malformed, { filename: file }));
    assert.notEqual(parserError, null);
    assert.deepEqual(
      errorOf(() => owner.ownsInput(file, malformed)),
      parserError,
    );
    config.include = [];
    assert.equal(createRawArtInputOwner(config).ownsInput(file, raw), false);
    assert.equal(
      owner.ownsInput(file, raw),
      true,
      "an already resolved owner retains its exact filter snapshot",
    );
    for (const source of [raw, "<style>.test{color:red}</style>", "<i18n>{}</i18n>", ""]) {
      const result = loadNative().compileSfc(source, { filename: file });
      assert.deepEqual(result.errors, [missingBlock]);
    }
    console.log(
      JSON.stringify({
        role: "raw-art-owner",
        nativeDescriptor: loadNative().parseSfc(raw, { filename: file }),
        nativeArt: loadNative().parseArt(raw, { filename: file }),
      }),
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("delegation precedes persistent restore, clears stale state and leaves ordinary SFCs compiled", async () => {
  const { root, config, owner } = project();
  try {
    const file = path.join(root, "src/Owned.art.vue");
    const ordinary = path.join(root, "src/Ordinary.vue");
    fs.writeFileSync(file, normal);
    fs.writeFileSync(ordinary, normal);
    const first = state(root);
    await compileAll(first);
    const compiled = first.cache.get(file);
    assert.ok(compiled);
    assert.ok(first.precompileMetadata.has(file));

    // This explicit protocol control claims an unchanged source, so old metadata
    // cannot suppress a newly active callback. The real Musea owner below still
    // refuses normal .art.vue input.
    const capability = {
      ownsInput: (candidate: string, source: string) => candidate === file && source === normal,
    };
    const claimed = ownersFor(capability);
    first.ssrCache.set(file, compiled);
    first.collectedCss.set(file, "stale CSS");
    await compileAll(first, claimed);
    assert.equal(first.cache.has(file), false);
    assert.equal(first.ssrCache.has(file), false);
    assert.equal(first.collectedCss.has(file), false);
    assert.equal(first.precompileMetadata.has(file), false);
    assert.ok(first.cache.has(ordinary));
    const cold = state(root);
    await compileAll(cold, claimed);
    assert.equal(cold.cache.has(file), false);
    assert.equal(cold.precompileMetadata.has(file), false);
    assert.ok(cold.cache.has(ordinary));

    fs.writeFileSync(file, raw);
    const actualOwners = ownersFor(owner);
    await compileAll(first, actualOwners);
    await compileAll(state(root), actualOwners);
    assert.equal(first.cache.has(file), false);
    assert.ok(first.cache.has(ordinary));
    await assert.rejects(compileAll(state(root)), /At least one <template> or <script>/);
    config.exclude = ["src/**"];
    await assert.rejects(
      compileAll(state(root), ownersFor(createRawArtInputOwner(config))),
      /At least one <template> or <script>/,
    );
    config.exclude = [];
    fs.writeFileSync(file, normal);
    await compileAll(first, actualOwners);
    assert.deepEqual(first.cache.get(file)?.code, compiled.code);
    assert.ok(first.precompileMetadata.has(file));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
