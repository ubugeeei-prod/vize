import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import {
  CRATE_QUOTA_BUDGET,
  CRATE_QUOTA_MARGIN_MS,
  CRATE_QUOTA_WINDOW_MS,
  inspectCrateQuota,
  readRollingCrateQuota,
  type RegistrySnapshot,
} from "../../../tools/support/release/release/quota.ts";
import { historyUrl, page } from "../../../tools/support/release/release/quota/registry.ts";

// Only this 51-byte response is frozen from the failed R2 observation. All
// version rows, cursors, clocks and HTTP responses below are explicitly mocked.
const terminalBody = '{"versions":[],"meta":{"total":0,"next_page":null}}';
const terminalHash = "9efd5e82e7fd659164b32d6f3b933a3b9b930e74f69058f11599c0d5129241c8";
const now = Date.parse("2026-10-31T08:22:32Z");
const initial = historyUrl("alpha", String(now));
const cursor = (index: number) =>
  `?sort=date&per_page=100&vize_quota_guard=${now}&seek=mock-boundary-${index}`;
function mockRows(count: number, recent: number) {
  return Array.from({ length: count }, (_, index) => ({
    id: count - index,
    crate: "alpha",
    num: index === 0 ? "0.609.0" : `0.1.${index}`,
    created_at: new Date(
      now -
        (index < recent ? 0 : CRATE_QUOTA_WINDOW_MS + CRATE_QUOTA_MARGIN_MS + 1) -
        index * 60_000,
    ).toISOString(),
    yanked: index === 1,
    checksum: "a".repeat(64),
    trustpub_data: null,
  }));
}
type Packet = {
  versions: ReturnType<typeof mockRows>;
  meta: { total: number; next_page: string | null };
};
function mockHistory(count: number, recent = 18): RegistrySnapshot[] {
  const rows = mockRows(count, recent);
  const snapshots: RegistrySnapshot[] = [];
  for (let offset = 0; offset < count; offset += 100) {
    snapshots.push({
      url: offset === 0 ? initial : new URL(cursor(offset / 100), initial).href,
      status: 200,
      date: new Date(now).toUTCString(),
      age: null,
      body: JSON.stringify({
        versions: rows.slice(offset, offset + 100),
        meta: { total: count, next_page: cursor(Math.floor(offset / 100) + 1) },
      }),
    });
  }
  snapshots.push({
    url: new URL(cursor(Math.ceil(count / 100)), initial).href,
    status: 200,
    date: new Date(now).toUTCString(),
    age: null,
    body: terminalBody,
  });
  return snapshots;
}
function inspect(snapshots: RegistrySnapshot[]) {
  return inspectCrateQuota("alpha", "0.609.0", "0.610.0", snapshots, null, null, now);
}
function edit(snapshots: RegistrySnapshot[], index: number, change: (packet: Packet) => void) {
  const packet = JSON.parse(snapshots[index]!.body) as Packet;
  change(packet);
  snapshots[index]!.body = JSON.stringify(packet);
}

test("the frozen 51-byte terminal seek packet retains its actual zero total", () => {
  assert.equal(Buffer.byteLength(terminalBody), 51);
  assert.equal(createHash("sha256").update(terminalBody).digest("hex"), terminalHash);
  const snapshot = mockHistory(100).at(-1)!;
  const decoded = page(snapshot, now);
  assert.deepEqual(decoded, { rows: [], total: 0, next: null, terminal: true });
  assert.equal(snapshot.body, terminalBody);
});

for (const count of [100, 200, 600]) {
  test(`mocked ${count}-version full-page boundary closes only with its terminal seek`, () => {
    const snapshots = mockHistory(count);
    const result = inspect(snapshots);
    assert.equal(result.totalVersions, count);
    assert.equal(result.pages.length, count / 100 + 1);
    assert.equal(result.pages.at(-1)!.bytes, 51);
    assert.equal(result.pages.at(-1)!.sha256, terminalHash);
    assert.equal(result.windowCount, 18);
    assert.equal(result.proposedCount, 19);
    assert.equal(result.newWrites, 1);
    assert.equal(result.allowed, true);
    assert.equal(page(snapshots.at(-1)!, now).total, 0);
    assert.equal(snapshots.at(-1)!.body, terminalBody);
  });
}

test("mocked HTTP enumeration retains the terminal raw digest and the 19/reserve1 policy", async () => {
  const snapshots = mockHistory(600);
  const requests: string[] = [];
  const mockFetch: typeof fetch = async (input, init) => {
    const url = input instanceof Request ? input.url : String(input);
    requests.push(url);
    assert.equal(init?.redirect, "error");
    const snapshot = snapshots.find((value) => value.url === url);
    assert.ok(snapshot, `unexpected mocked registry request: ${url}`);
    return new Response(snapshot.body, {
      status: snapshot.status,
      headers: { Date: snapshot.date! },
    });
  };
  const result = await readRollingCrateQuota(
    ["alpha"],
    "0.609.0",
    "0.610.0",
    null,
    mockFetch,
    () => now,
  );
  assert.equal(result.kind, "allowed");
  assert.equal(result.budget, CRATE_QUOTA_BUDGET);
  assert.equal(result.reserve, 1);
  assert.equal(result.remainingCrateWrites, 1);
  assert.deepEqual(result.failures, []);
  assert.deepEqual(
    requests,
    snapshots.map((value) => value.url),
  );
  assert.equal(result.observations[0]!.totalVersions, 600);
  assert.deepEqual(result.requests.at(-1), {
    url: snapshots.at(-1)!.url,
    status: 200,
    date: snapshots.at(-1)!.date,
    age: null,
    bytes: 51,
    sha256: terminalHash,
    completeBody: true,
  });
});

