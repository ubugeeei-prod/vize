import { spawnSync } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import { readFileSync } from "node:fs";
import { Script } from "node:vm";

/** Official Node 24.14.0 multi-platform index; changing this changes cache keys. */
export const SANDBOX_IMAGE =
  "node:24.14.0-bookworm-slim@sha256:d8e448a56fc63242f70026718378bd4b00f8c82e78d20eefb199224a4d8e33d8";

/** Typed refusal; unavailable isolation never falls back to a trusted callback. */
export class SandboxRuntimeError extends Error {
  constructor(code, message, options) {
    super(message, options);
    this.name = "SandboxRuntimeError";
    this.code = code;
  }
}

/** Run serialized callback source in a restricted container with no host mounts. */
export function createSandboxRunner(input, limits = {}) {
  const definition = validate(input);
  const timeoutMs = integer(limits.timeoutMs ?? 5000, 100, 60000, "timeoutMs");
  const maxBytes = integer(limits.maxBytes ?? 1048576, 1024, 4194304, "maxBytes");
  if (Object.keys(limits).some((key) => !["timeoutMs", "maxBytes"].includes(key))) {
    throw new TypeError("unsupported sandbox limit");
  }
  const worker = readFileSync(new URL("./sandbox-worker.js", import.meta.url), "utf8");
  const sdk = bundledSdk();
  const fingerprint = createHash("sha256")
    .update(JSON.stringify([definition, SANDBOX_IMAGE, worker, sdk, timeoutMs, maxBytes]))
    .digest("hex");
  return Object.freeze({
    name: definition.name,
    version: definition.version,
    visit: definition.visit,
    demands: definition.demands,
    provides: definition.provides,
    family: ["formatter", "output"].includes(definition.family) ? definition.family : undefined,
    cacheInputs: definition.cacheInputs,
    fingerprint,
    run(batchJson) {
      if (typeof batchJson !== "string") throw new TypeError("sandbox batch must be JSON text");
      JSON.parse(batchJson);
      const request = JSON.stringify({ definition, batchJson, sdk });
      if (Buffer.byteLength(request) > maxBytes)
        throw new RangeError("sandbox input limit exceeded");
      const prerequisite = spawnSync(
        "docker",
        ["image", "inspect", "--format={{.Id}}", SANDBOX_IMAGE],
        {
          encoding: "utf8",
          timeout: 5000,
          maxBuffer: 16384,
        },
      );
      if (prerequisite.error || prerequisite.status !== 0) {
        throw new SandboxRuntimeError(
          "runtime_unavailable",
          `Docker and the pre-pulled SANDBOX_IMAGE are required: ${prerequisite.error?.code ?? prerequisite.stderr.trim()}`,
          { cause: prerequisite.error },
        );
      }
      const name = `vize-plugin-${randomUUID()}`;
      let failure;
      let output;
      try {
        const started = performance.now();
        const result = spawnSync(
          "docker",
          [
            "run",
            "--rm",
            "--pull=never",
            "--name",
            name,
            "--label",
            "vize.plugin-sandbox=1",
            "--label",
            `vize.plugin-sandbox.host=${process.pid}`,
            "--network=none",
            "--read-only",
            "--user=65534:65534",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges:true",
            "--pids-limit=32",
            "--memory=128m",
            "--memory-swap=128m",
            "--cpus=1",
            "--ulimit=nofile=64:64",
            "--ipc=none",
            "--log-driver=none",
            "--workdir=/",
            "--entrypoint=node",
            "-i",
            SANDBOX_IMAGE,
            "--max-old-space-size=64",
            "--input-type=module",
            "--eval",
            worker,
          ],
          {
            input: request,
            encoding: "utf8",
            timeout: timeoutMs,
            maxBuffer: maxBytes,
            killSignal: "SIGKILL",
            windowsHide: true,
          },
        );
        if (result.error) {
          const stopped = new SandboxRuntimeError(
            "execution_stopped",
            `sandbox execution stopped: ${result.error.code}`,
            { cause: result.error },
          );
          // Preserve available captured output without automatically logging it.
          // A stopped process does not guarantee a complete worker transcript.
          Object.defineProperty(stopped, "execution", {
            value: Object.freeze({
              phase: "run",
              elapsedMs: performance.now() - started,
              status: result.status,
              signal: result.signal,
              stdout: result.stdout,
              stderr: result.stderr,
            }),
          });
          throw stopped;
        }
        if (result.status !== 0) {
          throw new SandboxRuntimeError(
            "execution_failed",
            `sandbox execution failed (${result.status}): ${result.stderr.trim()}`,
          );
        }
        if (Buffer.byteLength(result.stdout) > maxBytes)
          throw new SandboxRuntimeError("execution_stopped", "sandbox output limit exceeded");
        JSON.parse(result.stdout);
        output = result.stdout;
      } catch (error) {
        failure = error;
      }
      // Execution errors are captured above, so cleanup also runs after failure.
      // Killing the attached CLI does not kill its container.
      try {
        confirmRemoval(name);
      } catch (cleanup) {
        failure = new SandboxRuntimeError(
          "cleanup_failed",
          `sandbox container cleanup was not confirmed: ${cleanup.message}`,
          { cause: new AggregateError([failure, cleanup].filter(Boolean)) },
        );
      }
      if (failure) throw failure;
      return output;
    },
  });
}

