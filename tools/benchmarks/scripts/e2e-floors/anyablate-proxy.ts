// anyablate-proxy.ts <exec...>: content-mapper proxy applying anyAblate to every transform text; drops mappings inside rewritten statements.
import { spawn } from "node:child_process";
import { anyAblate } from "./anyablate.ts";
const [command, ...args] = process.argv.slice(2);
const overlaps = (r: [number, number][], s: number, e: number) => r.some(([a, b]) => s < b && e > a);
const child = spawn(command, args, { stdio: ["pipe", "pipe", "inherit"] });
process.stdin.pipe(child.stdin);
let buf = Buffer.alloc(0);
child.stdout.on("data", (chunk: Buffer) => {
  buf = buf.length ? Buffer.concat([buf, chunk]) : chunk;
  for (;;) {
    const h = buf.indexOf("\r\n\r\n"); if (h < 0) return;
    const len = Number(/Content-Length:\s*(\d+)/i.exec(buf.subarray(0, h).toString("ascii"))![1]);
    if (buf.length < h + 4 + len) return;
    let body = buf.subarray(h + 4, h + 4 + len); buf = buf.subarray(h + 4 + len);
    const msg = JSON.parse(body.toString("utf8"));
    if (msg.result && typeof msg.result.text === "string" && Array.isArray(msg.result.mappings)) {
      const { text, ranges } = anyAblate(msg.result.text);
      msg.result.text = text;
      if (ranges.length) {
        msg.result.mappings = msg.result.mappings.filter((m: number[]) => !overlaps(ranges, m[0], m[0] + Math.max(1, m[1])));
        if (msg.result.semanticLinks) msg.result.semanticLinks = msg.result.semanticLinks.filter((l: any) => !overlaps(ranges, l.targetStart, l.targetStart + l.targetLength));
        if (msg.result.diagnosticDirectives) msg.result.diagnosticDirectives.directives = msg.result.diagnosticDirectives.directives.filter((d: number[]) => !overlaps(ranges, d[2], Math.max(d[2] + 1, d[3])));
      }
      body = Buffer.from(JSON.stringify(msg));
    }
    process.stdout.write(`Content-Length: ${body.length}\r\n\r\n`); process.stdout.write(body);
  }
});
child.on("close", (code) => process.exit(code ?? 0));
