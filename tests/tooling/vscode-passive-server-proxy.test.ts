import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const proxy = fileURLToPath(
  new URL("../../editors/vscode/test/fixtures/passive-vize-server.cjs", import.meta.url),
);
const frame = (packet: unknown) => {
  const body = Buffer.from(JSON.stringify(packet));
  return Buffer.concat([
    Buffer.from(
      `Content-Length: ${body.length}\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n`,
    ),
    body,
  ]);
};
type Event = {
  kind: string;
  direction?: string;
  pid?: number;
  argv?: string[];
  binary?: string;
  binarySha256?: string;
  chunkBase64?: string;
  packet?: unknown;
  frameBase64?: string;
  code?: number | string;
  signal?: string;
};

function launch(directory: string, argv: string[], binary = process.execPath) {
  const evidence = path.join(directory, "protocol.jsonl");
  const child = spawn(process.execPath, [proxy, ...argv], {
    env: {
      ...process.env,
      VIZE_TEST_OBSERVED_SERVER_PATH: binary,
      VIZE_TEST_CONFIG_PROTOCOL_PATH: evidence,
    },
    stdio: ["pipe", "pipe", "pipe"],
  });
  const stdout: Buffer[] = [];
  const stderr: Buffer[] = [];
  child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
  child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
  const finished = new Promise<{ code: number | null; signal: NodeJS.Signals | null }>(
    (resolve, reject) => {
      child.on("error", reject);
      child.on("close", (code, signal) => resolve({ code, signal }));
    },
  );
  return {
    child,
    finished,
    stdout,
    stderr,
    events: (): Event[] =>
      fs
        .readFileSync(evidence, "utf8")
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line) as Event),
  };
}
function bytes(events: Event[], kind: string, direction?: string) {
  return Buffer.concat(
    events
      .filter((event) => event.kind === kind && event.direction === direction)
      .map((event) => Buffer.from(event.chunkBase64!, "base64")),
  );
}

test(
  "passive proxy forwards exact chunked Unicode frames, argv, stderr and nonzero exit",
  { timeout: 10000 },
  async (t) => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-passive-proxy-"));
    t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
    const server = path.join(directory, "actual-server.cjs");
    const request = { jsonrpc: "2.0", id: 7, method: "実際/echo", params: { text: "é🌸\r\n全体" } };
    const notification = {
      jsonrpc: "2.0",
      method: "outside/config",
      params: { uri: "file:///tmp/a%20b/設定.json" },
    };
    const response = { jsonrpc: "2.0", id: 7, result: "🌸é全体" };
    const input = Buffer.concat([frame(request), frame(notification)]);
    const output = Buffer.concat([frame(response), frame(notification)]);
    fs.writeFileSync(
      server,
      `const fs = require('node:fs');
const chunks = [];
process.stderr.write('ready\\n');
process.stdin.on('data', chunk => chunks.push(chunk));
process.stdin.on('end', () => {
 fs.writeFileSync(${JSON.stringify(path.join(directory, "received"))}, Buffer.concat(chunks));
 fs.writeFileSync(${JSON.stringify(path.join(directory, "argv.json"))}, JSON.stringify(process.argv.slice(2)));
 const output = Buffer.from(${JSON.stringify(output.toString("base64"))}, 'base64');
 let offset = 0;
 function send() {
  if (offset === output.length) { process.stderr.write('actual stderr 🌸\\n'); process.exitCode = 23; return; }
  const end = Math.min(offset + 3, output.length);
  process.stdout.write(output.subarray(offset, end)); offset = end; if (offset === 3) setTimeout(send, 30); else setImmediate(send);
 }
 send();
});`,
    );
    const argv = [server, "lsp", "--stdio", "space value", "設定🌸", "--version"];
    const run = launch(directory, argv);
    t.after(() => run.child.kill("SIGKILL"));
    await new Promise<void>((resolve) => run.child.stderr.once("data", () => resolve()));
    for (let offset = 0; offset < input.length; offset += 2) {
      run.child.stdin.write(input.subarray(offset, offset + 2));
      await new Promise<void>((resolve) =>
        offset === 0 ? setTimeout(resolve, 30) : setImmediate(resolve),
      );
    }
    run.child.stdin.end();
    assert.deepEqual(await run.finished, { code: 23, signal: null });
    assert.deepEqual(fs.readFileSync(path.join(directory, "received")), input);
    assert.deepEqual(
      JSON.parse(fs.readFileSync(path.join(directory, "argv.json"), "utf8")),
      argv.slice(1),
    );
    assert.deepEqual(Buffer.concat(run.stdout), output);
    assert.equal(Buffer.concat(run.stderr).toString(), "ready\nactual stderr 🌸\n");
    const events = run.events();
    const identity = events.find((event) => event.kind === "spawn")!;
    assert.deepEqual(identity, {
      kind: "spawn",
      pid: identity.pid,
      argv,
      binary: process.execPath,
      binarySha256: crypto
        .createHash("sha256")
        .update(fs.readFileSync(process.execPath))
        .digest("hex"),
    });
    assert.ok(Number.isInteger(identity.pid) && identity.pid! > 0);
    assert.ok(events.every((event) => event.pid === identity.pid));
    assert.deepEqual(bytes(events, "chunk", "client"), input);
    assert.deepEqual(bytes(events, "chunk", "server"), output);
    for (const direction of ["client", "server"]) {
      assert.ok(
        events.filter((event) => event.kind === "chunk" && event.direction === direction).length >
          1,
      );
    }
    assert.deepEqual(bytes(events, "stderr"), Buffer.concat(run.stderr));
    for (const [direction, packets] of [
      ["client", [request, notification]],
      ["server", [response, notification]],
    ] as const) {
      assert.deepEqual(
        events.filter((event) => event.kind === "frame" && event.direction === direction),
        packets.map((packet): Event => ({
          kind: "frame",
          direction,
          pid: identity.pid,
          packet,
          frameBase64: frame(packet).toString("base64"),
        })),
      );
    }
    assert.deepEqual(events.at(-1), { kind: "exit", pid: identity.pid, code: 23, signal: null });
  },
);

