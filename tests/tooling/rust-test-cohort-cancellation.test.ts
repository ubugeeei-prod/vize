import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
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
import { fileURLToPath, pathToFileURL } from "node:url";
import { parse } from "yaml";
import { requireRustTier } from "../../tools/support/compat/github/require-rust-tier.ts";

const runner = fileURLToPath(
  new URL("../../tools/support/compat/github/run-rust-test-cohort.ts", import.meta.url),
);
const posix = { skip: process.platform === "win32", timeout: 15_000 };

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

function fixture(t: TestContext, mode: string) {
  const directory = mkdtempSync(join(tmpdir(), "vize-rust-cancel-"));
  t.after(() => {
    const ready = join(directory, "ready.json");
    if (existsSync(ready)) {
      const pids = JSON.parse(readFileSync(ready, "utf8"));
      signalIfPresent(-pids.child, "SIGKILL");
    }
    rmSync(directory, { recursive: true, force: true });
  });
  const cargo = join(directory, "cargo");
  const plan = join(directory, "plan.json");
  const metadata = {
    workspace_root: directory,
    workspace_members: ["vize_relief"],
    packages: [
      {
        id: "vize_relief",
        name: "vize_relief",
        manifest_path: join(directory, "crates/vize_relief/Cargo.toml"),
        dependencies: [],
        targets: [{ name: "vize_relief", kind: ["lib"], test: true }],
      },
    ],
  };
  writeFileSync(
    plan,
    JSON.stringify({
      schemaVersion: 1,
      scope: "affected",
      packages: ["vize_relief"],
      cargoArgs: ["--package", "vize_relief"],
    }),
  );
  // Exercise the actual CLI and its metadata/argv validation without compiling
  // Rust. Both the fake Cargo and its grandchild are real OS processes.
  writeFileSync(
    cargo,
    `#!${process.execPath}
const { spawn } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");
const directory = ${JSON.stringify(directory)};
const mode = process.env.VIZE_CANCELLATION_MODE;
if (process.argv[2] === "metadata") {
  if (mode === "spawn-error") fs.unlinkSync(__filename);
  process.stdout.write(${JSON.stringify(JSON.stringify(metadata))});
} else if (process.argv[2] === "grandchild") {
  const log = (signal) => fs.appendFileSync(path.join(directory, "signals"), "grandchild " + signal + "\\n");
  process.on("SIGINT", () => {
    log("SIGINT");
    if (mode === "graceful") process.exit(0);
  });
  process.on("SIGTERM", () => {
    log("SIGTERM");
    if (mode !== "ignore-all") process.exit(0);
  });
  process.send("ready");
  setInterval(() => {}, 1000);
} else {
  fs.writeFileSync(path.join(directory, "invocation.json"), JSON.stringify({
    argv: process.argv.slice(2), cwd: process.cwd(), env: process.env,
  }));
  process.stdout.write("cargo stdout\\n");
  process.stderr.write("cargo stderr\\n");
  if (/^[0-9]+$/.test(mode)) process.exit(Number(mode));
  else if (mode === "child-signal") process.kill(process.pid, "SIGTERM");
  else {
    const grandchild = spawn(process.execPath, [__filename, "grandchild"], {
      stdio: ["ignore", "inherit", "inherit", "ipc"],
    });
    const log = (signal) => fs.appendFileSync(path.join(directory, "signals"), "child " + signal + "\\n");
    process.on("SIGINT", () => {
      log("SIGINT");
      if (mode === "leader-exits") process.exit(0);
    });
    process.on("SIGTERM", () => log("SIGTERM"));
    grandchild.once("exit", () => process.exit(0));
    grandchild.once("message", () => fs.writeFileSync(path.join(directory, "ready.json"),
      JSON.stringify({ child: process.pid, grandchild: grandchild.pid, runner: process.ppid })));
  }
}
`,
    { mode: 0o755 },
  );
  const options = ["--profile", "ci", "-E", "package(=vize_relief)", "space ' quote\nnewline"];
  const env = {
    ...process.env,
    PATH: mode === "spawn-error" ? directory : `${directory}:${process.env.PATH ?? ""}`,
    VIZE_CANCELLATION_MODE: mode,
    VIZE_CANCELLATION_ENV: "preserved ' literal\nvalue",
    VIZE_RUST_COHORT_TIMING_PATH: join(directory, "target/nextest/pr/timing.json"),
  };
  return { directory, env, argv: [runner, plan, "compiler", ...options], options };
}

