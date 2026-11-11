/** Pure framing, identity checks and Markdown for `content-mapper-replay.ts`. */

import { createHash } from "node:crypto";

import { median, quantile } from "./content-mapper-pair-report.ts";

export type ReplayMapper = {
  id: string;
  command: string;
  args: string[];
  env?: NodeJS.ProcessEnv;
};

export type ReplayRequest = {
  /** `JSON.stringify(id)`, the key replies are paired by. */
  key: string;
  method: string;
  fileName?: string;
  requestBytes: number;
  frame: Buffer;
  notification: boolean;
};

export type ReplayStream = {
  label: string;
  file: string;
  cwd: string;
  bytes: Buffer;
  requests: ReplayRequest[];
};

export type ReplayRun = {
  /** A warm-up's replies are compared; its timing is not reported. */
  mode: "warmup" | "burst" | "serial";
  wallMs: number;
  cpuSeconds?: number;
  peakMiB?: number;
  outputBytes: number;
  errors: number;
  /** Reply body sha256 by request key. */
  hashes: Map<string, string>;
  replyBytes: Map<string, number>;
  /** burst: arrival since start; serial: write to reply. */
  latencies: Map<string, number>;
};

export type MapperSummary = {
  id: string;
  burstWallMs?: number;
  burstWallMin?: number;
  burstCpuSeconds?: number;
  burstPeakMiB?: number;
  serialTotalMs?: number;
  serialP50Ms?: number;
  serialP90Ms?: number;
  serialMaxMs?: number;
  serialCpuSeconds?: number;
  outputBytes: number;
  errors: number;
  digest: string;
  allocations?: Record<string, number>;
};

export type ReplaySummary = {
  label: string;
  requests: number;
  transforms: number;
  requestBytes: number;
  runs: number;
  mappers: MapperSummary[];
  /** Median serial latency of the slowest transforms, per mapper. */
  slowest: { fileName: string; replyBytes: number; ms: Record<string, number> }[];
  repliesIdentical: boolean;
  failure?: string;
};

/** Each frame of a stream with its body; a truncated tail is an error. */
export function bodyFrames(bytes: Buffer): { frame: Buffer; body: Buffer }[] {
  const frames: { frame: Buffer; body: Buffer }[] = [];
  let at = 0;
  while (at < bytes.length) {
    const end = bytes.indexOf("\r\n\r\n", at);
    if (end < 0) throw new Error(`stream ends inside a header at byte ${at}`);
    const header = bytes.toString("latin1", at, end);
    const length = Number(/Content-Length:\s*(\d+)/i.exec(header)?.[1]);
    if (!Number.isFinite(length)) throw new Error(`frame without Content-Length at byte ${at}`);
    const start = end + 4;
    if (start + length > bytes.length) throw new Error(`stream ends inside a body at byte ${at}`);
    frames.push({
      frame: bytes.subarray(at, start + length),
      body: bytes.subarray(start, start + length),
    });
    at = start + length;
  }
  return frames;
}

/** Digest of every reply in request order, independent of arrival order. */
export function replyDigest(
  requests: readonly ReplayRequest[],
  hashes: Map<string, string>,
): string {
  const hash = createHash("sha256");
  for (const request of requests) {
    if (request.notification) continue;
    hash.update(`${request.key}=${hashes.get(request.key) ?? "missing"}\n`);
  }
  return hash.digest("hex");
}

const defined = (values: (number | undefined)[]): number[] =>
  values.filter((value): value is number => value !== undefined && Number.isFinite(value));
const medianOf = (values: (number | undefined)[]): number | undefined => {
  const present = defined(values);
  return present.length ? median(present) : undefined;
};

