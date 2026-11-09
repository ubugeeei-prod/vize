// stub-mapper.ts: content mapper answering every transform with `export default {} as any`.
// Measures tsc's fixed costs (process, config, libs, .ts files, IPC) with no projection.
const text = process.env.STUB_TEXT ?? "declare const __stub: any;\nexport default __stub;\n";
let buf = Buffer.alloc(0);
const send = (o: object) => { const b = Buffer.from(JSON.stringify(o)); process.stdout.write(`Content-Length: ${b.length}\r\n\r\n`); process.stdout.write(b); };
process.stdin.on("data", (c: Buffer) => {
  buf = buf.length ? Buffer.concat([buf, c]) : c;
  for (;;) {
    const h = buf.indexOf("\r\n\r\n"); if (h < 0) return;
    const len = Number(/Content-Length:\s*(\d+)/i.exec(buf.subarray(0, h).toString("ascii"))![1]);
    if (buf.length < h + 4 + len) return;
    const m = JSON.parse(buf.subarray(h + 4, h + 4 + len).toString("utf8")); buf = buf.subarray(h + 4 + len);
    if (m.method === "initialize") send({ jsonrpc: "2.0", id: m.id, result: { positionEncoding: "utf-8", diagnosticSource: "vize" } });
    else if (m.method === "openProject") send({ jsonrpc: "2.0", id: m.id, result: { configIdentity: "stub" } });
    else if (m.method === "closeProject") send({ jsonrpc: "2.0", id: m.id, result: null });
    else if (m.method === "transform") send({ jsonrpc: "2.0", id: m.id, result: { diagnostics: [], extension: ".ts", mappings: [], text } });
    else if (m.id != null) send({ jsonrpc: "2.0", id: m.id, result: null });
  }
});
process.stdin.on("end", () => process.exit(0));
