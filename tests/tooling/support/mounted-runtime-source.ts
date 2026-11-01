import assert from "node:assert/strict";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

/** Cache generated code, including a failed attempt, separately for both modes.
 * The entry and every other build flag are fixed by mounted-runtime-producer.
 * Each observer process starts at most two producers, with no silent retries.
 * The caller still imports the same data URL, preserving module identity.
 */
export function runtimeSourceCache(produce: (production: boolean) => Promise<string>) {
  const sources = new Map<boolean, Promise<string>>();
  return (production: boolean) => {
    assert.equal(typeof production, "boolean", "production runtime mode must be explicit");
    let source = sources.get(production);
    if (!source) {
      source = Promise.resolve().then(() => produce(production));
      sources.set(production, source);
    }
    return source;
  };
}

/** Wait for natural exit and both complete streams before accepting any code.
 * Keep even non-UTF8 output in failures; a passing payload cannot hide a signal.
 */
export function collectRuntimeProducer(
  child: ChildProcessWithoutNullStreams,
  production?: boolean,
) {
  return new Promise<string>((resolve, reject) => {
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    let spawnError: Error | undefined;
    let inputError: Error | undefined;
    child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
    child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
    child.on("error", (error) => (spawnError = error));
    child.stdin.on("error", (error) => (inputError = error));
    child.on("close", (code, signal) => {
      const out = Buffer.concat(stdout);
      const err = Buffer.concat(stderr);
      const directory = process.env.VIZE_MOUNTED_RUNTIME_PRODUCER_EVIDENCE;
      const failuresOnly = process.env.VIZE_MOUNTED_RUNTIME_PRODUCER_EVIDENCE_FAILURES_ONLY === "1";
      const name = `${production ? "production" : "development"}-${child.pid ?? "unspawned"}`;
      const statusFailed =
        spawnError !== undefined ||
        inputError !== undefined ||
        code !== 0 ||
        signal !== null ||
        err.length !== 0;
      const captureErrors: unknown[] = [];
      let failureRecorded = false;
      const captureFailure = (phase: "terminal-status" | "protocol-rejection") => {
        if (!directory || production === undefined || failureRecorded) return;
        failureRecorded = true;
        const prefix = `${name}-parent-${process.pid}`;
        const stream = (kind: string, bytes: Buffer) => ({
          file: `${prefix}.${kind}.bin`,
          bytes: bytes.length,
          sha256: createHash("sha256").update(bytes).digest("hex"),
          persisted: false,
          error: null as string | null,
        });
        const streams = { stdout: stream("stdout", out), stderr: stream("stderr", err) };
        try {
          mkdirSync(directory, { recursive: true });
        } catch (error) {
          captureErrors.push(error);
        }
        for (const [kind, bytes] of [
          ["stdout", out],
          ["stderr", err],
        ] as const) {
          try {
            writeFileSync(join(directory, streams[kind].file), bytes);
            streams[kind].persisted = true;
          } catch (error) {
            streams[kind].error = error instanceof Error ? error.message : String(error);
            captureErrors.push(error);
          }
        }
        const manifest = {
          schema: "vize.mounted-runtime-producer-failure.v1",
          accepted: false,
          phase,
          source: process.env.GITHUB_SHA ?? null,
          run: process.env.GITHUB_RUN_ID ?? null,
          attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
          pid: child.pid ?? null,
          production,
          // Declared spawn selection; this does not attest a physical crash image.
          selectedExecutable: child.spawnfile ?? null,
          parentExecutable: process.execPath,
          exitCode: code,
          signal,
          spawnError: spawnError?.message ?? null,
          inputError: inputError?.message ?? null,
          streams,
          completeStreamsPersisted: Object.values(streams).every((stream) => stream.persisted),
        };
        try {
          writeFileSync(join(directory, `${prefix}.failure.json`), JSON.stringify(manifest) + "\n");
        } catch (error) {
          captureErrors.push(error);
        }
      };
      try {
        if (statusFailed) captureFailure("terminal-status");
        if (directory && production !== undefined && !failuresOnly && !statusFailed) {
          mkdirSync(directory, { recursive: true });
          writeFileSync(join(directory, `${name}.stdout.bin`), out);
          writeFileSync(join(directory, `${name}.stderr.bin`), err);
        }
        assert.equal(spawnError, undefined, "runtime producer spawn failed");
        assert.equal(inputError, undefined, "runtime producer input failed");
        assert.equal(code, 0, "runtime producer must exit naturally with status 0");
        assert.equal(signal, null, "runtime producer must not signal");
        assert.deepEqual(err, Buffer.alloc(0), "runtime producer stderr must remain empty");
        const packet = JSON.parse(out.toString("utf8"));
        assert.deepEqual(Object.keys(packet), ["code"], "runtime producer packet fields");
        assert.equal(typeof packet.code, "string", "runtime producer must return code");
        assert.ok(packet.code.length > 0, "runtime producer code must not be empty");
        assert.deepEqual(out, Buffer.from(`${JSON.stringify(packet)}\n`), "whole producer packet");
        if (directory && production !== undefined && !failuresOnly) {
          writeFileSync(
            join(directory, `${name}.json`),
            `${JSON.stringify({
              pid: child.pid ?? null,
              production,
              exitCode: code,
              signal,
              spawnError: null,
              inputError: null,
              stdout: { bytes: out.length, sha256: createHash("sha256").update(out).digest("hex") },
              stderr: { bytes: err.length, sha256: createHash("sha256").update(err).digest("hex") },
            })}\n`,
          );
        }
        resolve(packet.code);
      } catch (cause) {
        captureFailure("protocol-rejection");
        reject(
          new Error(
            `mounted runtime producer failed\n${JSON.stringify({
              pid: child.pid ?? null,
              exitCode: code,
              signal,
              spawnError: spawnError?.message ?? null,
              inputError: inputError?.message ?? null,
              stdout: out.toString("utf8"),
              stdoutBytes: [...out],
              stderr: err.toString("utf8"),
              stderrBytes: [...err],
            })}`,
            {
              cause: captureErrors.length
                ? new AggregateError(
                    [cause, ...captureErrors],
                    "runtime producer failed; evidence capture also failed",
                  )
                : cause,
            },
          ),
        );
      }
    });
  });
}

export const mountedRuntimeSource = runtimeSourceCache((production) => {
  const child = spawn(
    process.execPath,
    [fileURLToPath(new URL("./mounted-runtime-producer.ts", import.meta.url))],
    { stdio: ["pipe", "pipe", "pipe"] },
  );
  const result = collectRuntimeProducer(child, production);
  child.stdin.end(JSON.stringify({ production }));
  return result;
});
