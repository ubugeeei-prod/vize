import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { parseExecTrace } from "./exec-custody.mjs";
import { sha256, CORSA_OVERRIDES } from "./providers.mjs";

export function napiNativeExecutions(files, node, expected) {
  const frontend = fs.realpathSync(node);
  const selected = [];
  let entered = false;
  for (const file of files) {
    const events = parseExecTrace(file.text).filter((row) => row.success);
    for (const event of events) {
      const physicalPath = fs.realpathSync(event.executable);
      if (physicalPath === frontend) {
        entered = true;
        continue;
      }
      assert(expected, "missing-runtime controls must not execute a compiler");
      assert.equal(physicalPath, expected.path, "only the selected physical SDK may execute");
      const imageSha256 = sha256(fs.readFileSync(physicalPath));
      assert.equal(imageSha256, expected.sha256);
      const exit = /\+\+\+ exited with ([0-9]+) \+\+\+\s*$/u.exec(file.text);
      assert(exit, "complete SDK exit record is required");
      assert.equal(Number(exit[1]), 0, "qualified native API owner must close cleanly");
      assert(
        event.argv.includes("--api") || event.argv.at(-1) === "--version",
        "only API or version-probe capability is observed",
      );
      selected.push({
        ...event,
        physicalPath,
        imageSha256,
        exitCode: Number(exit[1]),
        trace: file.file,
      });
    }
  }
  assert(entered, "actual Node/NAPI frontend exec is required");
  if (expected)
    assert(
      selected.some((row) => row.argv.includes("--api")),
      "actual native API exec is required",
    );
  return selected;
}

export function runNapiObservation({ strace, worker, input, cwd, env, directory, expected }) {
  fs.mkdirSync(directory, { recursive: true });
  const stdin = Buffer.from(JSON.stringify(input));
  fs.writeFileSync(path.join(directory, "stdin.bin"), stdin);
  const argv = [
    "--kill-on-exit",
    "-ff",
    "-s",
    "65536",
    "-e",
    "trace=execve",
    "-o",
    path.join(directory, "execve"),
    process.execPath,
    worker,
  ];
  const result = spawnSync(strace, argv, {
    input: stdin,
    cwd,
    env,
    timeout: 45000,
    maxBuffer: 16 * 1024 * 1024,
  });
  fs.writeFileSync(path.join(directory, "stdout.bin"), result.stdout ?? Buffer.alloc(0));
  fs.writeFileSync(path.join(directory, "stderr.bin"), result.stderr ?? Buffer.alloc(0));
  const traces = fs
    .readdirSync(directory)
    .filter((name) => name.startsWith("execve."))
    .sort()
    .map((file) => {
      const bytes = fs.readFileSync(path.join(directory, file));
      return { file, bytes: bytes.length, sha256: sha256(bytes), text: bytes.toString("utf8") };
    });
  const receipt = {
    argv: [strace, ...argv],
    cwd,
    environment: {
      HOME: env.HOME,
      PATH: env.PATH,
      selectors: Object.fromEntries(CORSA_OVERRIDES.map((name) => [name, env[name] ?? null])),
    },
    status: result.status,
    signal: result.signal,
    error: result.error?.message ?? null,
    stdin: { bytes: stdin.length, sha256: sha256(stdin) },
    stdout: { bytes: result.stdout?.length ?? 0, sha256: sha256(result.stdout ?? Buffer.alloc(0)) },
    stderr: { bytes: result.stderr?.length ?? 0, sha256: sha256(result.stderr ?? Buffer.alloc(0)) },
    traces,
    selectedExecutions: [],
    state: "pending",
    meaning:
      "Actual source NAPI selection and whole diagnostic DTO; no new upstream API schema or performance credit.",
  };
  const save = () =>
    fs.writeFileSync(path.join(directory, "process.json"), `${JSON.stringify(receipt, null, 2)}\n`);
  save();
  try {
    assert.equal(receipt.error, null);
    assert.equal(receipt.signal, null);
    assert.equal(receipt.status, 0);
    assert.equal(result.stderr.toString("utf8"), "");
    for (const trace of traces) assert(trace.bytes <= 16 * 1024 * 1024);
    receipt.selectedExecutions = napiNativeExecutions(traces, process.execPath, expected);
    const packet = JSON.parse(result.stdout.toString("utf8"));
    assert.equal(result.stdout.toString("utf8"), `${JSON.stringify(packet)}\n`);
    assert.deepEqual(Object.keys(packet), [
      "schema",
      "version",
      "inputSha256",
      "addon",
      "source",
      "results",
    ]);
    assert.equal(packet.schema, "vize.native-runtime-napi-observation");
    assert.equal(packet.version, 1);
    assert.equal(packet.inputSha256, sha256(stdin));
    assert.deepEqual(packet.addon, input.addon);
    assert.equal(packet.source, input.source);
    receipt.state = "PASS";
    save();
    return { receipt, packet };
  } catch (error) {
    receipt.state = "FAILED";
    receipt.validationError = { message: String(error), stack: error?.stack ?? null };
    save();
    throw error;
  }
}
