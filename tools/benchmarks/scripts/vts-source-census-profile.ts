/** Exact profiling schema plus collision-resistant, count-preserving source identity receipts. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export const VTS_SOURCE_CENSUS_SIZES = [1, 10, 100] as const;
export const VTS_SOURCE_CENSUS_REPETITIONS = 3;

export interface Profile {
  schema_version: number;
  micro?: {
    key: string;
    name: string;
    file: string;
    module: string;
    kind: string;
    line: number;
    calls: number;
    units: number;
  }[];
  spans: {
    key: string;
    count: number;
    wall_ns: unknown;
    alloc: null | { calls: number; bytes: number; self_calls: number; self_bytes: number };
  }[];
  allocation: unknown;
  truncation: { dropped_spans: number; dropped_counters: number };
}

export type IdentityEntry = {
  sourceType: string;
  byteLength: number;
  sha256: string;
  count: number;
};

export type IdentityComparison = {
  producerCalls: number;
  consumerCalls: number;
  matchedCalls: number;
  producerOnlyCalls: number;
  consumerOnlyCalls: number;
  producerComplete: boolean;
  consumerComplete: boolean;
  matches: boolean;
};

export type IdentityReceipt = {
  schema: string;
  version: number;
  algorithm: string;
  domain: string;
  identity: string;
  multiplicity: string;
  producers: IdentityEntry[];
  consumers: IdentityEntry[];
  comparison: IdentityComparison;
};

const sha256 = (value: string) => createHash("sha256").update(value).digest("hex");
const entryKey = (entry: IdentityEntry) =>
  JSON.stringify([entry.sourceType, entry.byteLength, entry.sha256]);
const compareText = (left: string, right: string) => (left < right ? -1 : left > right ? 1 : 0);
const compareEntry = (left: IdentityEntry, right: IdentityEntry) =>
  compareText(left.sourceType, right.sourceType) ||
  left.byteLength - right.byteLength ||
  compareText(left.sha256, right.sha256);

function validateEntries(entries: IdentityEntry[], role: string) {
  assert.ok(Array.isArray(entries));
  assert.deepEqual(entries, [...entries].sort(compareEntry), `${role} multiset must be sorted`);
  const keys = new Set<string>();
  for (const entry of entries) {
    assert.ok(entry.sourceType.length > 0);
    assert.ok(Number.isSafeInteger(entry.byteLength) && entry.byteLength >= 0);
    assert.match(entry.sha256, /^[a-f0-9]{64}$/u);
    assert.ok(Number.isSafeInteger(entry.count) && entry.count > 0);
    assert.ok(!keys.has(entryKey(entry)), `${role} identity must be unique before multiplicity`);
    keys.add(entryKey(entry));
  }
}

export function compareIdentityMultisets(
  producers: IdentityEntry[],
  consumers: IdentityEntry[],
): IdentityComparison {
  validateEntries(producers, "producer");
  validateEntries(consumers, "consumer");
  const producer = new Map(producers.map((entry) => [entryKey(entry), entry.count])),
    consumer = new Map(consumers.map((entry) => [entryKey(entry), entry.count]));
  let matchedCalls = 0,
    producerOnlyCalls = 0,
    consumerOnlyCalls = 0;
  for (const [key, count] of producer) {
    const other = consumer.get(key) ?? 0;
    matchedCalls += Math.min(count, other);
    producerOnlyCalls += Math.max(count - other, 0);
  }
  for (const [key, count] of consumer)
    consumerOnlyCalls += Math.max(count - (producer.get(key) ?? 0), 0);
  const producerCalls = producers.reduce((sum, entry) => sum + entry.count, 0),
    consumerCalls = consumers.reduce((sum, entry) => sum + entry.count, 0);
  return {
    producerCalls,
    consumerCalls,
    matchedCalls,
    producerOnlyCalls,
    consumerOnlyCalls,
    producerComplete: matchedCalls + producerOnlyCalls === producerCalls,
    consumerComplete: matchedCalls + consumerOnlyCalls === consumerCalls,
    matches: consumerOnlyCalls === 0,
  };
}

export function validateIdentityReceipt(receipt: IdentityReceipt) {
  assert.equal(receipt.schema, "vize.vts-source-identity");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.algorithm, "sha256");
  assert.equal(receipt.domain, "vize.vts-source-census.sha256.v1");
  assert.match(receipt.identity, /SourceType/u);
  assert.match(receipt.multiplicity, /multiset/u);
  const comparison = compareIdentityMultisets(receipt.producers, receipt.consumers);
  assert.deepEqual(receipt.comparison, comparison);
  assert.equal(comparison.matchedCalls + comparison.consumerOnlyCalls, comparison.consumerCalls);
  assert.equal(comparison.matchedCalls + comparison.producerOnlyCalls, comparison.producerCalls);
  assert.equal(comparison.matches, comparison.consumerOnlyCalls === 0);
  return {
    ...comparison,
    algorithm: receipt.algorithm,
    domain: receipt.domain,
    sourceTypeIncluded: true,
    multiplicityIncluded: true,
    producerMultisetSha256: sha256(JSON.stringify(receipt.producers)),
    consumerMultisetSha256: sha256(JSON.stringify(receipt.consumers)),
  };
}

export function censusProfile(
  profile: Profile,
  identityReceipt: IdentityReceipt,
  identityReceiptSha256: string,
  sfcs: number,
) {
  assert.equal(profile.schema_version, 1);
  assert.equal(profile.truncation.dropped_spans, 0);
  assert.equal(profile.truncation.dropped_counters, 0);
  assert.ok(profile.allocation, "profiling allocator must actually be installed");
  assert.ok(profile.micro && profile.micro.length > 0);
  assert.match(identityReceiptSha256, /^[a-f0-9]{64}$/u);
  const micro = profile.micro.filter(
    (row) =>
      row.name.startsWith("canon.dependency_scan.") ||
      row.name.startsWith("canon.import_facts.") ||
      row.name.startsWith("canon.import_rewriter.") ||
      row.name.startsWith("canon.source_census."),
  );
  const amount = (name: string) => {
    const rows = micro.filter((row) => row.name === name && row.kind === "counter");
    return rows.length ? rows.reduce((sum, row) => sum + row.units, 0) : null;
  };
  const cloneCalls = amount("canon.dependency_scan.content.clone.calls"),
    cloneSourceBytes = amount("canon.dependency_scan.content.clone.source_bytes"),
    generatedSourceBytes = amount("canon.dependency_scan.content.generated_source_bytes"),
    producerCalls = amount("canon.source_census.producer.calls");
  assert.ok(cloneCalls !== null && cloneCalls >= sfcs);
  assert.ok(
    cloneSourceBytes !== null &&
      generatedSourceBytes !== null &&
      generatedSourceBytes > 0 &&
      cloneSourceBytes >= generatedSourceBytes,
  );
  const spans = profile.spans.filter((row) =>
    micro.some((site) => site.kind === "span" && site.key === row.key),
  );
  const cloneSites = micro.filter(
    (row) => row.name === "canon.dependency_scan.content.clone" && row.kind === "span",
  );
  assert.ok(cloneSites.length > 0);
  for (const site of cloneSites) {
    const row = spans.find((span) => span.key === site.key);
    assert.ok(row && row.alloc && row.alloc.calls > 0 && row.alloc.bytes > 0);
    assert.equal(row.count, site.calls);
  }
  const cached = amount("canon.dependency_scan.occurrences.cached.calls"),
    uncached = amount("canon.dependency_scan.occurrences.uncached.calls");
  assert.ok(
    (cached ?? 0) + (uncached ?? 0) > 0,
    "cached and uncached dependency occurrence counters are both empty",
  );
  assert.ok(producerCalls !== null && producerCalls > 0);
  assert.ok(uncached !== null && uncached > 0);
  const identity = validateIdentityReceipt(identityReceipt);
  assert.ok(identity.consumerCalls > 0, "identity cannot pass vacuously without consumers");
  assert.equal(identity.producerCalls, producerCalls, "producer census must cover every hook");
  assert.equal(identity.consumerCalls, uncached, "consumer census must cover every uncached parse");
  return {
    sfcs,
    micro,
    spans,
    cloneCalls,
    cloneSourceBytes,
    generatedSourceBytes,
    cachedOccurrences: cached,
    uncachedOccurrences: uncached,
    producerParseInputs: producerCalls,
    allocation: profile.allocation,
    actualHashOperations: identity.producerCalls + identity.consumerCalls,
    physicalCopyBytes: null,
    sameContentParseIdentity: { ...identity, receiptSha256: identityReceiptSha256 },
    peakLiveBytes: null,
    processTreeRss: null,
    hardwareInstructions: null,
    parserAccounting: {
      analysisEntrySpans: [
        "canon.import_facts.collect_spans.parse",
        "canon.import_rewriter.rewrite_parse",
      ],
      identityScope:
        "generated rewrite parse producers versus every uncached dependency parse consumer",
    },
    scope:
      "digest observers run outside parse spans; their map allocations are included only in process-wide profiling allocation, never in shipping builds or parse-span allocation",
  };
}

export type CensusProfileReceipt = {
  stage: string;
  rawProfileSha256: string;
  rawIdentitySha256: string;
  value: ReturnType<typeof censusProfile>;
};

export function packCensusProfiles(profiles: CensusProfileReceipt[]) {
  const sites = new Map<string, number>();
  const sourceSites: Omit<NonNullable<Profile["micro"]>[number], "calls" | "units">[] = [];
  for (const profile of profiles)
    for (const row of profile.value.micro)
      if (!sites.has(row.key)) {
        sites.set(row.key, sourceSites.length);
        const { calls: _calls, units: _units, ...metadata } = row;
        sourceSites.push(metadata);
      }
  const selectedProfiles = VTS_SOURCE_CENSUS_SIZES.map((sfcs) => {
    const repetitions = profiles.filter((row) => row.value.sfcs === sfcs);
    assert.equal(
      repetitions.length,
      VTS_SOURCE_CENSUS_REPETITIONS,
      `${sfcs} SFCs require ${VTS_SOURCE_CENSUS_REPETITIONS} profiling repetitions`,
    );
    return {
      sfcs,
      repetitions: repetitions.map((profile) => {
        const { micro, spans, ...rest } = profile.value;
        return {
          stage: profile.stage,
          rawProfileSha256: profile.rawProfileSha256,
          rawIdentitySha256: profile.rawIdentitySha256,
          value: {
            ...rest,
            micro: micro.map(({ key, calls, units }) => ({ site: sites.get(key), calls, units })),
            spans: spans.map(({ key, ...metrics }) => ({ site: sites.get(key), ...metrics })),
          },
        };
      }),
    };
  });
  return {
    sourceSites,
    profiles: selectedProfiles,
    profileReceipts: profiles.map(({ stage, rawProfileSha256, rawIdentitySha256, value }) => ({
      stage,
      rawProfileSha256,
      rawIdentitySha256,
      sfcs: value.sfcs,
      identityMatches: value.sameContentParseIdentity.matches,
      producerMultisetSha256: value.sameContentParseIdentity.producerMultisetSha256,
      consumerMultisetSha256: value.sameContentParseIdentity.consumerMultisetSha256,
      identityReceiptSha256: value.sameContentParseIdentity.receiptSha256,
      matchedCalls: value.sameContentParseIdentity.matchedCalls,
      producerOnlyCalls: value.sameContentParseIdentity.producerOnlyCalls,
      consumerOnlyCalls: value.sameContentParseIdentity.consumerOnlyCalls,
    })),
  };
}