for (const [mode, expected] of [
  ["0", 0],
  ["7", 7],
  ["child-signal", 1],
] as const) {
  void test(`cohort CLI preserves argv, environment, cwd, output and ${mode} exit`, posix, (t) => {
    const { directory, env, argv, options } = fixture(t, mode);
    const result = spawnSync(process.execPath, argv, { cwd: directory, env, encoding: "utf8" });
    assert.equal(result.status, expected, result.stderr);
    assert.equal(result.signal, null);
    assert.match(result.stdout, /cargo stdout\n$/);
    assert.equal(result.stderr, "cargo stderr\n");
    const timing = JSON.parse(readFileSync(env.VIZE_RUST_COHORT_TIMING_PATH, "utf8"));
    assert.deepEqual(timing, {
      phase: "workspace-cohort",
      cohort: "compiler",
      elapsed_seconds: timing.elapsed_seconds,
      exit_code: expected,
    });
    assert.ok(Number.isSafeInteger(timing.elapsed_seconds) && timing.elapsed_seconds >= 0);
    const invocation = JSON.parse(readFileSync(join(directory, "invocation.json"), "utf8"));
    assert.deepEqual(invocation.argv, [
      "nextest",
      "run",
      "--package",
      "vize_relief",
      "--lib",
      ...options,
    ]);
    assert.equal(invocation.cwd, realpathSync(directory));
    const baseline = spawnSync(process.execPath, ["-p", "JSON.stringify(process.env)"], {
      env,
      encoding: "utf8",
    });
    assert.equal(baseline.status, 0);
    assert.deepEqual(invocation.env, JSON.parse(baseline.stdout));
  });
}

void test("cohort CLI preserves Cargo spawn errors as failures", posix, (t) => {
  const { directory, env, argv } = fixture(t, "spawn-error");
  const result = spawnSync(process.execPath, argv, { cwd: directory, env, encoding: "utf8" });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /spawn cargo ENOENT/);
  assert.equal(JSON.parse(readFileSync(env.VIZE_RUST_COHORT_TIMING_PATH, "utf8")).exit_code, 1);
});

