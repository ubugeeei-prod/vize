import assert from "node:assert/strict";
import { ChildProcess, spawn } from "node:child_process";
import { errorMonitor } from "node:events";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { PassThrough } from "node:stream";
import { test } from "node:test";
import {
  metadataCaptureDirectory,
  observeMetadataSpawn,
  validateMetadataCapture,
} from "../../tools/support/ci/cargo-metadata-observer.ts";

test("scoped preload URL retains its directory in a clean task environment", () => {
  const directory = "/tmp/metadata 雪 capture";
  const url = new URL("file:///repo/tools/support/ci/cargo-metadata-observer.ts");
  url.searchParams.set("capture", directory);
  assert.equal(metadataCaptureDirectory(url.href, {}), directory);
  assert.equal(metadataCaptureDirectory(import.meta.url, {}), undefined);
  assert.equal(
    metadataCaptureDirectory(import.meta.url, { VIZE_CARGO_METADATA_CAPTURE: directory }),
    directory,
  );
});

function fixture() {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-metadata-observer-"));
  const child = new ChildProcess();
  child.stdout = new PassThrough();
  child.stderr = new PassThrough();
  const calls: unknown[][] = [];
  const fake = new Proxy(spawn, {
    apply(_target, _receiver, args: unknown[]) {
      calls.push(args);
      return child;
    },
  });
  return {
    directory,
    child,
    calls,
    observed: observeMetadataSpawn(fake, directory),
    cleanup: () => fs.rmSync(directory, { recursive: true, force: true }),
    packet: (phase = "close") => {
      const invocation = path.join(directory, fs.readdirSync(directory)[0]);
      return path.join(
        invocation,
        fs.readdirSync(invocation).find((value) => value.endsWith(`-${phase}`))!,
      );
    },
  };
}

test("actual argv/options/child identity and complete split raw buffers remain unchanged", () => {
  const f = fixture();
  try {
    const argv = ["metadata", "--manifest-path", "crate/Cargo.toml", "--format-version", "1"];
    const options = { stdio: "pipe" as const, env: { PRIVATE_TOKEN: "synthetic-value" } };
    assert.equal(f.observed("cargo", argv, options), f.child);
    assert.equal(f.calls.length, 1);
    assert.equal(f.calls[0][1], argv);
    assert.equal(f.calls[0][2], options);
    argv[1] = "changed-after-spawn";
    const bytes = Buffer.from([0x7b, 0, 0xff, 0x7d]);
    f.child.stdout!.emit("data", bytes.subarray(0, 2));
    f.child.stdout!.emit("data", bytes.subarray(2));
    f.child.stderr!.emit("data", Buffer.from("whole stderr\r\n"));
    f.child.emit("close", 0, null);
    assert.deepEqual(fs.readFileSync(path.join(f.packet(), "stdout.bin")), bytes);
    assert.deepEqual(
      fs.readFileSync(path.join(f.packet(), "stderr.bin")),
      Buffer.from("whole stderr\r\n"),
    );
    const result = JSON.parse(fs.readFileSync(path.join(f.packet(), "process.json"), "utf8"));
    assert.equal(result.exitStatus, 0);
    assert.equal(result.signal, null);
    assert.equal(result.exactBufferEvents, true);
    assert.deepEqual(result.parentArgv, process.argv);
    assert.equal(result.parentExecPath, process.execPath);
    assert.deepEqual(result.argv, [
      "metadata",
      "--manifest-path",
      "crate/Cargo.toml",
      "--format-version",
      "1",
    ]);
    assert.equal(result.callerOptions.environmentOverridden, true);
    assert.equal(JSON.stringify(result).includes("synthetic-value"), false);
  } finally {
    f.cleanup();
  }
});

test("caller error handling is preserved; null exit and SIGKILL stay distinct from code0", () => {
  const f = fixture();
  try {
    f.observed("cargo", ["metadata"], { stdio: "pipe" });
    const received: Error[] = [];
    f.child.on("error", (value) => received.push(value));
    f.child.emit("error", Object.assign(new Error("original error"), { code: "EIO" }));
    f.child.emit("close", null, "SIGKILL");
    const result = JSON.parse(fs.readFileSync(path.join(f.packet(), "process.json"), "utf8"));
    assert.equal(result.exitStatus, null);
    assert.equal(result.signal, "SIGKILL");
    assert.equal(received.length, 1);
    assert.deepEqual(result.errors, [{ name: "Error", message: "original error", code: "EIO" }]);
    assert.deepEqual(fs.readFileSync(path.join(f.packet(), "stdout.bin")), Buffer.alloc(0));
  } finally {
    f.cleanup();
  }
});

test("without a caller listener, the original error still throws and partial bytes are retained first", () => {
  const f = fixture();
  try {
    f.observed("cargo", ["metadata"], { stdio: "pipe" });
    assert.equal(f.child.listenerCount("error"), 0);
    assert.equal(f.child.listenerCount(errorMonitor), 1);
    f.child.stdout!.emit("data", Buffer.from("partial\0"));
    const original = Object.assign(new Error("unhandled original"), { code: "ENOENT" });
    assert.throws(
      () => f.child.emit("error", original),
      (failure) => failure === original,
    );
    const result = JSON.parse(
      fs.readFileSync(path.join(f.packet("error"), "process.json"), "utf8"),
    );
    assert.equal(result.streamsClosed, false);
    assert.equal(result.exitStatus, null);
    assert.equal(result.errors[0].message, "unhandled original");
    assert.deepEqual(
      fs.readFileSync(path.join(f.packet("error"), "stdout.bin")),
      Buffer.from("partial\0"),
    );
  } finally {
    f.cleanup();
  }
});

test("non-metadata commands pass through once and produce no packet", () => {
  const f = fixture();
  try {
    assert.equal(f.observed("cargo", ["build"], { stdio: "pipe" }), f.child);
    f.child.emit("close", 0, null);
    assert.equal(f.calls.length, 1);
    assert.deepEqual(fs.readdirSync(f.directory), []);
  } finally {
    f.cleanup();
  }
});

test("capture validation refuses empty and partial packets without deleting available bytes", () => {
  const f = fixture();
  try {
    assert.throws(() => validateMetadataCapture(f.directory), /No Cargo metadata/);
    f.observed("cargo", ["metadata"], { stdio: "pipe" });
    f.child.stdout!.emit("data", Buffer.from("whole stdout\0"));
    f.child.stderr!.emit("data", Buffer.from("whole stderr\r\n"));
    f.child.emit("close", 0, null);
    validateMetadataCapture(f.directory);
    const packet = f.packet();
    const names = ["stdout.bin", "stderr.bin", "process.json"];
    const original = names.map((name) => fs.readFileSync(path.join(packet, name)));
    for (const [index, name] of names.entries()) {
      fs.unlinkSync(path.join(packet, name));
      assert.throws(() => validateMetadataCapture(f.directory), /ENOENT/);
      for (const [other, retained] of names.entries())
        if (other !== index)
          assert.deepEqual(fs.readFileSync(path.join(packet, retained)), original[other]);
      fs.writeFileSync(path.join(packet, name), original[index]);
    }
    fs.writeFileSync(path.join(packet, "process.json"), "{");
    assert.throws(() => validateMetadataCapture(f.directory), SyntaxError);
    assert.deepEqual(fs.readFileSync(path.join(packet, "stdout.bin")), original[0]);
  } finally {
    f.cleanup();
  }
});
