import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import {
  MAPPER_HOST_TARGETS,
  mapperHostTarget,
} from "../../tools/support/release/type-admission-mapper-host.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

const proxy = path.join(root, "tools/support/compat/content-mapper/process-receipt.ts");

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function withDirectory(check: (directory: string) => void | Promise<void>): () => Promise<void> {
  return async () => {
    const directory = mkdtempSync(path.join(tmpdir(), "vize-mapper-process-"));
    try {
      await check(directory);
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  };
}

test("the qualified mapper host selects exact Linux and macOS arm64 images", () => {
  assert.equal(mapperHostTarget("linux", "x64"), MAPPER_HOST_TARGETS["linux-x64"]);
  assert.equal(mapperHostTarget("darwin", "arm64"), MAPPER_HOST_TARGETS["darwin-arm64"]);
  assert.match(MAPPER_HOST_TARGETS["darwin-arm64"].package.integrity, /^sha512-/);
  assert.match(MAPPER_HOST_TARGETS["darwin-arm64"].image.sha256, /^[0-9a-f]{64}$/);
  assert.throws(() => mapperHostTarget("darwin", "x64"), /unsupported mapper host target/);
});

test("the macOS cold-start workflow runs once and retains complete process evidence", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "content-mapper-cold-start.yml")) as {
    jobs: Record<
      string,
      {
        "runs-on": string;
        steps: Array<{
          name?: string;
          if?: string;
          run?: string;
          with?: Record<string, string>;
        }>;
      }
    >;
  };
  const job = workflow.jobs["macos-arm64"];
  assert.equal(job["runs-on"], "blacksmith-12vcpu-macos-15");
  const prepare = job.steps.find(
    (step) => step.name === "Prepare the exact macOS Content Mapper host",
  );
  assert.ok(prepare?.run?.includes("@typescript/typescript-darwin-arm64/lib/tsc"));
  assert.ok(prepare?.run?.includes("VIZE_TEST_MEMO_PROCESS_RECEIPT_DIR"));
  const capture = job.steps.find(
    (step) => step.name === "Capture the first post-build mapper process",
  );
  assert.ok(capture?.run?.includes("memo_dependencies_check_members_and_preserve_unused_controls"));
  assert.ok(capture?.run?.includes("--exact --nocapture --test-threads=1"));
  assert.doesNotMatch(capture?.run ?? "", /retry|attempt|sleep/iu);
  const upload = job.steps.find(
    (step) => step.name === "Upload mapper process and provider receipts",
  );
  assert.equal(upload?.if, "${{ always() }}");
  assert.ok(upload?.with?.path.includes("cold-content-mapper-receipts"));
  assert.equal(upload?.with?.["if-no-files-found"], "error");
});

test(
  "the mapper proxy preserves protocol bytes and records a nonzero child exit",
  withDirectory((directory) => {
    const receipt = path.join(directory, "receipt.json");
    const child = [
      "let body = '';",
      "process.stdin.setEncoding('utf8');",
      "process.stdin.on('data', (chunk) => { body += chunk; });",
      "process.stdin.on('end', () => {",
      "  process.stdout.write('OUT:' + body);",
      "  process.stderr.write('ERR');",
      "  process.exitCode = 23;",
      "});",
    ].join("\n");
    const result = spawnSync(process.execPath, [proxy, receipt, process.execPath, "-e", child], {
      input: "request\n",
      encoding: "utf8",
    });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 23);
    assert.equal(result.signal, null);
    assert.equal(result.stdout, "OUT:request\n");
    assert.equal(result.stderr, "ERR");

    const packet = JSON.parse(readFileSync(receipt, "utf8"));
    assert.equal(packet.schema, "vize.contentMapperProcessReceipt.v1");
    assert.equal(packet.lifecycle, "closed");
    assert.equal(typeof packet.finishedAt, "string");
    assert.deepEqual(packet.command.file, process.execPath);
    assert.deepEqual(packet.command.args, ["-e", child]);
    assert.equal(packet.spawnError, null);
    assert.deepEqual(packet.exit, { code: 23, signal: null });
    for (const [name, value] of [
      ["stdin", "request\n"],
      ["stdout", "OUT:request\n"],
      ["stderr", "ERR"],
    ] as const) {
      assert.deepEqual(
        {
          bytes: packet.streams[name].bytes,
          capturedBytes: packet.streams[name].capturedBytes,
          complete: packet.streams[name].complete,
          sha256: packet.streams[name].sha256,
          text: Buffer.from(packet.streams[name].base64, "base64").toString(),
        },
        {
          bytes: Buffer.byteLength(value),
          capturedBytes: Buffer.byteLength(value),
          complete: true,
          sha256: sha256(value),
          text: value,
        },
      );
    }
  }),
);

