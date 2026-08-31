/**
 * Drives `@vizejs/vite-plugin-musea` and `@vizejs/musea-nuxt` hook by hook (#3464).
 *
 * Why per-hook and not an end-to-end `vite build`: #3426 established, for the
 * Nuxt module, that an end-to-end build puts the Vize module's own cost below
 * the run-to-run noise floor — Rollup and SFC compilation dominate, so the row
 * cannot move when the module regresses. Musea's plugin sits in exactly that
 * position, so an end-to-end row would publish a number that no Musea
 * regression can shift. These stages instead call the plugin's own hooks on a
 * pinned corpus, so each measured number belongs to code this repo owns.
 *
 * The hooks are called through the published entry point, not through internal
 * modules, so the lane measures what a user's Vite build measures. Building the
 * plugin is therefore a prerequisite, and a missing build names the task that
 * produces it rather than silently measuring nothing.
 */

import { createHash } from "node:crypto";
import { pathToFileURL } from "node:url";

export { resolveMuseaArtifacts } from "./musea-artifacts.mjs";

const NUXT_VIRTUAL_IDS = [
  "#imports",
  "#app",
  "#build",
  "nuxt/app",
  "#components",
  "#build/components",
  "nuxt/dist/app/components",
  "#app/composables/router",
  "#build/some-generated-module",
];

/**
 * Order strings by code unit, not by `localeCompare`.
 *
 * The digest a stage returns has to be identical on every machine for the
 * reproducibility check to mean anything, and `localeCompare` orders by the
 * runtime's locale. A sort that reorders under `LANG` would make two honest
 * runs disagree and fail the run for a difference that is not a regression.
 */
function byCodeUnit(a, b) {
  if (a === b) return 0;
  return a < b ? -1 : 1;
}

/** A hook may be a bare function or a `{ handler }` object; normalise both. */
function hookOf(plugin, name) {
  const hook = plugin[name];
  const fn = typeof hook === "function" ? hook : hook?.handler;
  if (typeof fn !== "function") {
    throw new Error(`musea-stages: plugin ${plugin.name} has no ${name} hook`);
  }
  return fn.bind(plugin);
}

/**
 * A `ResolvedConfig` carrying only the fields the plugin reads.
 *
 * `command: "build"` is deliberate: on `build` the plugin rethrows art
 * processing failures instead of logging them, so a corpus the plugin cannot
 * parse fails the run rather than producing a fast, meaningless number.
 */
function resolvedConfigFor(workDir, configPatch) {
  return {
    root: workDir,
    command: "build",
    mode: "production",
    plugins: [],
    build: {
      ssr: false,
      rollupOptions: configPatch?.build?.rollupOptions ?? {},
    },
  };
}

async function withoutConsole(fn) {
  const { log, error, warn } = console;
  console.log = () => {};
  console.error = () => {};
  console.warn = () => {};
  try {
    return await fn();
  } finally {
    Object.assign(console, { log, error, warn });
  }
}

/**
 * Load and configure the plugin, stopping before Rollup's `options` hook.
 *
 * A normal Vite build injects the Musea static inputs in `config`; `options`
 * preserves them unless the explicit standalone-static environment flag is
 * set. The benchmark models that ordinary build path and measures the hook
 * honestly rather than claiming it performs per-file work.
 */
