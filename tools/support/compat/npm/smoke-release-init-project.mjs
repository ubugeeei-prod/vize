/**
 * Project-level assertions for the fresh-project release smoke (#3956).
 *
 * Split from the driver so the flow of a matrix cell -- author, install, init,
 * install the plan, check -- reads as one page, and the contracts each step has
 * to satisfy live next to the reason they exist.
 */

import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";

import { normalizeReportedFile } from "./smoke-release-init-paths.mjs";
import { satisfiesVersionRange } from "./smoke-release-semver.mjs";
import { renderOutput, run, runResult } from "./smoke-process.mjs";

/** Overrides that would let the host, not the installed package, pick Corsa. */
const CORSA_ENV_VARS = ["CORSA_PATH", "CORSA_EXECUTABLE", "TSGO_PATH", "TSGO_EXECUTABLE"];

/** SGR escape sequences, built without embedding a control character in source. */
const ANSI_SGR = new RegExp(`${String.fromCharCode(27)}\\[[0-9;]*m`, "gu");

const EDITOR_REPORT = [
  "[vize init] editor integrations shipped with Vize:",
  "  VS Code: ubugeeei.vize (recommended in .vscode/extensions.json)",
  "  Zed: editors/zed",
  "  Neovim: editors/nvim",
  "  Vim: editors/vim",
  "  Helix: editors/helix",
  "  Emacs: editors/emacs",
];

export function managerBinary(manager) {
  return process.env[manager.binaryEnv] || manager.binary;
}

export function managerCommand(manager, args) {
  const envBinary = process.env[manager.binaryEnv];
  if (envBinary !== undefined) {
    return { command: envBinary, args };
  }
  if (manager.corepackSpec !== undefined) {
    return { command: "corepack", args: [manager.corepackSpec, ...args] };
  }
  return { command: manager.binary, args };
}

