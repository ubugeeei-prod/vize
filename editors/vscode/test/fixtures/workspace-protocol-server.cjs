// The fixture launcher supplies `options`; no environment mutation is needed.
const fs = require("node:fs");
function log(event) {
  fs.appendFileSync(
    options.log,
    JSON.stringify({ event, pid: process.pid, args: process.argv.slice(2) }) + "\n",
  );
}
log("start");
if (process.argv.includes("--version")) {
  console.log(`vize ${options.version}`);
  process.exit(0);
}
if (options.mode === "exit") process.exit(42);
if (options.mode === "timeout") process.on("SIGTERM", () => {});
let input = Buffer.alloc(0);
function send(packet) {
  const body = JSON.stringify(packet);
  process.stdout.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
}
process.stdin.on("data", (chunk) => {
  input = Buffer.concat([input, chunk]);
  while (true) {
    const headerEnd = input.indexOf("\r\n\r\n");
    if (headerEnd < 0) return;
    const length = Number(/Content-Length: (\d+)/.exec(input.subarray(0, headerEnd).toString())[1]);
    if (input.length < headerEnd + 4 + length) return;
    const packet = JSON.parse(input.subarray(headerEnd + 4, headerEnd + 4 + length).toString());
    input = input.subarray(headerEnd + 4 + length);
    log(packet.method);
    if (packet.method === "initialize") {
      fs.appendFileSync(
        options.log,
        JSON.stringify({ event: "initialize-params", params: packet.params }) + "\n",
      );
      if (options.mode === "timeout") continue;
      if (options.mode === "overflow") {
        process.stdout.write("Content-Length: 2000000\r\n\r\n");
        continue;
      }
      send(
        options.mode === "incompatible"
          ? { jsonrpc: "2.0", id: packet.id, error: { code: -32601, message: "Unknown method" } }
          : { jsonrpc: "2.0", id: packet.id, result: { capabilities: { textDocumentSync: 1 } } },
      );
    } else if (packet.method === "shutdown") {
      send({ jsonrpc: "2.0", id: packet.id, result: null });
    } else if (packet.method === "exit") {
      process.exit(0);
    }
  }
});
