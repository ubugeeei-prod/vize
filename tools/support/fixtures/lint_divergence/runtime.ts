import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { spawnSync } from "node:child_process";
import {
  get,
  number,
  object,
  projectString,
  projectStrings,
  readJson,
  text,
  errorMessage,
  parseJsonSource,
  canonicalJson,
  type Json,
} from "./value.ts";
import type { Args } from "./selection.ts";

export interface Launch {
  command: string;
  prefix: string[];
}
export function resolveVizeLaunch(root: string, vizeBin: string | null): Launch {
  const executable = process.platform === "win32" ? "vize.exe" : "vize";
  const absolute = (filename: string) =>
    path.isAbsolute(filename) ? filename : path.join(root, filename);
  const candidates = [vizeBin, process.env.VIZE_BIN]
    .filter((value): value is string => value !== undefined && value !== null)
    .map(absolute);
  candidates.push(
    ...["ci", "debug", "release"].map((profile) => path.join(root, "target", profile, executable)),
  );
  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      let resolved = candidate;
      try {
        resolved = fs.realpathSync.native(candidate);
      } catch {
        /* Match Rust's best-effort canonicalization. */
      }
      return { command: resolved, prefix: [] };
    }
  }
  return { command: "cargo", prefix: ["run", "-q", "-p", "vize", "--"] };
}
export function collectRunEvidence(root: string): Json {
  const git = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  return {
    commitSha: process.env.GITHUB_SHA ?? git.stdout?.trim() ?? "",
    runtime: { name: "node", version: process.version },
    machine: {
      platform: process.platform === "win32" ? "windows" : process.platform,
      arch: process.arch,
      logicalCpuCount: os.availableParallelism(),
    },
  };
}
export function runPatina(
  cwd: string,
  project: Json,
  args: Args,
  launch: Launch,
  expectedCount: number,
) {
  const argv = [
    ...launch.prefix,
    "lint",
    ...projectStrings(project, "vueGlobs"),
    "--format",
    "json",
    "--no-config",
  ];
  if (args.preset !== null) argv.push("--preset", args.preset);
  const output = spawnSync(launch.command, argv, {
    cwd,
    encoding: "utf8",
    env: { ...process.env, LANG: "C", LC_ALL: "C" },
    stdio: ["ignore", "pipe", "pipe"],
    maxBuffer: Infinity,
  });
  if (output.error) throw new Error(`vize lint failed to run: ${output.error.message}`);
  const status = output.status ?? 1;
  if (status !== 0 && status !== 1)
    throw new Error(`vize lint exited with unsupported status ${status}: ${output.stderr}`);
  let envelope: Json;
  try {
    envelope = parseJsonSource(output.stdout);
  } catch (error) {
    throw new Error(`vize lint produced invalid JSON: ${errorMessage(error)}`);
  }
  if (!Array.isArray(envelope)) throw new Error("vize lint envelope must be an array");
  if (envelope.length !== expectedCount)
    throw new Error(
      `${projectString(project, "id")}: vize lint reported ${envelope.length} files, expected ${expectedCount}`,
    );
  const findings: Json[] = [];
  for (const entry of envelope) {
    const file = get(entry, "file");
    if (typeof file !== "string") throw new Error("vize lint entry must carry file");
    const messages = get(entry, "messages");
    if (!Array.isArray(messages)) throw new Error("vize lint entry must carry messages");
    for (const message of messages) {
      // serde_json indexing upgrades null to an object and rejects other scalars.
      const value = object(message);
      if (message !== null && !value) throw new Error("vize lint message must be an object");
      findings.push({ ...value, file });
    }
  }
  return { results: envelope, findings };
}

export const baselineScript =
  '\nimport { createRequire } from "node:module";\nimport { readFileSync } from "node:fs";\nimport { baselineConfig } from "./tools/support/compat/fixtures/lint-divergence-baseline.mjs";\nconst input = JSON.parse(readFileSync(0, "utf8"));\nconst requireFromBench = createRequire(input.benchPackageJson);\nconst ESLint = requireFromBench("eslint").ESLint;\nconst plugin = requireFromBench("eslint-plugin-vue");\nconst vueParser = requireFromBench("vue-eslint-parser");\nconst scriptParser = requireFromBench("@typescript-eslint/parser");\nconst manifest = requireFromBench("eslint-plugin-vue/package.json");\nconst eslint = new ESLint({\n  cwd: input.cwd,\n  passOnNoPatterns: input.files.length === 0,\n  overrideConfigFile: true,\n  overrideConfig: baselineConfig({ vueParser, scriptParser, plugin }, input.rules),\n  errorOnUnmatchedPattern: false\n});\nif (input.files.length === 0) {\n  await eslint.calculateConfigForFile("__vize_empty_vue_config__.vue");\n}\nconst results = await eslint.lintFiles(input.files);\nconst enabled = new Set(Object.keys(input.rules));\nlet droppedConfigMessageCount = 0;\nconst retained = results.map((result) => ({\n  ...result,\n  messages: result.messages.filter((message) => {\n    if (message.ruleId == null || enabled.has(message.ruleId)) return true;\n    droppedConfigMessageCount += 1;\n    return false;\n  })\n}));\nconst runtimeVersions = Object.fromEntries(\n  ["eslint", "eslint-plugin-vue", "vue-eslint-parser", "@typescript-eslint/parser"]\n    .map(name => [name, requireFromBench(`${name}/package.json`).version])\n);\nprocess.stdout.write(JSON.stringify({ version: manifest.version, runtimeVersions, rawResults: results, results: retained, droppedConfigMessageCount }));\n';

export function runEslintBaseline(
  root: string,
  cwd: string,
  files: string[],
  rules: Record<string, string>,
) {
  if (!Object.keys(rules).length) {
    const version = get(
      readJson(path.join(root, "tests/_fixtures/patina-eslint-vue-rule-map.json")),
      "upstream",
      "version",
    );
    if (typeof version !== "string") throw new Error("rule map must record upstream version");
    const results = files.map((file): Json => ({ filePath: path.join(cwd, file), messages: [] }));
    return {
      version,
      runtimeVersions: {},
      rawResults: results,
      results,
      droppedConfigMessageCount: 0,
    };
  }
  const input: Json = {
    benchPackageJson: path.join(root, "tools/benchmarks/scripts/package.json"),
    cwd,
    files,
    rules,
  };
  const output = spawnSync("node", ["--input-type=module", "--eval", baselineScript], {
    cwd: root,
    encoding: "utf8",
    input: canonicalJson(input),
    maxBuffer: Infinity,
  });
  if (output.error) throw new Error(`eslint baseline failed to start: ${output.error.message}`);
  if (output.status !== 0)
    throw new Error(
      `eslint baseline exited ${output.status ?? 1}:\n${output.stdout}${output.stderr}`,
    );
  let parsed: Json;
  try {
    parsed = parseJsonSource(output.stdout);
  } catch (error) {
    throw new Error(`eslint baseline produced invalid JSON: ${errorMessage(error)}`);
  }
  const rawResults = get(parsed, "rawResults"),
    results = get(parsed, "results");
  if (!Array.isArray(rawResults))
    throw new Error("eslint baseline must retain complete raw results");
  if (!Array.isArray(results)) throw new Error("eslint baseline result must list results");
  return {
    rawResults,
    results,
    runtimeVersions: get(parsed, "runtimeVersions"),
    version: text(get(parsed, "version")),
    droppedConfigMessageCount: number(get(parsed, "droppedConfigMessageCount")),
  };
}
