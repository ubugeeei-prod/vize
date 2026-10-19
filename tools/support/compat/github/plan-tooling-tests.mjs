import { appendFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, matchesGlob, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { cacheInputs } from "../../../config/vite-plus/task-inputs.ts";
import {
  mergeOnlyToolingTests,
  toolingTestScopes,
} from "../../../config/vite-plus/tooling-test-scopes.ts";
import { changedPaths } from "./plan-source-checks.ts";
import {
  toolingLeadSteps,
  toolingShardMatrix,
  toolingShards,
  toolingTestFiles,
} from "./tooling-test-shards.ts";
import { nativeSetupCaptureRequired, toolingChecksRequired } from "./native-setup-capture.mjs";
import { nativeSsrCaptureRequired } from "./native-ssr-capture.mjs";
import { nativeScopedCaptureRequired } from "./native-scoped-capture.mjs";

import { nativeVaporCaptureRequired } from "./native-vapor-capture.mjs";
import { nativeAttributeValues7502CaptureRequired } from "./native-attribute-values-7502-capture.mjs";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const globalInputs = [...cacheInputs.workspace, "pnpm-workspace.yaml"];
const matchesAny = (path, patterns) => patterns.some((pattern) => matchesGlob(path, pattern));

export { toolingTestFiles };

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

// Prose never runs as code; it is selected only by name like any other path.
export const isProse = (path) => /\.mdx?$/.test(path);
const literal = /(["'`])((?:(?!\1)[^\\\n])*)\1/g;
const segmentRun = /(?:["'`][\w.@*-]+["'`]\s*,\s*)+["'`][\w.@*-]+["'`]/g;
export function proseReferences(source) {
  const references = new Set();
  for (const [, , text] of source.matchAll(literal)) {
    const prefix = text.split("${")[0];
    // A bare wildcard names nothing in particular.
    if (!/^[*./]*$/.test(prefix) && !/\s/.test(prefix)) references.add(prefix);
  }
  // readRepoFile("docs", "content", "guide.md") and path.join segments.
  for (const [run] of source.matchAll(segmentRun))
    references.add([...run.matchAll(/["'`]([^"'`]+)["'`]/g)].map(([, part]) => part).join("/"));
  return [...references];
}
export function readsProse(path, sources) {
  return sources.some((references) =>
    references.some((reference) => {
      const directory = reference.replace(/\/+$/, "");
      // A multi-segment name may be relative to a joined root (docs/, a
      // package directory), so it also matches as a path suffix or segment.
      const nested =
        directory.includes("/") &&
        !directory.startsWith(".") &&
        (path.endsWith(`/${directory}`) || path.includes(`/${directory}/`));
      return (
        path === directory ||
        path.startsWith(`${directory}/`) ||
        nested ||
        (reference.includes("*") && matchesGlob(path, reference))
      );
    }),
  );
}

// A module that most tests share (an LSP launcher, the task catalog) may walk a
// whole top-level tree for its own bookkeeping. Such a one-segment reference in
// a widely shared helper says nothing about what any one test exercises; the
// same reference in a test or a narrow helper still selects it.
export const SHARED_HELPER_IMPORTERS = 10;
const topLevelOnly = (reference) => !reference.replace(/\/[*/]*$/, "").includes("/");

export function planToolingTests(paths, { tier = "pr", cwd = root } = {}) {
  if (!["pr", "merge"].includes(tier)) throw new Error("tier must be pr or merge");
  const files = toolingTestFiles(cwd);
  const mergeOnly = new Set(mergeOnlyToolingTests);
  const code = paths.filter((path) => !isProse(path));
  const unknown =
    paths.length === 0 ||
    code.some((path) => !matchesAny(path, [...cacheInputs.tooling, ...globalInputs]));
  const global = code.some((path) => matchesAny(path, globalInputs));
  const candidates = files.filter((file) => tier === "pr" && !mergeOnly.has(file));
  const closures = new Map(candidates.map((file) => [file, localImportInputs(file, cwd)]));
  const importers = new Map();
  for (const { files: members } of closures.values())
    for (const member of members) importers.set(member, (importers.get(member) ?? 0) + 1);
  const referenceCache = new Map();
  const references = (member) => {
    if (!referenceCache.has(member)) {
      const found = proseReferences(readFileSync(join(cwd, member), "utf8"));
      referenceCache.set(
        member,
        (importers.get(member) ?? 0) > SHARED_HELPER_IMPORTERS
          ? found.filter((reference) => !topLevelOnly(reference))
          : found,
      );
    }
    return referenceCache.get(member);
  };
  const tests = files.filter((file) => {
    if (tier === "merge") return true;
    if (mergeOnly.has(file)) return false;
    if (unknown || global || paths.includes(file)) return true;
    const imports = closures.get(file);
    if (paths.some((path) => imports.files.includes(path))) return true;
    // A declared scope is curated: its inputs decide, and an incomplete closure
    // keeps that scope's broad inputs.
    const scope = toolingTestScopes.find((entry) => matchesAny(file, entry.tests));
    const inputs = scope && (imports.complete ? cacheInputs[scope.input] : cacheInputs.tooling);
    if (inputs && paths.some((path) => matchesAny(path, inputs))) return true;
    // Otherwise a test depends on its import closure and on every repository
    // path its sources name: a file, a directory above it or a glob over it.
    const sources = imports.files.map(references);
    return paths.some((path) => readsProse(path, sources));
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
            : "import closures and named repository paths",
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
  const capturePaths = /^0+$/.test(base) ? [".github/workflows/check.yml"] : paths;
  // Matrix entry 1 also runs these captures; packing reserves their measured time there.
  const plan = {
    ...planToolingTests(paths, { tier }),
    leadSteps: toolingLeadSteps(tier, {
      "native-ssr-capture":
        nativeSsrCaptureRequired(capturePaths) || nativeScopedCaptureRequired(capturePaths),
      "native-setup-capture": nativeSetupCaptureRequired(paths),
      "native-vapor-capture": nativeVaporCaptureRequired(capturePaths),
      "native-attribute-values-7502-capture":
        nativeAttributeValues7502CaptureRequired(capturePaths),
    }),
  };
  writeToolingPlan(plan, output);
  if (githubOutput && process.env.GITHUB_OUTPUT) {
    appendFileSync(
      process.env.GITHUB_OUTPUT,
      `tooling=${toolingChecksRequired(plan, paths) || nativeSsrCaptureRequired(capturePaths) || nativeVaporCaptureRequired(capturePaths) || nativeScopedCaptureRequired(capturePaths) || nativeAttributeValues7502CaptureRequired(capturePaths)}\nnative-vapor-capture=${nativeVaporCaptureRequired(capturePaths)}\nnative-setup-capture=${nativeSetupCaptureRequired(paths)}\nnative-ssr-capture=${nativeSsrCaptureRequired(capturePaths) || nativeScopedCaptureRequired(capturePaths)}\nnative-attribute-values-7502-capture=${nativeAttributeValues7502CaptureRequired(capturePaths)}\ntooling-matrix=${JSON.stringify(toolingShardMatrix(plan))}\n`,
    );
  }
  const shards = toolingShards(plan).map(({ testMs }) => Math.round(testMs / 1000));
  const message = `Tooling tests: ${plan.tests.length}/${plan.totalTests}; ${plan.deferredTests} deferred to T1; ${plan.reason}; ${shards.length} shard(s), estimated test seconds ${shards.join("/")}.\n`;
  process.stdout.write(message);
  if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, message);
}
