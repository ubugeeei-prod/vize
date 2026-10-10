import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { repoRoot, resolveMoonCommand, runMoonScript } from "../_helpers/moonbit.ts";
import { writeFakeCommand } from "./fake-command.ts";

export interface RegistryEvent {
  kind: string;
  args: string[];
  pid: number;
  marker: string | null;
  cargo: string | null;
  shellPid?: number;
  wrapperPid?: number;
  shellCommand?: string;
  completionDirectories: string[];
}

/** Only registry transport and the mutating Rust driver are controlled. */
export function releaseRegistryFixture(
  updateStatus = 0,
  authenticate = false,
  updateSignal = false,
  shellSignal: "SIGTERM" | "SIGKILL" | null = null,
) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-registry-"));
  const bin = path.join(dir, "bin");
  const log = path.join(dir, "events.jsonl");
  const temp = path.join(dir, "tmp");
  const cargo = '[workspace.package]\nversion = "0.440.0"\n';
  fs.mkdirSync(bin);
  fs.mkdirSync(temp);
  fs.mkdirSync(path.join(dir, "npm"));
  fs.writeFileSync(path.join(dir, "Cargo.toml"), cargo);
  fs.writeFileSync(log, "");
  // The updater authenticates its lexical CLI path against import.meta.url.
  // A symlinked tools tree would skip its main body, so keep its bytes at a
  // regular fixture path while controlling only the child Moon operation.
  for (const source of [
    "tools/support/release/moon-registry-update.mjs",
    "tools/commands/ci/github/release-local-guard.rs",
  ]) {
    fs.mkdirSync(path.dirname(path.join(dir, source)), { recursive: true });
    fs.copyFileSync(path.join(repoRoot, source), path.join(dir, source));
  }
  const record = `
    const fs = require('node:fs');
    const args = process.argv.slice(2);
    const record = (kind, details = {}) => fs.appendFileSync(process.env.REGISTRY_EVENTS,
      JSON.stringify({kind, args, pid: process.pid, ...details,
        marker: process.env.VIZE_RELEASE_REGISTRY_REFRESH ?? null,
        completionDirectories: fs.readdirSync(process.env.TMPDIR).filter(name => name.startsWith('vize-release-registry-')),
        cargo: fs.existsSync('Cargo.toml') ? fs.readFileSync('Cargo.toml', 'utf8') : null}) + '\\n');
  `;
  writeFakeCommand(
    bin,
    "moon",
    `${record}
    if (args[0] === 'update') {
      if (args.length !== 1) process.exit(99);
      let details = {};
      if (${JSON.stringify(shellSignal)}) {
        const execFileSync = require('node:child_process').execFileSync;
        const wrapper = process.ppid;
        const shellPid = Number(execFileSync('ps', ['-p', String(wrapper), '-o', 'ppid='], {encoding: 'utf8'}).trim());
        const wrapperCommand = execFileSync('ps', ['-p', String(wrapper), '-o', 'command='], {encoding: 'utf8'});
        const shellCommand = execFileSync('ps', ['-p', String(shellPid), '-o', 'command='], {encoding: 'utf8'});
        if (!Number.isSafeInteger(shellPid) || shellPid <= 1 || shellPid === wrapper || shellPid === process.pid ||
            !wrapperCommand.includes('moon-registry-update.mjs') || !shellCommand.includes('node "$@"')) {
          throw new Error('the exact owned wrapper/shell ancestry was not found');
        }
        details = {shellPid, shellCommand, wrapperPid: wrapper};
      }
      record('update', details);
      require('node:fs').writeSync(1, 'registry stdout\\n');
      require('node:fs').writeSync(2, 'registry stderr\\n');
      if (${updateSignal}) process.kill(process.pid, 'SIGTERM');
      if (${JSON.stringify(shellSignal)}) {
        process.kill(details.shellPid, ${JSON.stringify(shellSignal)});
        // Keep the update alive after its parent shell dies. An unchecked
        // WEXITSTATUS(0) must not authorize the release driver during this gap.
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 250);
      }
      process.exit(${updateStatus});
    }
    // Delegate the complete original argv to the real pinned compiler. The
    // release parser, controlling-terminal prompt and gate remain native.
    const result = require('node:child_process').spawnSync(
      process.env.REAL_MOON_BIN, args, {stdio: 'inherit'});
    if (result.error) throw result.error;
    if (result.signal) process.kill(process.pid, result.signal);
    else process.exitCode = result.status ?? 1;
  `,
  );
  writeFakeCommand(
    bin,
    "rust-script",
    `${record}
    record(args[1] === 'validate-preparation-target' ? 'authenticate' : 'driver');
    process.exit(${authenticate} && args[1] === 'validate-preparation-target' ? 0 : 83);
  `,
  );
  const realMoon = resolveMoonCommand(process.env);
  const env: NodeJS.ProcessEnv = {
    ...process.env,
    PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
    MOON_BIN: path.join(bin, process.platform === "win32" ? "moon.cmd" : "moon"),
    REAL_MOON_BIN: realMoon,
    REGISTRY_EVENTS: log,
    TMPDIR: temp,
    TMP: temp,
    TEMP: temp,
    VIZE_RELEASE_REGISTRY_REFRESH: "1",
  };
  if (realMoon === path.join(repoRoot, ".cache/moonbit/bin/moon")) {
    env.MOON_HOME = path.join(repoRoot, ".cache/moonbit");
  }
  const events = (): RegistryEvent[] =>
    fs
      .readFileSync(log, "utf8")
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line));
  const run = (args: string[]) => runMoonScript("release", args, { cwd: dir, env });
  const prompt = (answer: string, args: string[]) =>
    spawnSync(
      "python3",
      [
        path.join(repoRoot, "tests/tooling/support/pty-command.py"),
        "Proceed with release? [y/N]",
        answer,
        env.MOON_BIN!,
        "run",
        "-q",
        "--target",
        "native",
        path.join(repoRoot, "tools/moon/cmd/release"),
        "--",
        ...args,
      ],
      { cwd: dir, env, encoding: "utf8", timeout: 30_000 },
    );
  return {
    dir,
    env,
    cargo,
    events,
    run,
    prompt,
    assertUnchanged() {
      assert.equal(fs.readFileSync(path.join(dir, "Cargo.toml"), "utf8"), cargo);
    },
    dispose() {
      // A killed shell leaves its updater running until the controlled child
      // exits. Verify all exact owned PIDs disappear (including zombies) before
      // removing the fixture. The original updater remains unchanged.
      const targets = events()
        .filter((event) => event.shellPid)
        .flatMap((event) => [event.pid, event.wrapperPid!, event.shellPid!]);
      for (const pid of targets) {
        const deadline = Date.now() + 3_000;
        let state;
        do {
          state = spawnSync("ps", ["-p", String(pid), "-o", "pid=,command="], { encoding: "utf8" });
          if (state.status === 1 && state.stdout.trim() === "") break;
          Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 10);
        } while (Date.now() < deadline);
        assert.equal(state?.status, 1, `owned child ${pid} was not reaped: ${state?.stdout}`);
        assert.equal(state.stdout.trim(), "");
      }
      assert.deepEqual(
        fs.readdirSync(temp).filter((name) => name.startsWith("vize-release-registry-")),
        [],
        "the invocation's completion directory must be removed on success and failure",
      );
      fs.rmSync(dir, { recursive: true, force: true });
    },
  };
}