test(
  "version invocation remains a real CLI invocation and exits with stdin still open",
  { timeout: 10000 },
  async (t) => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-passive-version-"));
    t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
    const run = launch(directory, ["--version"]);
    t.after(() => run.child.kill("SIGKILL"));
    assert.deepEqual(await run.finished, { code: 0, signal: null });
    assert.equal(Buffer.concat(run.stdout).toString(), `${process.version}\n`);
    const events = run.events();
    assert.deepEqual(events.find((event) => event.kind === "spawn")?.argv, ["--version"]);
    assert.deepEqual(bytes(events, "chunk", "server"), Buffer.concat(run.stdout));
    assert.equal(
      events.some((event) => event.kind === "frame"),
      false,
    );
  },
);

test(
  "termination reaches the actual child and the proxy mirrors its signal",
  { timeout: 10000, skip: process.platform === "win32" },
  async (t) => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-passive-signal-"));
    t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
    const server = path.join(directory, "waiting.cjs");
    fs.writeFileSync(server, "process.stdout.write('ready'); process.stdin.resume();");
    const run = launch(directory, [server]);
    t.after(() => run.child.kill("SIGKILL"));
    await new Promise<void>((resolve) => run.child.stdout.once("data", () => resolve()));
    run.child.kill("SIGTERM");
    assert.deepEqual(await run.finished, { code: null, signal: "SIGTERM" });
    assert.deepEqual(run.events().at(-1), {
      kind: "exit",
      pid: run.events()[0].pid,
      code: null,
      signal: "SIGTERM",
    });
  },
);

test(
  "spawn errors terminate without waiting for the open client stdin",
  { timeout: 10000, skip: process.platform === "win32" },
  async (t) => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-passive-spawn-error-"));
    t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
    const binary = path.join(directory, "not-executable");
    fs.writeFileSync(binary, "not an executable", { mode: 0o600 });
    const run = launch(directory, ["lsp"], binary);
    t.after(() => run.child.kill("SIGKILL"));
    assert.deepEqual(await run.finished, { code: 1, signal: null });
    const events = run.events();
    assert.equal(events.find((event) => event.kind === "error")?.code, "EACCES");
    assert.equal(
      events.some((event) => event.kind === "spawn"),
      false,
    );
    assert.equal(Buffer.concat(run.stdout).length, 0);
  },
);
