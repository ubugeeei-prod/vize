import { appendFileSync, readdirSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { immutableRuntimeBundle, sha256 } from "./runtime-bundle-cache.ts";
import {
  boundedRuntimeEnvironment,
  runtimeBundleChunk,
  runtimeBundleOptions,
  runtimeModulesCovered,
} from "./runtime-bundle-context.ts";
import { fingerprintFiles, packageInputFiles } from "./runtime-bundle-inputs.ts";
import { vueVaporRuntimeEntry, vueVaporVersion } from "./vue-vapor-release.mjs";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const fromRoot = createRequire(join(root, "package.json"));

export async function pinnedVueRuntimeBundle(production: boolean) {
  const options = runtimeBundleOptions(vueVaporRuntimeEntry, production);
  const initialEnv = { ...process.env };
  const context = boundedRuntimeEnvironment(realpathSync(process.cwd()), initialEnv);
  let inputMs = 0,
    inventoryMs = 0,
    hashMs = 0,
    abiMs = 0,
    buildMs = 0,
    inputFiles: string[] = [],
    identity: Record<string, unknown> | null = null;
  const inputs = () => {
    if (initialEnv.VIZE_VUE_RUNTIME_BUNDLE_CACHE === "off" || context === null) return null;
    const start = performance.now();
    try {
      const inventoryStart = performance.now();
      const packages = packageInputFiles([vueVaporRuntimeEntry, fromRoot.resolve("vite-plus")]);
      const helpers = readdirSync(import.meta.dirname)
        .filter((name) => /\.(?:mjs|ts)$/.test(name))
        .map((name) => join(import.meta.dirname, name));
      const manifests = [
        "package.json",
        "pnpm-lock.yaml",
        "pnpm-workspace.yaml",
        "npm/ui/package.json",
      ].map((name) => join(root, name));
      for (let directory = context.cwd; ; directory = dirname(directory)) {
        manifests.push(join(directory, "package.json"));
        if (dirname(directory) === directory) break;
      }
      inputFiles = [
        ...packages.files,
        ...helpers,
        ...manifests,
        ...context.envFiles.map(({ path }) => path),
      ];
      inventoryMs += performance.now() - inventoryStart;
      const abiStart = performance.now();
      const glibc =
        (process.report.getReport() as { header?: { glibcVersionRuntime?: string } }).header
          ?.glibcVersionRuntime ?? null;
      abiMs += performance.now() - abiStart;
      const hashStart = performance.now();
      const files = fingerprintFiles(inputFiles);
      hashMs += performance.now() - hashStart;
      identity = {
        options,
        context,
        vue: vueVaporVersion,
        packages: packages.packages,
        missing: packages.missing,
        node: process.version,
        platform: process.platform,
        arch: process.arch,
        nodeVersions: process.versions,
        nodeTarget: process.config.variables.node_target_type,
        shlibSuffix: process.config.variables.shlib_suffix,
        glibc,
        files,
      };
      return identity;
    } catch {
      return null;
    } finally {
      inputMs += performance.now() - start;
    }
  };
  let versions: Record<string, string> | null = null,
    reusable = true;
  const bundle = await immutableRuntimeBundle({
    directory:
      initialEnv.VIZE_VUE_RUNTIME_BUNDLE_DIRECTORY ??
      join(initialEnv.RUNNER_TEMP ?? tmpdir(), `vize-vue-runtime-${sha256(root)}`),
    inputs,
    canStore: (result) =>
      reusable && runtimeModulesCovered(result.modules, inputFiles, vueVaporRuntimeEntry),
    build: async () => {
      const start = performance.now();
      try {
        const vite = await import("vite-plus");
        versions = { vite: vite.version, rolldown: vite.rolldownVersion };
        let modules: string[] = [];
        const result = await vite.build({
          ...options,
          plugins: [
            {
              name: "runtime-bundle-input-proof",
              generateBundle() {
                modules = [...this.getModuleIds()].sort();
              },
            },
          ],
        });
        const outputs = Array.isArray(result) ? result : [result];
        const chunk = runtimeBundleChunk(outputs.filter((output) => "output" in output));
        reusable = chunk.reusable;
        return { code: chunk.code, modules };
      } finally {
        buildMs += performance.now() - start;
      }
    },
  });
  // The original no-env-file Vite build sets this default even with write:false.
  if (bundle.cache === "hit" && !initialEnv.NODE_ENV) process.env.NODE_ENV = "production";
  if (initialEnv.VIZE_VUE_RUNTIME_BUNDLE_TIMINGS)
    appendFileSync(
      initialEnv.VIZE_VUE_RUNTIME_BUNDLE_TIMINGS,
      `${JSON.stringify({
        pid: process.pid,
        production,
        cwd: process.cwd(),
        cache: bundle.cache,
        inputMs,
        inventoryMs,
        hashMs,
        abiMs,
        buildMs,
        versions,
        modules: bundle.modules,
        bundleSha256: sha256(bundle.code),
        inputSha256: identity ? sha256(JSON.stringify(identity)) : null,
        inputFileCount: inputFiles.length,
      })}\n`,
    );
  return bundle.code;
}
