// eslint-disable comment collection + vize-analog mapping for the
// suppression-telemetry FP oracle (Davinci P0-13).
//
// Comment parsing mirrors vize_patina/src/context/eslint_directive.rs so
// the oracle reasons about exactly the pragmas vize itself honors:
// marker priority eslint-disable-next-line > eslint-disable-line >
// eslint-disable > eslint-enable, rule lists split on commas/whitespace
// with `-- reason` tails and comment-closer tokens stripped.
//
// Because vize honors these pragmas natively (verified: a suppressed line
// produces no diagnostic), the telemetry run must lint DEFUSED copies —
// same byte length, pragma markers rewritten so coordinates are stable —
// or the intersection under measurement is empty by construction.
//
// Rule-name mapping: eslint-plugin-vue names come from the committed
// parity fixture tests/_fixtures/patina-eslint-vue-rule-map.json (entries
// with status "mapped"). Core-ESLint/other-plugin names have no verified
// vize analog today; they are REPORTED as unmapped, never errors, and the
// sidecar below only ever grows with verified pairs.

import fs from "node:fs";
import path from "node:path";

import { repoRoot } from "./fpfn-shared.mjs";

export const RULE_MAP_FIXTURE = "tests/_fixtures/patina-eslint-vue-rule-map.json";

// Verified core-ESLint (and non-vue-plugin) analogs. Empty on purpose: no
// vize rule has been verified as a behavioral analog of a core rule yet
// (`no-console`, `no-unused-vars`, … have no counterpart in any preset).
export const CORE_ESLINT_TO_VIZE = {};

const MARKERS = [
  ["eslint-disable-next-line", "next-line"],
  ["eslint-disable-line", "line"],
  ["eslint-disable", "block"],
  ["eslint-enable", "enable"],
];

/** Parse one source line; null when it carries no eslint pragma. */
export function parseSuppressionLine(lineText) {
  if (!lineText.includes("eslint-")) return null;
  for (const [marker, kind] of MARKERS) {
    const at = lineText.indexOf(marker);
    if (at === -1) continue;
    return { kind, rules: parseRuleList(lineText.slice(at + marker.length)) };
  }
  return null;
}

