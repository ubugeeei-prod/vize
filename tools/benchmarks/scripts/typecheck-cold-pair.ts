/**
 * `vize check` pair: two `vize` builds check the same generated projects cold.
 *
 * Every timed run gets a fresh, empty cache root (`VIZE_CANON_CACHE_ROOT`,
 * `XDG_CACHE_HOME`), so each run builds and writes the whole mirror and
 * starts the checker from nothing. Runs rotate the build order after one
 * unrecorded warm-up per build fills the page cache. A cell whose builds or
 * runs report different diagnostics publishes no timing unless
 * `--allow-output-change` is set.
 *
 * Linux and WSL2 report the tree's CPU and largest resident set (GNU time)
 * plus sampled high-water marks of vize and its checker children; Windows
 * reports vize.exe's own CPU and peak working set. `--strace` adds one traced
 * run per build that attributes filesystem calls to process, syscall and path
 * (mirror, sources, node_modules, checker install).
 *
 * Usage:
 *   node tools/benchmarks/scripts/typecheck-cold-pair.ts \
 *     --bin base=/tmp/vize-base --bin head=target/ci-opt/vize \
 *     --root /tmp/cold-pair --projects vue:1000,vue:5000,monorepo:5000 \
 *     --runs 5 --label linux-ext4 --out /tmp/cold-pair-out [--strace]
 */

import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  type Dirent,
  createReadStream,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { createInterface } from "node:readline";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import {
  type Attribution,
  type Cell,
  type CheckSample,
  type PathRoots,
  StraceAttribution,
  attributionTable,
  markdown,
  processTotals,
  summarizeCell,
} from "./typecheck-cold-pair-report.ts";
import { type ScaleKind, generateScaleProject, writeProject } from "./scale-project-generator.ts";

type Build = { id: string; bin: string };
type Project = { name: string; dir: string; tsconfig: string };

const REPO = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const windows = process.platform === "win32";
const linux = process.platform === "linux";

/** A catalog pin from pnpm-workspace.yaml: `  <catalog>:` then `    "<name>": "<version>"`. */
export function catalogVersion(source: string, catalog: string, name: string): string {
  // A Windows checkout spells the file with CRLF.
  const workspace = source.replaceAll("\r\n", "\n");
  const block = new RegExp(`^  ${catalog}:\\n((?:    .*\\n)+)`, "mu").exec(workspace)?.[1];
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  const version = block && new RegExp(`^    "?${escaped}"?:\\s*"([^"]+)"`, "mu").exec(block)?.[1];
  if (!version) throw new Error(`no ${name} pin in catalog ${catalog}`);
  return version;
}

function npmInstall(dir: string, dependencies: Record<string, string>): void {
  writeFileSync(
    join(dir, "package.json"),
    `${JSON.stringify({ private: true, dependencies }, null, 2)}\n`,
  );
  const result = spawnSync(
    "npm",
    ["install", "--no-audit", "--no-fund", "--ignore-scripts", "--no-package-lock"],
    { cwd: dir, encoding: "utf8", shell: windows, timeout: 900_000 },
  );
  if (result.status !== 0) throw new Error(`npm install failed in ${dir}\n${result.stderr}`);
}

/** The pinned tsgo, installed once beside the projects. */
function installChecker(root: string, workspace: string): string {
  const platform = `${process.platform}-${process.arch}`;
  const name = `@typescript/typescript-${platform}`;
  const dir = join(root, "checker");
  mkdirSync(dir, { recursive: true });
  npmInstall(dir, { [name]: catalogVersion(workspace, "corsa-runtime", name) });
  const tsgo = join(dir, "node_modules", ...name.split("/"), "lib", windows ? "tsc.exe" : "tsc");
  if (!existsSync(tsgo)) throw new Error(`tsgo missing at ${tsgo}`);
  return tsgo;
}

function prepareProject(root: string, spec: string, vue: string): Project {
  const [kind = "", count = ""] = spec.split(":");
  if (!["vue", "monorepo"].includes(kind) || !/^\d+$/u.test(count)) {
    throw new Error(`--projects wants vue:<sfcs> or monorepo:<sfcs>, got ${spec}`);
  }
  const name = `${kind}-${count}`;
  const dir = join(root, name);
  const project = generateScaleProject({ kind: kind as ScaleKind, sfcs: Number(count) });
  writeProject(dir, project.files);
  npmInstall(dir, { vue });
  return { name, dir, tsconfig: join(dir, project.tsconfig ?? "tsconfig.json") };
}

