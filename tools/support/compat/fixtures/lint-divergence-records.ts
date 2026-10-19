import { isAbsolute, relative } from "node:path";

/**
 * Shared record shaping for the `vize.fixtureLintDivergence` comparator: the
 * two input dialects (patina JSON findings, ESLint JSON results) are normalized
 * into one record shape, and the map from `tests/_fixtures/patina-eslint-vue-rule-map.json`
 * is indexed so a baseline finding can be routed by the status of its rule.
 */

/** A finding, normalized to the identity the comparator classifies on. */
export function lintRecord(file, ruleId, severity, range, message) {
  const values = [range.line, range.column, range.endLine, range.endColumn];
  if (values.some((value) => !Number.isSafeInteger(value) || value < 1)) {
    invalid(`finding range must be positive safe integers: ${file} ${ruleId}`);
  }
  if (values[2] < values[0] || (values[2] === values[0] && values[3] < values[1])) {
    invalid(`finding has an inverted source range: ${file} ${ruleId}`);
  }
  const normalizedMessage = message.replace(/\s+/g, " ").trim();
  if (normalizedMessage.length === 0) invalid(`finding message must be non-empty: ${file}`);
  return {
    file,
    ruleId,
    severity,
    line: values[0],
    column: values[1],
    endLine: values[2],
    endColumn: values[3],
    message: normalizedMessage,
  };
}

/**
 * Index the checked-in rule map by upstream rule id.
 *
 * The map is exhaustive over the pinned `eslint-plugin-vue` surface, so a
 * baseline finding carrying a rule id the map does not know about means the pin
 * and the installed plugin have drifted apart — a hard error rather than a
 * silently ignored finding.
 */
export function indexRuleMap(ruleMap) {
  if (ruleMap == null || typeof ruleMap !== "object" || typeof ruleMap.entries !== "object") {
    invalid("rule map must carry entries");
  }
  const byUpstream = new Map();
  const patinaTargets = new Set();
  for (const [ruleId, entry] of Object.entries(ruleMap.entries)) {
    if (entry == null || typeof entry !== "object") invalid(`${ruleId} must have an object entry`);
    if (entry.status === "mapped") {
      if (typeof entry.patinaRule !== "string" || entry.patinaRule.length === 0) {
        invalid(`${ruleId} must name the patina rule it maps to`);
      }
      patinaTargets.add(entry.patinaRule);
    } else if (entry.status !== "unimplemented" && entry.status !== "intentional-divergence") {
      invalid(`${ruleId} has unsupported status ${JSON.stringify(entry.status)}`);
    }
    byUpstream.set(ruleId, entry);
  }
  return { byUpstream, patinaTargets };
}

/**
 * Normalize a patina finding array (the `vize lint --format json` message
 * shape, flattened to one record per message).
 */
export function collectPatinaFindings(findings, cwd) {
  if (!Array.isArray(findings)) invalid("patina findings must be an array");
  return findings.map((finding, index) => {
    const label = `patina findings[${index}]`;
    if (finding == null || typeof finding !== "object") invalid(`${label} must be an object`);
    const file = normalizePath(finding.file, cwd, `${label}.file`);
    if (typeof finding.ruleId !== "string" || finding.ruleId.length === 0) {
      invalid(`${label}.ruleId must be non-empty`);
    }
    return lintRecord(
      file,
      finding.ruleId,
      severity(finding.severity, label),
      finding,
      finding.message,
    );
  });
}

/**
 * Normalize ESLint's JSON formatter output. Findings without a rule id are
 * parse/processor errors rather than rule findings; they are counted and
 * excluded so a corpus file the reference parser cannot read never masquerades
 * as a parity gap.
 */
export function collectBaselineFindings(results, cwd) {
  if (!Array.isArray(results)) invalid("eslint results must be an array");
  const findings = [];
  let parseErrorCount = 0;
  let excludedNonVueCount = 0;
  let invalidRangeCount = 0;
  for (const [index, result] of results.entries()) {
    const label = `eslint results[${index}]`;
    if (result == null || typeof result !== "object" || !Array.isArray(result.messages)) {
      invalid(`${label} must carry messages`);
    }
    const file = normalizePath(result.filePath, cwd, `${label}.filePath`);
    for (const [messageIndex, message] of result.messages.entries()) {
      const messageLabel = `${label}.messages[${messageIndex}]`;
      if (message == null || typeof message !== "object")
        invalid(`${messageLabel} must be an object`);
      if (message.ruleId == null) {
        parseErrorCount += 1;
        continue;
      }
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
        lintRecord(
          file,
          message.ruleId,
          severity(message.severity, messageLabel),
          range,
          message.message,
        ),
      );
    }
  }
  return { findings, parseErrorCount, excludedNonVueCount, invalidRangeCount };
}

function isValidRange(range) {
  const values = [range.line, range.column, range.endLine, range.endColumn];
  if (values.some((value) => !Number.isSafeInteger(value) || value < 1)) return false;
  return (
    range.endLine > range.line || (range.endLine === range.line && range.endColumn >= range.column)
  );
}

function severity(value, label) {
  if (value === 2 || value === "error") return "error";
  if (value === 1 || value === "warning") return "warning";
  return invalid(`${label}.severity must be an ESLint severity (1 or 2)`);
}

export function normalizePath(value, cwd, label) {
  if (typeof value !== "string" || value.length === 0) invalid(`${label} must be non-empty`);
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
export function compareRecords(left, right) {
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

export function byteOrder(left, right) {
  if (left === right) return 0;
  const leftCodePoints = left[Symbol.iterator]();
  const rightCodePoints = right[Symbol.iterator]();
  while (true) {
    const leftPoint = leftCodePoints.next();
    const rightPoint = rightCodePoints.next();
    if (leftPoint.done || rightPoint.done) return leftPoint.done ? -1 : 1;
    const difference = leftPoint.value.codePointAt(0) - rightPoint.value.codePointAt(0);
    if (difference !== 0) return difference;
  }
}

export function invalid(message) {
  throw new Error(`Invalid lint divergence input: ${message}`);
}
