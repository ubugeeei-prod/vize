/**
 * Drives the project-local `vize lsp` of a fresh project over stdio (#3956).
 *
 * Usage: `smoke-release-lsp-session.ts <project-root>`. The project must be the
 * TypeScript fresh-project shape after `vize init` and the planned install:
 * `src/App.vue` renders `<HelloWorld :msg="title" />` and imports the
 * component from `./components/HelloWorld.vue`. The server binary is the
 * project's own `node_modules/vize/bin/vize`, so the session proves the
 * installed package, not a checkout build.
 */
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import type { ConfigLspReceipt } from "./smoke-release-config-lsp.ts";

const [projectRoot, mode, workspaceRoot] = process.argv.slice(2);
assert.ok(projectRoot, "Usage: smoke-release-lsp-session.ts <project-root>");
assert.ok(mode === undefined || mode === "--config-workspace", "unknown installed LSP mode");
const root = fs.realpathSync(projectRoot);
if (mode === "--config-workspace") assert.ok(workspaceRoot, "workspace root is required");
const workspace = workspaceRoot === undefined ? root : fs.realpathSync(workspaceRoot);
// Load config-only dependencies before spawning, so general public observers
// remain dependency-free and config custody still sees every child byte.
const configSession =
  mode === "--config-workspace" ? await import("./smoke-release-config-lsp.ts") : undefined;
const appPath = path.join(root, "src", "App.vue");
const app = pathToFileURL(appPath).href;
const componentPath = path.join(root, "src", "components", "HelloWorld.vue");
const appText = fs.readFileSync(appPath, "utf8");

// The server always traces editor preparation, which shows whether a typed
// request reached the checker. With VIZE_SMOKE_LSP_LOG_DIR set, the whole
// session (stderr, every feature result) is also kept there for CI artifacts,
// so a host-only failure such as the typeless Windows hover (#8940) shows
// where the checker path gave up.
const logDir = process.env.VIZE_SMOKE_LSP_LOG_DIR;
const server = spawn(process.execPath, [path.join(root, "node_modules/vize/bin/vize"), "lsp"], {
  cwd: root,
  stdio: ["pipe", "pipe", "pipe"],
  env: { ...process.env, VIZE_TRACE_EDITOR_PREPARATION: "1" },
});
let stderr = "";
server.stderr.on("data", (chunk: Buffer) => (stderr += chunk.toString("utf8")));
const results: { name: string; result: unknown }[] = [];
// INFO lines are mostly per-request noise; warnings and errors say where the
// checker path gave up.
function serverEvidence(): string {
  const lines = stderr.split(/\r?\n/);
  const notable = lines.filter((line) =>
    /\b(WARN|ERROR)\b|panicked|corsa|tsgo|vize_editor_preparation/i.test(line),
  );
  return [
    `--- server warnings and errors (${notable.length} of ${lines.length} lines) ---`,
    ...notable.slice(-120).map((line) => line.slice(0, 600)),
    "--- server stderr (tail) ---",
    stderr.slice(-3000),
  ].join("\n");
}
function writeSessionLog(custody?: ConfigLspReceipt): void {
  if (mode === "--config-workspace") {
    // Existing CI logs also retain the whole record when no artifact directory
    // was requested; no packet is reduced to its diagnostic codes or a summary.
    console.log(
      JSON.stringify({
        root,
        workspace,
        app,
        results,
        messages,
        stderr: custody?.stderr ?? stderr,
        custody,
      }),
    );
  }
  if (!logDir) return;
  fs.mkdirSync(logDir, { recursive: true });
  const name = `lsp-${process.platform}-${process.arch}-node${process.versions.node}-${process.pid}`;
  fs.writeFileSync(
    path.join(logDir, `${name}.stderr.log`),
    custody ? Buffer.from(custody.stderrBase64, "base64") : stderr,
  );
  if (custody)
    fs.writeFileSync(
      path.join(logDir, `${name}.stdout.lsp`),
      Buffer.from(custody.stdoutBase64, "base64"),
    );
  fs.writeFileSync(
    path.join(logDir, `${name}.results.json`),
    JSON.stringify(
      { root, workspace, app, results, messages, ...(custody ? { custody } : {}) },
      null,
      2,
    ),
  );
}

type Message = {
  id?: number;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: unknown;
};
const pending = new Map<number, (message: Message) => void>();
const notifications: Message[] = [];
const messages: Message[] = [];
let buffer = Buffer.alloc(0);
let protocolError: unknown;
server.stdout.on("data", (chunk: Buffer) => {
  buffer = Buffer.concat([buffer, chunk]);
  if (protocolError !== undefined) return;
  try {
    for (;;) {
      const header = buffer.indexOf("\r\n\r\n");
      if (header < 0) return;
      const length = Number(
        /Content-Length: (\d+)/i.exec(buffer.subarray(0, header).toString())?.[1],
      );
      if (mode === "--config-workspace")
        assert.ok(Number.isSafeInteger(length) && length >= 0, "invalid LSP Content-Length");
      if (buffer.length < header + 4 + length) return;
      const message = JSON.parse(buffer.subarray(header + 4, header + 4 + length).toString("utf8"));
      messages.push(message);
      buffer = buffer.subarray(header + 4 + length);
      if (message.id !== undefined && message.method === undefined)
        pending.get(message.id)?.(message);
      else if (message.method !== undefined) notifications.push(message);
    }
  } catch (error) {
    if (mode !== "--config-workspace") throw error;
    protocolError = error;
  }
});

