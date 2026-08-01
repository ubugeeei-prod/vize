/**
 * Differential oracle for the shareable Nuxt lint preset and its Nuxt emitter.
 *
 * Runs every case in `fixtures/corpus.json` through the real `@nuxt/eslint`
 * module, `@nuxt/eslint-config`, and `@nuxt/eslint-plugin`, and records what
 * they produce in `fixtures/nuxt-eslint-output.json`. That recorded file is the committed
 * ground truth: `src/oracle.test.ts` reads the preset and rule recordings with
 * no `@nuxt/*` dependency
 * at all, so the package's own suite stays offline and fast, while
 * `tests/tooling/nuxt-eslint-oracle.test.ts` re-runs this script in CI and
 * fails if the recording has drifted from the installed packages.
 *
 * Three upstream contracts are recorded, because the port splits along
 * the same seam:
 *
 *   1. `dirs` — the Nuxt project state (layers, `srcDir`, `dir` overrides,
 *      component directories) reduced to the directory lists every glob is
 *      built from. Produced by `@nuxt/eslint`'s module.
 *   2. `features` + the resolved config items — which named rule blocks exist,
 *      in what order, over which globs. Produced by `@nuxt/eslint-config`.
 *   3. `importGlobals` — the complete ordered globals list emitted after both
 *      Nuxt and Nitro publish their auto-import registries.
 *
 * Rule cases separately record the real plugin's exact diagnostics and output,
 * plus a second application proving fix convergence or non-fixable stability.
 *
 * Usage:
 *   node npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/oracle.mjs --check
 *   node npm/framework/nuxt-lint-config/test/nuxt-eslint-compat/oracle.mjs --write
 */
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { renderNuxtOxlintConfig } from "../../../nuxt/src/lint/emitter.ts";
import { buildNuxtLintPlan } from "../../src/plan.ts";
import { recordNoPageMetaRuntimeValuesCases } from "./no-page-meta-runtime-values-oracle.mjs";
import { recordPreferImportMetaCases } from "./prefer-import-meta-oracle.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const RECORDED_VIZE_PLUGIN_SPECIFIER = "../node_modules/oxlint-plugin-vize/dist/index.mjs";
export const corpusPath = join(here, "fixtures", "corpus.json");
export const recordedPath = join(here, "fixtures", "nuxt-eslint-output.json");

/**
 * Where the throwaway Nuxt roots are created.
 *
 * The generated config imports `@nuxt/eslint-config` through a path *relative
 * to itself*, so the scratch root has to sit somewhere those relative
 * specifiers still resolve — `node_modules/.vize/`, the same gitignored scratch
 * area the rest of the toolchain writes derived artifacts into. A system temp
 * directory does not work: on macOS its `/private` symlink makes the emitted
 * relative path resolve to a directory that does not exist.
 */
const scratchRoot = join(
  here,
  "..",
  "..",
  "..",
  "..",
  "..",
  "node_modules",
  ".vize",
  "nuxt-eslint-oracle",
);

/**
 * Config item names the port owns.
 *
 * These are the blocks that encode Nuxt project awareness. The remaining items
 * are `@nuxt/eslint-config`'s generic JavaScript/TypeScript/Vue/stylistic
 * preset content, which is a separate phase of the port — their *names and
 * order* are still recorded so nothing can disappear unnoticed.
 */
export const NUXT_OWNED_CONFIG_NAMES = [
  "nuxt/setup",
  "nuxt/vue/single-root",
  "nuxt/rules",
  "nuxt/pages",
  "nuxt/nuxt-config",
  "nuxt/sort-config",
  "nuxt/disables/routes",
  "nuxt/import-globals",
];

export function readCorpus() {
  return JSON.parse(readFileSync(corpusPath, "utf8"));
}

export function readRecorded() {
  return JSON.parse(readFileSync(recordedPath, "utf8"));
}

/**
 * A Nuxt instance stub carrying only what config generation reads.
 *
 * `@nuxt/eslint` touches `options.rootDir`, `options.buildDir`,
 * `options.dir`, and `options._layers`, and registers hooks it never fires
 * itself. Driving it with a stub keeps the oracle free of a real Nuxt build,
 * so the recording depends on the corpus alone.
 */
