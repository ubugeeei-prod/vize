import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";

const errorPacket = (error) =>
  error == null
    ? null
    : {
        name: error.name,
        ...Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]])),
      };

// One original campaign deadline/output budget owns BOTH fixed child groups.
export async function runBaselineProcesses(
  jobs,
  output,
  { deadlineMs = 900_000, maxBufferBytes = 32 * 1024 * 1024, startedAt } = {},
) {
  assert.notEqual(process.platform, "win32", "source Actions requires owned POSIX groups");
  assert.equal(jobs.length, 2);
  assert.deepEqual(
    jobs.map(({ index }) => index),
    [0, 1],
  );
  assert.ok(Number.isInteger(deadlineMs) && deadlineMs > 0 && deadlineMs <= 900_000);
  assert.ok(
    Number.isInteger(maxBufferBytes) && maxBufferBytes > 0 && maxBufferBytes <= 32 * 1024 * 1024,
  );
  assert.equal(new Set(jobs.map((job) => path.resolve(job.output))).size, 2);
  const started = startedAt ?? performance.now();
  assert.ok(Number.isFinite(started) && started <= performance.now());
  let failure = null,
    totalOutputBytes = 0;
  const children = [];
  const records = jobs.map((job) => ({
    index: job.index,
    command: job.command,
    args: job.args,
    cwd: job.cwd,
    output: job.output,
    pid: null,
    status: null,
    signal: null,
    error: null,
    closed: false,
    stdoutBytes: 0,
    stderrBytes: 0,
  }));
  const save = (record) => {
    try {
      fs.writeFileSync(
        path.join(record.output, "worker-process.json"),
        JSON.stringify(record, null, 2) + "\n",
      );
    } catch (error) {
      record.error ??= errorPacket(error);
      stop(`worker ${record.index} evidence write error`);
    }
  };
  const terminateGroup = (child, record, reason) => {
    if (!child.pid) return;
    try {
      process.kill(-child.pid, "SIGKILL");
      record.terminationReason ??= reason;
    } catch (error) {
      if (error.code !== "ESRCH") {
        record.terminationError = errorPacket(error);
        failure ??= `worker ${record.index} owned group termination error`;
      }
    }
  };
  const stop = (reason) => {
    failure ??= reason;
    for (const { child, record } of children) terminateGroup(child, record, reason);
  };
  const onInterrupt = () => stop("parent interrupted by SIGINT");
  const onTerminate = () => stop("parent interrupted by SIGTERM");
  const onExit = () => stop("parent exited before child drain");
  process.once("SIGINT", onInterrupt);
  process.once("SIGTERM", onTerminate);
  process.once("exit", onExit);
  const remainingMs = Math.max(0, deadlineMs - (performance.now() - started));
  const timer = setTimeout(() => stop("shared original 900s deadline"), remainingMs);
  if (remainingMs === 0) stop("shared original 900s deadline");
  try {
    await Promise.all(
      jobs.map(
        (job, index) =>
          new Promise((resolve) => {
            const record = records[index];
            try {
              fs.mkdirSync(job.output, { recursive: true });
              for (const stream of ["stdout", "stderr"])
                fs.writeFileSync(path.join(job.output, `worker.${stream}.bin`), "");
            } catch (error) {
              record.error = errorPacket(error);
              stop(`worker ${index} output setup error`);
              resolve();
              return;
            }
            if (failure) {
              record.error = { name: "NotStarted", message: failure };
              save(record);
              resolve();
              return;
            }
            let child;
            try {
              child = spawn(job.command, job.args, {
                cwd: job.cwd,
                env: job.env,
                detached: true,
                stdio: ["ignore", "pipe", "pipe"],
              });
            } catch (error) {
              record.error = errorPacket(error);
              stop(`worker ${index} spawn error`);
              save(record);
              resolve();
              return;
            }
            record.pid = child.pid ?? null;
            children.push({ child, record });
            save(record);
            for (const stream of ["stdout", "stderr"])
              child[stream].on("data", (chunk) => {
                // Capture the complete actual chunk before enforcing the shared bound.
                try {
                  fs.appendFileSync(path.join(job.output, `worker.${stream}.bin`), chunk);
                } catch (error) {
                  record.error ??= errorPacket(error);
                  stop(`worker ${index} capture error`);
                }
                record[`${stream}Bytes`] += chunk.length;
                totalOutputBytes += chunk.length;
                if (totalOutputBytes > maxBufferBytes) stop("shared original 32MiB output bound");
              });
            child.once("error", (error) => {
              record.error = errorPacket(error);
              stop(`worker ${index} process error`);
            });
            // Stop the sibling immediately, before waiting for a fatal child's pipes.
            child.once("exit", (status, signal) => {
              if (status !== 0 || signal !== null) stop(`worker ${index} fatal terminal`);
            });
            child.once("close", (status, signal) => {
              // Ignored pipes can hide a still-live grandchild after a normal exit.
              terminateGroup(child, record, "closed worker residual group cleanup");
              if (failure) stop(failure);
              record.status = status;
              record.signal = signal;
              record.closed = true;
              record.walltimeMs = performance.now() - started;
              save(record);
              resolve();
            });
            if (failure) stop(failure);
          }),
      ),
    );
  } finally {
    clearTimeout(timer);
    for (const { record } of children)
      assert.equal(record.closed, true, "every started child must close before completion");
    process.removeListener("SIGINT", onInterrupt);
    process.removeListener("SIGTERM", onTerminate);
    process.removeListener("exit", onExit);
  }
  const packet = {
    deadlineMs,
    maxBufferBytes,
    totalOutputBytes,
    failure,
    walltimeMs: performance.now() - started,
    workers: records,
  };
  fs.writeFileSync(
    path.join(output, "baseline-processes.json"),
    JSON.stringify(packet, null, 2) + "\n",
  );
  const stdout = Buffer.concat(
    records.map((record) => fs.readFileSync(path.join(record.output, "worker.stdout.bin"))),
  );
  const stderr = Buffer.concat(
    records.map((record) => fs.readFileSync(path.join(record.output, "worker.stderr.bin"))),
  );
  // Display order is fixed; each original byte stream remains separately saved.
  process.stdout.write(stdout);
  process.stderr.write(stderr);
  return {
    status:
      failure === null &&
      records.every(
        (record) => record.status === 0 && record.signal === null && record.error === null,
      )
        ? 0
        : 1,
    signal: null,
    ...(failure ? { error: { name: "BaselineCampaignFailure", message: failure } } : {}),
    stdout: stdout.toString("utf8"),
    stderr: stderr.toString("utf8"),
    packet,
  };
}
