// Run a command in a pseudo-terminal and answer after a prompt appears.
// usage: node pty-command.ts PROMPT RESPONSE COMMAND [ARG ...]
import fs, { type Dir } from "node:fs";
import path from "node:path";
import { spawn, Terminal, type IPty } from "zigpty";

const gitFlags = new Set([
  "--no-pager",
  "--paginate",
  "-p",
  "-P",
  "--bare",
  "--no-replace-objects",
  "--literal-pathspecs",
  "--glob-pathspecs",
  "--noglob-pathspecs",
  "--icase-pathspecs",
  "--no-optional-locks",
  "--no-lazy-fetch",
  "--no-advice",
]);
const gitValues = new Set(["-C", "-c", "--git-dir", "--work-tree", "--namespace"]);
const gitAssignments = ["--git-dir=", "--work-tree=", "--namespace=", "--config-env="];

/** Skip known Git global options; refuse ambiguous option shapes. */
export function gitOperation(argv: readonly string[]): string | null {
  let index = 1;
  while (index < argv.length) {
    const argument = argv[index] ?? "";
    if (gitFlags.has(argument)) {
      index += 1;
    } else if (gitValues.has(argument)) {
      const value = argv[index + 1];
      if (value === undefined) return null;
      if (!value && argument !== "-C") return null;
      index += 2;
    } else if (gitAssignments.some((prefix) => argument.startsWith(prefix))) {
      if (!argument.slice(argument.indexOf("=") + 1)) return null;
      if (argument.startsWith("--config-env=")) {
        const fields = argument.split("=");
        if (fields.length !== 3 || !fields[1] || !fields[2]) return null;
      }
      index += 1;
    } else if ((argument.startsWith("-C") || argument.startsWith("-c")) && argument.length > 2) {
      index += 1;
    } else if (!argument || argument.startsWith("-")) {
      return null;
    } else {
      return argument;
    }
  }
  return null;
}

const knownBinaries = new Set([
  "vp",
  "node",
  "sh",
  "bash",
  "moon",
  "moonc",
  "moondoc",
  "gcc",
  "cc",
  "clang",
  "git",
  "git-remote-http",
  "git-remote-https",
  "ssh",
  "release.exe",
  "release",
]);
const knownOperations: Readonly<Record<string, readonly string[]>> = {
  moon: ["update", "run", "build"],
  git: [
    "pull",
    "fetch",
    "remote-http",
    "remote-https",
    "index-pack",
    "credential",
    "credential-cache",
    "maintenance",
    "config",
    "rev-parse",
  ],
};

/** Return only fixed labels, including Git's dispatched transport stages. */
export function commandPhase(argv: readonly string[]): string {
  const binary = path.basename(argv[0] ?? "");
  const phase = knownBinaries.has(binary) ? binary : "other";
  const operation = binary === "git" ? gitOperation(argv) : (argv[1] ?? null);
  if (operation !== null && knownOperations[binary]?.includes(operation))
    return `${phase}:${operation}`;
  return phase;
}

/** Retain only anonymous channel identities, never paths or socket peers. */
export function processChannels(
  processPath: string,
  openDirectory: (directory: string) => Dir = fs.opendirSync,
): Record<string, string> {
  const channels: Record<string, string> = {};
  let directory: Dir;
  try {
    directory = openDirectory(`${processPath}/fd`);
  } catch {
    return channels;
  }
  try {
    let count = 0;
    for (let examined = 0; examined < 256; examined += 1) {
      const entry = directory.readSync();
      if (entry === null) break;
      if (!/^\d+$/u.test(entry.name)) continue;
      try {
        const target = fs.readlinkSync(path.join(entry.parentPath, entry.name));
        if (target.startsWith("pipe:[") || target.startsWith("socket:[")) {
          channels[entry.name] = target;
          count += 1;
        }
      } catch {
        continue;
      }
      if (count >= 64) break;
    }
  } catch {
    // Unreadable descriptor tables keep whatever was already observed.
  } finally {
    directory.closeSync();
  }
  return channels;
}

interface ProcessRecord {
  readonly pid: number;
  readonly ppid: number;
  readonly pgrp: number;
  readonly session: number;
  readonly foreground: number;
  readonly state: string;
  readonly phase: string;
  readonly wait: string;
  readonly channels: Record<string, string>;
}

/** Fields after `(comm)` in /proc/PID/stat: state, ppid, pgrp, session, tty, tpgid. */
function statFields(pid: string): string[] {
  const stat = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
  return stat
    .slice(stat.lastIndexOf(")") + 1)
    .trim()
    .split(/\s+/u);
}

function sessionProcesses(session: number): ProcessRecord[] {
  const processes: ProcessRecord[] = [];
  for (const name of fs.readdirSync("/proc")) {
    if (!/^\d+$/u.test(name)) continue;
    try {
      const stat = statFields(name);
      if (Number(stat[3]) !== session) continue;
      const cmdline = Buffer.alloc(4096);
      const fd = fs.openSync(`/proc/${name}/cmdline`, "r");
      const length = fs.readSync(fd, cmdline, 0, cmdline.length, 0);
      fs.closeSync(fd);
      const argv = cmdline.subarray(0, length).toString("utf8").split("\0");
      processes.push({
        pid: Number(name),
        ppid: Number(stat[1]),
        pgrp: Number(stat[2]),
        session: Number(stat[3]),
        foreground: Number(stat[5]),
        state: stat[0] ?? "",
        phase: commandPhase(argv),
        wait: fs.readFileSync(`/proc/${name}/wchan`, "utf8").trim(),
        channels: processChannels(`/proc/${name}`),
      });
    } catch {
      continue;
    }
    if (processes.length >= 256) break;
  }
  return processes.sort((left, right) => left.pid - right.pid);
}

