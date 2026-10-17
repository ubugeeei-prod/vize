import path from "node:path";
import {
  classifyReferenceCoverage,
  eligibleReferenceCoverage,
} from "../lint_reference_coverage.ts";
import {
  baselineIntentionalReason,
  collectBaselineFindings,
  collectPatinaFindings,
  compareRecords,
  groupByIdentity,
  outsideBaselineSurface,
  pairRuleLocationDivergences,
  recordValue,
  type LintRecord,
} from "./records.ts";
import {
  byteOrder,
  canonicalJson,
  get,
  object,
  readText,
  sha256,
  sortValues,
  text,
  type Json,
  type JsonObject,
} from "./value.ts";

export function indexRuleMap(ruleMap: Json) {
  const entries = object(get(ruleMap, "entries"));
  if (!entries) throw new Error("rule map must carry entries");
  const targets = new Set<string>();
  for (const ruleId of Object.keys(entries).sort(byteOrder)) {
    const entry = entries[ruleId];
    if (get(entry, "status") === "mapped") {
      const rule = get(entry, "patinaRule");
      if (typeof rule !== "string")
        throw new Error(`${ruleId} must name the patina rule it maps to`);
      targets.add(rule);
    }
  }
  return { entries, targets };
}
export function selectComparableRules(
  ruleMap: Json,
  preset: string | null,
  includeUnimplemented: boolean,
) {
  const entries = object(get(ruleMap, "entries"));
  if (!entries) throw new Error("rule map must carry entries");
  const rules: Record<string, string> = {};
  let mappedRuleCount = 0,
    skippedByPresetCount = 0;
  for (const upstream of Object.keys(entries).sort(byteOrder)) {
    const entry = entries[upstream],
      status = get(entry, "status");
    if (status === "unimplemented") {
      if (includeUnimplemented) rules[upstream] = "warn";
    } else if (status === "mapped") {
      mappedRuleCount++;
      if (typeof get(entry, "patinaRule") !== "string")
        throw new Error(`${upstream} must name patinaRule`);
      const presets = get(entry, "patinaPresets");
      if (preset !== null && (!Array.isArray(presets) || !presets.includes(preset))) {
        skippedByPresetCount++;
        continue;
      }
      rules[upstream] = get(entry, "patinaSeverity") === "error" ? "error" : "warn";
    } else if (status === "intentional-divergence") {
      /* explicitly outside the comparison */
    } else if (typeof status === "string")
      throw new Error(`${upstream} has unsupported status ${JSON.stringify(status)}`);
    else throw new Error(`${upstream} must record status`);
  }
  return { rules, mappedRuleCount, skippedByPresetCount };
}

