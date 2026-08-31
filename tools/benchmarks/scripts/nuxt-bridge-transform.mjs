/**
 * Cost of the Vize Nuxt module's own per-module bridge transform (#3426).
 *
 * The published Nuxt row is an end-to-end `nuxt build`, where Vue SFC
 * compilation is ~2% of the total and everything the Vize Nuxt module itself
 * does sits below the run-to-run noise floor. So the module's own cost has no
 * signal there. This benchmark supplies one: it feeds N Vize-emitted modules
 * through `vizejs:nuxt-transform-bridge`'s four stages and reports, per stage,
 * how many modules actually do work and how long the stage takes.
 *
 * The asymmetry is the point. Three stages are gated by the cheap regexes in
 * `npm/framework/nuxt/src/bridge-fast-path.ts` and skip modules that cannot
 * need them. The composable auto-import stage (`unimportCtx.injectImports`,
 * `npm/framework/nuxt/src/index.ts`) is **not** gated and runs on every module.
 *
 *   node tools/benchmarks/scripts/nuxt-bridge-transform.mjs
 *   node tools/benchmarks/scripts/nuxt-bridge-transform.mjs --modules 2000 --runs 5
 *
 * `injectImports` is only timed when `unimport` resolves from this checkout. It
 * is Nuxt's dependency, not the Vize module's, so a bare workspace cannot
 * measure it; the run says so rather than guessing.
 */

import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

import {
  hasComponentBridgeInput,
  hasI18nBridgeInput,
  hasStableKeyBridgeInput,
} from "../npm/framework/nuxt/src/bridge-fast-path.ts";

function parseArgs(argv) {
  const options = { modules: 1000, runs: 3 };
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === "--modules") options.modules = Number(argv[++i]);
    else if (argv[i] === "--runs") options.runs = Number(argv[++i]);
  }
  return options;
}

/**
 * A Vize-emitted component module, shaped like what the plugin hands the bridge:
 * a render function over `resolveComponent`/`createElementBlock`, plus a setup
 * scope. One module in ten references an auto-imported Nuxt component, one in
 * ten calls an i18n helper, and one in ten uses `useFetch`, which is roughly how
 * often those appear in the 500-file synthetic Nuxt app the published row uses.
 */
function makeModule(index) {
  const componentBridge = index % 10 === 0 ? '  const _c = _resolveComponent("NuxtLink")\n' : "";
  const i18nBridge = index % 10 === 1 ? "  const label = $t('nav.title')\n" : "";
  const stableKey = index % 10 === 2 ? "  const { data } = useFetch('/api/items')\n" : "";
  return `import { openBlock as _openBlock, createElementBlock as _createElementBlock, toDisplayString as _toDisplayString } from "vue"

export default {
  __name: "Component${index}",
  setup(__props) {
    const count = ref(0)
    const route = useRoute()
${componentBridge}${i18nBridge}${stableKey}    return (_ctx, _cache) => {
      return (_openBlock(), _createElementBlock("div", null, _toDisplayString(count.value) + " " + _toDisplayString(route.path), 1 /* TEXT */))
    }
  }
}
`;
}

/** Resolve Nuxt's `unimport` if this checkout happens to have it. */
async function loadUnimport() {
  const require = createRequire(pathToFileURL(`${process.cwd()}/`).href);
  try {
    return await import(pathToFileURL(require.resolve("unimport")).href);
  } catch {
    return null;
  }
}

function timed(fn, runs) {
  let matched = 0;
  const started = process.hrtime.bigint();
  for (let run = 0; run < runs; run += 1) matched = fn();
  const elapsedMs = Number(process.hrtime.bigint() - started) / 1e6 / runs;
  return { elapsedMs, matched };
}

const { modules: moduleCount, runs } = parseArgs(process.argv.slice(2));
const sources = Array.from({ length: moduleCount }, (_, index) => makeModule(index));

const gates = [
  { name: "1. component auto-imports", gate: hasComponentBridgeInput },
  { name: "2. i18n helper injection", gate: hasI18nBridgeInput },
  { name: "4. stable injected keys", gate: hasStableKeyBridgeInput },
];

console.log(`modules: ${moduleCount}, runs: ${runs}`);
console.log("| stage | gated | modules doing work | ms per pass | ms per module |");
console.log("| --- | --- | ---: | ---: | ---: |");

for (const { name, gate } of gates) {
  const { elapsedMs, matched } = timed(() => sources.filter((source) => gate(source)).length, runs);
  console.log(
    `| ${name} | yes | ${matched} / ${moduleCount} | ${elapsedMs.toFixed(3)} | ${(elapsedMs / moduleCount).toFixed(6)} |`,
  );
}

const unimport = await loadUnimport();
if (!unimport) {
  console.log(
    `| 3. composable auto-imports | **no** | ${moduleCount} / ${moduleCount} | not measured | not measured |`,
  );
  console.log("");
  console.log(
    "`unimport` does not resolve from this checkout, so stage 3 was not timed. It is Nuxt's",
  );
  console.log(
    "dependency: run this from a Nuxt app's node_modules to time it. The counts above still hold —",
  );
  console.log(
    "stage 3 has no fast-path gate, so it reaches every module while its three siblings skip most.",
  );
} else {
  const ctx = unimport.createUnimport({
    presets: [{ from: "vue", imports: ["ref", "computed", "watch"] }],
    imports: [{ name: "useRoute", from: "#app" }],
  });
  await ctx.init();
  const started = process.hrtime.bigint();
  for (let run = 0; run < runs; run += 1) {
    for (const [index, source] of sources.entries()) {
      await ctx.injectImports(source, `\0/src/Component${index}.vue.ts?vue&vize`);
    }
  }
  const elapsedMs = Number(process.hrtime.bigint() - started) / 1e6 / runs;
  console.log(
    `| 3. composable auto-imports | **no** | ${moduleCount} / ${moduleCount} | ${elapsedMs.toFixed(3)} | ${(elapsedMs / moduleCount).toFixed(6)} |`,
  );
}
