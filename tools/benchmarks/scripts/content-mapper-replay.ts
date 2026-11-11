/**
 * Content-mapper stream replay: mappers answer request streams a TypeScript
 * host recorded on a real project, and every reply is compared by id.
 *
 * A stream is the exact bytes the host wrote to a mapper's stdin
 * (`initialize`, `openProject`, `transform`…), recorded with the byte
 * probe's `VIZE_MAPPER_PROBE_STREAM`. Each run starts a fresh mapper in the
 * stream's project directory, so imported type sources resolve as they did.
 *
 * - burst: the whole stream is on stdin at once, as `tsc` queues a program.
 *   Reports wall, CPU (user+sys of the mapper process) and peak RSS.
 * - serial: one request at a time; reports each transform's reply latency
 *   (write to last reply byte), so slow files can be named.
 *
 * Every run of every mapper must answer every request with the same bytes
 * (replies are paired by id, so request-order and ready-order mappers
 * compare). A mapper built with `--features profiling` also reports its
 * allocation counters through `VIZE_CONTENT_MAPPER_ALLOC_REPORT`.
 *
 * Usage:
 *   node tools/benchmarks/scripts/content-mapper-replay.ts \
 *     --stream misskey=/tmp/misskey.bin@/tmp/proj/misskey \
 *     --mapper base=/tmp/vize-base --mapper head=target/ci-opt/vize \
 *     --mapper launcher=node:npm/cli/bin/vize \
 *     --ready --runs 5 --out /tmp/replay-out
 */

