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
      try {
        const directory = process.env.VIZE_MOUNTED_RUNTIME_PRODUCER_EVIDENCE;
        if (directory && production !== undefined) {
          mkdirSync(directory, { recursive: true });
          const name = `${production ? "production" : "development"}-${child.pid ?? "unspawned"}`;
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
        if (directory && production !== undefined) {
          const name = `${production ? "production" : "development"}-${child.pid ?? "unspawned"}`;
          writeFileSync(
            join(directory, `${name}.json`),
            `${JSON.stringify({
              pid: child.pid ?? null,
              production,
              exitCode: code,
              signal,
              spawnError: spawnError?.message ?? null,
              inputError: inputError?.message ?? null,
              stdout: { bytes: out.length, sha256: createHash("sha256").update(out).digest("hex") },
              stderr: { bytes: err.length, sha256: createHash("sha256").update(err).digest("hex") },
            })}\n`,
          );
        }
        resolve(packet.code);
      } catch (cause) {
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
            { cause },
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
