import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import {
  appendFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { TextDecoder } from "node:util";
import { fileURLToPath } from "node:url";

export const effectiveDenoConfig = '{\n  "nodeModulesDir": "none"\n}\n';
export const bytesSha256 = (bytes: Uint8Array | string) =>
  createHash("sha256").update(bytes).digest("hex");

/** Only runtime necessities; callers add fresh public registry/cache settings. */
export function publicEnvironment(parent: NodeJS.ProcessEnv): NodeJS.ProcessEnv {
  const result: NodeJS.ProcessEnv = {};
  for (const key of [
    "PATH",
    "Path",
    "PATHEXT",
    "COMSPEC",
    "SystemRoot",
    "WINDIR",
    "TMPDIR",
    "TMP",
    "TEMP",
    "LANG",
    "LC_ALL",
    "HOME",
    "USERPROFILE",
    "APPDATA",
    "LOCALAPPDATA",
  ]) {
    if (parent[key] !== undefined) result[key] = parent[key];
  }
  return result;
}

/** Retain every received byte before interpreting process status or text. No retry/output cap. */
export async function captureCommand(
  executable: string,
  args: string[],
  directory: string,
  options: { cwd?: string; env?: NodeJS.ProcessEnv; timeoutMs?: number } = {},
) {
  mkdirSync(directory, { recursive: false });
  const stdout: Buffer[] = [],
    stderr: Buffer[] = [];
  for (const name of ["stdout.bin", "stderr.bin"])
    writeFileSync(path.join(directory, name), Buffer.alloc(0), { flag: "wx" });
  const child = spawn(executable, args, {
    cwd: options.cwd,
    env: options.env ?? process.env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stdout.on("data", (bytes: Buffer) => {
    appendFileSync(path.join(directory, "stdout.bin"), bytes);
    stdout.push(bytes);
  });
  child.stderr.on("data", (bytes: Buffer) => {
    appendFileSync(path.join(directory, "stderr.bin"), bytes);
    stderr.push(bytes);
  });
  const timeoutMs = options.timeoutMs ?? 120_000;
  let timedOut = false;
  const timer = setTimeout(() => {
    timedOut = true;
    child.kill("SIGKILL");
  }, timeoutMs);
  const outcome = await new Promise<{
    status: number | null;
    signal: string | null;
    error?: string;
  }>((done) => {
    let error: string | undefined;
    child.on("error", (caught) => {
      error = caught.message;
    });
    child.on("close", (status, signal) => {
      clearTimeout(timer);
      done({
        status,
        signal,
        ...(error
          ? { error }
          : timedOut
            ? { error: `Command timed out after ${timeoutMs}ms` }
            : {}),
      });
    });
  });
  const out = Buffer.concat(stdout),
    err = Buffer.concat(stderr);
  const custody = {
    command: executable,
    args,
    cwd: options.cwd ?? process.cwd(),
    ...outcome,
    stdout: { bytes: out.length, sha256: bytesSha256(out) },
    stderr: { bytes: err.length, sha256: bytesSha256(err) },
  };
  writeFileSync(path.join(directory, "process.json"), JSON.stringify(custody, null, 2) + "\n", {
    flag: "wx",
  });
  return { ...outcome, stdout: out, stderr: err, custody };
}
export function strictUtf8(bytes: Buffer) {
  const text = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(bytes);
  assert.deepEqual(Buffer.from(text), bytes, "text decoding must preserve all original bytes");
  return text;
}

/** Pull-request proof only: no admission, signing, operator or GitHub mutation. */
async function probe(output: string) {
  mkdirSync(output, { recursive: false });
  const head = "03882089eff75664abe99ba71868e33739496b01";
  const environment = publicEnvironment(process.env);
  const run = async (
    label: string,
    executable: string,
    args: string[],
    cwd?: string,
    env = environment,
  ) => {
    const result = await captureCommand(executable, args, path.join(output, label), { cwd, env });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0);
    return result.stdout;
  };
  await run("fetch", "git", ["fetch", "--no-tags", "origin", head]);
  const source = await run("source", "git", [
    "--no-replace-objects",
    "show",
    `${head}:tools/support/release/jsr/consumer.ts`,
  ]);
  assert.equal(
    bytesSha256(source),
    "410be5021e5c1bff706191bf68045ddbc2edbaa47b13325ddde3daa22926851c",
  );
  const project = mkdtempSync(path.join(os.tmpdir(), "vize-recovery-host-probe-"));
  const cache = path.join(project, "deno-cache");
  mkdirSync(cache);
  writeFileSync(path.join(project, "consumer.ts"), source);
  writeFileSync(path.join(project, "deno.json"), effectiveDenoConfig);
  writeFileSync(
    path.join(project, "package.json"),
    '{ "name": "vize-jsr-consumer", "private": true, "type": "module" }\n',
  );
  const runtime = strictUtf8(await run("runtime", "deno", ["--version"]));
  assert.match(runtime, /^deno 2\.9\.7 /);
  const stdout = await run(
    "consumer",
    "deno",
    ["run", "--config", "deno.json", "--allow-all", "consumer.ts", "jsr:@vizejs/vize@0.457.0"],
    project,
    { ...environment, DENO_DIR: cache },
  );
  assert.equal(strictUtf8(stdout), "Verified jsr:@vizejs/vize@0.457.0 (no flags)\n");
  writeFileSync(
    path.join(output, "probe.json"),
    JSON.stringify(
      {
        head,
        consumerSha256: bytesSha256(source),
        effectiveDenoConfig,
        effectiveDenoConfigSha256: bytesSha256(effectiveDenoConfig),
        runtime,
        stdout: strictUtf8(stdout),
        scope: "Read-only consumer evidence, no recovery authority",
        success: true,
      },
      null,
      2,
    ) + "\n",
  );
  if (existsSync(path.join(project, "deno.lock")))
    writeFileSync(path.join(output, "deno.lock"), readFileSync(path.join(project, "deno.lock")));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv[2], "probe");
  assert.ok(process.argv[3]);
  await probe(process.argv[3]);
}