function createNuxtStub(rootDir, entry) {
  const hooks = new Map();
  const layers = entry.project.layers.map((layer) => ({
    config: { ...layer, srcDir: join(rootDir, layer.srcDir) },
  }));
  return {
    options: {
      rootDir,
      buildDir: join(rootDir, ".nuxt"),
      srcDir: layers[0].config.srcDir,
      dir: entry.project.dir ?? {},
      _layers: layers,
    },
    hook(name, handler) {
      if (!hooks.has(name)) hooks.set(name, []);
      hooks.get(name).push(handler);
    },
    async callHook(name, ...args) {
      for (const handler of hooks.get(name) ?? []) await handler(...args);
    },
  };
}

/**
 * Run the real module for one case and return the directory lists it derived.
 *
 * The generated config module is imported rather than text-matched: it exports
 * the resolved options object, which is the value the config presets actually
 * consume.
 */
async function recordDirs(setupConfigGen, entry) {
  mkdirSync(scratchRoot, { recursive: true });
  const rootDir = mkdtempSync(join(scratchRoot, "case-"));
  try {
    mkdirSync(join(rootDir, ".nuxt"), { recursive: true });
    const nuxt = createNuxtStub(rootDir, entry);
    // `autoInit` is disabled so the oracle never writes an `eslint.config.mjs`
    // outside its temporary root, and never walks the filesystem looking for one.
    await setupConfigGen({ config: { autoInit: false } }, nuxt);
    const generated = await import(join(rootDir, ".nuxt", "eslint.config.mjs"));
    return generated.options.dirs;
  } finally {
    rmSync(rootDir, { force: true, recursive: true });
  }
}

/** Record the full globals object emitted from Nuxt and Nitro's registries. */
async function recordImportGlobals(setupConfigGen, entry, importGlobals) {
  mkdirSync(scratchRoot, { recursive: true });
  const rootDir = mkdtempSync(join(scratchRoot, "globals-"));
  try {
    mkdirSync(join(rootDir, ".nuxt"), { recursive: true });
    const nuxt = createNuxtStub(rootDir, entry);
    await setupConfigGen({ config: { autoInit: false } }, nuxt);
    await nuxt.callHook("imports:context", {
      getImports: async () => structuredClone(importGlobals.nuxt),
    });
    await nuxt.callHook("nitro:init", {
      unimport: { getImports: async () => structuredClone(importGlobals.nitro) },
    });
    await nuxt.callHook("builder:generateApp");

    const generated = await import(join(rootDir, ".nuxt", "eslint.config.mjs"));
    const configs = await generated.configs;
    const globals = configs.find((item) => item.name === "nuxt/import-globals")?.languageOptions
      ?.globals;
    if (!globals) throw new Error("@nuxt/eslint did not emit nuxt/import-globals");
    return globals;
  } finally {
    rmSync(rootDir, { force: true, recursive: true });
  }
}

/** Reduce one resolved flat-config item to its serialisable contract. */
function recordConfigItem(item) {
  return {
    name: item.name ?? null,
    files: item.files ?? null,
    ignores: item.ignores ?? null,
    rules: item.rules ?? null,
    globals: item.languageOptions?.globals ?? null,
  };
}

/**
 * Whether an item belongs to the Nuxt-aware surface this phase ports.
 *
 * The ignore block is matched structurally because upstream leaves it unnamed;
 * `eslint-config-flat-gitignore`'s block is excluded by having a name.
 */
function isNuxtOwned(item) {
  if (NUXT_OWNED_CONFIG_NAMES.includes(item.name)) return true;
  return item.name === undefined && Array.isArray(item.ignores);
}

/**
 * Read the version of a package from one of its resolved entry points.
 *
 * Neither `@nuxt/eslint` nor `@nuxt/eslint-config` exports `./package.json`, so
 * the manifest is read beside the `dist/` directory the entry lives in.
 */
function packageVersionFrom(entryUrl) {
  const manifest = fileURLToPath(new URL("../package.json", entryUrl));
  return JSON.parse(readFileSync(manifest, "utf8")).version;
}

