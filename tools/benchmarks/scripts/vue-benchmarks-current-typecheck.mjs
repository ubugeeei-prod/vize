#!/usr/bin/env node
import assert from "node:assert/strict";
import { mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  captureProcess,
  decodeCapture,
  fileEvidence,
  sha256,
} from "./vue-benchmarks-current-typecheck-capture.mjs";
import {
  assertPristine,
  assertOutside,
  archiveInputs,
  FIXTURE_PATH,
  git,
  prepareFixture,
  sourceInventory,
  workspaceInventory,
} from "./vue-benchmarks-current-typecheck-fixture.mjs";

const repository = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");

function parseArgs(argv) {
  const allowed = new Set([
    "--vize-bin",
    "--corsa-bin",
    "--vue-dir",
    "--output",
    "--source-sha",
    "--vue-tsc-bin",
  ]);
  const result = {};
  for (let i = 0; i < argv.length; i += 2) {
    assert.ok(allowed.has(argv[i]), `unknown option ${argv[i]}`);
    assert.ok(argv[i + 1] && !argv[i + 1].startsWith("--"), `missing value for ${argv[i]}`);
    assert.equal(result[argv[i]], undefined, `duplicate option ${argv[i]}`);
    result[argv[i]] = argv[i + 1];
  }
  for (const name of ["--vize-bin", "--corsa-bin", "--vue-dir", "--output", "--source-sha"]) {
    assert.ok(result[name], `${name} is required; there is no package/global binary fallback`);
  }
  return result;
}

