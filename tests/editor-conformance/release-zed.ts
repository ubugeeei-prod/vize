// Launch the packaged Zed extension in the actual pinned Linux editor against
// a project that the packaged `vize init` prepared. The extension must resolve
// `vize` from PATH, start the exact candidate through the transparent tap, and
// receive the lint diagnostic authored in src/Lint.vue.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

import { readTranscript } from "./support/transcript.ts";

const timeoutMs = 180_000;

async function main() {
  const workspace = requiredDirectory("VIZE_RELEASE_ZED_WORKSPACE");
  const server = requiredFile("VIZE_SERVER_PATH");
  const zed = requiredFile("VIZE_TEST_ZED_PATH");
  const packaged = requiredDirectory("VIZE_RELEASE_ZED_EXTENSION");
  const evidence = requiredPath("VIZE_RELEASE_ZED_EVIDENCE");
  fs.mkdirSync(evidence, { recursive: true });

  const session = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-zed-"));
  const userData = path.join(session, "user-data");
  const config = path.join(session, "config");
  const shim = path.join(session, "bin");
  const installed = path.join(userData, "extensions", "installed", "vize");
  fs.mkdirSync(installed, { recursive: true });
  fs.mkdirSync(path.join(config, "zed"), { recursive: true });
  fs.mkdirSync(shim, { recursive: true });
  fs.cpSync(packaged, installed, { recursive: true });
  fs.writeFileSync(path.join(userData, "extensions", "installed", ".rescan"), "");
  fs.writeFileSync(
    path.join(config, "zed", "settings.json"),
    `${JSON.stringify({
      auto_install_extensions: { vize: false },
      auto_update_extensions: { vize: false },
      telemetry: { diagnostics: false, metrics: false },
    })}\n`,
  );

  const transcript = path.join(evidence, "zed.transcript.jsonl");
  fs.rmSync(transcript, { force: true });
  const tap = path.resolve(import.meta.dirname, "lsp-tap.mjs");
  fs.writeFileSync(
    path.join(shim, "vize"),
    `#!/bin/sh\nexec ${quote(process.execPath)} ${quote(tap)} ${quote(server)} "$@"\n`,
    { mode: 0o755 },
  );

  const version = spawnSync(zed, ["--version"], { encoding: "utf8" });
  assert.equal(version.status, 0, version.stderr);
  fs.writeFileSync(path.join(evidence, "zed-version.txt"), version.stdout);
  fs.writeFileSync(
    path.join(evidence, "identity.json"),
    `${JSON.stringify(
      {
        client: version.stdout.trim(),
        extension: fs.realpathSync(installed),
        server: fs.realpathSync(server),
        workspace: fs.realpathSync(workspace),
      },
      null,
      2,
    )}\n`,
  );

  const output = fs.openSync(path.join(evidence, "zed.log"), "w");
  const child = spawn(
    zed,
    ["--foreground", "--user-data-dir", userData, path.join(workspace, "src", "Lint.vue")],
    {
      cwd: workspace,
      env: {
        ...process.env,
        PATH: `${shim}${path.delimiter}${process.env.PATH ?? ""}`,
        VIZE_CONFORMANCE_TRANSCRIPT: transcript,
        XDG_CONFIG_HOME: config,
      },
      stdio: ["ignore", output, output],
    },
  );

  try {
    await waitForDiagnostic(transcript, workspace, child);
  } finally {
    child.kill("SIGTERM");
    await Promise.race([
      new Promise((resolve) => child.once("close", resolve)),
      sleep(10_000).then(() => child.kill("SIGKILL")),
    ]);
    fs.closeSync(output);
    fs.rmSync(session, { force: true, recursive: true });
  }
}

async function waitForDiagnostic(
  transcript: string,
  workspace: string,
  child: ReturnType<typeof spawn>,
) {
  const deadline = Date.now() + timeoutMs;
  const document = fs.realpathSync(path.join(workspace, "src", "Lint.vue"));
  while (Date.now() < deadline) {
    if (child.exitCode !== null)
      throw new Error(`Zed exited before diagnostics: ${child.exitCode}`);
    const entries = readTranscript(transcript);
    const found = entries.some(
      (entry) =>
        entry.dir === "s2c" &&
        entry.msg?.method === "textDocument/publishDiagnostics" &&
        String(entry.msg.params?.uri ?? "").endsWith("/src/Lint.vue") &&
        Array.isArray(entry.msg.params?.diagnostics) &&
        entry.msg.params.diagnostics.some(
          (diagnostic: { code?: unknown }) => diagnostic.code === "vue/require-v-for-key",
        ),
    );
    if (found) {
      assert.ok(document.startsWith(fs.realpathSync(workspace) + path.sep));
      return;
    }
    await sleep(200);
  }
  throw new Error("Zed did not receive the release-project diagnostic before the deadline");
}

function requiredPath(name: string): string {
  const value = process.env[name];
  assert.ok(value, `${name} must be set`);
  return path.resolve(value);
}

function requiredFile(name: string): string {
  const value = requiredPath(name);
  assert.ok(fs.statSync(value).isFile(), `${name} must name a file`);
  return value;
}

function requiredDirectory(name: string): string {
  const value = requiredPath(name);
  assert.ok(fs.statSync(value).isDirectory(), `${name} must name a directory`);
  return value;
}

function quote(value: string): string {
  return `'${value.replaceAll("'", `'\\''`)}'`;
}

await main();
