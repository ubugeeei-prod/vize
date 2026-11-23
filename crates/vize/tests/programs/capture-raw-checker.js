#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
const image = fs.realpathSync(process.env.VIZE_RAW_NATIVE);
const directory = fs.mkdtempSync(path.join(process.env.VIZE_RAW_CAPTURE, "call-"));
const argv = process.argv.slice(2),
  cwd = fs.realpathSync(process.cwd());
const hash = (b) => createHash("sha256").update(b).digest("hex");
const write = (p, v) =>
  fs.writeFileSync(path.join(directory, p), JSON.stringify(v, null, 2) + "\n", { flag: "wx" });
const i = argv.indexOf("--project"),
  config = i >= 0 ? path.resolve(cwd, argv[i + 1]) : null;
const kind = argv.includes("--checkers")
  ? "main-cli"
  : config
    ? "option-probe"
    : argv.length === 1 && argv[0] === "--version"
      ? "version"
      : "UNKNOWN";
write("request.json", {
  argv,
  cwd,
  config,
  kind,
  executable: image,
  imageBefore: hash(fs.readFileSync(image)),
  node: { version: process.version, executable: fs.realpathSync(process.execPath) },
  scope:
    "test-only passive CLI forwarding; input closure and native loaded-file membership UNOBSERVED",
});
if (config) {
  try {
    fs.copyFileSync(config, path.join(directory, "config.bytes"));
  } catch (e) {
    write("config-read-error.json", { error: String(e) });
  }
}
const env = { ...process.env };
delete env.VIZE_RAW_NATIVE;
delete env.VIZE_RAW_CAPTURE;
const r = spawnSync(image, argv, { cwd, env, timeout: 120000, maxBuffer: 64 * 1024 * 1024 });
fs.writeFileSync(path.join(directory, "native-replay.stdout.bin"), r.stdout ?? Buffer.alloc(0), {
  flag: "wx",
});
fs.writeFileSync(path.join(directory, "native-replay.stderr.bin"), r.stderr ?? Buffer.alloc(0), {
  flag: "wx",
});
write("native-replay.status.json", {
  status: r.status,
  signal: r.signal,
  error: r.error?.message ?? null,
  pid: r.pid,
});
try {
  write("image-after.json", { sha256: hash(fs.readFileSync(image)) });
} catch (e) {
  write("image-read-error.json", { error: String(e) });
}
if (r.stdout?.length) process.stdout.write(r.stdout);
if (r.stderr?.length) process.stderr.write(r.stderr);
if (r.signal) process.kill(process.pid, r.signal);
else process.exitCode = r.status ?? 1;
