/** Exact profile schema plus selected source callsites; absence is unknown. */
import assert from "node:assert/strict";
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
export function censusProfile(profile: Profile, sfcs: number) {
  assert.equal(profile.schema_version, 1);
  assert.equal(profile.truncation.dropped_spans, 0);
  assert.equal(profile.truncation.dropped_counters, 0);
  assert.ok(profile.allocation, "profiling allocator must actually be installed");
  assert.ok(profile.micro && profile.micro.length > 0);
  const micro = profile.micro.filter(
    (row) =>
      row.name.startsWith("canon.dependency_scan.") ||
      row.name.startsWith("canon.import_facts.") ||
      row.name.startsWith("canon.import_rewriter.") ||
      row.name.startsWith("croquis.oxc."),
  );
  const amount = (name: string) => {
    const rows = micro.filter((row) => row.name === name && row.kind === "counter");
    return rows.length ? rows.reduce((sum, row) => sum + row.units, 0) : null;
  };
  const cloneCalls = amount("canon.dependency_scan.content.clone.calls"),
    cloneSourceBytes = amount("canon.dependency_scan.content.clone.source_bytes"),
    generatedSourceBytes = amount("canon.dependency_scan.content.generated_source_bytes");
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
    "dependency precheck must actually reach occurrence collection",
  );
  return {
    sfcs,
    micro,
    spans,
    cloneCalls,
    cloneSourceBytes,
    generatedSourceBytes,
    cachedOccurrences: cached,
    uncachedOccurrences: uncached,
    allocation: profile.allocation,
    actualHashOperations: null,
    physicalCopyBytes: null,
    sameContentParseIdentity: null,
    peakLiveBytes: null,
    processTreeRss: null,
    hardwareInstructions: null,
    parserAccounting: {
      analysisEntrySpans: [
        "canon.import_facts.collect_spans.parse",
        "canon.import_rewriter.rewrite_parse",
      ],
      physicalParserSpans: [
        "croquis.oxc.original",
        "croquis.oxc.recovery",
        "canon.import_facts.collect_spans.recovery_parse",
        "canon.import_rewriter.declaration_parse",
      ],
      coverage: "selected Parser::parse sites only; other direct OXC sites remain uninstrumented",
    },
    scope:
      "source callsite entries and logical source lengths; synchronous Rust span allocation requests; cumulative process allocations are separate; absent branch sites are unobserved, never hardware/software zero; nested spans change self attribution",
  };
}

export type CensusProfileReceipt = {
  stage: string;
  rawProfileSha256: string;
  value: ReturnType<typeof censusProfile>;
};
export function packCensusProfiles(profiles: CensusProfileReceipt[]) {
  const sites = new Map<string, number>();
  const sourceSites: Omit<NonNullable<Profile["micro"]>[number], "calls" | "units">[] = [];
  for (const profile of profiles)
    for (const row of profile.value.micro) {
      if (!sites.has(row.key)) {
        sites.set(row.key, sourceSites.length);
        const { calls: _calls, units: _units, ...metadata } = row;
        sourceSites.push(metadata);
      }
    }
  const selectedProfiles = [1, 10, 100].map((sfcs) => {
    const profile = profiles.find((row) => row.value.sfcs === sfcs);
    assert.ok(profile);
    const { micro, spans, ...rest } = profile.value;
    return {
      stage: profile.stage,
      rawProfileSha256: profile.rawProfileSha256,
      value: {
        ...rest,
        micro: micro.map(({ key, calls, units }) => ({ site: sites.get(key), calls, units })),
        spans: spans.map(({ key, ...metrics }) => ({ site: sites.get(key), ...metrics })),
      },
    };
  });

  return {
    sourceSites,
    profiles: selectedProfiles,
    profileReceipts: profiles.map(({ stage, rawProfileSha256, value }) => ({
      stage,
      rawProfileSha256,
      sfcs: value.sfcs,
      cloneCalls: value.cloneCalls,
      cloneSourceBytes: value.cloneSourceBytes,
    })),
  };
}
