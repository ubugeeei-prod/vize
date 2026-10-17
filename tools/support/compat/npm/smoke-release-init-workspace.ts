/**
 * Fresh pnpm workspace cell of the release smoke (#3956).
 *
 * The matrix cells in `smoke-release-init-fresh.mjs` are standalone projects.
 * In a workspace the lockfile, the `node_modules` store and the manager's
 * overrides live in the workspace root while `vize init` runs on one package,
 * which is where package-manager detection used to fall back to npm. This cell
 * builds that layout from the packed artifacts and drives the same TypeScript
 * shape through init, install and the clean/broken/repaired check triple.
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runConfigWorkspaceChecks } from "./smoke-release-config-workspace.ts";
import { CONFIG_PACKAGE_NAMES } from "./smoke-release-config-fixture.ts";

// @ts-expect-error -- untyped release smoke helpers
import { CONFIG_FREE_PROJECT_SHAPES } from "./smoke-release-init-config-free.mjs";
// @ts-expect-error -- untyped release smoke helpers
import { packedFileSpec, packedRedirects } from "./smoke-release-init-fresh.mjs";
// @ts-expect-error -- untyped release smoke helpers
import { PACKAGE_MANAGERS } from "./smoke-release-init-managers.mjs";
import {
  assertFreshProject,
  checkReport,
  expectedInitOutput,
  projectEnv,
  readJson,
  reportedDiagnostics,
  runGeneratedCheck,
  runManager,
  writeFiles,
  // @ts-expect-error -- untyped release smoke helpers
} from "./smoke-release-init-project.mjs";
// @ts-expect-error -- untyped release smoke helpers
import { run } from "./smoke-process.mjs";

interface SmokeContext {
  readonly freshRoot: string;
  readonly packed: ReadonlyMap<string, string>;
  readonly peers: Readonly<Record<string, string>>;
  readonly tempDir: string;
  readonly versions: ReadonlyMap<string, string>;
  readonly vizeBin: string;
}

const PACKAGE_DIRECTORY = path.join("packages", "app");
const LSP_SESSION = fileURLToPath(new URL("./smoke-release-lsp-session.ts", import.meta.url));

/**
 * Runs hover, definition, completion and rename against the project-local
 * `vize lsp` of a TypeScript fresh project (see `smoke-release-lsp-session.ts`).
 */
export function runProjectLspSession(projectRoot: string): void {
  run(process.execPath, [LSP_SESSION, projectRoot], { cwd: projectRoot, env: projectEnv() });
}

/** Link both sibling manifests as one workspace graph, using only the packed plan. */
export function installPackedWorkspaceDependencies(
  workspace: string,
  plannedDependencies: readonly string[],
  packed: ReadonlyMap<string, string>,
): void {
  for (const name of CONFIG_PACKAGE_NAMES) {
    const filename = path.join(workspace, "packages", name, "package.json");
    const manifest = readJson(filename);
    for (const dependency of plannedDependencies) {
      const artifact = packed.get(dependency);
      assert.ok(artifact, `missing packed workspace dependency: ${dependency}`);
      manifest.devDependencies[dependency] = packedFileSpec(artifact);
    }
    fs.writeFileSync(filename, `${JSON.stringify(manifest, null, 2)}\n`);
  }
  const manager = PACKAGE_MANAGERS.pnpm;
  // This install intentionally applies a new dependency plan after bootstrap.
  runManager(manager, [...manager.bootstrapArgs, "--no-frozen-lockfile"], { cwd: workspace });
}

