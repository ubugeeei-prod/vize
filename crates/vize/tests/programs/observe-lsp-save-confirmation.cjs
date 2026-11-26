// Controlled protocol peer only: no native-provider qualification is inferred.
const mode = process.argv[2];
let bytes = Buffer.alloc(0);
let publication;
function send(packet) {
  const body = Buffer.from(JSON.stringify(packet));
  require("node:fs").writeSync(
    1,
    Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]),
  );
}
function receive(packet) {
  if (packet.method === "textDocument/didChange") {
    publication = {
      jsonrpc: "2.0",
      method: "textDocument/publishDiagnostics",
      params: {
        uri: packet.params.textDocument.uri,
        version: packet.params.textDocument.version,
        diagnostics: [],
      },
    };
    send(publication);
  } else if (packet.method === "textDocument/didSave") {
    require("node:fs").writeSync(2, "confirmation-peer: saved\n");
    const next = structuredClone(publication);
    if (mode === "missing") return process.exit(0);
    if (mode === "wrong-uri") next.params.uri = "file:///foreign/App.vue";
    if (mode === "wrong-version") next.params.version++;
    if (mode === "wrong-payload")
      next.params.diagnostics = [
        {
          range: {
            start: { line: 0, character: 0 },
            end: { line: 0, character: 1 },
          },
          severity: 1,
          code: 2322,
          source: "vize/types",
          message: "controlled wrong diagnostic",
        },
      ];
    send(next);
    if (mode === "wrong-uri" || mode === "wrong-version") return process.exit(0);
    if (mode === "late-duplicate") send(next);
    if (mode === "late-wrong") {
      next.params.diagnostics = [
        {
          range: {
            start: { line: 0, character: 0 },
            end: { line: 0, character: 1 },
          },
          severity: 1,
          code: 2322,
          source: "vize/types",
          message: "controlled late diagnostic",
        },
      ];
      send(next);
    }
  } else if (packet.method === "exit") process.exit(0);
  else if (packet.id !== undefined) send({ jsonrpc: "2.0", id: packet.id, result: null });
}
process.stdin.on("data", (chunk) => {
  bytes = Buffer.concat([bytes, chunk]);
  for (;;) {
    const end = bytes.indexOf("\r\n\r\n");
    if (end < 0) return;
    const length = Number(/Content-Length: (\d+)/i.exec(bytes.subarray(0, end).toString())[1]);
    if (bytes.length < end + 4 + length) return;
    const packet = JSON.parse(bytes.subarray(end + 4, end + 4 + length));
    bytes = bytes.subarray(end + 4 + length);
    receive(packet);
  }
});
