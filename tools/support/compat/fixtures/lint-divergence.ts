import { createHash } from "node:crypto";
import { isAbsolute } from "node:path";

import {
  byteOrder,
  collectBaselineFindings,
  collectPatinaFindings,
  compareRecords,
  indexRuleMap,
  invalid,
  normalizePath,
} from "./lint-divergence-records.mjs";

/**
 * Compare a patina lint run against an `eslint-plugin-vue` run over the same
 * files and classify every finding, mirroring
 * [`compareTypecheckDiagnostics`](./typecheck-divergence.mjs) for the linter.
 *
 * Two linters agreeing on a *count* is not parity, so findings are normally
 * matched on the full location tuple — file, rule, severity and the whole
 * `line`/`column`/`endLine`/`endColumn` range. Message wording is deliberately
 * excluded from that identity: the two implementations phrase the same finding
 * differently, and forcing byte equality there would bury real location
 * divergences under prose diffs. A matched pair whose wording differs is still
 * recorded, in `messageDifferences`, so the wording gap stays visible.
 *
 * A small set of mapped rules has a stable, rule-owned anchor difference where
 * Patina and eslint-plugin-vue report the same semantic finding on different
 * AST nodes. Those pairs are removed from false-positive/false-negative counts
 * and recorded in `ruleLocationDivergences`, preserving both source ranges and
 * messages so the location mismatch remains reviewable.
 *
 * Baseline findings are routed by the status their rule carries in the
 * checked-in rule map (`tests/_fixtures/patina-eslint-vue-rule-map.json`):
 *
 * - `mapped` — compared against the patina rule it names. An unmatched baseline
 *   finding is a false negative; an unmatched patina finding for a rule some
 *   entry maps to is a false positive.
 * - `unimplemented` — a known coverage gap with a tracking issue, collected in
 *   `unimplemented` rather than counted as a parity failure.
 * - `intentional-divergence` — a documented decision not to match upstream,
 *   collected in `intentionalDivergences`.
 *
 * Patina findings for rules no map entry targets are patina-only coverage, not
 * false positives; they land in `patinaOnlyRuleFindings`.
 *
 * `documentedDivergences` is the reviewed ledger of expected differences. An
 * entry can only ever cancel exactly one false positive against exactly one
 * false negative that share a file and a rule pair, so a patina-only or
 * upstream-only finding can never be absorbed by one.
 */