/** Run the whole corpus, returning the recordable payload. */
export async function runOracle() {
  const moduleEntry = import.meta.resolve("@nuxt/eslint");
  const configEntry = import.meta.resolve("@nuxt/eslint-config/flat");
  const [{ setupConfigGen }, { createConfigForNuxt, resolveOptions }] = await Promise.all([
    import(new URL("./chunks/index.mjs", moduleEntry).href),
    import(configEntry),
  ]);

  const corpus = readCorpus();
  const [preferImportMeta, noPageMetaRuntimeValues] = await Promise.all([
    recordPreferImportMetaCases(moduleEntry, corpus, packageVersionFrom),
    recordNoPageMetaRuntimeValuesCases(moduleEntry, corpus, packageVersionFrom),
  ]);
  if (preferImportMeta.pluginVersion !== noPageMetaRuntimeValues.pluginVersion) {
    throw new Error("Nuxt rule oracles resolved different @nuxt/eslint-plugin versions");
  }

  // The directory defaults only apply to hand-written configs — a config
  // generated from a Nuxt instance always supplies every list — so they are
  // recorded from `resolveOptions` directly rather than through the module.
  const dirDefaults = {};
  for (const entry of corpus.dirDefaultCases) {
    dirDefaults[entry.id] = resolveOptions({
      features: {},
      dirs: structuredClone(entry.dirs),
    }).dirs;
  }

  const cases = {};
  for (const entry of corpus.cases) {
    const dirs = await recordDirs(setupConfigGen, entry);
    const options = { features: entry.config, dirs };
    const resolved = resolveOptions(structuredClone(options));
    const items = await createConfigForNuxt(structuredClone(options));
    const nuxtConfigs = items.filter(isNuxtOwned).map(recordConfigItem);
    cases[entry.id] = {
      dirs,
      features: resolved.features,
      configNames: items.map((item) => item.name ?? null),
      nuxtConfigs,
      oxlintConfig: renderNuxtOxlintConfig(
        buildNuxtLintPlan(resolved.features, dirs),
        RECORDED_VIZE_PLUGIN_SPECIFIER,
      ),
    };
  }

  const importGlobalsEntry = corpus.cases[0];
  if (!importGlobalsEntry) throw new Error("the import-globals oracle requires a project case");
  const recordedImportGlobals = await recordImportGlobals(
    setupConfigGen,
    importGlobalsEntry,
    corpus.importGlobals,
  );
  const importGlobalsCase = cases[importGlobalsEntry.id];
  if (!importGlobalsCase) throw new Error("the import-globals oracle case was not recorded");
  const importGlobalsPlan = buildNuxtLintPlan(importGlobalsCase.features, importGlobalsCase.dirs);
  const initialOxlintConfig = renderNuxtOxlintConfig(
    importGlobalsPlan,
    RECORDED_VIZE_PLUGIN_SPECIFIER,
  );
  const regeneratedOxlintConfig = renderNuxtOxlintConfig(
    [
      ...importGlobalsPlan,
      {
        name: "nuxt/import-globals",
        globals: recordedImportGlobals,
      },
    ],
    RECORDED_VIZE_PLUGIN_SPECIFIER,
  );

  return {
    schemaVersion: 4,
    description:
      "Recorded @nuxt/eslint output for every case in corpus.json. Generated — do not hand-edit; re-record with oracle.mjs --write.",
    moduleVersion: packageVersionFrom(moduleEntry),
    configVersion: packageVersionFrom(configEntry),
    pluginVersion: preferImportMeta.pluginVersion,
    // `@nuxt/eslint-config` defaults `features.typescript` to whether the
    // `typescript` package resolves. Recording the probe's answer keeps the
    // offline test honest about which branch the rest of the recording is on.
    typeScriptDetected: resolveOptions({ features: {}, dirs: {} }).features.typescript,
    importGlobals: {
      globals: Object.keys(recordedImportGlobals),
      artifacts: {
        initial: initialOxlintConfig,
        regenerated: regeneratedOxlintConfig,
      },
    },
    preferImportMetaCases: preferImportMeta.cases,
    noPageMetaRuntimeValuesCases: noPageMetaRuntimeValues.cases,
    dirDefaults,
    cases,
  };
}

if (import.meta.url === `file://${process.argv[1]}`) {
  const write = process.argv.includes("--write");
  const recorded = await runOracle();
  if (write) {
    writeFileSync(recordedPath, `${JSON.stringify(recorded, null, 2)}\n`);
    console.log(`recorded ${Object.keys(recorded.cases).length} cases -> ${recordedPath}`);
  } else {
    const expected = readRecorded();
    const actual = JSON.stringify(recorded, null, 2);
    if (JSON.stringify(expected, null, 2) !== actual) {
      console.error("nuxt-eslint-output.json is out of date; re-record with --write");
      process.exitCode = 1;
    } else {
      console.log(`oracle matches ${Object.keys(recorded.cases).length} recorded cases`);
    }
  }
}
