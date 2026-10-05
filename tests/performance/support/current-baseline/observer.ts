import assert from "node:assert/strict";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import type { VerifiedLspLaunch } from "../../../tooling/support/lsp/launch.ts";
import { baselineRoot, writeJson } from "./inputs.ts";
import { hash, MAX_WIRE_BYTES, type Chunk } from "./wire.ts";

export class BaselineObserver {
  output: string;
  chunks: Record<"client" | "server" | "stderr", Chunk[]> = { client: [], server: [], stderr: [] };
  lengths = { client: 0, server: 0, stderr: 0 };
  total = 0;
  spawnNs = "";
  failure: string | null = null;
  sampler?: ChildProcessWithoutNullStreams;
  closed: Promise<void> = Promise.resolve();
  launch: VerifiedLspLaunch;
  repoRoot: string;

  constructor(repoRoot: string, launch: VerifiedLspLaunch, resolveLaunchMs: number) {
    this.output = baselineRoot()!;
    this.repoRoot = repoRoot;
    this.launch = launch;
    assert.equal(process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
    assert.equal(process.platform, "linux");
    assert(
      fs.existsSync(path.join(this.output, "fixture.json")),
      "input custody must precede spawn",
    );
    writeJson(path.join(this.output, "launch.json"), {
      ...launch,
      resolveLaunchMs,
      resolveLaunchScope:
        "source receipt/binary hashing plus existing version probe; before timed server spawn",
    });
    for (const stream of ["client", "server", "stderr"]) {
      fs.writeFileSync(path.join(this.output, `${stream}.bin`), "", { flag: "wx" });
    }
    fs.writeFileSync(path.join(this.output, "chunks.jsonl"), "", { flag: "wx" });
    fs.writeFileSync(path.join(this.output, "cycles.jsonl"), "", { flag: "wx" });
  }

  beforeSpawn(): void {
    this.spawnNs = process.hrtime.bigint().toString();
  }

  attach(child: ChildProcessWithoutNullStreams): void {
    assert(child.pid, "actual server PID is required");
    const stat = fs.readFileSync(`/proc/${child.pid}/stat`, "utf8");
    const ticks = stat.slice(stat.lastIndexOf(")") + 2).split(" ")[19];
    assert.match(ticks, /^\d+$/);
    writeJson(path.join(this.output, "process-start.json"), {
      pid: child.pid,
      startTicks: ticks,
      spawnNs: this.spawnNs,
    });
    child.stdout.on("data", (bytes: Buffer) => this.append("server", bytes));
    child.stderr.on("data", (bytes: Buffer) => this.append("stderr", bytes));
    child.on("error", (error) => {
      this.failure = error.message;
    });
    const sampler = spawn(
      "python3",
      [
        path.join(this.repoRoot, "tests/performance/support/current-baseline/sample_rss.py"),
        String(child.pid),
        ticks,
        path.join(this.output, "rss.jsonl"),
      ],
      { stdio: ["pipe", "pipe", "pipe"] },
    );
    this.sampler = sampler;
    sampler.stdout.on("data", (bytes: Buffer) =>
      fs.appendFileSync(path.join(this.output, "sampler.stdout"), bytes),
    );
    sampler.stderr.on("data", (bytes: Buffer) =>
      fs.appendFileSync(path.join(this.output, "sampler.stderr"), bytes),
    );
    let samplerError: string | null = null;
    sampler.on("error", (error) => {
      samplerError = error.message;
    });
    sampler.stdin.on("error", (error) => {
      samplerError ??= error.message;
    });
    const stopped = new Promise<{ code: number | null; signal: string | null }>((resolve) =>
      sampler.once("close", (code, signal) => resolve({ code, signal })),
    );
    this.closed = new Promise<void>((resolve) =>
      child.once("close", (code, signal) => {
        sampler.stdin.end("stop\n");
        const deadline = setTimeout(() => sampler.kill("SIGKILL"), 3000);
        void stopped.then((status) => {
          clearTimeout(deadline);
          writeJson(path.join(this.output, "process.json"), {
            exitStatus: code,
            signal,
            processError: this.failure,
            sampler: { ...status, error: samplerError },
            streams: Object.fromEntries(
              Object.keys(this.chunks).map((key) => {
                const stream = key as keyof typeof this.chunks;
                return [
                  stream,
                  {
                    bytes: this.lengths[stream],
                    sha256: hash(fs.readFileSync(path.join(this.output, `${stream}.bin`))),
                  },
                ];
              }),
            ),
          });
          resolve();
        });
      }),
    );
  }

  append(stream: keyof BaselineObserver["chunks"], bytes: Buffer): void {
    const ns = process.hrtime.bigint().toString();
    if (bytes.length === 0) return;
    if (this.total + bytes.length > MAX_WIRE_BYTES) {
      this.failure ??= "complete raw observation exceeds 16MiB quota";
      this.sampler?.stdin.end("stop\n");
      return;
    }
    const chunk = { start: this.lengths[stream], end: this.lengths[stream] + bytes.length, ns };
    fs.appendFileSync(path.join(this.output, `${stream}.bin`), bytes);
    fs.appendFileSync(
      path.join(this.output, "chunks.jsonl"),
      `${JSON.stringify({ stream, ...chunk })}\n`,
    );
    this.lengths[stream] = chunk.end;
    this.total += bytes.length;
    this.chunks[stream].push(chunk);
  }

  cycle(value: unknown): void {
    fs.appendFileSync(path.join(this.output, "cycles.jsonl"), `${JSON.stringify(value)}\n`);
  }
}

let current: BaselineObserver | undefined;
export function newBaselineObserver(
  repoRoot: string,
  launch?: VerifiedLspLaunch,
  resolveLaunchMs = 0,
): BaselineObserver | undefined {
  if (!baselineRoot()) return undefined;
  assert(launch, "baseline refuses non-source launch/fallback");
  assert.equal(current, undefined, "exactly one server per fresh baseline session");
  current = new BaselineObserver(repoRoot, launch, resolveLaunchMs);
  return current;
}
export const baselineObserver = (): BaselineObserver | undefined => current;