function writeJson(path, value) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`, { flag: "wx" });
}

function decodedRun(fixture, raw) {
  const decoded = decodeCapture(raw);
  const stdout = fixture.stripAnsi(decoded.stdout);
  const stderr = fixture.stripAnsi(decoded.stderr);
  assert.equal(
    fixture.bootstrapFailure(stdout + stderr),
    false,
    "upstream bootstrap failure detected",
  );
  return { ...decoded, stdout, stderr, combined: stdout + stderr };
}

function reportRun(fixture, raw, expectedVueFiles, config) {
  const decoded = decodedRun(fixture, raw);
  const report = JSON.parse(decoded.stdout);
  assert.ok(Array.isArray(report.files));
  assert.ok(Array.isArray(report.programs));
  assert.equal(report.fileCount, report.files.length);
  assert.ok(report.files.every((entry) => Array.isArray(entry.diagnostics)));
  assert.deepEqual(
    report.files.filter((entry) => entry.file.endsWith(".vue")).map(({ file }) => file),
    expectedVueFiles,
    "authored Vue coverage differs",
  );
  assert.ok(
    report.files
      .filter((entry) => entry.file.endsWith(".vue"))
      .every((entry) => typeof entry.virtualTs === "string" && entry.virtualTs.length > 0),
  );
  assert.equal(report.programs.length, 1);
  assert.equal(report.programs[0].root, ".");
  assert.equal(report.programs[0].tsconfig, config);
  for (const file of expectedVueFiles) assert.ok(report.programs[0].files.includes(file));
  const combined = report.files
    .filter((entry) => entry.diagnostics.length)
    .map((entry) => `${entry.file}\n${entry.diagnostics.join("\n")}`)
    .join("\n");
  const diags = fixture.parse(combined);
  assert.equal(diags.length, report.errorCount + report.warningCount);
  return { run: { ...decoded, combined }, report, diagnostics: diags };
}

export async function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const sourceSha = git(repository, "rev-parse", "HEAD");
  assert.equal(
    sourceSha,
    args["--source-sha"],
    "source checkout differs from requested build head",
  );
  assert.equal(git(repository, "status", "--porcelain", "--untracked-files=no"), "");
  const requestedOutput = resolve(args["--output"]);
  const output = join(realpathSync(dirname(requestedOutput)), basename(requestedOutput));
  assertOutside(output, realpathSync(join(repository, FIXTURE_PATH)));
  mkdirSync(output, { recursive: false });
  const rawDirectory = join(output, "raw");
  const workRoot = join(output, "workspace");
  const cli = resolve(args["--vize-bin"]);
  const corsa = resolve(args["--corsa-bin"]);
  const fixture = await prepareFixture(repository, workRoot, resolve(args["--vue-dir"]));
  archiveInputs(fixture.upstream, fixture.inventory, join(output, "inputs", "upstream"));
  const authoredInputs = workspaceInventory(workRoot);
  const expectedVueFiles = authoredInputs
    .map(({ path }) => path)
    .filter((path) => path.endsWith(".vue"));
  const providerRoot = dirname(resolve(args["--vue-dir"]));
  assert.equal(basename(providerRoot), "node_modules", "use a dedicated pinned npm installation");
  const providerInputs = workspaceInventory(providerRoot, true);
  const providerLock = fileEvidence(join(dirname(providerRoot), "package-lock.json"));
  writeJson(join(output, "inputs.json"), {
    authoredInputs,
    providerRoot,
    providerInputs,
    providerLock,
  });
  const capture = (id, command, commandArgs) =>
    captureProcess(rawDirectory, id, command, commandArgs, workRoot, {
      CORSA_PATH: corsa,
      NODE_PATH: [providerRoot, process.env.NODE_PATH ?? ""]
        .filter(Boolean)
        .join(process.platform === "win32" ? ";" : ":"),
    });
  const identities = {};
  for (const [id, command] of [
    ["vize", cli],
    ["corsa", corsa],
  ]) {
    const header = readFileSync(command).subarray(0, 20);
    assert.deepEqual(header.subarray(0, 4), Buffer.from([0x7f, 0x45, 0x4c, 0x46]));
    assert.equal(header[4], 2, "require an actual ELF64 native binary");
    assert.equal(header[5], 1, "require little-endian native binary");
    assert.equal(header.readUInt16LE(18), 62, "require actual Linux x86-64 code");
    const raw = capture(`${id}-version`, command, ["--version"]);
    const version = decodeCapture(raw);
    assert.equal(version.status, 0, "version probe failed");
    if (id === "corsa") assert.match(version.stdout + version.stderr, /\b7\.0\.2\b/);
    identities[id] = {
      ...fileEvidence(command),
      elfHeader: header.toString("hex"),
      version,
      raw: raw.observation,
    };
  }
  writeJson(join(output, "custody.json"), {
    sourceSha,
    sourceTree: git(repository, "rev-parse", "HEAD^{tree}"),
    workflowRun: process.env.GITHUB_RUN_ID ?? null,
    workflowAttempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
    workflowRef: process.env.GITHUB_WORKFLOW_REF ?? null,
    executedWorkflowSha: process.env.EXECUTED_WORKFLOW_SHA ?? null,
    node: {
      version: process.version,
      executable: fileEvidence(process.execPath),
      platform: process.platform,
      arch: process.arch,
    },
    identities,
    upstreamRevision: git(fixture.upstream, "rev-parse", "HEAD"),
    upstreamInventory: fixture.inventory,
    vue: fixture.vue,
    providerLock,
    configs: fixture.configs,
    sourceBuiltBinaryProvenance: "requires hosted build log and workflow checkout reconciliation",
    timing: "not measured",
    profile: "none",
    authenticCpuCaptures: 0,
  });
  const native = [];
  const pairs = [];
  for (const config of ["tsconfig.json", fixture.fallthroughTsconfig]) {
    const label = config === "tsconfig.json" ? "shared" : "fallthrough";
    const commandArgs = ["check", ".", "--tsconfig", config, "--corsa-path", corsa];
    const text = capture(`vize-${label}-text`, cli, commandArgs);
    const textRun = decodedRun(fixture, text);
    assert.ok([0, 1].includes(textRun.status), "unexpected native checker exit status");
    const textDiagnostics = fixture.parse(textRun.combined);
    writeJson(join(output, `vize-${label}-text-diagnostics.json`), textDiagnostics);
    const reports = [];
    for (const servers of [1, 2]) {
      const raw = capture(`vize-${label}-json-${servers}`, cli, [
        ...commandArgs,
        "--format",
        "json",
        "--quiet",
        "--show-virtual-ts",
        "--servers",
        String(servers),
      ]);
      const observed = reportRun(fixture, raw, expectedVueFiles, config);
      assert.ok([0, 1].includes(observed.run.status), "unexpected native JSON exit status");
      writeJson(join(output, `vize-${label}-report-${servers}.json`), observed.report);
      reports.push(observed);
      native.push({ label, servers, raw: raw.observation, diagnostics: observed.diagnostics });
    }
    assert.deepEqual(
      reports[0].report,
      reports[1].report,
      "complete ordered 1/2-server reports differ",
    );
    // Both presentations are retained. Upstream's original text parser scores
    // text independently; JSON never supplies the reference expectations.
    pairs.push({ label, textRun, reports });
  }
  const [shared, fallthrough] = pairs;
  const scores = [
    {
      invocation: "native-default-text",
      ...fixture.score(fixture.cases, "vize-check", shared.textRun, fallthrough.textRun),
    },
  ];
  for (let index = 0; index < 2; index++) {
    scores.push({
      invocation: `native-json-${index + 1}`,
      ...fixture.score(
        fixture.cases,
        "vize-check",
        shared.reports[index].run,
        fallthrough.reports[index].run,
      ),
    });
  }
  let reference = null;
  if (args["--vue-tsc-bin"]) {
    const referencePath = resolve(args["--vue-tsc-bin"]);
    const vueTscManifest = join(providerRoot, "vue-tsc", "package.json");
    const typescriptManifest = join(providerRoot, "typescript", "package.json");
    assert.equal(JSON.parse(readFileSync(vueTscManifest)).version, "3.3.11");
    assert.equal(JSON.parse(readFileSync(typescriptManifest)).version, "6.0.3");
    const versionRaw = capture("vue-tsc-version", referencePath, ["--version"]);
    const version = decodeCapture(versionRaw);
    assert.equal(version.status, 0);
    assert.match(version.stdout + version.stderr, /\b6\.0\.3\b/);
    const observations = [];
    for (const config of ["tsconfig.json", fixture.fallthroughTsconfig]) {
      const label = config === "tsconfig.json" ? "shared" : "fallthrough";
      const raw = capture(`vue-tsc-${label}`, referencePath, ["--noEmit", "-p", config]);
      const run = decodedRun(fixture, raw);
      assert.ok([0, 1, 2].includes(run.status), "unexpected reference checker exit status");
      observations.push({ raw: raw.observation, run, diagnostics: fixture.parse(run.combined) });
    }
    reference = {
      ...fileEvidence(referencePath),
      version,
      versionRaw: versionRaw.observation,
      manifests: [fileEvidence(vueTscManifest), fileEvidence(typescriptManifest)],
      observations,
    };
    scores.push({
      invocation: "upstream-vue-tsc-text",
      ...fixture.score(fixture.cases, "vue-tsc", observations[0].run, observations[1].run),
    });
  }
  assertPristine(fixture.upstream);
  assert.deepEqual(fixture.inventory, sourceInventory(fixture.upstream));
  for (const input of authoredInputs) {
    assert.equal(
      fileEvidence(join(workRoot, input.path)).sha256,
      input.sha256,
      `authored input changed during checks: ${input.path}`,
    );
  }
  assert.deepEqual(providerInputs, workspaceInventory(providerRoot, true));
  assert.equal(fileEvidence(providerLock.path).sha256, providerLock.sha256);
  const summary = {
    schema: "vize.currentUpstreamTypecheck",
    version: 1,
    sourceSha,
    originalCaseCount: fixture.cases.length,
    originalCases: fixture.cases.map(({ caseId, meta }) => ({ caseId, meta })),
    published: fixture.published,
    scores,
    native,
    reference,
    acceptance: "evidence only; any failing plant remains unqualified",
    complete: scores
      .filter(({ invocation }) => invocation.startsWith("native-"))
      .every(
        ({ pass, fail, warn, skip, unattributed }) =>
          pass === 154 && fail === 0 && warn === 0 && skip === 0 && unattributed === 0,
      ),
    nativeMigrationCredit: 0,
    speedClaim: null,
    programEvidence:
      "CLI authored root/config membership; no native transitive graph closure claim",
  };
  writeJson(join(output, "summary.json"), summary);
  writeFileSync(
    join(output, "summary.sha256"),
    `${sha256(readFileSync(join(output, "summary.json")))}\n`,
    { flag: "wx" },
  );
  console.log(
    JSON.stringify({
      sourceSha,
      scores: scores.map(({ invocation, pass, fail, warn, skip, unattributed }) => ({
        invocation,
        pass,
        fail,
        warn,
        skip,
        unattributed,
      })),
      complete: summary.complete,
    }),
  );
  if (!summary.complete) process.exitCode = 1;
  return summary;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try {
    await main();
  } catch (error) {
    console.error(error);
    process.exitCode = 1;
  }
}
