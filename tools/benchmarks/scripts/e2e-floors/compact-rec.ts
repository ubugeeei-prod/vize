// compactRecording(src, dst): compactResult() over every recorded response.
import { copyFileSync, existsSync, mkdirSync, openSync, readFileSync, writeSync } from "node:fs";
import { compactResult } from "./compact-text.ts";
export function compactRecording(src: string, dst: string): string {
  mkdirSync(dst, { recursive: true });
  const bodies = readFileSync(`${src}/bodies.bin`);
  const bfd = openSync(`${dst}/bodies.bin`, "w"), ifd = openSync(`${dst}/index.jsonl`, "w");
  let off = 0, before = 0, removed = 0;
  for (const l of readFileSync(`${src}/index.jsonl`, "utf8").trim().split("\n")) {
    const [f, o, len] = JSON.parse(l); before += len;
    const msg = JSON.parse("{" + bodies.subarray(o, o + len).toString("utf8"));
    removed += compactResult(msg.result).removed;
    const body = Buffer.from(JSON.stringify(msg).slice(1));
    writeSync(bfd, body); writeSync(ifd, JSON.stringify([f, off, body.length]) + "\n"); off += body.length;
  }
  if (existsSync(`${src}/init.json`)) copyFileSync(`${src}/init.json`, `${dst}/init.json`);
  return `reply bytes ${before} -> ${off} (${(100 * (off - before) / before).toFixed(1)}%), text bytes removed ${removed}`;
}
