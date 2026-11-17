// Measurement-only driver for quiet-runner lint/LSP comparisons (bench branch, not for merge).
// node perf-linter-quiet.ts --base-bin B --head-bin H --mk DIR --mkta DIR --bench DIR --runs N --lsp-rounds R --lsp-edits E --out FILE
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { appendFileSync, readFileSync } from "node:fs";
import { join } from "node:path";

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
const runs = Number(arg("runs", "30"));
const out = arg("out");
const write = (line = "") => {
  appendFileSync(out, line + "\n");
  console.log(line);
};

type Sample = { wall: number; cpu: number; user: number; sys: number; rss: number; code: number | null; sha: string; timedOut: boolean };

function once(binary: string, cwd: string, args: string[], env: Record<string, string> = {}): Sample {
  const start = performance.now();
  const result = spawnSync("/usr/bin/time", ["-f", "%U %S %M", binary, ...args], {
    cwd,
    env: { ...process.env, ...env },
    maxBuffer: 1 << 30,
    timeout: 180_000,
  });
  const wall = performance.now() - start;
  const stderr = result.stderr?.toString() ?? "";
  const last = stderr.trim().split("\n").at(-1) ?? "";
  const [user, sys, rss] = last.split(" ").map(Number);
  const stdout = (result.stdout?.toString() ?? "")
    .split("\n")
    .filter((line) => !line.startsWith("Linted "))
    .join("\n");
  return {
    wall,
    cpu: ((user || 0) + (sys || 0)) * 1000,
    user: (user || 0) * 1000,
    sys: (sys || 0) * 1000,
    rss: (rss || 0) / 1024,
    code: result.status,
    sha: createHash("sha256").update(stdout).digest("hex").slice(0, 16),
    timedOut: result.error !== undefined,
  };
}

// Exact instruction count of one run under Callgrind (no cache simulation), threads serialized.
function instructions(binary: string, cwd: string, args: string[]): number {
  const result = spawnSync(
    "valgrind",
    ["--tool=callgrind", "--callgrind-out-file=/dev/null", "--cache-sim=no", binary, ...args],
    { cwd, maxBuffer: 1 << 30, env: { ...process.env, RAYON_NUM_THREADS: "1" } },
  );
  const match = /Collected : (\d+)/.exec(result.stderr?.toString() ?? "");
  return match ? Number(match[1]) : NaN;
}

function irScenario(name: string, cwd: string, args: string[]) {
  const base = instructions(bins.base, cwd, args);
  const head = instructions(bins.head, cwd, args);
  write(`| ${name} | callgrind Ir, 1 thread | base | ${base.toLocaleString("en-US")} | | | | | |`);
  write(`| | | head | ${head.toLocaleString("en-US")} | | | | | |`);
  write(`| | | head/base | ${(head / base).toFixed(4)} (${(((head - base) / base) * 100).toFixed(2)} %) | | | | | |`);
}

const pct = (values: number[], q: number) => {
  const sorted = [...values].sort((a, b) => a - b);
  const k = (sorted.length - 1) * q;
  const f = Math.floor(k);
  const c = Math.min(f + 1, sorted.length - 1);
  return sorted[f] + (sorted[c] - sorted[f]) * (k - f);
};

function scenario(name: string, cwd: string, args: string[], count: number, env: Record<string, string> = {}) {
  for (const role of ["base", "head"] as const) once(bins[role], cwd, args, env);
  const samples: Record<"base" | "head", Sample[]> = { base: [], head: [] };
  for (let round = 0; round < count; round++) {
    const order = round % 2 === 0 ? (["base", "head"] as const) : (["head", "base"] as const);
    for (const role of order) samples[role].push(once(bins[role], cwd, args, env));
  }
  const row = (role: "base" | "head") => {
    const xs = samples[role];
    const walls = xs.map((x) => x.wall);
    const shas = [...new Set(xs.map((x) => x.sha))];
    return `${pct(walls, 0.5).toFixed(1)} / ${pct(walls, 0.95).toFixed(1)} / ${Math.max(...walls).toFixed(1)} ms | ${pct(
      xs.map((x) => x.cpu),
      0.5,
    ).toFixed(0)} ms (user ${pct(
      xs.map((x) => x.user),
      0.5,
    ).toFixed(0)} / sys ${pct(
      xs.map((x) => x.sys),
      0.5,
    ).toFixed(0)}) | ${pct(
      xs.map((x) => x.rss),
      0.5,
    ).toFixed(1)} MiB | ${[...new Set(xs.map((x) => x.code))].join(",")} | ${xs.filter((x) => x.timedOut).length} | ${shas.join(",")}`;
  };
  const ratio = pct(samples.head.map((x) => x.wall), 0.5) / pct(samples.base.map((x) => x.wall), 0.5);
  write(`| ${name} | ${count} | base | ${row("base")} |`);
  write(`| | | head | ${row("head")} |`);
  write(`| | | head/base p50 | ${ratio.toFixed(3)} | | | | | |`);
}

