import { isAbsolute, relative } from "node:path";

/**
 * Shared record shaping for the `vize.fixtureLintDivergence` comparator: the
 * two input dialects (patina JSON findings, ESLint JSON results) are normalized
 * into one record shape, and the map from `tests/_fixtures/patina-eslint-vue-rule-map.json`
 * is indexed so a baseline finding can be routed by the status of its rule.
 */

export type Severity = "error" | "warning";

export interface LintRange {
  readonly line: number;
  readonly column: number;
  readonly endLine: number;
  readonly endColumn: number;
}

export interface LintRecord extends LintRange {
  readonly file: string;
  readonly ruleId: string;
  readonly severity: Severity;
  readonly message: string;
}

export type RuleMapStatus = "mapped" | "unimplemented" | "intentional-divergence";

export interface RuleMapEntry {
  readonly status: RuleMapStatus;
  readonly patinaRule?: string;
  readonly [field: string]: unknown;
}

export interface RuleMapIndex {
  readonly byUpstream: Map<string, RuleMapEntry>;
  readonly patinaTargets: Set<string>;
}

export function isRecord(value: unknown): value is Readonly<Record<string, unknown>> {
  return value != null && typeof value === "object" && !Array.isArray(value);
}

function rangeValues(range: Readonly<Record<string, unknown>> | LintRange): unknown[] {
  return [range.line, range.column, range.endLine, range.endColumn];
}

function isPositive(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 1;
}

/** A finding, normalized to the identity the comparator classifies on. */
export function lintRecord(
  file: string,
  ruleId: string,
  severity: Severity,
  range: Readonly<Record<string, unknown>> | LintRange,
  message: unknown,
): LintRecord {
  const [line, column, endLine, endColumn] = rangeValues(range);
  if (!isPositive(line) || !isPositive(column) || !isPositive(endLine) || !isPositive(endColumn)) {
    return invalid(`finding range must be positive safe integers: ${file} ${ruleId}`);
  }
  if (endLine < line || (endLine === line && endColumn < column)) {
    invalid(`finding has an inverted source range: ${file} ${ruleId}`);
  }
  if (typeof message !== "string") return invalid(`finding message must be non-empty: ${file}`);
  const normalizedMessage = message.replace(/\s+/g, " ").trim();
  if (normalizedMessage.length === 0) invalid(`finding message must be non-empty: ${file}`);
  return { file, ruleId, severity, line, column, endLine, endColumn, message: normalizedMessage };
}

/**
 * Index the checked-in rule map by upstream rule id.
 *
 * The map is exhaustive over the pinned `eslint-plugin-vue` surface, so a
 * baseline finding carrying a rule id the map does not know about means the pin
 * and the installed plugin have drifted apart — a hard error rather than a
 * silently ignored finding.
 */
export function indexRuleMap(ruleMap: unknown): RuleMapIndex {
  if (!isRecord(ruleMap) || !isRecord(ruleMap.entries)) {
    return invalid("rule map must carry entries");
  }
  const byUpstream = new Map<string, RuleMapEntry>();
  const patinaTargets = new Set<string>();
  for (const [ruleId, entry] of Object.entries(ruleMap.entries)) {
    if (!isRecord(entry)) return invalid(`${ruleId} must have an object entry`);
    const { status } = entry;
    if (status === "mapped") {
      if (typeof entry.patinaRule !== "string" || entry.patinaRule.length === 0) {
        invalid(`${ruleId} must name the patina rule it maps to`);
      }
      patinaTargets.add(String(entry.patinaRule));
    } else if (status !== "unimplemented" && status !== "intentional-divergence") {
      invalid(`${ruleId} has unsupported status ${JSON.stringify(status)}`);
    }
    byUpstream.set(ruleId, { ...entry, status: status as RuleMapStatus });
  }
  return { byUpstream, patinaTargets };
}

/**
 * Normalize a patina finding array (the `vize lint --format json` message
 * shape, flattened to one record per message).
 */
export function collectPatinaFindings(findings: unknown, cwd: string): LintRecord[] {
  if (!Array.isArray(findings)) return invalid("patina findings must be an array");
  return findings.map((finding: unknown, index) => {
    const label = `patina findings[${index}]`;
    if (!isRecord(finding)) return invalid(`${label} must be an object`);
    const file = normalizePath(finding.file, cwd, `${label}.file`);
    const { ruleId } = finding;
    if (typeof ruleId !== "string" || ruleId.length === 0) {
      return invalid(`${label}.ruleId must be non-empty`);
    }
    return lintRecord(file, ruleId, severity(finding.severity, label), finding, finding.message);
  });
}

