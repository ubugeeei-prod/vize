import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

/** Preserve own data descriptors without evaluating error accessors. */
export function errorEvidence(error, seen = new Map()) {
  if (error === null) return { type: "null" };
  const type = typeof error;
  if (type === "symbol") {
    if (seen.has(error)) return { reference: seen.get(error) };
    const id = seen.size;
    seen.set(error, id);
    return { id, type, description: error.description, globalKey: Symbol.keyFor(error) };
  }
  if (type !== "object" && type !== "function") {
    return { type, value: Object.is(error, -0) ? "-0" : String(error) };
  }
  if (seen.has(error)) return { reference: seen.get(error) };
  const id = seen.size;
  seen.set(error, id);
  return {
    id,
    type,
    properties: Reflect.ownKeys(error).map((key) => {
      const descriptor = Object.getOwnPropertyDescriptor(error, key);
      return {
        key: errorEvidence(key, seen),
        enumerable: descriptor.enumerable,
        configurable: descriptor.configurable,
        ...(Object.hasOwn(descriptor, "value")
          ? { writable: descriptor.writable, value: errorEvidence(descriptor.value, seen) }
          : {
              getter: descriptor.get ? errorEvidence(descriptor.get, seen) : null,
              setter: descriptor.set ? errorEvidence(descriptor.set, seen) : null,
            }),
      };
    }),
  };
}

/** Capture original binary streams before decoding, parsing or rejecting. */
export function captureProcess(directory, id, command, args, cwd, extraEnv = {}) {
  assert.match(id, /^[a-z0-9-]+$/);
  const env = { ...process.env, NO_COLOR: "1", FORCE_COLOR: "0", ...extraEnv };
  const options = { cwd, env, timeout: 300_000, maxBuffer: 64 * 1024 * 1024 };
  const result = spawnSync(command, args, options);
  mkdirSync(directory, { recursive: true });
  const streams = {};
  for (const name of ["stdout", "stderr"]) {
    const bytes = result[name];
    if (bytes === null || bytes === undefined) {
      streams[name] = { present: false };
      continue;
    }
    assert.ok(Buffer.isBuffer(bytes));
    const file = `${id}.${name}.bin`;
    writeFileSync(join(directory, file), bytes, { flag: "wx" });
    streams[name] = { present: true, file, bytes: bytes.length, sha256: sha256(bytes) };
  }
  const observation = {
    id,
    command,
    args,
    cwd,
    environment: {
      NO_COLOR: env.NO_COLOR,
      FORCE_COLOR: env.FORCE_COLOR,
      CORSA_PATH: env.CORSA_PATH ?? null,
      NODE_PATH: env.NODE_PATH ?? null,
      LANG: env.LANG ?? null,
      LC_ALL: env.LC_ALL ?? null,
    },
    limits: { timeout: options.timeout, maxBuffer: options.maxBuffer },
    status: result.status,
    signal: result.signal,
    error: result.error ? errorEvidence(result.error) : null,
    streams,
  };
  writeFileSync(join(directory, `${id}.json`), `${JSON.stringify(observation, null, 2)}\n`, {
    flag: "wx",
  });
  return { observation, result };
}

export function decodeCapture(captured) {
  assert.equal(captured.observation.error, null, "raw process error was retained");
  assert.equal(captured.result.signal, null, "raw termination signal was retained");
  assert.ok(
    Number.isInteger(captured.result.status) && captured.result.status >= 0,
    "process did not produce an exit status",
  );
  const stdout = captured.result.stdout?.toString("utf8") ?? "";
  const stderr = captured.result.stderr?.toString("utf8") ?? "";
  return { status: captured.result.status, stdout, stderr, combined: stdout + stderr };
}

export function fileEvidence(path) {
  const bytes = readFileSync(path);
  return { path, bytes: bytes.length, sha256: sha256(bytes) };
}
