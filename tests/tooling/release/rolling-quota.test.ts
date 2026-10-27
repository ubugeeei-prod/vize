import { mockCrate, mockProduced, mockProducerResponse } from "./quota-fixtures.ts";
import assert from "node:assert/strict";
import { test } from "node:test";
import {
  CRATE_QUOTA_BUDGET,
  CRATE_QUOTA_MARGIN_MS,
  CRATE_QUOTA_WINDOW_MS,
  inspectCrateQuota,
  readRollingCrateQuota,
  type RegistrySnapshot,
  type ReleaseOwnership,
} from "../../../tools/support/release/release/quota.ts";

const now = Date.parse("2026-10-28T00:00:00Z");
const checksum = "a".repeat(64);
const owner: ReleaseOwnership = {
  version: "0.602.0",
  runId: "42",
  workflowHead: "b".repeat(40),
  candidateSha: "c".repeat(40),
  producerSource: "d".repeat(40),
  runAttempt: 1,
  nativeManifestGitBlob: "e".repeat(40),
  inventoryGitBlob: "f".repeat(40),
};
function url(crate = "alpha") {
  return `https://crates.io/api/v1/crates/${crate}/versions?sort=date&per_page=100&vize_quota_guard=${now}`;
}
function rows(count: number, crate = "alpha", own = false) {
  return Array.from({ length: count }, (_, i) => ({
    id: i + 1,
    crate,
    num: own && i === 0 ? "0.602.0" : i === (own ? 1 : 0) ? "0.601.0" : `0.600.${i}`,
    created_at: new Date(now - i * 30 * 60_000).toISOString(),
    yanked: i === 2,
    checksum: own && i === 0 ? mockCrate(crate, "0.602.0", owner.candidateSha).checksum : checksum,
    trustpub_data: {
      provider: "github",
      repository: "ubugeeei-prod/vize",
      run_id: "42",
      sha: owner.workflowHead,
    },
  }));
}
type Version = ReturnType<typeof rows>[number];
function snapshot(
  versions: Version[],
  total = versions.length,
  next: string | null = null,
  requestUrl = url(),
): RegistrySnapshot {
  return {
    url: requestUrl,
    status: 200,
    date: new Date(now).toUTCString(),
    age: null,
    body: JSON.stringify({ versions, meta: { total, next_page: next } }),
  };
}
function exact(
  row: Version,
  requestUrl = "https://crates.io/api/v1/crates/alpha/0.602.0",
): RegistrySnapshot {
  return { ...snapshot([]), url: requestUrl, body: JSON.stringify({ version: row }) };
}
function inspect(
  versions: Version[],
  ownership: ReleaseOwnership | null = null,
  proof: RegistrySnapshot | null = null,
) {
  return inspectCrateQuota(
    "alpha",
    "0.601.0",
    "0.602.0",
    [snapshot(versions)],
    proof,
    ownership,
    now,
    null,
    ownership === null ? null : mockProduced(ownership).proof,
    ownership === null ? null : mockCrate("alpha", "0.602.0", ownership.candidateSha).proof,
  );
}
test("19-after-write policy counts yanked and prerelease versions", () => {
  const values = rows(18);
  values[3]!.num = "0.600.3-rc.1";
  const result = inspect(values);
  assert.equal(CRATE_QUOTA_BUDGET, 19);
  assert.equal(result.windowCount, 18);
  assert.equal(result.proposedCount, 19);
  assert.equal(result.newWrites, 1);
  assert.equal(result.allowed, true);
  assert.equal(result.recent.filter((row) => row.yanked).length, 1);
  assert.equal(
    result.recent.some((row) => row.num.endsWith("-rc.1")),
    true,
  );
  assert.match(result.pages[0]!.sha256, /^[a-f0-9]{64}$/);
});
test("the next version is refused at 19 and has an exact conservative eligibility time", () => {
  const values = rows(19),
    result = inspect(values);
  assert.equal(result.allowed, false);
  assert.equal(result.proposedCount, 20);
  assert.equal(
    result.nextEligibleAt,
    new Date(
      Date.parse(values.at(-1)!.created_at) + CRATE_QUOTA_WINDOW_MS + CRATE_QUOTA_MARGIN_MS + 1,
    ).toISOString(),
  );
});
test("the rolling boundary is inclusive with a two-minute margin", () => {
  const values = rows(2);
  values[1]!.created_at = new Date(
    now - CRATE_QUOTA_WINDOW_MS - CRATE_QUOTA_MARGIN_MS,
  ).toISOString();
  assert.equal(inspect(values).windowCount, 2);
  values[1]!.created_at = new Date(
    now - CRATE_QUOTA_WINDOW_MS - CRATE_QUOTA_MARGIN_MS - 1,
  ).toISOString();
  assert.equal(inspect(values).windowCount, 1);
});
test("an old last row never proves coverage when a seek page is outstanding", () => {
  const values = rows(3);
  values[1]!.created_at = new Date(now - 25 * 60 * 60_000).toISOString();
  values[2]!.created_at = new Date(now - 26 * 60 * 60_000).toISOString();
  const seek = `?sort=date&per_page=100&vize_quota_guard=${now}&seek=next`;
  const first = snapshot(values.slice(0, 2), 3, seek);
  assert.throws(
    () => inspectCrateQuota("alpha", "0.601.0", "0.602.0", [first], null, null, now),
    /entire history/,
  );
  const last = snapshot(values.slice(2), 3, null, new URL(seek, url()).href);
  const result = inspectCrateQuota("alpha", "0.601.0", "0.602.0", [first, last], null, null, now);
  assert.equal(result.totalVersions, 3);
  assert.equal(result.pages.length, 2);
  assert.equal(result.windowCount, 1);
});
test("schema, IDs, version uniqueness and creation ordering fail closed", () => {
  const mutations: Array<(r: Version[]) => void> = [
    (r) => {
      r[1]!.id = r[0]!.id;
    },
    (r) => {
      r[1]!.num = r[0]!.num;
    },
    (r) => {
      r[0]!.crate = "foreign";
    },
    (r) => {
      r[0]!.created_at = "invalid";
    },
    (r) => {
      r[0]!.created_at = new Date(now + 61_000).toISOString();
    },
    (r) => {
      r[1]!.created_at = new Date(now + 1).toISOString();
    },
    (r) => {
      r[0]!.checksum = "wrong";
    },
    (r) => {
      r[0]!.num = "broken";
    },
    (r) => {
      r[0]!.num = "0.599.0";
    },
  ];
  for (const mutate of mutations) {
    const values = rows(2);
    mutate(values);
    assert.throws(() => inspect(values));
  }
});
test("stale/missing dates, stale Age and invalid sort or continuation are refused", () => {
  for (const change of [
    { date: null },
    { date: new Date(now - 121_000).toUTCString() },
    { age: "61" },
    { age: "-1" },
    { age: "NaN" },
    { url: url().replace("sort=date", "sort=semver") },
  ]) {
    const value = { ...snapshot(rows(2)), ...change };
    assert.throws(() => inspectCrateQuota("alpha", "0.601.0", "0.602.0", [value], null, null, now));
  }
  const foreign = snapshot(rows(1), 2, "https://example.com/steal");
  assert.throws(
    () => inspectCrateQuota("alpha", "0.601.0", "0.602.0", [foreign], null, null, now),
    /relative seek/,
  );
});
test("same-number foreign versions are not recovery admission", () => {
  const values = rows(25, "alpha", true),
    proof = exact(values[0]!);
  assert.throws(() => inspect(values), /genuine producer/);
  assert.throws(
    () => inspect(values, { ...owner, runId: "43" }, proof),
    /immutable release producer/,
  );
  assert.throws(
    () => inspect(values, { ...owner, workflowHead: "e".repeat(40) }, proof),
    /immutable release producer/,
  );
});
test("recovery requires independent exact-version checksum equality", () => {
  const values = rows(25, "alpha", true);
  assert.throws(
    () => inspect(values, owner, exact({ ...values[0]!, checksum: "f".repeat(64) })),
    /checksum/,
  );
  const result = inspect(values, owner, exact(values[0]!));
  assert.equal(result.newWrites, 0);
  assert.equal(result.windowCount, 25);
  assert.equal(result.allowed, true);
  assert.equal(result.ownVersion?.checksum, values[0]!.checksum);
});
const response = (body: string, status = 200) =>
  new Response(body, { status, headers: { Date: new Date(now).toUTCString() } });
