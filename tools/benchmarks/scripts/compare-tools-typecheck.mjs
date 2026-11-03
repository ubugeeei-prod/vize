import { existsSync, mkdirSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { availableParallelism, tmpdir } from "node:os";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

import { resolveVuePackageDir } from "./check-gate-env.mjs";
import { createSurface, ENGINE_CLASSES_BY_SURFACE } from "./compare-tools-report.mjs";
import { runTypecheckCommand } from "./typecheck-command.mjs";
import {
  CHECKER_SETTINGS,
  createMapperLanes,
  mapperHostBins,
  prepareMapperPackages,
  resolveMapperHosts,
  settingLabel,
  vizeCheckerLaneId,
} from "./typecheck-mapper-hosts.ts";
import { gateTypecheckVariants, recordTypecheckRejection } from "./typecheck-readiness.mjs";

const benchDir = dirname(fileURLToPath(import.meta.url));

/** vue-tsc runs on Node's default heap otherwise, which a 10,000-SFC project overflows. */
export const VUE_TSC_NODE_OPTIONS = "--max-old-space-size=65536";

const GOLAR_CONFIG = `import { defineConfig } from "golar/unstable";
import "@golar/vue";

export default defineConfig({});
`;

export function prepareTypecheckPackages(dir) {
  const packages = [
    ["golar", join(benchDir, "node_modules", "golar")],
    ["@golar", join(benchDir, "node_modules", "@golar")],
    ["vue", resolveVuePackageDir()],
  ];
  for (const [name, source] of packages) {
    const target = join(dir, "node_modules", name);
    if (!existsSync(target)) {
      mkdirSync(dirname(target), { recursive: true });
      symlinkSync(source, target, "dir");
    }
  }
  writeFileSync(join(dir, "golar.config.ts"), GOLAR_CONFIG);
}

export function prepareTypecheckDir({
  inputDir,
  files,
  workRoot,
  copySelectedFiles,
  prepareProject = prepareTypecheckPackages,
}) {
  const outputDir = join(workRoot, `check-${files.length}`);
  copySelectedFiles(inputDir, outputDir, files, ["vite.config.mjs"]);
  writeFileSync(
    join(outputDir, "tsconfig.json"),
    `${JSON.stringify(
      {
        extends: relative(outputDir, join(inputDir, "tsconfig.json")).split(sep).join("/"),
        vueCompilerOptions: { strictTemplates: true },
        include: files,
      },
      null,
      2,
    )}\n`,
  );
  prepareProject(outputDir);
  return outputDir;
}

/**
 * `vize check` keeps its generated project and tsgo's `--incremental` build
 * info in the user cache, so a second run over an unchanged project replays
 * the first one's diagnostics instead of checking. Every invocation gets an
 * empty cache (`XDG_CACHE_HOME`, which `dirs::cache_dir` reads on Linux), so
 * each timed run does the full check the other lanes do.
 */
function withEmptyCache(run) {
  const cache = mkdtempSync(join(tmpdir(), "vize-check-cache-"));
  try {
    return run(cache);
  } finally {
    rmSync(cache, { recursive: true, force: true });
  }
}

/**
 * @param {{
 *   fileCount: number,
 *   vizeBin: string,
 *   corsaPath: string,
 *   resolveWorkspaceBin: (name: string) => string,
 *   mapperHosts?: import("./typecheck-mapper-hosts.ts").MapperHosts | null,
 *   cpus?: number,
 *   runCommand?: Function,
 * }} options
 */
export function createTypecheckToolVariants({
  fileCount,
  vizeBin,
  corsaPath,
  resolveWorkspaceBin,
  mapperHosts = null,
  cpus = availableParallelism(),
  runCommand = runTypecheckCommand,
}) {
  const vueTscBin = resolveWorkspaceBin("vue-tsc");
  const verterTscBin = resolveWorkspaceBin("verter-tsc");
  const golarBin = resolveWorkspaceBin("golar");
  return [
    {
      id: "vue-tsc",
      label: "vue-tsc",
      tool: "vue-tsc",
      setting: "default",
      files: fileCount,
      run: (cwd) =>
        runCommand(vueTscBin, ["--noEmit", "-p", join(cwd, "tsconfig.json"), "--pretty", "false"], {
          cwd,
          env: { NODE_OPTIONS: VUE_TSC_NODE_OPTIONS },
        }),
    },
    {
      id: "verter-tsc",
      label: "verter-tsc",
      tool: "verter-tsc",
      setting: "default",
      files: fileCount,
      run: (cwd) =>
        runCommand(verterTscBin, ["--noEmit", "-p", join(cwd, "tsconfig.json")], {
          cwd,
          env: { VERTER_TSGO_BIN: corsaPath },
        }),
    },
    {
      id: "golar-typecheck",
      label: "Golar typecheck",
      tool: "Golar typecheck",
      setting: "default",
      files: fileCount,
      run: (cwd) =>
        runCommand(golarBin, ["typecheck"], {
          cwd,
        }),
    },
    {
      id: "golar-default",
      label: "Golar (lint+check)",
      tool: "Golar (lint+check)",
      setting: "default",
      files: fileCount,
      run: (cwd) =>
        runCommand(golarBin, [], {
          cwd,
        }),
    },
    ...[
      {
        id: "vize-check-1t",
        label: "Vize check (1T)",
        setting: "--servers 1, RAYON_NUM_THREADS=1",
        args: ["--servers", "1"],
        env: { RAYON_NUM_THREADS: "1" },
      },
      { id: "vize-check-max", label: "Vize check (max)", setting: "auto servers", args: [] },
      ...CHECKER_SETTINGS.map((setting) => ({
        id: vizeCheckerLaneId(setting),
        label: `Vize check (${settingLabel(setting, cpus)})`,
        setting: settingLabel(setting, cpus),
        args: setting.args(cpus),
      })),
    ].map(({ id, label, setting, args, env = {} }) => ({
      id,
      label,
      tool: "vize check",
      setting,
      files: fileCount,
      format: "json",
      run: (cwd) =>
        withEmptyCache((cache) =>
          runCommand(
            vizeBin,
            [
              "check",
              ".",
              "--quiet",
              "--format",
              "json",
              "--tsconfig",
              join(cwd, "tsconfig.json"),
              "--corsa-path",
              corsaPath,
              ...args,
            ],
            { cwd, env: { ...env, XDG_CACHE_HOME: cache } },
          ),
        ),
    })),
    ...(mapperHosts ? createMapperLanes({ fileCount, hosts: mapperHosts, cpus, runCommand }) : []),
  ];
}

export function typecheckToolBins(optionalWorkspaceBin) {
  return {
    vueTscBin: optionalWorkspaceBin("vue-tsc"),
    verterTscBin: optionalWorkspaceBin("verter-tsc"),
    golarBin: optionalWorkspaceBin("golar"),
    ...mapperHostBins(),
  };
}

/**
 * Gate every lane on the planted diagnostics, then measure the survivors.
 * `prepareDir` writes the project and installs its packages; `id` picks the
 * surface's lanes from ENGINE_CLASSES_BY_SURFACE.
 */
export async function measureTypecheckSurface({
  id,
  label,
  files,
  bytes,
  options,
  prepareDir,
  resolveWorkspaceBin,
  measureVariants,
}) {
  const vizeBin = resolve(options.vizeBin);
  if (!existsSync(vizeBin)) {
    throw new Error(`Vize CLI not found: ${vizeBin}`);
  }
  // Fail closed: without a resolvable native TypeScript engine `vize check`
  // is not measuring type checking, and a timing published from it would be
  // the heuristic-fallback result the upstream vue-benchmarks report hit.
  if (!options.backend.ready) {
    throw new Error(`Type-check surface requires a ready tsgo backend: ${options.backend.reason}`);
  }
  const mapperHosts = resolveMapperHosts();
  const prepareProject = (dir) => {
    prepareTypecheckPackages(dir);
    prepareMapperPackages(dir, mapperHosts);
  };
  const checkDir = prepareDir(prepareProject);
  const rejectedVariants = [];
  const onRejected = (variant, error, phase) =>
    recordTypecheckRejection(rejectedVariants, variant, error, phase);
  const engineClasses = ENGINE_CLASSES_BY_SURFACE[id];
  const variants = gateTypecheckVariants(
    createTypecheckToolVariants({
      fileCount: files,
      vizeBin,
      corsaPath: options.backend.corsaPath,
      resolveWorkspaceBin,
      mapperHosts,
    }).filter((variant) => variant.id in engineClasses),
    checkDir,
    prepareProject,
    onRejected,
  );

  return createSurface({
    id,
    label,
    files,
    bytes,
    variants: (await measureVariants(variants, options, onRejected)).map((measured) => ({
      ...measured,
      correctness: variants.find((variant) => variant.id === measured.id).correctness,
    })),
    rejectedVariants,
    baselineId: "vue-tsc",
    vizeSingleId: "vize-check-1t" in engineClasses ? "vize-check-1t" : null,
    vizeMaxId: "vize-check-max",
    // vue-tsc drives the JavaScript TypeScript compiler; vize check drives
    // native tsgo. Declaring the classes suppresses the cross-engine ratio.
    engineClasses,
  });
}
