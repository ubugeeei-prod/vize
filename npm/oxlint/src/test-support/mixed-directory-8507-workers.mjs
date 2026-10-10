// Fixed independent fixture partitions; one unchanged deadline owns both children.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawn } from "node:child_process";

const wholeError = (error) =>
  error == null
    ? null
    : {
        name: error.name,
        ...Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]])),
      };
const observationKey = (record) =>
  JSON.stringify([record.fixture, record.format ?? null, record.surface]);

export function mixedCells(plan) {
  return plan.cases.flatMap((fixture) =>
    (fixture.surface === "cli" ? ["implicit-default", ...plan.formats] : [null]).map((format) => ({
      fixture,
      format,
    })),
  );
}

export function joinMixedCaptures(plan, custody, captures) {
  assert.equal(captures.length, 2);
  const cells = mixedCells(plan);
  const keys = (selected) =>
    selected.flatMap(({ fixture, format }) =>
      fixture.surface === "cli"
        ? ["stock", "wrapper"].map((surface) =>
            observationKey({ fixture: fixture.name, format, surface }),
          )
        : [observationKey({ fixture: fixture.name, surface: "native" })],
    );
  const all = new Map();
  let shared, sharedEnvironment;
  for (const [index, capture] of captures.entries()) {
    assert.equal(capture.complete, true);
    assert.equal(capture.workerIndex, index);
    const selected = cells.filter((_, cellIndex) => cellIndex % 2 === index);
    assert.deepEqual(capture.passed, {
      cli: selected.filter(({ fixture }) => fixture.surface === "cli").length,
      native: selected.filter(({ fixture }) => fixture.surface === "native").length,
    });
    assert.deepEqual(capture.observations.map(observationKey), keys(selected));
    const {
      complete: _complete,
      workerIndex: _workerIndex,
      custody: workerCustody,
      initialEvents: _initialEvents,
      observations: _observations,
      passed: _passed,
      environment: workerEnvironment,
      ...metadata
    } = capture;
    const directory = path.join(path.dirname(custody.calls), `worker-${index}`);
    assert.equal(workerCustody.calls, path.join(directory, "source-native-calls.jsonl"));
    assert.equal(
      workerEnvironment.VIZE_OXLINT_NATIVE_CUSTODY,
      path.join(directory, "source-custody.json"),
    );
    const { VIZE_OXLINT_NATIVE_CUSTODY: _configuration, ...environment } = workerEnvironment;
    const { calls: _calls, ...authority } = workerCustody;
    const { calls: _parentCalls, ...parentAuthority } = custody;
    assert.deepEqual(authority, parentAuthority);
    if (shared) {
      assert.deepEqual(metadata, shared);
      assert.deepEqual(environment, sharedEnvironment);
    } else {
      shared = metadata;
      sharedEnvironment = environment;
    }
    for (const record of capture.observations) {
      const key = observationKey(record);
      assert.ok(!all.has(key), `duplicate original observation: ${key}`);
      all.set(key, record);
    }
  }
  const expectedKeys = keys(cells);
  assert.equal(expectedKeys.length, 98);
  assert.equal(all.size, expectedKeys.length);
  const passed = {
    cli: captures.reduce((sum, capture) => sum + capture.passed.cli, 0),
    native: captures.reduce((sum, capture) => sum + capture.passed.native, 0),
  };
  assert.deepEqual(passed, { cli: 45, native: 8 });
  return {
    ...shared,
    complete: true,
    custody,
    workerEnvironments: captures.map((capture) => capture.environment),
    initialEvents: captures.flatMap((capture) => capture.initialEvents),
    observations: expectedKeys.map((key) => {
      assert.ok(all.has(key), `missing original observation: ${key}`);
      return all.get(key);
    }),
    passed,
  };
}