type Lsp = { open: number; latencies: number[]; sha: string };

function lspSession(binary: string, root: string, file: string, edits: number): Promise<Lsp> {
  return new Promise((resolve, reject) => {
    const child = spawn(binary, ["lsp"], { cwd: root, stdio: ["pipe", "pipe", "ignore"] });
    let buffer = Buffer.alloc(0);
    const queue: any[] = [];
    const waiters: ((message: any) => void)[] = [];
    child.stdout.on("data", (data: Buffer) => {
      buffer = Buffer.concat([buffer, data]);
      for (;;) {
        const separator = buffer.indexOf("\r\n\r\n");
        if (separator < 0) return;
        const length = Number(/content-length: *(\d+)/i.exec(buffer.subarray(0, separator).toString())?.[1]);
        if (buffer.length < separator + 4 + length) return;
        const message = JSON.parse(buffer.subarray(separator + 4, separator + 4 + length).toString());
        buffer = buffer.subarray(separator + 4 + length);
        if (message.method && message.id !== undefined) {
          send({ jsonrpc: "2.0", id: message.id, result: null });
          continue;
        }
        queue.push(message);
        while (waiters.length && queue.length) waiters.shift()!(queue.shift());
      }
    });
    child.on("error", reject);
    const send = (message: unknown) => {
      const body = Buffer.from(JSON.stringify(message));
      child.stdin.write(`Content-Length: ${body.length}\r\n\r\n`);
      child.stdin.write(body);
    };
    const next = () =>
      queue.length ? Promise.resolve(queue.shift()) : new Promise<any>((done) => waiters.push(done));
    const uri = "file://" + join(root, file);
    const waitDiagnostics = async (version: number) => {
      for (;;) {
        const message = await next();
        if (message.method === "textDocument/publishDiagnostics" && message.params.uri === uri) {
          const v = message.params.version;
          if (v == null || v >= version) return message.params.diagnostics;
        }
      }
    };
    (async () => {
      send({
        jsonrpc: "2.0",
        id: 1,
        method: "initialize",
        params: {
          processId: process.pid,
          rootUri: "file://" + root,
          capabilities: { textDocument: { publishDiagnostics: { versionSupport: true } } },
          workspaceFolders: [{ uri: "file://" + root, name: "r" }],
        },
      });
      for (;;) if ((await next()).id === 1) break;
      send({ jsonrpc: "2.0", method: "initialized", params: {} });
      const text = readFileSync(join(root, file), "utf8");
      let start = performance.now();
      send({
        jsonrpc: "2.0",
        method: "textDocument/didOpen",
        params: { textDocument: { uri, languageId: "vue", version: 1, text } },
      });
      await waitDiagnostics(1);
      const open = performance.now() - start;
      const latencies: number[] = [];
      let last: any[] = [];
      for (let i = 0; i < edits; i++) {
        const version = i + 2;
        start = performance.now();
        send({
          jsonrpc: "2.0",
          method: "textDocument/didChange",
          params: { textDocument: { uri, version }, contentChanges: [{ text: text + (i % 2 === 0 ? " " : "") }] },
        });
        const diagnostics = await waitDiagnostics(version);
        latencies.push(performance.now() - start);
        if (i % 2 === 1) last = diagnostics;
      }
      const canonical = JSON.stringify(last.map((d) => JSON.stringify(d, Object.keys(d).sort())).sort());
      child.kill();
      resolve({ open, latencies, sha: createHash("sha256").update(canonical).digest("hex").slice(0, 16) });
    })().catch(reject);
  });
}

async function lsp(root: string, file: string, rounds: number, edits: number) {
  const sessions: Record<"base" | "head", Lsp[]> = { base: [], head: [] };
  await lspSession(bins.base, root, file, 4);
  await lspSession(bins.head, root, file, 4);
  for (let round = 0; round < rounds; round++) {
    const order = round % 2 === 0 ? (["base", "head"] as const) : (["head", "base"] as const);
    for (const role of order) sessions[role].push(await lspSession(bins[role], root, file, edits));
  }
  write(`| LSP edit loop (${file}) | ${rounds}×${edits} | role | edit p50 / p95 (all edits) | open p50 | diagnostics sha |`);
  write("| --- | --- | --- | --- | --- | --- |");
  for (const role of ["base", "head"] as const) {
    const all = sessions[role].flatMap((s) => s.latencies);
    write(
      `| | | ${role} | ${pct(all, 0.5).toFixed(2)} / ${pct(all, 0.95).toFixed(2)} ms | ${pct(
        sessions[role].map((s) => s.open),
        0.5,
      ).toFixed(1)} ms | ${[...new Set(sessions[role].map((s) => s.sha))].join(",")} |`,
    );
  }
}

