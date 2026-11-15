import { createHash, randomUUID } from "node:crypto";
import {
  chmodSync,
  closeSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const payload = Buffer.concat([
  Buffer.from("process.argv.splice(1, 1);\n"),
  readFileSync(path.join(root, "crates/vize_carton/src/config/loader/vite-runtime.mjs")),
  readFileSync(path.join(root, "crates/vize_carton/src/config/loader/module-runtime.mjs")),
  readFileSync(path.join(root, "crates/vize_carton/src/config/loader/config-inputs.mjs")),
  Buffer.from("\n//# sourceURL=vize:config-runtime\n"),
]);
const lane = process.argv.find((argument) => argument.startsWith("--lane="))?.slice(7);
const samples = Number(
  process.argv.find((argument) => argument.startsWith("--samples="))?.slice(10) ?? 31,
);
const iterations = Number(
  process.argv.find((argument) => argument.startsWith("--iterations="))?.slice(13) ?? 100,
);
if (!lane || !["base", "head"].includes(lane))
  throw new Error("usage: --lane=base|head [--samples=N] [--iterations=N]");
if (!Number.isSafeInteger(samples) || samples < 1) throw new Error("samples must be positive");
if (!Number.isSafeInteger(iterations) || iterations < 1)
  throw new Error("iterations must be positive");

const owner = mkdtempSync(path.join(os.tmpdir(), "vize-config-runtime-storage-"));
const timings: number[] = [];
let createdFiles = 0;
let writtenBytes = 0;

function baselineInvocation(directory: string): void {
  const file = path.join(directory, `vize-config-runtime-${randomUUID()}.mjs`);
  const descriptor = openSync(file, "wx", 0o600);
  createdFiles += 1;
  try {
    writeFileSync(descriptor, payload);
    writtenBytes += payload.byteLength;
  } finally {
    closeSync(descriptor);
  }
  readFileSync(file);
  rmSync(file);
}

function cachedInvocation(directory: string): void {
  const vize = path.join(directory, "vize");
  const cache = path.join(vize, "config-runtime-v1");
  mkdirSync(cache, { recursive: true, mode: 0o700 });
  chmodSync(vize, 0o700);
  chmodSync(cache, 0o700);
  readdirSync(cache).filter((name) => name.endsWith(".tmp"));
  const hash = createHash("sha256").update(payload).digest("hex");
  const runtime = path.join(cache, `${hash}.mjs`);
  const lease = path.join(cache, `${hash}.lease`);
  try {
    const descriptor = openSync(lease, "wx", 0o600);
    closeSync(descriptor);
    createdFiles += 1;
  } catch (error) {
    if (!(error instanceof Error && "code" in error && error.code === "EEXIST")) throw error;
  }
  try {
    const descriptor = openSync(runtime, "wx", 0o600);
    try {
      writeFileSync(descriptor, payload);
      writtenBytes += payload.byteLength;
    } finally {
      closeSync(descriptor);
    }
    createdFiles += 1;
  } catch (error) {
    if (!(error instanceof Error && "code" in error && error.code === "EEXIST")) throw error;
  }
  const actual = readFileSync(runtime);
  if (!actual.equals(payload)) throw new Error("cached runtime bytes changed");
  readdirSync(cache).filter((name) => /^[a-f0-9]{64}\.mjs$/u.test(name));
}

try {
  const invocation = lane === "base" ? baselineInvocation : cachedInvocation;
  for (let sample = 0; sample < samples; sample += 1) {
    const directory = path.join(owner, `sample-${sample}`);
    mkdirSync(directory, { mode: 0o700 });
    for (let iteration = 0; iteration < iterations; iteration += 1) {
      const started = process.hrtime.bigint();
      invocation(directory);
      timings.push(Number(process.hrtime.bigint() - started) / 1e6);
    }
  }
  timings.sort((left, right) => left - right);
  const retained = readdirSync(owner, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => path.join(entry.parentPath, entry.name));
  const retainedBytes = retained.reduce((total, file) => total + statSync(file).size, 0);
  const percentile = (value: number) => timings[Math.ceil(timings.length * value) - 1];
  process.stdout.write(
    `${JSON.stringify({
      lane,
      samples,
      iterations,
      invocations: timings.length,
      payloadBytes: payload.byteLength,
      wallMs: {
        p50: percentile(0.5),
        p95: percentile(0.95),
        total: timings.reduce((total, value) => total + value, 0),
      },
      storage: {
        createdFiles,
        writtenBytes,
        retainedFiles: retained.length,
        retainedBytes,
        perIsolatedSample: {
          createdFiles: createdFiles / samples,
          writtenBytes: writtenBytes / samples,
          retainedFiles: retained.length / samples,
          retainedBytes: retainedBytes / samples,
        },
      },
    })}\n`,
  );
} finally {
  rmSync(owner, { recursive: true, force: true });
}