import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { closeSync, mkdirSync, openSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { parseArgs } from "node:util";

import {
  type ReplayMapper,
  type ReplayRequest,
  type ReplayRun,
  type ReplayStream,
  bodyFrames,
  markdown,
  summarizeReplay,
} from "./content-mapper-replay-report.ts";

/**
 * `id=path` runs a native `vize`; `id=node:path` runs a JS launcher under
 * this Node. `ready` asks a native mapper for replies as each is ready, as
 * the npm launcher does; request order otherwise.
 */
export function parseMapper(spec: string, ready = false): ReplayMapper {
  const at = spec.indexOf("=");
  if (at <= 0) throw new Error(`--mapper wants id=path or id=node:path, got ${spec}`);
  const id = spec.slice(0, at);
  const target = spec.slice(at + 1);
  if (target.startsWith("node:")) {
    return { id, command: process.execPath, args: [resolve(target.slice(5)), "content-mapper"] };
  }
  const args = ready ? ["content-mapper", "--reply-when-ready"] : ["content-mapper"];
  return { id, command: resolve(target), args };
}

/** `label=stream.bin@projectDir`. */
export function parseStream(spec: string): ReplayStream {
  const at = spec.indexOf("=");
  const cwdAt = spec.lastIndexOf("@");
  if (at <= 0 || cwdAt <= at) throw new Error(`--stream wants label=file@dir, got ${spec}`);
  const file = resolve(spec.slice(at + 1, cwdAt));
  const bytes = readFileSync(file);
  const requests: ReplayRequest[] = bodyFrames(bytes).map(({ frame, body }) => {
    const message = JSON.parse(body.toString("utf8")) as {
      id?: unknown;
      method?: string;
      params?: { fileName?: string; content?: string };
    };
    return {
      key: JSON.stringify(message.id ?? null),
      method: message.method ?? "",
      fileName: message.params?.fileName,
      requestBytes: frame.length,
      frame,
      notification: message.id === undefined,
    };
  });
  return { label: spec.slice(0, at), file, cwd: resolve(spec.slice(cwdAt + 1)), bytes, requests };
}

type Rusage = { cpuSeconds?: number; peakMiB?: number };

/** `/usr/bin/time` around the mapper: GNU on Linux, BSD `-l` on macOS. */
function timed(mapper: ReplayMapper, metrics: string): { command: string; args: string[] } {
  if (process.platform === "linux") {
    return {
      command: "/usr/bin/time",
      args: ["-f", "%U %S %M", "-o", metrics, mapper.command, ...mapper.args],
    };
  }
  if (process.platform === "darwin") {
    return { command: "/usr/bin/time", args: ["-l", mapper.command, ...mapper.args] };
  }
  return { command: mapper.command, args: mapper.args };
}

function rusage(metrics: string, stderr: string): Rusage {
  if (process.platform === "linux") {
    const [user, system, maxKiB] = readFileSync(metrics, "utf8").trim().split(/\s+/).map(Number);
    return { cpuSeconds: (user ?? NaN) + (system ?? NaN), peakMiB: (maxKiB ?? NaN) / 1024 };
  }
  if (process.platform === "darwin") {
    const user = Number(/([\d.]+) user/.exec(stderr)?.[1]);
    const system = Number(/([\d.]+) sys/.exec(stderr)?.[1]);
    const peak = Number(/(\d+)\s+maximum resident set size/.exec(stderr)?.[1]);
    return { cpuSeconds: user + system, peakMiB: peak / 1048576 };
  }
  return {};
}

const RESULT_ID = /^\{"id":("(?:[^"\\]|\\.)*"|-?\d+|null),"jsonrpc"/;

/** The request key of a reply body, without decoding a large result. */
export function replyKey(body: Buffer): { key: string; error: boolean } {
  // Sorted keys put `id` first in a result, and `error` first in an error.
  const head = RESULT_ID.exec(body.toString("utf8", 0, Math.min(body.length, 512)));
  if (head?.[1] !== undefined) return { key: JSON.stringify(JSON.parse(head[1])), error: false };
  const reply = JSON.parse(body.toString("utf8")) as { id?: unknown; error?: unknown };
  return { key: JSON.stringify(reply.id ?? null), error: reply.error !== undefined };
}

/**
 * Replies as they arrive, read like a host would: chunks are joined only
 * once a frame is complete, and bodies are hashed after the run, so the
 * reader never holds the mapper's stdout back.
 */
function replyCollector(onReply?: (key: string) => void) {
  let chunks: Buffer[] = [];
  let buffered = 0;
  let need = 0;
  const bodies = new Map<string, Buffer>();
  const arrivals = new Map<string, number>();
  let outputBytes = 0;
  let errors = 0;
  const push = (chunk: Buffer) => {
    outputBytes += chunk.length;
    chunks.push(chunk);
    buffered += chunk.length;
    if (buffered < need) return;
    let pending = chunks.length === 1 ? (chunks[0] as Buffer) : Buffer.concat(chunks, buffered);
    for (;;) {
      const end = pending.indexOf("\r\n\r\n");
      if (end < 0) break;
      const length = Number(
        /Content-Length:\s*(\d+)/i.exec(pending.toString("latin1", 0, end))?.[1],
      );
      if (!Number.isFinite(length)) throw new Error("mapper reply without Content-Length");
      if (pending.length < end + 4 + length) {
        need = end + 4 + length;
        break;
      }
      need = 0;
      const now = performance.now();
      const body = pending.subarray(end + 4, end + 4 + length);
      pending = pending.subarray(end + 4 + length);
      const { key, error } = replyKey(body);
      if (error) errors++;
      if (bodies.has(key)) throw new Error(`mapper answered id ${key} twice`);
      bodies.set(key, body);
      arrivals.set(key, now);
      onReply?.(key);
    }
    chunks = pending.length ? [pending] : [];
    buffered = pending.length;
  };
  const done = () => {
    const hashes = new Map<string, string>();
    const bytes = new Map<string, number>();
    for (const [key, body] of bodies) {
      hashes.set(key, createHash("sha256").update(body).digest("hex"));
      bytes.set(key, body.length);
    }
    return { outputBytes, errors, hashes, replyBytes: bytes };
  };
  return { push, arrivals, done };
}

async function exit(child: ReturnType<typeof spawn>, label: string): Promise<string> {
  let stderr = "";
  child.stderr?.on("data", (chunk: Buffer) => (stderr += chunk.toString("utf8")));
  const code = await new Promise<number | null>((done) => child.on("close", done));
  if (code !== 0) throw new Error(`${label} exited with ${code}: ${stderr.slice(-2000)}`);
  return stderr;
}

async function burst(
  mapper: ReplayMapper,
  stream: ReplayStream,
  scratch: string,
): Promise<ReplayRun> {
  const metrics = join(scratch, `${mapper.id}.metrics`);
  const { command, args } = timed(mapper, metrics);
  const input = openSync(stream.file, "r");
  const start = performance.now();
  const child = spawn(command, args, {
    cwd: stream.cwd,
    stdio: [input, "pipe", "pipe"],
    env: mapper.env,
  });
  closeSync(input);
  const replies = replyCollector();
  child.stdout?.on("data", replies.push);
  const stderr = await exit(child, `${mapper.id} burst on ${stream.label}`);
  const wallMs = performance.now() - start;
  return {
    mode: "burst",
    wallMs,
    ...rusage(metrics, stderr),
    ...replies.done(),
    latencies: new Map([...replies.arrivals].map(([key, at]) => [key, at - start])),
  };
}

async function serial(
  mapper: ReplayMapper,
  stream: ReplayStream,
  scratch: string,
): Promise<ReplayRun> {
  const metrics = join(scratch, `${mapper.id}.serial.metrics`);
  const { command, args } = timed(mapper, metrics);
  const start = performance.now();
  const child = spawn(command, args, {
    cwd: stream.cwd,
    stdio: ["pipe", "pipe", "pipe"],
    env: mapper.env,
  });
  let waiting: { key: string; done: () => void } | undefined;
  const replies = replyCollector((key) => {
    if (waiting?.key === key) {
      const { done } = waiting;
      waiting = undefined;
      done();
    }
  });
  child.stdout?.on("data", replies.push);
  const closed = exit(child, `${mapper.id} serial on ${stream.label}`);
  const latencies = new Map<string, number>();
  const stdin = child.stdin;
  if (!stdin) throw new Error("mapper stdin is not piped");
  for (const request of stream.requests) {
    const sent = performance.now();
    if (request.notification) {
      stdin.write(request.frame);
      continue;
    }
    const reply = new Promise<void>((done) => (waiting = { key: request.key, done }));
    stdin.write(request.frame);
    await reply;
    latencies.set(request.key, (replies.arrivals.get(request.key) ?? NaN) - sent);
  }
  stdin.end();
  const stderr = await closed;
  return {
    mode: "serial",
    wallMs: performance.now() - start,
    ...rusage(metrics, stderr),
    ...replies.done(),
    latencies,
  };
}

/** Counters a `--features profiling` mapper writes at exit; absent otherwise. */
function allocationReport(path: string): Record<string, number> | undefined {
  try {
    return JSON.parse(readFileSync(path, "utf8")) as Record<string, number>;
  } catch {
    return undefined;
  }
}

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      stream: { type: "string", multiple: true },
      mapper: { type: "string", multiple: true },
      runs: { type: "string", default: "5" },
      modes: { type: "string", default: "burst,serial" },
      out: { type: "string" },
      "allow-output-change": { type: "boolean", default: false },
      slowest: { type: "string", default: "10" },
      // Native mappers answer as each reply is ready, as under the npm launcher.
      ready: { type: "boolean", default: false },
      // id=KEY=VALUE: an environment variable for one mapper only.
      env: { type: "string", multiple: true },
    },
  });
  const streams = (values.stream ?? []).map(parseStream);
  const mappers = (values.mapper ?? []).map((spec) => parseMapper(spec, values.ready));
  for (const spec of values.env ?? []) {
    const [id, key, ...rest] = spec.split("=");
    const mapper = mappers.find((candidate) => candidate.id === id);
    if (!mapper || !key || rest.length === 0)
      throw new Error(`--env wants id=KEY=VALUE, got ${spec}`);
    mapper.env = { ...(mapper.env ?? process.env), [key]: rest.join("=") };
  }
  if (streams.length === 0 || mappers.length === 0 || !values.out) {
    throw new Error("usage: --stream label=file@dir --mapper id=path [--mapper ...] --out <dir>");
  }
  const runs = Number(values.runs);
  const modes = values.modes.split(",") as ("burst" | "serial")[];
  const out = resolve(values.out);
  mkdirSync(out, { recursive: true });
  const scratch = join(tmpdir(), `vize-replay-${process.pid}`);
  mkdirSync(scratch, { recursive: true });
  const summaries = [];
  for (const stream of streams) {
    const results = new Map<string, ReplayRun[]>(mappers.map((m) => [m.id, []]));
    const allocations = new Map<string, Record<string, number>>();
    // One untimed warm-up per mapper fills the page cache for all of them,
    // and a profiling build reports its allocation counters from it. Its
    // replies still join the identity check.
    for (const mapper of mappers) {
      const report = join(scratch, `${mapper.id}.alloc.json`);
      rmSync(report, { force: true });
      const warmup = await burst(
        {
          ...mapper,
          env: { ...(mapper.env ?? process.env), VIZE_CONTENT_MAPPER_ALLOC_REPORT: report },
        },
        stream,
        scratch,
      );
      results.get(mapper.id)?.push({ ...warmup, mode: "warmup" });
      const counters = allocationReport(report);
      if (counters) allocations.set(mapper.id, counters);
    }
    for (let run = 0; run < runs; run++) {
      const order = run % 2 === 0 ? mappers : [...mappers].reverse();
      for (const mode of modes) {
        for (const mapper of order) {
          const sample = await (mode === "burst" ? burst : serial)(mapper, stream, scratch);
          results.get(mapper.id)?.push(sample);
        }
      }
    }
    const summary = summarizeReplay({
      stream,
      mappers: mappers.map((m) => m.id),
      results,
      allocations,
      slowest: Number(values.slowest),
      allowOutputChange: values["allow-output-change"],
    });
    summaries.push(summary);
    console.error(`${stream.label}: ${summary.failure ?? "replies identical"}`);
  }
  rmSync(scratch, { recursive: true, force: true });
  writeFileSync(join(out, "replay.json"), JSON.stringify(summaries, null, 2));
  writeFileSync(join(out, "replay.md"), markdown(summaries));
  process.stdout.write(markdown(summaries));
  if (summaries.some((summary) => summary.failure)) process.exitCode = 1;
}

if (import.meta.main) await main();
