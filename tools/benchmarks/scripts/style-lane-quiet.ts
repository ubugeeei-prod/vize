// Measurement-only driver for the css/* style-lane comparison (bench branch, not for merge).
// node style-lane-quiet.ts --base-bin B --head-bin H --corpora name=dir,... --presets p,... --runs N --ir name,... --out FILE
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFileSync } from "node:fs";

const argv = process.argv.slice(2);
const arg = (name: string, fallback?: string): string => {
  const index = argv.indexOf(`--${name}`);
  if (index < 0) {
    if (fallback === undefined) throw new Error(`missing --${name}`);
    return fallback;
  }
  return argv[index + 1];
};
const bins = { base: arg("base-bin"), head: arg("head-bin") };
const runs = Number(arg("runs", "15"));
const out = arg("out");
const corpora = arg("corpora")
  .split(",")
  .map((entry) => entry.split("=") as [string, string]);
const presets = arg("presets", "incremental,opinionated,nuxt").split(",");
const irCorpora = new Set(arg("ir", "").split(",").filter(Boolean));
const write = (line = "") => {
  appendFileSync(out, line + "\n");
  console.log(line);
};

// Findings of one JSON report, as `file:line:column rule` keys.
function findings(stdout: string): string[] {
  try {
    const report = JSON.parse(stdout) as { file: string; messages: { ruleId: string; line: number; column: number }[] }[];
    return report.flatMap((file) => file.messages.map((m) => `${file.file}:${m.line}:${m.column} ${m.ruleId}`));
  } catch {
    return [];
  }
}

function diffFindings(cwd: string, args: string[]): string {
  const output = (binary: string) =>
    spawnSync(binary, args, { cwd, maxBuffer: 1 << 30 }).stdout?.toString() ?? "";
  const base = findings(output(bins.base));
  const head = findings(output(bins.head));
  const count = (xs: string[]) => {
    const map = new Map<string, number>();
    for (const x of xs) map.set(x, (map.get(x) ?? 0) + 1);
    return map;
  };
  const b = count(base);
  const h = count(head);
  const byRule = new Map<string, { added: number; removed: number }>();
  const examples: string[] = [];
  for (const key of new Set([...b.keys(), ...h.keys()])) {
    const delta = (h.get(key) ?? 0) - (b.get(key) ?? 0);
    if (delta === 0) continue;
    const rule = key.split(" ").at(-1) ?? "";
    const entry = byRule.get(rule) ?? { added: 0, removed: 0 };
    if (delta > 0) entry.added += delta;
    else entry.removed -= delta;
    byRule.set(rule, entry);
    if (examples.length < 6) examples.push(`${delta > 0 ? "+" : "-"}${key}`);
  }
  const rules = [...byRule].map(([rule, { added, removed }]) => `${rule} +${added}/-${removed}`).join(", ");
  return `${rules || "none"}${examples.length ? ` (e.g. ${examples.join("; ")})` : ""}`;
}

type Sample = { wall: number; cpu: number; rss: number; code: number | null; sha: string; timedOut: boolean };

function once(binary: string, cwd: string, args: string[]): Sample {
  const start = performance.now();
  const result = spawnSync("/usr/bin/time", ["-f", "%U %S %M", binary, ...args], {
    cwd,
    maxBuffer: 1 << 30,
    timeout: 600_000,
  });
  const wall = performance.now() - start;
  const last = (result.stderr?.toString() ?? "").trim().split("\n").at(-1) ?? "";
  const [user, sys, rss] = last.split(" ").map(Number);
  // The JSON report is the whole diagnostic output: identical bytes mean identical findings.
  const sha = createHash("sha256")
    .update(result.stdout ?? Buffer.alloc(0))
    .digest("hex")
    .slice(0, 12);
  return {
    wall,
    cpu: ((user || 0) + (sys || 0)) * 1000,
    rss: (rss || 0) / 1024,
    code: result.status,
    sha,
    timedOut: result.error !== undefined,
  };
}

const pct = (values: number[], q: number) => {
  const sorted = [...values].sort((a, b) => a - b);
  const k = (sorted.length - 1) * q;
  const f = Math.floor(k);
  const c = Math.min(f + 1, sorted.length - 1);
  return sorted[f] + (sorted[c] - sorted[f]) * (k - f);
};

function instructions(binary: string, cwd: string, args: string[]): number {
  const result = spawnSync(
    "valgrind",
    ["--tool=callgrind", "--callgrind-out-file=/dev/null", "--cache-sim=no", binary, ...args],
    { cwd, maxBuffer: 1 << 30, env: { ...process.env, RAYON_NUM_THREADS: "1" } },
  );
  const match = /Collected : (\d+)/.exec(result.stderr?.toString() ?? "");
  return match ? Number(match[1]) : NaN;
}

write("| corpus | preset | side | wall p50 / p95 (ms) | CPU p50 (ms) | peak RSS p50 (MiB) | exit | output sha | Ir (1 thread) |");
write("| --- | --- | --- | --- | --- | --- | --- | --- | --- |");
for (const [name, dir] of corpora) {
  for (const preset of presets) {
    const args = ["lint", ".", "--no-config", "--preset", preset, "--format", "json"];
    for (const role of ["base", "head"] as const) once(bins[role], dir, args);
    const samples: Record<"base" | "head", Sample[]> = { base: [], head: [] };
    for (let round = 0; round < runs; round++) {
      const order = round % 2 === 0 ? (["base", "head"] as const) : (["head", "base"] as const);
      for (const role of order) samples[role].push(once(bins[role], dir, args));
    }
    const ir = irCorpora.has(name)
      ? { base: instructions(bins.base, dir, args), head: instructions(bins.head, dir, args) }
      : null;
    const row = (role: "base" | "head") => {
      const xs = samples[role];
      const walls = xs.map((x) => x.wall);
      return `${pct(walls, 0.5).toFixed(1)} / ${pct(walls, 0.95).toFixed(1)} | ${pct(
        xs.map((x) => x.cpu),
        0.5,
      ).toFixed(0)} | ${pct(
        xs.map((x) => x.rss),
        0.5,
      ).toFixed(1)} | ${[...new Set(xs.map((x) => x.code))].join(",")} | ${[...new Set(xs.map((x) => x.sha))].join(",")} | ${
        ir ? ir[role].toLocaleString("en-US") : ""
      }`;
    };
    write(`| ${name} | ${preset} | base | ${row("base")} |`);
    write(`| | | head | ${row("head")} |`);
    const ratio = pct(samples.head.map((x) => x.wall), 0.5) / pct(samples.base.map((x) => x.wall), 0.5);
    const cpuRatio = pct(samples.head.map((x) => x.cpu), 0.5) / pct(samples.base.map((x) => x.cpu), 0.5);
    const irRatio = ir ? (ir.head / ir.base).toFixed(4) : "";
    write(`| | | head/base | wall ${ratio.toFixed(3)} | CPU ${cpuRatio.toFixed(3)} | | | | ${irRatio} |`);
    if (samples.base[0]?.sha !== samples.head[0]?.sha) {
      write(`| | | finding diff | ${diffFindings(dir, args)} | | | | | |`);
    }
  }
}
