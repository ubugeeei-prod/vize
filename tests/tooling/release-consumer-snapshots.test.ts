import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { object, type Json } from "../../tools/support/release/consumer_recovery/policy.ts";
import {
  sameOriginalSnapshot,
  sameRunSnapshot,
} from "../../tools/support/release/consumer_recovery/snapshots.ts";

const fixture = (name: string): Json =>
  JSON.parse(readFileSync(new URL(`../_fixtures/release/${name}.json`, import.meta.url), "utf8"));
const retained = fixture("consumer-recovery");
function frozen(value: Json): Json {
  for (const child of Object.values(value))
    if (child && typeof child === "object") frozen(child as Json);
  return Object.freeze(value);
}
const before = { pushed_at: "2026-10-17T01:30:03Z", updated_at: "2026-10-17T01:25:44Z" };
const after = { pushed_at: "2026-10-17T01:34:38Z", updated_at: "2026-10-17T01:33:25Z" };
const run = structuredClone(object(retained.run));
const repositories = ["repository", "head_repository"];
for (const key of repositories) Object.assign(object(run[key]), before);

test("run permits only repository clock values without changing either raw packet", () => {
  const current = structuredClone(run);
  for (const key of repositories) Object.assign(object(current[key]), after);
  assert.notDeepEqual(current, run, "the former raw comparison must reproduce failure");
  const rawBefore = [JSON.stringify(run), JSON.stringify(current)];
  sameRunSnapshot(frozen(current), frozen(run), "repository clocks changed");
  assert.deepEqual([JSON.stringify(run), JSON.stringify(current)], rawBefore);
});

for (const key of repositories) {
  test(`run keeps ${key} field presence, type and other metadata exact`, () => {
    for (const field of ["pushed_at", "updated_at"]) {
      for (const replacement of [null, 123, {}, undefined]) {
        const current = structuredClone(run);
        const repository = object(current[key]);
        if (replacement === undefined) delete repository[field];
        else repository[field] = replacement;
        assert.throws(() => sameRunSnapshot(current, run, "repository clock schema changed"));
      }
    }
    for (const [field, replacement] of [
      ["id", 1],
      ["full_name", "foreign/vize"],
      ["created_at", "2026-10-17T02:00:00Z"],
      ["default_branch", "other"],
      ["size", -1],
    ]) {
      const current = structuredClone(run);
      object(current[key])[String(field)] = replacement;
      assert.throws(() => sameRunSnapshot(current, run, `repository ${field} changed`));
    }
  });
}

const runChanges: [string, unknown][] = [
  ["id", 1],
  ["head_sha", "f".repeat(40)],
  ["run_attempt", 5],
  ["event", "push"],
  ["head_branch", "other"],
  ["path", ".github/workflows/other.yml"],
  ["status", "in_progress"],
  ["conclusion", "success"],
  ["updated_at", after.updated_at],
  ["pushed_at", after.pushed_at],
  ["created_at", after.updated_at],
];
for (const [field, value] of runChanges) {
  test(`original and recovery run comparisons refuse changed ${field}`, () => {
    const current = structuredClone(run);
    current[field] = value;
    assert.throws(() => sameRunSnapshot(current, run, "run changed"));
  });
}

test("original custody still compares every retained proof, job, artifact and tag field", () => {
  const original: Json = {
    ...retained,
    run,
    policy: { sha: "a".repeat(40), authority: [{ file: "policy.ts", sha256: "a".repeat(64) }] },
    logIdentity: { sha256: "a".repeat(64), bytes: String(retained.log).length },
    artifacts: [{ id: 123, digest: `sha256:${"a".repeat(64)}`, expired: false }],
    pin: { tagRef: { object: { sha: "a".repeat(40) } }, tagObject: { message: "original" } },
    sourceReceipt: { body: "original", head: { sha: run.head_sha } },
  };
  const current = structuredClone(original);
  for (const field of ["repository", "head_repository"])
    Object.assign(object(object(current.run)[field]), after);
  sameOriginalSnapshot(current, original, "during fresh consumers");
  for (const field of [
    "policy",
    "jobs",
    "failed",
    "log",
    "logIdentity",
    "blobs",
    "artifacts",
    "pin",
    "sourceReceipt",
  ]) {
    const changed = structuredClone(current);
    changed[field] = { replaced: changed[field] };
    assert.throws(() => sameOriginalSnapshot(changed, original, "after requalification"), field);
  }
});

test("unrecognized repository-shaped data and nested clocks remain strict", () => {
  for (const original of [
    { ...run, extra: { repository: before } },
    { ...run, repository: { ...object(run.repository), owner: before } },
  ]) {
    const current: Json = structuredClone(original);
    if (Object.hasOwn(current, "extra"))
      Object.assign(object(object(current.extra).repository), after);
    else Object.assign(object(object(current.repository).owner), after);
    assert.throws(() => sameRunSnapshot(current, original, "unrecognized clock changed"));
  }
});
