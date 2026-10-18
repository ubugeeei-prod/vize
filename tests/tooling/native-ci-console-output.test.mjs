import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { ciConsoleOutput } from "../../npm/native/scripts/ci-console-output.mjs";
import {
  bytes,
  nativeHistoryDirectory,
} from "../../npm/native/scripts/formatter-history-build.mts";

void test("CI display retains the complete stream before omitting only Cargo metadata from the console", () => {
  const nativeDir = fs.mkdtempSync(path.join(os.tmpdir(), "native-ci-console-"));
  try {
    const metadata = Buffer.from(
      Array.from({ length: 431 }, (_, index) =>
        JSON.stringify({ reason: "compiler-artifact", index, filenames: ["x".repeat(900)] }),
      ).join("\n") + '\n{"reason":"build-script-executed","linked_paths":["all retained"]}\n',
    );
    const visible = Buffer.concat([
      Buffer.from(
        JSON.stringify({
          reason: "compiler-message",
          message: { rendered: "large diagnostic ".repeat(4_000) },
        }) +
          '\n{"reason":"new-protocol-kind","all":"visible"}\n' +
          '{"reason":"build-finished","success":false}\n',
      ),
      Buffer.from([110, 111, 116, 32, 74, 83, 79, 78, 255, 13, 10]),
      Buffer.from("unterminated output"),
    ]);
    const raw = Buffer.concat([metadata, visible]);
    const stderr = Buffer.concat([Buffer.from("whole Rust diagnostic\n"), Buffer.from([255, 0])]);
    const packet = { stdout: bytes(raw), stderr: bytes(stderr), exitStatus: 7 };
    const directory = nativeHistoryDirectory(nativeDir);
    fs.mkdirSync(directory, { recursive: true });
    fs.writeFileSync(path.join(directory, "cargo.stdout.bin"), raw);
    fs.writeFileSync(path.join(directory, "cargo.stderr.bin"), stderr);
    const output = ciConsoleOutput(nativeDir, "cargo", packet);
    const summaryEnd = output.stdout.indexOf(10) + 1;
    assert(output.stdout.subarray(summaryEnd).equals(visible));
    assert(output.stderr.equals(stderr));
    assert.match(output.stdout.subarray(0, summaryEnd).toString(), /cargo\.stdout\.bin/);
    assert(output.stdout.subarray(0, summaryEnd).includes(Buffer.from(String(raw.length))));
    assert(output.stdout.subarray(0, summaryEnd).includes(Buffer.from(packet.stdout.sha256)));
    assert(fs.readFileSync(path.join(directory, "cargo.stdout.bin")).equals(raw));
    assert(fs.readFileSync(path.join(directory, "cargo.stderr.bin")).equals(stderr));
    assert.equal(packet.exitStatus, 7, "display gives no process or build acceptance");

    // The synchronizer's stdout is not the Cargo metadata protocol.
    fs.writeFileSync(path.join(directory, "sync.stdout.bin"), raw);
    fs.writeFileSync(path.join(directory, "sync.stderr.bin"), stderr);
    const sync = ciConsoleOutput(nativeDir, "sync", packet);
    assert(sync.stdout.subarray(sync.stdout.indexOf(10) + 1).equals(raw));

    assert.throws(() => ciConsoleOutput(nativeDir, "missing", packet), /ENOENT/);
    fs.writeFileSync(path.join(directory, "cargo.stdout.bin"), raw.subarray(1));
    assert.throws(() => ciConsoleOutput(nativeDir, "cargo", packet));
    fs.writeFileSync(path.join(directory, "cargo.stdout.bin"), raw);
    assert.throws(() =>
      ciConsoleOutput(nativeDir, "cargo", {
        ...packet,
        stderr: { ...packet.stderr, sha256: "0".repeat(64) },
      }),
    );
  } finally {
    fs.rmSync(nativeDir, { recursive: true, force: true });
  }
});
