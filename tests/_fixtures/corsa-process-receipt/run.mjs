import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, realpathSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const fixture = fileURLToPath(new URL(".", import.meta.url));
const native = realpathSync(process.env.VIZE_CORSA_PATH);
const output = process.env.VIZE_RECEIPT_OUTPUT ?? join(fixture, "artifacts", String(Date.now()));
mkdirSync(output, { recursive: true });
const version = spawnSync(native, ["--version"], { encoding: "utf8" });
if (version.status !== 0 || version.stdout.trim() !== "Version 7.0.2") {
  throw new Error(`Expected native TypeScript 7.0.2: ${JSON.stringify(version)}`);
}
const identity = {
  canonicalPath: native,
  bytes: statSync(native).size,
  sha256: createHash("sha256").update(readFileSync(native)).digest("hex"),
  version: version.stdout.trim(),
  versionStdout: version.stdout,
  versionStderr: version.stderr,
  versionStatus: version.status,
};
const identityPath = join(output, "native-identity.json");
writeFileSync(identityPath, `${JSON.stringify(identity, null, 2)}\n`);
const manifest = join(fixture, "Cargo.toml");
const runs = [];
for (const binary of ["process-receipt-prototype", "native_receipt"]) {
  const args = ["run", "--offline", "--locked", "--manifest-path", manifest, "--bin", binary];
  const started = process.hrtime.bigint();
  const run = spawnSync(process.env.CARGO ?? "cargo", args, {
    cwd: output,
    env: { ...process.env, VIZE_CORSA_PATH: native, VIZE_CORSA_IDENTITY_FILE: identityPath },
    timeout: 60_000,
    maxBuffer: 8 * 1024 * 1024,
  });
  writeFileSync(join(output, `${binary}.stdout.bin`), run.stdout ?? Buffer.alloc(0));
  writeFileSync(join(output, `${binary}.stderr.bin`), run.stderr ?? Buffer.alloc(0));
  runs.push({
    binary,
    args,
    status: run.status,
    signal: run.signal,
    error: run.error?.message ?? null,
    elapsedNs: String(process.hrtime.bigint() - started),
  });
  writeFileSync(join(output, "runs.json"), `${JSON.stringify(runs, null, 2)}\n`);
  process.stdout.write(run.stdout ?? Buffer.alloc(0));
  if (run.status !== 0 || run.error)
    throw new Error(`Receipt law failed: ${binary}; retained ${output}`);
}
console.log(`Retained complete receipt evidence: ${output}`);
