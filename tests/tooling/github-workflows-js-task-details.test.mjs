import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

const helper = fileURLToPath(
  new URL(
    "../../.github/actions/test-js-packages-with-history/native-fault-evidence.ts",
    import.meta.url,
  ),
);
const importHelper = `const fault = await import(${JSON.stringify(pathToFileURL(helper).href)});
const unavailable = () => { const error = new Error("typed observation gap"); error.name = "PermissionError"; throw error; };
fault.observers.privateGuest = () => ({ privateFirecrackerGuest: false });
fault.observers.sourceSnapshot = unavailable;
`;
const environment = (fixture) => ({
  ...process.env,
  GITHUB_ACTIONS: "true",
  GITHUB_RUN_ID: "123",
  GITHUB_RUN_ATTEMPT: "1",
  GITHUB_JOB: "law",
  RUNNER_TEMP: fixture,
});
const receiptAt = (fixture) =>
  JSON.parse(
    readFileSync(join(fixture, "js-native-fault-evidence-123-1-law/outcome.json"), "utf8"),
  );

for (const [originalStatus, detailsStatus] of [
  [0, 0],
  [9, 0],
  [11, 23],
  [13, "SIGTERM"],
  [17, "unavailable"],
]) {
  void test(`failed task summary preserves original ${originalStatus} with details ${detailsStatus}`, () => {
    const fixture = mkdtempSync(join(tmpdir(), "vize-js-task-details-"));
    try {
      const calls = join(fixture, "calls.jsonl");
      writeFileSync(
        join(fixture, "vp"),
        `#!${process.execPath}
import fs from "node:fs";
const args = process.argv.slice(2);
fs.appendFileSync(${JSON.stringify(calls)}, JSON.stringify(args) + "\\n");
if (args.includes("--last-details")) {
  process.stdout.write("task child exit=17\\n");
  process.stderr.write("task stderr\\n");
  const status = ${JSON.stringify(detailsStatus)};
  if (typeof status === "string") process.kill(process.pid, status);
  else process.exit(status);
} else process.exit(${originalStatus});
`,
        { mode: 0o755 },
      );
      const command = ["vp", "run", "--workspace-root", "test:js:builders"];
      const source =
        importHelper +
        `fault.observers.observeDescendants = () => {};
${detailsStatus === "unavailable" ? "fault.observers.taskDetails = unavailable;" : ""}
process.exitCode = await fault.main(["--", ...${JSON.stringify(command)}], "linux");`;
      const result = spawnSync(process.execPath, ["--input-type=module", "-e", source], {
        cwd: fixture,
        env: { ...environment(fixture), PATH: `${fixture}:${process.env.PATH}` },
        encoding: "utf8",
      });
      assert.equal(result.status, originalStatus, result.stderr);
      assert.equal(result.stdout, "");
      assert.equal(result.stderr, "");
      assert.deepEqual(
        readFileSync(calls, "utf8").trim().split("\n").map(JSON.parse),
        originalStatus === 0 || detailsStatus === "unavailable"
          ? [command.slice(1)]
          : [command.slice(1), ["run", "--last-details"]],
      );
      const receipt = receiptAt(fixture);
      assert.deepEqual(
        {
          status: receipt.commandExitCode,
          signal: receipt.commandSignal,
          returnCode: receipt.commandReturnCode,
        },
        { status: originalStatus, signal: null, returnCode: originalStatus },
      );
      if (originalStatus === 0) {
        assert.equal("taskDetails" in receipt, false);
      } else if (detailsStatus === "unavailable") {
        assert.equal("taskDetails" in receipt, false);
        assert.equal(receipt.observationGaps["task-details"], "PermissionError");
      } else {
        const authority = (text) => ({
          bytes: Buffer.byteLength(text),
          sha256: createHash("sha256").update(text).digest("hex"),
          publicBytes: Buffer.byteLength(text),
          selection: "whole stream",
          truncated: false,
        });
        const expected = {
          argv: ["vp", "run", "--last-details"],
          status: detailsStatus === "SIGTERM" ? -15 : detailsStatus,
          stdout: "task child exit=17\n",
          stdoutAuthority: authority("task child exit=17\n"),
          stderr: "task stderr\n",
          stderrAuthority: authority("task stderr\n"),
        };
        assert.deepEqual(receipt.taskDetails, expected);
        assert.deepEqual(
          JSON.parse(
            readFileSync(join(fixture, "js-native-fault-evidence-123-1-law/task-details.json")),
          ),
          expected,
        );
      }
    } finally {
      rmSync(fixture, { recursive: true, force: true });
    }
  });
}
