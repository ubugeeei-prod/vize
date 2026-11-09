// replay-mapper.ts: answers transforms instantly from $REC_DIR (zero mapper wait floor); initialize/openProject like Vize.
import { readFileSync } from "node:fs";
const dir = process.env.REC_DIR!;
const bodies = readFileSync(`${dir}/bodies.bin`);
const index: Record<string, [number, number]> = {}; for (const l of readFileSync(`${dir}/index.jsonl`, "utf8").trim().split("\n")) { const [f, o, n] = JSON.parse(l); index[f] = [o, n]; }
let buf = Buffer.alloc(0);
const out = (b: Buffer) => { process.stdout.write(`Content-Length: ${b.length}\r\n\r\n`); process.stdout.write(b); };
const send = (o: object) => out(Buffer.from(JSON.stringify(o)));
let misses = 0;
process.stdin.on("data", (c: Buffer) => {
  buf = buf.length ? Buffer.concat([buf, c]) : c;
  for (;;) {
    const h = buf.indexOf("\r\n\r\n"); if (h < 0) return;
    const len = Number(/Content-Length:\s*(\d+)/i.exec(buf.subarray(0, h).toString("ascii"))![1]);
    if (buf.length < h + 4 + len) return;
    const m = JSON.parse(buf.subarray(h + 4, h + 4 + len).toString("utf8")); buf = buf.subarray(h + 4 + len);
    if (m.method === "initialize") send({ jsonrpc: "2.0", id: m.id, result: { positionEncoding: "utf-8", diagnosticSource: "vize" } });
    else if (m.method === "openProject") send({ jsonrpc: "2.0", id: m.id, result: { configIdentity: "replay" } });
    else if (m.method === "closeProject") send({ jsonrpc: "2.0", id: m.id, result: null });
    else if (m.method === "transform") {
      const e = index[m.params.fileName];
      if (!e) { misses++; send({ jsonrpc: "2.0", id: m.id, error: { code: -32603, message: "replay miss " + m.params.fileName } }); continue; }
      out(Buffer.concat([Buffer.from(`{"id":${JSON.stringify(m.id)},`), bodies.subarray(e[0], e[0] + e[1])]));
    } else if (m.id != null) send({ jsonrpc: "2.0", id: m.id, result: null });
  }
});
process.stdin.on("end", () => { if (misses) process.stderr.write(`replay misses: ${misses}\n`); process.exit(0); });