test("a terminal boundary does not admit an over-budget new write or waive ownership", () => {
  const result = inspect(mockHistory(600, 19));
  assert.equal(result.windowCount, 19);
  assert.equal(result.proposedCount, 20);
  assert.equal(result.newWrites, 1);
  assert.equal(result.allowed, false);
  assert.ok(Date.parse(result.nextEligibleAt) > now);
  const existing = mockHistory(100);
  edit(existing, 0, (packet) => {
    packet.versions[1]!.num = "0.610.0";
  });
  assert.throws(() => inspect(existing), /genuine producer/);
});

test("first, early, partial, missing or continued terminal pages never prove coverage", () => {
  const missing = mockHistory(100).slice(0, -1);
  assert.throws(() => inspect(missing), /entire history/);
  const first = { ...mockHistory(100).at(-1)!, url: initial };
  assert.throws(() => inspect([first]));
  assert.throws(() => inspect([mockHistory(100).at(-1)!]));
  const early = mockHistory(200);
  early.splice(1, 1);
  early[1]!.url = new URL(cursor(1), initial).href;
  assert.throws(() => inspect(early));
  assert.throws(() => inspect(mockHistory(150)));
  const continued = mockHistory(100);
  continued.push({ ...continued.at(-1)!, url: new URL(cursor(2), initial).href });
  assert.throws(() => inspect(continued), /cursor/);
});

test("terminal shape and seek requirements reject missing or contradictory metadata", () => {
  for (const body of [
    '{"versions":[],"meta":{"total":100,"next_page":null}}',
    '{"versions":[],"meta":{"total":"0","next_page":null}}',
    '{"versions":[],"meta":{"total":0}}',
    JSON.stringify({ versions: [], meta: { total: 0, next_page: cursor(2) } }),
    '{"versions":[],"meta":{"next_page":null}}',
    '{"versions":[],"meta":null}',
    '{"versions":null,"meta":{"total":0,"next_page":null}}',
  ]) {
    const snapshots = mockHistory(100);
    snapshots.at(-1)!.body = body;
    assert.throws(() => inspect(snapshots));
  }
  const terminal = mockHistory(100).at(-1)!;
  for (const url of [initial, initial + "&seek="]) {
    assert.throws(() => page({ ...terminal, url }, now));
  }
});

test("terminal handling preserves stable totals, unique rows, order and authentic cursor chains", () => {
  const mutations: Array<(snapshots: RegistrySnapshot[]) => void> = [
    (s) =>
      edit(s, 1, (p) => {
        p.meta.total = 199;
      }),
    (s) =>
      edit(s, 0, (p) => {
        p.meta.total = 201;
      }),
    (s) =>
      edit(s, 1, (p) => {
        p.versions[0]!.id = 200;
      }),
    (s) =>
      edit(s, 1, (p) => {
        p.versions[0]!.num = "0.609.0";
      }),
    (s) =>
      edit(s, 1, (p) => {
        p.versions[0]!.created_at = new Date(now).toISOString();
      }),
    (s) =>
      edit(s, 0, (p) => {
        p.meta.next_page = cursor(1).replace("sort=date", "sort=semver");
      }),
    (s) =>
      edit(s, 0, (p) => {
        p.meta.next_page = cursor(1).replace(String(now), "foreign");
      }),
    (s) =>
      edit(s, 0, (p) => {
        p.meta.next_page = "https://example.com/seek";
      }),
    (s) => {
      s[1]!.url = s[0]!.url;
    },
    (s) => {
      s.at(-1)!.url = new URL(cursor(3), initial).href;
    },
  ];
  for (const mutate of mutations) {
    const snapshots = mockHistory(200);
    mutate(snapshots);
    assert.throws(() => inspect(snapshots));
  }
});

test("terminal evidence remains fresh, bounded and subject to successful HTTP status", () => {
  for (const mutation of [
    { date: null },
    { date: new Date(now - 121_000).toUTCString() },
    { age: "61" },
    { status: 429 },
    { body: " ".repeat(2 * 1024 * 1024 + 1) },
  ]) {
    const snapshots = mockHistory(100);
    Object.assign(snapshots.at(-1)!, mutation);
    assert.throws(() => inspect(snapshots));
  }
  assert.throws(() => inspect(mockHistory(1000)), /complete history pages/);
});
