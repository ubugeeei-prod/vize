import { createHash } from "node:crypto";
import {
  CRATE_QUOTA_MARGIN_MS,
  MAX_BYTES,
  ORIGIN,
  PAGE_SIZE,
  type HistoryVersion,
  type RegistrySnapshot,
  type ReleaseOwnership,
} from "./types.ts";
export function requireValue(value: unknown, message: string): asserts value {
  if (!value) throw new Error(message);
}
export function object(value: unknown): Record<string, unknown> {
  requireValue(
    value !== null && typeof value === "object" && !Array.isArray(value),
    "object required",
  );
  return value as Record<string, unknown>;
}
export function historyUrl(crate: string, nonce: string): string {
  return `${ORIGIN}/api/v1/crates/${encodeURIComponent(crate)}/versions?sort=date&per_page=${PAGE_SIZE}&vize_quota_guard=${nonce}`;
}
export function continuation(value: unknown, current: string): string | null {
  if (value === null) return null;
  requireValue(typeof value === "string" && value.startsWith("?"), "relative seek cursor required");
  const next = new URL(value, current);
  const previous = new URL(current);
  requireValue(
    next.origin === ORIGIN && next.pathname === previous.pathname && next.hash === "",
    "cursor origin/path differs",
  );
  requireValue(
    next.searchParams.get("sort") === "date" &&
      next.searchParams.get("per_page") === String(PAGE_SIZE),
    "cursor query differs",
  );
  requireValue(
    next.searchParams.get("vize_quota_guard") === previous.searchParams.get("vize_quota_guard"),
    "cursor freshness nonce differs",
  );
  requireValue(Boolean(next.searchParams.get("seek")), "seek cursor missing");
  requireValue(
    [...next.searchParams.keys()].every((key) =>
      ["sort", "per_page", "vize_quota_guard", "seek"].includes(key),
    ),
    "unexpected cursor filter",
  );
  return next.href;
}
export function evidence(snapshot: RegistrySnapshot, now: number, expectedStatus = 200) {
  requireValue(snapshot.status === expectedStatus, `registry HTTP ${snapshot.status}`);
  const date = Date.parse(snapshot.date ?? "");
  requireValue(
    Number.isFinite(date) && Math.abs(now - date) <= CRATE_QUOTA_MARGIN_MS,
    "stale or missing response Date",
  );
  if (snapshot.age !== null) {
    requireValue(/^\d+$/.test(snapshot.age) && Number(snapshot.age) <= 60, "stale or invalid Age");
  }
  const bytes = Buffer.byteLength(snapshot.body);
  requireValue(bytes > 0 && bytes <= MAX_BYTES, "registry response size");
  const { body, ...headers } = snapshot;
  return { ...headers, bytes, sha256: createHash("sha256").update(body).digest("hex") };
}
export function page(snapshot: RegistrySnapshot, now: number) {
  evidence(snapshot, now);
  const body = object(JSON.parse(snapshot.body) as unknown);
  requireValue(
    Array.isArray(body.versions) && body.versions.length > 0 && body.versions.length <= PAGE_SIZE,
    "history versions required",
  );
  const meta = object(body.meta);
  requireValue(
    Number.isSafeInteger(meta.total) && Number(meta.total) >= body.versions.length,
    "history total required",
  );
  return {
    rows: body.versions as unknown[],
    total: Number(meta.total),
    next: continuation(meta.next_page, snapshot.url),
  };
}
export function versionRow(value: unknown, crate: string, now: number): HistoryVersion {
  const row = object(value);
  requireValue(row.crate === crate, "crate identity differs");
  requireValue(Number.isSafeInteger(row.id) && Number(row.id) > 0, "version ID required");
  requireValue(
    typeof row.num === "string" &&
      /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(row.num),
    "version required",
  );
  requireValue(
    typeof row.created_at === "string" &&
      /^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d(?:\.\d+)?Z$/.test(row.created_at),
    "creation timestamp required",
  );
  const timestamp = Date.parse(row.created_at);
  requireValue(
    Number.isFinite(timestamp) && timestamp <= now + 60_000,
    "invalid or future creation timestamp",
  );
  requireValue(typeof row.yanked === "boolean", "yanked state required");
  requireValue(
    typeof row.checksum === "string" && /^[0-9a-f]{64}$/.test(row.checksum),
    "crate checksum required",
  );
  return {
    id: Number(row.id),
    num: row.num,
    timestamp,
    createdAt: row.created_at,
    yanked: row.yanked,
    checksum: row.checksum,
    trustpub: row.trustpub_data,
  };
}
export function owns(row: HistoryVersion, ownership: ReleaseOwnership): void {
  const proof = object(row.trustpub);
  requireValue(
    row.num === ownership.version &&
      proof.provider === "github" &&
      proof.repository === "ubugeeei-prod/vize" &&
      String(proof.run_id) === ownership.runId &&
      proof.sha === ownership.workflowHead,
    "same version is not owned by the immutable release producer",
  );
}