export interface BaselineFindings {
  readonly findings: LintRecord[];
  readonly parseErrorCount: number;
  readonly excludedNonVueCount: number;
  readonly invalidRangeCount: number;
}

/**
 * Normalize ESLint's JSON formatter output. Findings without a rule id are
 * parse/processor errors rather than rule findings; they are counted and
 * excluded so a corpus file the reference parser cannot read never masquerades
 * as a parity gap.
 */
export function collectBaselineFindings(results: unknown, cwd: string): BaselineFindings {
  if (!Array.isArray(results)) return invalid("eslint results must be an array");
  const findings: LintRecord[] = [];
  let parseErrorCount = 0;
  let excludedNonVueCount = 0;
  let invalidRangeCount = 0;
  for (const [index, result] of (results as readonly unknown[]).entries()) {
    const label = `eslint results[${index}]`;
    if (!isRecord(result) || !Array.isArray(result.messages)) {
      return invalid(`${label} must carry messages`);
    }
    const file = normalizePath(result.filePath, cwd, `${label}.filePath`);
    for (const [messageIndex, message] of (result.messages as readonly unknown[]).entries()) {
      const messageLabel = `${label}.messages[${messageIndex}]`;
      if (!isRecord(message)) return invalid(`${messageLabel} must be an object`);
      if (message.ruleId == null) {
        parseErrorCount += 1;
        continue;
      }
      const { ruleId } = message;
      if (typeof ruleId !== "string") return invalid(`${messageLabel}.ruleId must be a string`);
      if (!file.endsWith(".vue")) {
        excludedNonVueCount += 1;
        continue;
      }
      const range = {
        line: message.line,
        column: message.column,
        endLine: message.endLine ?? message.line,
        endColumn: message.endColumn ?? message.column,
      };
      if (!isValidRange(range)) {
        invalidRangeCount += 1;
        continue;
      }
      findings.push(
        lintRecord(file, ruleId, severity(message.severity, messageLabel), range, message.message),
      );
    }
  }
  return { findings, parseErrorCount, excludedNonVueCount, invalidRangeCount };
}

function isValidRange(range: Readonly<Record<string, unknown>>): boolean {
  const [line, column, endLine, endColumn] = rangeValues(range);
  if (!isPositive(line) || !isPositive(column) || !isPositive(endLine) || !isPositive(endColumn))
    return false;
  return endLine > line || (endLine === line && endColumn >= column);
}

function severity(value: unknown, label: string): Severity {
  if (value === 2 || value === "error") return "error";
  if (value === 1 || value === "warning") return "warning";
  return invalid(`${label}.severity must be an ESLint severity (1 or 2)`);
}

export function normalizePath(value: unknown, cwd: string, label: string): string {
  if (typeof value !== "string" || value.length === 0) return invalid(`${label} must be non-empty`);
  let normalized = value.replaceAll("\\", "/");
  if (isAbsolute(normalized)) normalized = relative(cwd, normalized).replaceAll("\\", "/");
  if (normalized.startsWith("./")) normalized = normalized.slice(2);
  if (
    normalized.length === 0 ||
    isAbsolute(normalized) ||
    /^[A-Za-z]:\//.test(normalized) ||
    normalized.split("/").some((segment) => segment === "" || segment === "." || segment === "..")
  ) {
    invalid(`${label} must stay inside the fixture workspace`);
  }
  return normalized;
}

/** Total order over records, byte-ordered on strings so output is stable. */
export function compareRecords(left: LintRecord, right: LintRecord): number {
  return (
    byteOrder(left.file, right.file) ||
    left.line - right.line ||
    left.column - right.column ||
    left.endLine - right.endLine ||
    left.endColumn - right.endColumn ||
    byteOrder(left.ruleId, right.ruleId) ||
    byteOrder(left.severity, right.severity) ||
    byteOrder(left.message, right.message)
  );
}

export function byteOrder(left: string, right: string): number {
  if (left === right) return 0;
  const leftCodePoints = left[Symbol.iterator]();
  const rightCodePoints = right[Symbol.iterator]();
  while (true) {
    const leftPoint = leftCodePoints.next();
    const rightPoint = rightCodePoints.next();
    if (leftPoint.done || rightPoint.done) return leftPoint.done ? -1 : 1;
    const difference =
      (leftPoint.value.codePointAt(0) ?? 0) - (rightPoint.value.codePointAt(0) ?? 0);
    if (difference !== 0) return difference;
  }
}

export function invalid(message: string): never {
  throw new Error(`Invalid lint divergence input: ${message}`);
}