/** Bounded Linux process metadata; never record argv, input, or environment. */
class LifecycleEvidence {
  private readonly started = performance.now();
  private nextSnapshot = 0;
  private records = 0;
  private readonly destination: number | null;
  private readonly child: number;

  constructor(child: number) {
    this.child = child;
    const evidencePath = process.env.VIZE_PTY_EVIDENCE;
    this.destination = evidencePath
      ? fs.openSync(`${evidencePath}.${process.pid}.jsonl`, "wx")
      : null;
  }

  record(event: string, force = false, failure = false): void {
    const now = performance.now();
    if ((this.destination === null && !failure) || this.records >= 128) return;
    if (!force && now < this.nextSnapshot) return;
    this.nextSnapshot = now + 500;
    const linux = process.platform === "linux";
    let foreground: number | null = null;
    if (linux) {
      try {
        foreground = Number(statFields(String(this.child))[5]);
      } catch {
        foreground = null;
      }
    }
    const record = {
      event,
      elapsed: Math.round(now - this.started) / 1000,
      child: this.child,
      foreground,
      processes: linux ? sessionProcesses(this.child) : [],
    };
    if (this.destination !== null) fs.writeSync(this.destination, `${JSON.stringify(record)}\n`);
    if (failure) fs.writeSync(2, `PTY lifecycle: ${JSON.stringify(record)}\n`);
    this.records += 1;
  }

  close(): void {
    if (this.destination !== null) fs.closeSync(this.destination);
  }
}

function within(promise: Promise<unknown>, milliseconds: number): Promise<boolean> {
  return new Promise((resolve) => {
    const timer = setTimeout(() => resolve(false), milliseconds);
    void promise.then(() => {
      clearTimeout(timer);
      resolve(true);
    });
  });
}

function signalGroup(pid: number, signal: NodeJS.Signals | 0): boolean {
  try {
    process.kill(-pid, signal);
    return true;
  } catch {
    return false;
  }
}

async function terminateProcessGroup(child: IPty, exited: Promise<unknown>): Promise<void> {
  signalGroup(child.pid, "SIGTERM");
  await within(exited, 1000);
  if (!signalGroup(child.pid, 0)) return;
  signalGroup(child.pid, "SIGKILL");
  await within(exited, 1000);
}

/** Wait until the master has been quiet briefly, so output written just before exit is kept. */
async function outputSettled(
  lastOutput: () => number,
  quietMs = 100,
  limitMs = 2000,
): Promise<void> {
  const deadline = performance.now() + limitMs;
  while (performance.now() < deadline && performance.now() - lastOutput() < quietMs) {
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
}

async function main(argv: readonly string[]): Promise<number> {
  const [prompt, response, file, ...args] = argv;
  if (prompt === undefined || response === undefined || file === undefined) {
    fs.writeSync(2, "usage: pty-command.ts PROMPT RESPONSE COMMAND [ARG ...]\n");
    return 2;
  }
  const env: Record<string, string> = {};
  for (const [key, value] of Object.entries(process.env)) if (value !== undefined) env[key] = value;
  let lastOutput = performance.now();
  // A terminal-backed reader keeps draining the master after the child exits;
  // the plain reader is destroyed on exit and drops the last writes on Linux.
  const terminal = new Terminal({ data: (_terminal, data) => onOutput(Buffer.from(data)) });
  const child = spawn(file, args, { env, cwd: process.cwd(), terminal });
  const exited = new Promise<{ exitCode: number; signal: number }>((resolve) => {
    child.onExit(resolve);
  });
  const evidence = new LifecycleEvidence(child.pid);
  let output = Buffer.alloc(0);
  let answered = false;
  function onOutput(bytes: Buffer): void {
    lastOutput = performance.now();
    fs.writeSync(1, bytes);
    if (answered) return;
    output = Buffer.concat([output, bytes]);
    if (!output.includes(prompt)) return;
    evidence.record("prompt", true);
    child.write(response);
    answered = true;
    evidence.record("answered", true);
  }
  const snapshots = setInterval(() => evidence.record("waiting"), 100);
  const finished = await within(exited, 25_000);
  clearInterval(snapshots);
  if (!finished) {
    evidence.record("deadline", true, true);
    await terminateProcessGroup(child, exited);
    evidence.record("terminated", true);
    evidence.close();
    child.close();
    terminal.close();
    return 124;
  }
  const { exitCode, signal } = await exited;
  await outputSettled(() => lastOutput);
  evidence.record("child-exit", true);
  evidence.close();
  child.close();
  terminal.close();
  if (!answered) return 125;
  return signal ? 128 + signal : exitCode;
}

if (import.meta.main) process.exitCode = await main(process.argv.slice(2));