for (const [signal, mode, escalated] of [
  ["SIGINT", "graceful", false],
  ["SIGTERM", "graceful", false],
  ["SIGINT", "ignore-int", true],
  ["SIGINT", "leader-exits", true],
  ["SIGINT", "ignore-all", true],
  ["SIGTERM", "ignore-all", true],
] as const) {
  void test(
    `cohort cancellation ${signal}/${mode} drains only its own child group`,
    posix,
    async (t) => {
      const { directory, env, argv } = fixture(t, mode);
      const unrelated = spawn(process.execPath, ["-e", "setInterval(() => {}, 1000)"], {
        stdio: "ignore",
      });
      const workflow = parse(
        readFileSync(
          new URL("../../.github/workflows/pr-rust-checks.yml", import.meta.url),
          "utf8",
        ),
      ) as { jobs: Record<string, { steps: { name?: string; run?: string }[] }> };
      const step = workflow.jobs["pr-rust-shard"].steps.find(
        ({ name }) => name === "Build and run the affected Rust test cohort",
      );
      assert.ok(step?.run);
      const helperDirectory = join(directory, "tools/support/compat/github");
      mkdirSync(helperDirectory, { recursive: true });
      for (const name of ["rust-build-evidence.ts", "compare-unused-bindings-profiles.ts"])
        writeFileSync(
          join(helperDirectory, name),
          readFileSync(new URL(name, pathToFileURL(runner))),
        );
      writeFileSync(
        join(helperDirectory, "run-rust-test-cohort.ts"),
        `import { main } from ${JSON.stringify(pathToFileURL(runner).href)}; await main();\n`,
      );
      Object.assign(env, {
        RUNNER_TEMP: directory,
        RUST_PLAN_JSON: readFileSync(argv[1], "utf8"),
        COHORT: "compiler",
        NEXTEST_PROFILE: "pr",
        NEXTEST_FILTER: "package(=vize_relief)",
      });
      const command = ["bash", "--noprofile", "--norc", "-eo", "pipefail", "-c", step.run];
      const child = spawn(command[0], command.slice(1), { cwd: directory, env });
      let stderr = "";
      child.stdout.resume();
      child.stderr.setEncoding("utf8").on("data", (chunk: string) => {
        stderr += chunk;
      });
      const exit = new Promise<{ code: number | null; signal: string | null }>(
        (resolve, reject) => {
          child.once("error", reject);
          // Observe the runner itself, not pipe closure by a surviving grandchild.
          child.once("exit", (code, signal) => resolve({ code, signal }));
        },
      );
      const ready = join(directory, "ready.json");
      t.after(async () => {
        if (existsSync(ready)) {
          const pids = JSON.parse(readFileSync(ready, "utf8"));
          signalIfPresent(-pids.child, "SIGKILL");
        }
        if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
        unrelated.kill("SIGKILL");
        await exit;
      });
      await waitFor(() => {
        assert.equal(child.exitCode, null, stderr);
        return existsSync(ready);
      }, "Cargo and grandchild readiness");
      const pids = JSON.parse(readFileSync(ready, "utf8"));
      assert.ok(unrelated.pid);
      assert.notEqual(unrelated.pid, pids.child);
      assert.equal(pids.runner, child.pid, "the signalled step PID must become the Node runner");
      const started = performance.now();
      // Actions signals the process it launched (Bash), not a discovered Node
      // descendant. The workflow must make that entry PID own cancellation.
      child.kill(signal);
      const result = await Promise.race([
        exit,
        delay(7_000, undefined, { ref: false }).then(() => ({
          code: "still-running",
          signal: null,
        })),
      ]);
      assert.deepEqual(result, { code: signal === "SIGINT" ? 130 : 143, signal: null }, stderr);
      const elapsed = performance.now() - started;
      assert.ok(elapsed < 7_000, `cancellation took ${elapsed}ms`);
      assert.equal(
        signalIfPresent(pids.child, 0),
        false,
        "Cargo must be reaped before runner exit",
      );
      assert.equal(
        signalIfPresent(pids.grandchild, 0),
        false,
        "grandchild must be gone before exit",
      );
      assert.equal(
        signalIfPresent(-pids.child, 0),
        false,
        "owned group must be drained before exit",
      );
      assert.equal(signalIfPresent(unrelated.pid, 0), true, "unrelated process must survive");
      const timing = JSON.parse(
        readFileSync(join(directory, "target/nextest/pr/timing.json"), "utf8"),
      );
      assert.equal(timing.phase, "workspace-cohort");
      assert.equal(timing.cohort, "compiler");
      assert.equal(timing.exit_code, signal === "SIGINT" ? 130 : 143);
      assert.ok(Number.isSafeInteger(timing.elapsed_seconds) && timing.elapsed_seconds >= 0);
      for (const result of ["failure", "cancelled"]) {
        assert.equal(
          requireRustTier("pull_request", "true", {
            "pr-rust-build": { result: "success" },
            "pr-rust-shard": { result },
          }).exitCode,
          1,
        );
      }
      const signals = readFileSync(join(directory, "signals"), "utf8").trim().split("\n");
      assert.ok(signals.includes(`child ${signal}`), signals.join(", "));
      assert.ok(signals.includes(`grandchild ${signal}`), signals.join(", "));
      if (escalated) {
        assert.ok(signals.includes("grandchild SIGTERM"), signals.join(", "));
        assert.ok(elapsed >= 1_000, "ignored cancellation must reach the second signal");
        if (signal === "SIGTERM") {
          assert.equal(signals.filter((entry) => entry === "grandchild SIGTERM").length, 2);
          assert.equal(signals.filter((entry) => entry === "child SIGTERM").length, 2);
        }
      }
      if (mode === "ignore-all") assert.ok(elapsed >= 4_000, "SIGKILL escalation must run");
    },
  );
}
