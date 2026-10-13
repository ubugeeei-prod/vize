import assert from "node:assert/strict";
import childProcess, { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, mock } from "node:test";
import { syncBuiltinESMExports } from "node:module";
import { primeStarCounterpartHistory } from "../../tools/support/compat/github/source-counterpart-snapshot.mjs";
const git = (owner, args, input) =>
  execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: owner,
    input,
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" },
  });
function coldPrimeFixture(run) {
  const owner = fs.mkdtempSync(path.join(os.tmpdir(), "vize-star-prime-mutex-"));
  try {
    git(owner, ["init", "--quiet"]);
    run(owner, path.join(owner, ".git/objects/info/star-counterpart-prime.lock"));
  } finally {
    mock.restoreAll();
    syncBuiltinESMExports();
    fs.rmSync(owner, { recursive: true, force: true });
  }
}
await test("a dead owned priming PID can be recovered without accepting a failed strict worker", () => {
  coldPrimeFixture((owner, lock) => {
    const dead = childProcess.spawnSync(process.execPath, ["-e", ""]);
    fs.writeFileSync(lock, dead.pid + ":1", { mode: 0o600 });
    let calls = 0;
    mock.method(childProcess, "spawnSync", (_command, args, options) => {
      calls++;
      assert(args[2].includes("ensureHostedLiteralSource"));
      assert(options.timeout <= 1500);
      return { status: 1, pid: undefined };
    });
    syncBuiltinESMExports();
    assert.throws(
      () => primeStarCounterpartHistory(owner, { timeoutMs: 1500 }),
      /bounded strict literal/u,
    );
    assert.equal(calls, 1);
    assert.equal(fs.existsSync(lock), false);
  });
});
await test("a token ENOSPC failure still removes the lock acquired before its write", () => {
  coldPrimeFixture((owner, lock) => {
    const original = fs.writeFileSync;
    mock.method(fs, "writeFileSync", (target, ...args) => {
      if (typeof target === "number")
        throw Object.assign(new Error("controlled ENOSPC"), { code: "ENOSPC" });
      return original(target, ...args);
    });
    assert.throws(() => primeStarCounterpartHistory(owner), { code: "ENOSPC" });
    assert.equal(fs.existsSync(lock), false);
  });
});
await test("the owner worker deadline kills its real process group and preserves timeout failure", () => {
  coldPrimeFixture((owner, lock) => {
    const previousObservation = process.env.VIZE_TOOLING_LISTING_OBSERVATION_DIR;
    process.env.VIZE_TOOLING_LISTING_OBSERVATION_DIR = path.join(owner, "observation");
    const original = childProcess.spawnSync,
      mark = path.join(owner, "descendant.pid");
    const worker = `const {spawn}=require('node:child_process'); const fs=require('node:fs'); const d=spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{stdio:'ignore'}); fs.writeFileSync(${JSON.stringify(mark)},String(d.pid)); setInterval(()=>{},1000);`;
    mock.method(childProcess, "spawnSync", (command, args, options) => {
      assert(args[2].includes("ensureHostedLiteralSource"));
      assert(options.detached);
      assert(options.timeout <= 1500);
      return original(command, ["-e", worker], options);
    });
    syncBuiltinESMExports();
    try {
      assert.throws(() => primeStarCounterpartHistory(owner, { timeoutMs: 1500 }), {
        code: "ETIMEDOUT",
      });
      const events = fs
        .readFileSync(
          path.join(owner, "observation", "history-prime-" + process.pid + ".jsonl"),
          "utf8",
        )
        .trim()
        .split("\n")
        .map((line) => JSON.parse(line));
      assert.equal(events[0].event, "missing-history");
      const result = events.find((event) => event.event === "worker-result");
      assert.equal(result.status, null);
      assert.equal(result.signal, "SIGKILL");
      assert.equal(result.error.code, "ETIMEDOUT");
      assert(
        events.some((event) => event.event === "cleanup-signal" && event.group === result.group),
      );
    } finally {
      if (previousObservation === undefined)
        delete process.env.VIZE_TOOLING_LISTING_OBSERVATION_DIR;
      else process.env.VIZE_TOOLING_LISTING_OBSERVATION_DIR = previousObservation;
    }
    const pid = Number(fs.readFileSync(mark, "utf8"));
    const state = original("ps", ["-o", "stat=", "-p", String(pid)], { encoding: "utf8" });
    assert(state.status !== 0 || state.stdout.trim().startsWith("Z"));
    assert.equal(fs.existsSync(lock), false);
  });
});