export function runWorkspacePackageCheck(context: SmokeContext): void {
  const shape = CONFIG_FREE_PROJECT_SHAPES["vite-vue-ts"];
  const manager = PACKAGE_MANAGERS.pnpm;
  assert.ok(
    shape.requires.every((name: string) => context.packed.has(name)),
    "the invoked workspace acceptance requires both packed public packages",
  );

  const workspacePath = path.join(context.freshRoot, "pnpm workspace");
  fs.mkdirSync(workspacePath, { recursive: true });
  const workspace = fs.realpathSync.native(workspacePath);
  const project = path.join(workspace, PACKAGE_DIRECTORY);
  const authored = shape.files(context.peers);
  const overrides = Object.entries(packedRedirects(context, manager, shape)).map(
    ([name, spec]) => `  ${JSON.stringify(name)}: ${JSON.stringify(spec)}`,
  );
  writeFiles(workspace, {
    "package.json": `${JSON.stringify({ name: "vize-fresh-workspace", private: true }, null, 2)}\n`,
    "pnpm-workspace.yaml": ["packages:", '  - "packages/*"', "overrides:", ...overrides, ""].join(
      "\n",
    ),
  });
  writeFiles(project, authored);
  const control = path.join(workspace, "packages", "control");
  const controlManifest = JSON.parse(authored["package.json"]);
  controlManifest.name = "vize-config-control";
  writeFiles(control, {
    ...authored,
    "package.json": `${JSON.stringify(controlManifest, null, 2)}\n`,
  });
  assertFreshProject(workspace, context, []);
  runManager(manager, manager.bootstrapArgs, { cwd: workspace });
  assert.ok(fs.existsSync(path.join(workspace, manager.lockfile)), "no workspace lockfile");
  assert.ok(!fs.existsSync(path.join(project, manager.lockfile)), "the package has a lockfile");
  const authoredManifest = JSON.parse(authored["package.json"]);
  assert.deepEqual(readJson(path.join(project, "package.json")), authoredManifest);
  assert.deepEqual(readJson(path.join(control, "package.json")), controlManifest);

  const init = (flags: readonly string[]): string =>
    run(process.execPath, [context.vizeBin, "init", project, ...shape.initFlags, ...flags], {
      cwd: context.tempDir,
      env: projectEnv(),
    });
  assert.equal(init(["--dry-run"]), expectedInitOutput(shape, project, manager, "dry"));
  assert.deepEqual(readJson(path.join(project, "package.json")), authoredManifest);
  assert.deepEqual(readJson(path.join(control, "package.json")), controlManifest);
  assert.equal(init(["--no-install"]), expectedInitOutput(shape, project, manager, "apply"));
  // --no-install writes the integrations and scripts; the printed dependency
  // plan is applied below. Every other complete authored manifest field stays.
  const configuredManifest = { ...authoredManifest, scripts: shape.expectedScripts };
  assert.deepEqual(readJson(path.join(project, "package.json")), configuredManifest);
  assert.deepEqual(readJson(path.join(control, "package.json")), controlManifest);

  // Init's complete generated plan was checked above. Redirect only its named
  // dependencies to this campaign's tarballs, then link the entire workspace:
  // an app-scoped pnpm add does not install an edited sibling manifest.
  installPackedWorkspaceDependencies(workspace, shape.plannedDependencies, context.packed);
  const plannedSpecs = Object.fromEntries(
    shape.plannedDependencies.map((name: string) => [
      name,
      packedFileSpec(context.packed.get(name)),
    ]),
  );
  for (const [root, before] of [
    [project, configuredManifest],
    [control, controlManifest],
  ] as const) {
    assert.deepEqual(readJson(path.join(root, "package.json")), {
      ...before,
      devDependencies: { ...before.devDependencies, ...plannedSpecs },
    });
  }
  assert.deepEqual(
    Object.keys(readJson(path.join(project, "package.json")).devDependencies).sort(),
    shape.expectedDevDependencies,
  );
  const vize = fs.realpathSync(path.join(project, "node_modules", "vize"));
  assert.ok(vize.startsWith(workspace), `vize resolved outside the workspace: ${vize}`);
  assert.equal(readJson(path.join(vize, "package.json")).version, context.versions.get("vize"));
  const controlVize = fs.realpathSync(path.join(control, "node_modules", "vize"));
  const relativeControlVize = path.relative(workspace, controlVize);
  assert.ok(!path.isAbsolute(relativeControlVize) && !relativeControlVize.startsWith(".."));
  assert.equal(
    readJson(path.join(controlVize, "package.json")).version,
    context.versions.get("vize"),
  );

  const clean = checkReport(project);
  assert.equal(clean.status, 0, `clean workspace package failed vize check\n${clean.rendered}`);
  assert.deepEqual(reportedDiagnostics(clean.report, project), []);
  const scripted = runGeneratedCheck(project, manager, []);
  assert.equal(scripted.status, 0, "the generated vize:check script failed in the workspace");

  writeFiles(project, shape.check.broken);
  const broken = checkReport(project);
  assert.equal(broken.status, 1, `broken workspace package did not fail\n${broken.rendered}`);
  assert.deepEqual(reportedDiagnostics(broken.report, project), shape.check.brokenDiagnostics);

  const repairs = Object.keys(shape.check.broken).map((name) => [name, authored[name]]);
  writeFiles(project, Object.fromEntries(repairs));
  const repaired = checkReport(project);
  assert.equal(repaired.status, 0, `repaired workspace package failed\n${repaired.rendered}`);
  runProjectLspSession(project);
  console.log("runtime: fresh pnpm workspace package (init, install, check triple, LSP)");
  runConfigWorkspaceChecks(workspace);
}
