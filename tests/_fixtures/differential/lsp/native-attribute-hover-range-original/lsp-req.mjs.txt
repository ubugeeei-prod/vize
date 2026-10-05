// node lsp-req.mjs <folder> <file> <method> [<line>:<character>...] [newName]
// Starts `vize lsp` on <folder>, opens every .vue file in it, waits 10 s, then sends <method> for <file> at each
// zero-based position and prints the result. textDocument/rename takes the new name as the last argument;
// textDocument/codeAction is sent once per published diagnostic of <file>; completion prints the item labels.
import { spawn } from "node:child_process";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [folderArg, fileArg, method, ...rest] = process.argv.slice(2);
const newName = method === "textDocument/rename" ? rest.pop() : undefined;
const folder = resolve(folderArg);
const server = spawn(resolve("node_modules/.bin/vize"), ["lsp", "--stdio"], { cwd: folder, stdio: ["pipe", "pipe", "ignore"] });
const write = (msg) => {
  const body = JSON.stringify({ jsonrpc: "2.0", ...msg });
  server.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
};
const pending = new Map();
const diagnostics = new Map();
let nextId = 0;
let buffer = Buffer.alloc(0);
server.stdout.on("data", (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);
  for (let end; (end = buffer.indexOf("\r\n\r\n")) >= 0; ) {
    const length = Number(/Content-Length: (\d+)/i.exec(buffer.subarray(0, end))[1]);
    if (buffer.length < end + 4 + length) break;
    const msg = JSON.parse(buffer.subarray(end + 4, end + 4 + length));
    buffer = buffer.subarray(end + 4 + length);
    if (msg.method && msg.id !== undefined) write({ id: msg.id, result: null });
    else if (msg.id !== undefined) pending.get(msg.id)?.(msg);
    else if (msg.method === "textDocument/publishDiagnostics") diagnostics.set(msg.params.uri, msg.params.diagnostics);
  }
});
const request = (m, params) => new Promise((done) => { pending.set(++nextId, done); write({ id: nextId, method: m, params }); });
const notify = (m, params) => write({ method: m, params });

await request("initialize", { processId: process.pid, rootUri: pathToFileURL(folder).href, capabilities: {} });
notify("initialized", {});
for (const name of readdirSync(folder).filter((n) => n.endsWith(".vue"))) {
  const path = resolve(folder, name);
  notify("textDocument/didOpen", { textDocument: { uri: pathToFileURL(path).href, languageId: "vue", version: 1, text: readFileSync(path, "utf8") } });
}
await new Promise((r) => setTimeout(r, 10000));
const uri = pathToFileURL(resolve(folder, fileArg)).href;
const show = (label, result) => {
  const items = Array.isArray(result) ? result : result?.items;
  const text = method === "textDocument/completion" && items ? items.map((i) => i.label).join(", ") : JSON.stringify(result ?? null);
  console.log(`${label}: ${text.replaceAll(pathToFileURL(folder).href + "/", "")}`);
};
if (method === "textDocument/codeAction") {
  for (const d of diagnostics.get(uri) ?? []) {
    const { result } = await request(method, { textDocument: { uri }, range: d.range, context: { diagnostics: [d] } });
    show(`${d.range.start.line}:${d.range.start.character} ${d.code}`, result?.map((a) => a.title));
  }
} else {
  for (const p of rest) {
    const [line, character] = p.split(":").map(Number);
    const context = method === "textDocument/references" ? { includeDeclaration: true } : undefined;
    const params = { textDocument: { uri }, position: { line, character }, context, newName };
    const response = await request(method, params);
    show(`${method} @ ${p}`, response.error ? { error: response.error } : (response.result ?? null));
  }
}
await request("shutdown", null);
notify("exit", null);
