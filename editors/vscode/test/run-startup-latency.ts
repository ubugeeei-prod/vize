/**
 * VS Code time to first Vize diagnostic, alternating extension builds on one
 * generated workspace. Each run starts VS Code with a fresh profile whose
 * release-server cache already holds the given server, as a marketplace
 * install does after its first download.
 *
 *   node editors/vscode/test/run-startup-latency.ts generate --root <dir> \
 *     [--layout root|nested] [--sfcs 2000] [--noise 30000] [--node-modules 10000]
 *   node editors/vscode/test/run-startup-latency.ts run --code <VS Code executable> \
 *     --server <vize> --workspace <dir> --extension base=<dir> --extension head=<dir> \
 *     [--rounds 10] [--out <dir>]
 */

import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

const probe = path.join(path.dirname(fileURLToPath(import.meta.url)), "suite/startup-latency.cjs");
const TARGET_FILE = "bench-target.txt";

const LINT_APP = `<script setup lang="ts">
const items = [1, 2, 3];
</script>

<template>
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
`;

function component(index: number): string {
  return `<script setup lang="ts">
const props = defineProps<{ label: string; count?: number }>();
const doubled = (props.count ?? ${index}) * 2;
</script>

<template>
  <span :data-index="${index}">{{ props.label }} {{ doubled }}</span>
</template>
`;
}

function tsconfig(include: string[]): string {
  const compilerOptions = {
    strict: true,
    module: "ESNext",
    moduleResolution: "Bundler",
    target: "ESNext",
    noEmit: true,
  };
  return `${JSON.stringify({ compilerOptions, include }, null, 2)}\n`;
}

/** A monorepo-shaped tree: SFCs, ordinary files the search walks, and node_modules it skips. */
function generate(root: string, layout: string, sfcs: number, noise: number, modules: number) {
  if (layout !== "root" && layout !== "nested") throw new Error(`unknown layout ${layout}`);
  fs.rmSync(root, { recursive: true, force: true });
  const write = (file: string, text: string) => {
    const target = path.join(root, file);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, text);
  };
  const packages = Math.max(1, Math.ceil(sfcs / 200));
  const name = (index: number) => `pkg-${String(index).padStart(3, "0")}`;
  write("package.json", `${JSON.stringify({ name: "startup-bench", private: true })}\n`);
  if (layout === "root") write("tsconfig.json", tsconfig(["src", "packages/*/src"]));
  const target = layout === "root" ? "src/App.vue" : `packages/${name(0)}/src/App.vue`;
  write(target, LINT_APP);
  for (let index = 0; index < sfcs; index++) {
    const owner = name(index % packages);
    write(`packages/${owner}/src/components/Component${index}.vue`, component(index));
  }
  for (let index = 0; index < packages; index++) {
    write(`packages/${name(index)}/package.json`, `${JSON.stringify({ name: name(index) })}\n`);
    write(`packages/${name(index)}/tsconfig.json`, tsconfig(["src"]));
  }
  for (let index = 0; index < noise; index++) {
    const owner = name(index % packages);
    const bucket = Math.floor(index / 100);
    write(`packages/${owner}/fixtures/case-${bucket}/sample-${index}.json`, `{"id":${index}}\n`);
  }
  for (let index = 0; index < modules; index++) {
    const bucket = Math.floor(index / 100);
    write(`node_modules/dep-${bucket}/lib/module-${index}.js`, `module.exports = ${index};\n`);
  }
  write(TARGET_FILE, `${target}\n`);
}

const releaseTargets: Record<string, string> = {
  "darwin-arm64": "aarch64-apple-darwin",
  "darwin-x64": "x86_64-apple-darwin",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "linux-x64": "x86_64-unknown-linux-gnu",
  "win32-arm64": "aarch64-pc-windows-msvc",
  "win32-x64": "x86_64-pc-windows-msvc",
};

interface Run {
  label: string;
  round: number;
  shown: number;
  activated: number;
  firstDiagnostic: number;
  discoveryEnter?: number;
  discoveryReturn?: number;
  clientStartEnter?: number;
  clientStartFinish?: number;
}

function findLog(directory: string): string | undefined {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      const found = findLog(file);
      if (found) return found;
    } else if (/^\d+-Vize\.log$/.test(entry.name)) {
      return file;
    }
  }
  return undefined;
}