function parseRuleList(raw) {
  const beforeReason = raw.split("--")[0].replaceAll("*/", " ").replaceAll("-->", " ");
  return beforeReason
    .split(/[\s,]+/)
    .map((rule) => rule.replace(/^["'[\]{}();]+|["'[\]{}();]+$/g, ""))
    .filter((rule) => rule.length > 0);
}

/**
 * Collect every suppression in a source: comments plus the expanded
 * per-rule line ranges ({rule: name|null, startLine, endLine|null,
 * commentLine, kind}; rule null = bare disable covering all rules).
 */
export function scanSuppressions(source) {
  const comments = [];
  const ranges = [];
  const openBlocks = []; // indices into `ranges` with endLine null
  const lines = source.split("\n");
  for (let i = 0; i < lines.length; i += 1) {
    const lineNumber = i + 1;
    const parsed = parseSuppressionLine(lines[i]);
    if (parsed == null) continue;
    comments.push({ line: lineNumber, kind: parsed.kind, rules: parsed.rules });
    const ruleList = parsed.rules.length === 0 ? [null] : parsed.rules;
    if (parsed.kind === "next-line") {
      for (const rule of ruleList) {
        ranges.push({
          rule,
          startLine: lineNumber + 1,
          endLine: lineNumber + 1,
          commentLine: lineNumber,
          kind: parsed.kind,
        });
      }
    } else if (parsed.kind === "line") {
      for (const rule of ruleList) {
        ranges.push({
          rule,
          startLine: lineNumber,
          endLine: lineNumber,
          commentLine: lineNumber,
          kind: parsed.kind,
        });
      }
    } else if (parsed.kind === "block") {
      for (const rule of ruleList) {
        openBlocks.push(ranges.length);
        ranges.push({
          rule,
          startLine: lineNumber,
          endLine: null,
          commentLine: lineNumber,
          kind: parsed.kind,
        });
      }
    } else {
      // eslint-enable closes open blocks (all of them for a bare enable,
      // matching-rule ones otherwise).
      for (let open = openBlocks.length - 1; open >= 0; open -= 1) {
        const range = ranges[openBlocks[open]];
        if (parsed.rules.length === 0 || parsed.rules.includes(range.rule ?? "")) {
          range.endLine = lineNumber;
          openBlocks.splice(open, 1);
        }
      }
    }
  }
  return { comments, ranges };
}

/** Rewrite pragma lines only, byte-length-preserving ("esl1nt-"). */
export function defuseSuppressions(source) {
  const lines = source.split("\n");
  let changed = false;
  for (let i = 0; i < lines.length; i += 1) {
    if (parseSuppressionLine(lines[i]) != null) {
      lines[i] = lines[i].replaceAll("eslint-", "esl1nt-");
      changed = true;
    }
  }
  return { defused: lines.join("\n"), changed };
}

/** Load the committed eslint→vize rule mapping. */
export function loadRuleMap() {
  const fixture = JSON.parse(fs.readFileSync(path.join(repoRoot, RULE_MAP_FIXTURE), "utf8"));
  const mapped = new Map();
  for (const [eslintName, entry] of Object.entries(fixture.entries)) {
    if (entry.status === "mapped") mapped.set(eslintName, entry.patinaRule);
  }
  for (const [eslintName, vizeName] of Object.entries(CORE_ESLINT_TO_VIZE)) {
    mapped.set(eslintName, vizeName);
  }
  return {
    mapped,
    fixturePath: RULE_MAP_FIXTURE,
    fixtureMappedCount: mapped.size - Object.keys(CORE_ESLINT_TO_VIZE).length,
    coreSidecarCount: Object.keys(CORE_ESLINT_TO_VIZE).length,
  };
}

function rangeCovers(range, line) {
  return line >= range.startLine && (range.endLine == null || line <= range.endLine);
}

/**
 * Intersect defused-run diagnostics with the suppression ranges.
 * A diagnostic is an FP candidate only when a NAMED suppression maps to
 * exactly its vize rule id (the mapped-rules filter from the P0-13 brief:
 * real projects suppress THEIR toolchains' rules, so unfiltered
 * intersection drowns). Diagnostics on bare-suppressed lines are counted
 * for scope proof but are not candidates.
 */
export function intersectSuppressions(diagnosticsByPath, suppressionsByPath, ruleMap) {
  const candidates = [];
  let onBareLines = 0;
  for (const [filePath, rows] of diagnosticsByPath) {
    const suppressions = suppressionsByPath.get(filePath);
    if (!suppressions) continue;
    for (const row of rows) {
      let isCandidate = false;
      for (const range of suppressions.ranges) {
        if (!rangeCovers(range, row.line)) continue;
        if (range.rule == null) {
          onBareLines += 1;
          continue;
        }
        if (ruleMap.mapped.get(range.rule) === row.ruleId && !isCandidate) {
          isCandidate = true;
          candidates.push({
            path: filePath,
            line: row.line,
            column: row.column,
            endLine: row.endLine,
            endColumn: row.endColumn,
            severity: row.severity,
            vizeRule: row.ruleId,
            eslintRule: range.rule,
            commentLine: range.commentLine,
            kind: range.kind,
          });
        }
      }
    }
  }
  candidates.sort(
    (a, b) =>
      (a.path < b.path ? -1 : a.path > b.path ? 1 : 0) ||
      a.line - b.line ||
      a.column - b.column ||
      (a.vizeRule < b.vizeRule ? -1 : a.vizeRule > b.vizeRule ? 1 : 0),
  );
  return { candidates, onBareLines };
}
