import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { readFile, realpath } from "node:fs/promises";
import path from "node:path";
import { promisify } from "node:util";
import { rejectOverrides } from "./installed.ts";
import type { VrtFixture } from "./vrt_fixtures.ts";
import { retainReport, type Evidence } from "./vrt_artifacts.ts";

const execute = promisify(execFile);
export async function installedVrtBin() {
  const consumer = await realpath(process.cwd());
  const directory = await realpath(path.join(consumer, "node_modules/@vizejs/vite-plugin-musea"));
  assert.ok(directory.startsWith(path.join(consumer, "node_modules") + path.sep));
  const manifest = JSON.parse(await readFile(path.join(directory, "package.json"), "utf8"));
  assert.equal(manifest.name, "@vizejs/vite-plugin-musea");
  assert.equal(typeof manifest.bin?.["musea-vrt"], "string", "published Musea CLI is mandatory");
  const bin = await realpath(path.resolve(directory, manifest.bin["musea-vrt"]));
  assert.ok(bin.startsWith(directory + path.sep));
  return bin;
}

export async function runVrtCli(
  f: VrtFixture,
  e: Evidence,
  bin: string,
  phase: string,
  args: string[],
  expected = 0,
) {
  rejectOverrides();
  let stdout: Buffer = Buffer.alloc(0),
    stderr: Buffer = Buffer.alloc(0),
    code: number | string | null = 0,
    signal: string | null = null;
  try {
    const result = await execute(process.execPath, [bin, ...args], {
      cwd: f.root,
      timeout: 120_000,
      maxBuffer: 8 * 1024 * 1024,
      encoding: "buffer",
    });
    stdout = result.stdout;
    stderr = result.stderr;
  } catch (error) {
    const failed = error as {
      stdout?: Buffer;
      stderr?: Buffer;
      code?: number | string;
      signal?: string;
    };
    stdout = failed.stdout ?? stdout;
    stderr = failed.stderr ?? stderr;
    code = failed.code ?? null;
    signal = failed.signal ?? null;
  }
  await e.save(`${phase}/stdout.log`, stdout);
  await e.save(`${phase}/stderr.log`, stderr);
  const receipt = { phase, command: process.execPath, bin, args, cwd: f.root, code, signal };
  await e.json(`${phase}/process.json`, receipt);
  e.records.push(receipt);
  assert.equal(signal, null, JSON.stringify(receipt));
  assert.equal(code, expected, stdout.toString() + stderr.toString());
}

export async function captureHostedCli(
  f: VrtFixture,
  e: Evidence,
  bin: string,
  url: string,
  phase: string,
  update = false,
  expected = 0,
) {
  const output = path.join(f.root, "cli-output");
  const args = [
    "--gallery-url",
    url,
    "--config",
    f.config,
    "--output",
    output,
    "--json",
    "--threshold",
    "0",
    "--workers",
    "2",
  ];
  if (update) args.push("--update");
  else args.push("--ci");
  await runVrtCli(f, e, bin, phase, args, expected);
  return retainReport(e, phase, path.join(output, "vrt-report.json"));
}
