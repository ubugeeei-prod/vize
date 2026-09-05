/**
 * Fresh-project release smoke (#3956).
 *
 * `smoke-release-init-typecheck.mjs` runs `vize init` inside the install tree
 * that already holds the packed packages, so Node resolves `vize` from a parent
 * `node_modules` and the generated dependency plan is never installed. This
 * module joins the two halves the issue names: it creates a project *outside*
 * that tree, runs `vize init` from the packed CLI the way `npx` would, installs
 * exactly the dependency plan the run printed, and then drives the generated
 * project-local `vize check` through the clean/broken/repaired triple using
 * only the project's own `node_modules`.
 *
 * Nothing here reads the Vize checkout after the tarballs are packed; the
 * isolation assertions in `smoke-release-init-project.mjs` are what keep that
 * true.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { run } from "./smoke-process.mjs";
import { PACKAGE_MANAGERS } from "./smoke-release-init-managers.mjs";
import { withPoisonedVizePath } from "./smoke-release-path-poison.mjs";
import { FRESH_INIT_MATRIX, PROJECT_SHAPES } from "./smoke-release-init-shapes.mjs";
import {
  assertFreshProject,
  assertMissingCorsaGuidance,
  assertProjectLocalToolchain,
  byCodeUnit,
  checkReport,
  expectedInitOutput,
  projectEnv,
  readJson,
  reportedDiagnostics,
  runGeneratedCheck,
  runManager,
  snapshotFiles,
  writeFiles,
} from "./smoke-release-init-project.mjs";

function runInit(context, projectRoot, args) {
  return run(process.execPath, [context.vizeBin, "init", projectRoot, ...args], {
    cwd: context.tempDir,
    env: projectEnv(),
  });
}

/**
 * Installs exactly the dependency plan `init` printed.
 *
 * Only the *source* of a Vize-owned package is redirected: the names, their
 * order, and the manager's own install argv are the planner's. Direct
 * dependencies always take the tarball on the command line. Package managers
 * that still resolve transitive copies of those names also get the redirect
 * table; npm does not, because it rejects an override that collides with a
 * direct spec.
 */
function installPlannedDependencies(context, projectRoot, manager, shape) {
  const manifestPath = path.join(projectRoot, "package.json");
  const manifest = readJson(manifestPath);
  const redirects = {};
  for (const [name, tarball] of context.packed) {
    if (manager.redirectPlannedDependencies || !shape.plannedDependencies.includes(name)) {
      redirects[name] = `file:${tarball}`;
    }
  }
  const redirectFiles = manager.redirect(manifest, redirects) ?? {};
  fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
  writeFiles(projectRoot, redirectFiles);

  const specs = shape.plannedDependencies.map((name) => {
    const tarball = context.packed.get(name);
    return tarball === undefined ? name : `${name}@file:${tarball}`;
  });
  runManager(manager, [...manager.installArgs, ...specs, ...manager.installFlags], {
    cwd: projectRoot,
  });
}