async function configurePlugin(museaEntry, workDir) {
  if (process.env.VIZE_MUSEA_STATIC_BUILD === "1") {
    throw new Error(
      "musea-stages: unset VIZE_MUSEA_STATIC_BUILD; this lane models an ordinary Vite build",
    );
  }
  const { musea } = await import(pathToFileURL(museaEntry).href);
  const plugins = musea({ include: ["**/*.art.vue"], exclude: ["node_modules/**", "dist/**"] });
  const [plugin] = plugins;
  // If the factory ever returns more than the one plugin, measuring only the
  // first would quietly under-report the package's cost. Fail instead.
  if (plugins.length !== 1 || plugin?.name !== "vite-plugin-musea") {
    throw new Error(
      `musea-stages: expected one plugin named vite-plugin-musea, got [${plugins.map((entry) => entry?.name).join(", ")}]`,
    );
  }
  const userConfig = {
    root: workDir,
    build: { rollupOptions: { input: "musea-user-entry.html" } },
  };
  const configPatch = await withoutConsole(() =>
    hookOf(plugin, "config")(userConfig, { command: "build", mode: "production" }),
  );
  await withoutConsole(() =>
    hookOf(plugin, "configResolved")(resolvedConfigFor(workDir, configPatch)),
  );
  return { plugin, rollupOptions: structuredClone(configPatch?.build?.rollupOptions ?? {}) };
}

async function runOptions(configured) {
  return hookOf(configured.plugin, "options")(configured.rollupOptions);
}

async function runBuildStart(plugin) {
  await withoutConsole(() => hookOf(plugin, "buildStart")());
}

async function startPlugin(museaEntry, workDir) {
  const configured = await configurePlugin(museaEntry, workDir);
  await runOptions(configured);
  await runBuildStart(configured.plugin);
  return configured.plugin;
}

function virtualIdsFor(plugin, files) {
  const resolveId = hookOf(plugin, "resolveId");
  const ids = ["\0musea-gallery", "\0musea-manifest"];
  for (const file of files) {
    const resolved = resolveId(file);
    if (resolved == null) {
      throw new Error(`musea-stages: plugin did not resolve ${file}`);
    }
    ids.push(resolved);
  }
  return ids;
}

function digestOf(modules) {
  const hash = createHash("sha256");
  for (const code of modules) {
    const text = code ?? "";
    hash.update(`${Buffer.byteLength(text)}:`);
    hash.update(text);
  }
  return hash.digest("hex");
}

function digestOptions(observation) {
  return digestOf([JSON.stringify(observation)]);
}

function digestWholePlugin(output) {
  return digestOf([
    JSON.stringify({
      rollupOptions: output.rollupOptions,
      optionsReturned: output.optionsReturned ?? null,
    }),
    digestOf(output.modules),
    digestOf(output.transformed),
  ]);
}

function loadAll(plugin, ids) {
  const load = hookOf(plugin, "load");
  return ids.map((id) => {
    const code = load(id);
    if (code == null) {
      throw new Error(`musea-stages: plugin did not load ${id}`);
    }
    return code;
  });
}

async function transformAll(plugin, ids, modules) {
  const transform = hookOf(plugin, "transform");
  const results = [];
  for (const [index, id] of ids.entries()) {
    const result = await transform(modules[index], id);
    results.push(result == null ? modules[index] : (result.code ?? modules[index]));
  }
  return results;
}

/**
 * The measurable stages, each with untimed `prepare` and `observe` phases and
 * a timed `run`.
 *
 * `prepare` exists so a stage measures only its own hook: the `load` stage must
 * not pay for `buildStart`, and the `transform` stage must not pay for `load`.
 * `observe` is deliberately outside the timer: `buildStart` itself returns no
 * value, so its observable is the deterministic module graph it populated.
 * The final stage runs every measured hook on a fresh plugin, which is the
 * number that answers "what does this plugin cost a build".
 */
