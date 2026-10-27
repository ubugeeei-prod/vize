import { readProducedCandidate } from "./producer.ts";
import { readOwnedCrateSource } from "./crate_source.ts";
import { createHash } from "node:crypto";
import { historyUrl, object, page, requireValue } from "./registry.ts";
import { inspectCrateQuota } from "./inspect.ts";
import {
  CRATE_QUOTA_BUDGET,
  CRATE_QUOTA_MARGIN_MS,
  MAX_BYTES,
  MAX_PAGES,
  ORIGIN,
  type CrateQuotaObservation,
  type CrateQuotaReport,
  type RegistrySnapshot,
  type RegistryReadReceipt,
  type ReleaseOwnership,
  type ProducedCandidateProof,
} from "./types.ts";
async function boundedBody(response: Response): Promise<string> {
  requireValue(response.body, "registry body missing");
  const reader = response.body.getReader(),
    chunks: Uint8Array[] = [];
  let bytes = 0;
  try {
    for (;;) {
      const part = await reader.read();
      if (part.done) break;
      bytes += part.value.byteLength;
      requireValue(bytes <= MAX_BYTES, "registry body exceeds byte limit");
      chunks.push(part.value);
    }
  } finally {
    await reader.cancel().catch(() => {});
    reader.releaseLock();
  }
  return new TextDecoder("utf-8", { fatal: true }).decode(Buffer.concat(chunks));
}
export async function readRollingCrateQuota(
  crates: readonly string[],
  latestRelease: string | null,
  target: string,
  ownership: ReleaseOwnership | null = null,
  fetchImpl: typeof fetch = globalThis.fetch,
  clock: () => number = Date.now,
): Promise<CrateQuotaReport> {
  const now = clock(),
    observations: CrateQuotaObservation[] = [],
    failures: CrateQuotaReport["failures"] = [];
  const requests: RegistryReadReceipt[] = [];
  let producerRead: Promise<ProducedCandidateProof> | undefined;
  let produced: ProducedCandidateProof | null = null;
  const get = async (url: string): Promise<RegistrySnapshot> => {
    const witness: RegistryReadReceipt = {
      url,
      status: null,
      date: null,
      age: null,
      bytes: null,
      sha256: null,
      completeBody: false,
    };
    try {
      requireValue(
        Number.isFinite(clock()) && clock() >= now && clock() - now <= CRATE_QUOTA_MARGIN_MS,
        "quota read deadline or clock changed",
      );
      const response = await fetchImpl(url, {
        redirect: "error",
        headers: {
          "Cache-Control": "no-cache",
          "User-Agent": "vize-release-quota (https://github.com/ubugeeei-prod/vize)",
        },
        signal: AbortSignal.timeout(15_000),
      });
      witness.status = response.status;
      witness.date = response.headers.get("date");
      witness.age = response.headers.get("age");
      const body = await boundedBody(response);
      witness.bytes = Buffer.byteLength(body);
      witness.sha256 = createHash("sha256").update(body).digest("hex");
      witness.completeBody = true;
      requests.push(witness);
      return { url, status: response.status, date: witness.date, age: witness.age, body };
    } catch (error) {
      witness.error = String(error);
      requests.push(witness);
      throw error;
    }
  };
  if (
    !Number.isFinite(now) ||
    latestRelease === null ||
    crates.length === 0 ||
    new Set(crates).size !== crates.length
  ) {
    failures.push({
      crate: "*",
      message: "clock, canonical inventory and completed release required",
    });
  } else {
    for (let index = 0; index < crates.length; index += 4) {
      const rows = await Promise.all(
        crates.slice(index, index + 4).map(async (crate) => {
          try {
            requireValue(/^[a-zA-Z0-9_-]+$/.test(crate), "invalid crate name");
            const snapshots: RegistrySnapshot[] = [],
              visited = new Set<string>();
            let next: string | null = historyUrl(crate, String(now));
            while (next !== null) {
              requireValue(
                snapshots.length < MAX_PAGES && !visited.has(next),
                "history exceeds page bound or loops",
              );
              visited.add(next);
              const snapshot = await get(next);
              snapshots.push(snapshot);
              next = page(snapshot, now).next;
            }
            const hasTarget = snapshots.some((s) =>
              page(s, now).rows.some((v) => object(v).num === target),
            );
            const hasLatest = snapshots.some((s) =>
              page(s, now).rows.some((v) => object(v).num === latestRelease),
            );
            const priorProof = hasLatest
              ? null
              : await get(
                  `${ORIGIN}/api/v1/crates/${encodeURIComponent(crate)}/${encodeURIComponent(latestRelease)}?vize_quota_guard=${now}`,
                );
            const proof = hasTarget
              ? await get(
                  `${ORIGIN}/api/v1/crates/${encodeURIComponent(crate)}/${encodeURIComponent(target)}?vize_quota_guard=${now}`,
                )
              : null;
            let sourceProof = null;
            if (hasTarget) {
              requireValue(
                ownership !== null,
                "existing candidate cannot waive new writes without genuine ownership",
              );
              produced = await (producerRead ??= readProducedCandidate(ownership, fetchImpl));
              const existing = snapshots
                .flatMap((s) => page(s, now).rows)
                .map(object)
                .find((row) => row.num === target);
              requireValue(
                existing !== undefined && typeof existing.checksum === "string",
                "owned crate checksum required",
              );
              sourceProof = await readOwnedCrateSource(
                crate,
                target,
                existing.checksum,
                ownership.candidateSha,
                fetchImpl,
              );
            }
            return {
              observation: inspectCrateQuota(
                crate,
                latestRelease,
                target,
                snapshots,
                proof,
                ownership,
                now,
                priorProof,
                produced,
                sourceProof,
              ),
            };
          } catch (error) {
            return { failure: { crate, message: String(error) } };
          }
        }),
      );
      for (const row of rows) {
        if (row.observation) observations.push(row.observation);
        if (row.failure) failures.push(row.failure);
      }
    }
  }
  const finished = clock();
  if (!Number.isFinite(finished) || finished < now || finished - now > CRATE_QUOTA_MARGIN_MS)
    failures.push({ crate: "*", message: "clock changed or observations expired before decision" });
  const kind =
    failures.length > 0
      ? "unavailable"
      : observations.every((row) => row.allowed)
        ? "allowed"
        : "blocked";
  return {
    kind,
    checkedAt: Number.isFinite(now) ? new Date(now).toISOString() : "invalid-clock",
    budget: CRATE_QUOTA_BUDGET,
    reserve: 1,
    marginMs: CRATE_QUOTA_MARGIN_MS,
    remainingCrateWrites: observations.reduce((sum, row) => sum + row.newWrites, 0),
    nextEligibleAt:
      kind === "unavailable"
        ? null
        : new Date(
            Math.max(now, ...observations.map((r) => Date.parse(r.nextEligibleAt))),
          ).toISOString(),
    observations,
    failures,
    requests,
    produced,
  };
}
