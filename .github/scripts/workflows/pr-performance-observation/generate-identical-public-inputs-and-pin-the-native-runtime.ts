import { requiredEnv } from "../shared/files.ts";
import { pathToFileURL } from "node:url";
import { resolve as resolveWorkflowInput } from "node:path";
import { appendFileSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
const { generateCorpus } = (await import(
  pathToFileURL(resolveWorkflowInput("./tools/benchmarks/scripts/generate.mjs")).href
)) as typeof import("../../../../tools/benchmarks/scripts/generate.mjs");
const require = createRequire(resolve("tests/package.json"));
const vue = dirname(realpathSync(require.resolve("vue/package.json")));
const corsa = realpathSync("node_modules/@typescript/typescript-linux-x64/lib/tsc");
const dir = join(requiredEnv("RUNNER_TEMP"), "pr-performance-inputs");
generateCorpus({ fileCount: 1000, benchDir: dir });
const config = JSON.parse(readFileSync(join(dir, "tsconfig.json"), "utf8"));
config.compilerOptions.paths.vue = [vue];
writeFileSync(join(dir, "tsconfig.json"), JSON.stringify(config, null, 2) + "\n");
appendFileSync(
  requiredEnv("GITHUB_ENV"),
  `OBSERVATION_INPUT=${dir}\nOBSERVATION_VUE=${vue}\nOBSERVATION_CORSA=${corsa}\n`,
);