function mockRegistry(
  ownCrates: Set<string>,
  count = 25,
  badChecksum = false,
  wrongVcs = false,
): typeof fetch {
  return (async (input: string | URL | Request, init?: RequestInit) => {
    const produced = mockProducerResponse(owner, input, init);
    if (produced) return produced;
    const request = new URL(input instanceof Request ? input.url : input);
    const source = wrongVcs ? "0".repeat(40) : owner.candidateSha;
    if (request.origin === "https://static.crates.io") {
      const crate = request.pathname.split("/")[2]!;
      return new Response(new Uint8Array(mockCrate(crate, "0.602.0", source).archive));
    }
    const crate = request.pathname.split("/")[4]!;
    const own = ownCrates.has(crate),
      values = rows(own ? count : Math.min(count, 19), crate, own);
    if (own && wrongVcs) values[0]!.checksum = mockCrate(crate, "0.602.0", source).checksum;
    if (request.pathname.endsWith("/versions")) return response(snapshot(values).body);
    const candidate = values.find((row) => row.num === "0.602.0");
    assert.ok(candidate);
    return response(
      exact({ ...candidate, ...(badChecksum ? { checksum: "f".repeat(64) } : {}) }).body,
    );
  }) as typeof fetch;
}

test("mixed recovery gates only missing crate writes; a mismatched proof refuses the whole cut", async () => {
  const mixed = await readRollingCrateQuota(
    ["alpha", "beta"],
    "0.601.0",
    "0.602.0",
    owner,
    mockRegistry(new Set(["alpha"])),
    () => now,
  );
  assert.equal(mixed.kind, "blocked");
  assert.equal(mixed.remainingCrateWrites, 1);
  assert.equal(mixed.observations.find((r) => r.crate === "alpha")?.newWrites, 0);
  const mismatch = await readRollingCrateQuota(
    ["alpha", "beta"],
    "0.601.0",
    "0.602.0",
    owner,
    mockRegistry(new Set(["alpha"]), 25, true),
    () => now,
  );
  assert.equal(mismatch.kind, "unavailable");
  assert.match(mismatch.failures[0]!.message, /checksum/);
});
test("all 28 already-published own crates can finish channels outside the new-write budget", async () => {
  const names = Array.from({ length: 28 }, (_, i) => `crate_${i}`);
  const result = await readRollingCrateQuota(
    names,
    "0.601.0",
    "0.602.0",
    owner,
    mockRegistry(new Set(names)),
    () => now,
  );
  assert.equal(result.kind, "allowed");
  assert.equal(result.remainingCrateWrites, 0);
  assert.equal(result.observations.length, 28);
  assert.equal(
    result.observations.every((r) => r.windowCount === 25 && r.ownVersion !== null),
    true,
  );
});
test("429, malformed JSON, missing history and transport errors never admit a release", async () => {
  const mocks: Array<typeof fetch> = [
    (async () => response("rate limited", 429)) as typeof fetch,
    (async () => response("{")) as typeof fetch,
    (async () => response("{}")) as typeof fetch,
    (async () => {
      throw new Error("timeout");
    }) as typeof fetch,
  ];
  for (const fetchImpl of mocks) {
    const result = await readRollingCrateQuota(
      ["alpha"],
      "0.601.0",
      "0.602.0",
      null,
      fetchImpl,
      () => now,
    );
    assert.equal(result.kind, "unavailable");
    assert.equal(result.nextEligibleAt, null);
    assert.equal(result.failures.length, 1);
  }
});
test("expired observations, backwards/invalid clock and duplicate inventories are refused", async () => {
  const fetchImpl = mockRegistry(new Set(), 18);
  let calls = 0;
  const expired = await readRollingCrateQuota(
    ["alpha"],
    "0.601.0",
    "0.602.0",
    null,
    fetchImpl,
    () => (calls++ === 0 ? now : now + CRATE_QUOTA_MARGIN_MS + 1),
  );
  assert.equal(expired.kind, "unavailable");
  const duplicate = await readRollingCrateQuota(
    ["alpha", "alpha"],
    "0.601.0",
    "0.602.0",
    null,
    fetchImpl,
    () => now,
  );
  assert.equal(duplicate.kind, "unavailable");
  const invalid = await readRollingCrateQuota(
    ["alpha"],
    "0.601.0",
    "0.602.0",
    null,
    fetchImpl,
    () => NaN,
  );
  assert.equal(invalid.kind, "unavailable");
  calls = 0;
  const backwards = await readRollingCrateQuota(
    ["alpha"],
    "0.601.0",
    "0.602.0",
    null,
    fetchImpl,
    () => (calls++ === 0 ? now : now - 1),
  );
  assert.equal(backwards.kind, "unavailable");
});

