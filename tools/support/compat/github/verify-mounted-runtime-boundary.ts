import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { closeSync, mkdirSync, openSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

const directory = resolve(process.argv[2] ?? "target/mounted-runtime-boundary");
mkdirSync(directory, { recursive: true });
const producerDirectory = join(directory, "producers");
mkdirSync(producerDirectory, { recursive: true });
assert.deepEqual(readdirSync(producerDirectory), [], "producer evidence must start empty");
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const sources = [
  "tests/tooling/mounted-runtime-source.test.ts",
  "tests/tooling/support/mounted-runtime-source.ts",
  "tests/tooling/support/mounted-runtime-producer.ts",
  "tests/tooling/support/mounted-runtime-native-boundary.ts",
  "tests/tooling/support/mounted-runtime-native-exit.mjs",
  "tests/tooling/support/davinci-mounted-trace.mjs",
  "tests/tooling/support/patterned-template-runtime.mjs",
  "tests/tooling/support/vue-vapor-release.mjs",
  "tools/support/compat/github/verify-mounted-runtime-boundary.ts",
];
writeFileSync(
  join(directory, "source.json"),
  `${JSON.stringify({
    node: { executable: process.execPath, version: process.version },
    source: process.env.GITHUB_SHA ?? null,
    sources: Object.fromEntries(sources.map((file) => [file, hash(readFileSync(file))])),
  })}\n`,
);

const outcomes = [];
for (const [name, command] of [
  ["node-laws", ["--test", "--test-reporter=tap", "tests/tooling/mounted-runtime-source.test.ts"]],
  ["native-exit", ["tests/tooling/support/mounted-runtime-native-exit.mjs"]],
] as const) {
  const errors: string[] = [];
  const streams = Object.fromEntries(
    ["stdout", "stderr"].map((stream) => [
      stream,
      {
        fd: openSync(join(directory, `${name}.${stream}.bin`), "wx"),
        hash: createHash("sha256"),
        bytes: 0,
      },
    ]),
  );
  const child = spawn(process.execPath, [...command], {
    env: {
      ...process.env,
      VIZE_MOUNTED_RUNTIME_PRODUCER_EVIDENCE: name === "native-exit" ? producerDirectory : "",
    },
    stdio: ["pipe", "pipe", "pipe"],
  });
  for (const stream of ["stdout", "stderr"] as const) {
    child[stream].on("data", (chunk: Buffer) => {
      const target = streams[stream];
      target.hash.update(chunk);
      target.bytes += chunk.length;
      try {
        writeFileSync(target.fd, chunk);
      } catch (error) {
        errors.push(`${stream}: ${String(error)}`);
      }
    });
    child[stream].on("error", (error) => errors.push(`${stream}: ${error.message}`));
  }
  child.on("error", (error) => errors.push(error.message));
  child.stdin.on("error", (error) => errors.push(`stdin: ${error.message}`));
  const status = new Promise<{ exitCode: number | null; signal: string | null }>((resolve) =>
    child.once("close", (exitCode, signal) => resolve({ exitCode, signal })),
  );
  child.stdin.end();
  const outcome = {
    name,
    command,
    pid: child.pid ?? null,
    ...(await status),
    errors,
    stdout: { bytes: streams.stdout.bytes, sha256: streams.stdout.hash.digest("hex") },
    stderr: { bytes: streams.stderr.bytes, sha256: streams.stderr.hash.digest("hex") },
  };
  for (const stream of Object.values(streams)) {
    try {
      closeSync(stream.fd);
    } catch (error) {
      errors.push(`close: ${String(error)}`);
    }
  }
  outcomes.push(outcome);
  writeFileSync(join(directory, "outcomes.json"), `${JSON.stringify(outcomes)}\n`);
}

// Retain both commands and their complete streams before applying either gate.
for (const outcome of outcomes) {
  assert.deepEqual(outcome.errors, [], `${outcome.name}: complete stream capture is required`);
  assert.equal(outcome.exitCode, 0, `${outcome.name}: natural exit must be 0`);
  assert.equal(outcome.signal, null, `${outcome.name}: no signal is acceptable`);
  assert.equal(outcome.stderr.bytes, 0, `${outcome.name}: stderr must stay empty`);
}
const tap = readFileSync(join(directory, "node-laws.stdout.bin"), "utf8");
assert.deepEqual(
  [...tap.matchAll(/^ok \d+ - (.+)$/gmu)].map((match) => match[1]),
  [
    "one generated-code promise per mode preserves identity and never retries failure",
    "producer acceptance requires the complete canonical packet and strict natural exit",
  ],
);
for (const [key, count] of Object.entries({
  tests: 2,
  pass: 2,
  fail: 0,
  cancelled: 0,
  skipped: 0,
  todo: 0,
}))
  assert.match(tap, new RegExp(`^# ${key} ${count}$`, "mu"));
assert.deepEqual(
  readFileSync(join(directory, "native-exit.stdout.bin")),
  Buffer.from('{"passed":1}'),
);

const producerFiles = readdirSync(producerDirectory).filter((file) => file.endsWith(".json"));
assert.equal(producerFiles.length, 2, "both repeated modes must create exactly two producers");
const producers = producerFiles.map((file) => {
  const receipt = JSON.parse(readFileSync(join(producerDirectory, file), "utf8"));
  const name = file.slice(0, -".json".length);
  const stdout = readFileSync(join(producerDirectory, `${name}.stdout.bin`));
  const stderr = readFileSync(join(producerDirectory, `${name}.stderr.bin`));
  assert.equal(receipt.exitCode, 0);
  assert.equal(receipt.signal, null);
  assert.equal(receipt.spawnError, null);
  assert.equal(receipt.inputError, null);
  assert.deepEqual(receipt.stdout, { bytes: stdout.length, sha256: hash(stdout) });
  assert.deepEqual(receipt.stderr, { bytes: stderr.length, sha256: hash(stderr) });
  assert.equal(stderr.length, 0);
  const packet = JSON.parse(stdout.toString("utf8"));
  assert.deepEqual(Object.keys(packet), ["code"]);
  assert.equal(typeof packet.code, "string");
  assert.ok(packet.code.length > 0);
  assert.deepEqual(stdout, Buffer.from(`${JSON.stringify(packet)}\n`));
  return { ...receipt, codeSha256: hash(Buffer.from(packet.code)) };
});
assert.deepEqual(
  producers.map(({ production }) => production).sort((left, right) => Number(left) - Number(right)),
  [false, true],
);
assert.equal(new Set(producers.map(({ pid }) => pid)).size, 2);
assert.equal(new Set(producers.map(({ codeSha256 }) => codeSha256)).size, 2);
writeFileSync(
  join(directory, "summary.json"),
  `${JSON.stringify({ tests: 2, regressions: 1, producers, outcomes, accepted: true })}\n`,
);