/** Code-unit order, so the comparison is locale-independent across runners. */
export function byCodeUnit(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

/**
 * Environment for every command run inside the fresh project.
 *
 * The Corsa overrides are stripped so the run proves the *installed* CLI's own
 * discovery of `typescript@7`; a host that exported `CORSA_PATH`
 * would otherwise hide a packaging failure.
 */
export function projectEnv(extra = {}) {
  const env = { ...process.env, ...extra };
  for (const name of CORSA_ENV_VARS) {
    if (!(name in extra)) delete env[name];
  }
  return env;
}

export function managerEnv(manager, extra = {}) {
  const env = projectEnv({ ...manager.environment, ...extra });
  if (manager.corepackSpec !== undefined && !("COREPACK_ENABLE_PROJECT_SPEC" in extra)) {
    env.COREPACK_ENABLE_PROJECT_SPEC = "0";
  }
  return env;
}

export function runManager(manager, args, options = {}) {
  const runner = managerCommand(manager, args);
  return run(runner.command, runner.args, {
    ...options,
    env: managerEnv(manager, options.env),
  });
}

function isOutside(base, target) {
  const relative = path.relative(base, target);
  return relative !== "" && (path.isAbsolute(relative) || relative.startsWith(".."));
}

export function writeFiles(root, files) {
  for (const [relative, source] of Object.entries(files)) {
    const filename = path.join(root, relative);
    fs.mkdirSync(path.dirname(filename), { recursive: true });
    fs.writeFileSync(filename, source);
  }
}

export function snapshotFiles(root, names) {
  return Object.fromEntries(
    names.map((name) => {
      const filename = path.join(root, name);
      return [name, fs.existsSync(filename) ? fs.readFileSync(filename, "utf8") : null];
    }),
  );
}

export function readJson(filename) {
  return JSON.parse(fs.readFileSync(filename, "utf8"));
}

function resolveShapeField(shape, name, ...args) {
  const value = shape[name];
  return typeof value === "function" ? value(...args) : value;
}

/**
 * Proves the project is genuinely fresh before `init` touches it.
 *
 * A `node_modules` or `package.json` in any ancestor would let Node resolve
 * `vize` from outside the project, which is exactly the workspace link the
 * issue's first acceptance criterion rules out.
 */
export function assertFreshProject(projectRoot, context, initialAbsentFiles = []) {
  assert.ok(
    isOutside(context.installDir, projectRoot),
    `${projectRoot} is inside the install tree`,
  );
  assert.ok(isOutside(context.repoRoot, projectRoot), `${projectRoot} is inside the Vize checkout`);
  for (
    let directory = path.dirname(projectRoot);
    directory.startsWith(context.tempDir);
    directory = path.dirname(directory)
  ) {
    for (const leaked of ["node_modules", "package.json"]) {
      assert.equal(
        fs.existsSync(path.join(directory, leaked)),
        false,
        `${path.join(directory, leaked)} would leak into the fresh project`,
      );
    }
    if (directory === context.tempDir) break;
  }
  for (const name of initialAbsentFiles) {
    assert.equal(fs.existsSync(path.join(projectRoot, name)), false, `${name} exists before init`);
  }
  assert.equal(fs.existsSync(path.join(projectRoot, "node_modules")), false);
}

function planLines(verb, { features, created, updated, scripts, command }) {
  const lines = ["[vize init] plan:", ...features];
  for (const filename of created) lines.push(`[vize init] ${verb} create ${filename}`);
  for (const filename of updated) lines.push(`[vize init] ${verb} update ${filename}`);
  if (scripts.length > 0) lines.push(`[vize init] ${verb} add scripts: ${scripts.join(", ")}`);
  if (command !== null) lines.push(`[vize init] ${verb} run: ${command}`);
  if (created.length + updated.length + (command === null ? 0 : 1) === 0) {
    lines.push("[vize init] nothing to do; the project is already configured");
  }
  return lines;
}

/**
 * The complete stdout `vize init` must produce, for each of the three runs a
 * cell performs: the dry run, the run that applies the plan, and the re-run that
 * proves idempotency.
 */
export function expectedInitOutput(shape, projectRoot, manager, mode) {
  const dependencies = shape.plannedDependencies.join(" ");
  const installCommand = `${manager.id} ${manager.installArgs.join(" ")} ${dependencies}`;
  const applied = {
    detection: resolveShapeField(shape, "detection", manager),
    features: shape.features,
    created: shape.createdFiles,
    updated: shape.updatedFiles,
    scripts: shape.addedScripts,
  };
  const plans = {
    dry: { ...applied, verb: "would", command: installCommand, editors: false },
    apply: { ...applied, verb: "will", command: null, editors: true },
    rerun: {
      verb: "will",
      detection: resolveShapeField(shape, "reconfiguredDetection", manager),
      features: shape.reconfiguredFeatures,
      created: [],
      updated: [],
      scripts: [],
      command: null,
      editors: true,
    },
  };
  const plan = plans[mode];
  return [
    `[vize init] detected in ${projectRoot}:`,
    ...plan.detection,
    ...planLines(plan.verb, plan),
    ...(plan.editors ? EDITOR_REPORT : []),
    "",
  ].join("\n");
}

/**
 * Proves binary discovery resolved inside the fresh project.
 *
 * The installed tree must resolve inside the fresh project, must carry the
 * packed version, and must have brought the Corsa runtime the CLI declares as
 * an optional dependency -- the release-only failure an `--omit=optional`
 * install or a trimmed tarball would cause.
 */
export function assertProjectLocalToolchain(context, projectRoot, shape) {
  const nodeModules = path.join(projectRoot, "node_modules");
  const realProjectRoot = fs.realpathSync(projectRoot);
  const installedRoots = new Map();
  for (const name of shape.plannedDependencies) {
    const installed = path.join(nodeModules, ...name.split("/"));
    assert.ok(fs.existsSync(installed), `${name} is missing from the fresh project`);
    const resolved = fs.realpathSync(installed);
    assert.ok(isOutside(context.repoRoot, resolved), `${name} resolved into the Vize checkout`);
    assert.ok(isOutside(context.installDir, resolved), `${name} resolved into the install tree`);
    assert.ok(
      !isOutside(realProjectRoot, resolved),
      `${name} resolved outside the fresh project: ${resolved}`,
    );
    assert.equal(readJson(path.join(resolved, "package.json")).version, context.versions.get(name));
    installedRoots.set(name, resolved);
  }
  const vizeRoot = installedRoots.get("vize");
  assert.equal(typeof vizeRoot, "string", "fresh project did not install vize");
  const vizeBin = projectLocalVizeBin(projectRoot);
  assert.ok(fs.existsSync(vizeBin), "fresh project did not install project-local vize bin");
  assert.ok(
    !isOutside(realProjectRoot, fs.realpathSync(vizeBin)),
    `vize bin resolved outside the fresh project: ${vizeBin}`,
  );
  const vizeRequire = createRequire(path.join(vizeRoot, "package.json"));
  const corsaPackage = `@typescript/typescript-${process.platform}-${process.arch}`;
  const vizePackageJson = readJson(path.join(vizeRoot, "package.json"));
  const declaredCorsaRange = vizePackageJson.optionalDependencies?.[corsaPackage];
  assert.equal(
    typeof declaredCorsaRange,
    "string",
    `installed vize does not declare optional dependency ${corsaPackage}`,
  );
  const corsaManifest = vizeRequire.resolve(`${corsaPackage}/package.json`);
  const corsaPackageJson = readJson(corsaManifest);
  const corsaVersion = corsaPackageJson.version;
  assert.equal(corsaPackageJson.name, corsaPackage);
  assert.ok(
    satisfiesVersionRange(corsaVersion, declaredCorsaRange),
    `${corsaPackage}@${corsaVersion} does not satisfy installed vize optional dependency ${declaredCorsaRange}`,
  );
  assert.ok(
    typeof corsaVersion === "string" && Number.parseInt(corsaVersion.split(".")[0] ?? "", 10) >= 7,
    "installed vize did not bring TypeScript 7 for Corsa",
  );
  assert.ok(
    !isOutside(realProjectRoot, fs.realpathSync(corsaManifest)),
    "installed vize did not bring project-local TypeScript 7 for Corsa",
  );
}

/** Runs the `vize:check` script `init` generated, through the project's manager. */
export function runGeneratedCheck(projectRoot, manager, extra, env = {}) {
  const runner = managerCommand(manager, manager.runScriptArgs("vize:check", extra));
  return runResult(runner.command, runner.args, {
    cwd: projectRoot,
    env: managerEnv(manager, env),
  });
}

export function projectLocalVizeBin(projectRoot) {
  return path.join(projectRoot, "node_modules", "vize", "bin", "vize");
}

export function runProjectLocalCheck(projectRoot, extra, env = {}) {
  return runResult(process.execPath, [projectLocalVizeBin(projectRoot), "check", ...extra], {
    cwd: projectRoot,
    env: projectEnv(env),
  });
}

function renderInvocation(command, args, cwd, result) {
  const rendered = renderOutput(result);
  return [
    `command: ${JSON.stringify([command, ...args])}`,
    `cwd: ${cwd}`,
    `status: ${result.status ?? "<null>"}`,
    `signal: ${result.signal ?? "<none>"}`,
    rendered === "" ? "stdout/stderr: <empty>" : rendered,
  ].join("\n");
}

export function checkReport(projectRoot) {
  const args = [projectLocalVizeBin(projectRoot), "check", "--format", "json", "--quiet"];
  const result = runResult(process.execPath, args, {
    cwd: projectRoot,
    env: projectEnv(),
  });
  const rendered = renderInvocation(process.execPath, args, projectRoot, result);
  let report;
  try {
    report = JSON.parse(result.stdout);
  } catch (error) {
    throw new Error(`project-local vize check did not produce JSON\n${rendered}`, {
      cause: error,
    });
  }
  return { rendered, report, status: result.status };
}

export function reportedDiagnostics(report, projectRoot) {
  return report.files
    .filter((file) => file.diagnostics.length > 0)
    .map((file) => ({
      file: normalizeReportedFile(file.file, projectRoot),
      diagnostics: file.diagnostics,
    }));
}

/**
 * The first-run failure contract: a missing Corsa runtime must name the package
 * and the exact command that installs it, and must fail rather than report a
 * clean project.
 *
 * SGR sequences are stripped and path separators normalised before comparing, so
 * the assertion stays a full-equality one on the message a user reads: the CLI
 * colours this path unconditionally today, and neither honouring `NO_COLOR`
 * later nor Windows' separator may red-light the release smoke.
 */
export function assertMissingCorsaGuidance(projectRoot, manager) {
  const missing = path.join(projectRoot, "no-such-corsa-runtime");
  const direct = runProjectLocalCheck(projectRoot, ["--quiet"], { CORSA_PATH: missing });
  assert.notEqual(direct.status, 0, "a missing Corsa runtime silently disabled type checking");
  const message = renderOutput(direct).replaceAll(ANSI_SGR, "").replaceAll("\\", "/");
  assert.equal(
    message,
    [
      "Error: error: corsa not found",
      "",
      `Configured Corsa executable does not exist: ${missing.replaceAll("\\", "/")}`,
      "",
      "To install, run:",
      "",
      `  ${manager.corsaInstallCommand} ${manager.installArgs.join(" ")} typescript@^7`,
    ].join("\n"),
  );
  const scripted = runGeneratedCheck(projectRoot, manager, [], { CORSA_PATH: missing });
  assert.notEqual(scripted.status, 0, "the generated script hid the missing Corsa runtime");
}
