/**
 * Syscall census of the content mapper on Linux: which filesystem calls one
 * transform costs, per build, in the editor's serial shape (every request a
 * fresh snapshot) and the host's burst shape (one shared snapshot).
 *
 * Usage (Linux with strace):
 *   node tools/benchmarks/scripts/content-mapper-census.ts \
 *     --bin base=/tmp/vize-base --bin head=target/ci-opt/vize \
 *     --project /tmp/pair/vue-1000 --stream /tmp/pair/stream-1000.bin \
 *     --serial-files 200 --out /tmp/census.md
 */

import { spawn, spawnSync } from "node:child_process";
import { readFileSync, readdirSync, rmSync, writeFileSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

type Census = { calls: Map<string, number>; paths: Map<string, number>; transforms: number };

/** `openat(AT_FDCWD, "/x/y.ts", ...) = 3` → syscall name and the first quoted path. */
export function parseStraceLine(line: string): { call: string; path?: string } | undefined {
  const match = /^(?:\[pid\s+\d+\]\s+|\d+\s+)?([a-z_0-9]+)\((.*)$/.exec(line);
  if (!match?.[1]) return undefined;
  const path = /"((?:[^"\\]|\\.)*)"/.exec(match[2] ?? "")?.[1];
  return { call: match[1], path };
}

