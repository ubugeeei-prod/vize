import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { once } from "node:events";
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import type { ServerResponse } from "node:http";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import type { TestContext } from "node:test";

const script = path.resolve(
  import.meta.dirname,
  "../../.github/actions/vscode-host-smoke/download-artifact.sh",
);
const options = { skip: process.platform === "win32" ? "Linux editor CI downloader" : false };
const content = "verified pinned editor bytes";
const checksum = createHash("sha256").update(content).digest("hex");

test("both VS Code host suites run after a bounded pinned editor download", () => {
  const action = readFileSync(path.join(path.dirname(script), "action.yml"), "utf8");
  const downloadAt = action.indexOf("- name: Download pinned VS Code with a total time budget");
  assert.ok(downloadAt >= 0);
  for (const name of [
    "Run VS Code host lifecycle and delayed-response regressions",
    "Run VS Code host smoke against the real server",
  ]) {
    assert.ok(action.indexOf(`- name: ${name}`) > downloadAt, name);
  }
  const step = action.slice(downloadAt).split(/\n {4}- name: /)[0];
  assert.match(step, /timeout --signal=TERM --kill-after=10s 300s vp run/);
  assert.match(step, /VIZE_TEST_VSCODE_VERSION: "1\.107\.1"/);
});

test(
  "editor download retries a failed transport before publishing verified bytes",
  options,
  async (t) => {
    let attempts = 0;
    const fixture = await serverFixture(t, (response) => {
      if (++attempts === 1) response.writeHead(503).end("try again");
      else response.end(content);
    });
    const result = await download(fixture, checksum);
    assert.equal(result.code, 0, result.stderr);
    assert.equal(attempts, 2);
    assert.equal(readFileSync(fixture.destination, "utf8"), content);
    assert.deepEqual(readdirSync(fixture.directory), ["editor"]);
  },
);

test(
  "editor download rejects a checksum mismatch and retains the prior artifact",
  options,
  async (t) => {
    const fixture = await serverFixture(t, (response) => response.end("corrupt"));
    writeFileSync(fixture.destination, content);
    const result = await download(fixture, checksum);
    assert.notEqual(result.code, 0);
    assert.match(result.stderr, /checksum did NOT match|computed checksum did NOT match/);
    assert.equal(readFileSync(fixture.destination, "utf8"), content);
    assert.deepEqual(readdirSync(fixture.directory), ["editor"]);
  },
);

test("editor download terminates a stalled body and removes partial files", options, async (t) => {
  const fixture = await serverFixture(t, (response) => {
    response.writeHead(200);
    response.write("unfinished");
  });
  const result = await download(fixture, checksum, {
    VIZE_EDITOR_DOWNLOAD_TIMEOUT_SECONDS: "1",
    VIZE_EDITOR_DOWNLOAD_RETRY_SECONDS: "1",
  });
  assert.notEqual(result.code, 0);
  assert.match(result.stderr, /timed out/i);
  assert.deepEqual(readdirSync(fixture.directory), []);
});

async function serverFixture(t: TestContext, respond: (response: ServerResponse) => void) {
  const directory = mkdtempSync(path.join(os.tmpdir(), "vize-editor-download-"));
  const server = createServer((_request, response) => respond(response));
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  t.after(() => {
    server.closeAllConnections();
    server.close();
    rmSync(directory, { recursive: true, force: true });
  });
  const address = server.address();
  assert.ok(address && typeof address === "object");
  return {
    directory,
    destination: path.join(directory, "editor"),
    url: `http://127.0.0.1:${address.port}/editor`,
  };
}

async function download(
  fixture: { url: string; destination: string },
  digest: string,
  environment = {},
) {
  const child = spawn("bash", [script, fixture.url, fixture.destination, digest], {
    env: { ...process.env, ...environment },
    stdio: ["ignore", "pipe", "pipe"],
    timeout: 10_000,
  });
  let stderr = "";
  child.stdout.resume();
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  const [code, signal] = await once(child, "close");
  assert.equal(signal, null, stderr);
  return { code, stderr };
}
