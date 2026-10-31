import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import type { Packet } from "./support/content-mapper-instance-globals-10617.ts";
import { createInstanceGlobalsProcessLog } from "./support/content-mapper-instance-globals-process-log-10617.ts";

const packet = (index: number): Packet => ({
  label: `capture/${index}/日本語🍣`,
  command: 'C:\\tools\\"native"',
  args: ["--project", `path/${index}/雪🌸.json`, '\r\nquoted"\\value'],
  cwd: "/owned/プロジェクト",
  status: index % 3,
  signal: null,
  error: null,
  stdout: `line ${index}\r\n"日本語🍣"\n${JSON.stringify({ id: index, result: { text: "a\r\nb" } })}\n`,
  stderr: index % 2 ? "full\r\nerror\n" : "",
  elapsedMs: index + 0.125,
});
const legacy = (records: Packet[]) => Buffer.from(`${JSON.stringify(records, null, 2)}\n`);
function temporary(run: (directory: string) => void) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-process-log-10617-"));
  try {
    run(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

test("every completed prefix preserves complete legacy bytes and leaves caller references alone", () => {
  temporary((directory) => {
    const file = path.join(directory, "processes.json");
    const append = createInstanceGlobalsProcessLog(file);
    assert.equal(fs.existsSync(file), false);
    const records: Packet[] = [];
    for (let index = 0; index < 64; index++) {
      const value = packet(index),
        args = value.args;
      const original = structuredClone(value);
      records.push(value);
      append(value);
      assert(fs.readFileSync(file).equals(legacy(records)), `complete prefix ${index + 1}`);
      assert.deepEqual(value, original);
      assert.equal(value.args, args);
      assert.equal(Object.isFrozen(value), false);
      assert.equal(Object.isFrozen(args), false);
    }
  });
});

test("failed process packets are complete evidence before the existing producer guards throw", () => {
  temporary((directory) => {
    const file = path.join(directory, "processes.json"),
      append = createInstanceGlobalsProcessLog(file);
    const records = [
      {
        ...packet(0),
        status: null,
        error: "spawn ENOENT",
        stdout: "partial stdout",
        stderr: "partial stderr",
      },
      { ...packet(1), status: null, signal: "SIGTERM" },
    ];
    for (const [index, value] of records.entries()) {
      append(value);
      assert(fs.readFileSync(file).equals(legacy(records.slice(0, index + 1))));
      assert.throws(() => {
        assert.equal(value.error, null);
        assert.equal(value.signal, null);
      });
    }
  });
});

test("interleaved owned outputs have independent cursors and preserve the old first-write reset", () => {
  temporary((directory) => {
    const first = path.join(directory, "first.json"),
      second = path.join(directory, "second.json");
    fs.writeFileSync(first, "previous invocation\n");
    const a = createInstanceGlobalsProcessLog(first),
      b = createInstanceGlobalsProcessLog(second);
    const left: Packet[] = [],
      right: Packet[] = [];
    for (let index = 0; index < 6; index++) {
      left.push(packet(index));
      a(left.at(-1)!);
      right.push(packet(index + 20));
      b(right.at(-1)!);
      assert(fs.readFileSync(first).equals(legacy(left)));
      assert(fs.readFileSync(second).equals(legacy(right)));
    }
  });
});

test("short writes complete every UTF8 byte without revisiting old records", (context) => {
  temporary((directory) => {
    const original = fs.writeSync;
    let calls = 0;
    context.mock.method(
      fs,
      "writeSync",
      (fd: number, body: Buffer, offset: number, length: number, position: number) => {
        calls++;
        return original(fd, body, offset, Math.min(length, 7), position);
      },
    );
    const file = path.join(directory, "processes.json"),
      append = createInstanceGlobalsProcessLog(file);
    const values = [packet(1), packet(2)];
    append(values[0]);
    assert(fs.readFileSync(file).equals(legacy(values.slice(0, 1))));
    append(values[1]);
    assert(fs.readFileSync(file).equals(legacy(values)));
    assert(calls > 10);
  });
});

test("a partial-write failure keeps partial bytes visible and cannot be resumed", (context) => {
  temporary((directory) => {
    const file = path.join(directory, "processes.json"),
      append = createInstanceGlobalsProcessLog(file);
    const first = packet(0),
      second = packet(1);
    append(first);
    const before = fs.readFileSync(file),
      original = fs.writeSync;
    let calls = 0;
    context.mock.method(
      fs,
      "writeSync",
      (fd: number, body: Buffer, offset: number, length: number, position: number) => {
        if (calls++ > 0) throw new Error("owned partial write failure");
        return original(fd, body, offset, Math.min(length, 11), position);
      },
    );
    assert.throws(() => append(second), /owned partial write failure/u);
    const partial = fs.readFileSync(file);
    const complete = legacy([first, second]);
    assert(partial.equals(complete.subarray(0, before.length - 3 + 11)));
    assert(partial.subarray(0, before.length - 3).equals(before.subarray(0, -3)));
    assert.throws(() => JSON.parse(partial.toString()));
    assert.throws(() => append(packet(2)), /failed process log/u);
    assert(fs.readFileSync(file).equals(partial));
  });
});

test("a zero-length write fails without a retry loop or concealed evidence", (context) => {
  temporary((directory) => {
    context.mock.method(fs, "writeSync", () => 0);
    const file = path.join(directory, "processes.json"),
      append = createInstanceGlobalsProcessLog(file);
    assert.throws(() => append(packet(0)));
    assert.equal(fs.readFileSync(file).length, 0);
    assert.throws(() => append(packet(1)), /failed process log/u);
  });
});

test("tampered, truncated or extended closing tails refuse before writing and remain visible", () => {
  for (const tamper of [
    (file: string) => fs.appendFileSync(file, "\n"),
    (file: string) => fs.truncateSync(file, fs.statSync(file).size - 1),
    (file: string) => {
      const contents = fs.readFileSync(file);
      contents[contents.length - 2] = "}".charCodeAt(0);
      fs.writeFileSync(file, contents);
    },
  ])
    temporary((directory) => {
      const file = path.join(directory, "processes.json"),
        append = createInstanceGlobalsProcessLog(file);
      append(packet(0));
      tamper(file);
      const broken = fs.readFileSync(file);
      assert.throws(() => append(packet(1)));
      assert(fs.readFileSync(file).equals(broken));
      assert.throws(() => append(packet(2)), /failed process log/u);
    });
});

test("a replaced file cannot inherit an earlier invocation's append cursor", () => {
  temporary((directory) => {
    const file = path.join(directory, "processes.json"),
      append = createInstanceGlobalsProcessLog(file);
    append(packet(0));
    fs.renameSync(file, path.join(directory, "previous.json"));
    fs.copyFileSync(path.join(directory, "previous.json"), file);
    const replaced = fs.readFileSync(file);
    assert.throws(() => append(packet(1)), /process log replaced/u);
    assert(fs.readFileSync(file).equals(replaced));
  });
});