/** Offsets of the extension's own trace lines from the probe's open request. */
function traceOffsets(log: string, epoch: number): Partial<Run> {
  const offsets: Partial<Run> = {};
  const marks: Array<[keyof Run, RegExp]> = [
    ["discoveryEnter", /\] findFiles enter /],
    ["discoveryReturn", /\] findFiles (?:return|refused|error) /],
    ["clientStartEnter", /\] client start enter /],
    ["clientStartFinish", /\] client start finish /],
  ];
  for (const line of log.split(/\r?\n/)) {
    const stamp = /^\[config ([^\]]+)\]/.exec(line);
    if (!stamp) continue;
    for (const [key, pattern] of marks) {
      if (offsets[key] === undefined && pattern.test(line)) {
        (offsets as Record<string, number>)[key] = Date.parse(stamp[1]) - epoch;
      }
    }
  }
  return offsets;
}

interface Launch {
  code: string;
  extension: string;
  workspace: string;
  server: string;
  version: string;
  file: string;
  profiles: string;
}

async function launch({
  code,
  extension,
  workspace,
  server,
  version,
  file,
  profiles,
}: Launch): Promise<{ result: Record<string, number>; log: string }> {
  fs.mkdirSync(profiles, { recursive: true });
  const profile = fs.mkdtempSync(path.join(profiles, "vize-startup-"));
  try {
    const userData = path.join(profile, "user-data");
    const target = releaseTargets[`${process.platform}-${process.arch}`];
    if (!target) throw new Error(`no release target for ${process.platform}-${process.arch}`);
    const cached = path.join(
      userData,
      "User/globalStorage/ubugeeei.vize/servers",
      version,
      target,
      path.basename(server),
    );
    fs.mkdirSync(path.dirname(cached), { recursive: true });
    // One executable path for every run: a fresh copy would time the first-exec
    // malware and code-signing scans that an installed cache pays only once.
    try {
      if (process.platform === "win32") fs.linkSync(server, cached);
      else fs.symlinkSync(server, cached);
    } catch {
      fs.copyFileSync(server, cached);
      fs.chmodSync(cached, 0o755);
    }
    fs.writeFileSync(
      path.join(userData, "User/settings.json"),
      JSON.stringify({
        "chat.disableAIFeatures": true,
        "extensions.autoCheckUpdates": false,
        "extensions.autoUpdate": false,
        "telemetry.telemetryLevel": "off",
        "update.mode": "none",
        "workbench.startupEditor": "none",
      }),
    );
    const resultPath = path.join(profile, "result.json");
    const args = [
      "--no-sandbox",
      "--disable-gpu-sandbox",
      "--disable-updates",
      "--skip-welcome",
      "--skip-release-notes",
      "--no-cached-data",
      "--disable-workspace-trust",
      "--disable-extensions",
      `--extensionTestsPath=${probe}`,
      `--extensionDevelopmentPath=${extension}`,
      `--user-data-dir=${userData}`,
      `--extensions-dir=${path.join(profile, "extensions")}`,
      workspace,
    ];
    const env = {
      ...process.env,
      VIZE_BENCH_FILE: file,
      VIZE_BENCH_RESULT: resultPath,
      VIZE_TEST_ENABLE_HOST_COMMANDS: "1",
    };
    const shell = process.platform === "win32" && code.endsWith(".cmd");
    const child = spawn(shell ? `"${code}"` : code, shell ? args.map((arg) => `"${arg}"`) : args, {
      env,
      shell,
      stdio: ["ignore", "ignore", "pipe"],
    });
    let stderr = "";
    child.stderr.on("data", (chunk: Buffer) => {
      stderr = (stderr + chunk.toString()).slice(-4096);
    });
    const status = await new Promise<number | null>((resolve) => child.on("close", resolve));
    const logPath = findLog(path.join(userData, "logs"));
    const log = logPath ? fs.readFileSync(logPath, "utf8") : "";
    if (status !== 0 || !fs.existsSync(resultPath)) {
      throw new Error(`VS Code exited ${status}:\n${stderr}\n${log.slice(-4096)}`);
    }
    if (!log.includes("Using release server")) {
      throw new Error(`the run did not use the seeded release server:\n${log.slice(-4096)}`);
    }
    return { result: JSON.parse(fs.readFileSync(resultPath, "utf8")), log };
  } finally {
    try {
      fs.rmSync(profile, { recursive: true, force: true, maxRetries: 10 });
    } catch (error) {
      // A Windows server that is still exiting can hold its executable briefly.
      process.stderr.write(`profile cleanup failed: ${String(error)}\n`);
    }
  }
}

