import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

export type Packet = {
  label: string;
  argv: readonly string[];
  cwd: string;
  status: number | null;
  signal: string | null;
  error: string | null;
  stdout: string;
  stderr: string;
  durationMs: number;
};
export const sha256 = (bytes: string | Buffer): string =>
  createHash("sha256").update(bytes).digest("hex");
export function cleanEnv(): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    NPM_CONFIG_REGISTRY: "https://registry.npmjs.org/",
  };
  for (const key of [
    "NODE_PATH",
    "NAPI_RS_NATIVE_LIBRARY_PATH",
    "NAPI_RS_FORCE_WASI",
    "VIZE_NATIVE_PATH",
  ])
    delete env[key];
  return env;
}
export function npmArgv(...args: string[]): string[] {
  const home = dirname(process.execPath);
  const cli = [
    join(home, "node_modules/npm/bin/npm-cli.js"),
    resolve(home, "../lib/node_modules/npm/bin/npm-cli.js"),
  ].find(existsSync);
  assert.ok(cli, "actual npm shipped with Node must exist");
  return [process.execPath, cli, ...args];
}
export class Recorder {
  readonly root: string;
  readonly packets: Packet[] = [];
  constructor(root: string) {
    this.root = resolve(root);
    mkdirSync(this.root, { recursive: true });
  }
  json(name: string, value: unknown): void {
    writeFileSync(join(this.root, name), JSON.stringify(value, null, 2) + "\n");
  }
  run(
    label: string,
    argv: readonly string[],
    cwd: string,
    env = cleanEnv(),
    timeout = 240_000,
  ): Packet {
    assert.ok(argv.length > 0);
    const start = performance.now();
    const result = spawnSync(argv[0], argv.slice(1), {
      cwd,
      env,
      encoding: "utf8",
      timeout,
      maxBuffer: 64 * 1024 * 1024,
      shell: false,
    });
    const packet: Packet = {
      label,
      argv,
      cwd,
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: result.stdout ?? "",
      stderr: result.stderr ?? "",
      durationMs: performance.now() - start,
    };
    this.packets.push(packet);
    writeFileSync(join(this.root, label + ".stdout"), packet.stdout);
    writeFileSync(join(this.root, label + ".stderr"), packet.stderr);
    this.json(label + ".packet.json", packet);
    this.json("commands.json", this.packets);
    process.stdout.write(packet.stdout);
    process.stderr.write(packet.stderr);
    return packet;
  }
}
export function success(packet: Packet): void {
  assert.equal(packet.error, null, packet.label);
  assert.equal(packet.signal, null, packet.label);
  assert.equal(packet.status, 0, packet.label + "\n" + packet.stdout + packet.stderr);
}
