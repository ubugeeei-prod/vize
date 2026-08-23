import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  existsSync,
  globSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { expectedCompilerOutputs } from "./tool-matrix-compiler-paths.mjs";
import { displayCommand, toolArgs, typecheckCorpusGlobs } from "./tool-matrix-command.mjs";
import { snapshotFormatterInputs, validateFormatterOutput } from "./tool-matrix-formatter.mjs";
import { collectTypecheckerAuthoredPaths, collectVueInputPaths } from "./tool-matrix-inputs.mjs";
import { validateLinterOutput } from "./tool-matrix-linter.mjs";
import { validatedFileCount } from "./tool-matrix-metrics.mjs";
import {
  summarizeTypecheckerCoverage,
  validateTypecheckerOutput,
} from "./tool-matrix-typechecker.mjs";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
export function runTool(project, tool, args, launch, outputDir) {
  const prepared = prepareToolRun(project, tool, args, launch, outputDir);
  if (prepared.run != null) return prepared.run;
  try {
    const startedAt = Date.now();
    const result = spawnSync(launch.command, prepared.commandArgs, {
      cwd: prepared.cwd,
      encoding: "utf8",
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
      maxBuffer: 1024 * 1024 * 1024,
      timeout: args.timeoutMs,
    });
    return completeToolRun({ ...prepared, result, startedAt });
  } finally {
    cleanupToolRun(prepared);
  }
}

export async function runToolWithHeartbeat(project, tool, args, launch, outputDir, progress = {}) {
  const prepared = prepareToolRun(project, tool, args, launch, outputDir);
  if (prepared.run != null) return prepared.run;
  try {
    const startedAt = Date.now();
    const result = await spawnWithHeartbeat(launch.command, prepared.commandArgs, {
      cwd: prepared.cwd,
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
      heartbeatMs: args.heartbeatMs ?? 30_000,
      maxBuffer: 1024 * 1024 * 1024,
      projectId: project.id,
      timeoutMs: args.timeoutMs,
      tool,
      write: progress.write ?? process.stderr.write.bind(process.stderr),
    });
    return completeToolRun({ ...prepared, result, startedAt });
  } finally {
    cleanupToolRun(prepared);
  }
}

function prepareToolRun(project, tool, args, launch, outputDir) {
  const cwd = resolve(repoRoot, project.fixturePath);
  const fixtureExists = existsSync(cwd);
  const compilerOutputDir =
    tool === "compiler" && !args.dryRun && fixtureExists
      ? mkdtempSync(join(tmpdir(), "vize-fixture-compiler-"))
      : null;
  const commandArgs = [
    ...launch.prefix,
    ...toolArgs(project, tool, compilerOutputDir ?? "<compiler-output>"),
  ];
  const base = {
    tool,
    command: displayCommand(launch.command, commandArgs),
    cwd: relative(repoRoot, cwd),
    durationMs: 0,
    fileCount: null,
    exitCode: null,
    outputPath: null,
    coverage: null,
  };
  if (args.dryRun) return { run: { ...base, status: "planned" } };
  if (!fixtureExists) return { run: { ...base, status: "missing-fixture" } };
  const formatterStateBefore =
    tool === "formatter" ? snapshotFormatterInputs(cwd, project.vueGlobs) : null;
  const expectedToolFiles =
    tool === "typechecker" || tool === "linter" || tool === "formatter"
      ? collectVueInputPaths(cwd, tool === "typechecker" ? typecheckCorpusGlobs(project) : project.vueGlobs)
      : null;
  return {
    base,
    commandArgs,
    compilerOutputDir,
    cwd,
    expectedToolFiles,
    formatterStateBefore,
    outputDir,
    project,
    tool,
  };
}

function cleanupToolRun(prepared) {
  if (prepared.compilerOutputDir != null) {
    rmSync(prepared.compilerOutputDir, { recursive: true, force: true });
  }
}

