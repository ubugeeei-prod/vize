import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  mkdirSync,
  copyFileSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  renameSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { toolingProgram } from "./support/program-path.ts";
import {
  privateGuest,
  runCapture,
  sha256File,
} from "../../.github/actions/test-js-packages-with-history/native-fault-support.ts";
import {
  corePid,
  producer,
} from "../../.github/actions/test-js-packages-with-history/native-fault-core.ts";

const environment = (fixture, id = "123") => ({
  ...process.env,
  GITHUB_ACTIONS: "true",
  GITHUB_RUN_ID: id,
  GITHUB_RUN_ATTEMPT: "1",
  GITHUB_JOB: "law",
  RUNNER_TEMP: fixture,
  VIZE_FAULT_NONCE: "space and 'quote'\nnewline",
});
const receiptAt = (fixture, id = "123") =>
  JSON.parse(
    readFileSync(join(fixture, `js-native-fault-evidence-${id}-1-law/outcome.json`), "utf8"),
  );

void test("only authenticated private Firecracker guests permit kernel changes", () => {
  for (const [vm, container, vendor, allowed] of [
    [[0, "kvm"], [1, "none"], "Firecracker", true],
    [[0, "kvm"], [0, "docker"], "Firecracker", false],
    [[0, "kvm"], [1, "none"], "QEMU", false],
    [[1, "none"], [1, "none"], "Firecracker", false],
    [[null, ""], [1, "none"], "Firecracker", false],
    [[0, "kvm"], [null, ""], "Firecracker", false],
  ]) {
    const capture = (argv) => {
      const [status, stdout] = argv.at(-1) === "--vm" ? vm : container;
      return { status, stdout };
    };
    assert.equal(privateGuest(capture, () => vendor).privateFirecrackerGuest, allowed);
  }
});

for (const outcome of ["0", "7", "SIGTERM", "SIGKILL"]) {
  void test(`observer preserves one original argv/environment/output/cwd and ${outcome}`, () => {
    const fixture = mkdtempSync(join(tmpdir(), "vize-js-native-fault-"));
    try {
      const child = join(fixture, "original.mjs");
      const calls = join(fixture, "calls.jsonl");
      copyFileSync(toolingProgram("process/record-command-outcome.js"), child);
      const env = environment(fixture);
      const result = spawnSync(
        process.execPath,
        [
          toolingProgram("workflow/run-observed-command.js"),
          "normal",
          process.execPath,
          child,
          calls,
          outcome,
          "quoted argument with ' space\nnewline",
        ],
        { cwd: fixture, env, encoding: "utf8" },
      );
      const records = readFileSync(calls, "utf8").trim().split("\n").map(JSON.parse);
      assert.equal(records.length, 1);
      assert.deepEqual(records[0].argv, [calls, outcome, "quoted argument with ' space\nnewline"]);
      assert.equal(records[0].cwd, realpathSync(fixture));
      const baseline = spawnSync(
        process.execPath,
        [toolingProgram("process/print-process-environment.js")],
        {
          env,
          encoding: "utf8",
        },
      );
      assert.equal(baseline.status, 0);
      assert.deepEqual(records[0].env, JSON.parse(baseline.stdout));
      assert.equal(result.stdout, "original stdout\n");
      assert.equal(result.stderr, "original stderr\n");
      if (outcome.startsWith("SIG")) assert.equal(result.signal, outcome);
      else assert.equal(result.status, Number(outcome));
      const receipt = receiptAt(fixture);
      assert.deepEqual(receipt.resourcesBefore, { syntheticResources: true });
      assert.deepEqual(receipt.resourcesAfter, { syntheticResources: true });
      for (const name of ["resources-before.json", "resources-after.json"])
        assert.deepEqual(
          JSON.parse(
            readFileSync(join(fixture, "js-native-fault-evidence-123-1-law", name), "utf8"),
          ),
          { syntheticResources: true },
        );
      assert.equal(receipt.coreCapture, "unavailable");
      assert.deepEqual(receipt.cores, []);
      assert.equal(receipt.command.length, 5);
      assert.equal("coreSetup" in receipt, false);
      assert.equal("coreRestore" in receipt, false);
      assert.equal(receipt.observationGaps["source-before"], "PermissionError");
      assert.equal(receipt.observationGaps["source-after"], "PermissionError");
      assert.equal(receipt.helpers.length, 5);
      assert.deepEqual(
        receipt.helpers.map((row) => row.path.split("/").at(-1)),
        [
          "native-fault-evidence.ts",
          "native-fault-core.ts",
          "native-fault-support.ts",
          "native-fault-retention.ts",
          "native-fault-resources.ts",
        ],
      );
    } finally {
      rmSync(fixture, { recursive: true, force: true });
    }
  });
}