export function createMuseaStages({ artifacts, workDir, files }) {
  const museaEntry = artifacts.museaPlugin.measuredPath;
  const nuxtEntry = artifacts.museaNuxt.measuredPath;
  let optionsStage = null;
  let buildStartStage = null;
  let loadStage = null;
  let transformStage = null;
  let nuxtPlugin = null;

  return [
    {
      id: "musea-options",
      label: "options: preserve configured Rollup inputs",
      units: 1,
      unitLabel: "build hooks",
      prepare: async () => {
        const configured = await configurePlugin(museaEntry, workDir);
        optionsStage = {
          configured,
          before: structuredClone(configured.rollupOptions),
        };
      },
      run: async () => runOptions(optionsStage.configured),
      observe: async (returned) =>
        digestOptions({
          before: optionsStage.before,
          after: optionsStage.configured.rollupOptions,
          returned: returned ?? null,
        }),
    },
    {
      id: "musea-build-start",
      label: "buildStart: scan + parse art files",
      units: files.length,
      unitLabel: "art files",
      prepare: async () => {
        const configured = await configurePlugin(museaEntry, workDir);
        await runOptions(configured);
        buildStartStage = configured.plugin;
      },
      run: async () => runBuildStart(buildStartStage),
      observe: async () => {
        const ids = virtualIdsFor(buildStartStage, files);
        return digestOf(loadAll(buildStartStage, ids));
      },
    },
    {
      id: "musea-load",
      label: "load: generate art modules",
      units: files.length,
      unitLabel: "art files",
      prepare: async () => {
        const plugin = await startPlugin(museaEntry, workDir);
        loadStage = { plugin, ids: virtualIdsFor(plugin, files) };
      },
      run: async () => loadAll(loadStage.plugin, loadStage.ids),
      observe: async (modules) => digestOf(modules),
    },
    {
      id: "musea-transform",
      label: "transform: TS to JS on generated modules",
      units: files.length,
      unitLabel: "art files",
      prepare: async () => {
        const plugin = await startPlugin(museaEntry, workDir);
        const ids = virtualIdsFor(plugin, files);
        transformStage = { plugin, ids, modules: loadAll(plugin, ids) };
      },
      run: async () =>
        transformAll(transformStage.plugin, transformStage.ids, transformStage.modules),
      observe: async (modules) => digestOf(modules),
    },
    {
      id: "musea-nuxt-virtual",
      label: "musea-nuxt: resolve Nuxt mock specifiers",
      units: NUXT_VIRTUAL_IDS.length * files.length,
      unitLabel: "resolutions",
      prepare: async () => {
        const { nuxtMusea } = await import(pathToFileURL(nuxtEntry).href);
        nuxtPlugin = nuxtMusea({ route: { path: "/", params: {} } });
      },
      run: async () => {
        const resolveId = hookOf(nuxtPlugin, "resolveId");
        const load = hookOf(nuxtPlugin, "load");
        // `resolveId` runs per import site, so the module graph hits it once
        // for every Nuxt specifier in every art module. `load` does not: Vite
        // caches a virtual module by id, so the two mock modules are generated
        // once per build no matter how large the gallery is. Modelling that
        // split is the point — inflating the `load` count would report a cost
        // no real build pays.
        /** @type {Set<string>} */
        const resolved = new Set();
        for (let file = 0; file < files.length; file += 1) {
          for (const id of NUXT_VIRTUAL_IDS) {
            const virtualId = resolveId(id);
            if (virtualId != null) {
              if (typeof virtualId !== "string") {
                throw new Error(`musea-stages: musea-nuxt resolved ${id} to a non-string id`);
              }
              resolved.add(virtualId);
            }
          }
        }
        return [...resolved].sort(byCodeUnit).map((id) => {
          const code = load(id);
          if (code == null) {
            throw new Error(`musea-stages: musea-nuxt did not load ${id}`);
          }
          return code;
        });
      },
      observe: async (modules) => digestOf(modules),
    },
    {
      id: "musea-plugin-total",
      label: "whole plugin: config + options + buildStart + load + transform",
      units: files.length,
      unitLabel: "art files",
      prepare: async () => {},
      run: async () => {
        const configured = await configurePlugin(museaEntry, workDir);
        const optionsReturned = await runOptions(configured);
        await runBuildStart(configured.plugin);
        const plugin = configured.plugin;
        const ids = virtualIdsFor(plugin, files);
        const modules = loadAll(plugin, ids);
        const transformed = await transformAll(plugin, ids, modules);
        return {
          rollupOptions: configured.rollupOptions,
          optionsReturned,
          modules,
          transformed,
        };
      },
      observe: async (output) => digestWholePlugin(output),
    },
  ];
}
