import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { arg } from "./utils.ts";

const workspace = arg(0);
const version = arg(1);
const out = arg(2);
assert.match(version, /^0\.\d+\.0$/);
fs.mkdirSync(workspace, { recursive: true });
fs.mkdirSync(out, { recursive: true });
assert.equal(fs.existsSync(path.join(workspace, "node_modules")), false, "consumer must be fresh");
const manifest = {
  name: "vize-public-release-acceptance",
  private: true,
  type: "module",
  dependencies: {
    vize: version,
    "@vizejs/native": version,
    "@vizejs/wasm": version,
    vue: "3.6.0-rc.13",
  },
};
fs.writeFileSync(path.join(workspace, "package.json"), JSON.stringify(manifest, null, 2));
const config = path.join(workspace, ".npmrc");
fs.writeFileSync(config, "registry=https://registry.npmjs.org/\n");
const args = [
  "install",
  "--ignore-scripts",
  "--legacy-peer-deps",
  "--no-audit",
  "--no-fund",
  "--prefer-online",
  "--userconfig",
  config,
  "--cache",
  path.join(workspace, ".npm-cache"),
];
const result = spawnSync("npm", args, {
  cwd: workspace,
  encoding: "utf8",
  timeout: 480000,
  maxBuffer: 8 * 1024 * 1024,
});
fs.writeFileSync(
  path.join(out, "install-packet.json"),
  JSON.stringify(
    {
      manifest,
      command: ["npm", ...args],
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      stdout: result.stdout,
      stderr: result.stderr,
    },
    null,
    2,
  ),
);
process.stdout.write(result.stdout ?? "");
process.stderr.write(result.stderr ?? "");
assert.equal(result.status, 0);
assert.equal(result.signal, null);
