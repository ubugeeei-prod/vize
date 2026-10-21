import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { gunzipSync } from "node:zlib";

type Packet = Record<string, any> & { file: string; messages: Record<string, any>[] };
type Changes = Record<string, number>;

export interface LintSnapshotAlignment {
  path: string;
  previousSha256: string;
  currentSha256: string;
  packetCount: number;
  beforeMessages: number;
  afterMessages: number;
  changedPackets: number;
  aggregateChanges: Changes;
  packetReviews: {
    file: string;
    beforeSha256: string;
    afterSha256: string;
    changes: Changes;
  }[];
}

const hash = (value: string) => createHash("sha256").update(value).digest("hex");

function canonical(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.entries(value)
      .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
      .map(([key, entry]) => `${JSON.stringify(key)}:${canonical(entry)}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

function classify(before: Packet, after: Packet): Changes {
  const changes = new Map<string, number>();
  const count = (name: string, amount = 1) => changes.set(name, (changes.get(name) ?? 0) + amount);
  const beforeMessages = [...before.messages];
  const afterMessages = [...after.messages];
  const beforeExact = new Map<string, number>();
  const afterExact = new Map<string, number>();
  for (const message of beforeMessages)
    beforeExact.set(canonical(message), (beforeExact.get(canonical(message)) ?? 0) + 1);
  for (const message of afterMessages)
    afterExact.set(canonical(message), (afterExact.get(canonical(message)) ?? 0) + 1);
  const common = new Map<string, number>();
  for (const [key, amount] of beforeExact)
    common.set(key, Math.min(amount, afterExact.get(key) ?? 0));
  const unmatched = (messages: Record<string, any>[]) => {
    const used = new Map<string, number>();
    return messages.filter((message) => {
      const key = canonical(message);
      const seen = used.get(key) ?? 0;
      if (seen < (common.get(key) ?? 0)) {
        used.set(key, seen + 1);
        return false;
      }
      return true;
    });
  };
  let remainingBefore = unmatched(beforeMessages);
  let remainingAfter = unmatched(afterMessages);
  const pairs: [Record<string, any>, Record<string, any>][] = [];
  const pairBy = (keyOf: (message: Record<string, any>) => string) => {
    const left = new Map<string, [number, Record<string, any>][]>() as Map<
      string,
      [number, Record<string, any>][]
    >;
    const right = new Map<string, [number, Record<string, any>][]>() as Map<
      string,
      [number, Record<string, any>][]
    >;
    remainingBefore.forEach((message, index) => {
      const key = keyOf(message);
      left.set(key, [...(left.get(key) ?? []), [index, message]]);
    });
    remainingAfter.forEach((message, index) => {
      const key = keyOf(message);
      right.set(key, [...(right.get(key) ?? []), [index, message]]);
    });
    const pairedLeft = new Set<number>();
    const pairedRight = new Set<number>();
    for (const key of [...left.keys()].filter((key) => right.has(key)).sort()) {
      const beforeGroup = left.get(key)!;
      const afterGroup = right.get(key)!;
      for (let index = 0; index < Math.min(beforeGroup.length, afterGroup.length); index++) {
        pairedLeft.add(beforeGroup[index][0]);
        pairedRight.add(afterGroup[index][0]);
        pairs.push([beforeGroup[index][1], afterGroup[index][1]]);
      }
    }
    remainingBefore = remainingBefore.filter((_, index) => !pairedLeft.has(index));
    remainingAfter = remainingAfter.filter((_, index) => !pairedRight.has(index));
  };
  const ranges = ["line", "column", "endLine", "endColumn"];
  pairBy((message) => canonical([message.ruleId, message.severity, message.message]));
  pairBy((message) =>
    canonical([message.ruleId, message.severity, ...ranges.map((key) => message[key])]),
  );
  for (const [beforeMessage, afterMessage] of pairs) {
    const rule = beforeMessage.ruleId ?? afterMessage.ruleId ?? "<none>";
    if (ranges.some((key) => beforeMessage[key] !== afterMessage[key])) count(`range:${rule}`);
    if (beforeMessage.help !== afterMessage.help) count(`help:${rule}`);
    const withoutPresentation = (message: Record<string, any>) =>
      Object.fromEntries(
        Object.entries(message).filter(([key]) => key !== "help" && !ranges.includes(key)),
      );
    if (
      canonical(withoutPresentation(beforeMessage)) !== canonical(withoutPresentation(afterMessage))
    )
      count(`payload:${rule}`);
  }
  for (const message of remainingBefore) count(`removed:${message.ruleId ?? "<none>"}`);
  for (const message of remainingAfter) count(`added:${message.ruleId ?? "<none>"}`);
  if (
    pairs.length === 0 &&
    remainingBefore.length === 0 &&
    remainingAfter.length === 0 &&
    canonical(beforeMessages) !== canonical(afterMessages)
  )
    count("message-order");
  for (const key of ["errorCount", "warningCount", "fatalErrorCount"])
    if (before[key] !== after[key]) count(`count:${key}`);
  const withoutMessages = (packet: Packet) =>
    Object.fromEntries(
      Object.entries(packet).filter(
        ([key]) => !["messages", "errorCount", "warningCount", "fatalErrorCount"].includes(key),
      ),
    );
  if (canonical(withoutMessages(before)) !== canonical(withoutMessages(after)))
    count("packet-fields");
  return Object.fromEntries([...changes].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)));
}

export function loadLintSnapshotAlignments(root: string): Map<string, LintSnapshotAlignment> {
  const payload = JSON.parse(
    gunzipSync(
      fs.readFileSync(
        path.join(
          root,
          "tests/_fixtures/_projects/lint-review/2026-10-21-snapshot-alignment/review.json.gz",
        ),
      ),
    ).toString(),
  ) as { entries: LintSnapshotAlignment[] };
  assert.equal(payload.entries.length, 7);
  return new Map(payload.entries.map((entry) => [entry.path, entry]));
}

export function verifyLintSnapshotAlignment(
  previousRaw: string,
  currentRaw: string,
  review: LintSnapshotAlignment,
): void {
  assert.equal(hash(previousRaw), review.previousSha256);
  assert.equal(hash(currentRaw), review.currentSha256);
  const before = JSON.parse(previousRaw) as Packet[];
  const after = JSON.parse(currentRaw) as Packet[];
  assert.equal(after.length, review.packetCount);
  assert.deepEqual(
    before.map((packet) => packet.file),
    after.map((packet) => packet.file),
  );
  assert.equal(
    before.reduce((total, packet) => total + packet.messages.length, 0),
    review.beforeMessages,
  );
  assert.equal(
    after.reduce((total, packet) => total + packet.messages.length, 0),
    review.afterMessages,
  );
  const aggregate = new Map<string, number>();
  const packetReviews = before.flatMap((beforePacket, index) => {
    const afterPacket = after[index];
    if (canonical(beforePacket) === canonical(afterPacket)) return [];
    const changes = classify(beforePacket, afterPacket);
    for (const [name, amount] of Object.entries(changes))
      aggregate.set(name, (aggregate.get(name) ?? 0) + amount);
    return [
      {
        file: beforePacket.file,
        beforeSha256: hash(canonical(beforePacket)),
        afterSha256: hash(canonical(afterPacket)),
        changes,
      },
    ];
  });
  assert.equal(packetReviews.length, review.changedPackets);
  assert.deepEqual(packetReviews, review.packetReviews);
  assert.deepEqual(
    Object.fromEntries([...aggregate].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))),
    review.aggregateChanges,
  );
}