void test("the command writes regular files that the observer relays in full (#12422)", () => {
  // A command holding the step's pipe could see EAGAIN once another process
  // made it nonblocking; a regular file cannot, whatever the reader does.
  const fixture = mkdtempSync(join(tmpdir(), "vize-js-native-fault-relay-"));
  try {
    const result = spawnSync(
      process.execPath,
      [
        toolingProgram("workflow/run-observed-command.js"),
        "normal",
        process.execPath,
        toolingProgram("process/burst-file-streams.js"),
      ],
      {
        env: environment(fixture, "126"),
        encoding: "utf8",
        maxBuffer: 16 * 1024 * 1024,
      },
    );
    assert.equal(result.status, 0, result.stderr);
    const lines = result.stdout.split("\n");
    assert.equal(lines[0], "true true");
    assert.equal(lines.length, 66);
    for (let i = 0; i < 64; i++)
      assert.equal(lines[i + 1], `${String(i).padStart(2, "0")} ${"x".repeat(8190)}`);
    assert.equal(result.stderr, "burst stderr\n");
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});

void test("diagnostic collection failures retain an original command failure", () => {
  const fixture = mkdtempSync(join(tmpdir(), "vize-js-native-fault-gap-"));
  try {
    const result = spawnSync(
      process.execPath,
      [
        toolingProgram("workflow/run-observed-command.js"),
        "descendants",
        process.execPath,
        toolingProgram("process/exit-with-status.js"),
        "9",
        "300",
      ],
      {
        env: environment(fixture, "124"),
        encoding: "utf8",
      },
    );
    assert.equal(result.status, 9, result.stderr);
    const receipt = receiptAt(fixture, "124");
    assert.equal(receipt.observationGaps["process-maps"], "PermissionError");
    assert.equal(receipt.commandReturnCode, 9);
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});

void test(
  "observer forwards termination to the unchanged command and records the signal",
  { skip: process.platform === "win32", timeout: 5000 },
  async () => {
    const fixture = mkdtempSync(join(tmpdir(), "vize-js-native-fault-signal-"));
    try {
      const child = spawn(
        process.execPath,
        [
          toolingProgram("workflow/run-observed-command.js"),
          "normal",
          process.execPath,
          toolingProgram("process/signal-ready.js"),
        ],
        {
          env: environment(fixture),
          stdio: ["ignore", "pipe", "pipe"],
        },
      );
      const result = new Promise((resolve, reject) => {
        child.once("error", reject);
        child.once("exit", (code, signal) => resolve({ code, signal }));
      });
      child.stdout.once("data", () => child.kill("SIGTERM"));
      assert.deepEqual(await result, { code: null, signal: "SIGTERM" });
      const receipt = receiptAt(fixture);
      assert.equal(receipt.forwardedSignal, 15);
      assert.equal(receipt.commandReturnCode, -15);
    } finally {
      rmSync(fixture, { recursive: true, force: true });
    }
  },
);

void test("diagnostic capture enforces its requested timeout and retains partial output authority", () => {
  const result = runCapture(
    [process.execPath, toolingProgram("process/time-out-diagnostic.js")],
    false,
    500,
  );

  assert.equal(result.status, undefined);
  assert.equal(result.unavailable, "Error");
  assert.equal(result.stdout, "started\n");
  assert.equal(result.stdoutAuthority.bytes, Buffer.byteLength(result.stdout));
  assert.equal(
    result.stdoutAuthority.sha256,
    createHash("sha256").update(result.stdout).digest("hex"),
  );
  assert.equal(result.stdoutAuthority.truncated, false);
});

void test("public gdb text excludes startup arguments and retains bounded stream authority", () => {
  const prefix = "Core was generated by argument-secret\nVIZE_NATIVE_FAULT_BEGIN\n";
  const suffix = "\nVIZE_NATIVE_FAULT_END\n";
  const text = `${prefix}${"f".repeat(300000)}${suffix}`;
  const result = runCapture(
    [process.execPath, toolingProgram("workflow/emit-debugger-output.js"), prefix, suffix],
    true,
  );
  assert.doesNotMatch(result.stdout + result.stderr, /argument-secret/);
  assert.equal(result.stdout.length, 262144);
  assert.equal(result.stderr, "");
  assert.equal(result.stdoutAuthority.sha256, createHash("sha256").update(text).digest("hex"));
  assert.equal(result.stdoutAuthority.truncated, true);
});

void test("real core process PID and retained start ticks bind the exact physical ELF; unknowns refuse", () => {
  const fixture = mkdtempSync(join(tmpdir(), "vize-js-native-fault-core-"));
  try {
    const output = join(fixture, "evidence");
    const packets = join(output, "processes");
    mkdirSync(packets, { recursive: true });
    const header = Buffer.alloc(64);
    Buffer.from([0x7f, 0x45, 0x4c, 0x46, 2, 1]).copy(header);
    header.writeUInt16LE(4, 16);
    header.writeUInt16LE(62, 18);
    header.writeBigUInt64LE(64n, 32);
    header.writeUInt16LE(56, 54);
    header.writeUInt16LE(1, 56);
    const status = Buffer.alloc(40);
    status.writeInt32LE(999, 32);
    const info = Buffer.alloc(136);
    info.writeInt32LE(123, 24);
    const note = (kind, body) => {
      const prefix = Buffer.alloc(20);
      prefix.writeUInt32LE(5);
      prefix.writeUInt32LE(body.length, 4);
      prefix.writeUInt32LE(kind, 8);
      Buffer.from("CORE\0").copy(prefix, 12);
      return Buffer.concat([prefix, body]);
    };
    const notes = Buffer.concat([note(1, status), note(3, info)]);
    const program = Buffer.alloc(56);
    program.writeUInt32LE(4);
    program.writeBigUInt64LE(120n, 8);
    program.writeBigUInt64LE(BigInt(notes.length), 32);
    const core = join(fixture, "core.123");
    writeFileSync(core, Buffer.concat([header, program, notes]));
    assert.equal(corePid(core), 123);
    const executable = join(fixture, "actual-node");
    const elf = Buffer.from(header);
    elf.writeUInt16LE(2, 16);
    writeFileSync(executable, elf);
    const identity = {
      path: executable,
      mappedInode: statSync(executable).ino,
      bytes: 64,
      sha256: createHash("sha256").update(elf).digest("hex"),
    };
    const packet = { pid: 123, startTicks: "456", executable, providers: [identity] };
    const first = join(packets, "123-0.json");
    writeFileSync(first, JSON.stringify(packet));
    const result = producer(core, output, sha256File);
    assert.equal(result.path, executable);
    assert.equal(result.startTicks, "456");
    const fixedSlot = join(packets, "tail-001.json");
    renameSync(first, fixedSlot);
    assert.equal(producer(core, output, sha256File).pid, 123);
    writeFileSync(fixedSlot, JSON.stringify({ ...packet, pid: 999 }));
    assert.throws(() => producer(core, output));
    writeFileSync(fixedSlot, JSON.stringify(packet));
    renameSync(fixedSlot, first);
    const second = join(packets, "123-1.json");
    writeFileSync(second, JSON.stringify({ ...packet, startTicks: "789" }));
    assert.throws(() => producer(core, output));
    rmSync(second);
    writeFileSync(executable, Buffer.concat([elf.subarray(0, -1), Buffer.from("changed")]));
    assert.throws(() => producer(core, output));
    writeFileSync(executable, elf);
    rmSync(first);
    assert.throws(() => producer(core, output));
    const renamed = join(fixture, "core.999");
    renameSync(core, renamed);
    assert.throws(() => producer(renamed, output));
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});

for (const gap of ["source", "guest", "gdb"]) {
  void test(`${gap} collection failures execute the original command once and preserve failure`, () => {
    const fixture = mkdtempSync(join(tmpdir(), "vize-js-native-fault-once-"));
    try {
      if (gap === "gdb") {
        const cores = join(fixture, "js-native-fault-cores-125-1-law");
        mkdirSync(cores);
        writeFileSync(join(cores, "core.1"), "typed control");
      }
      const result = spawnSync(
        process.execPath,
        [
          toolingProgram("workflow/run-observed-command.js"),
          gap,
          process.execPath,
          toolingProgram("process/record-once.js"),
          "11",
        ],
        {
          cwd: fixture,
          env: environment(fixture, "125"),
          encoding: "utf8",
        },
      );
      assert.equal(result.status, 11, result.stderr);
      assert.equal(readFileSync(join(fixture, "calls"), "utf8"), "once");
      const receipt = receiptAt(fixture, "125");
      assert.equal(receipt.commandReturnCode, 11);
      assert.equal(receipt.coreCapture, "unavailable");
      assert.equal(
        receipt.observationGaps[
          gap === "source" ? "source-before" : gap === "gdb" ? "core-core.1" : gap
        ],
        "PermissionError",
      );
      if (gap === "source")
        assert.equal(receipt.observationGaps["source-after"], "PermissionError");
      assert.equal(typeof receipt.observerFileHash.bytes, "number");
      assert.equal(typeof receipt.observerFileHash.seconds, "number");
    } finally {
      rmSync(fixture, { recursive: true, force: true });
    }
  });
}

void test("observation setup refusal executes the original command once with its status", () => {
  const fixture = mkdtempSync(join(tmpdir(), "vize-js-native-fault-refusal-"));
  try {
    mkdirSync(join(fixture, "js-native-fault-evidence-123-1-law"));
    const result = spawnSync(
      process.execPath,
      [
        toolingProgram("workflow/run-observed-command.js"),
        "normal",
        process.execPath,
        toolingProgram("process/record-once.js"),
        "17",
      ],
      {
        cwd: fixture,
        env: environment(fixture),
        encoding: "utf8",
      },
    );
    assert.equal(result.status, 17, result.stderr);
    assert.equal(readFileSync(join(fixture, "calls"), "utf8"), "once");
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});
