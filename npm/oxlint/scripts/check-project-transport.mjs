// Reuse the existing JS Actions source build; never qualify a downloaded addon.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import { beginN8nReplay, finishN8nReplay } from "./n8n-replay-qualification.mjs";
import {
  hostVersions,
  parseHostOptions,
  requireSuccessfulHosts,
  runHostProcesses,
} from "./ci-host-concurrency.mjs";
import { stageProjectNative } from "./project-native-staging.ts";
import { qualifyMixedDirectory8507 } from "./mixed-directory-8507-qualification.mjs";
import {
  nativeHistoryReceipt,
  validateNativeHistoryBuild,
} from "../../native/scripts/formatter-history-build.mjs";

const options = parseHostOptions(process.argv.slice(2));
const packageDir = fileURLToPath(new URL("..", import.meta.url));
const root = path.resolve(packageDir, "../..");
const started = performance.now();
const timing = {
  schema: "vize.oxlint.ci-phase-walltime",
  version: 1,
  mode: options.mode,
  workers: options.workers,
  status: "failed",
  phaseWalltimeMs: {},
  hosts: [],
};
let artifacts;
const temporaryDirectories = [];
const timed = async (name, action) => {
  const phaseStart = performance.now();
  try {
    return await action();
  } finally {
    timing.phaseWalltimeMs[name] = performance.now() - phaseStart;
  }
};
try {
  assert.equal(process.env.GITHUB_ACTIONS, "true", "this source qualification runs in Actions");
  for (const key of [
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_OXLINT_NATIVE_CUSTODY",
  ])
    assert.ok(!process.env[key], `ambient ${key} cannot qualify source identity`);
  const nativeDir = path.join(root, "npm/native");
  const receipt = JSON.parse(fs.readFileSync(nativeHistoryReceipt(nativeDir), "utf8"));
  validateNativeHistoryBuild(nativeDir, receipt);
  assert.equal(receipt.source.head, process.env.GITHUB_SHA);
  timing.source = receipt.source;
  timing.binarySha256 = receipt.frozen.sha256;
  const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
  const staged = stageProjectNative(
    path.join(root, "target/oxlint-original-project-transport"),
    receipt.frozen.path,
    nativeHistoryReceipt(nativeDir),
    receipt.frozen.sha256,
  );
  artifacts = staged.artifacts;
  const { binary } = staged;
  if (process.env.GITHUB_OUTPUT)
    fs.appendFileSync(
      process.env.GITHUB_OUTPUT,
      `staging-directory=${artifacts}\nphase-walltime=${path.join(artifacts, "phase-walltime.json")}\n`,
    );
  const mixedDirectory8507 = await timed("mixedDirectory8507", () =>
    qualifyMixedDirectory8507({ root, packageDir, artifacts, receipt, binary }),
  );
  const n8nReplay = await timed("baselinePreparation", () =>
    beginN8nReplay({ root, packageDir, artifacts, receipt, binary }),
  );
  const worker = fileURLToPath(new URL("./ci-host-worker.mjs", import.meta.url));
  const jobs = hostVersions.map(({ version, types }) => {
    const output = path.join(artifacts, version);
    fs.mkdirSync(output);
    const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-oxlint-project-host-"));
    temporaryDirectories.push(temporary);
    const configuration = path.join(output, "worker-config.json");
    fs.writeFileSync(
      configuration,
      JSON.stringify(
        {
          packageDir,
          artifacts,
          binary,
          version,
          types,
          temporary,
          replay: {
            output: n8nReplay.output,
            custody: n8nReplay.custody,
            reference: n8nReplay.reference,
          },
        },
        null,
        2,
      ) + "\n",
    );
    return {
      id: version,
      output,
      command: process.execPath,
      args: [worker, configuration],
      cwd: packageDir,
      // All worker-created temporary projects, including n8n, live inside this
      // owned directory so parent cleanup also handles timeout or interruption.
      env: { ...process.env, TMPDIR: temporary, TMP: temporary, TEMP: temporary },
    };
  });
  const results = await timed("hosts", () => runHostProcesses(jobs, { workers: options.workers }));
  timing.hosts = results.map((result, index) => {
    const file = path.join(jobs[index].output, "phase-walltime.json");
    const phases = fs.existsSync(file) ? JSON.parse(fs.readFileSync(file, "utf8")) : {};
    return {
      ...hostVersions[index],
      status: result.status === 0 && !result.signal && !result.error ? "passed" : "failed",
      elapsedMs: result.elapsedMs,
      phaseWalltimeMs: phases.phaseWalltimeMs ?? {},
    };
  });
  requireSuccessfulHosts(results); // Every host settles and saves evidence first.
  await timed("aggregation", () => {
    const qualifications = jobs.map(({ output }, index) => {
      const { qualification, n8nHost, n8nCustody } = JSON.parse(
        fs.readFileSync(path.join(output, "host-worker-result.json"), "utf8"),
      );
      assert.equal(qualification.version, hostVersions[index].version);
      assert.equal(qualification.types, hostVersions[index].types);
      assert.equal(n8nHost.version, hostVersions[index].version);
      n8nReplay.hosts.push(n8nHost);
      n8nReplay.hostCustodies.push({ version: n8nHost.version, custody: n8nCustody });
      return qualification;
    });
    validateNativeHistoryBuild(nativeDir, receipt);
    assert.equal(hash(fs.readFileSync(binary)), receipt.frozen.sha256);
    finishN8nReplay(n8nReplay);
    fs.writeFileSync(
      path.join(artifacts, "qualification.json"),
      JSON.stringify(
        {
          source: receipt.source,
          toolchain: receipt.toolchain,
          binarySha256: receipt.frozen.sha256,
          qualifications,
          mixedDirectory8507,
          limits: [
            "authored wrapper controls only",
            "full licensed native/Vize-layer replay is qualified separately in n8n/qualification.json",
            "direct51 scriptless, excluded n8n-local plugins and installed acceptance remain unfinished",
          ],
        },
        null,
        2,
      ) + "\n",
    );
  });
  timing.status = "passed";
} finally {
  for (const temporary of temporaryDirectories)
    fs.rmSync(temporary, { recursive: true, force: true });
  timing.phaseWalltimeMs.total = performance.now() - started;
  // Timings never serialize ambient environment, arguments or error stacks.
  const bytes = JSON.stringify(timing, null, 2) + "\n";
  if (artifacts) fs.writeFileSync(path.join(artifacts, "phase-walltime.json"), bytes);
  if (options.phaseWalltime) {
    const destination = path.resolve(options.phaseWalltime);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, bytes);
  }
}
