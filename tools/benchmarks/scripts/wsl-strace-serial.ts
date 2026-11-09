/**
 * Time attribution of a serial content-mapper session under `strace -T`:
 * kernel time per filesystem call shape, per transform.
 *
 * Usage: node wsl-strace-serial.ts <vize> <projectParent> <label> [transforms]
 */
import { spawn } from "node:child_process";
import * as fs from "node:fs";
import { join } from "node:path";
import { COMPILER_OPTIONS, generateScaleProject, writeProject } from "./scale-project-generator.ts";

const [bin, parent, label, countText] = process.argv.slice(2);
const count = Number(countText ?? "100");
const project = join(parent, "vue-1000");
if (!fs.existsSync(join(project, "tsconfig.json"))) writeProject(project, generateScaleProject({ kind: "vue", sfcs: 1000 }).files);
const files: string[] = [];
const walk = (d: string) => {
  for (const e of fs.readdirSync(d, { withFileTypes: true })) {
    if (e.name === "node_modules" || e.name.startsWith(".")) continue;
    const p = join(d, e.name);
    if (e.isDirectory()) walk(p);
    else if (e.name.endsWith(".vue")) files.push(p);
  }
};
walk(project);
files.sort();
// Feature SFCs only: skip App/global/kit at the head of the sorted list.
const picked = files.filter((f) => f.includes("/features/")).slice(0, count);

async function session(traced: boolean): Promise<number[]> {
  const trace = join(parent, `strace-${label}.txt`);
  const argv = traced
    ? ["strace", "-f", "-T", "-qq", "-e", "trace=%file,%desc", "-o", trace, bin, "content-mapper"]
    : [bin, "content-mapper"];
  const child = spawn(argv[0], argv.slice(1), { cwd: project, stdio: ["pipe", "pipe", "inherit"] });
  let pending = Buffer.alloc(0);
  let wake: (() => void) | undefined;
  child.stdout.on("data", (chunk: Buffer) => {
    pending = pending.length ? Buffer.concat([pending, chunk]) : chunk;
    for (;;) {
      const end = pending.indexOf("\r\n\r\n");
      if (end < 0) return;
      const length = Number(/Content-Length:\s*(\d+)/i.exec(pending.toString("ascii", 0, end))![1]);
      if (pending.length < end + 4 + length) return;
      pending = pending.subarray(end + 4 + length);
      const done = wake; wake = undefined; done?.();
    }
  });
  let id = 1;
  const send = (method: string, params: unknown) => {
    const body = Buffer.from(JSON.stringify({ jsonrpc: "2.0", id: id++, method, params }));
    const reply = new Promise<void>((r) => (wake = r));
    child.stdin.write(Buffer.concat([Buffer.from(`Content-Length: ${body.length}\r\n\r\n`), body]));
    return reply;
  };
  await send("initialize", { positionEncodings: ["utf-8", "utf-16"], locale: "en" });
  await send("openProject", { configFileName: join(project, "tsconfig.json"), projectHandle: "p", options: {}, compilerOptions: COMPILER_OPTIONS });
  const lat: number[] = [];
  for (const file of picked) {
    const content = fs.readFileSync(file, "utf8");
    const t = performance.now();
    await send("transform", { fileName: file, content, projectHandle: "p" });
    lat.push(performance.now() - t);
  }
  child.stdin.end();
  await new Promise((r) => child.on("close", r));
  return lat;
}

const median = (xs: number[]) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];
const plain = await session(false);
const traced = await session(true);
const shape = (p: string) =>
  (p.startsWith(project) ? "<project>" + p.slice(project.length) : p)
    .replace(/\/f\d{4}(?=\/|$)/g, "/fNNNN")
    .replace(/[A-Z][a-z]+\d+\./g, "Name.");
const byCall = new Map<string, [number, number]>();
const byShape = new Map<string, [number, number]>();
const add = (m: Map<string, [number, number]>, k: string, us: number) => {
  const v = m.get(k) ?? [0, 0];
  v[0]++; v[1] += us; m.set(k, v);
};
let inTransforms = false;
for (const line of fs.readFileSync(join(parent, `strace-${label}.txt`), "utf8").split("\n")) {
  const m = /^\d+\s+([a-z_0-9]+)\((.*)<([\d.]+)>$/.exec(line);
  if (!m) continue;
  const [, call, rest, secs] = m;
  const path = /"((?:[^"\\]|\\.)*)"/.exec(rest)?.[1] ?? "";
  // Start counting at the first transform's request read.
  if (!inTransforms && path.includes("/features/")) inTransforms = true;
  if (!inTransforms) continue;
  const us = Number(secs) * 1e6;
  add(byCall, call, us);
  add(byShape, `${call} ${shape(path)}`, us);
}
const n = picked.length;
const lines = [
  `### strace -T serial attribution: ${label} (${n} feature transforms)`,
  "",
  `untraced serial p50 ${median(plain).toFixed(2)} ms; traced p50 ${median(traced).toFixed(2)} ms`,
  "",
  "| call | calls/transform | kernel µs/transform |",
  "|---|---:|---:|",
  ...[...byCall].sort((a, b) => b[1][1] - a[1][1]).map(([k, [c, us]]) => `| ${k} | ${(c / n).toFixed(2)} | ${(us / n).toFixed(0)} |`),
  "",
  "| probe | calls/transform | kernel µs/transform | µs/call |",
  "|---|---:|---:|---:|",
  ...[...byShape].sort((a, b) => b[1][1] - a[1][1]).slice(0, 60).map(([k, [c, us]]) => `| \`${k}\` | ${(c / n).toFixed(2)} | ${(us / n).toFixed(0)} | ${(us / c).toFixed(0)} |`),
  "",
];
console.log(lines.join("\n"));
if (process.env.PROBE_OUT) fs.appendFileSync(process.env.PROBE_OUT, lines.join("\n") + "\n");
