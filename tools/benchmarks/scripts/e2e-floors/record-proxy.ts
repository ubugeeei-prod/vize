// record-proxy.ts <exec...>: forwards host<->mapper; appends each transform response (id stripped) to $REC_DIR/bodies.bin + index.jsonl.
import { spawn } from "node:child_process";
import { openSync, writeSync } from "node:fs";
const [command, ...args] = process.argv.slice(2);
const dir = process.env.REC_DIR!;
const bfd = openSync(`${dir}/bodies.bin`, "w"); const ifd = openSync(`${dir}/index.jsonl`, "w");
const files = new Map<string, string>(); let off = 0;
function reader(on: (body: Buffer) => void) {
  let buf = Buffer.alloc(0);
  return (chunk: Buffer) => {
    buf = buf.length ? Buffer.concat([buf, chunk]) : chunk;
    for (;;) {
      const h = buf.indexOf("\r\n\r\n"); if (h < 0) return;
      const len = Number(/Content-Length:\s*(\d+)/i.exec(buf.subarray(0, h).toString("ascii"))![1]);
      if (buf.length < h + 4 + len) return;
      on(buf.subarray(h + 4, h + 4 + len)); buf = buf.subarray(h + 4 + len);
    }
  };
}
const child = spawn(command, args, { stdio: ["pipe", "pipe", "inherit"] });
const fromHost = reader((b) => { const m = JSON.parse(b.toString("utf8")); if (m.method === "transform") files.set(JSON.stringify(m.id), m.params.fileName); });
const fromMapper = reader((b) => {
  const m = JSON.parse(b.toString("utf8")); const f = files.get(JSON.stringify(m.id)); if (!f || !m.result) return;
  const body = Buffer.from(JSON.stringify({ jsonrpc: "2.0", result: m.result }).slice(1));
  writeSync(bfd, body); writeSync(ifd, JSON.stringify([f, off, body.length]) + "\n"); off += body.length;
});
process.stdin.on("data", (c: Buffer) => { fromHost(c); child.stdin.write(c); });
process.stdin.on("end", () => child.stdin.end());
child.stdout.on("data", (c: Buffer) => { fromMapper(c); process.stdout.write(c); });
child.on("close", (code) => process.exit(code ?? 0));
