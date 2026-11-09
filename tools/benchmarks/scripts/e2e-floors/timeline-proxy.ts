// timeline-proxy.ts <exec...>: forwards bytes host<->mapper and logs per-message timestamps to $TL_LOG (jsonl, written as it goes).
import { spawn } from "node:child_process";
import { openSync, writeSync } from "node:fs";
const [command, ...args] = process.argv.slice(2);
const fd = openSync(process.env.TL_LOG!, "w");
const rec = (o: object) => writeSync(fd, JSON.stringify({ t: performance.now(), ...o }) + "\n");
rec({ ev: "proxy-start" });
function reader(on: (msg: any, bytes: number) => void) {
  let buf = Buffer.alloc(0);
  return (chunk: Buffer) => {
    buf = buf.length ? Buffer.concat([buf, chunk]) : chunk;
    for (;;) {
      const h = buf.indexOf("\r\n\r\n"); if (h < 0) return;
      const len = Number(/Content-Length:\s*(\d+)/i.exec(buf.subarray(0, h).toString("ascii"))![1]);
      if (buf.length < h + 4 + len) return;
      const body = buf.subarray(h + 4, h + 4 + len); buf = buf.subarray(h + 4 + len);
      on(JSON.parse(body.toString("utf8")), len);
    }
  };
}
const child = spawn(command, args, { stdio: ["pipe", "pipe", "inherit"] });
const fromHost = reader((m, n) => rec({ ev: "req", id: m.id, method: m.method, file: m.params?.fileName, bytes: n, ...(m.method === "openProject" ? { params: m.params } : {}) }));
const fromMapper = reader((m, n) => rec({ ev: "res", id: m.id, bytes: n, err: m.error?.message }));
process.stdin.on("data", (c: Buffer) => { fromHost(c); child.stdin.write(c); });
process.stdin.on("end", () => { rec({ ev: "stdin-end" }); child.stdin.end(); });
child.stdout.on("data", (c: Buffer) => { fromMapper(c); process.stdout.write(c); });
child.on("close", (code) => { rec({ ev: "close" }); process.exit(code ?? 0); });
