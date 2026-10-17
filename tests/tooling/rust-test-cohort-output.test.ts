import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import type { ChildProcess } from "node:child_process";
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import type { TestContext } from "node:test";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";

const runner = fileURLToPath(
  new URL("../../tools/support/compat/github/run-rust-test-cohort.ts", import.meta.url),
);
const posix = { skip: process.platform === "win32", timeout: 15_000 };
const banner = Buffer.from(
  "Cargo targets (compiler): 0 integration targets and shared unit targets\n",
);
// Preserve raw bytes, including invalid UTF-8 and splits across UTF-8 sequences.
const payload = Buffer.from(Array.from({ length: 524_301 }, (_, index) => index % 256));

function signalIfPresent(pid: number, signal: NodeJS.Signals | 0): boolean {
  try {
    process.kill(pid, signal);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ESRCH") return false;
    throw error;
  }
}

async function waitFor(predicate: () => boolean, description: string) {
  const deadline = performance.now() + 8_000;
  while (!predicate()) {
    assert.ok(performance.now() < deadline, `timed out waiting for ${description}`);
    await delay(20);
  }
}

function observe(child: ChildProcess) {
  const stdout: Buffer[] = [];
  const stderr: Buffer[] = [];
  child.stdout?.on("data", (chunk: Buffer) => stdout.push(chunk));
  child.stderr?.on("data", (chunk: Buffer) => stderr.push(chunk));
  const exit = new Promise<{ code: number | null; signal: NodeJS.Signals | null }>(
    (resolve, reject) => {
      child.once("error", reject);
      child.once("exit", (code, signal) => resolve({ code, signal }));
    },
  );
  const close = new Promise<void>((resolve) => child.once("close", () => resolve()));
  return {
    exit,
    close,
    stdout: () => Buffer.concat(stdout),
    stderr: () => Buffer.concat(stderr).toString(),
  };
}

function fixture(t: TestContext, mode: string) {
  const directory = mkdtempSync(join(tmpdir(), "vize-rust-output-"));
  const ready = join(directory, "ready.json");
  t.after(() => {
    if (existsSync(ready))
      signalIfPresent(-JSON.parse(readFileSync(ready, "utf8")).child, "SIGKILL");
    rmSync(directory, { recursive: true, force: true });
  });
  const plan = join(directory, "plan.json");
  writeFileSync(
    plan,
    JSON.stringify({
      schemaVersion: 1,
      scope: "affected",
      packages: ["vize_relief"],
      cargoArgs: ["--package", "vize_relief"],
    }),
  );
  writeFileSync(join(directory, "payload"), payload);
  const metadata = {
    workspace_root: directory,
    workspace_members: ["vize_relief"],
    packages: [
      {
        id: "vize_relief",
        name: "vize_relief",
        dependencies: [],
        manifest_path: join(directory, "crates/vize_relief/Cargo.toml"),
        targets: [{ name: "vize_relief", kind: ["lib"], test: true }],
      },
    ],
  };
  writeFileSync(
    join(directory, "cargo"),
    `#!${process.execPath}
const { spawn } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const dir = ${JSON.stringify(directory)};
const mode = process.env.VIZE_OUTPUT_MODE;
const log = (text) => fs.appendFileSync(path.join(dir, "signals"), text + "\\n");
if (process.argv[2] === "metadata") {
  if (mode === "spawn-error") fs.unlinkSync(__filename);
  process.stdout.write(${JSON.stringify(JSON.stringify(metadata))});
} else if (process.argv[2] === "grandchild") {
  process.on("SIGINT", () => log("grandchild SIGINT"));
  process.on("SIGTERM", () => { log("grandchild SIGTERM"); process.exit(0); });
  process.send("ready");
  setInterval(() => {}, 1000);
} else {
  fs.writeFileSync(path.join(dir, "invocation.json"), JSON.stringify({ argv: process.argv.slice(2), cwd: process.cwd(), env: process.env }));
  process.stderr.write("cargo stderr\\n");
  const bytes = fs.readFileSync(path.join(dir, "payload"));
  if (mode === "0" || mode === "7") {
    process.stdout.write(bytes.subarray(0, 13));
    process.stdout.write(bytes.subarray(13), () => process.exit(Number(mode)));
  } else {
    const grandchild = spawn(process.execPath, [__filename, "grandchild"], { stdio: ["ignore", "inherit", "inherit", "ipc"] });
    process.on("SIGINT", () => log("child SIGINT"));
    process.on("SIGTERM", () => log("child SIGTERM"));
    grandchild.once("exit", () => process.stdout.write("stopped\\n", () => process.exit(0)));
    grandchild.once("message", () => {
      const ready = () => fs.writeFileSync(path.join(dir, "ready.json"), JSON.stringify({ child: process.pid, grandchild: grandchild.pid, runner: process.ppid }));
      if (mode === "sink-error") {
        ready();
        const timer = setInterval(() => {
          if (fs.existsSync(path.join(dir, "emit"))) { clearInterval(timer); process.stdout.write(bytes); }
        }, 10);
      } else process.stdout.write(bytes, ready);
    });
  }
}
`,
    { mode: 0o755 },
  );
  const options = ["--profile", "ci", "-E", "package(=vize_relief)", "space ' quote\nnewline"];
  const env = {
    ...process.env,
    PATH: `${directory}:${process.env.PATH ?? ""}`,
    VIZE_OUTPUT_MODE: mode,
    VIZE_OUTPUT_ENV: "preserved ' literal\nvalue",
    VIZE_RUST_BUILD_STDOUT_PATH: join(directory, "cargo.jsonl"),
    VIZE_RUST_COHORT_TIMING_PATH: join(directory, "timing.json"),
  };
  return { directory, ready, env, options, argv: [runner, plan, "compiler", ...options] };
}

