import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { downloadAttempts } from "../../editors/vscode/test/download-attempts.mjs";

function fixture(t, body) {
  const directory = mkdtempSync(join(tmpdir(), "vize-vscode-download-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const script = join(directory, "attempt.mjs");
  writeFileSync(script, body);
  return { directory, script };
}

await test("a failed acquisition exits before a genuine successful retry", async (t) => {
  const { directory, script } = fixture(
    t,
    `
    import { existsSync, writeFileSync } from "node:fs";
    const mark = process.argv[2];
    if (!existsSync(mark)) { writeFileSync(mark, "aborted"); process.exit(1); }
    writeFileSync(mark, "actual-success");
  `,
  );
  const mark = join(directory, "attempt-state");
  assert.equal(await downloadAttempts(process.execPath, [script, mark]), 0);
  assert.equal(readFileSync(mark, "utf8"), "actual-success");
});

await test("exhausted real failures retain the final nonzero exit", async (t) => {
  const { script } = fixture(t, "process.exit(7)");
  assert.equal(await downloadAttempts(process.execPath, [script]), 7);
});

await test("a hung acquisition and its descendant are reaped within the original budget", async (t) => {
  const { directory, script } = fixture(
    t,
    `
    import { spawn } from "node:child_process";
    import { writeFileSync } from "node:fs";
    const descendant = spawn(process.execPath, ["-e", "setInterval(()=>{}, 1000)"], { stdio: "ignore" });
    writeFileSync(process.argv[2], String(descendant.pid));
    process.on("SIGTERM", () => {});
    setInterval(() => {}, 1000);
  `,
  );
  const pidPath = join(directory, "descendant-pid");
  const start = performance.now();
  assert.equal(
    await downloadAttempts(process.execPath, [script, pidPath], {
      attempts: 1,
      attemptTimeoutMs: 200,
      totalTimeoutMs: 1_000,
      teardownMs: 20,
    }),
    124,
  );
  assert(performance.now() - start < 1_000);
  const pid = Number(readFileSync(pidPath, "utf8"));
  // A killed Linux descendant may briefly remain a zombie until PID1 reaps it.
  const state = spawnSync("ps", ["-o", "stat=", "-p", String(pid)], { encoding: "utf8" });
  assert(state.status !== 0 || state.stdout.trim().startsWith("Z"));
});

await test("both original child streams survive retries without rewriting", (t) => {
  const { directory, script } = fixture(
    t,
    `
    import { writeSync } from "node:fs";
    writeSync(1, Buffer.from([0x00, 0xff, 0x0a]));
    writeSync(2, "aborted: original failure\\n");
    process.exit(1);
  `,
  );
  const driver = join(directory, "driver.mjs");
  const helper = new URL("../../editors/vscode/test/download-attempts.mjs", import.meta.url).href;
  writeFileSync(
    driver,
    `import { downloadAttempts } from ${JSON.stringify(helper)};
process.exitCode = await downloadAttempts(process.execPath, [${JSON.stringify(script)}]);`,
  );
  const result = spawnSync(process.execPath, [driver]);
  assert.equal(result.status, 1);
  assert.deepEqual(result.stdout, Buffer.from([0x00, 0xff, 0x0a, 0x00, 0xff, 0x0a]));
  assert.equal(result.stderr.toString().split("aborted: original failure\n").length, 3);
});

await test("the total deadline prevents a hanging retry from extending acquisition", async (t) => {
  const { script } = fixture(t, "process.on('SIGTERM', () => {}); setInterval(() => {}, 1000)");
  const start = performance.now();
  assert.equal(
    await downloadAttempts(process.execPath, [script], {
      attemptTimeoutMs: 100,
      totalTimeoutMs: 170,
      teardownMs: 10,
    }),
    124,
  );
  assert(performance.now() - start < 300);
});
