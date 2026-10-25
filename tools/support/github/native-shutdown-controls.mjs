import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const directory = resolve(process.argv[2]);
const source = "crates/vize_canon/src/lsp_client/editor_lsp/native_shutdown.rs";
const original = readFileSync(source, "utf8");
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const environment = { ...process.env };
delete environment.VIZE_TEST_DISABLE_TSGO;

function run(stem, selector, expectedStatus) {
  const argv = [
    "test", "--locked", "--profile", "ci", "-p", "vize_canon", "--lib", selector,
    "--", "--nocapture", "--test-threads=1", "--color", "never",
  ];
  const result = spawnSync("cargo", argv, {
    env: environment,
    timeout: 900000,
    maxBuffer: 32 * 1024 * 1024,
  });
  const stdout = result.stdout ?? Buffer.alloc(0);
  const stderr = result.stderr ?? Buffer.alloc(0);
  writeFileSync(join(directory, `${stem}.stdout`), stdout);
  writeFileSync(join(directory, `${stem}.stderr`), stderr);
  writeFileSync(join(directory, `${stem}.json`), JSON.stringify({
    source: process.env.SOURCE_SHA,
    productSourceSha256: sha256(readFileSync(source)),
    command: { executable: "cargo", argv },
    observation: {
      exitCode: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
      wholeStdout: stdout.toString("utf8"),
      wholeStderr: stderr.toString("utf8"),
      stdoutSha256: sha256(stdout),
      stderrSha256: sha256(stderr),
    },
  }, null, 2) + "\n");
  process.stdout.write(stdout);
  process.stderr.write(stderr);
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, expectedStatus);
  return stdout.toString("utf8");
}

const positive = run("shutdown-controls", "native_shutdown::", 0);
assert.match(positive, /test result: ok\. \d+ passed; 0 failed; 0 ignored; /);
const current = [
  "    let events = client.subscribe();\n",
  "    let completed = acknowledged.and_then(|()| completion::drain(&events));\n",
  "    completed.and(closed)\n",
];
for (const fragment of current) assert.equal(original.split(fragment).length, 2);
const oldClose = original
  .replace(current[0], "")
  .replace(current[1], "")
  .replace(current[2], "    acknowledged.and(closed)\n");
writeFileSync(join(directory, "negative-control-original.rs"), original);
writeFileSync(join(directory, "negative-control-old-close.rs"), oldClose);
try {
  writeFileSync(source, oldClose);
  const negative = run(
    "shutdown-old-close-negative",
    "lsp_client::editor_lsp::native_shutdown::tests::completion::malformed_missing_stale_and_eof_completion_still_close_the_owned_process",
    101,
  );
  assert.match(negative, /test result: FAILED\. 0 passed; 1 failed; 0 ignored; /);
  assert.match(negative, /malformed_missing_stale_and_eof_completion_still_close_the_owned_process \.\.\. FAILED/);
  const stderr = readFileSync(join(directory, "shutdown-old-close-negative.stderr"), "utf8");
  assert.match(stderr, /called `Result::unwrap_err\(\)` on an `Ok` value: \(\)/);
} finally {
  writeFileSync(source, original);
}
assert.equal(readFileSync(source, "utf8"), original);
