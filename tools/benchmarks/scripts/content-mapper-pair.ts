/**
 * Content-mapper replay pair: two `vize` builds answer the identical
 * TypeScript content-mapper request stream on the same generated project.
 *
 * Two shapes mirror the hosts:
 * - burst: every transform is queued at once, as `tsc` does for a program;
 *   the mapper overlaps them and shares one imported-type snapshot.
 * - serial: one transform at a time, as an editor sends them; every request
 *   starts a fresh snapshot, so filesystem probes are paid per request.
 *
 * Runs rotate the build order. A cell whose builds or runs disagree on the
 * response bytes publishes no timing unless `--allow-output-change` is set.
 * Burst CPU and peak memory come from GNU time (Linux), BSD time (macOS) or
 * the process object (Windows).
 *
 * Usage:
 *   node tools/benchmarks/scripts/content-mapper-pair.ts \
 *     --bin base=/tmp/vize-base --bin head=target/ci-opt/vize \
 *     --root /tmp/pair --sizes 1000,5000 --runs 5 --label linux-ext4 --out /tmp/pair-out
 */

import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  closeSync,
  mkdirSync,
  openSync,
  readFileSync,
  readdirSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { parseArgs } from "node:util";

import { COMPILER_OPTIONS, generateScaleProject, writeProject } from "./scale-project-generator.ts";
import {
  type BurstSample,
  type Cell,
  type SerialSample,
  frame,
  markdown,
  summarizeCell,
} from "./content-mapper-pair-report.ts";

/** A mapper process: a `vize` build, or a reference mapper timed beside them. */
type Build = { id: string; bin: string; argv: string[]; reference: boolean };

function sfcFiles(dir: string): string[] {
  const out: string[] = [];
  const walk = (current: string) => {
    for (const entry of readdirSync(current, { withFileTypes: true })) {
      if (entry.name === "node_modules" || entry.name.startsWith(".")) continue;
      const path = join(current, entry.name);
      if (entry.isDirectory()) walk(path);
      else if (entry.name.endsWith(".vue")) out.push(path);
    }
  };
  walk(dir);
  return out.sort();
}

type Message = { method: string; params: unknown };

function sessionMessages(project: string, files: string[]): Message[] {
  return [
    { method: "initialize", params: { positionEncodings: ["utf-8", "utf-16"], locale: "en" } },
    {
      method: "openProject",
      params: {
        configFileName: join(project, "tsconfig.json"),
        projectHandle: "pair",
        options: {},
        compilerOptions: COMPILER_OPTIONS,
      },
    },
    ...files.map((fileName) => ({
      method: "transform",
      params: { fileName, content: readFileSync(fileName, "utf8"), projectHandle: "pair" },
    })),
  ];
}

const parseNumber = (text: string | undefined): number | undefined => {
  const value = Number(text);
  return text !== undefined && Number.isFinite(value) ? value : undefined;
};

/** Run one burst replay: stdin is the framed stream file, stdout is hashed. */
function burst(build: Build, project: string, stream: string, scratch: string): BurstSample {
  const output = join(scratch, `${build.id}.out`);
  const metrics = join(scratch, `${build.id}.metrics`);
  const start = performance.now();
  let cpuSeconds: number | undefined;
  let processWallMs: number | undefined;
  let peakMiB: number | undefined;
  if (process.platform === "win32") {
    // Start-Process keeps the exited process object, which still answers its
    // processor time and peak working set. The stopwatch excludes PowerShell's
    // own startup, which would otherwise dominate the wall time.
    const script = [
      `$w = [System.Diagnostics.Stopwatch]::StartNew();`,
      `$p = Start-Process -FilePath '${build.bin}' -ArgumentList '${build.argv.join(" ")}'`,
      `-WorkingDirectory '${project}' -RedirectStandardInput '${stream}'`,
      `-RedirectStandardOutput '${output}' -NoNewWindow -PassThru -Wait;`,
      `$w.Stop(); "$($p.ExitCode) $($p.TotalProcessorTime.TotalSeconds) $($p.PeakWorkingSet64) $($w.Elapsed.TotalMilliseconds)"`,
    ].join(" ");
    const result = spawnSync("powershell.exe", ["-NoProfile", "-Command", script], {
      encoding: "utf8",
    });
    const [code, cpu, peak, elapsed] = result.stdout.trim().split(/\s+/);
    processWallMs = parseNumber(elapsed);
    if (result.status !== 0 || code !== "0")
      throw new Error(`${build.id} failed: ${result.stderr}`);
    cpuSeconds = parseNumber(cpu);
    peakMiB = (parseNumber(peak) ?? NaN) / 1048576;
  } else {
    const gnu = process.platform === "linux";
    const timer = gnu ? ["-f", "%U %S %M", "-o", metrics] : ["-l"];
    const input = openSync(stream, "r");
    const out = openSync(output, "w");
    const result = spawnSync("/usr/bin/time", [...timer, build.bin, ...build.argv], {
      cwd: project,
      stdio: [input, out, "pipe"],
      encoding: "utf8",
    });
    closeSync(input);
    closeSync(out);
    if (result.status !== 0) throw new Error(`${build.id} failed: ${result.stderr}`);
    if (gnu) {
      const [user, system, maxKiB] = readFileSync(metrics, "utf8").trim().split(/\s+/).map(Number);
      cpuSeconds = user + system;
      peakMiB = maxKiB / 1024;
    } else {
      const stderr = result.stderr;
      const user = parseNumber(/([\d.]+) user/.exec(stderr)?.[1]);
      const system = parseNumber(/([\d.]+) sys/.exec(stderr)?.[1]);
      cpuSeconds = user !== undefined && system !== undefined ? user + system : undefined;
      peakMiB =
        (parseNumber(/(\d+)\s+maximum resident set size/.exec(stderr)?.[1]) ?? NaN) / 1048576;
    }
  }
  const wallMs = processWallMs ?? performance.now() - start;
  const bytes = readFileSync(output);
  return {
    wallMs,
    cpuSeconds,
    peakMiB: Number.isFinite(peakMiB) ? peakMiB : undefined,
    errors: errorReplies(bytes),
    outputBytes: bytes.length,
    outputSha: createHash("sha256").update(bytes).digest("hex"),
  };
}

