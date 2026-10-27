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
  successor?: {
    previousRaw: string;
    fixturePin: string;
    review: LintSnapshotAlignment;
  };
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
  const alignments = new Map(payload.entries.map((entry) => [entry.path, entry]));
  admitSnapshotSuccessors(root, alignments);
  return alignments;
}

interface SourceWitness {
  project: string;
  file: string;
  fixturePin: string;
  path: string;
  blob: string;
  bytes: number;
  sourceSha256: string;
  source: string;
}

function admitSnapshotSuccessors(root: string, alignments: Map<string, LintSnapshotAlignment>) {
  const directory = path.join(
    root,
    "tests/_fixtures/_projects/lint-review/2026-10-27-snapshot-successor",
  );
  const inventory = JSON.parse(fs.readFileSync(path.join(directory, "inventory.json"), "utf8"));
  assert.deepEqual(
    inventory.map((entry: { path: string }) => entry.path),
    ["snapshots.json.gz", "source-witnesses.json.gz"],
  );
  const archived = new Map<string, any>();
  for (const entry of inventory) {
    const bytes = fs.readFileSync(path.join(directory, entry.path));
    assert.equal(bytes.length, entry.bytes);
    assert.equal(createHash("sha256").update(bytes).digest("hex"), entry.sha256);
    archived.set(entry.path, JSON.parse(gunzipSync(bytes).toString()));
  }
  const successors = archived.get("snapshots.json.gz");
  assert.equal(successors.schema, "vize-lint-snapshot-successor-v1");
  assert.equal(successors.snapshotCommit, "9228b7e0a75490ea7c9d530c92b274537d0332df");
  assert.equal(
    createHash("sha256")
      .update(fs.readFileSync(path.join(root, successors.predecessorReviewPath)))
      .digest("hex"),
    successors.predecessorReviewSha256,
    "the original alignment archive remains intact",
  );
  const read = (name: string) =>
    JSON.parse(
      gunzipSync(
        fs.readFileSync(path.join(root, "tests/_fixtures/_projects/lint-review", name)),
      ).toString(),
    );
  const identities = new Map<string, { pin: string; blob: string; bytes: number; path: string }>();
  for (const project of read("2026-10-11/source-provenance.json.gz")) {
    const name = path.basename(project.snapshot, "-lint.snap");
    assert.equal(project.original_fixture_pin, project.actual_fixture_pin);
    for (const file of project.files)
      identities.set(`${name}/${file.file}`, {
        pin: project.actual_fixture_pin,
        blob: file.source_blob_oid,
        bytes: file.source_bytes,
        path: file.file,
      });
  }
  for (const [name, archive] of [
    ["misskey", "misskey-all-583-file-packet-source-provenance.json.gz"],
    ["npmx.dev", "npmx-all-220-file-packet-source-provenance.json.gz"],
  ]) {
    const project = read(`2026-10-11-executable-configs/${archive}`);
    for (const file of project.files)
      identities.set(`${name}/${file.file}`, {
        pin: project.originalRevision,
        blob: file.gitBlob,
        bytes: file.sourceBytes,
        path: file.originalGitPath,
      });
  }
  const witnesses = new Map<string, SourceWitness>();
  for (const witness of archived.get("source-witnesses.json.gz") as SourceWitness[]) {
    const key = `${witness.project}/${witness.file}`;
    assert.ok(!witnesses.has(key));
    const source = Buffer.from(witness.source);
    assert.deepEqual(identities.get(key), {
      pin: witness.fixturePin,
      blob: witness.blob,
      bytes: source.length,
      path: witness.path,
    });
    assert.equal(source.length, witness.bytes);
    assert.equal(hash(witness.source), witness.sourceSha256);
    assert.equal(
      createHash("sha1").update(`blob ${source.length}\0`).update(source).digest("hex"),
      witness.blob,
    );
    witnesses.set(key, witness);
  }
  const counts = { attributeRanges: 0, scopedPlainText: 0, newComponentEvents: 0 };
  const used = new Set<string>();
  assert.equal(successors.entries.length, 6);
  for (const successor of successors.entries) {
    const predecessor = alignments.get(successor.path);
    assert.ok(predecessor);
    assert.equal(hash(successor.previousRaw), predecessor.currentSha256);
    assert.equal(successor.alignment.path, predecessor.path);
    verifyLintSnapshotAlignment(successor.previousRaw, successor.currentRaw, successor.alignment);
    const before = JSON.parse(successor.previousRaw) as Packet[];
    const after = JSON.parse(successor.currentRaw) as Packet[];
    for (const [index, packet] of before.entries()) {
      if (canonical(packet) === canonical(after[index])) continue;
      const key = `${successor.project}/${packet.file}`;
      const witness = witnesses.get(key);
      assert.ok(witness);
      assert.equal(witness.fixturePin, successor.fixturePin);
      used.add(key);
      verifySuccessorPolicy(packet, after[index], witness.source, counts);
    }
    predecessor.successor = {
      previousRaw: successor.previousRaw,
      fixturePin: successor.fixturePin,
      review: successor.alignment,
    };
  }
  assert.deepEqual(counts, successors.policyCounts);
  assert.deepEqual(counts, { attributeRanges: 1112, scopedPlainText: 55, newComponentEvents: 4 });
  assert.equal(used.size, 395);
  assert.equal(used.size, witnesses.size, "every changed whole packet retains its pinned source");
}