function launch(t: TestContext, f: ReturnType<typeof fixture>) {
  // Exercise the entry PID used by Actions, with no pipeline or waiting shell.
  const child = spawn(
    "bash",
    [
      "--noprofile",
      "--norc",
      "-eo",
      "pipefail",
      "-c",
      'exec "$@"',
      "--",
      process.execPath,
      ...f.argv,
    ],
    { cwd: f.directory, env: f.env },
  );
  const observation = observe(child);
  t.after(async () => {
    if (existsSync(f.ready))
      signalIfPresent(-JSON.parse(readFileSync(f.ready, "utf8")).child, "SIGKILL");
    if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
    await observation.close;
  });
  return { child, ...observation };
}

function receipt(f: ReturnType<typeof fixture>, status: number) {
  const timing = JSON.parse(readFileSync(f.env.VIZE_RUST_COHORT_TIMING_PATH, "utf8"));
  assert.deepEqual(timing, {
    phase: "workspace-cohort",
    cohort: "compiler",
    elapsed_seconds: timing.elapsed_seconds,
    exit_code: status,
  });
  assert.ok(Number.isSafeInteger(timing.elapsed_seconds) && timing.elapsed_seconds >= 0);
}

for (const status of [0, 7]) {
  void test(`captured cohort stdout preserves all bytes and exit ${status}`, posix, async (t) => {
    const f = fixture(t, String(status));
    const result = launch(t, f);
    assert.deepEqual(await result.exit, { code: status, signal: null }, result.stderr());
    // The file must be complete at leader exit, before inherited pipes close.
    assert.deepEqual(
      readFileSync(f.env.VIZE_RUST_BUILD_STDOUT_PATH),
      Buffer.concat([banner, payload]),
    );
    await result.close;
    assert.deepEqual(result.stdout(), Buffer.concat([banner, payload]));
    assert.equal(result.stderr(), "cargo stderr\n");
    receipt(f, status);
    const invocation = JSON.parse(readFileSync(join(f.directory, "invocation.json"), "utf8"));
    assert.deepEqual(invocation.argv, [
      "nextest",
      "run",
      "--package",
      "vize_relief",
      "--lib",
      ...f.options,
    ]);
    assert.equal(invocation.cwd, realpathSync(f.directory));
    const expectedEnv = spawnSync(
      "bash",
      [
        "--noprofile",
        "--norc",
        "-c",
        'exec "$@"',
        "--",
        process.execPath,
        "-p",
        "JSON.stringify(process.env)",
      ],
      { env: f.env, cwd: f.directory, encoding: "utf8" },
    );
    assert.equal(expectedEnv.status, 0);
    assert.deepEqual(invocation.env, JSON.parse(expectedEnv.stdout));
  });
}

void test("capture open failure prevents Cargo launch and records failure", posix, async (t) => {
  const f = fixture(t, "0");
  f.env.VIZE_RUST_BUILD_STDOUT_PATH = f.directory;
  const result = launch(t, f);
  assert.deepEqual(await result.exit, { code: 1, signal: null });
  await result.close;
  assert.match(result.stderr(), /EISDIR/);
  assert.equal(existsSync(join(f.directory, "invocation.json")), false);
  receipt(f, 1);
});

