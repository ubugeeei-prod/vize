#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
const root = process.env.VIZE_DIRECTIVE_NATIVE_CAPTURE;
fs.mkdirSync(root, { recursive: true });
const out = fs.mkdtempSync(path.join(root, "call-")),
  argv = process.argv.slice(2);
fs.writeFileSync(
  path.join(out, "request.json"),
  JSON.stringify({ executable: process.env.VIZE_DIRECTIVE_NATIVE, argv, cwd: process.cwd() }),
);
const pi = argv.indexOf("--project");
if (pi >= 0) {
  const cfg = path.resolve(process.cwd(), argv[pi + 1]);
  fs.copyFileSync(cfg, path.join(out, "tsconfig.json"));
}
const r = spawnSync(process.env.VIZE_DIRECTIVE_NATIVE, argv, {
  env: process.env,
  cwd: process.cwd(),
  maxBuffer: 64 * 1024 * 1024,
  timeout: 85000,
});
fs.writeFileSync(path.join(out, "stdout.bin"), r.stdout ?? Buffer.alloc(0));
fs.writeFileSync(path.join(out, "stderr.bin"), r.stderr ?? Buffer.alloc(0));
fs.writeFileSync(
  path.join(out, "response.json"),
  JSON.stringify({ status: r.status, signal: r.signal, error: r.error?.message ?? null }),
);
if (r.stdout?.length) await new Promise((resolve) => process.stdout.write(r.stdout, resolve));
if (r.stderr?.length) await new Promise((resolve) => process.stderr.write(r.stderr, resolve));
if (r.signal) process.kill(process.pid, r.signal);
else process.exitCode = r.status ?? 1;
