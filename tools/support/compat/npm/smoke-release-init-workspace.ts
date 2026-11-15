/**
 * Fresh pnpm and Bun workspace cells of the release smoke (#3956, #13012).
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

import { CONFIG_FREE_PROJECT_SHAPES } from "./smoke-release-init-config-free.ts";
import { packedFileSpec, packedRedirects } from "./smoke-release-init-fresh.mjs";
import { PACKAGE_MANAGERS } from "./smoke-release-init-managers.ts";
import {
  assertFreshProject,
  checkReport,
  expectedInitOutput,
  managerBinary,
  projectEnv,
  readJson,
  reportedDiagnostics,
  runGeneratedCheck,
  runManager,
  writeFiles,
} from "./smoke-release-init-project.ts";
import { run } from "./smoke-process.ts";
import {
  packedDownloadBytes,
  readLspDiagnosticSettle,
  recordPackedSmokeTiming,
} from "./smoke-release-init-timing.ts";
import {
  type WorkspacePackageManager,
  workspaceRootFiles,
} from "./smoke-release-init-workspace-roots.ts";

interface SmokeContext {
  readonly freshRoot: string;
  readonly installDir: string;
  readonly repoRoot: string;
  readonly packed: ReadonlyMap<string, string>;
  readonly peers: Readonly<Record<string, string>>;
  readonly tempDir: string;
  readonly versions: ReadonlyMap<string, string>;
  readonly vizeBin: string;
}

const PACKAGE_DIRECTORY = path.join("packages", "app");
const LSP_SESSION = fileURLToPath(new URL("./smoke-release-lsp-session.ts", import.meta.url));

function installedWorkspacePackageRoot(
  project: string,
  workspace: string,
  packageName: string,
): string {
  for (let directory = project; ; directory = path.dirname(directory)) {
    const candidate = path.join(directory, "node_modules", ...packageName.split("/"));
    if (fs.existsSync(candidate)) return fs.realpathSync.native(candidate);
    if (directory === workspace) break;
  }
  throw new Error(`${packageName} is not installed in ${workspace}`);
}

/**
 * Runs the shape's authored feature checks against the project-local `vize lsp`
 * of a TypeScript fresh project (see `smoke-release-lsp-session.ts`).
 */
export function runProjectLspSession(
  projectRoot: string,
  projectShape: "vite" | "nuxt" = "vite",
  runtime = process.execPath,
  vizeBin = path.join(projectRoot, "node_modules/vize/bin/vize"),
): void {
  run(process.execPath, [LSP_SESSION, projectRoot, projectShape], {
    cwd: projectRoot,
    env: projectEnv({ VIZE_SMOKE_LSP_RUNTIME: runtime, VIZE_SMOKE_LSP_BIN: vizeBin }),
  });
}

export function runWorkspacePackageCheck(
  context: SmokeContext,
  managerId: WorkspacePackageManager = "pnpm",
): void {
  const shape = CONFIG_FREE_PROJECT_SHAPES["vite-vue-ts"];
  const manager = PACKAGE_MANAGERS[managerId];
  if (!shape.requires.every((name: string) => context.packed.has(name))) return;

  const workspacePath = path.join(context.freshRoot, `${manager.id} workspace`);
  fs.mkdirSync(workspacePath, { recursive: true });
  const workspace = fs.realpathSync.native(workspacePath);
  const project = path.join(workspace, PACKAGE_DIRECTORY);
  const authored = shape.files(context.peers);
  const redirects = packedRedirects(context, manager, shape);
  writeFiles(workspace, workspaceRootFiles(managerId, redirects));
  writeFiles(project, { ...authored, ...manager.projectFiles });
  assertFreshProject(workspace, context, []);
  runManager(manager, manager.bootstrapArgs, { cwd: workspace });
  assert.ok(fs.existsSync(path.join(workspace, manager.lockfile)), "no workspace lockfile");
  assert.ok(!fs.existsSync(path.join(project, manager.lockfile)), "the package has a lockfile");

  const init = (flags: readonly string[]): string =>
    run(managerId === "bun" ? managerBinary(manager) : process.execPath, [
      context.vizeBin,
      "init",
      project,
      ...shape.initFlags,
      ...flags,
    ], {
      cwd: context.tempDir,
      env: projectEnv(),
    });
  assert.equal(init(["--dry-run"]), expectedInitOutput(shape, project, manager, "dry"));
  assert.equal(init(["--no-install"]), expectedInitOutput(shape, project, manager, "apply"));

  const specs = shape.plannedDependencies.map(
    (name: string) => `${name}@${packedFileSpec(context.packed.get(name))}`,
  );
  const installStarted = performance.now();
  runManager(manager, [...manager.installArgs, ...specs, ...manager.installFlags], {
    cwd: project,
  });
  const installMs = performance.now() - installStarted;
  const vize = installedWorkspacePackageRoot(project, workspace, "vize");
  const vizeBin = path.join(vize, "bin", "vize");
  const runtime = managerId === "bun" ? managerBinary(manager) : process.execPath;
  assert.ok(vize.startsWith(workspace), `vize resolved outside the workspace: ${vize}`);
  assert.equal(readJson(path.join(vize, "package.json")).version, context.versions.get("vize"));

  const checkStarted = performance.now();
  const clean = checkReport(project, runtime, vizeBin);
  const firstCheckMs = performance.now() - checkStarted;
  assert.equal(clean.status, 0, `clean workspace package failed vize check\n${clean.rendered}`);
  assert.deepEqual(reportedDiagnostics(clean.report, project), []);
  const scripted = runGeneratedCheck(project, manager, []);
  assert.equal(scripted.status, 0, "the generated vize:check script failed in the workspace");

  writeFiles(project, shape.check.broken);
  const broken = checkReport(project, runtime, vizeBin);
  assert.equal(broken.status, 1, `broken workspace package did not fail\n${broken.rendered}`);
  assert.deepEqual(reportedDiagnostics(broken.report, project), shape.check.brokenDiagnostics);

  const repairs = Object.keys(shape.check.broken).map((name) => [name, authored[name]]);
  writeFiles(project, Object.fromEntries(repairs));
  const repaired = checkReport(project, runtime, vizeBin);
  assert.equal(repaired.status, 0, `repaired workspace package failed\n${repaired.rendered}`);
  runProjectLspSession(project, "vite", runtime, vizeBin);
  const lsp = readLspDiagnosticSettle(project);
  recordPackedSmokeTiming({
    projectShape: shape.id,
    packageManager: manager.id,
    host: `${process.platform}-${process.arch}`,
    installMs,
    firstCheckMs,
    ...packedDownloadBytes(context.packed, shape.plannedDependencies),
    lspSession: true,
    lspDiagnosticsNotifications: lsp.notifications,
    lspDiagnosticsSettled: lsp.settled,
    lspDiagnosticsSettleMs: lsp.settleMs,
    diagnosticQuietMs: lsp.diagnosticQuietMs,
  });
  console.log(
    `runtime: fresh ${manager.id} workspace package (init, install, check triple, LSP)`,
  );
}
