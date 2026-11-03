import { existsSync, mkdirSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

import { resolveVuePackageDir } from "./check-gate-env.mjs";
import { createSurface, ENGINE_CLASSES_BY_SURFACE } from "./compare-tools-report.mjs";
import { runTypecheckCommand } from "./typecheck-command.mjs";
import {
  createMapperLanes,
  mapperHostBins,
  prepareMapperPackages,
  resolveMapperHosts,
} from "./typecheck-mapper-hosts.ts";
import { gateTypecheckVariants, recordTypecheckRejection } from "./typecheck-readiness.mjs";

const benchDir = dirname(fileURLToPath(import.meta.url));

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
 * @param {{
 *   fileCount: number,
 *   vizeBin: string,
 *   corsaPath: string,
 *   resolveWorkspaceBin: (name: string) => string,
 *   mapperHosts?: import("./typecheck-mapper-hosts.ts").MapperHosts | null,
 *   runCommand?: Function,
 * }} options
 */
export function createTypecheckToolVariants({
  fileCount,
  vizeBin,
  corsaPath,
  resolveWorkspaceBin,
  mapperHosts = null,
  runCommand = runTypecheckCommand,
}) {
  const vueTscBin = resolveWorkspaceBin("vue-tsc");
  const verterTscBin = resolveWorkspaceBin("verter-tsc");
  const golarBin = resolveWorkspaceBin("golar");
  return [
    {
      id: "vue-tsc",
      label: "vue-tsc",
      files: fileCount,
      run: (cwd) =>
        runCommand(vueTscBin, ["--noEmit", "-p", join(cwd, "tsconfig.json"), "--pretty", "false"], {
          cwd,
        }),
    },
    {
      id: "verter-tsc",
      label: "verter-tsc",
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
      files: fileCount,
      run: (cwd) =>
        runCommand(golarBin, ["typecheck"], {
          cwd,
        }),
    },
    {
      id: "golar-default",
      label: "Golar (lint+check)",
      files: fileCount,
      run: (cwd) =>
        runCommand(golarBin, [], {
          cwd,
        }),
    },
    ...["1t", "max"].map((lane) => ({
      id: `vize-check-${lane}`,
      label: `Vize check (${lane === "1t" ? "1T" : "max"})`,
      files: fileCount,
      format: "json",
      run: (cwd) =>
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
            ...(lane === "1t" ? ["--servers", "1"] : []),
          ],
          { cwd, env: lane === "1t" ? { RAYON_NUM_THREADS: "1" } : {} },
        ),
    })),
    ...(mapperHosts ? createMapperLanes({ fileCount, hosts: mapperHosts, runCommand }) : []),
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
 * `prepareDir` copies the corpus and installs each project's packages.
 */
export async function measureTypecheckSurface({
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
  const variants = gateTypecheckVariants(
    createTypecheckToolVariants({
      fileCount: files.length,
      vizeBin,
      corsaPath: options.backend.corsaPath,
      resolveWorkspaceBin,
      mapperHosts,
    }),
    checkDir,
    prepareProject,
    onRejected,
  );

  return createSurface({
    id: "check",
    label: "Type check",
    files: files.length,
    bytes,
    variants: (await measureVariants(variants, options, onRejected)).map((measured) => ({
      ...measured,
      correctness: variants.find((variant) => variant.id === measured.id).correctness,
    })),
    rejectedVariants,
    baselineId: "vue-tsc",
    vizeSingleId: "vize-check-1t",
    vizeMaxId: "vize-check-max",
    // vue-tsc drives the JavaScript TypeScript compiler; vize check drives
    // native tsgo. Declaring the classes suppresses the cross-engine ratio.
    engineClasses: ENGINE_CLASSES_BY_SURFACE.check,
  });
}
