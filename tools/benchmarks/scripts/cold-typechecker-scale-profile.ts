/** Hardware counters / source stacks for the checked process after fixture preparation. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { parseArgs } from "node:util";
import { pathToFileURL } from "node:url";
import { prepareColdScale, verifyColdDiagnostics } from "./cold-typechecker-scale.ts";
import { coldScaleCommand } from "./cold-typechecker-scale-command.ts";
import { captureSourceOperationPerf } from "./source-operation-perf.ts";

export function captureColdScalePerf(options: {
  provider: string;
  out: string;
  files: number;
  tool: "cli" | "mapper";
  mode: "stat" | "record";
  perf: string;
  source: string;
  binary?: string;
  addr2line?: string;
  events?: "core" | "cache";
  calibration?: string;
}) {
  const { out, provider, files, tool } = options;
  assert.ok(!existsSync(out), "refuse to overwrite profiling evidence");
  mkdirSync(join(out, "tmp"), { recursive: true });
  const corpus = prepareColdScale(join(out, "project"), provider, files, false);
  const { command, backend } = coldScaleCommand({ ...options, app: corpus.app });
  const sha = (path: string) => createHash("sha256").update(readFileSync(path)).digest("hex");
  writeFileSync(join(out, "corpus.json"), JSON.stringify(corpus, null, 2) + "\n");
  // Preparation, census, hashing and report rendering are outside perf's child window.
  const evidence = join(out, "direct-process");
  const receipt = captureSourceOperationPerf({
    mode: options.mode,
    perfExecutable: options.perf,
    addr2lineExecutable: options.addr2line,
    eventGroup: options.events,
    calibrationPath: options.calibration,
    outputDir: evidence,
    command,
    identity: {
      source: options.source,
      corpusSha256: corpus.corpusSha256,
      files,
      sfcFiles: corpus.sfcFiles,
      sourceBytes: corpus.sourceBytes,
      backend,
      backendSha256: sha(backend),
      providerLockSha256: sha(join(provider, "package-lock.json")),
      rustFeatures: "normal binary; allocation instrumentation disabled",
      affinity: "inherited from caller; workflow pins eight CPUs",
      threads: {
        GOMAXPROCS: process.env.GOMAXPROCS,
        RAYON_NUM_THREADS: process.env.RAYON_NUM_THREADS,
      },
      cold: {
        process: "fresh",
        project: "unique",
        sourceConfigIncremental: false,
        effectiveBackendIncremental: tool === "cli",
        cacheQualified: false,
        osPageCache: "not flushed",
      },
    },
  });
  let correctness: { passed: boolean; diagnostics?: unknown; error?: string };
  try {
    assert.equal(receipt.collection, "collected", "PMU unavailable means command may not have run");
    assert.equal(receipt.commandExit.status, 0, "direct command must complete successfully");
    correctness = {
      passed: true,
      diagnostics: verifyColdDiagnostics(
        readFileSync(join(evidence, "stdout.txt"), "utf8"),
        tool,
        files,
        false,
      ),
    };
  } catch (error) {
    correctness = { passed: false, error: String(error) };
  }
  const report = {
    collection: receipt.collection,
    correctness,
    qualified: receipt.qualified && correctness.passed,
    receipt: "direct-process/receipt.json",
  };
  writeFileSync(join(out, "acceptance.json"), JSON.stringify(report, null, 2) + "\n");
  console.log(JSON.stringify(report));
  assert.notEqual(receipt.collection, "failed", "source profiling collection failed");
  if (receipt.collection === "collected") assert.ok(correctness.passed, correctness.error);
  if (!report.qualified)
    console.warn("Profiling evidence is UNQUALIFIED; inspect direct-process/receipt.json.");
  return report;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      provider: { type: "string" },
      out: { type: "string" },
      files: { type: "string" },
      tool: { type: "string", default: "cli" },
      mode: { type: "string", default: "stat" },
      perf: { type: "string" },
      source: { type: "string" },
      binary: { type: "string" },
      addr2line: { type: "string" },
      events: { type: "string", default: "core" },
      calibration: { type: "string" },
    },
  });
  assert.ok(values.provider && values.out && values.perf && values.source);
  assert.ok(values.tool === "cli" || values.tool === "mapper");
  assert.ok(values.mode === "stat" || values.mode === "record");
  assert.ok(values.events === "core" || values.events === "cache");
  captureColdScalePerf({
    provider: resolve(values.provider),
    out: resolve(values.out),
    files: Number(values.files),
    tool: values.tool,
    mode: values.mode,
    perf: values.perf,
    source: values.source,
    binary: values.binary,
    addr2line: values.addr2line,
    events: values.events,
    calibration: values.calibration,
  });
}
