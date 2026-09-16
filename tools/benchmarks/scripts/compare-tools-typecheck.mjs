import { existsSync, mkdirSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join, relative, sep } from "node:path";
import { fileURLToPath } from "node:url";

import { resolveVuePackageDir } from "./check-gate-env.mjs";
import { runTypecheckCommand } from "./typecheck-command.mjs";

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

export function prepareTypecheckDir({ inputDir, files, workRoot, copySelectedFiles }) {
  const outputDir = join(workRoot, `check-${files.length}`);
  copySelectedFiles(inputDir, outputDir, files, ["vize.config.json"]);
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
  prepareTypecheckPackages(outputDir);
  return outputDir;
}

export function createTypecheckToolVariants({
  fileCount,
  vizeBin,
  corsaPath,
  resolveWorkspaceBin,
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
  ];
}

export function typecheckToolBins(optionalWorkspaceBin) {
  return {
    vueTscBin: optionalWorkspaceBin("vue-tsc"),
    verterTscBin: optionalWorkspaceBin("verter-tsc"),
    golarBin: optionalWorkspaceBin("golar"),
  };
}