/**
 * Burst through a pipe, as the TypeScript host reads the mapper. On Windows a
 * redirected output file dominated the wall time (Start-Process to a file
 * 3.3 s, to NUL 0.33 s, for 5,000 SFCs), which no host pays.
 */
async function burstPipe(build: Build, project: string, stream: string): Promise<BurstSample> {
  const input = openSync(stream, "r");
  const start = performance.now();
  const child = spawn(build.bin, build.argv, {
    cwd: project,
    stdio: [input, "pipe", "inherit"],
  });
  const hash = createHash("sha256");
  let outputBytes = 0;
  let errors = 0;
  let carry: Buffer = Buffer.alloc(0);
  const stdout = child.stdout;
  if (stdout === null) throw new Error("content mapper stdout is not piped");
  stdout.on("data", (chunk: Buffer) => {
    hash.update(chunk);
    outputBytes += chunk.length;
    const window = Buffer.concat([carry, chunk]);
    errors += errorReplies(window);
    carry = window.subarray(Math.max(0, window.length - (ERROR_REPLY.length - 1)));
  });
  const code = await new Promise<number | null>((done) => child.on("close", done));
  const wallMs = performance.now() - start;
  closeSync(input);
  if (code !== 0) throw new Error(`${build.id} burst exited with ${code}`);
  return { wallMs, errors, outputBytes, outputSha: hash.digest("hex") };
}

const ERROR_REPLY = Buffer.from('"error":{');

/** JSON-RPC error replies in a run of response bytes. */
function errorReplies(bytes: Buffer): number {
  let count = 0;
  for (let at = bytes.indexOf(ERROR_REPLY); at >= 0; at = bytes.indexOf(ERROR_REPLY, at + 1))
    count++;
  return count;
}