/** Files and bytes a run left in its cache root: the materialized mirror. */
function treeSize(dir: string): { files: number; bytes: number } {
  let files = 0;
  let bytes = 0;
  const walk = (current: string) => {
    let entries: Dirent[];
    try {
      entries = readdirSync(current, { withFileTypes: true });
    } catch {
      return;
    }
    for (const entry of entries) {
      const path = join(current, entry.name);
      if (entry.isDirectory()) walk(path);
      else if (entry.isFile()) {
        files += 1;
        bytes += statSync(path).size;
      }
    }
  };
  walk(dir);
  return { files, bytes };
}

const checkArgs = (project: Project, tsgo: string) => [
  "check",
  "--format",
  "json",
  "--quiet",
  "--tsconfig",
  project.tsconfig,
  "--corsa-path",
  tsgo,
];

function freshCache(cacheRoot: string, tag: string): { dir: string; env: NodeJS.ProcessEnv } {
  const dir = join(cacheRoot, tag);
  rmSync(dir, { recursive: true, force: true });
  mkdirSync(join(dir, "xdg"), { recursive: true });
  return {
    dir,
    env: {
      ...process.env,
      VIZE_CANON_CACHE_ROOT: join(dir, "canon"),
      XDG_CACHE_HOME: join(dir, "xdg"),
    },
  };
}

const kib = (status: string, field: string): number | undefined => {
  const value = new RegExp(`^${field}:\\s+(\\d+) kB`, "mu").exec(status)?.[1];
  return value === undefined ? undefined : Number(value);
};

/** Poll /proc for the high-water marks of vize and of every checker it spawned. */
function sampleTree(rootPid: number): { stop: () => { vize?: number; children?: number } } {
  const peaks = new Map<number, { name: string; hwm: number }>();
  // Each thread lists the children it spawned, and vize spawns from workers.
  const children = (pid: number): number[] => {
    try {
      return readdirSync(`/proc/${pid}/task`).flatMap((task) => {
        try {
          return readFileSync(`/proc/${pid}/task/${task}/children`, "utf8")
            .trim()
            .split(/\s+/u)
            .filter(Boolean)
            .map(Number);
        } catch {
          return [];
        }
      });
    } catch {
      return [];
    }
  };
  const tick = () => {
    const stack = [rootPid];
    while (stack.length) {
      const pid = stack.pop() ?? 0;
      try {
        const status = readFileSync(`/proc/${pid}/status`, "utf8");
        const name = /^Name:\s+(\S+)/mu.exec(status)?.[1] ?? "";
        const hwm = kib(status, "VmHWM") ?? 0;
        if (hwm >= (peaks.get(pid)?.hwm ?? 0)) peaks.set(pid, { name, hwm });
      } catch {
        // exited between listing and reading
      }
      stack.push(...children(pid));
    }
  };
  const timer = setInterval(tick, 20);
  return {
    stop: () => {
      clearInterval(timer);
      let vize: number | undefined;
      let sum = 0;
      for (const { name, hwm } of peaks.values()) {
        if (name === "time") continue;
        if (name.startsWith("vize")) vize = Math.max(vize ?? 0, hwm);
        else sum += hwm;
      }
      return { vize: vize === undefined ? undefined : vize / 1024, children: sum / 1024 };
    },
  };
}

