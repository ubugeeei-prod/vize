import { writeFileSync } from "node:fs";
import { join, relative, sep } from "node:path";

const GOLAR_CONFIG = `import { defineConfig } from "golar/unstable";
import "@golar/vue";

export default defineConfig({});
`;

export function prepareTypecheckDir({ inputDir, files, workRoot, copySelectedFiles }) {
  const outputDir = join(workRoot, `check-${files.length}`);
  copySelectedFiles(inputDir, outputDir, files, ["vize.config.json"]);
  writeFileSync(
    join(outputDir, "tsconfig.json"),
    `${JSON.stringify(
      {
        extends: relative(outputDir, join(inputDir, "tsconfig.json")).split(sep).join("/"),
        include: files,
      },
      null,
      2,
    )}\n`,
  );
  writeFileSync(join(outputDir, "golar.config.ts"), GOLAR_CONFIG);
  return outputDir;
}

export function createTypecheckToolVariants({
  fileCount,
  checkDir,
  tsconfigPath,
  corsaPath,
  resolveWorkspaceBin,
  runCommand,
}) {
  const vueTscBin = resolveWorkspaceBin("vue-tsc");
  const verterTscBin = resolveWorkspaceBin("verter-tsc");
  const golarBin = resolveWorkspaceBin("golar");
  return [
    {
      id: "vue-tsc",
      label: "vue-tsc",
      files: fileCount,
      measure: () =>
        runCommand(vueTscBin, ["--noEmit", "-p", tsconfigPath], {
          cwd: checkDir,
          allowNonZeroExit: true,
        }),
    },
    {
      id: "verter-tsc",
      label: "verter-tsc",
      files: fileCount,
      measure: () =>
        runCommand(verterTscBin, ["--noEmit", "-p", tsconfigPath], {
          cwd: checkDir,
          allowNonZeroExit: true,
          env: { VERTER_TSGO_BIN: corsaPath },
        }),
    },
    {
      id: "golar-typecheck",
      label: "Golar typecheck",
      files: fileCount,
      measure: () =>
        runCommand(golarBin, ["typecheck"], {
          cwd: checkDir,
          allowNonZeroExit: true,
        }),
    },
    {
      id: "golar-default",
      label: "Golar (lint+check)",
      files: fileCount,
      measure: () =>
        runCommand(golarBin, [], {
          cwd: checkDir,
          allowNonZeroExit: true,
        }),
    },
  ];
}

export function typecheckToolBins(optionalWorkspaceBin) {
  return {
    vueTscBin: optionalWorkspaceBin("vue-tsc"),
    verterTscBin: optionalWorkspaceBin("verter-tsc"),
    golarBin: optionalWorkspaceBin("golar"),
  };
}
