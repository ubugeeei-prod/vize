import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { installMapperCapture, mapperCaptureAsset } from "./template-alias-paired-11822-capture.ts";

void test("the shipped typed capture preserves input/output/errors and child exit status", (t) => {
  const root = mkdtempSync(join(tmpdir(), "mapper-capture-runtime-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const packageRoot = join(root, "consumer", "node_modules", "vize");
  mkdirSync(packageRoot, { recursive: true });
  assert.ok(readFileSync(mapperCaptureAsset).length > 0);
  const directory = join(root, "captured");
  const command = installMapperCapture(packageRoot, directory, [
    process.execPath,
    "-e",
    "process.stdin.pipe(process.stdout); process.stderr.write('original stderr'); process.exitCode = 3;",
  ]);
  const body = "日本語🌸";
  const input = Buffer.from(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
  const result = spawnSync(command[0], command.slice(1), { input, timeout: 5000 });
  assert.equal(result.error, undefined);
  assert.equal(result.status, 3);
  assert.ok(result.stdout.equals(input));
  assert.equal(result.stderr.toString(), "original stderr");
  const files = readdirSync(directory);
  for (const [prefix, expected] of [
    ["in-", input],
    ["out-", input],
    ["err-", result.stderr],
  ] as const) {
    const file = files.find((name) => name.startsWith(prefix))!;
    assert.ok(file);
    assert.ok(readFileSync(join(directory, file)).equals(expected));
  }
  const receipt = JSON.parse(
    readFileSync(
      join(
        directory,
        files.find((name) => name.startsWith("process-"))!,
      ),
      "utf8",
    ),
  );
  assert.equal(receipt.status, 3);
  assert.equal(receipt.error, null);
});
