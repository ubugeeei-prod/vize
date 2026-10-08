import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { repoRoot } from "../_helpers/realworld-patch.ts";
import { resolveTsgoBinary } from "../_helpers/realworld-typecheck.ts";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";
import { auditSession, readJson } from "./support/current-baseline/audit.ts";
import { assessCampaign, type SessionAssessment } from "./support/current-baseline/assessment.ts";
import {
  git,
  prepareCampaignDirectory,
  sourceCustody,
  writeJson,
} from "./support/current-baseline/inputs.ts";
import { hash } from "./support/current-baseline/wire.ts";

const output = path.resolve(repoRoot, "target/vize-tests/lsp-current-baseline");
const previousMetrics = path.join(repoRoot, "target/vize-tests/metrics/misskey-lsp-churn");

async function execute(directory: string, backend: string) {
  fs.rmSync(previousMetrics, { recursive: true, force: true });
  const child = spawn(
    process.execPath,
    ["--test", "--test-concurrency=1", "tests/performance/lsp-churn-stress.test.ts"],
    {
      cwd: repoRoot,
      detached: true,
      env: {
        ...process.env,
        VIZE_LSP_CURRENT_BASELINE: directory,
        VIZE_LSP_REQUIRE_SOURCE_BUILD: "1",
        VIZE_LSP_BIN: path.join(repoRoot, "target/ci/vize"),
        VIZE_PERF_BUDGET_SCALE: "1",
        VIZE_TEST_TSGO: backend,
        CORSA_PATH: backend,
        VIZE_CORSA_DEV_PATHS: "0",
      },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  let failure: string | null = null;
  let total = 0;
  const terminate = () => {
    if (child.pid) process.kill(-child.pid, "SIGKILL");
  };
  const deadline = setTimeout(() => {
    failure = "original test process exceeded 350s cleanup deadline";
    terminate();
  }, 350000);
  for (const stream of ["stdout", "stderr"] as const) {
    fs.writeFileSync(path.join(directory, `test.${stream}`), "", { flag: "wx" });
    child[stream].on("data", (bytes: Buffer) => {
      total += bytes.length;
      if (total > 16 * 1024 * 1024) {
        failure ??= "test output exceeds 16MiB bound";
        terminate();
      } else fs.appendFileSync(path.join(directory, `test.${stream}`), bytes);
    });
  }
  child.on("error", (error) => {
    failure = error.message;
  });
  const result = await new Promise<{ code: number | null; signal: string | null }>((resolve) =>
    child.once("close", (code, signal) => resolve({ code, signal })),
  );
  clearTimeout(deadline);
  writeJson(path.join(directory, "test-process.json"), { ...result, failure });
  if (fs.existsSync(path.join(previousMetrics, "metrics.json"))) {
    fs.copyFileSync(
      path.join(previousMetrics, "metrics.json"),
      path.join(directory, "churn-metrics.json"),
    );
  }
  assert.equal(result.code, 0, "complete original semantic/budget test must pass");
  assert.equal(result.signal, null);
  assert.equal(failure, null);
}

async function main() {
  prepareCampaignDirectory(output);
  assert.equal(process.platform, "linux");
  assert.match(process.env.VIZE_BASELINE_EXPECTED_SOURCE ?? "", /^[0-9a-f]{40}$/);
  assert.equal(git(repoRoot, "rev-parse", "HEAD"), process.env.VIZE_BASELINE_EXPECTED_SOURCE);
  assert.equal(process.env.VIZE_PERF_BUDGET_SCALE ?? "1", "1");
  const sessions: SessionAssessment[] = [];
  let custody: ReturnType<typeof sourceCustody> | null = null;
  try {
    const backend = fs.realpathSync(resolveTsgoBinary());
    custody = sourceCustody(repoRoot, backend);
    const expected = expectedBuildIdentity(repoRoot);
    validateBuildReceipt(
      readJson(repoRoot, `${expected.binaryPath}.differential-build.json`),
      expected,
    );
    writeJson(path.join(output, "custody.json"), {
      ...custody,
      binary: expected,
      workload: {
        fixture: "misskey",
        sessions: 3,
        cyclesPerSession: 40,
        originalEditsPerCycle: 4,
        extraCompletionsPerCycle: 1,
        samplerIntervalMs: 50,
      },
      hardware: {
        cpuCount: os.cpus().length,
        cpuModel: os.cpus()[0]?.model ?? null,
        memoryBytes: os.totalmem(),
        kernel: os.release(),
      },
      controlledEnvironment: {
        RAYON_NUM_THREADS: process.env.RAYON_NUM_THREADS ?? null,
        GOMAXPROCS: process.env.GOMAXPROCS ?? null,
        VIZE_PERF_BUDGET_SCALE: "1",
        VIZE_CORSA_DEV_PATHS: "0",
      },
      clock:
        "Node hrtime and Linux CLOCK_MONOTONIC; completed transport chunk, client attempted stdin write",
      instrumentation:
        "bounded synchronous raw writes and separate RSS sampler; observations describe this instrumented workload",
    });
    for (let index = 0; index < 3; index += 1) {
      const directory = path.join(output, `session-${index + 1}`);
      fs.mkdirSync(directory);
      const session: (typeof sessions)[number] = {
        index: index + 1,
        status: "failed",
        failure: null,
        report: null,
      };
      sessions.push(session);
      try {
        await execute(directory, backend);
        assert.equal(git(repoRoot, "rev-parse", "HEAD"), custody.sourceRevision);
        assert.equal(git(repoRoot, "rev-parse", "HEAD^{tree}"), custody.sourceTree);
        assert.equal(
          git(repoRoot, "status", "--porcelain", "--untracked-files=no"),
          "",
          "source stayed immutable throughout execution",
        );
        assert.equal(
          hash(fs.readFileSync(path.join(repoRoot, expected.binaryPath))),
          expected.binarySha256,
        );
        session.report = auditSession(directory, expected, custody);
        session.status = "passed";
      } catch (error) {
        session.failure = error instanceof Error ? error.message : String(error);
      }
      writeJson(path.join(directory, "assessment.json"), session);
    }
  } catch (error) {
    sessions.push({
      index: 0,
      status: "failed",
      failure: error instanceof Error ? error.message : String(error),
      report: null,
    });
  }
  const assessment = assessCampaign(sessions);
  writeJson(path.join(output, "assessment.json"), {
    schema: "vize.lsp.current-source-baseline",
    version: 1,
    ...assessment,
    custody,
    limits:
      "single fixture real RPC; shared refresh causality unknown; three fresh processes are not cold cache or startup p95; sampled RSS is not peak/PSS; GUI, native/default/history and 10x acceptance unfinished",
    nativeAcceptance: 0,
  });
  assert(
    assessment.status === "passed",
    "all three complete fresh sessions must qualify; failures and raw evidence retained",
  );
}

await main();