test("a fresh complete public history admits an ordinary next cut", async () => {
  const result = await readRollingCrateQuota(
    ["alpha", "beta"],
    "0.601.0",
    "0.602.0",
    null,
    mockRegistry(new Set(), 18),
    () => now,
  );
  assert.equal(result.kind, "allowed");
  assert.equal(result.remainingCrateWrites, 2);
  assert.equal(
    result.observations.every((row) => row.proposedCount === 19),
    true,
  );
});

test("a genuinely absent prior crate stays informational; stale history is not treated as absence", () => {
  const values = rows(2);
  values[0]!.num = "0.599.0";
  const missing = {
    ...snapshot([]),
    status: 404,
    url: "https://crates.io/api/v1/crates/alpha/0.601.0?vize_quota_guard=" + now,
    body: '{"errors":[{"detail":"crate version does not exist"}]}',
  };
  const accepted = inspectCrateQuota(
    "alpha",
    "0.601.0",
    "0.602.0",
    [snapshot(values)],
    null,
    null,
    now,
    missing,
  );
  assert.equal(accepted.allowed, true);
  assert.equal(accepted.priorMissing?.status, 404);
  assert.throws(
    () =>
      inspectCrateQuota("alpha", "0.601.0", "0.602.0", [snapshot(values)], null, null, now, {
        ...missing,
        status: 200,
      }),
    /registry HTTP/,
  );
  assert.throws(
    () =>
      inspectCrateQuota("alpha", "0.601.0", "0.602.0", [snapshot(values)], null, null, now, {
        ...missing,
        url: "https://crates.io/api/v1/crates/foreign/0.601.0",
      }),
    /identity/,
  );
});

