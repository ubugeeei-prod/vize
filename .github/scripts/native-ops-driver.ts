import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  closeSync,
  existsSync,
  globSync,
  lstatSync,
  mkdirSync,
  openSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  realpathSync,
  renameSync,
  statSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { controlProgram, gdbProgram } from "./native-ops-data.ts";

interface RootExit {
  readonly pid: number;
  readonly exitCode: number | null;
  readonly signal: string | null;
  readonly signalNumber: number | null;
}
interface Summary {
  readonly qualified: false;
  readonly rootExit: RootExit | null;
  readonly observerGaps: readonly unknown[];
  readonly remainingInferiors: readonly number[];
}
interface Event {
  readonly event: string;
  readonly monotonicNs: number;
  readonly pid?: number;
  readonly signal?: string | null;
  readonly signalNumber?: number | null;
  readonly process?: {
    readonly pid: number;
    readonly executable: { readonly sha256: string };
    readonly argv: readonly string[];
    readonly personality: string;
  };
  readonly thread?: { readonly pid: number; readonly ptid: readonly number[] };
}
interface ControlReceipt {
  readonly qualified: false;
  readonly observerValidated: true;
  readonly driverSHA256: string;
  readonly dataSHA256: string;
}
const driver = fileURLToPath(import.meta.url);
const data = fileURLToPath(new URL("./native-ops-data.ts", import.meta.url));
const expectedSource = "710dfca619c82d76078a72ee64629632c798dea5";
const expectedVp = "d824ad1a2f5080c71a4444444b26e0fde0fbc6ccc7e47054cd436f7f0b5fc572";
const originalCommand = [
  "node",
  ".github/actions/test-js-packages-with-history/native-fault-evidence.ts",
  "--",
  "vp",
  "run",
  "--workspace-root",
  "test:js:builders",
];
const earlySettings = [
  "set auto-load off",
  "set auto-load local-gdbinit off",
  "set debuginfod enabled off",
];
const secretKeys = [
  "GITHUB_TOKEN",
  "GH_TOKEN",
  "NPM_TOKEN",
  "NODE_AUTH_TOKEN",
  "ACTIONS_RUNTIME_TOKEN",
  "ACTIONS_ID_TOKEN_REQUEST_TOKEN",
];
const digest = (value: Uint8Array | string): string =>
  createHash("sha256").update(value).digest("hex");
