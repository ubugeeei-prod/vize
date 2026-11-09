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
import { closeSync, mkdirSync, openSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
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

type Build = { id: string; bin: string };

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
  let peakMiB: number | undefined;
  if (process.platform === "win32") {
    // Start-Process keeps the exited process object, which still answers its
    // processor time and peak working set.
    const script = [
      `$p = Start-Process -FilePath '${build.bin}' -ArgumentList 'content-mapper'`,
      `-WorkingDirectory '${project}' -RedirectStandardInput '${stream}'`,
      `-RedirectStandardOutput '${output}' -NoNewWindow -PassThru -Wait;`,
      `"$($p.ExitCode) $($p.TotalProcessorTime.TotalSeconds) $($p.PeakWorkingSet64)"`,
    ].join(" ");
    const result = spawnSync("powershell.exe", ["-NoProfile", "-Command", script], {
      encoding: "utf8",
    });
    const [code, cpu, peak] = result.stdout.trim().split(/\s+/);
    if (result.status !== 0 || code !== "0")
      throw new Error(`${build.id} failed: ${result.stderr}`);
    cpuSeconds = parseNumber(cpu);
    peakMiB = (parseNumber(peak) ?? NaN) / 1048576;
  } else {
    const gnu = process.platform === "linux";
    const timer = gnu ? ["-f", "%U %S %M", "-o", metrics] : ["-l"];
    const input = openSync(stream, "r");
    const out = openSync(output, "w");
    const result = spawnSync("/usr/bin/time", [...timer, build.bin, "content-mapper"], {
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
  const wallMs = performance.now() - start;
  const bytes = readFileSync(output);
  return {
    wallMs,
    cpuSeconds,
    peakMiB: Number.isFinite(peakMiB) ? peakMiB : undefined,
    outputBytes: bytes.length,
    outputSha: createHash("sha256").update(bytes).digest("hex"),
  };
}

/** Drive one transform at a time and time each reply. */
async function serial(build: Build, project: string, messages: Message[]): Promise<SerialSample> {
  const child = spawn(build.bin, ["content-mapper"], {
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
    },
  });
  const builds: Build[] = (values.bin ?? []).map((spec) => {
    const at = spec.indexOf("=");
    if (at <= 0) throw new Error(`--bin wants id=path, got ${spec}`);
    return { id: spec.slice(0, at), bin: resolve(spec.slice(at + 1)) };
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
    const files = sfcFiles(project);
    const messages = sessionMessages(project, files);
    const stream = join(root, `stream-${sfcs}.bin`);
    writeFileSync(
      stream,
      Buffer.concat(messages.map((m, i) => frame({ jsonrpc: "2.0", id: i + 1, ...m }))),
    );
    const scratch = join(root, `scratch-${sfcs}`);
    mkdirSync(scratch, { recursive: true });
    const bursts = new Map<string, BurstSample[]>(builds.map((b) => [b.id, []]));
    const serials = new Map<string, SerialSample[]>(builds.map((b) => [b.id, []]));
    // One unrecorded warm-up per build fills the page cache for every build.
    for (const build of builds) burst(build, project, stream, scratch);
    const serialMessages = messages.slice(0, 2 + Math.min(serialFiles, files.length));
    for (let run = 0; run < runs; run++) {
      const order = run % 2 === 0 ? builds : [...builds].reverse();
      for (const build of order) bursts.get(build.id)?.push(burst(build, project, stream, scratch));
      for (const build of order)
        serials.get(build.id)?.push(await serial(build, project, serialMessages));
    }
    const cell = summarizeCell({
      label: values.label,
      sfcs,
      serialFiles: serialMessages.length - 2,
      builds: builds.map((b) => b.id),
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
