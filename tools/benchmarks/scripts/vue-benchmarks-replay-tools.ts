import { existsSync, mkdirSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

import { benchDir, reportFromText, run } from "./vue-benchmarks-replay-core.mjs";

const GOLAR_CONFIG = `import { defineConfig } from "golar/unstable";
import "@golar/vue";

export default defineConfig({});
`;

export function createReplayTools({ selectedTools, vize, tsgo, verterTsc, golar }) {
  const linkGolarPackage = (nodeModulesPath, packageName) => {
    const linkPath = join(nodeModulesPath, ...packageName.split("/"));
    if (existsSync(linkPath)) {
      return null;
    }
    mkdirSync(dirname(linkPath), { recursive: true });
    symlinkSync(join(benchDir, "node_modules", ...packageName.split("/")), linkPath, "dir");
    return linkPath;
  };
  const runGolar = (cwd, args) => {
    const configPath = join(cwd, "golar.config.ts");
    const nodeModulesPath = join(cwd, "node_modules");
    const createdLinks = [];
    writeFileSync(configPath, GOLAR_CONFIG);
    if (!existsSync(nodeModulesPath)) {
      mkdirSync(nodeModulesPath);
    }
    for (const packageName of ["golar", "@golar"]) {
      const linkPath = linkGolarPackage(nodeModulesPath, packageName);
      if (linkPath != null) {
        createdLinks.push(linkPath);
      }
    }
    try {
      return run(golar.path, args, { cwd });
    } finally {
      rmSync(configPath, { force: true });
      for (const linkPath of createdLinks.reverse()) {
        rmSync(linkPath, { recursive: true, force: true });
      }
    }
  };
  return selectedTools.map((id) => {
    if (id === "vize") {
      return {
        id,
        label: "Vize",
        version: vize.version,
        run: (cwd) => {
          const result = run(
            vize.path,
            [
              "check",
              ".",
              "--quiet",
              "--format",
              "json",
              "--tsconfig",
              "tsconfig.json",
              "--corsa-path",
              tsgo.path,
            ],
            { cwd },
          );
          let report = null;
          try {
            report = JSON.parse(result.stdout);
          } catch {
            report = null;
          }
          return { ...result, report };
        },
      };
    }
    if (id === "verter-tsc") {
      return {
        id,
        label: "verter-tsc",
        version: verterTsc.version,
        run: (cwd) => {
          const result = run(verterTsc.path, ["--noEmit", "-p", "tsconfig.json"], {
            cwd,
            env: { VERTER_TSGO_BIN: tsgo.path },
          });
          return {
            ...result,
            report: reportFromText(result.status, `${result.stdout}\n${result.stderr}`),
          };
        },
      };
    }
    if (id === "golar-typecheck") {
      return {
        id,
        label: "Golar typecheck",
        version: golar.version,
        run: (cwd) => {
          const result = runGolar(cwd, ["typecheck"]);
          return {
            ...result,
            report: reportFromText(result.status, `${result.stdout}\n${result.stderr}`),
          };
        },
      };
    }
    return {
      id,
      label: "Golar (lint+check)",
      version: golar.version,
      run: (cwd) => {
        const result = runGolar(cwd, []);
        return {
          ...result,
          report: reportFromText(result.status, `${result.stdout}\n${result.stderr}`),
        };
      },
    };
  });
}
