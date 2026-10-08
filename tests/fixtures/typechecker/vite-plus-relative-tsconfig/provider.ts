import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import {
  expectedBuildIdentity,
  validateBuildReceipt,
} from "../../../differential/build-receipt.ts";
import { repoRoot, sha256 } from "./fixture.ts";

let prepared: ReturnType<typeof captureProvider> | undefined;

function captureProvider() {
  const identity = expectedBuildIdentity(repoRoot);
  validateBuildReceipt(
    JSON.parse(
      fs.readFileSync(
        path.join(repoRoot, `${identity.binaryPath}.differential-build.json`),
        "utf8",
      ),
    ),
    identity,
  );
  // Resolve through the same dependency context as the genuine plugin loader.
  const require = createRequire(path.join(repoRoot, "npm/builder/vite/src/config.ts"));
  const vpPackage = require.resolve("vite-plus/package.json");
  const vp = path.join(
    path.dirname(vpPackage),
    JSON.parse(fs.readFileSync(vpPackage, "utf8")).bin.vp,
  );
  const argv = [vp, "pack"];
  const cwd = path.join(repoRoot, "npm/cli");
  const result = spawnSync(process.execPath, argv, {
    cwd,
    env: process.env,
    timeout: 120_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const processRow = {
    command: process.execPath,
    argv,
    cwd,
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdout: result.stdout?.toString("utf8") ?? "",
    stderr: result.stderr?.toString("utf8") ?? "",
    stdoutBytes: Array.from(result.stdout ?? []),
    stderrBytes: Array.from(result.stderr ?? []),
  };
  const output = path.join(
    repoRoot,
    "target/differential/typechecker/vite-plus-relative-tsconfig/provider.json",
  );
  fs.mkdirSync(path.dirname(output), { recursive: true });
  fs.writeFileSync(
    output,
    `${JSON.stringify({ sourceBuild: identity, process: processRow }, null, 2)}\n`,
  );
  assert.equal(processRow.error, null, processRow.error ?? "");
  assert.equal(processRow.signal, null);
  assert.equal(processRow.status, 0, processRow.stderr || processRow.stdout);
  const configPath = fs.realpathSync(require.resolve("vize/config"));
  assert.equal(configPath, fs.realpathSync(path.join(cwd, "dist/config.mjs")));
  const nativeRoot = path.join(repoRoot, "npm/native");
  const bindings = fs
    .readdirSync(nativeRoot)
    .filter((name) => /^vize-vitrine\..+\.node$/.test(name));
  assert.equal(
    bindings.length,
    1,
    "the existing tooling preparation must supply one local source addon",
  );
  const binding = fs.realpathSync(path.join(nativeRoot, bindings[0]));
  const local = require(binding) as { normalizeVizeConfig: unknown };
  const selected = require("@vizejs/native") as { normalizeVizeConfig: unknown };
  assert.equal(typeof local.normalizeVizeConfig, "function");
  assert.equal(
    selected.normalizeVizeConfig,
    local.normalizeVizeConfig,
    "config normalization must use the local source addon",
  );
  const provider = {
    config: { path: configPath, sha256: sha256(fs.readFileSync(configPath)) },
    nativeAddon: { path: binding, sha256: sha256(fs.readFileSync(binding)) },
    process: processRow,
  };
  fs.writeFileSync(output, `${JSON.stringify({ sourceBuild: identity, ...provider }, null, 2)}\n`);
  return provider;
}

export function sourceConfigProvider() {
  prepared ??= captureProvider();
  return prepared;
}