function median(values: number[]): number {
  const sorted = values.toSorted((left, right) => left - right);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

function report(runs: Run[], labels: string[], title: string): string {
  const metrics: Array<[string, (run: Run) => number | undefined]> = [
    ["extension active (client started)", (run) => run.activated],
    ["first Vize diagnostic", (run) => run.firstDiagnostic],
    ["config discovery returned", (run) => run.discoveryReturn],
    ["client start entered", (run) => run.clientStartEnter],
    [
      "config discovery duration",
      (run) =>
        run.discoveryReturn !== undefined && run.discoveryEnter !== undefined
          ? run.discoveryReturn - run.discoveryEnter
          : undefined,
    ],
  ];
  const lines = [
    `### ${title}`,
    "",
    `Milliseconds after the SFC open request; median (min–max) of ${runs.length / labels.length} alternating runs per build.`,
    "",
    `| metric | ${labels.join(" | ")} |`,
    `|---|${labels.map(() => "---:").join("|")}|`,
  ];
  for (const [name, read] of metrics) {
    const cells = labels.map((label) => {
      const values = runs
        .filter((run) => run.label === label)
        .map(read)
        .filter((value): value is number => typeof value === "number");
      if (values.length === 0) return "–";
      const round = (value: number) => Math.round(value);
      return `${round(median(values))} (${round(Math.min(...values))}–${round(Math.max(...values))})`;
    });
    lines.push(`| ${name} | ${cells.join(" | ")} |`);
  }
  return `${lines.join("\n")}\n`;
}

async function main() {
  const [command, ...rest] = process.argv.slice(2);
  const { values } = parseArgs({
    args: rest,
    options: {
      root: { type: "string" },
      layout: { type: "string", default: "root" },
      sfcs: { type: "string", default: "2000" },
      noise: { type: "string", default: "30000" },
      "node-modules": { type: "string", default: "10000" },
      code: { type: "string" },
      server: { type: "string" },
      workspace: { type: "string" },
      extension: { type: "string", multiple: true },
      rounds: { type: "string", default: "10" },
      out: { type: "string" },
      title: { type: "string", default: "VS Code startup" },
      profiles: { type: "string", default: os.tmpdir() },
    },
  });
  if (command === "generate") {
    if (!values.root) throw new Error("--root is required");
    generate(
      path.resolve(values.root),
      values.layout,
      Number(values.sfcs),
      Number(values.noise),
      Number(values["node-modules"]),
    );
    return;
  }
  if (command !== "run") throw new Error("usage: run-startup-latency.ts generate|run ...");
  const { code, server, workspace, out } = values;
  if (!code || !server || !workspace)
    throw new Error("--code, --server and --workspace are required");
  const builds = (values.extension ?? []).map((entry) => {
    const separator = entry.indexOf("=");
    if (separator <= 0) throw new Error(`--extension expects label=dir: ${entry}`);
    const directory = path.resolve(entry.slice(separator + 1));
    const manifest = JSON.parse(fs.readFileSync(path.join(directory, "package.json"), "utf8"));
    return { label: entry.slice(0, separator), directory, version: String(manifest.version) };
  });
  if (builds.length === 0) throw new Error("at least one --extension is required");
  if (new Set(builds.map((build) => build.version)).size !== 1)
    throw new Error("every extension build must declare the same version");
  const file = fs.readFileSync(path.join(workspace, TARGET_FILE), "utf8").trim();
  const rounds = Number(values.rounds);
  const runs: Run[] = [];
  for (let round = 0; round < rounds; round++) {
    const order = round % 2 === 0 ? builds : builds.toReversed();
    for (const build of order) {
      const request: Launch = {
        code,
        extension: build.directory,
        workspace: path.resolve(workspace),
        server: path.resolve(server),
        version: build.version,
        file,
        profiles: path.resolve(values.profiles),
      };
      let session;
      // A VS Code launch that fails before the probe ran is retried once.
      try {
        session = await launch(request);
      } catch (error) {
        process.stderr.write(`retrying ${build.label} round ${round}: ${String(error)}\n`);
        session = await launch(request);
      }
      const { result, log } = session;
      if (out) {
        fs.mkdirSync(path.join(out, "logs"), { recursive: true });
        fs.writeFileSync(path.join(out, "logs", `${build.label}-${round}.log`), log);
      }
      const run: Run = {
        label: build.label,
        round,
        shown: result.shown,
        activated: result.activated,
        firstDiagnostic: result.firstDiagnostic,
        ...traceOffsets(log, result.epoch),
      };
      runs.push(run);
      process.stderr.write(`${JSON.stringify(run)}\n`);
    }
  }
  const markdown = report(
    runs,
    builds.map((build) => build.label),
    values.title,
  );
  process.stdout.write(markdown);
  if (out) {
    fs.mkdirSync(out, { recursive: true });
    fs.writeFileSync(path.join(out, "startup.md"), markdown);
    fs.writeFileSync(path.join(out, "startup.json"), `${JSON.stringify(runs, null, 2)}\n`);
  }
}

await main();