export function compareLintFindings({
  projectId,
  cwd,
  ruleMap,
  patinaFindings,
  eslintResults,
  documentedDivergences = [],
}) {
  if (typeof projectId !== "string" || projectId.length === 0) invalid("project id is required");
  if (typeof cwd !== "string" || !isAbsolute(cwd)) invalid("cwd must be absolute");
  const { byUpstream, patinaTargets } = indexRuleMap(ruleMap);
  const patina = collectPatinaFindings(patinaFindings, cwd);
  const baselineInput = collectBaselineFindings(eslintResults, cwd);

  const unimplemented = [];
  const intentionalDivergences = [];
  const comparableBaseline = [];
  for (const finding of baselineInput.findings) {
    const entry = byUpstream.get(finding.ruleId);
    if (entry == null) {
      invalid(`baseline rule ${finding.ruleId} is absent from the pinned rule map`);
    }
    if (entry.status === "unimplemented") {
      unimplemented.push({ ...finding, issue: entry.issue });
      continue;
    }
    if (entry.status === "intentional-divergence") {
      intentionalDivergences.push({ ...finding, reason: entry.reason });
      continue;
    }
    comparableBaseline.push({
      ...finding,
      ruleId: entry.patinaRule,
      upstreamRuleId: finding.ruleId,
    });
  }

  const patinaOnlyRuleFindings = [];
  const comparablePatina = [];
  for (const finding of patina) {
    (patinaTargets.has(finding.ruleId) ? comparablePatina : patinaOnlyRuleFindings).push(finding);
  }

  const patinaGroups = groupByIdentity(comparablePatina);
  const baselineGroups = groupByIdentity(comparableBaseline);
  const identities = [...new Set([...patinaGroups.keys(), ...baselineGroups.keys()])].sort(
    byteOrder,
  );
  const shared = [];
  const messageDifferences = [];
  const falsePositives = [];
  const falseNegatives = [];

  for (const identity of identities) {
    const candidates = patinaGroups.get(identity) ?? [];
    const expected = baselineGroups.get(identity) ?? [];
    const commonCount = Math.min(candidates.length, expected.length);
    for (let index = 0; index < commonCount; index += 1) {
      const candidate = candidates[index];
      const pair = {
        file: candidate.file,
        ruleId: candidate.ruleId,
        upstreamRuleId: expected[index].upstreamRuleId,
        severity: candidate.severity,
        line: candidate.line,
        column: candidate.column,
        endLine: candidate.endLine,
        endColumn: candidate.endColumn,
        patinaMessage: candidate.message,
        baselineMessage: expected[index].message,
      };
      shared.push(pair);
      if (pair.patinaMessage !== pair.baselineMessage) messageDifferences.push(pair);
    }
    falsePositives.push(...candidates.slice(commonCount));
    falseNegatives.push(...expected.slice(commonCount));
  }

  shared.sort(compareShared);
  messageDifferences.sort(compareShared);
  falsePositives.sort(compareRecords);
  falseNegatives.sort(compareRecords);
  unimplemented.sort(compareRecords);
  intentionalDivergences.sort(compareRecords);
  patinaOnlyRuleFindings.sort(compareRecords);
  const documented = pairDocumentedDivergences(
    selectDocumentedDivergences(documentedDivergences, projectId, cwd),
    falsePositives,
    falseNegatives,
  );
  const ruleLocationDivergences = pairRuleLocationDivergences(falsePositives, falseNegatives);

  const classified = {
    shared,
    messageDifferences,
    falsePositives,
    falseNegatives,
    ruleLocationDivergences,
    unimplemented,
    intentionalDivergences,
    patinaOnlyRuleFindings,
    documentedDivergences: documented,
  };
  const summary = {
    patinaFindingCount: patina.length,
    baselineFindingCount: baselineInput.findings.length,
    comparableBaselineCount: comparableBaseline.length,
    sharedCount: shared.length,
    messageDifferenceCount: messageDifferences.length,
    documentedDivergenceCount: documented.length,
    ruleLocationDivergenceCount: ruleLocationDivergences.length,
    falsePositiveCount: falsePositives.length,
    falseNegativeCount: falseNegatives.length,
    unimplementedCount: unimplemented.length,
    intentionalDivergenceCount: intentionalDivergences.length,
    patinaOnlyRuleFindingCount: patinaOnlyRuleFindings.length,
    baselineParseErrorCount: baselineInput.parseErrorCount,
    baselineExcludedNonVueCount: baselineInput.excludedNonVueCount,
    baselineInvalidRangeCount: baselineInput.invalidRangeCount,
    falsePositiveRatio: ratio(falsePositives.length, comparablePatina.length),
    falseNegativeRatio: ratio(falseNegatives.length, comparableBaseline.length),
  };
  return {
    schema: "vize.fixtureLintDivergence",
    version: 1,
    project: projectId,
    upstream: { package: ruleMap.upstream.package, version: ruleMap.upstream.version },
    summary,
    ...classified,
    sha256: createHash("sha256")
      .update(JSON.stringify({ summary, ...classified }))
      .digest("hex"),
  };
}

