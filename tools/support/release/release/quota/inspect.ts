import { evidence, object, owns, page, requireValue, versionRow } from "./registry.ts";
import {
  CRATE_QUOTA_BUDGET,
  CRATE_QUOTA_MARGIN_MS,
  CRATE_QUOTA_WINDOW_MS,
  MAX_PAGES,
  ORIGIN,
  PAGE_SIZE,
  type CrateQuotaObservation,
  type HistoryVersion,
  type RegistrySnapshot,
  type ReleaseOwnership,
} from "./types.ts";
/** Enumerate the entire history; no date-order or first-page coverage inference. */
export function inspectCrateQuota(
  crate: string,
  latestRelease: string,
  target: string,
  snapshots: readonly RegistrySnapshot[],
  exactProof: RegistrySnapshot | null,
  ownership: ReleaseOwnership | null,
  now: number,
  priorProof: RegistrySnapshot | null = null,
): CrateQuotaObservation {
  requireValue(Number.isFinite(now), "invalid clock");
  requireValue(
    snapshots.length > 0 && snapshots.length <= MAX_PAGES,
    "complete history pages required",
  );
  let expected: string | null = snapshots[0]?.url ?? null;
  const first = new URL(expected ?? "");
  requireValue(
    first.origin === ORIGIN &&
      first.pathname === "/api/v1/crates/" + encodeURIComponent(crate) + "/versions",
    "initial history origin/path differs",
  );
  requireValue(
    first.searchParams.get("sort") === "date" &&
      first.searchParams.get("per_page") === String(PAGE_SIZE) &&
      !first.searchParams.has("seek"),
    "initial history sort/page differs",
  );
  requireValue(
    Boolean(first.searchParams.get("vize_quota_guard")),
    "initial freshness nonce missing",
  );
  const urls = new Set<string>(),
    ids = new Set<number>(),
    numbers = new Set<string>();
  const versions: HistoryVersion[] = [];
  let total: number | null = null,
    previous = Infinity;
  for (const snapshot of snapshots) {
    requireValue(
      expected !== null && snapshot.url === expected && !urls.has(snapshot.url),
      "history cursor loop or missing page",
    );
    urls.add(snapshot.url);
    const decoded = page(snapshot, now);
    requireValue(total === null || total === decoded.total, "history total changed during read");
    total = decoded.total;
    for (const value of decoded.rows) {
      const row = versionRow(value, crate, now);
      requireValue(!ids.has(row.id) && !numbers.has(row.num), "duplicate history ID or version");
      requireValue(row.timestamp <= previous, "history not sorted by creation time");
      ids.add(row.id);
      numbers.add(row.num);
      previous = row.timestamp;
      versions.push(row);
    }
    expected = decoded.next;
  }
  requireValue(expected === null && total === versions.length, "entire history not enumerated");
  let priorMissing: CrateQuotaObservation["priorMissing"] = null;
  if (!numbers.has(latestRelease)) {
    requireValue(priorProof !== null, "latest completed release missing from history");
    const expectedProof = new URL(priorProof.url);
    requireValue(
      expectedProof.origin === ORIGIN &&
        expectedProof.pathname ===
          "/api/v1/crates/" + encodeURIComponent(crate) + "/" + encodeURIComponent(latestRelease),
      "prior-version proof identity differs",
    );
    priorMissing = evidence(priorProof, now, 404);
  }
  const own = versions.find((row) => row.num === target);
  let ownVersion: CrateQuotaObservation["ownVersion"] = null;
  if (own !== undefined) {
    requireValue(
      ownership !== null && ownership.version === target && exactProof !== null,
      "existing candidate requires genuine producer and exact-version proof",
    );
    owns(own, ownership);
    const rawProof = evidence(exactProof, now),
      body = object(JSON.parse(exactProof.body) as unknown);
    const exact = versionRow(body.version, crate, now);
    owns(exact, ownership);
    requireValue(
      exact.id === own.id && exact.checksum === own.checksum && exact.createdAt === own.createdAt,
      "candidate exact-version checksum/identity mismatch",
    );
    ownVersion = { checksum: own.checksum, proof: rawProof };
  }
  const recent = versions.filter(
    (row) => row.timestamp >= now - CRATE_QUOTA_WINDOW_MS - CRATE_QUOTA_MARGIN_MS,
  );
  const newWrites = ownVersion === null ? 1 : 0,
    proposedCount = recent.length + newWrites;
  const allowed = newWrites === 0 || proposedCount <= CRATE_QUOTA_BUDGET;
  const expiring = [...recent].reverse()[proposedCount - CRATE_QUOTA_BUDGET - 1];
  requireValue(allowed || expiring !== undefined, "next eligible timestamp unavailable");
  let next = now;
  if (!allowed) {
    requireValue(expiring !== undefined, "next expiry required");
    next = expiring.timestamp + CRATE_QUOTA_WINDOW_MS + CRATE_QUOTA_MARGIN_MS + 1;
  }
  const nextEligibleAt = new Date(next).toISOString();
  return {
    crate,
    totalVersions: versions.length,
    windowCount: recent.length,
    newWrites,
    proposedCount,
    allowed,
    nextEligibleAt,
    recent,
    pages: snapshots.map((s) => evidence(s, now)),
    ownVersion,
    priorMissing,
  };
}
