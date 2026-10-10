import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { writeBuildReceipt } from "../differential/build-receipt.ts";
import { sha256 } from "../differential/harness.mjs";
import { frameMessage } from "../differential/lsp-wire.ts";

// Explicitly synthetic transport/termination law, never product acceptance.
// Execute the real driver's completion suffix after the existing child exit.
test("early provider hover failure drains existing close handlers and retains whole wire", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-provider-failure-close-"));
  try {
    const git = (...args: string[]) => execFileSync("git", args, { cwd: root });
    git("init", "-q");
    git("config", "user.name", "CI Test");
    git("config", "user.email", "ci@example.invalid");
    fs.writeFileSync(path.join(root, "Cargo.toml"), '[workspace.package]\nversion = "1.0.0"\n');
    git("add", ".");
    git("commit", "-qm", "synthetic close observer source");
    const binary = path.join(root, "target/ci/vize");
    fs.mkdirSync(path.dirname(binary), { recursive: true });
    fs.writeFileSync(
      binary,
      // A short-lived descendant retains the original inherited pipes after
      // child exit. This makes exit-before-close genuine and deterministic.
      '#!/bin/sh\nif [ "$1" = "--version" ]; then printf "vize 1.0.0\\n"; exit 0; fi\ncat\n(sleep 0.1; printf "observer stderr 🧪\\n" >&2) &\nexit 23\n',
      { mode: 0o755 },
    );
    writeBuildReceipt(root);
    const driver = fs.readFileSync(
      new URL("./support/n8n-authored-registry-acceptance.ts", import.meta.url),
      "utf8",
    );
    const start = driver.lastIndexOf("\nif (failure)");
    assert.ok(start >= 0, "the original driver completion must remain explicit");
    const suffix = driver.slice(start);
    const request = frameMessage({ jsonrpc: "2.0", id: 1, method: "textDocument/hover" });
    const reply = frameMessage({
      jsonrpc: "2.0",
      id: 1,
      result: { contents: "const props: __DefineProps<Props, never>" },
    });
    const frames = Buffer.concat([request, reply]);
    const processModule = new URL("./support/lsp/session-process.ts", import.meta.url).href;
    const runner = `
import { once } from "node:events";
import { spawnLspSessionProcess, recordLspClientWire } from ${JSON.stringify(processModule)};
const child = spawnLspSessionProcess(${JSON.stringify(root)}, true, ${JSON.stringify(binary)});
const exited = once(child, "exit");
const frames = Buffer.from(${JSON.stringify(frames.toString("base64"))}, "base64");
recordLspClientWire(child, frames.toString("utf8"));
child.stdin.end(frames);
await exited;
const failure = new Error("hover for props is missing value: string");
${suffix}
`;
    const runnerPath = path.join(root, "failure-runner.ts");
    fs.writeFileSync(runnerPath, runner);
    const result = spawnSync(process.execPath, [runnerPath], {
      encoding: "utf8",
      timeout: 10_000,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 1);
    assert.equal(result.signal, null);
    assert.equal(result.stdout, "");
    assert.match(result.stderr, /Error: hover for props is missing value: string/);
    const captures = path.join(root, "target/differential/lsp-sessions");
    const entries = fs.readdirSync(captures);
    assert.equal(entries.length, 1);
    const directory = path.join(captures, entries[0]);
    const observed = JSON.parse(fs.readFileSync(path.join(directory, "observation.json"), "utf8"));
    assert.equal(observed.state, "process-closed");
    assert.deepEqual(observed.process, { exitStatus: 23, signal: null, error: null });
    for (const stream of ["client", "server"] as const) {
      const bytes = fs.readFileSync(path.join(directory, observed.streams[stream].path));
      assert.deepEqual(bytes, frames);
      assert.equal(observed.streams[stream].sha256, sha256(frames));
      assert.equal(observed.streams[stream].truncated, false);
      assert.deepEqual(observed.streams[stream].framing, { state: "complete", messageCount: 2 });
    }
    assert.equal(
      fs.readFileSync(path.join(directory, "stderr.bin"), "utf8"),
      "observer stderr 🧪\n",
    );
    assert.equal(observed.acceptance.nativeHandled, 0);
    assert.equal(observed.acceptance.wholeFixesClosed, 0);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
