import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
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

for (const stream of ["stdin", "stdout", "stderr"] as const) {
  void test(`a ${stream} source failure retains its receipt and terminates the owned child`, (t) => {
    const root = mkdtempSync(join(tmpdir(), "mapper-capture-source-error-"));
    t.after(() => rmSync(root, { recursive: true, force: true }));
    const packageRoot = join(root, "consumer", "node_modules", "vize");
    mkdirSync(packageRoot, { recursive: true });
    const directory = join(root, "captured");
    const command = installMapperCapture(packageRoot, directory, [
      process.execPath,
      "-e",
      "setInterval(() => {}, 1000)",
    ]);
    const message = `synthetic ${stream} source failure`;
    const preload = join(root, "fail-source.mjs");
    writeFileSync(
      preload,
      `import { createRequire, syncBuiltinESMExports } from 'node:module';
const require = createRequire(import.meta.url);
const childProcess = require('node:child_process');
const original = childProcess.spawn;
childProcess.spawn = (...args) => {
  const child = original(...args);
  const source = ${stream === "stdin" ? "process.stdin" : `child.${stream}`};
  setImmediate(() => source.emit('error', new Error(${JSON.stringify(message)})));
  return child;
};
syncBuiltinESMExports();
`,
    );
    const result = spawnSync(command[0], ["--import", preload, ...command.slice(1)], {
      input: Buffer.alloc(0),
      timeout: 5000,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.status, 1);
    assert.equal(result.stdout.toString(), "");
    assert.equal(result.stderr.toString(), "");
    const files = readdirSync(directory);
    const receipt = JSON.parse(
      readFileSync(
        join(
          directory,
          files.find((name) => name.startsWith("process-"))!,
        ),
        "utf8",
      ),
    );
    assert.equal(receipt.error, message);
    assert.equal(receipt.status, null);
    assert.equal(receipt.signal, "SIGTERM");
    for (const prefix of ["in-", "out-", "err-"]) {
      const file = files.find((name) => name.startsWith(prefix));
      assert.ok(file);
      assert.deepEqual(readFileSync(join(directory, file)), Buffer.alloc(0));
    }
  });
}
