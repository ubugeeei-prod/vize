/**
 * Project preparation for `typecheck-vs-vue-tsc.ts`: the registry's pinned
 * real projects plus the synthetic #9727-shaped pnpm workspace.
 */

import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

import { DEFAULT_SHAPE, generateWorkspace } from "./check-workspace.ts";

export interface BenchProject {
  id: string;
  /** Directory both tools run in. */
  cwd: string;
  /** tsconfig path relative to `cwd`. */
  tsconfig: string;
}

interface RegistryProject {
  id: string;
  repository: string;
  revision: string;
  tsconfig?: string;
  typecheckPerformance?: {
    packageManager: string;
    packageManagerVersion: string;
    baseline?: { tsconfig?: string; prepare?: string[] };
  };
}

export const SYNTHETIC_ID = "synthetic-workspace";

function run(command: string, args: string[], cwd: string, timeoutMs = 1_200_000) {
  // Pinned projects declare engines for their own CI; the check needs no runtime match.
  const env = { ...process.env, npm_config_engine_strict: "false" };
  const result = spawnSync(command, args, { cwd, env, stdio: "inherit", timeout: timeoutMs });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed in ${cwd} (${result.status})`);
  }
}

export function registryProjects(repoRoot: string): RegistryProject[] {
  const registry = JSON.parse(
    readFileSync(join(repoRoot, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
  ) as { projects: RegistryProject[] };
  return registry.projects.filter((project) => project.typecheckPerformance);
}

/** Package manager invocation pinned to the project's own version. */
function packageManager(name: string, version: string): [string, string[]] {
  return name === "npm" ? ["npm", []] : ["npx", ["--yes", `${name}@${version}`]];
}

function installArgs(name: string, root: string): string[] {
  if (name === "npm") return ["ci", "--ignore-scripts", "--no-audit", "--no-fund"];
  if (name === "yarn") return ["install", "--immutable", "--mode=skip-build"];
  const args = ["install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"];
  return existsSync(join(root, "pnpm-workspace.yaml")) ? args : [...args, "--ignore-workspace"];
}

function populated(dir: string) {
  return existsSync(dir) && readdirSync(dir).some((entry) => entry !== ".git");
}

/**
 * Clone (shallow, pinned revision), install and run the registry's baseline
 * preparation once; a `.tcbench-ready` stamp makes reruns free.
 */
export function prepareRealProject(
  repoRoot: string,
  id: string,
  fixturesDir: string,
): BenchProject {
  const project = registryProjects(repoRoot).find((candidate) => candidate.id === id);
  const performance = project?.typecheckPerformance;
  if (!project || !performance) throw new Error(`no typecheckPerformance project ${id}`);
  const submodule = join(repoRoot, "tests/_fixtures/_git", id);
  const root = populated(submodule) ? submodule : resolve(fixturesDir, id);
  const stamp = join(root, ".tcbench-ready");
  const tsconfig = performance.baseline?.tsconfig ?? project.tsconfig ?? "tsconfig.json";
  if (existsSync(stamp) && readFileSync(stamp, "utf8").trim() === project.revision) {
    return { id, cwd: root, tsconfig };
  }
  if (!populated(root)) {
    mkdirSync(root, { recursive: true });
    run("git", ["init", "-q"], root);
    run("git", ["remote", "add", "origin", project.repository], root);
    run("git", ["fetch", "-q", "--depth", "1", "origin", project.revision], root);
    run("git", ["checkout", "-q", "FETCH_HEAD"], root);
  }
  const [manager, prefix] = packageManager(
    performance.packageManager,
    performance.packageManagerVersion,
  );
  run(manager, [...prefix, ...installArgs(performance.packageManager, root)], root);
  const prepare = performance.baseline?.prepare;
  if (prepare) run(manager, [...prefix, ...prepare.slice(1)], root);
  writeFileSync(stamp, `${project.revision}\n`);
  return { id, cwd: root, tsconfig };
}

/**
 * Three planted findings (script, template, child prop) so a run that
 * silently skips checking cannot look fast.
 */
const PLANTED = `<script setup lang="ts">
import Child from "./components/Component1.vue";
const count: number = "planted";
</script>

<template>
  <div>{{ count.planted }}</div>
  <Child :title="42" />
</template>
`;

export function prepareSyntheticWorkspace(
  fixturesDir: string,
  links: { vue?: string; typescript?: string },
  components = DEFAULT_SHAPE.appComponents,
): BenchProject {
  if (!Number.isSafeInteger(components) || components < 2)
    throw new Error("synthetic component count must be an integer of at least two");
  const root = resolve(fixturesDir, SYNTHETIC_ID);
  const app = generateWorkspace(root, { ...DEFAULT_SHAPE, appComponents: components }, links);
  writeFileSync(join(app, "src", "Planted.vue"), PLANTED);
  return { id: SYNTHETIC_ID, cwd: app, tsconfig: "tsconfig.json" };
}