export function summarizeReplay(input: {
  stream: ReplayStream;
  mappers: string[];
  results: Map<string, ReplayRun[]>;
  allocations?: Map<string, Record<string, number>>;
  slowest: number;
  allowOutputChange: boolean;
}): ReplaySummary {
  const { stream } = input;
  const answered = stream.requests.filter((request) => !request.notification);
  const transforms = answered.filter((request) => request.method === "transform");
  const digests = new Map<string, Set<string>>();
  const failures: string[] = [];
  for (const id of input.mappers) {
    const own = new Set<string>();
    for (const run of input.results.get(id) ?? []) {
      const missing = answered.filter((request) => !run.hashes.has(request.key));
      if (missing.length > 0)
        failures.push(`${id} ${run.mode} left ${missing.length} requests unanswered`);
      own.add(replyDigest(stream.requests, run.hashes));
    }
    digests.set(id, own);
    if (own.size > 1) failures.push(`${id} answered runs differently`);
  }
  const all = new Set([...digests.values()].flatMap((own) => [...own]));
  const repliesIdentical = all.size === 1;
  if (!repliesIdentical && !input.allowOutputChange && !failures.length) {
    failures.push(`mappers answered differently: ${firstDifference(input, answered)}`);
  }
  const mappers = input.mappers.map((id): MapperSummary => {
    const runs = input.results.get(id) ?? [];
    const bursts = runs.filter((run) => run.mode === "burst");
    const serials = runs.filter((run) => run.mode === "serial");
    const serialTotals = serials.map((run) =>
      transforms.reduce((sum, request) => sum + (run.latencies.get(request.key) ?? 0), 0),
    );
    const perRequest = transforms.map((request) =>
      median(defined(serials.map((run) => run.latencies.get(request.key)))),
    );
    const first = runs[0];
    return {
      id,
      burstWallMs: medianOf(bursts.map((run) => run.wallMs)),
      burstWallMin: bursts.length ? Math.min(...bursts.map((run) => run.wallMs)) : undefined,
      burstCpuSeconds: medianOf(bursts.map((run) => run.cpuSeconds)),
      burstPeakMiB: medianOf(bursts.map((run) => run.peakMiB)),
      serialTotalMs: medianOf(serialTotals),
      serialP50Ms: serials.length ? quantile(defined(perRequest), 0.5) : undefined,
      serialP90Ms: serials.length ? quantile(defined(perRequest), 0.9) : undefined,
      serialMaxMs: serials.length ? Math.max(...defined(perRequest)) : undefined,
      serialCpuSeconds: medianOf(serials.map((run) => run.cpuSeconds)),
      outputBytes: first?.outputBytes ?? 0,
      errors: Math.max(0, ...runs.map((run) => run.errors)),
      digest: [...(digests.get(id) ?? [])][0] ?? "",
      ...(input.allocations?.get(id) ? { allocations: input.allocations.get(id) } : {}),
    };
  });
  const lead = input.mappers[0] ?? "";
  const leadSerials = (input.results.get(lead) ?? []).filter((run) => run.mode === "serial");
  const slowest = transforms
    .map((request) => ({
      request,
      ms: median(defined(leadSerials.map((run) => run.latencies.get(request.key)))),
    }))
    .filter((entry) => Number.isFinite(entry.ms))
    .sort((a, b) => b.ms - a.ms)
    .slice(0, input.slowest)
    .map(({ request }) => ({
      fileName: request.fileName ?? request.key,
      replyBytes: (input.results.get(lead) ?? [])[0]?.replyBytes.get(request.key) ?? 0,
      ms: Object.fromEntries(
        input.mappers.map((id) => [
          id,
          median(
            defined(
              (input.results.get(id) ?? [])
                .filter((run) => run.mode === "serial")
                .map((run) => run.latencies.get(request.key)),
            ),
          ),
        ]),
      ),
    }));
  return {
    label: stream.label,
    requests: answered.length,
    transforms: transforms.length,
    requestBytes: stream.bytes.length,
    runs: Math.max(
      0,
      ...[...input.results.values()].map(
        (runs) => runs.filter((run) => run.mode !== "warmup").length,
      ),
    ),
    mappers,
    slowest,
    repliesIdentical,
    ...(failures.length ? { failure: failures.join("; ") } : {}),
  };
}

/** The first request two mappers answered with different bytes. */
function firstDifference(
  input: { mappers: string[]; results: Map<string, ReplayRun[]> },
  answered: readonly ReplayRequest[],
): string {
  const [lead, ...rest] = input.mappers;
  const leadRun = input.results.get(lead ?? "")?.[0];
  for (const id of rest) {
    const run = input.results.get(id)?.[0];
    for (const request of answered) {
      if (leadRun?.hashes.get(request.key) !== run?.hashes.get(request.key)) {
        return `${lead} vs ${id} at ${request.method} ${request.fileName ?? request.key}`;
      }
    }
  }
  return "unknown request";
}

const fixed = (value: number | undefined, digits = 0): string =>
  value === undefined || !Number.isFinite(value) ? "-" : value.toFixed(digits);

export function markdown(summaries: readonly ReplaySummary[]): string {
  const lines: string[] = [];
  for (const summary of summaries) {
    lines.push(
      `### ${summary.label}: ${summary.transforms} transforms, ${(summary.requestBytes / 1048576).toFixed(1)} MiB requests, ${summary.runs} runs per mapper`,
      "",
      summary.failure
        ? `**${summary.failure}**`
        : "Replies byte-identical across every run and mapper.",
      "",
      "| mapper | burst wall ms (min) | burst CPU s | peak MiB | serial total ms | serial p50 / p90 / max ms | serial CPU s | reply MiB | errors |",
      "|---|---|---|---|---|---|---|---|---|",
    );
    for (const mapper of summary.mappers) {
      lines.push(
        `| ${mapper.id} | ${fixed(mapper.burstWallMs)} (${fixed(mapper.burstWallMin)}) | ${fixed(mapper.burstCpuSeconds, 3)} | ${fixed(mapper.burstPeakMiB, 1)} | ${fixed(mapper.serialTotalMs)} | ${fixed(mapper.serialP50Ms, 2)} / ${fixed(mapper.serialP90Ms, 2)} / ${fixed(mapper.serialMaxMs, 1)} | ${fixed(mapper.serialCpuSeconds, 3)} | ${(mapper.outputBytes / 1048576).toFixed(2)} | ${mapper.errors} |`,
      );
    }
    const counted = summary.mappers.filter((mapper) => mapper.allocations);
    if (counted.length) {
      lines.push(
        "",
        "| mapper | allocations | allocated MiB | reallocs | realloc MiB |",
        "|---|---|---|---|---|",
      );
      for (const mapper of counted) {
        const a = mapper.allocations ?? {};
        lines.push(
          `| ${mapper.id} | ${a.allocationCalls ?? "-"} | ${fixed((a.allocatedBytes ?? NaN) / 1048576, 1)} | ${a.reallocCalls ?? "-"} | ${fixed((a.reallocNewBytes ?? NaN) / 1048576, 1)} |`,
        );
      }
    }
    if (summary.slowest.length) {
      const ids = summary.mappers.map((mapper) => mapper.id);
      lines.push(
        "",
        `| slowest serial transforms | reply KiB | ${ids.map((id) => `${id} ms`).join(" | ")} |`,
        `|---|---|${ids.map(() => "---").join("|")}|`,
      );
      for (const entry of summary.slowest) {
        lines.push(
          `| ${entry.fileName.split("/").slice(-3).join("/")} | ${(entry.replyBytes / 1024).toFixed(0)} | ${ids.map((id) => fixed(entry.ms[id], 2)).join(" | ")} |`,
        );
      }
    }
    lines.push("");
  }
  return `${lines.join("\n")}\n`;
}
