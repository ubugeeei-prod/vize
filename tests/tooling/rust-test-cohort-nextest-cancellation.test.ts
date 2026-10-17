import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parse } from "yaml";

const enabled = process.env.VIZE_CANCELLATION_REAL_NEXTEST === "1";
const root = fileURLToPath(new URL("../../", import.meta.url));

function signalIfPresent(pid: number, signal: NodeJS.Signals | 0) {
  try {
    process.kill(pid, signal);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ESRCH") return false;
    throw error;
  }
}

void test(
  "pinned nextest reaps its separate test groups for both workflow cancellation signals",
  {
    skip: enabled
      ? false
      : "set VIZE_CANCELLATION_REAL_NEXTEST=1 with installed Rust and cargo-nextest 0.9.146",
    timeout: 45_000,
  },
  async (t) => {
    assert.notEqual(process.platform, "win32", "this probe requires POSIX process groups");
    const directory = mkdtempSync(join(tmpdir(), "vize-nextest-cancel-"));
    t.after(() => rmSync(directory, { recursive: true, force: true }));
    const env = {
      ...process.env,
      CARGO_NET_OFFLINE: "true",
      CARGO_TARGET_DIR: join(directory, "target"),
      PROBE_NODE: process.execPath,
      PROBE_CHILD: join(directory, "child.mjs"),
      RUNNER_TEMP: directory,
      RUST_PLAN_JSON: JSON.stringify({
        schemaVersion: 1,
        scope: "affected",
        packages: ["vize_relief"],
        cargoArgs: ["--package", "vize_relief"],
      }),
      COHORT: "compiler",
      NEXTEST_PROFILE: "pr",
      NEXTEST_FILTER: "package(=vize_relief)",
    };
    const version = spawnSync("cargo", ["nextest", "--version"], {
      cwd: directory,
      env,
      encoding: "utf8",
    });
    assert.equal(version.status, 0, version.error?.message ?? version.stderr);
    assert.match(version.stdout, /^cargo-nextest 0\.9\.146(?:\s|$)/);
    mkdirSync(join(directory, "crates/vize_relief/src"), { recursive: true });
    mkdirSync(join(directory, ".config"));
    mkdirSync(join(directory, "tools/support/compat/github"), { recursive: true });
    writeFileSync(
      join(directory, "Cargo.toml"),
      '[workspace]\nmembers=["crates/vize_relief"]\nresolver="3"\n[profile.ci]\ninherits="test"\ndebug=0\n',
    );
    writeFileSync(
      join(directory, "crates/vize_relief/Cargo.toml"),
      '[package]\nname="vize_relief"\nversion="0.0.0"\nedition="2024"\n',
    );
    writeFileSync(join(directory, ".config/nextest.toml"), "[profile.pr]\n");
    writeFileSync(
      join(directory, "crates/vize_relief/src/lib.rs"),
      `
#[cfg(test)]
mod tests {
    unsafe extern "C" {
        fn signal(sig: i32, handler: usize) -> usize;
        fn getpgrp() -> i32;
        fn getppid() -> i32;
        fn getpgid(pid: i32) -> i32;
    }
    #[test]
    fn ignores_cancellation_with_descendant() {
        unsafe { signal(2, 1); signal(15, 1); }
        let child = std::process::Command::new(std::env::var("PROBE_NODE").unwrap())
            .arg(std::env::var("PROBE_CHILD").unwrap()).spawn().unwrap();
        while !std::path::Path::new(&std::env::var("PROBE_CHILD_READY").unwrap()).exists() {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let record = format!("{{\\"test\\":{},\\"pgid\\":{},\\"nextest\\":{},\\"nextest_pgid\\":{},\\"descendant\\":{}}}",
            std::process::id(), unsafe { getpgrp() }, unsafe { getppid() },
            unsafe { getpgid(getppid()) }, child.id());
        std::fs::write(std::env::var("PROBE_READY").unwrap(), record).unwrap();
        loop { std::thread::sleep(std::time::Duration::from_millis(100)); }
    }
}
`,
    );
    writeFileSync(
      join(directory, "child.mjs"),
      `
import { writeFileSync } from "node:fs";
process.on("SIGINT", () => {});
process.on("SIGTERM", () => {});
writeFileSync(process.env.PROBE_CHILD_READY, "ready");
setInterval(() => {}, 1000);
`,
    );
    const runner = join(root, "tools/support/compat/github/run-rust-test-cohort.ts");
    for (const name of ["rust-build-evidence.ts", "compare-unused-bindings-profiles.ts"])
      writeFileSync(
        join(directory, "tools/support/compat/github", name),
        readFileSync(join(root, "tools/support/compat/github", name)),
      );
    writeFileSync(
      join(directory, "tools/support/compat/github/run-rust-test-cohort.ts"),
      `import { main } from ${JSON.stringify(pathToFileURL(runner).href)}; await main();\n`,
    );
    // Only this dependency-free temporary package is compiled, never the repo.
    const build = spawnSync(
      "cargo",
      ["nextest", "list", "--cargo-profile", "ci", "--profile", "pr"],
      {
        cwd: directory,
        env,
        encoding: "utf8",
        timeout: 20_000,
      },
    );
    assert.equal(build.status, 0, build.error?.message ?? build.stderr);
    const workflow = parse(
      readFileSync(join(root, ".github/workflows/pr-rust-checks.yml"), "utf8"),
    ) as {
      jobs: Record<string, { steps: { name?: string; run?: string }[] }>;
    };
    const step = workflow.jobs["pr-rust-shard"].steps.find(
      ({ name }) => name === "Build and run the affected Rust test cohort",
    );
    assert.ok(step?.run);
    for (const signal of ["SIGINT", "SIGTERM"] as const) {
      const ready = join(directory, `${signal}.json`);
      const child = spawn("bash", ["--noprofile", "--norc", "-eo", "pipefail", "-c", step.run], {
        cwd: directory,
        env: { ...env, PROBE_READY: ready, PROBE_CHILD_READY: `${ready}.child` },
      });
      let output = "";
      child.stdout.on("data", (chunk: Buffer) => {
        output += chunk.toString();
      });
      child.stderr.on("data", (chunk: Buffer) => {
        output += chunk.toString();
      });
      const exit = new Promise<{ code: number | null; signal: string | null }>(
        (resolve, reject) => {
          child.once("error", reject);
          child.once("exit", (code, signal) => resolve({ code, signal }));
        },
      );
      let pids:
        | { test: number; pgid: number; nextest: number; nextest_pgid: number; descendant: number }
        | undefined;
      try {
        const deadline = performance.now() + 10_000;
        while (!existsSync(ready)) {
          assert.equal(child.exitCode, null, output);
          assert.ok(performance.now() < deadline, output);
          await delay(20);
        }
        pids = JSON.parse(readFileSync(ready, "utf8"));
        assert.ok(pids);
        assert.equal(pids.test, pids.pgid);
        assert.notEqual(pids.pgid, pids.nextest_pgid, "nextest test has a separate process group");
        child.kill(signal); // Signal the actual Actions entry PID, never a discovered descendant.
        const result = await Promise.race([
          exit,
          delay(7_000, undefined, { ref: false }).then(() => "timeout"),
        ]);
        assert.deepEqual(result, { code: signal === "SIGINT" ? 130 : 143, signal: null }, output);
        assert.match(output, /Killing due to second signal/);
        for (const pid of [pids.test, -pids.pgid, pids.nextest, pids.descendant]) {
          assert.equal(
            signalIfPresent(pid, 0),
            false,
            `${signal}: process/group ${pid} survived\n${output}`,
          );
        }
        const timing = JSON.parse(
          readFileSync(join(directory, "target/nextest/pr/timing.json"), "utf8"),
        );
        assert.equal(timing.exit_code, signal === "SIGINT" ? 130 : 143);
      } finally {
        if (pids) signalIfPresent(-pids.pgid, "SIGKILL");
        if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
        await exit;
      }
    }
  },
);
