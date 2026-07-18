import { spawnSync } from "node:child_process";
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

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

export function runTool(project, tool, args, launch, outputDir) {
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
    exitCode: null,
    outputPath: null,
  };
  if (args.dryRun) return { ...base, status: "planned" };
  if (!fixtureExists) return { ...base, status: "missing-fixture" };

  try {
    const startedAt = Date.now();
    const result = spawnSync(launch.command, commandArgs, {
      cwd,
      encoding: "utf8",
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
      maxBuffer: 1024 * 1024 * 1024,
      timeout: args.timeoutMs,
    });
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
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
    };
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
        payload.parsed = JSON.parse(result.stdout);
      } catch (error) {
        payload.parseError = errorMessage(error);
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
    return { ...completed, status: result.status === 0 ? "ok" : "findings" };
  } finally {
    if (compilerOutputDir != null) rmSync(compilerOutputDir, { recursive: true, force: true });
  }
}

function inspectCompilerArtifacts(cwd, patterns, expectedFileCount, outputDir) {
  const inputPaths = [
    ...new Set(
      patterns.flatMap((pattern) =>
        globSync(pattern, { cwd })
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
    validateCompilerArtifact(outputPath, artifact, inputByOutputPath.get(outputPath));
    errorCount += artifact.errors.length;
    warningCount += artifact.warnings.length;
  }

  return {
    inputFileCount: inputPaths.length,
    outputFileCount: outputPaths.length,
    errorCount,
    warningCount,
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

function toolArgs(project, tool, compilerOutputDir) {
  if (tool === "compiler") {
    return [
      "build",
      ...project.vueGlobs,
      "--format",
      "json",
      "--output",
      compilerOutputDir,
      "--template-syntax",
      "quirks",
      "--continue-on-error",
      "--no-config",
    ];
  }
  if (tool === "linter") {
    return [
      "lint",
      ...project.vueGlobs,
      "--format",
      "json",
      "--preset",
      "ecosystem",
      "--no-config",
    ];
  }
  if (tool === "typechecker") {
    const args = ["check", ...project.vueGlobs, "--format", "json", "--no-config"];
    if (project.tsconfig != null) args.push("--tsconfig", project.tsconfig);
    return args;
  }
  return ["fmt", ...project.vueGlobs, "--check", "--no-config"];
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
function displayCommand(command, args) {
  return [command, ...args].map(shellQuote).join(" ");
}
function shellQuote(value) {
  return /^[A-Za-z0-9_./:=@*-]+$/.test(value) ? value : `'${value.replaceAll("'", "'\\''")}'`;
}