function confirmRemoval(name) {
  const deadline = performance.now() + 5000;
  const run = (args) =>
    spawnSync("docker", args, {
      encoding: "utf8",
      timeout: Math.max(1, Math.ceil(deadline - performance.now())),
      maxBuffer: 16384,
      killSignal: "SIGKILL",
    });
  const detail = (result) =>
    `${result.error?.code ?? `status ${result.status}`}: ${result.stderr?.trim() ?? ""}`;
  let last = "cleanup deadline expired";
  while (performance.now() < deadline) {
    const removed = run(["rm", "--force", name]);
    if (!removed.error && removed.status === 0) return;
    last = `rm: ${detail(removed)}`;
    if (performance.now() >= deadline) break;
    // --rm may already be deleting the container. An rm error is not proof of
    // absence: inspect this exact unique name before returning or retrying.
    const inspected = run(["container", "inspect", "--format={{.Id}}", name]);
    last += `; inspect: ${detail(inspected)}`;
    if (
      !inspected.error &&
      inspected.status !== 0 &&
      new RegExp(
        `^(?:Error response from daemon: |Error: )No such (?:object|container): ${name}$`,
      ).test(inspected.stderr.trim())
    )
      return;
    if (
      inspected.error ||
      inspected.status !== 0 ||
      !/^[a-f0-9]{64}$/.test(inspected.stdout.trim())
    ) {
      throw new Error(last);
    }
    // It still exists, including removal-in-progress. All retries share the
    // original deadline; daemon errors and timeout never count as success.
  }
  throw new Error(last);
}