export function compareLintFindings(
  project: string,
  cwd: string,
  ruleMap: Json,
  patinaFindings: Json[],
  eslintResults: Json[],
  context: JsonObject,
): JsonObject {
  const index = indexRuleMap(ruleMap),
    patina = collectPatinaFindings(patinaFindings, cwd),
    baseline = collectBaselineFindings(eslintResults, cwd);
  const sourceFor = (file: string): string | null => {
    try {
      return readText(path.join(cwd, file));
    } catch {
      return null;
    }
  };
  const expectations: Json[] = eligibleReferenceCoverage(project, context, sourceFor).map(
    (entry) => {
      const finding = entry.finding as Json;
      const observed = patina.filter(
        (record) => canonicalJson(recordValue(record)) === canonicalJson(finding),
      ).length;
      return {
        case: entry.id,
        expectedFinding: finding,
        sourceSha256: entry.sourceSha256,
        expectedCount: 1,
        observedCount: observed,
        passed: observed === 1,
      };
    },
  );
  const unimplemented: Json[] = [],
    intentionalDivergences: Json[] = [],
    comparableBaseline: LintRecord[] = [];
  for (const finding of baseline.findings) {
    const entry = index.entries[finding.ruleId];
    if (entry === undefined)
      throw new Error(`baseline rule ${finding.ruleId} is absent from the pinned rule map`);
    const status = text(get(entry, "status"));
    if (status === "unimplemented") unimplemented.push({ ...finding, issue: get(entry, "issue") });
    else if (status === "intentional-divergence")
      intentionalDivergences.push({ ...finding, reason: get(entry, "reason") });
    else if (status === "mapped") {
      const reason = baselineIntentionalReason(finding);
      if (reason !== null) intentionalDivergences.push({ ...finding, reason });
      else {
        const target = get(entry, "patinaRule");
        if (typeof target !== "string")
          throw new Error(`${finding.ruleId} must name the patina rule it maps to`);
        comparableBaseline.push({ ...finding, ruleId: target, upstreamRuleId: finding.ruleId });
      }
    } else throw new Error(`${finding.ruleId} has unsupported status ${JSON.stringify(status)}`);
  }
  const comparablePatina: LintRecord[] = [],
    patinaOnlyRuleFindings: Json[] = [],
    patinaOutsideBaselineSurface: Json[] = [];
  const sources = new Map<string, string | null>(),
    coveredCases = new Set<string>();
  for (const finding of patina) {
    if (!sources.has(finding.file)) sources.set(finding.file, sourceFor(finding.file));
    const source = sources.get(finding.file) ?? null;
    if (source !== null && outsideBaselineSurface(finding, source))
      patinaOutsideBaselineSurface.push(recordValue(finding));
    else if (index.targets.has(finding.ruleId)) {
      const coverage =
        source === null
          ? null
          : classifyReferenceCoverage(project, context, { ...finding }, source, () =>
              comparableBaseline.some(
                (other) => other.file === finding.file && other.ruleId === finding.ruleId,
              ),
            );
      const key = coverage === null ? "" : canonicalJson(coverage.case as Json);
      if (coverage !== null && !coveredCases.has(key)) {
        coveredCases.add(key);
        patinaOutsideBaselineSurface.push({ ...finding, referenceCoverage: coverage as Json });
      } else comparablePatina.push(finding);
    } else patinaOnlyRuleFindings.push(recordValue(finding));
  }
  const candidateGroups = groupByIdentity(comparablePatina),
    expectedGroups = groupByIdentity(comparableBaseline);
  const shared: Json[] = [],
    messageDifferences: Json[] = [],
    falsePositives: LintRecord[] = [],
    falseNegatives: LintRecord[] = [];
  for (const identity of [...new Set([...candidateGroups.keys(), ...expectedGroups.keys()])].sort(
    byteOrder,
  )) {
    const candidates = candidateGroups.get(identity) ?? [],
      expected = expectedGroups.get(identity) ?? [],
      common = Math.min(candidates.length, expected.length);
    for (let i = 0; i < common; i++) {
      const candidate = candidates[i],
        other = expected[i];
      const pair: Json = {
        file: candidate.file,
        ruleId: candidate.ruleId,
        upstreamRuleId: other.upstreamRuleId ?? null,
        severity: candidate.severity,
        line: candidate.line,
        column: candidate.column,
        endLine: candidate.endLine,
        endColumn: candidate.endColumn,
        patinaMessage: candidate.message,
        baselineMessage: other.message,
      };
      shared.push(pair);
      if (candidate.message !== other.message) messageDifferences.push(pair);
    }
    falsePositives.push(...candidates.slice(common));
    falseNegatives.push(...expected.slice(common));
  }
  const ruleLocationDivergences = sortValues(
    pairRuleLocationDivergences(falsePositives, falseNegatives),
  );
  falsePositives.sort(compareRecords);
  falseNegatives.sort(compareRecords);
  const ratio = (count: number, total: number) => (total === 0 ? 0 : count / total);
  const summary: JsonObject = {
    referenceCoverageIntegrityErrorCount: expectations.filter(
      (entry) => get(entry, "passed") !== true,
    ).length,
    patinaFindingCount:
      comparablePatina.length + patinaOnlyRuleFindings.length + patinaOutsideBaselineSurface.length,
    baselineFindingCount:
      comparableBaseline.length + unimplemented.length + intentionalDivergences.length,
    comparableBaselineCount: comparableBaseline.length,
    sharedCount: shared.length,
    messageDifferenceCount: messageDifferences.length,
    documentedDivergenceCount: 0,
    ruleLocationDivergenceCount: ruleLocationDivergences.length,
    falsePositiveCount: falsePositives.length,
    falseNegativeCount: falseNegatives.length,
    unimplementedCount: unimplemented.length,
    intentionalDivergenceCount: intentionalDivergences.length,
    patinaOnlyRuleFindingCount: patinaOnlyRuleFindings.length,
    patinaOutsideBaselineSurfaceCount: patinaOutsideBaselineSurface.length,
    baselineParseErrorCount: baseline.parseErrorCount,
    baselineExcludedNonVueCount: baseline.excludedNonVueCount,
    baselineInvalidRangeCount: baseline.invalidRangeCount,
    falsePositiveRatio: ratio(falsePositives.length, comparablePatina.length),
    falseNegativeRatio: ratio(falseNegatives.length, comparableBaseline.length),
  };
  const classified: JsonObject = {
    referenceCoverageExpectations: expectations,
    shared: sortValues(shared),
    messageDifferences: sortValues(messageDifferences),
    falsePositives: falsePositives.map(recordValue),
    falseNegatives: falseNegatives.map(recordValue),
    ruleLocationDivergences,
    unimplemented: sortValues(unimplemented),
    intentionalDivergences: sortValues(intentionalDivergences),
    patinaOnlyRuleFindings: sortValues(patinaOnlyRuleFindings),
    patinaOutsideBaselineSurface: sortValues(patinaOutsideBaselineSurface),
    documentedDivergences: [],
  };
  return {
    schema: "vize.fixtureLintDivergence",
    version: 1,
    project,
    upstream: {
      package: get(ruleMap, "upstream", "package"),
      version: get(ruleMap, "upstream", "version"),
    },
    summary,
    ...classified,
    sha256: sha256(canonicalJson({ summary, ...classified })),
  };
}
