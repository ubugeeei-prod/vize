import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
const [runtime, bin, root, app] = JSON.parse(process.argv[2]);
const child = spawn(runtime, [bin, "lsp"], { cwd: root, stdio: ["pipe", "pipe", "pipe"] });
let buffer = Buffer.alloc(0);
let stderr = "";
let settled = false;
let cleanTimer: NodeJS.Timeout | undefined;
let diagnostics: unknown[] = [];
let shutdownRequested = false;
const timer = setTimeout(
  () =>
    finish(
      new Error(
        "LSP smoke timed out with diagnostics: " + JSON.stringify(diagnostics) + "\n" + stderr,
      ),
    ),
  60_000,
);
function send(message: Record<string, unknown>) {
  const body = Buffer.from(JSON.stringify({ jsonrpc: "2.0", ...message }));
  child.stdin.write(
    Buffer.concat([Buffer.from("Content-Length: " + body.length + "\r\n\r\n"), body]),
  );
}
function finish(error: Error) {
  if (settled) return;
  settled = true;
  clearTimeout(timer);
  clearTimeout(cleanTimer);
  if (error) {
    console.error(error);
    child.kill();
    process.exitCode = 1;
  }
}
child.stderr.on("data", (chunk) => {
  stderr = (stderr + chunk).slice(-4000);
});
child.stdout.on("data", (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);
  for (;;) {
    const header = buffer.indexOf("\r\n\r\n");
    if (header < 0) return;
    const length = Number(
      /Content-Length: (\d+)/i.exec(buffer.subarray(0, header).toString())?.[1],
    );
    if (buffer.length < header + 4 + length) return;
    const message = JSON.parse(buffer.subarray(header + 4, header + 4 + length).toString());
    buffer = buffer.subarray(header + 4 + length);
    if (message.id === 1) {
      send({ method: "initialized", params: {} });
      send({
        method: "textDocument/didOpen",
        params: {
          textDocument: {
            uri: pathToFileURL(app).href,
            languageId: "vue",
            version: 1,
            text: readFileSync(app, "utf8"),
          },
        },
      });
    } else if (
      message.method === "textDocument/publishDiagnostics" &&
      message.params.uri === pathToFileURL(app).href
    ) {
      diagnostics = message.params.diagnostics ?? [];
      clearTimeout(cleanTimer);
      if (diagnostics.length === 0)
        cleanTimer = setTimeout(() => {
          shutdownRequested = true;
          send({ id: 2, method: "shutdown", params: null });
        }, 500);
    } else if (message.id === 2) {
      send({ method: "exit", params: null });
    }
  }
});
child.on("exit", (code) => {
  if (!settled && (code !== 0 || !shutdownRequested))
    return finish(
      new Error(
        "LSP exited " + code + " with diagnostics: " + JSON.stringify(diagnostics) + "\n" + stderr,
      ),
    );
  if (!settled) {
    settled = true;
    clearTimeout(timer);
  }
});
const rootUri = pathToFileURL(root).href;
send({
  id: 1,
  method: "initialize",
  params: {
    processId: process.pid,
    rootUri,
    workspaceFolders: [{ uri: rootUri, name: "root" }],
    capabilities: {},
  },
});
