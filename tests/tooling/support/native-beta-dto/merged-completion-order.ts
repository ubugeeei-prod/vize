import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { parseNativeIo } from "./native-io.ts";

type Witness = Readonly<{
  captureSha256: string;
  tid: number;
  entryLine: number;
  completionLine: number | null;
  op: "read" | "write";
  fd: number;
  returnedBytes: number | null;
}>;
const verified = new WeakSet<Witness>();

/** Prototype evidence reader; it does not replace any historical audit law. */
export function observeMergedCompletionOrder(lines: readonly string[], expectedSha256: string) {
  const captureSha256 = createHash("sha256")
    .update(lines.join("\n") + "\n")
    .digest("hex");
  assert.match(expectedSha256, /^[a-f0-9]{64}$/u);
  assert.equal(captureSha256, expectedSha256, "original collector stream identity required");
  const pending = new Map<number, { start: number; text: string; syscall: string }>();
  const observations: Witness[] = [];
  for (const [index, raw] of lines.entries()) {
    assert.ok(!/[\r\n]/u.test(raw), "one original trace record per line required");
    const match = /^([1-9][0-9]*) +([0-9]+\.[0-9]{6} .+)$/u.exec(raw);
    assert.ok(match, `unclosed merged event ${index + 1}`);
    const tid = Number(match[1]);
    assert.ok(Number.isSafeInteger(tid));
    let text = match[2]!;
    let entryLine = index + 1;
    if (text.endsWith("<unfinished ...>")) {
      assert.ok(!pending.has(tid), "one in-flight syscall per traced thread");
      const syscall = /^[0-9]+\.[0-9]{6} ([a-z0-9_]+)\(/u.exec(text)?.[1];
      assert.ok(syscall);
      pending.set(tid, { start: entryLine, text: text.slice(0, -16), syscall });
      continue;
    }
    const resumed = /^[0-9]+\.[0-9]{6} <\.\.\. ([a-z0-9_]+) resumed>/u.exec(text);
    if (resumed) {
      const start = pending.get(tid);
      assert.ok(start && start.syscall === resumed[1], "matching thread entry required");
      entryLine = start.start;
      text = start.text + text.slice(resumed[0].length);
      pending.delete(tid);
    }
    const call = parseNativeIo(text);
    if (!call) continue;
    const witness = Object.freeze({
      captureSha256,
      tid,
      entryLine,
      completionLine: call.outcome === "completed" ? index + 1 : null,
      op: call.op,
      fd: call.fd,
      returnedBytes: call.outcome === "completed" ? call.returnedBytes : null,
    });
    verified.add(witness);
    observations.push(witness);
  }
  return observations;
}

/** Compare observed exit/entry stops in one serialized tracer stream. */
export function requireMergedCompletionBefore(earlier: Witness, later: Witness) {
  assert.ok(verified.has(earlier) && verified.has(later), "raw-derived witnesses required");
  assert.equal(earlier.captureSha256, later.captureSha256, "one original merged stream required");
  assert.ok(earlier.completionLine !== null, "known completed syscall required");
  assert.ok(earlier.completionLine < later.entryLine, "completion must precede observed entry");
}
