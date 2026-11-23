import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { installToolingProgram, toolingProgram } from "./support/program-path.ts";

const environment = (fixture: string) => ({
  ...process.env,
  GITHUB_ACTIONS: "true",
  GITHUB_RUN_ID: "123",
  GITHUB_RUN_ATTEMPT: "1",
  GITHUB_JOB: "law",
  RUNNER_TEMP: fixture,
});
const receiptAt = (fixture: string) =>
  JSON.parse(
    readFileSync(join(fixture, "js-native-fault-evidence-123-1-law/outcome.json"), "utf8"),
  ) as {
    commandExitCode: number;
    commandSignal: string | null;
    commandReturnCode: number;
    taskDetails?: unknown;
    observationGaps: Record<string, string>;
  };

const statuses: Array<readonly [number, number | "SIGTERM" | "unavailable"]> = [
  [0, 0],
  [9, 0],
  [11, 23],
  [13, "SIGTERM"],
  [17, "unavailable"],
];
for (const [originalStatus, detailsStatus] of statuses) {
  void test(`failed task summary preserves original ${originalStatus} with details ${detailsStatus}`, () => {
    const fixture = mkdtempSync(join(tmpdir(), "vize-js-task-details-"));
    try {
      const calls = join(fixture, "calls.jsonl");
      installToolingProgram(join(fixture, "vp"), "workflow/mock-task-details.js");
      const command = ["vp", "run", "--workspace-root", "test:js:builders"];
      const result = spawnSync(
        process.execPath,
        [
          toolingProgram("workflow/run-observed-command.js"),
          detailsStatus === "unavailable" ? "task-details-unavailable" : "task-details",
          ...command,
        ],
        {
          cwd: fixture,
          env: {
            ...environment(fixture),
            VIZE_DETAILS_STATUS: JSON.stringify(detailsStatus),
            VIZE_ORIGINAL_STATUS: String(originalStatus),
            PATH: `${fixture}:${process.env.PATH}`,
          },
          encoding: "utf8",
        },
      );
      assert.equal(result.status, originalStatus, result.stderr);
      assert.equal(result.stdout, "");
      assert.equal(result.stderr, "");
      assert.deepEqual(
        readFileSync(calls, "utf8")
          .trim()
          .split("\n")
          .map((line) => JSON.parse(line) as string[]),
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
        const authority = (text: string) => ({
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
            readFileSync(
              join(fixture, "js-native-fault-evidence-123-1-law/task-details.json"),
              "utf8",
            ),
          ),
          expected,
        );
      }
    } finally {
      rmSync(fixture, { recursive: true, force: true });
    }
  });
}
