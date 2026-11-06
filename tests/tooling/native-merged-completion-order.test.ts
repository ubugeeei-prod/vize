import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import {
  observeMergedCompletionOrder,
  requireMergedCompletionBefore,
} from "./support/native-beta-dto/merged-completion-order.ts";
import { requireCompletedBefore } from "./support/native-beta-dto/native-io.ts";

const eof = '10 1.000001 read(0, "", 4096) = 0';
const write = '11 1.000002 write(2, "\\x0a", 1) = 1';
const digest = (lines: readonly string[]) =>
  createHash("sha256")
    .update(lines.join("\n") + "\n")
    .digest("hex");
const observe = (lines: readonly string[]) => observeMergedCompletionOrder(lines, digest(lines));

test("merged completed result precedes a later observed entry without invented return times", () => {
  const [earlier, later] = observe([eof, write]);
  requireMergedCompletionBefore(earlier!, later!);
  // The same old per-thread observations must keep refusing the missing time bound.
  assert.throws(
    () =>
      requireCompletedBefore(
        {
          tid: 10,
          line: 1,
          at: "1.000001",
          completion: {
            returnedAt: null,
            returnedBy: { at: "1.000009", line: 2, basis: "next-same-thread-event" },
          },
        },
        { tid: 11, line: 1, at: "1.000002" },
      ),
    /completion must precede/u,
  );
});

test("an explicit resumed completion must itself precede the other thread entry", () => {
  const [earlier, later] = observe([
    "10 1.000001 read(0,  <unfinished ...>",
    '10 1.000002 <... read resumed>"", 4096) = 0',
    '11 1.000003 write(2, "\\x0a", 1) = 1',
  ]);
  requireMergedCompletionBefore(earlier!, later!);
});

test("an entry followed by another write before its resumed completion stays refused", () => {
  const [later, earlier] = observe([
    "10 1.000001 read(0,  <unfinished ...>",
    write,
    '10 1.000003 <... read resumed>"", 4096) = 0',
  ]);
  assert.throws(() => requireMergedCompletionBefore(earlier!, later!), /completion must precede/u);
});

test("unknown, foreign, copied and reordered witnesses cannot establish completion", () => {
  const [unknown, later] = observe(['10 1.000001 write(2, "\\x0a", 1) = ? <unavailable>', write]);
  assert.throws(() => requireMergedCompletionBefore(unknown!, later!), /known completed/u);
  const [earlier, ownLater] = observe([eof, write]);
  const [foreign] = observe([write]);
  assert.throws(() => requireMergedCompletionBefore(earlier!, foreign!), /one original/u);
  assert.throws(() => requireMergedCompletionBefore({ ...earlier! }, ownLater!), /raw-derived/u);
  assert.throws(
    () => requireMergedCompletionBefore(ownLater!, earlier!),
    /completion must precede/u,
  );
});

test("missing, mismatched and duplicate unfinished entries refuse fabricated joins", () => {
  assert.throws(() => observe(['10 1.000001 <... read resumed>"", 4096) = 0']));
  assert.throws(() =>
    observe([
      "10 1.000001 read(0,  <unfinished ...>",
      '10 1.000002 <... write resumed>"", 4096) = 0',
    ]),
  );
  assert.throws(() =>
    observe(["10 1.000001 read(0,  <unfinished ...>", "10 1.000002 read(0,  <unfinished ...>"]),
  );
});

test("sorting, dropping or changing raw rows cannot reuse the original collector digest", () => {
  const original = [eof, write];
  for (const changed of [[write, eof], [eof], [eof, write.replace("1.000002", "1.000009")]])
    assert.throws(
      () => observeMergedCompletionOrder(changed, digest(original)),
      /original collector stream identity/u,
    );
});
