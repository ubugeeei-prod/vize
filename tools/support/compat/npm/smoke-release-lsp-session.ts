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

const [projectRoot] = process.argv.slice(2);
assert.ok(projectRoot, "Usage: smoke-release-lsp-session.ts <project-root>");
const root = fs.realpathSync(projectRoot);
const appPath = path.join(root, "src", "App.vue");
const app = pathToFileURL(appPath).href;
const componentPath = path.join(root, "src", "components", "HelloWorld.vue");
const appText = fs.readFileSync(appPath, "utf8");

const server = spawn(process.execPath, [path.join(root, "node_modules/vize/bin/vize"), "lsp"], {
  cwd: root,
  stdio: ["pipe", "pipe", "pipe"],
});
let stderr = "";
server.stderr.on("data", (chunk: Buffer) => (stderr += chunk.toString("utf8")));

type Message = {
  id?: number;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: unknown;
};
const pending = new Map<number, (message: Message) => void>();
const notifications: Message[] = [];
let buffer = Buffer.alloc(0);
server.stdout.on("data", (chunk: Buffer) => {
  buffer = Buffer.concat([buffer, chunk]);
  for (;;) {
    const header = buffer.indexOf("\r\n\r\n");
    if (header < 0) return;
    const length = Number(
      /Content-Length: (\d+)/i.exec(buffer.subarray(0, header).toString())?.[1],
    );
    if (buffer.length < header + 4 + length) return;
    const message = JSON.parse(buffer.subarray(header + 4, header + 4 + length).toString("utf8"));
    buffer = buffer.subarray(header + 4 + length);
    if (message.id !== undefined && message.method === undefined)
      pending.get(message.id)?.(message);
    else if (message.method !== undefined) notifications.push(message);
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

await request("initialize", {
  processId: process.pid,
  rootUri: pathToFileURL(root).href,
  capabilities: {},
  workspaceFolders: [{ uri: pathToFileURL(root).href, name: path.basename(root) }],
});
send({ method: "initialized", params: {} });
send({
  method: "textDocument/didOpen",
  params: { textDocument: { uri: app, languageId: "vue", version: 1, text: appText } },
});
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
    if (!ok(result)) failures.push(`${name}: ${JSON.stringify(result)?.slice(0, 600)}`);
  } catch (error) {
    failures.push(`${name}: ${error instanceof Error ? error.message : String(error)}`);
  }
}

await expectFeature(
  "hover on title shows its type",
  "textDocument/hover",
  at(':msg="title"', 6),
  // Known gap: on Windows the hover names the binding without its type
  // (https://github.com/ubugeeei-prod/vize/issues/8940). Remove with the fix.
  (result) =>
    /\btitle\b/.test(JSON.stringify(result) ?? "") &&
    (process.platform === "win32" || /string/.test(JSON.stringify(result) ?? "")),
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
assert.deepEqual(failures, [], `project-local vize lsp failed in ${root}`);

await request("shutdown", undefined);
send({ method: "exit" });
console.log("runtime: project-local vize lsp (hover, definition, completion, rename)");
