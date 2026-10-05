import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const [file, languageId = "typescript"] = process.argv.slice(2);
const lsp = spawn("node_modules/.bin/vize", ["lsp", "--stdio"], { stdio: ["pipe", "pipe", "ignore"] });
const send = msg => {
  const s = JSON.stringify({ jsonrpc: "2.0", ...msg });
  lsp.stdin.write(`Content-Length: ${Buffer.byteLength(s)}\r\n\r\n${s}`);
};
let buf = "";
lsp.stdout.on("data", d => {
  buf += d;
  for (;;) {
    const m = /Content-Length: (\d+)\r\n\r\n/.exec(buf);
    if (!m || buf.length < m.index + m[0].length + Number(m[1])) return;
    const msg = JSON.parse(buf.slice(m.index + m[0].length, m.index + m[0].length + Number(m[1])));
    buf = buf.slice(m.index + m[0].length + Number(m[1]));
    if (msg.id === 1) {
      send({ method: "initialized", params: {} });
      const uri = pathToFileURL(resolve(file)).href;
      send({ method: "textDocument/didOpen", params: { textDocument: { uri, languageId, version: 1, text: readFileSync(file, "utf8") } } });
    }
    if (msg.method === "textDocument/publishDiagnostics") {
      console.log(JSON.stringify(msg.params.diagnostics.map(d => ({ source: d.source, message: d.message, range: d.range }))));
    }
  }
});
send({ id: 1, method: "initialize", params: { processId: process.pid, rootUri: pathToFileURL(process.cwd()).href, capabilities: {} } });
setTimeout(() => process.exit(0), 8000);