function completeToolRun({
  base,
  compilerOutputDir,
  cwd,
  expectedToolFiles,
  formatterStateBefore,
  outputDir,
  project,
  result,
  startedAt,
  tool,
}) {
  const stdout = result.stdout ?? "";
  const stderr = result.stderr ?? "";
  const rawPath = join(outputDir, `${project.id}-${tool}.json`);
  const completed = {
    ...base,
    durationMs: Date.now() - startedAt,
    exitCode: result.status,
    outputPath: relative(repoRoot, rawPath),
  };
  const payload = {
    schema: "vize.fixtureToolRun",
    version: 1,
    project: project.id,
    tool,
    exitCode: result.status,
    stdout,
    stderr,
  };
  if (result.signal != null) payload.signal = result.signal;
  if (result.error != null) {
    payload.spawnError = errorMessage(result.error);
  } else if (tool === "compiler" && (result.status === 0 || result.status === 1)) {
    try {
      payload.compilerArtifacts = inspectCompilerArtifacts(
        cwd,
        project.vueGlobs,
        project.expectedVueFileCount,
        compilerOutputDir,
      );
    } catch (error) {
      payload.validationError = errorMessage(error);
    }
  } else if (tool !== "formatter" && (result.status === 0 || result.status === 1)) {
    try {
      payload.parsed = JSON.parse(stdout);
    } catch (error) {
      payload.parseError = errorMessage(error);
    }
    if (tool === "typechecker" && payload.parseError == null) {
      try {
        payload.typecheckerCoverage = validateTypecheckerOutput(
          project,
          payload.parsed,
          result.status,
          expectedToolFiles,
          collectTypecheckerAuthoredPaths(cwd),
        );
      } catch (error) {
        payload.validationError = errorMessage(error);
      }
    }
    if (tool === "linter" && payload.parseError == null) {
      try {
        validateLinterOutput(project, payload.parsed, result.status, expectedToolFiles);
      } catch (error) {
        payload.validationError = errorMessage(error);
      }
    }
  } else if (tool === "formatter" && (result.status === 0 || result.status === 1)) {
    try {
      const formatterStateAfter = snapshotFormatterInputs(cwd, project.vueGlobs);
      payload.formatterCheck = validateFormatterOutput(
        project,
        payload.stdout,
        payload.stderr,
        result.status,
        formatterStateBefore,
        formatterStateAfter,
        expectedToolFiles,
      );
    } catch (error) {
      payload.validationError = errorMessage(error);
    }
  }
  writeFileSync(rawPath, `${JSON.stringify(payload, null, 2)}\n`);

  if (result.error != null) {
    return { ...completed, status: "failed", failure: errorMessage(result.error) };
  }
  if (result.status !== 0 && result.status !== 1) {
    return { ...completed, status: "failed", failure: failureOutput(result) };
  }
  if (payload.validationError != null) {
    return { ...completed, status: "failed", failure: payload.validationError };
  }
  if (payload.parseError != null) {
    return {
      ...completed,
      status: "failed",
      failure: `invalid JSON output: ${payload.parseError}`,
    };
  }
  return {
    ...completed,
    fileCount: validatedFileCount(tool, payload),
    coverage:
      tool === "typechecker" ? summarizeTypecheckerCoverage(payload.typecheckerCoverage) : null,
    status: result.status === 0 ? "ok" : "findings",
  };
}