function validate(input) {
  if (!input || typeof input !== "object" || Array.isArray(input))
    throw new TypeError("sandbox manifest required");
  const allowed = [
    "name",
    "version",
    "family",
    "visit",
    "demands",
    "provides",
    "cacheInputs",
    "rules",
    "callback",
    "configuration",
  ];
  if (Object.keys(input).some((key) => !allowed.includes(key)))
    throw new TypeError("unsupported sandbox manifest field");
  if (
    typeof input.name !== "string" ||
    !input.name ||
    typeof input.version !== "string" ||
    !input.version
  ) {
    throw new TypeError("sandbox name and version must be nonempty strings");
  }
  const family = input.family ?? "rule";
  if (!["rule", "transform", "provider", "formatter", "output"].includes(family))
    throw new TypeError("unsupported sandbox family");
  for (const key of ["visit", "demands", "provides"]) {
    if (
      input[key] !== undefined &&
      (!Array.isArray(input[key]) || input[key].some((v) => typeof v !== "string"))
    ) {
      throw new TypeError(`sandbox ${key} must contain strings`);
    }
  }
  if (family === "provider" && !input.provides?.length)
    throw new TypeError("provider must declare provides");
  if (
    input.cacheInputs !== undefined &&
    (!Array.isArray(input.cacheInputs) ||
      input.cacheInputs.some(
        (v) =>
          !v ||
          typeof v.name !== "string" ||
          typeof v.value !== "string" ||
          Object.keys(v).length !== 2,
      ))
  )
    throw new TypeError("sandbox cache inputs need name and value strings");
  if (family === "rule") {
    if (
      !input.rules ||
      Array.isArray(input.rules) ||
      typeof input.rules !== "object" ||
      input.callback !== undefined
    ) {
      throw new TypeError("rule sandbox needs callback sources in rules");
    }
    for (const [id, source] of Object.entries(input.rules)) {
      if (!id) throw new TypeError("rule id must be nonempty");
      parseCallback(source);
    }
  } else {
    if (input.rules !== undefined) throw new TypeError("stage sandbox uses callback, not rules");
    parseCallback(input.callback);
  }
  assertJson(input.configuration ?? null);
  return freeze(JSON.parse(JSON.stringify({ ...input, family })));
}

function parseCallback(source) {
  if (
    typeof source !== "string" ||
    source.length > 65536 ||
    !/^(?:async\s+)?(?:function\b|\([^]*?\)\s*=>|[\w$]+\s*=>)/.test(source.trim())
  ) {
    throw new TypeError("sandbox callback must be standalone function source");
  }
  // Parsing only: this never evaluates guest code in the host process.
  new Script(`(${source})`, { filename: "sandbox-callback.js" });
}

function assertJson(value) {
  if (
    value === null ||
    typeof value === "string" ||
    typeof value === "boolean" ||
    (typeof value === "number" && Number.isFinite(value))
  )
    return;
  if (
    value &&
    typeof value === "object" &&
    (Array.isArray(value) || Object.getPrototypeOf(value) === Object.prototype)
  ) {
    for (const child of Object.values(value)) assertJson(child);
    return;
  }
  throw new TypeError("sandbox configuration must be JSON data");
}

function bundledSdk() {
  const data = (source) => `data:text/javascript;base64,${Buffer.from(source).toString("base64")}`;
  const files = Object.fromEntries(
    ["index.js", "stages.js", "identity.js", "package.json"].map((file) => [
      file,
      readFileSync(new URL(file, import.meta.url)),
    ]),
  );
  // Match the actual package identity's byte-level hash. Its filesystem reader
  // cannot run from a data URL, so transport the resulting identity as a module.
  const hash = createHash("sha256");
  for (const [file, bytes] of Object.entries(files)) {
    hash.update(file).update(String(bytes.length)).update(bytes);
  }
  const identity = data(`export const SDK_FINGERPRINT = ${JSON.stringify(hash.digest("hex"))};`);
  const stage = data(
    files["stages.js"].toString("utf8").replace('from "./identity.js"', `from "${identity}"`),
  );
  const fixes = data(readFileSync(new URL("./fixes.js", import.meta.url), "utf8"));
  const index = files["index.js"]
    .toString("utf8")
    .replace('from "./identity.js"', `from "${identity}"`)
    .replace('from "./stages.js"', `from "${stage}"`)
    .replace('from "./fixes.js"', `from "${fixes}"`);
  return data(index);
}

function integer(value, min, max, name) {
  if (!Number.isInteger(value) || value < min || value > max)
    throw new RangeError(`invalid sandbox ${name}`);
  return value;
}

function freeze(value) {
  if (value && typeof value === "object") {
    for (const child of Object.values(value)) freeze(child);
    Object.freeze(value);
  }
  return value;
}