function sourceRange(source: string, message: Record<string, any>): string {
  // Lint JSON columns count Unicode characters, with one-based exclusive ends.
  const lines = source.split("\n").map((line) => Array.from(line));
  const start = message.line - 1;
  const end = message.endLine - 1;
  if (start === end) return lines[start].slice(message.column - 1, message.endColumn - 1).join("");
  return [
    lines[start].slice(message.column - 1).join(""),
    ...lines.slice(start + 1, end).map((line) => line.join("")),
    lines[end].slice(0, message.endColumn - 1).join(""),
  ].join("\n");
}

function verifySuccessorPolicy(
  before: Packet,
  after: Packet,
  source: string,
  counts: { attributeRanges: number; scopedPlainText: number; newComponentEvents: number },
) {
  const added = [...after.messages];
  const removed = before.messages.filter((message) => {
    const index = added.findIndex((candidate) => canonical(candidate) === canonical(message));
    if (index === -1) return true;
    added.splice(index, 1);
    return false;
  });
  for (const message of removed) {
    const attribute = message.ruleId === "vue/attribute-hyphenation";
    assert.ok(attribute || message.ruleId === "vue/require-scoped-style");
    const allowed = attribute ? ["line", "column", "endLine", "endColumn"] : ["help"];
    const stable = (value: Record<string, any>) =>
      canonical(
        Object.fromEntries(Object.entries(value).filter(([key]) => !allowed.includes(key))),
      );
    const candidates = added
      .map((candidate, index) => ({ candidate, index }))
      .filter(({ candidate }) => stable(candidate) === stable(message))
      .sort(
        (left, right) =>
          Math.abs(left.candidate.line - message.line) * 10_000 +
          Math.abs(left.candidate.column - message.column) -
          (Math.abs(right.candidate.line - message.line) * 10_000 +
            Math.abs(right.candidate.column - message.column)),
      );
    assert.ok(candidates.length > 0, "a whole diagnostic may change only reviewed fields");
    const { candidate, index } = candidates[0];
    added.splice(index, 1);
    if (attribute) {
      const name = sourceRange(source, message);
      const whole = sourceRange(source, candidate);
      assert.match(name, /^[A-Za-z_:][A-Za-z\d_:.-]*$/);
      assert.ok(
        whole.includes(name),
        "the new range contains the original authored attribute name",
      );
      assert.match(whole, /^(?::|v-bind:|v-model:)?[A-Za-z_]/);
      assert.ok(whole.length >= name.length);
      const comparePosition = (
        leftLine: number,
        leftColumn: number,
        rightLine: number,
        rightColumn: number,
      ) => leftLine - rightLine || leftColumn - rightColumn;
      assert.ok(
        comparePosition(candidate.line, candidate.column, message.line, message.column) <= 0,
      );
      assert.ok(
        comparePosition(
          message.endLine,
          message.endColumn,
          candidate.endLine,
          candidate.endColumn,
        ) <= 0,
      );
      counts.attributeRanges++;
    } else {
      assert.equal(candidate.help, message.help.replaceAll("`", ""));
      counts.scopedPlainText++;
    }
  }
  for (const message of added) {
    assert.equal(message.ruleId, "vue/v-on-event-hyphenation");
    assert.equal(message.severity, 1);
    const listener = sourceRange(source, message);
    const lines = source.split("\n");
    const prefix = [
      ...lines.slice(0, message.line - 1),
      Array.from(lines[message.line - 1])
        .slice(0, message.column - 1)
        .join(""),
    ].join("\n");
    const tag = /^<([\w-]+)/.exec(prefix.slice(prefix.lastIndexOf("<")))?.[1];
    assert.ok(tag && (tag === "component" || /^[A-Z]/.test(tag) || tag.includes("-")));
    const event = /^@([^=\s.]+)=/.exec(listener)?.[1];
    assert.ok(event && /[A-Z]/.test(event));
    const suggested = event.replace(/([a-z\d])([A-Z])/g, "$1-$2").toLowerCase();
    assert.equal(
      message.message,
      `[vize:vue/v-on-event-hyphenation] Custom event listeners on components should be hyphenated: use '${suggested}'`,
    );
    counts.newComponentEvents++;
  }
  const { messages: _beforeMessages, warningCount: beforeWarnings, ...beforeFields } = before;
  const { messages: _afterMessages, warningCount: afterWarnings, ...afterFields } = after;
  assert.deepEqual(afterFields, beforeFields, "all other whole packet fields stay intact");
  assert.equal(afterWarnings, beforeWarnings + added.length);
}

export function verifyLintSnapshotAlignment(
  previousRaw: string,
  currentRaw: string,
  review: LintSnapshotAlignment,
): void {
  if (review.successor) {
    const { successor, ...predecessor } = review;
    verifyLintSnapshotAlignment(previousRaw, successor.previousRaw, predecessor);
    verifyLintSnapshotAlignment(successor.previousRaw, currentRaw, successor.review);
    return;
  }
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