function selectDocumentedDivergences(values, projectId, cwd) {
  if (!Array.isArray(values)) invalid("documented divergences must be an array");
  const selected = [];
  const identities = new Set();
  for (const [index, value] of values.entries()) {
    const label = `documented divergence ${index}`;
    if (value == null || typeof value !== "object") invalid(`${label} must be an object`);
    if (typeof value.project !== "string" || value.project.length === 0) {
      invalid(`${label} must name a project`);
    }
    const file = normalizePath(value.file, cwd, `${label}.file`);
    if (!file.endsWith(".vue")) invalid(`${label} must reference a .vue file`);
    if (typeof value.ruleId !== "string" || value.ruleId.length === 0) {
      invalid(`${label}.ruleId must name the patina rule`);
    }
    const patina = documentedSide(value.patina, `${label}.patina`);
    const baseline = documentedSide(value.baseline, `${label}.baseline`);
    if (sameSpan(patina, baseline) && patina.severity === baseline.severity) {
      invalid(`${label} must record a difference between the two linters`);
    }
    if (!Number.isSafeInteger(value.issue) || value.issue < 1) {
      invalid(`${label}.issue must be the tracking issue number`);
    }
    // A ledger entry suppresses a real divergence, so it has to carry a written
    // rationale a reviewer can check rather than a placeholder.
    const reason = typeof value.reason === "string" ? value.reason.replace(/\s+/g, " ").trim() : "";
    if (reason.length < 40) invalid(`${label}.reason must explain why the divergence is expected`);
    const identity = [value.project, file, value.ruleId, patina.line, patina.column].join("\0");
    if (identities.has(identity)) invalid(`${label} duplicates an earlier documented divergence`);
    identities.add(identity);
    if (value.project !== projectId) continue;
    selected.push({ file, ruleId: value.ruleId, patina, baseline, issue: value.issue, reason });
  }
  return selected.sort(
    (left, right) =>
      byteOrder(left.file, right.file) ||
      byteOrder(left.ruleId, right.ruleId) ||
      left.patina.line - right.patina.line ||
      left.patina.column - right.patina.column,
  );
}

function pairDocumentedDivergences(expected, falsePositives, falseNegatives) {
  const paired = [];
  for (const divergence of expected) {
    const positiveIndex = findDocumented(falsePositives, divergence, divergence.patina);
    const negativeIndex = findDocumented(falseNegatives, divergence, divergence.baseline);
    if (positiveIndex < 0 || negativeIndex < 0) continue;
    falsePositives.splice(positiveIndex, 1);
    falseNegatives.splice(negativeIndex, 1);
    paired.push(divergence);
  }
  return paired;
}

function findDocumented(records, divergence, side) {
  return records.findIndex(
    (candidate) =>
      candidate.file === divergence.file &&
      candidate.ruleId === divergence.ruleId &&
      candidate.severity === side.severity &&
      sameSpan(candidate, side),
  );
}

function documentedSide(value, label) {
  if (value == null || typeof value !== "object") invalid(`${label} must be an object`);
  if (value.severity !== "error" && value.severity !== "warning") {
    invalid(`${label}.severity must be error or warning`);
  }
  const span = {};
  for (const field of ["line", "column", "endLine", "endColumn"]) {
    if (!Number.isSafeInteger(value[field]) || value[field] < 1) {
      invalid(`${label}.${field} must be a positive safe integer`);
    }
    span[field] = value[field];
  }
  return { severity: value.severity, ...span };
}

function pairRuleLocationDivergences(falsePositives, falseNegatives) {
  const paired = [];
  for (let positiveIndex = 0; positiveIndex < falsePositives.length;) {
    const positive = falsePositives[positiveIndex];
    const match = findRuleLocationDivergence(positive, falseNegatives);
    if (match == null) {
      positiveIndex += 1;
      continue;
    }
    const [negative] = falseNegatives.splice(match.negativeIndex, 1);
    falsePositives.splice(positiveIndex, 1);
    paired.push(ruleLocationDivergence(positive, negative, match));
  }
  return paired.sort(compareRuleLocationDivergences);
}

