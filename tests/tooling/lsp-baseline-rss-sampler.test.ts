import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { owned, parseStat } from "../performance/support/current-baseline/sample-rss.ts";

const samplerPath = path.resolve(
  import.meta.dirname,
  "../performance/support/current-baseline/sample-rss.ts",
);

void test("sampler ownership keeps reparented children and refuses reused root PID", () => {
  const row = (startTicks: string, parentPid: number) => ({ startTicks, parentPid });
  const tree = new Map([
    [1, row("a", 0)],
    [2, row("b", 1)],
    [3, row("c", 2)],
  ]);
  assert.deepEqual(owned(tree, 1, "a", new Map([[1, "a"]])), [1, 2, 3]);
  const reparented = new Map([[2, row("b", 0)]]);
  const known = new Map([
    [1, "a"],
    [2, "b"],
  ]);
  assert.deepEqual(owned(reparented, 1, "a", known), [2]);
  const recycled = new Map([[2, row("other", 0)]]);
  assert.deepEqual(owned(recycled, 1, "a", known), [], "a recycled child PID is not owned");
  const reused = new Map([[1, row("new", 0)]]);
  assert.throws(() => owned(reused, 1, "a", new Map([[1, "a"]])), /root PID was reused/u);
});

void test("a stat line is read after its command name, whatever the name contains", () => {
  const tail =
    "S 7 9 9 0 -1 4194304 1 0 0 0 2 3 0 0 20 0 1 0 4242 1000000 250 18446744073709551615";
  assert.deepEqual(parseStat(9, `9 (node) ${tail}`, 4096), {
    pid: 9,
    parentPid: 7,
    startTicks: "4242",
    state: "S",
    rssKiB: 1000,
  });
  assert.equal(parseStat(9, `9 (a) b (c)) ${tail}`, 16384).rssKiB, 4000);
  assert.throws(() => parseStat(9, "9 (node) S 7", 4096), /unexpected stat line/u);
});

void test(
  "the RSS sampler observes a real process and finishes cleanly on stop",
  { skip: process.platform !== "linux" },
  async () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "lsp-baseline-rss-"));
    const child = spawn("sleep", ["30"], { stdio: "ignore" });
    try {
      assert.ok(child.pid);
      const stat = fs.readFileSync(`/proc/${child.pid}/stat`, "utf8");
      const ticks = stat.slice(stat.lastIndexOf(")") + 2).split(" ")[19];
      const destination = path.join(directory, "rss.jsonl");
      const sampler = spawn(
        process.execPath,
        [samplerPath, String(child.pid), ticks, destination],
        {
          stdio: ["pipe", "ignore", "pipe"],
        },
      );
      let stderr = "";
      sampler.stderr.on("data", (bytes: Buffer) => {
        stderr += bytes.toString("utf8");
      });
      const closed = new Promise<number | null>((resolve) => sampler.once("close", resolve));
      // Wait until the sampler has written a sample that contains the live root.
      const sampled = (): boolean =>
        fs.existsSync(destination) &&
        fs.readFileSync(destination, "utf8").includes(`"kind":"sample","ns":`) &&
        fs.readFileSync(destination, "utf8").includes(`{"pid":${child.pid},`);
      for (let attempt = 0; attempt < 200 && !sampled(); attempt += 1) {
        await new Promise((resolve) => setTimeout(resolve, 50));
      }
      assert.ok(sampled(), `the sampler never observed the root: ${stderr}`);
      const exited = new Promise((resolve) => child.once("exit", resolve));
      child.kill("SIGKILL");
      await exited;
      sampler.stdin.end("stop\n");
      assert.equal(await closed, 0, stderr);
      const rows = fs
        .readFileSync(destination, "utf8")
        .trimEnd()
        .split("\n")
        .map((line) => JSON.parse(line) as Record<string, unknown>);
      assert.deepEqual(
        { kind: rows[0].kind, pid: rows[0].pid, startTicks: rows[0].startTicks },
        { kind: "start", pid: child.pid, startTicks: ticks },
      );
      const observed = rows
        .filter((sample) => sample.kind === "sample")
        .flatMap((sample) => sample.members as Record<string, unknown>[])
        .filter((member) => member.pid === child.pid);
      assert.ok(observed.length > 0, "the live root is sampled");
      for (const member of observed) {
        assert.equal(member.startTicks, ticks);
        assert.ok(Number(member.rssKiB) > 0);
        assert.match(String(member.executable), /sleep$/u);
        assert.match(String(member.executableSha256), /^[0-9a-f]{64}$/u);
      }
      const finish = rows.at(-1);
      assert.deepEqual(
        [finish?.kind, finish?.stopped, finish?.failure, finish?.survivorsBeforeForcedCleanup],
        ["finish", true, null, []],
      );
    } finally {
      child.kill("SIGKILL");
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