function send(message: object): void {
  const body = JSON.stringify({ jsonrpc: "2.0", ...message });
  server.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
}
let nextId = 0;
function request(method: string, params: unknown): Promise<unknown> {
  const id = ++nextId;
  return new Promise((resolve, reject) => {
    const timer = setTimeout(
      () => reject(new Error(`${method} timed out\n${stderr.slice(-4000)}`)),
      60_000,
    );
    pending.set(id, (message) => {
      clearTimeout(timer);
      if (message.error) reject(new Error(`${method} failed: ${JSON.stringify(message.error)}`));
      else resolve(message.result);
    });
    send({ id, method, params });
  });
}
function position(text: string, needle: string, offset = 0) {
  const index = text.indexOf(needle);
  assert.ok(index >= 0, `${needle} is not in App.vue`);
  const before = text.slice(0, index + offset).split("\n");
  return { line: before.length - 1, character: before.at(-1)!.length };
}

async function initialize(): Promise<void> {
  await request("initialize", {
    processId: process.pid,
    rootUri: pathToFileURL(workspace).href,
    capabilities: {},
    workspaceFolders: [{ uri: pathToFileURL(workspace).href, name: path.basename(workspace) }],
  });
  send({ method: "initialized", params: {} });
  send({
    method: "textDocument/didOpen",
    params: { textDocument: { uri: app, languageId: "vue", version: 1, text: appText } },
  });
}

const at = (needle: string, offset = 0) => ({
  textDocument: { uri: app },
  position: position(appText, needle, offset),
});

// URIs are compared as paths: servers may spell a Windows drive as C: or c%3A.
const samePath = (left: string, right: string) =>
  process.platform === "win32" ? left.toLowerCase() === right.toLowerCase() : left === right;
function mentions(result: unknown, file: string): boolean {
  const json = JSON.stringify(result) ?? "";
  const uris = [...json.matchAll(/"file:\/\/[^"]+"/g)].map(
    (match) => JSON.parse(match[0]) as string,
  );
  return uris.some((uri) => samePath(path.resolve(fileURLToPath(uri)), file));
}

if (configSession) {
  await configSession.runConfigLspSession(server, {
    run: async () => {
      await initialize();
      await configSession.runConfigWorkspaceLsp({
        workspace,
        notifications,
        request,
        send,
        record: (name, result) => results.push({ name, result }),
      });
    },
    shutdown: () => request("shutdown", undefined),
    exit: () => {
      send({ method: "exit" });
      server.stdin.end();
    },
    protocolState: () => ({ remainder: buffer, error: protocolError }),
    save: writeSessionLog,
  });
} else {
  await initialize();
  // Every feature is checked and every failure reported, so one run shows the
  // whole state of a host.
  const failures: string[] = [];
  async function expectFeature(
    name: string,
    method: string,
    params: unknown,
    ok: (result: unknown) => boolean,
  ) {
    try {
      const result = await request(method, params);
      results.push({ name, result });
      if (!ok(result)) failures.push(`${name}: ${JSON.stringify(result)?.slice(0, 600)}`);
    } catch (error) {
      results.push({ name, result: String(error) });
      failures.push(`${name}: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  await expectFeature(
    "hover on title shows its type",
    "textDocument/hover",
    at(':msg="title"', 6),
    (result) =>
      /\btitle\b/.test(JSON.stringify(result) ?? "") && /string/.test(JSON.stringify(result) ?? ""),
  );
  await expectFeature(
    "definition of HelloWorld reaches the component",
    "textDocument/definition",
    at("HelloWorld from", 1),
    (result) => mentions(result, componentPath),
  );
  await expectFeature(
    "template completion offers title",
    "textDocument/completion",
    at(':msg="title"', 6),
    (result) => {
      const items = Array.isArray(result)
        ? result
        : ((result as { items?: unknown[] } | null)?.items ?? []);
      return items.some((item) => (item as { label?: string }).label === "title");
    },
  );
  await expectFeature(
    "rename of title edits App.vue",
    "textDocument/rename",
    { ...at("const title", 6), newName: "heading" },
    (result) => mentions(result, appPath),
  );
  writeSessionLog();
  assert.deepEqual(failures, [], `project-local vize lsp failed in ${root}\n${serverEvidence()}`);

  await request("shutdown", undefined);
  send({ method: "exit" });
  console.log("runtime: project-local vize lsp (hover, definition, completion, rename)");
}