const sha = (file: string): string => digest(readFileSync(file));
function write(file: string, value: unknown): void {
  writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
}
function parse<T>(file: string): T {
  return JSON.parse(readFileSync(file, "utf8")) as T;
}
function rootStatus(summary: Summary): number {
  const root = summary.rootExit;
  assert.ok(root, "No authoritative original root exit");
  if (root.exitCode !== null) return root.exitCode;
  assert.ok(
    Number.isInteger(root.signalNumber) && root.signalNumber! > 0 && root.signalNumber! < 128,
  );
  return 128 + root.signalNumber!;
}
function events(directory: string): Event[] {
  return readFileSync(path.join(directory, "events.jsonl"), "utf8")
    .trim()
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line) as Event);
}
function summary(directory: string): Summary {
  return parse<Summary>(path.join(directory, "summary.json"));
}
function run(command: readonly string[], output?: string): number {
  assert.ok(command[0]);
  const result = spawnSync(command[0], command.slice(1), { stdio: "inherit" });
  assert.ifError(result.error);
  assert.equal(result.signal, null, "Orchestration terminated by signal");
  assert.notEqual(result.status, null);
  if (output)
    write(path.join(output, "orchestration.json"), {
      qualified: false,
      command,
      status: result.status,
    });
  return result.status!;
}
function trace(command: readonly string[], output: string): number {
  mkdirSync(output);
  const program = path.join(output, "gdb.py");
  const launch = path.join(output, "launch.gdb");
  writeFileSync(program, gdbProgram);
  writeFileSync(
    launch,
    "python\nimport runpy\nrunpy.run_path(" +
      JSON.stringify(program) +
      ", init_globals={'NATIVE_OPS_OUTPUT': " +
      JSON.stringify(output) +
      "})\nend\n",
  );
  write(path.join(output, "invocation.json"), {
    qualified: false,
    command,
    cwd: process.cwd(),
    driverSHA256: sha(driver),
    dataSHA256: sha(data),
    gdbProgramSHA256: digest(gdbProgram),
    earlySettingsBeforeExecutableLoad: earlySettings,
    observer: "GNU GDB ptrace events; possible internal/software breakpoints",
    deliberateInferiorDataOrRegisterEdits: false,
    ASLR: "retained; disable-randomization off",
    compilerStackOrProfileOverrides: false,
  });
  const stdout = openSync(path.join(output, "stdout.raw"), "wx");
  const stderr = openSync(path.join(output, "stderr.raw"), "wx");
  let status: number | null = null;
  try {
    const result = spawnSync(
      "gdb",
      [
        "--quiet",
        "--nx",
        "--batch",
        ...earlySettings.flatMap((setting) => ["-iex", setting]),
        "--command",
        launch,
        "--args",
        ...command,
      ],
      { stdio: ["ignore", stdout, stderr] },
    );
    assert.ifError(result.error);
    assert.equal(result.signal, null, "GDB infrastructure terminated; no product inference");
    status = result.status;
  } finally {
    closeSync(stdout);
    closeSync(stderr);
  }
  const observed = summary(output);
  write(path.join(output, "driver-outcome.json"), {
    qualified: false,
    debuggerStatus: status,
    originalStdoutSHA256: sha(path.join(output, "stdout.raw")),
    originalStderrSHA256: sha(path.join(output, "stderr.raw")),
  });
  assert.equal(status, 0, "GDB infrastructure failed; no product acceptance");
  return rootStatus(observed);
}
function childTrace(binary: string, output: string, ...args: string[]): number {
  return run([process.execPath, driver, "trace", output, "--", binary, ...args]);
}
function control(output: string): void {
  mkdirSync(output, { recursive: true });
  const source = path.join(output, "short-child.c");
  const binary = path.join(output, "short-child");
  writeFileSync(source, controlProgram);
  assert.equal(run(["cc", "-O0", "-g", "-pthread", source, "-o", binary]), 0);
  const first = path.join(output, "trace");
  assert.equal(childTrace(binary, first), 7);
  const stdout = readFileSync(path.join(first, "stdout.raw"), "utf8");
  assert.ok(stdout.includes("CONTROL_SIGSEGV_HANDLER\n"));
  const resultLine = stdout.split("\n").find((line) => line.startsWith("CONTROL_RESULT "));
  assert.ok(resultLine);
  const result = JSON.parse(resultLine.slice("CONTROL_RESULT ".length)) as {
    childPid: number;
    signal: number;
    elapsedNs: number;
  };
  const match = /CONTROL_THREAD (\d+) (\d+)/u.exec(stdout);
  assert.ok(match && match[1] && match[2]);
  const pid = Number(match[1]),
    tid = Number(match[2]);
  assert.equal(pid, result.childPid);
  assert.equal(result.signal, 6);
  assert.ok(result.elapsedNs < 250_000_000, JSON.stringify(result));
  const observed = events(first);
  assert.deepEqual(summary(first).observerGaps, []);
  const stops = observed.filter((event) => event.event === "stop");
  for (const signal of ["SIGSEGV", "SIGABRT"]) {
    assert.ok(
      stops.some(
        (event) =>
          event.signal === signal && event.process?.pid === pid && event.thread?.ptid[1] === tid,
      ),
    );
  }
  assert.ok(
    stops.some(
      (event) =>
        event.process?.pid === pid &&
        event.process.executable.sha256 === sha(binary) &&
        event.process.argv.includes("worker"),
    ),
  );
  assert.ok(
    stops.every(
      (event) => event.process && !(Number.parseInt(event.process.personality, 16) & 0x40000),
    ),
  );
  assert.ok(Number(readFileSync("/proc/sys/kernel/randomize_va_space", "utf8")) > 0);
  assert.ok(
    observed.some(
      (event) =>
        event.event === "NewThreadEvent" &&
        event.thread?.pid === pid &&
        event.thread.ptid[1] === tid,
    ),
  );
  assert.ok(
    observed.some(
      (event) => event.event === "exit" && event.pid === pid && event.signal === "SIGABRT",
    ),
  );
  const signaled = path.join(output, "root-signal");
  assert.equal(childTrace(binary, signaled, "root-signal"), 137);
  assert.deepEqual(summary(signaled).observerGaps, []);
  assert.equal(summary(signaled).rootExit?.signalNumber, 9);
  assert.equal(summary(signaled).rootExit?.signal, "SIGKILL");
  assert.ok(
    readFileSync(path.join(signaled, "stdout.raw"), "utf8").includes("CONTROL_HANDLED_SEGV\n"),
  );
  const late = path.join(output, "parent-first");
  assert.equal(childTrace(binary, late, "parent-first"), 13);
  const lateSummary = summary(late),
    lateEvents = events(late);
  assert.deepEqual(lateSummary.observerGaps, []);
  assert.deepEqual(lateSummary.remainingInferiors, []);
  assert.equal(lateSummary.rootExit?.exitCode, 13);
  const rootExit = lateEvents.find(
    (event) => event.event === "exit" && event.pid === lateSummary.rootExit?.pid,
  );
  assert.ok(rootExit);
  const faults = lateEvents.filter(
    (event) => event.event === "fault-state" && event.signal === "SIGABRT",
  );
  assert.ok(faults.length > 0 && faults.every((event) => event.monotonicNs > rootExit.monotonicNs));
  assert.ok(
    lateEvents.some(
      (event) => event.event === "exit" && event.signalNumber === 6 && event.pid === faults[0]?.pid,
    ),
  );
  const receipt: ControlReceipt & Record<string, unknown> = {
    qualified: false,
    observerValidated: true,
    driverSHA256: sha(driver),
    dataSHA256: sha(data),
    originalParentExit: 7,
    originalChildSignal: "SIGABRT",
    observedChild: result,
    workerTid: tid,
    handledSegvThenRootTerminalSignal: "SIGKILL",
    parentFirstExit: 13,
    lateGrandchildFaultObserved: true,
  };
  write(path.join(output, "control-complete.json"), receipt);
}
function tool(name: string): Record<string, unknown> {
  const candidates = (process.env.PATH ?? "")
    .split(path.delimiter)
    .map((entry) => path.resolve(entry || ".", name));
  const selected = candidates.find(
    (candidate) => existsSync(candidate) && statSync(candidate).mode & 0o111,
  );
  if (!selected) return { name, missing: true };
  const symlinks: { link: string; target: string }[] = [];
  let current = selected;
  for (let index = 0; index < 32 && lstatSync(current).isSymbolicLink(); index++) {
    const target = readlinkSync(current);
    symlinks.push({ link: current, target });
    current = path.resolve(path.dirname(current), target);
  }
  const resolved = realpathSync(selected);
  return {
    name,
    selected,
    symlinks,
    realpath: resolved,
    sha256: sha(resolved),
    bytes: statSync(resolved).size,
  };
}
function revision(): string {
  const result = spawnSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" });
  assert.equal(result.status, 0);
  return result.stdout.trim();
}
function original(output: string, receiptPath: string): number {
  const receipt = parse<ControlReceipt>(receiptPath);
  assert.equal(receipt.observerValidated, true);
  assert.equal(receipt.qualified, false);
  assert.equal(receipt.driverSHA256, sha(driver));
  assert.equal(receipt.dataSHA256, sha(data));
  assert.equal(revision(), expectedSource);
  assert.equal(process.env.GITHUB_RUN_ATTEMPT, "1", "No workflow rerun replay");
  const tools = ["vp", "pnpm", "node", "rustc", "cargo"].map(tool);
  assert.equal(tools.find((item) => item.name === "vp")?.sha256, expectedVp);
  mkdirSync(path.dirname(output), { recursive: true });
  writeFileSync(path.join(path.dirname(output), "original-replay-claimed"), expectedSource + "\n", {
    flag: "wx",
  });
  write(path.join(path.dirname(output), "before-original.json"), {
    qualified: false,
    testedSource: revision(),
    orchestrationSha: process.env.GITHUB_SHA,
    runner: {
      type: os.type(),
      release: os.release(),
      machine: os.machine(),
      architecture: os.arch(),
    },
    logicalCPUs: os.cpus().length,
    cpuinfo: readFileSync("/proc/cpuinfo", "utf8"),
    limits: readFileSync("/proc/self/limits", "utf8"),
    tools,
    controlReceipt: receiptPath,
    oldPhysicalCPUAndOldUnrecordedLimitsClaimedEqual: false,
  });
  const status = trace(originalCommand, output),
    observed = summary(output);
  const faultCount = events(output).filter((event) => event.event === "fault-state").length;
  write(path.join(path.dirname(output), "diagnostic-result.json"), {
    qualified: false,
    originalExitStatus: status,
    testedSourceAfter: revision(),
    faultEvents: faultCount,
    observerGaps: observed.observerGaps,
    result: observed.observerGaps.length
      ? "observation incomplete; incident remains unresolved"
      : faultCount
        ? "fault evidence captured; human attribution required"
        : "not reproduced; incident remains unresolved",
    noAutomaticRetry: true,
    noPerformanceClaim: true,
  });
  return status || (observed.observerGaps.length ? 2 : 0);
}
function publish(privateRoot: string, publicRoot: string, extraPatterns: readonly string[]): void {
  assert.ok(existsSync(privateRoot), "No private capture exists");
  assert.ok(!lstatSync(privateRoot).isSymbolicLink(), "Private root cannot be a symlink");
  assert.ok(!existsSync(publicRoot), "Public export is immutable");
  const staging = publicRoot + ".unsealed";
  mkdirSync(staging);
  const secrets = secretKeys.flatMap((key) => (process.env[key] ? [process.env[key]!] : []));
  const receipts: Record<string, unknown>[] = [];
  function visit(directory: string, base = privateRoot, prefix = "capture"): void {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const source = path.join(directory, entry.name);
      assert.ok(!entry.isSymbolicLink(), "Symlinks cannot enter a public export");
      if (entry.isDirectory()) {
        visit(source, base, prefix);
        continue;
      }
      assert.ok(entry.isFile(), "Only regular diagnostic files can be exported");
      const relative = path.join(prefix, path.relative(base, source));
      const before = statSync(source),
        raw = readFileSync(source),
        after = statSync(source);
      assert.ok(
        before.size === after.size && before.mtimeMs === after.mtimeMs,
        "Capture changed during export",
      );
      const originalSHA256 = digest(raw);
      let text: string;
      try {
        text = new TextDecoder("utf8", { fatal: true, ignoreBOM: true }).decode(raw);
      } catch {
        receipts.push({
          path: relative,
          originalSHA256,
          originalBytes: raw.length,
          exported: false,
          reason: "binary; digest only",
        });
        continue;
      }
      let knownSecretMatches = 0;
      for (const secret of secrets) {
        knownSecretMatches += text.split(secret).length - 1;
        text = text.replaceAll(secret, "<redacted>");
      }
      assert.ok(
        secrets.every((secret) => !text.includes(secret)),
        "Secret remains after projection",
      );
      const projected = Buffer.from(text),
        publicPath = relative + ".public.txt";
      const destination = path.join(staging, publicPath);
      mkdirSync(path.dirname(destination), { recursive: true });
      writeFileSync(destination, projected, { flag: "wx" });
      receipts.push({
        path: relative,
        publicPath,
        originalSHA256,
        originalBytes: raw.length,
        publicSHA256: digest(projected),
        publicBytes: projected.length,
        knownSecretMatches,
        rawBytesPreserved: raw.equals(projected),
        exported: true,
      });
    }
  }
  visit(privateRoot);
  const extras = [...new Set(extraPatterns.flatMap((pattern) => [...globSync(pattern)]))];
  for (const [index, source] of extras.entries()) {
    assert.ok(!lstatSync(source).isSymbolicLink(), "Extra evidence cannot be a symlink");
    if (statSync(source).isDirectory()) visit(source, source, "source-" + index);
    else {
      assert.ok(!lstatSync(source).isSymbolicLink(), "Extra evidence cannot be a symlink");
      const isolated = path.join(privateRoot, "source-evidence-" + index);
      mkdirSync(isolated);
      writeFileSync(path.join(isolated, path.basename(source)), readFileSync(source));
      visit(isolated, isolated, "source-" + index);
    }
  }
  write(path.join(staging, "sealed-manifest.json"), {
    schema: "native-ops.public-export/v1",
    qualified: false,
    projectionCompleteForListedFiles: true,
    scope: "Known credential values redacted; raw capture is never an artifact path",
    files: receipts,
  });
  renameSync(staging, publicRoot);
}
function main(): number {
  const [mode, rawOutput, ...rest] = process.argv.slice(2);
  assert.ok(rawOutput, "Output directory is required");
  const output = path.resolve(rawOutput);
  if (mode === "control") {
    control(output);
    return 0;
  }
  if (mode === "publish") {
    assert.ok(rest[0]);
    publish(output, path.resolve(rest[0]), rest.slice(1));
    return 0;
  }
  if (mode === "trace") {
    assert.equal(rest[0], "--");
    return trace(rest.slice(1), output);
  }
  assert.equal(mode, "original");
  assert.ok(rest[0]);
  return original(output, path.resolve(rest[0]));
}
process.exitCode = main();