export async function runMixedWorkers({ packageDir, output, custody, preload, environment }) {
  assert.notEqual(process.platform, "win32", "source Actions requires owned POSIX process groups");
  const argv = ["src/mixed-directory-8507.test.mjs"];
  const records = [0, 1].map((index) => {
    const directory = path.join(output, `worker-${index}`);
    fs.mkdirSync(directory);
    const configuration = path.join(directory, "source-custody.json");
    const calls = path.join(directory, "source-native-calls.jsonl");
    const capture = path.join(directory, "source-after.json");
    fs.writeFileSync(configuration, JSON.stringify({ ...custody, calls }, null, 2) + "\n");
    return {
      index,
      command: [process.execPath, ...argv],
      cwd: packageDir,
      configuration,
      calls,
      capture,
      pid: null,
      status: null,
      signal: null,
      error: null,
      stdoutBytes: [],
      stderrBytes: [],
    };
  });
  const children = [];
  const started = performance.now();
  let failure = null;
  let totalOutputBytes = 0;
  const stop = (reason) => {
    failure ??= reason;
    for (const { index, child, closed } of children) {
      if (closed || !child.pid) continue;
      records[index].terminationReason ??= reason;
      try {
        // Each child owns its process group, including its original stock/wrapper descendants.
        process.kill(-child.pid, "SIGTERM");
      } catch (error) {
        if (error.code !== "ESRCH") records[index].terminationError = wholeError(error);
      }
    }
  };
  const deadline = setTimeout(() => stop("shared 180s deadline"), 180_000);
  const completions = records.map(
    (record) =>
      new Promise((resolve) => {
        const { index, configuration, capture } = record;
        const directory = path.dirname(configuration);
        const env = {
          ...environment,
          NODE_OPTIONS:
            `${environment.NODE_OPTIONS ?? ""} --require=${JSON.stringify(preload)}`.trim(),
          VIZE_OXLINT_NATIVE_CUSTODY: configuration,
          VIZE_OXLINT_SOURCE_BIN: path.join(packageDir, "dist/cli.mjs"),
          VIZE_OXLINT_MIXED_CAPTURE: capture,
          VIZE_OXLINT_MIXED_WORKER: String(index),
        };
        const save = () =>
          fs.writeFileSync(
            path.join(directory, "source-process.json"),
            JSON.stringify(record, null, 2) + "\n",
          );
        save();
        let child;
        try {
          child = spawn(process.execPath, argv, {
            cwd: packageDir,
            env,
            detached: true,
            stdio: ["ignore", "pipe", "pipe"],
          });
        } catch (error) {
          record.error = wholeError(error);
          save();
          stop(`worker ${index} spawn error`);
          resolve();
          return;
        }
        const owned = { index, child, closed: false };
        children.push(owned);
        record.pid = child.pid ?? null;
        const streams = { stdout: [], stderr: [] };
        for (const stream of ["stdout", "stderr"])
          child[stream].on("data", (chunk) => {
            streams[stream].push(Buffer.from(chunk));
            totalOutputBytes += chunk.length;
            if (totalOutputBytes > 64 * 1024 * 1024) stop("shared original 64MiB output bound");
          });
        child.on("error", (error) => {
          record.error = wholeError(error);
          stop(`worker ${index} spawn error`);
        });
        child.on("exit", (status, signal) => {
          if (status !== 0 || signal !== null) stop(`worker ${index} fatal terminal`);
        });
        child.on("close", (status, signal) => {
          owned.closed = true;
          record.status = status;
          record.signal = signal;
          record.walltimeMs = performance.now() - started;
          record.stdoutBytes = Array.from(Buffer.concat(streams.stdout));
          record.stderrBytes = Array.from(Buffer.concat(streams.stderr));
          save();
          if (status !== 0 || signal !== null || record.error !== null)
            stop(`worker ${index} fatal terminal`);
          resolve();
        });
        if (failure) stop(failure);
      }),
  );
  try {
    await Promise.all(completions);
  } finally {
    clearTimeout(deadline);
  }
  const packet = {
    deadlineMs: 180_000,
    maxBufferBytes: 64 * 1024 * 1024,
    totalOutputBytes,
    failure,
    walltimeMs: performance.now() - started,
    workers: records,
  };
  fs.writeFileSync(
    path.join(output, "source-process.json"),
    JSON.stringify(packet, null, 2) + "\n",
  );
  for (const record of records) {
    process.stdout.write(Buffer.from(record.stdoutBytes));
    process.stderr.write(Buffer.from(record.stderrBytes));
  }
  assert.equal(failure, null);
  for (const record of records) {
    assert.equal(record.signal, null);
    assert.equal(record.error, null);
    assert.equal(record.status, 0);
  }
  return records;
}