function runFreshProjectCell(context, cell) {
  const manager = PACKAGE_MANAGERS[cell.packageManager];
  const shape = PROJECT_SHAPES[cell.shape];
  const projectRootPath = path.join(context.freshRoot, `${shape.id}-${manager.id}`);
  const authored = { ...shape.files(context.peers), ...manager.projectFiles };
  const tracked = [...Object.keys(authored), ...shape.createdFiles, ...shape.updatedFiles];

  fs.mkdirSync(projectRootPath, { recursive: true });
  const projectRoot = fs.realpathSync.native(projectRootPath);
  writeFiles(projectRoot, authored);
  assertFreshProject(projectRoot, context, shape.initialAbsentFiles);
  runManager(manager, manager.bootstrapArgs, { cwd: projectRoot });
  assert.ok(fs.existsSync(path.join(projectRoot, manager.lockfile)), `no ${manager.lockfile}`);

  const beforeInit = snapshotFiles(projectRoot, tracked);
  assert.equal(
    runInit(context, projectRoot, [...shape.initFlags, "--dry-run"]),
    expectedInitOutput(shape, projectRoot, manager, "dry"),
  );
  assert.deepEqual(
    snapshotFiles(projectRoot, tracked),
    beforeInit,
    "--dry-run wrote to the project",
  );

  assert.equal(
    runInit(context, projectRoot, [...shape.initFlags, "--no-install"]),
    expectedInitOutput(shape, projectRoot, manager, "apply"),
  );
  for (const [name, source] of Object.entries(shape.expectedFiles)) {
    assert.equal(fs.readFileSync(path.join(projectRoot, name), "utf8"), source, `${name} mismatch`);
  }
  assert.deepEqual(readJson(path.join(projectRoot, "package.json")).scripts, shape.expectedScripts);

  installPlannedDependencies(context, projectRoot, manager, shape);
  assertProjectLocalToolchain(context, projectRoot, shape);
  assert.deepEqual(
    Object.keys(readJson(path.join(projectRoot, "package.json")).devDependencies).sort(byCodeUnit),
    shape.expectedDevDependencies,
    "the install added dependencies the plan did not name",
  );

  const clean = checkReport(projectRoot);
  assert.equal(clean.status, 0, `clean project failed vize:check\n${clean.rendered}`);
  assert.deepEqual(reportedDiagnostics(clean.report, projectRoot), []);
  assert.equal(clean.report.errorCount, 0);
  // The documented command, run exactly as `docs/content/guide/init.md` and the
  // generated script spell it, with no extra arguments. The poisoned PATH makes
  // the command fail closed if it stops resolving the fresh project's package.
  const documented = runGeneratedCheck(projectRoot, manager, [], withPoisonedVizePath(projectRoot));
  assert.equal(documented.status, 0, `documented vize:check failed\n${documented.stderr}`);

  const afterInit = snapshotFiles(projectRoot, tracked);
  assert.equal(
    runInit(context, projectRoot, [...shape.initFlags, "--no-install"]),
    expectedInitOutput(shape, projectRoot, manager, "rerun"),
  );
  assert.deepEqual(
    snapshotFiles(projectRoot, tracked),
    afterInit,
    "a second init run changed the project",
  );

  writeFiles(projectRoot, shape.check.broken);
  const broken = checkReport(projectRoot);
  assert.notEqual(broken.status, 0, "the broken project passed vize:check");
  assert.deepEqual(reportedDiagnostics(broken.report, projectRoot), shape.check.brokenDiagnostics);
  assert.equal(
    broken.report.errorCount,
    shape.check.brokenDiagnostics.reduce((total, file) => total + file.diagnostics.length, 0),
  );

  const repairs = Object.keys(shape.check.broken).map((name) => [name, authored[name]]);
  writeFiles(projectRoot, Object.fromEntries(repairs));
  const repaired = checkReport(projectRoot);
  assert.equal(repaired.status, 0, `repaired project failed vize:check\n${repaired.rendered}`);
  assert.deepEqual(reportedDiagnostics(repaired.report, projectRoot), []);

  assertMissingCorsaGuidance(projectRoot, manager);
  console.log(`runtime: fresh ${shape.id} project via ${manager.id} (init, install, check triple)`);
}

/**
 * Entry point called from the runtime smoke once the tarballs exist.
 *
 * A cell whose packed packages are not part of this run is skipped rather than
 * failed, so the narrower native-only release job stays green.
 */
export function runFreshProjectInitChecks(context) {
  const freshRootPath = path.join(context.tempDir, "fresh");
  fs.mkdirSync(freshRootPath, { recursive: true });
  const freshRoot = fs.realpathSync.native(freshRootPath);
  let ran = 0;
  for (const cell of FRESH_INIT_MATRIX) {
    const shape = PROJECT_SHAPES[cell.shape];
    if (!shape.requires.every((name) => context.packed.has(name))) continue;
    runFreshProjectCell({ ...context, freshRoot }, cell);
    ran += 1;
  }
  if (ran === 0) console.log("runtime: fresh-project init matrix skipped (packages not packed)");
}