const mk = arg("mk");
const mkta = arg("mkta");
const bench = arg("bench");
write("| scenario | runs | role | wall p50 / p95 / max | CPU p50 | max RSS p50 | exit | timeouts | stdout sha |");
write("| --- | ---: | --- | --- | --- | --- | --- | ---: | --- |");
for (const spec of arg("scenarios", "mk,mk-json,mk-all,bench,ta").split(",")) {
  if (spec === "mk") scenario("misskey `lint src` (text)", mk, ["lint", "src"], runs);
  if (spec === "mk-json") scenario("misskey `lint src --format json`", mk, ["lint", "src", "--format", "json"], runs);
  if (spec === "mk-all")
    scenario("misskey `--preset all --format json`", mk, ["lint", "src", "--preset", "all", "--format", "json"], runs);
  if (spec === "mk-all-text") scenario("misskey `--preset all` (text)", mk, ["lint", "src", "--preset", "all"], runs);
  const threads = /^mk-json-t(\d+)$/.exec(spec);
  if (threads)
    scenario(`misskey \`--format json\`, RAYON_NUM_THREADS=${threads[1]}`, mk, ["lint", "src", "--format", "json"], runs, {
      RAYON_NUM_THREADS: threads[1],
    });
  // `mk-json-e<NAME>=<VALUE>+<NAME>=<VALUE>`: the same JSON lint with extra environment.
  const envSpec = /^mk-json-e(.+)$/.exec(spec);
  if (envSpec) {
    const env = Object.fromEntries(envSpec[1].split("+").map((pair) => pair.split("=") as [string, string]));
    scenario(`misskey \`--format json\`, ${envSpec[1]}`, mk, ["lint", "src", "--format", "json"], runs, env);
  }
  const benchThreads = /^bench-t(\d+)$/.exec(spec);
  if (benchThreads)
    scenario(`1000-SFC \`--format json\`, RAYON_NUM_THREADS=${benchThreads[1]}`, bench, ["lint", ".", "--format", "json"], runs, {
      RAYON_NUM_THREADS: benchThreads[1],
    });
  if (spec === "ir-mk") irScenario("misskey `--format json` instructions", mk, ["lint", "src", "--format", "json"]);
  if (spec === "ir-mk-text") irScenario("misskey text instructions", mk, ["lint", "src"]);
  if (spec === "ir-bench") irScenario("1000-SFC `--format json` instructions", bench, ["lint", ".", "--format", "json"]);
  if (spec === "ir-ta")
    irScenario("misskey type-aware instructions", mkta, ["lint", "src", "--strict-reactivity", "--format", "json"]);
  if (spec === "ver") scenario("`vize --version` (process start)", mk, ["--version"], runs);
  if (spec === "one") scenario("misskey one SFC `lint src/components/MkButton.vue`", mk, ["lint", "src/components/MkButton.vue"], runs);
  if (spec === "ir-ver") irScenario("`vize --version` instructions", mk, ["--version"]);
  if (spec === "ir-one") irScenario("one SFC instructions", mk, ["lint", "src/components/MkButton.vue"]);
  if (spec === "phases" || spec === "phases-json") {
    const extra = spec === "phases-json" ? ["--format", "json"] : [];
    for (let round = 0; round < 3; round++) {
      const result = spawnSync(bins.base, ["lint", "src", ...extra, "--profile"], { cwd: mk, maxBuffer: 1 << 30 });
      const text = `${result.stdout?.toString() ?? ""}\n${result.stderr?.toString() ?? ""}`.replace(/\x1b\[[0-9;]*m/g, "");
      const lines = text.split("\n").filter((line) => /Total wall time|^  (load config|collect files|lint wall|render output|untracked wall|cumulative work) /.test(line));
      write(["", `phases ${spec} round ${round}:`, "", "```", ...lines, "```", ""].join("\n"));
    }
  }
  if (spec === "bench") scenario("1000-SFC benchmark `lint .` (text)", bench, ["lint", "."], runs);
  if (spec === "ta")
    scenario(
      "misskey `--strict-reactivity --format json` (type-aware)",
      mkta,
      ["lint", "src", "--strict-reactivity", "--format", "json"],
      runs,
    );
}
if (arg("lsp-rounds", "0") !== "0") {
  write();
  await lsp(mk, "src/components/MkDrive.vue", Number(arg("lsp-rounds")), Number(arg("lsp-edits", "40")));
}
