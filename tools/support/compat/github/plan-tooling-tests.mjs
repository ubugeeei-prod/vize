import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, matchesGlob, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { cacheInputs } from "../../../config/vite-plus/task-inputs.ts";
import {
  mergeOnlyToolingTests,
  toolingTestScopes,
} from "../../../config/vite-plus/tooling-test-scopes.ts";
import { changedPaths } from "./plan-source-checks.mjs";
import { toolingShardMatrix } from "./tooling-test-shards.ts";
import { nativeSetupCaptureRequired, toolingChecksRequired } from "./native-setup-capture.mjs";

import { nativeVaporCaptureRequired } from "./native-vapor-capture.mjs";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const globalInputs = [...cacheInputs.workspace, "pnpm-workspace.yaml"];
const matchesAny = (path, patterns) => patterns.some((pattern) => matchesGlob(path, pattern));

export function toolingTestFiles(cwd = root) {
  return ["tests/tooling", "tests/tooling/davinci", "tests/tooling/release"]
    .flatMap((directory) =>
      readdirSync(join(cwd, directory))
        .filter((name) => /\.test\.(?:ts|mjs)$/.test(name))
        .map((name) => `${directory}/${name}`),
    )
    .sort((left, right) => (left < right ? -1 : left > right ? 1 : 0));
}

// Add every literal local import, including transitive helpers, to the input
// set. Unresolved imports restore broad inputs rather than making a skip safe.
export function localImportInputs(file, cwd = root, visited = new Set()) {
  if (visited.has(file)) return { files: [], complete: true };
  visited.add(file);
  const files = [file];
  let complete = true;
  const source = readFileSync(join(cwd, file), "utf8");
  const imports = /\b(?:from\s*|import\s*(?:\(\s*)?)["']([^"']+)["']/g;
  for (const [, specifier] of source.matchAll(imports)) {
    if (!specifier.startsWith(".")) continue;
    const dependency = relative(cwd, resolve(cwd, dirname(file), specifier));
    if (dependency.startsWith("..") || !existsSync(join(cwd, dependency))) {
      complete = false;
      continue;
    }
    const child = localImportInputs(dependency, cwd, visited);
    files.push(...child.files);
    complete &&= child.complete;
  }
  // Nonliteral imports cannot establish a complete dependency closure.
  if (/\bimport\s*\(\s*[^"'\s]/.test(source)) complete = false;
  return { files, complete };
}

export function planToolingTests(paths, { tier = "pr", cwd = root } = {}) {
  if (!["pr", "merge"].includes(tier)) throw new Error("tier must be pr or merge");
  const files = toolingTestFiles(cwd);
  const mergeOnly = new Set(mergeOnlyToolingTests);
  const unknown =
    paths.length === 0 ||
    paths.some((path) => !matchesAny(path, [...cacheInputs.tooling, ...globalInputs]));
  const global = paths.some((path) => matchesAny(path, globalInputs));
  const tests = files.filter((file) => {
    if (tier === "merge") return true;
    if (mergeOnly.has(file)) return false;
    if (unknown || global || paths.includes(file)) return true;
    const scope = toolingTestScopes.find((entry) => matchesAny(file, entry.tests));
    const imports = localImportInputs(file, cwd);
    const inputs = scope && imports.complete ? cacheInputs[scope.input] : cacheInputs.tooling;
    return paths.some((path) => matchesAny(path, [...inputs, ...imports.files]));
  });
  return {
    version: 1,
    tier,
    tests,
    totalTests: files.length,
    deferredTests: tier === "merge" ? 0 : files.filter((file) => mergeOnly.has(file)).length,
    reason:
      tier === "merge"
        ? "full merge suite"
        : unknown
          ? "unknown inputs: broad PR suite"
          : global
            ? "shared task or dependency inputs"
            : "declared task and import inputs",
  };
}

export function writeToolingPlan(plan, output) {
  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, `${JSON.stringify(plan, null, 2)}\n`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [base, head, ...options] = process.argv.slice(2);
  if (![base, head].every((sha) => /^[0-9a-f]{40}$/.test(sha ?? ""))) {
    throw new Error("expected full base and head commit SHAs");
  }
  let tier = "pr";
  let output = "target/tooling-test-plan.json";
  let githubOutput = false;
  for (let index = 0; index < options.length; index += 1) {
    if (options[index] === "--tier") tier = options[++index];
    else if (options[index] === "--output") output = options[++index];
    else if (options[index] === "--github-output") githubOutput = true;
    else throw new Error(`unknown option: ${options[index]}`);
  }
  const paths = /^0+$/.test(base) ? [] : changedPaths(base, head, root);
  const plan = planToolingTests(paths, { tier });
  writeToolingPlan(plan, output);
  if (githubOutput && process.env.GITHUB_OUTPUT) {
    appendFileSync(
      process.env.GITHUB_OUTPUT,
      `tooling=${toolingChecksRequired(plan, paths) || nativeVaporCaptureRequired(paths)}\nnative-vapor-capture=${nativeVaporCaptureRequired(paths)}\nnative-setup-capture=${nativeSetupCaptureRequired(paths)}\ntooling-matrix=${JSON.stringify(toolingShardMatrix(plan))}\n`,
    );
  }
  const message = `Tooling tests: ${plan.tests.length}/${plan.totalTests}; ${plan.deferredTests} deferred to T1; ${plan.reason}.\n`;
  process.stdout.write(message);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, message);
}