/** Drive one transform at a time and time each reply. */
async function serial(build: Build, project: string, messages: Message[]): Promise<SerialSample> {
  const child = spawn(build.bin, build.argv, {
    cwd: project,
    stdio: ["pipe", "pipe", "inherit"],
  });
  const hash = createHash("sha256");
  let pending: Buffer = Buffer.alloc(0);
  let resolveReply: (() => void) | undefined;
  child.stdout.on("data", (chunk: Buffer) => {
    pending = pending.length === 0 ? chunk : Buffer.concat([pending, chunk]);
    for (;;) {
      const end = pending.indexOf("\r\n\r\n");
      if (end < 0) return;
      const length = Number(
        /Content-Length:\s*(\d+)/i.exec(pending.toString("ascii", 0, end))?.[1],
      );
      if (pending.length < end + 4 + length) return;
      hash.update(pending.subarray(end + 4, end + 4 + length));
      pending = pending.subarray(end + 4 + length);
      const done = resolveReply;
      resolveReply = undefined;
      done?.();
    }
  });
  const latencies: number[] = [];
  let id = 1;
  for (const message of messages) {
    const reply = new Promise<void>((done) => (resolveReply = done));
    const start = performance.now();
    child.stdin.write(frame({ jsonrpc: "2.0", id: id++, ...message }));
    await reply;
    if (message.method === "transform") latencies.push(performance.now() - start);
  }
  child.stdin.end();
  const code = await new Promise<number | null>((done) => child.on("close", done));
  if (code !== 0) throw new Error(`${build.id} serial session exited with ${code}`);
  return { latencies, outputSha: hash.digest("hex") };
}

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      bin: { type: "string", multiple: true },
      root: { type: "string" },
      sizes: { type: "string", default: "1000,5000" },
      runs: { type: "string", default: "5" },
      "serial-files": { type: "string", default: "300" },
      label: { type: "string", default: process.platform },
      out: { type: "string" },
      "allow-output-change": { type: "boolean", default: false },
      // id=<server.js>: a Node content mapper timed beside the builds, whose
      // answers differ by design and only have to be error-free.
      reference: { type: "string", multiple: true },
      // The TypeScript package a reference mapper resolves from the project.
      typescript: { type: "string" },
    },
  });
  const builds: Build[] = (values.bin ?? []).map((spec) => {
    const at = spec.indexOf("=");
    if (at <= 0) throw new Error(`--bin wants id=path, got ${spec}`);
    return {
      id: spec.slice(0, at),
      bin: resolve(spec.slice(at + 1)),
      argv: ["content-mapper"],
      reference: false,
    };
  });
  const references: Build[] = (values.reference ?? []).map((spec) => {
    const at = spec.indexOf("=");
    if (at <= 0) throw new Error(`--reference wants id=server.js, got ${spec}`);
    return {
      id: spec.slice(0, at),
      bin: process.execPath,
      argv: [resolve(spec.slice(at + 1))],
      reference: true,
    };
  });
  if (builds.length < 1 || !values.root || !values.out) {
    throw new Error("usage: --bin id=path [--bin id=path] --root <dir> --out <dir>");
  }
  const root = resolve(values.root);
  const out = resolve(values.out);
  const runs = Number(values.runs);
  const serialFiles = Number(values["serial-files"]);
  mkdirSync(out, { recursive: true });
  const cells: Cell[] = [];
  for (const sfcs of values.sizes.split(",").map(Number)) {
    const project = join(root, `vue-${sfcs}`);
    writeProject(project, generateScaleProject({ kind: "vue", sfcs }).files);
    if (values.typescript) {
      mkdirSync(join(project, "node_modules"), { recursive: true });
      symlinkSync(
        resolve(values.typescript),
        join(project, "node_modules", "typescript"),
        "junction",
      );
    }
    const files = sfcFiles(project);
    const messages = sessionMessages(project, files);
    const stream = join(root, `stream-${sfcs}.bin`);
    writeFileSync(
      stream,
      Buffer.concat(messages.map((m, i) => frame({ jsonrpc: "2.0", id: i + 1, ...m }))),
    );
    const scratch = join(root, `scratch-${sfcs}`);
    mkdirSync(scratch, { recursive: true });
    const all = [...builds, ...references];
    const bursts = new Map<string, BurstSample[]>(all.map((b) => [b.id, []]));
    const serials = new Map<string, SerialSample[]>(all.map((b) => [b.id, []]));
    // One unrecorded warm-up per build fills the page cache for every build.
    const measure = (build: Build) =>
      process.platform === "win32"
        ? burstPipe(build, project, stream)
        : Promise.resolve(burst(build, project, stream, scratch));
    for (const build of all) await measure(build);
    const serialMessages = messages.slice(0, 2 + Math.min(serialFiles, files.length));
    for (let run = 0; run < runs; run++) {
      const order = run % 2 === 0 ? all : [...all].reverse();
      for (const build of order) bursts.get(build.id)?.push(await measure(build));
      for (const build of order)
        serials.get(build.id)?.push(await serial(build, project, serialMessages));
    }
    const cell = summarizeCell({
      label: values.label,
      sfcs,
      serialFiles: serialMessages.length - 2,
      builds: builds.map((b) => b.id),
      references: references.map((b) => b.id),
      bursts,
      serials,
      allowOutputChange: values["allow-output-change"],
    });
    cells.push(cell);
    console.log(JSON.stringify(cell));
  }
  writeFileSync(join(out, `pair-${values.label}.json`), JSON.stringify(cells, null, 2));
  writeFileSync(join(out, `pair-${values.label}.md`), markdown(values.label, cells));
  process.stdout.write(markdown(values.label, cells));
  if (cells.some((cell) => cell.failure)) process.exitCode = 1;
}

await main();