test("refusals retain the original HTTP packet hash rather than inventing a quota time", async () => {
  const result = await readRollingCrateQuota(
    ["alpha"],
    "0.601.0",
    "0.602.0",
    null,
    (async () => response("rate limited", 429)) as typeof fetch,
    () => now,
  );
  assert.equal(result.kind, "unavailable");
  assert.equal(result.requests[0]?.status, 429);
  assert.equal(result.requests[0]?.bytes, 12);
  assert.equal(result.requests[0]?.completeBody, true);
  assert.match(result.requests[0]?.sha256 ?? "", /^[a-f0-9]{64}$/);
  assert.equal(result.nextEligibleAt, null);
});
test("build metadata and prereleases both consume rolling slots", () => {
  const values = rows(18);
  values[2]!.num = "0.600.2-rc.1+build.7";
  assert.equal(inspect(values).windowCount, 18);
});

test("same R/H/version and consistent registry checksums cannot waive a different intended C", async () => {
  const result = await readRollingCrateQuota(
    ["alpha"],
    "0.601.0",
    "0.602.0",
    { ...owner, candidateSha: "9".repeat(40), producerSource: "8".repeat(40) },
    mockRegistry(new Set(["alpha"])),
    () => now,
  );
  assert.equal(result.kind, "unavailable");
  assert.match(result.failures[0]?.message ?? "", /matching intended candidate/);
});
test("a genuine producer C still refuses consistent foreign archive bytes whose Cargo source is another C", async () => {
  const result = await readRollingCrateQuota(
    ["alpha"],
    "0.601.0",
    "0.602.0",
    owner,
    mockRegistry(new Set(["alpha"]), 25, false, true),
    () => now,
  );
  assert.equal(result.kind, "unavailable");
  assert.match(result.failures[0]?.message ?? "", /Cargo VCS source/);
});
test("own-number pure packets without a genuine producer and crate source proof grant no zero-write waiver", () => {
  const values = rows(25, "alpha", true);
  assert.throws(
    () =>
      inspectCrateQuota(
        "alpha",
        "0.601.0",
        "0.602.0",
        [snapshot(values)],
        exact(values[0]!),
        owner,
        now,
      ),
    /original producer artifact/,
  );
});