void test("capture flushes the banner when Cargo cannot spawn", posix, async (t) => {
  const f = fixture(t, "spawn-error");
  // Metadata removes this sole Cargo executable before the actual spawn.
  f.env.PATH = f.directory;
  const result = spawn(process.execPath, f.argv, { cwd: f.directory, env: f.env });
  const observation = observe(result);
  assert.deepEqual(await observation.exit, { code: 1, signal: null });
  assert.deepEqual(readFileSync(f.env.VIZE_RUST_BUILD_STDOUT_PATH), banner);
  await observation.close;
  assert.match(observation.stderr(), /spawn cargo ENOENT/);
  receipt(f, 1);
});

for (const signal of ["SIGINT", "SIGTERM"] as const) {
  void test(
    `captured ${signal} cancellation drains descendants and flushes emitted bytes`,
    posix,
    async (t) => {
      const f = fixture(t, "wait");
      const result = launch(t, f);
      const unrelated = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], {
        stdio: "ignore",
      });
      t.after(() => unrelated.kill("SIGKILL"));
      await waitFor(() => existsSync(f.ready), "Cargo and grandchild readiness");
      const pids = JSON.parse(readFileSync(f.ready, "utf8"));
      assert.equal(pids.runner, result.child.pid);
      const started = performance.now();
      result.child.kill(signal);
      const status = signal === "SIGINT" ? 130 : 143;
      assert.deepEqual(await result.exit, { code: status, signal: null }, result.stderr());
      assert.ok(performance.now() - started < 7_000);
      assert.equal(signalIfPresent(-pids.child, 0), false);
      assert.equal(signalIfPresent(pids.grandchild, 0), false);
      assert.equal(signalIfPresent(unrelated.pid!, 0), true);
      assert.deepEqual(
        readFileSync(f.env.VIZE_RUST_BUILD_STDOUT_PATH),
        Buffer.concat([banner, payload, Buffer.from("stopped\n")]),
      );
      await result.close;
      assert.deepEqual(result.stdout(), readFileSync(f.env.VIZE_RUST_BUILD_STDOUT_PATH));
      assert.match(
        result.stderr(),
        new RegExp(`Rust cohort received ${signal}; stopping its owned Cargo group\\.`),
      );
      receipt(f, status);
      const signals = readFileSync(join(f.directory, "signals"), "utf8").trim().split("\n").sort();
      assert.deepEqual(
        signals,
        signal === "SIGINT"
          ? ["child SIGINT", "child SIGTERM", "grandchild SIGINT", "grandchild SIGTERM"]
          : ["child SIGTERM", "grandchild SIGTERM"],
      );
    },
  );
}

void test(
  "a real mid-stream capture failure stops only the owned Cargo group",
  posix,
  async (t) => {
    const f = fixture(t, "sink-error");
    const fifo = join(f.directory, "capture.fifo");
    const made = spawnSync("mkfifo", [fifo], { encoding: "utf8" });
    assert.equal(made.status, 0, made.stderr);
    f.env.VIZE_RUST_BUILD_STDOUT_PATH = fifo;
    // A named pipe gives a real EPIPE after startup on both Linux and macOS.
    const reader = spawn(
      process.execPath,
      [
        "-e",
        `
    const fs = require("node:fs");
    const fd = fs.openSync(process.argv[1], "r");
    fs.readSync(fd, Buffer.alloc(4096), 0, 4096, null);
    process.on("message", () => { fs.closeSync(fd); process.exit(0); });
  `,
        fifo,
      ],
      { stdio: ["ignore", "ignore", "pipe", "ipc"] },
    );
    const readerExit = new Promise<void>((resolve) => reader.once("exit", () => resolve()));
    t.after(() => reader.kill("SIGKILL"));
    const result = launch(t, f);
    const unrelated = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], {
      stdio: "ignore",
    });
    t.after(() => unrelated.kill("SIGKILL"));
    await waitFor(() => existsSync(f.ready), "Cargo and grandchild readiness");
    const pids = JSON.parse(readFileSync(f.ready, "utf8"));
    assert.equal(pids.runner, result.child.pid);
    reader.send("close");
    await readerExit;
    writeFileSync(join(f.directory, "emit"), "");
    assert.deepEqual(await result.exit, { code: 1, signal: null }, result.stderr());
    assert.equal(signalIfPresent(-pids.child, 0), false);
    assert.equal(signalIfPresent(pids.grandchild, 0), false);
    assert.equal(signalIfPresent(unrelated.pid!, 0), true);
    await result.close;
    assert.match(result.stderr(), /stdout capture failed; stopping its owned Cargo group/);
    assert.match(result.stderr(), /EPIPE/);
    assert.deepEqual(readFileSync(join(f.directory, "signals"), "utf8").trim().split("\n").sort(), [
      "child SIGTERM",
      "grandchild SIGTERM",
    ]);
    receipt(f, 1);
  },
);