function spawnWithHeartbeat(command, commandArgs, options) {
  const startedAt = Date.now();
  const heartbeatMs = Math.max(1, options.heartbeatMs);
  writeProgress(options.write, "start", {
    projectId: options.projectId,
    tool: options.tool,
    timeoutMs: options.timeoutMs,
  });
  return new Promise((settle) => {
    let stdout = "";
    let stderr = "";
    let stdoutBytes = 0;
    let stderrBytes = 0;
    let spawnError = null;
    let timeoutError = null;
    let maxBufferError = null;
    let forceKillTimer = null;
    let forceSettleTimer = null;
    let terminating = false;
    let settled = false;
    const child = spawn(command, commandArgs, {
      cwd: options.cwd,
      detached: process.platform !== "win32",
      env: options.env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const stopTimers = () => {
      clearInterval(heartbeat);
      clearTimeout(timeout);
      clearTimeout(forceKillTimer);
      clearTimeout(forceSettleTimer);
    };
    const settleResult = (status, signal) => {
      if (settled) return;
      settled = true;
      stopTimers();
      const normalizedStatus = spawnError == null ? status : null;
      writeProgress(options.write, "finish", {
        projectId: options.projectId,
        tool: options.tool,
        elapsedMs: Date.now() - startedAt,
        status: normalizedStatus,
        signal,
      });
      settle({
        error: spawnError ?? timeoutError ?? maxBufferError,
        signal,
        status: normalizedStatus,
        stderr,
        stdout,
      });
    };
    const signalChildTree = (signal) => {
      if (child.pid == null) {
        child.kill(signal);
        return;
      }
      if (process.platform !== "win32") {
        try {
          process.kill(-child.pid, signal);
          return;
        } catch (error) {
          if (error?.code === "ESRCH") return;
        }
      }
      child.kill(signal);
    };
    const beginTermination = () => {
      if (terminating) return;
      terminating = true;
      signalChildTree("SIGTERM");
      forceKillTimer = setTimeout(() => signalChildTree("SIGKILL"), 5_000);
      forceKillTimer.unref?.();
      forceSettleTimer = setTimeout(() => settleResult(null, "SIGKILL"), 10_000);
      forceSettleTimer.unref?.();
    };
    const killForMaxBuffer = (streamName) => {
      if (maxBufferError == null) {
        maxBufferError = new Error(`${streamName} maxBuffer exceeded`);
        maxBufferError.code = "ENOBUFS";
        beginTermination();
      }
    };
    child.stdout?.setEncoding("utf8");
    child.stderr?.setEncoding("utf8");
    child.stdout?.on("data", (chunk) => {
      stdoutBytes += Buffer.byteLength(chunk);
      if (stdoutBytes > options.maxBuffer) killForMaxBuffer("stdout");
      else stdout += chunk;
    });
    child.stderr?.on("data", (chunk) => {
      stderrBytes += Buffer.byteLength(chunk);
      if (stderrBytes > options.maxBuffer) killForMaxBuffer("stderr");
      else stderr += chunk;
    });
    child.once("error", (error) => {
      spawnError = error;
    });
    child.once("close", (status, signal) => {
      settleResult(status, signal);
    });
    const heartbeat = setInterval(() => {
      writeProgress(options.write, "still-running", {
        projectId: options.projectId,
        tool: options.tool,
        elapsedMs: Date.now() - startedAt,
      });
    }, heartbeatMs);
    heartbeat.unref?.();
    const timeout = setTimeout(() => {
      timeoutError = new Error(`spawn timed out after ${options.timeoutMs}ms`);
      timeoutError.code = "ETIMEDOUT";
      writeProgress(options.write, "timeout", {
        projectId: options.projectId,
        tool: options.tool,
        elapsedMs: Date.now() - startedAt,
        timeoutMs: options.timeoutMs,
      });
      beginTermination();
    }, options.timeoutMs);
    timeout.unref?.();
  });
}

function writeProgress(write, event, fields) {
  const parts = [`[tool-matrix] ${event}`];
  for (const [key, value] of Object.entries(fields)) {
    if (value == null) continue;
    parts.push(`${key}=${value}`);
  }
  try {
    write(`${parts.join(" ")}\n`);
  } catch {
    // Progress logging must never change the tool result.
  }
}

function inspectCompilerArtifacts(cwd, patterns, expectedFileCount, outputDir) {
  const inputPaths = [
    ...new Set(
      patterns.flatMap((pattern) =>
        globSync(pattern, { cwd, exclude: [".yarn/**", "**/node_modules/**"] })
          .filter((entry) => statSync(resolve(cwd, entry)).isFile())
          .map((entry) => entry.replaceAll("\\", "/")),
      ),
    ),
  ].sort((left, right) => left.localeCompare(right));
  if (expectedFileCount != null && inputPaths.length !== expectedFileCount) {
    throw new Error(
      `compiler input count mismatch: expected ${expectedFileCount}, matched ${inputPaths.length}`,
    );
  }
  if (inputPaths.length === 0 && expectedFileCount !== 0) {
    throw new Error("compiler matched no Vue files");
  }

  const outputPaths = collectFiles(outputDir);
  const nonJsonPaths = outputPaths.filter((entry) => !entry.endsWith(".json"));
  if (nonJsonPaths.length > 0) {
    throw new Error(`compiler emitted non-JSON artifacts: ${nonJsonPaths.join(", ")}`);
  }
  if (outputPaths.length !== inputPaths.length) {
    throw new Error(
      `compiler artifact count mismatch: ${inputPaths.length} inputs, ${outputPaths.length} outputs`,
    );
  }
  const inputByOutputPath = expectedCompilerOutputs(cwd, patterns, inputPaths);
  const missingPaths = [...inputByOutputPath.keys()].filter(
    (entry) => !outputPaths.includes(entry),
  );
  const unexpectedPaths = outputPaths.filter((entry) => !inputByOutputPath.has(entry));
  if (missingPaths.length > 0 || unexpectedPaths.length > 0) {
    throw new Error(
      `compiler artifact path mismatch: missing [${missingPaths.join(", ")}], unexpected [${unexpectedPaths.join(", ")}]`,
    );
  }

  const digest = createHash("sha256");
  let errorCount = 0;
  let warningCount = 0;
  const findings = [];
  for (const outputPath of outputPaths) {
    const source = readFileSync(join(outputDir, outputPath), "utf8");
    digest.update(outputPath);
    digest.update("\0");
    digest.update(source);
    digest.update("\0");
    let artifact;
    try {
      artifact = JSON.parse(source);
    } catch (error) {
      throw new Error(`invalid compiler JSON artifact ${outputPath}: ${errorMessage(error)}`);
    }
    const inputPath = inputByOutputPath.get(outputPath);
    validateCompilerArtifact(outputPath, artifact, inputPath);
    errorCount += artifact.errors.length;
    warningCount += artifact.warnings.length;
    if (artifact.errors.length > 0 || artifact.warnings.length > 0) {
      findings.push({ file: inputPath, errors: artifact.errors, warnings: artifact.warnings });
    }
  }

  return {
    inputFileCount: inputPaths.length,
    outputFileCount: outputPaths.length,
    errorCount,
    warningCount,
    findings,
    sha256: digest.digest("hex"),
  };
}

function collectFiles(root, directory = root, files = []) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const absolute = join(directory, entry.name);
    if (entry.isDirectory()) collectFiles(root, absolute, files);
    else if (entry.isFile()) files.push(relative(root, absolute).replaceAll("\\", "/"));
    else throw new Error(`compiler emitted unsupported artifact: ${entry.name}`);
  }
  return files.sort((left, right) => left.localeCompare(right));
}