test(
  "the mapper proxy distinguishes an executable startup failure",
  withDirectory((directory) => {
    const receipt = path.join(directory, "receipt.json");
    const missing = path.join(directory, "missing-mapper");
    const result = spawnSync(process.execPath, [proxy, receipt, missing], { encoding: "utf8" });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 127);
    assert.equal(result.signal, null);
    const packet = JSON.parse(readFileSync(receipt, "utf8"));
    assert.equal(packet.spawnError.code, "ENOENT");
    assert.equal(packet.lifecycle, "closed");
    assert.equal(packet.exit.code, -2);
    assert.equal(packet.exit.signal, null);
  }),
);

test(
  "the mapper proxy records and preserves a child signal",
  { skip: process.platform === "win32" },
  withDirectory((directory) => {
    const receipt = path.join(directory, "receipt.json");
    const result = spawnSync(
      process.execPath,
      [proxy, receipt, process.execPath, "-e", 'process.kill(process.pid, "SIGTERM")'],
      { encoding: "utf8" },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.status, null);
    assert.equal(result.signal, "SIGTERM");
    const packet = JSON.parse(readFileSync(receipt, "utf8"));
    assert.equal(packet.spawnError, null);
    assert.equal(packet.lifecycle, "closed");
    assert.deepEqual(packet.exit, { code: null, signal: "SIGTERM" });
  }),
);

test(
  "the mapper proxy durably snapshots protocol bytes before host SIGKILL teardown",
  { skip: process.platform === "win32" },
  withDirectory(async (directory) => {
    const receipt = path.join(directory, "receipt.json");
    const child = [
      "process.stdin.once('data', (chunk) => process.stdout.write('ACK:' + chunk));",
      "process.stdin.resume();",
    ].join("\n");
    const proxyProcess = spawn(process.execPath, [proxy, receipt, process.execPath, "-e", child], {
      stdio: ["pipe", "pipe", "pipe"],
    });
    proxyProcess.stdin.write("request\n");
    const stdout = await new Promise<Buffer>((resolve) =>
      proxyProcess.stdout.once("data", resolve),
    );
    assert.equal(stdout.toString(), "ACK:request\n");
    proxyProcess.kill("SIGKILL");
    const [code, signal] = await new Promise<[number | null, NodeJS.Signals | null]>((resolve) =>
      proxyProcess.once("close", (closedCode, closedSignal) => resolve([closedCode, closedSignal])),
    );
    assert.equal(code, null);
    assert.equal(signal, "SIGKILL");

    const packet = JSON.parse(readFileSync(receipt, "utf8"));
    assert.equal(packet.lifecycle, "running");
    assert.equal(packet.finishedAt, null);
    assert.equal(packet.exit, null);
    assert.equal(packet.spawnError, null);
    assert.equal(packet.streams.stdin.utf8, "request\n");
    assert.equal(packet.streams.stdout.utf8, "ACK:request\n");
    assert.equal(packet.streams.stderr.utf8, "");
  }),
);