/** Collapse per-file spellings so one row stands for one kind of probe. */
export function pathShape(path: string, project: string): string {
  const relative = path.startsWith(project) ? `<project>${path.slice(project.length)}` : path;
  return relative
    .replace(/\/f\d{4}\//g, "/fNNNN/")
    .replace(/[A-Z][a-z]+\d{2}\./g, "Name.")
    .replace(/\d+/g, "N");
}

function collect(files: string[], project: string, transforms: number): Census {
  const census: Census = { calls: new Map(), paths: new Map(), transforms };
  for (const file of files) {
    for (const line of readFileSync(file, "utf8").split("\n")) {
      const parsed = parseStraceLine(line);
      if (!parsed) continue;
      census.calls.set(parsed.call, (census.calls.get(parsed.call) ?? 0) + 1);
      if (parsed.path) {
        const key = `${parsed.call} ${pathShape(parsed.path, project)}`;
        census.paths.set(key, (census.paths.get(key) ?? 0) + 1);
      }
    }
  }
  return census;
}

const TRACE = "trace=%file,%desc,%memory,futex";

function burstCensus(bin: string, project: string, stream: string, transforms: number): Census {
  const dir = mkdtempSync(join(tmpdir(), "census-"));
  const input = readFileSync(stream);
  const result = spawnSync(
    "strace",
    ["-f", "-ff", "-qq", "-e", TRACE, "-o", join(dir, "t"), bin, "content-mapper"],
    {
      cwd: project,
      input,
      maxBuffer: 1 << 30,
    },
  );
  if (result.status !== 0) throw new Error(`strace burst failed: ${result.stderr.toString()}`);
  const census = collect(
    readdirSync(dir).map((name) => join(dir, name)),
    project,
    transforms,
  );
  rmSync(dir, { recursive: true, force: true });
  return census;
}

async function serialCensus(
  bin: string,
  project: string,
  messages: Buffer[],
  transforms: number,
): Promise<Census> {
  const dir = mkdtempSync(join(tmpdir(), "census-"));
  const child = spawn(
    "strace",
    ["-f", "-ff", "-qq", "-e", TRACE, "-o", join(dir, "t"), bin, "content-mapper"],
    {
      cwd: project,
      stdio: ["pipe", "pipe", "inherit"],
    },
  );
  let pending: Buffer = Buffer.alloc(0);
  let wake: (() => void) | undefined;
  child.stdout.on("data", (chunk: Buffer) => {
    pending = Buffer.concat([pending, chunk]);
    for (;;) {
      const end = pending.indexOf("\r\n\r\n");
      if (end < 0) return;
      const length = Number(
        /Content-Length:\s*(\d+)/i.exec(pending.toString("ascii", 0, end))?.[1],
      );
      if (pending.length < end + 4 + length) return;
      pending = pending.subarray(end + 4 + length);
      const done = wake;
      wake = undefined;
      done?.();
    }
  });
  for (const message of messages) {
    const reply = new Promise<void>((done) => (wake = done));
    child.stdin.write(message);
    await reply;
  }
  child.stdin.end();
  await new Promise((done) => child.on("close", done));
  const census = collect(
    readdirSync(dir).map((name) => join(dir, name)),
    project,
    transforms,
  );
  rmSync(dir, { recursive: true, force: true });
  return census;
}

/** Split a framed stream back into its frames. */
function frames(stream: Buffer): Buffer[] {
  const out: Buffer[] = [];
  let offset = 0;
  while (offset < stream.length) {
    const end = stream.indexOf("\r\n\r\n", offset);
    if (end < 0) break;
    const length = Number(
      /Content-Length:\s*(\d+)/i.exec(stream.toString("ascii", offset, end))?.[1],
    );
    out.push(stream.subarray(offset, end + 4 + length));
    offset = end + 4 + length;
  }
  return out;
}

export function censusMarkdown(
  label: string,
  rows: { build: string; shape: string; census: Census }[],
): string {
  const lines = [`### Content mapper syscall census: ${label}`, ""];
  const calls = [...new Set(rows.flatMap((row) => [...row.census.calls.keys()]))]
    .map((call) => ({
      call,
      total: Math.max(...rows.map((row) => row.census.calls.get(call) ?? 0)),
    }))
    .sort((a, b) => b.total - a.total)
    .slice(0, 16)
    .map(({ call }) => call);
  lines.push(
    `| build | shape | transforms | ${calls.join(" | ")} |`,
    `|---|---|---:|${calls.map(() => "---:").join("|")}|`,
  );
  for (const row of rows) {
    const per = (call: string) =>
      ((row.census.calls.get(call) ?? 0) / row.census.transforms).toFixed(1);
    lines.push(
      `| ${row.build} | ${row.shape} | ${row.census.transforms} | ${calls.map(per).join(" | ")} |`,
    );
  }
  lines.push(
    "",
    "Calls per transform. Top filesystem probes by path shape (calls per transform):",
    "",
  );
  for (const row of rows) {
    lines.push(
      `<details><summary>${row.build} ${row.shape}</summary>`,
      "",
      "| calls | probe |",
      "|---:|---|",
    );
    for (const [key, count] of [...row.census.paths].sort((a, b) => b[1] - a[1]).slice(0, 30)) {
      lines.push(`| ${(count / row.census.transforms).toFixed(2)} | \`${key}\` |`);
    }
    lines.push("", "</details>", "");
  }
  return `${lines.join("\n")}\n`;
}

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      bin: { type: "string", multiple: true },
      project: { type: "string" },
      stream: { type: "string" },
      "serial-files": { type: "string", default: "200" },
      label: { type: "string", default: "linux" },
      out: { type: "string" },
    },
  });
  if (!values.project || !values.stream || !values.out || !values.bin?.length) {
    throw new Error("usage: --bin id=path --project <dir> --stream <file> --out <file>");
  }
  const project = resolve(values.project);
  const stream = readFileSync(resolve(values.stream));
  const all = frames(stream);
  const transforms = all.length - 2;
  const serial = all.slice(0, 2 + Math.min(Number(values["serial-files"]), transforms));
  const rows: { build: string; shape: string; census: Census }[] = [];
  for (const spec of values.bin) {
    const at = spec.indexOf("=");
    const [build, bin] = [spec.slice(0, at), resolve(spec.slice(at + 1))];
    rows.push({
      build,
      shape: "burst",
      census: burstCensus(bin, project, resolve(values.stream), transforms),
    });
    rows.push({
      build,
      shape: "serial",
      census: await serialCensus(bin, project, serial, serial.length - 2),
    });
  }
  const markdown = censusMarkdown(values.label, rows);
  writeFileSync(values.out, markdown);
  process.stdout.write(markdown);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