function validateCompilerArtifact(outputPath, artifact, inputPath) {
  if (artifact == null || typeof artifact !== "object" || Array.isArray(artifact)) {
    throw new Error(`invalid compiler artifact envelope: ${outputPath}`);
  }
  const expectedKeys = [
    "code",
    "css",
    "errors",
    "filename",
    "macro_artifacts",
    "script_lang",
    "warnings",
  ];
  const actualKeys = Object.keys(artifact).sort();
  if (JSON.stringify(actualKeys) !== JSON.stringify(expectedKeys)) {
    throw new Error(`invalid compiler artifact keys in ${outputPath}: ${actualKeys.join(", ")}`);
  }
  const expectedFilename = inputPath.slice(inputPath.lastIndexOf("/") + 1);
  if (artifact.filename !== expectedFilename) {
    throw new Error(
      `compiler filename mismatch in ${outputPath}: expected ${expectedFilename}, received ${artifact.filename}`,
    );
  }
  if (typeof artifact.code !== "string") throw new Error(`invalid compiler code in ${outputPath}`);
  if (artifact.css !== null && typeof artifact.css !== "string") {
    throw new Error(`invalid compiler css in ${outputPath}`);
  }
  if (typeof artifact.script_lang !== "string") {
    throw new Error(`invalid compiler script_lang in ${outputPath}`);
  }
  for (const field of ["errors", "warnings"]) {
    if (
      !Array.isArray(artifact[field]) ||
      artifact[field].some((entry) => typeof entry !== "string")
    ) {
      throw new Error(`invalid compiler ${field} in ${outputPath}`);
    }
  }
  if (!Array.isArray(artifact.macro_artifacts)) {
    throw new Error(`invalid compiler macro_artifacts in ${outputPath}`);
  }
}

export function resolveVizeLaunch(vizeBin, dryRun) {
  const candidates = [
    vizeBin,
    process.env.VIZE_BIN,
    join(repoRoot, "target", "ci", executableName("vize")),
    join(repoRoot, "target", "debug", executableName("vize")),
    join(repoRoot, "target", "release", executableName("vize")),
  ]
    .filter(Boolean)
    .map((candidate) => resolve(candidate));
  for (const candidate of candidates) {
    if (!existsSync(candidate)) continue;
    if (dryRun) return { command: candidate, prefix: [], label: candidate };
    const probe = spawnSync(candidate, ["--version"], {
      encoding: "utf8",
      timeout: 10_000,
    });
    if (probe.status === 0) return { command: candidate, prefix: [], label: candidate };
  }
  if (vizeBin != null && !dryRun) throw new Error(`Vize executable is not runnable: ${vizeBin}`);
  return {
    command: "cargo",
    prefix: ["run", "-q", "-p", "vize", "--"],
    label: "cargo run -q -p vize --",
  };
}

function executableName(name) {
  return process.platform === "win32" ? `${name}.exe` : name;
}
function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}
function failureOutput(result) {
  return { stdout: truncate(result.stdout), stderr: truncate(result.stderr) };
}
function truncate(value) {
  return value.length <= 4000 ? value : `${value.slice(0, 4000)}\n...<truncated>`;
}