async function runOnce(
  build: Build,
  project: Project,
  tsgo: string,
  root: string,
  cacheRoot: string,
  tag: string,
): Promise<CheckSample> {
  const cache = freshCache(cacheRoot, tag);
  const scratch = join(root, "scratch");
  mkdirSync(scratch, { recursive: true });
  const output = join(scratch, `${tag}.json`);
  const metrics = join(scratch, `${tag}.time`);
  const start = performance.now();
  let exitCode: number;
  let cpuSeconds: number | undefined;
  let peakMiB: number | undefined;
  let vizeHwmMiB: number | undefined;
  let childHwmMiB: number | undefined;
  let processWallMs: number | undefined;
  if (windows) {
    const quote = (value: string) => `'${value.replace(/'/gu, "''")}'`;
    const args = checkArgs(project, tsgo)
      .map((arg) => `"${arg}"`)
      .join(",");
    // The process object's own start and exit times give the wall time, so
    // PowerShell's startup and the polling below stay outside it. An exited
    // process no longer answers its peak working set, so vize and its checker
    // children are sampled while they run.
    const script = [
      `$env:VIZE_CANON_CACHE_ROOT = ${quote(cache.env.VIZE_CANON_CACHE_ROOT ?? "")};`,
      `$p = Start-Process -FilePath ${quote(build.bin)} -ArgumentList @(${args})`,
      `-WorkingDirectory ${quote(project.dir)} -RedirectStandardOutput ${quote(output)}`,
      `-RedirectStandardError ${quote(`${output}.err`)} -NoNewWindow -PassThru;`,
      "$vizePeak = 0; $children = @{};",
      "while (-not $p.HasExited) {",
      "  $p.Refresh(); if ($p.PeakWorkingSet64 -gt $vizePeak) { $vizePeak = $p.PeakWorkingSet64 };",
      '  foreach ($c in Get-CimInstance Win32_Process -Filter "ParentProcessId=$($p.Id)") {',
      "    $kib = [int64]$c.PeakWorkingSetSize; if (-not $children.ContainsKey($c.ProcessId) -or $children[$c.ProcessId] -lt $kib) { $children[$c.ProcessId] = $kib } };",
      "  Start-Sleep -Milliseconds 250 };",
      "$p.WaitForExit();",
      "$childKiB = 0; foreach ($v in $children.Values) { $childKiB += $v };",
      "@{ exit = $p.ExitCode; cpu = $p.TotalProcessorTime.TotalSeconds; vizePeak = $vizePeak;",
      "   childKiB = $childKiB; wallMs = ($p.ExitTime - $p.StartTime).TotalMilliseconds } | ConvertTo-Json -Compress",
    ].join(" ");
    const result = spawnSync("powershell.exe", ["-NoProfile", "-Command", script], {
      encoding: "utf8",
      env: cache.env,
    });
    const measured = JSON.parse(result.stdout.trim().split(/\r?\n/u).pop() ?? "{}") as {
      exit?: number;
      cpu?: number;
      vizePeak?: number;
      childKiB?: number;
      wallMs?: number;
    };
    processWallMs = measured.wallMs;
    exitCode = measured.exit ?? -1;
    cpuSeconds = measured.cpu;
    vizeHwmMiB = (measured.vizePeak ?? NaN) / 1048576;
    childHwmMiB = (measured.childKiB ?? NaN) / 1024;
    peakMiB = Math.max(vizeHwmMiB, childHwmMiB);
  } else {
    const timer = linux ? ["-f", "%U %S %M", "-o", metrics] : ["-l"];
    const child = spawn("/usr/bin/time", [...timer, build.bin, ...checkArgs(project, tsgo)], {
      cwd: project.dir,
      env: cache.env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const sampler = linux && child.pid ? sampleTree(child.pid) : undefined;
    const chunks: Buffer[] = [];
    let stderr = "";
    child.stdout.on("data", (chunk: Buffer) => chunks.push(chunk));
    child.stderr.on("data", (chunk: Buffer) => (stderr += chunk.toString()));
    exitCode = await new Promise<number>((done) => child.on("close", (code) => done(code ?? -1)));
    const peaks = sampler?.stop();
    vizeHwmMiB = peaks?.vize;
    childHwmMiB = peaks?.children;
    writeFileSync(output, Buffer.concat(chunks));
    if (linux) {
      // GNU time first notes a non-zero exit (vize reports diagnostics with 1).
      const last = readFileSync(metrics, "utf8").trim().split("\n").pop() ?? "";
      const [user, system, maxKiB] = last.trim().split(/\s+/u).map(Number);
      cpuSeconds = (user ?? NaN) + (system ?? NaN);
      peakMiB = (maxKiB ?? NaN) / 1024;
    } else {
      const user = Number(/([\d.]+) user/u.exec(stderr)?.[1]);
      const system = Number(/([\d.]+) sys/u.exec(stderr)?.[1]);
      cpuSeconds = user + system;
      peakMiB = Number(/(\d+)\s+maximum resident set size/u.exec(stderr)?.[1]) / 1048576;
    }
  }
  const wallMs = processWallMs ?? performance.now() - start;
  const bytes = readFileSync(output);
  const mirror = treeSize(cache.dir);
  rmSync(cache.dir, { recursive: true, force: true });
  return {
    wallMs,
    cpuSeconds: Number.isFinite(cpuSeconds) ? cpuSeconds : undefined,
    peakMiB: Number.isFinite(peakMiB) ? peakMiB : undefined,
    vizeHwmMiB,
    childHwmMiB,
    exitCode,
    outputBytes: bytes.length,
    outputSha: createHash("sha256").update(bytes).digest("hex"),
    mirrorFiles: mirror.files,
    mirrorBytes: mirror.bytes,
  };
}

/** One traced cold run: filesystem calls by process, syscall and path class. */
async function traceOnce(
  build: Build,
  project: Project,
  tsgo: string,
  cacheRoot: string,
  out: string,
): Promise<Attribution> {
  const tag = `strace-${project.name}-${build.id}`;
  const cache = freshCache(cacheRoot, tag);
  const log = join(out, `${tag}.log`);
  // Process creation is traced too, so every thread joins its process.
  const calls = [
    "%file,read,pread64,write,pwrite64,getdents64,close,fsync,fdatasync,ftruncate,fchmod,flock",
    "clone,clone3,fork,vfork",
  ].join(",");
  const result = spawnSync(
    "strace",
    [
      "-f",
      "-y",
      "-qq",
      "--seccomp-bpf",
      "-e",
      "signal=none",
      "-e",
      `trace=${calls}`,
      "-o",
      log,
      build.bin,
      ...checkArgs(project, tsgo),
    ],
    { cwd: project.dir, env: cache.env, stdio: ["ignore", "ignore", "pipe"], encoding: "utf8" },
  );
  if (result.status !== 0 && result.status !== 1) {
    throw new Error(`strace ${build.id} failed: ${result.stderr}`);
  }
  const roots: PathRoots = {
    mirror: cache.dir,
    modules: join(project.dir, "node_modules"),
    project: project.dir,
    checker: dirname(dirname(dirname(dirname(tsgo)))),
  };
  // A 5,000-SFC trace outgrows one string; stream it line by line.
  const attribution = new StraceAttribution(roots);
  for await (const line of createInterface({ input: createReadStream(log), crlfDelay: Infinity })) {
    attribution.push(line);
  }
  rmSync(log, { force: true });
  rmSync(cache.dir, { recursive: true, force: true });
  return attribution.finish();
}

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      bin: { type: "string", multiple: true },
      root: { type: "string" },
      "cache-root": { type: "string" },
      projects: { type: "string", default: "vue:1000,vue:5000,monorepo:5000" },
      runs: { type: "string", default: "5" },
      label: { type: "string", default: process.platform },
      out: { type: "string" },
      strace: { type: "boolean", default: false },
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
  const cacheRoot = resolve(values["cache-root"] ?? join(root, "cache"));
  const runs = Number(values.runs);
  mkdirSync(root, { recursive: true });
  mkdirSync(out, { recursive: true });
  const workspace = readFileSync(join(REPO, "pnpm-workspace.yaml"), "utf8");
  const vue = catalogVersion(workspace, "vue-beta", "vue");
  const tsgo = installChecker(root, workspace);
  const cells: Cell[] = [];
  const traces: string[] = [];
  for (const spec of values.projects.split(",")) {
    const project = prepareProject(root, spec, vue);
    const samples = new Map<string, CheckSample[]>(builds.map((b) => [b.id, []]));
    for (const build of builds)
      await runOnce(build, project, tsgo, root, cacheRoot, `warm-${build.id}`);
    for (let run = 0; run < runs; run++) {
      const order = run % 2 === 0 ? builds : [...builds].reverse();
      for (const build of order) {
        const sample = await runOnce(
          build,
          project,
          tsgo,
          root,
          cacheRoot,
          `${project.name}-${build.id}-${run}`,
        );
        samples.get(build.id)?.push(sample);
      }
    }
    const cell = summarizeCell({
      label: values.label,
      project: project.name,
      builds: builds.map((b) => b.id),
      samples,
      allowOutputChange: values["allow-output-change"],
    });
    cells.push(cell);
    console.log(JSON.stringify(cell));
    if (values.strace && linux) {
      const traced: [string, Attribution][] = [];
      for (const build of builds) {
        traced.push([build.id, await traceOnce(build, project, tsgo, cacheRoot, out)]);
      }
      writeFileSync(
        join(out, `strace-${values.label}-${project.name}.json`),
        JSON.stringify(Object.fromEntries(traced.map(([id, c]) => [id, Object.fromEntries(c)]))),
      );
      traces.push(
        `#### ${project.name}: filesystem calls per process\n\n${processTotals(traced)}\n` +
          `#### ${project.name}: by syscall and path\n\n${attributionTable(traced)}`,
      );
    }
    rmSync(project.dir, { recursive: true, force: true });
  }
  const report = markdown(values.label, cells, windows) + traces.join("\n");
  writeFileSync(join(out, `cold-pair-${values.label}.json`), JSON.stringify(cells, null, 2));
  writeFileSync(join(out, `cold-pair-${values.label}.md`), report);
  process.stdout.write(report);
  if (cells.some((cell) => cell.failure)) process.exitCode = 1;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) await main();