function findRuleLocationDivergence(positive, falseNegatives) {
  const rule = ruleLocationDivergenceRules[positive.ruleId];
  if (rule == null) return null;
  for (let negativeIndex = 0; negativeIndex < falseNegatives.length; negativeIndex += 1) {
    const negative = falseNegatives[negativeIndex];
    if (
      positive.file !== negative.file ||
      positive.ruleId !== negative.ruleId ||
      positive.severity !== negative.severity
    ) {
      continue;
    }
    const subject = rule.subject(positive, negative);
    if (subject == null) continue;
    return { negativeIndex, reason: rule.reason, subject };
  }
  return null;
}

const ruleLocationDivergenceRules = {
  "vue/no-unused-components": {
    reason:
      "patina reports the SFC/script anchor while eslint-plugin-vue reports the component registration property.",
    subject(positive, negative) {
      const positiveComponent = unusedComponentName(positive.message);
      if (
        positiveComponent == null ||
        positiveComponent !== unusedComponentName(negative.message)
      ) {
        return null;
      }
      return `component:${positiveComponent}`;
    },
  },
  "vue/require-v-for-key": {
    reason:
      "patina reports the v-for directive range while eslint-plugin-vue reports the owning element range.",
    subject(positive, negative) {
      return containsSpan(negative, positive) ? "missing-v-for-key" : null;
    },
  },
};

function ruleLocationDivergence(positive, negative, match) {
  return {
    file: positive.file,
    ruleId: positive.ruleId,
    upstreamRuleId: negative.upstreamRuleId,
    severity: positive.severity,
    subject: match.subject,
    reason: match.reason,
    patina: side(positive),
    baseline: side(negative),
  };
}

function side(value) {
  return {
    line: value.line,
    column: value.column,
    endLine: value.endLine,
    endColumn: value.endColumn,
    message: value.message,
  };
}

function unusedComponentName(message) {
  return (
    message.match(/Component '([^']+)' is registered but never used(?: in template)?/u)?.[1] ??
    message.match(/The "([^"]+)" component has been registered but not used\./u)?.[1] ??
    null
  );
}

function sameSpan(left, right) {
  return (
    left.line === right.line &&
    left.column === right.column &&
    left.endLine === right.endLine &&
    left.endColumn === right.endColumn
  );
}

function containsSpan(outer, inner) {
  return compareSpanStart(outer, inner) <= 0 && compareSpanEnd(outer, inner) >= 0;
}

function compareSpanStart(left, right) {
  return left.line - right.line || left.column - right.column;
}

function compareSpanEnd(left, right) {
  return left.endLine - right.endLine || left.endColumn - right.endColumn;
}

function groupByIdentity(records) {
  const groups = new Map();
  for (const value of records) {
    const identity = [
      value.file,
      value.ruleId,
      value.severity,
      value.line,
      value.column,
      value.endLine,
      value.endColumn,
    ].join("\0");
    const group = groups.get(identity) ?? [];
    group.push(value);
    groups.set(identity, group);
  }
  for (const group of groups.values()) group.sort(compareRecords);
  return groups;
}

function compareShared(left, right) {
  return (
    byteOrder(left.file, right.file) ||
    left.line - right.line ||
    left.column - right.column ||
    left.endLine - right.endLine ||
    left.endColumn - right.endColumn ||
    byteOrder(left.ruleId, right.ruleId) ||
    byteOrder(left.patinaMessage, right.patinaMessage) ||
    byteOrder(left.baselineMessage, right.baselineMessage)
  );
}

function compareRuleLocationDivergences(left, right) {
  return (
    byteOrder(left.file, right.file) ||
    byteOrder(left.ruleId, right.ruleId) ||
    byteOrder(left.subject, right.subject) ||
    compareSpanStart(left.patina, right.patina) ||
    compareSpanEnd(left.patina, right.patina) ||
    compareSpanStart(left.baseline, right.baseline) ||
    compareSpanEnd(left.baseline, right.baseline) ||
    byteOrder(left.patina.message, right.patina.message) ||
    byteOrder(left.baseline.message, right.baseline.message)
  );
}

function ratio(count, total) {
  return total === 0 ? 0 : count / total;
}
