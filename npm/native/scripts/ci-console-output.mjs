import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { hash, nativeHistoryDirectory } from "./formatter-history-build.mts";

export function ciConsoleOutput(nativeDir, label, processPacket) {
  const directory = nativeHistoryDirectory(nativeDir);
  const streams = Object.fromEntries(
    ["stdout", "stderr"].map((stream) => {
      const file = path.join(directory, `${label}.${stream}.bin`);
      const raw = fs.readFileSync(file);
      assert(raw.equals(Buffer.from(processPacket[stream].base64, "base64")));
      assert.equal(hash(raw), processPacket[stream].sha256);
      return [stream, raw];
    }),
  );
  const visible = [];
  for (let start = 0; start < streams.stdout.length;) {
    const newline = streams.stdout.indexOf(10, start);
    const end = newline < 0 ? streams.stdout.length : newline + 1;
    const line = streams.stdout.subarray(start, end);
    let reason;
    try {
      reason = JSON.parse(line.toString()).reason;
    } catch {
      // Non-JSON output, unknown records and diagnostics remain visible in full.
    }
    if (label !== "cargo" || !["compiler-artifact", "build-script-executed"].includes(reason))
      visible.push(line);
    start = end;
  }
  const retained = path.relative(nativeDir, path.join(directory, `${label}.stdout.bin`));
  const summary = `${label} stdout retained: ${retained} (${streams.stdout.length} bytes; sha256 ${processPacket.stdout.sha256})\n`;
  return { stdout: Buffer.concat([Buffer.from(summary), ...visible]), stderr: streams.stderr };
}
