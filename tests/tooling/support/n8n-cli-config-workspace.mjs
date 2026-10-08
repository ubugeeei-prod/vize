import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { root, projection, scopedConfig } from "./n8n-cli-config-inputs.mjs";

export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
export function writeJson(destination, value) {
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.writeFileSync(destination, JSON.stringify(value, null, 2) + "\n");
}

// This is an owned projection package. It is deliberately not the unpublished
// upstream @n8n package, whose branch source is read-only requirement evidence.
export function installConfigPackages(workspace) {
  fs.mkdirSync(path.join(workspace, ".git"), { recursive: true });
  fs.writeFileSync(path.join(workspace, "package.json"), '{"private":true,"type":"module"}\n');
  const source = path.join(root, "npm/cli");
  const destination = path.join(workspace, "node_modules/vize");
  fs.mkdirSync(destination, { recursive: true });
  fs.copyFileSync(path.join(source, "package.json"), path.join(destination, "package.json"));
  assert.ok(
    fs.statSync(path.join(source, "dist/config.mjs")).isFile(),
    "freshly built public vize/config is required",
  );
  fs.cpSync(path.join(source, "dist"), path.join(destination, "dist"), { recursive: true });
  const settings = path.join(workspace, "node_modules/@vize-acceptance/n8n-cli-settings");
  fs.mkdirSync(settings, { recursive: true });
  fs.writeFileSync(
    path.join(settings, "package.json"),
    JSON.stringify({
      name: "@vize-acceptance/n8n-cli-settings",
      private: true,
      type: "module",
      exports: { ".": "./index.js" },
    }) + "\n",
  );
  fs.writeFileSync(
    path.join(settings, "index.js"),
    "export const settings = " + JSON.stringify({ linter: projection.linter }, null, 2) + ";\n",
  );
  return fs
    .globSync("**/*", { cwd: destination })
    .filter((file) => fs.statSync(path.join(destination, file)).isFile())
    .map((file) => ({
      file: "node_modules/vize/" + file,
      sha256: sha256(fs.readFileSync(path.join(destination, file))),
    }));
}

export function writeConfig(workspace, packageRoot, mode, ruleOptions) {
  const config = scopedConfig(packageRoot, mode === "scoped");
  if (ruleOptions) Object.assign(config.linter.ruleOptions, ruleOptions);
  const settingsOverrides = {
    ...config.linter,
    rules: Object.fromEntries(
      Object.entries(config.linter.rules).filter(
        ([name, severity]) => projection.linter.rules[name] !== severity,
      ),
    ),
  };
  const source = [
    'import { defineConfig } from "vize/config";',
    'import { settings } from "@vize-acceptance/n8n-cli-settings";',
    "export default defineConfig({",
    "  linter: { ...settings.linter,",
    "    rules: { ...settings.linter.rules, ..." + JSON.stringify(settingsOverrides.rules) + " },",
    "    ruleOptions: " + JSON.stringify(config.linter.ruleOptions) + ",",
    "  },",
    ...(config.entries ? ["  entries: " + JSON.stringify(config.entries) + ","] : []),
    "});",
    "",
  ].join("\n");
  const destination = path.join(workspace, packageRoot, `vize.${mode}.config.ts`);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.writeFileSync(destination, source);
  return { path: destination, sha256: sha256(source), source, config };
}

export function copyOriginals(workspace, corpus) {
  for (const { file } of [...corpus.files, ...corpus.licenses]) {
    const destination = path.join(workspace, file);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.copyFileSync(path.join(corpus.fixture, file), destination);
  }
}

export function assertUnchanged(workspace, entries) {
  for (const { file, sha256: expected } of entries) {
    assert.equal(sha256(fs.readFileSync(path.join(workspace, file))), expected, file);
  }
}

/** Named recording step: retain the whole JSON packet, canonicalize array order only. */
export function recordCliPackets(packets) {
  assert.ok(Array.isArray(packets), "CLI must produce its complete JSON file array");
  return structuredClone(packets).sort((a, b) =>
    Buffer.compare(Buffer.from(a.file), Buffer.from(b.file)),
  );
}

export function runCli({
  binary,
  workspace,
  packageRoot,
  files,
  config,
  receiptPath,
  outsideCwd = false,
}) {
  const cwd = outsideCwd ? workspace : path.join(workspace, packageRoot);
  const patterns = outsideCwd ? files.map((file) => path.join(packageRoot, file)) : files;
  const args = [
    "lint",
    "--config",
    config.path,
    "--format",
    "json",
    "--locale",
    "en",
    "--help-level",
    "none",
    ...patterns,
  ];
  const started = performance.now();
  const result = spawnSync(binary, args, {
    cwd,
    encoding: "utf8",
    timeout: 120_000,
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, NO_COLOR: "1" },
  });
  const receipt = {
    binary,
    args,
    cwd,
    config,
    elapsedMs: performance.now() - started,
    status: result.status,
    signal: result.signal,
    ...(result.error
      ? {
          error: {
            name: result.error.name,
            message: result.error.message,
            code: result.error.code,
            stack: result.error.stack,
          },
        }
      : {}),
    stdout: result.stdout,
    stderr: result.stderr,
  };
  // Keep raw stdout/stderr and failure metadata before parsing or asserting.
  writeJson(receiptPath, receipt);
  assert.equal(result.error, undefined, receiptPath);
  assert.equal(result.signal, null, receiptPath);
  assert.ok(result.status === 0 || result.status === 1, receiptPath);
  const rawPackets = JSON.parse(result.stdout);
  const packets = recordCliPackets(rawPackets);
  assert.deepEqual(
    packets.map(({ file }) => file),
    [...patterns].sort((a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b))),
    "every explicit input, including clean/scriptless files, must remain in the report",
  );
  const errors = packets.reduce((sum, packet) => sum + packet.errorCount, 0);
  assert.equal(result.status, errors ? 1 : 0, "CLI exit status agrees with complete error counts");
  writeJson(receiptPath.replace(/\.json$/u, ".recorded.json"), packets);
  return packets;
}
