import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { publishWorkerRecord } from "./n8n-worker-record-publication.mjs";

const fixture = () => {
  const output = fs.mkdtempSync(path.join(os.tmpdir(), "vize-n8n-record-publication-"));
  const target = path.join(output, "worker-process.json");
  const record = {
    index: 0,
    command: process.execPath,
    args: ["-e", "日本語\r\n"],
    cwd: output,
    output,
    pid: process.pid,
    status: null,
    signal: null,
    error: null,
    closed: false,
    stdoutBytes: 0,
    stderrBytes: 0,
  };
  const bytes = JSON.stringify(record, null, 2) + "\n";
  fs.writeFileSync(target, bytes);
  return { output, target, record, bytes };
};

void test("a real partial replacement write exposes only the previous complete worker packet", () => {
  const { output, target, record, bytes } = fixture();
  const write = fs.writeFileSync;
  let observed;
  try {
    fs.writeFileSync = (file, data, options) => {
      assert.equal(path.basename(file), "record.json");
      assert.equal(options.flag, "wx");
      const fd = fs.openSync(file, options.flag);
      try {
        fs.writeSync(fd, data.slice(0, 1));
        observed = fs.readFileSync(target, "utf8");
        assert.deepEqual(JSON.parse(observed), record);
        fs.writeSync(fd, data.slice(1));
      } finally {
        fs.closeSync(fd);
      }
    };
    const replacement = {
      ...record,
      status: 0,
      closed: true,
      stdoutBytes: 17,
      stderrBytes: 9,
      walltimeMs: 26,
      terminationReason: "closed worker residual group cleanup",
    };
    publishWorkerRecord(replacement);
    assert.equal(observed, bytes);
    assert.equal(fs.readFileSync(target, "utf8"), JSON.stringify(replacement, null, 2) + "\n");
    assert.deepEqual(fs.readdirSync(output), ["worker-process.json"]);
  } finally {
    fs.writeFileSync = write;
    fs.rmSync(output, { recursive: true });
  }
});

void test("failed actual partial publication preserves the complete old packet and refuses completion", () => {
  const { output, target, record, bytes } = fixture();
  const write = fs.writeFileSync;
  const failure = Object.assign(new Error("actual staged write failure"), { code: "EIO" });
  try {
    fs.writeFileSync = (file, data, options) => {
      write(file, data.slice(0, 1), options);
      assert.equal(fs.readFileSync(target, "utf8"), bytes);
      throw failure;
    };
    assert.throws(
      () => publishWorkerRecord({ ...record, status: 0, closed: true }),
      (error) => error === failure,
    );
    assert.equal(fs.readFileSync(target, "utf8"), bytes);
    assert.deepEqual(JSON.parse(fs.readFileSync(target, "utf8")), record);
    assert.deepEqual(fs.readdirSync(output), ["worker-process.json"]);
  } finally {
    fs.writeFileSync = write;
    fs.rmSync(output, { recursive: true });
  }
});
