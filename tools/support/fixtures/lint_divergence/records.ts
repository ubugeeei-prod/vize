import path from "node:path";
import { byteOrder, get, requiredString, unsigned, type Json, type Integer } from "./value.ts";

export interface LintRecord {
  file: string;
  ruleId: string;
  upstreamRuleId?: string;
  severity: string;
  line: Integer;
  column: Integer;
  endLine: Integer;
  endColumn: Integer;
  message: string;
}
export function recordValue(record: LintRecord): Json {
  return { ...record };
}
export function normalizePath(value: string, cwd: string, label: string): string {
  let normalized = value.replaceAll("\\", "/");
  if (path.isAbsolute(normalized)) {
    // Path::strip_prefix compares lexical components; path.relative would erase
    // authored '..' segments and accept paths the Rust contract refuses.
    const components = (value: string) =>
      value
        .replaceAll("\\", "/")
        .split("/")
        .filter((part) => part !== "" && part !== ".");
    const input = components(normalized),
      root = components(cwd);
    if (root.every((part, index) => input[index] === part))
      normalized = input.slice(root.length).join("/");
  }
  if (normalized.startsWith("./")) normalized = normalized.slice(2);
  if (
    !normalized ||
    normalized.startsWith("/") ||
    normalized.split("/").some((part) => !part || part === "." || part === "..")
  )
    throw new Error(`${label} must stay inside the fixture workspace`);
  return normalized;
}
function severity(value: Json, label: string): string {
  if (value === 2 || value === "error") return "error";
  if (value === 1 || value === "warning") return "warning";
  throw new Error(`${label}.severity must be an ESLint severity (1 or 2)`);
}
function validateRange(record: LintRecord): void {
  if (
    [record.line, record.column, record.endLine, record.endColumn].some((n) => n === 0 || n === 0n)
  )
    throw new Error(
      `finding range must be positive safe integers: ${record.file} ${record.ruleId}`,
    );
  if (
    record.endLine < record.line ||
    (record.endLine === record.line && record.endColumn < record.column)
  )
    throw new Error(`finding has an inverted source range: ${record.file} ${record.ruleId}`);
}
const normalizeMessage = (message: string) =>
  message
    .split(/\p{White_Space}+/u)
    .filter(Boolean)
    .join(" ");
function requiredInteger(value: Json, key: string, label: string): Integer {
  const result = unsigned(get(value, key));
  if (result === undefined || result === 0 || result === 0n)
    throw new Error(`${label}.${key} must be a positive integer`);
  return result;
}
export function collectPatinaFindings(findings: Json[], cwd: string): LintRecord[] {
  return findings.map((finding, index) => {
    const label = `patina findings[${index}]`;
    const file = normalizePath(requiredString(finding, "file", label), cwd, "file"),
      ruleId = requiredString(finding, "ruleId", label),
      level = severity(get(finding, "severity"), label),
      line = requiredInteger(finding, "line", label),
      column = requiredInteger(finding, "column", label);
    const result: LintRecord = {
      file,
      ruleId,
      severity: level,
      line,
      column,
      endLine: unsigned(get(finding, "endLine")) ?? line,
      endColumn: unsigned(get(finding, "endColumn")) ?? column,
      message: normalizeMessage(requiredString(finding, "message", label)),
    };
    validateRange(result);
    return result;
  });
}
export function collectBaselineFindings(results: Json[], cwd: string) {
  const findings: LintRecord[] = [];
  let parseErrorCount = 0,
    excludedNonVueCount = 0,
    invalidRangeCount = 0;
  for (const [index, result] of results.entries()) {
    const label = `eslint results[${index}]`,
      file = normalizePath(requiredString(result, "filePath", label), cwd, "filePath");
    const messages = get(result, "messages");
    if (!Array.isArray(messages)) throw new Error(`${label} must carry messages`);
    for (const message of messages) {
      if (get(message, "ruleId") === null) {
        parseErrorCount++;
        continue;
      }
      if (!file.endsWith(".vue")) {
        excludedNonVueCount++;
        continue;
      }
      const ruleId = requiredString(message, "ruleId", label),
        level = severity(get(message, "severity"), label);
      const line = unsigned(get(message, "line")),
        column = unsigned(get(message, "column"));
      const endLine = get(message, "endLine") === null ? line : unsigned(get(message, "endLine"));
      const endColumn =
        get(message, "endColumn") === null ? column : unsigned(get(message, "endColumn"));
      if (
        line === undefined ||
        column === undefined ||
        endLine === undefined ||
        endColumn === undefined ||
        !line ||
        !column ||
        !endLine ||
        !endColumn ||
        endLine < line ||
        (endLine === line && endColumn < column)
      ) {
        invalidRangeCount++;
        continue;
      }
      findings.push({
        file,
        ruleId,
        severity: level,
        line,
        column,
        endLine,
        endColumn,
        message: normalizeMessage(requiredString(message, "message", label)),
      });
    }
  }
  return { findings, parseErrorCount, excludedNonVueCount, invalidRangeCount };
}
export function compareRecords(a: LintRecord, b: LintRecord): number {
  const numeric = (left: Integer, right: Integer) => (left < right ? -1 : left > right ? 1 : 0);
  return (
    byteOrder(a.file, b.file) ||
    numeric(a.line, b.line) ||
    numeric(a.column, b.column) ||
    numeric(a.endLine, b.endLine) ||
    numeric(a.endColumn, b.endColumn) ||
    byteOrder(a.ruleId, b.ruleId) ||
    byteOrder(a.severity, b.severity) ||
    byteOrder(a.message, b.message)
  );
}
export function groupByIdentity(records: LintRecord[]): Map<string, LintRecord[]> {
  const groups = new Map<string, LintRecord[]>();
  for (const record of records) {
    const key = [
      record.file,
      record.ruleId,
      record.severity,
      record.line,
      record.column,
      record.endLine,
      record.endColumn,
    ].join("\0");
    const group = groups.get(key) ?? [];
    group.push(record);
    groups.set(key, group);
  }
  for (const group of groups.values()) group.sort(compareRecords);
  return groups;
}
const quotedName = (message: string): string | undefined => /'([^']*)'/.exec(message)?.[1];
export function baselineIntentionalReason(finding: LintRecord): string | null {
  const name = quotedName(finding.message)?.replace(/^(?:v-bind:|:)/, "");
  return finding.ruleId === "vue/attribute-hyphenation" &&
    name !== undefined &&
    /^(?:aria|data)[A-Z]/.test(name)
    ? "Vize keeps camelCase aria*/data* names on components: type checkers read hyphenated aria-*/data-* names as fallthrough attributes, so hyphenating them would drop the prop type check."
    : null;
}
export function outsideBaselineSurface(finding: LintRecord, source: string): boolean {
  const tagStart = source.indexOf("<template"),
    tagEnd = source.indexOf(">", tagStart);
  if (tagStart >= 0 && tagEnd >= 0) {
    const tag = source.slice(tagStart, tagEnd),
      bodyStart = tagEnd + 1,
      bodyEnd = source.indexOf("</template>", bodyStart);
    if (
      [`lang="pug"`, "lang='pug'", "lang=pug"].some((s) => tag.includes(s)) &&
      !tag.includes("src=") &&
      bodyEnd >= 0
    ) {
      const line = (offset: number) => source.slice(0, offset).split("\n").length;
      const start = line(bodyStart) + (/^[\r\n]/.test(source.slice(bodyStart)) ? 1 : 0);
      if (finding.line >= start && finding.line <= line(bodyEnd)) return true;
    }
  }
  if (finding.ruleId !== "vue/no-unused-components") return false;
  const name = quotedName(finding.message);
  if (name === undefined) return false;
  for (const rest of source.split("<script").slice(1)) {
    const end = rest.indexOf(">"),
      close = rest.indexOf("</script>", end + 1);
    if (
      end < 0 ||
      close < 0 ||
      !rest
        .slice(0, end)
        .split(/\p{White_Space}+/u)
        .some((attribute) => attribute === "setup" || attribute.startsWith("setup="))
    )
      continue;
    for (const sourceLine of rest.slice(end + 1, close).split(/\r?\n/)) {
      const line = sourceLine.trimStart();
      if (!line.startsWith("import ")) continue;
      let index = line.indexOf(name);
      while (index >= 0) {
        if (
          !/[A-Za-z0-9_$]/.test(line[index - 1] ?? "") &&
          !/[A-Za-z0-9_$]/.test(line[index + name.length] ?? "")
        )
          return true;
        if (!name) break;
        index = line.indexOf(name, index + name.length);
      }
    }
  }
  return false;
}
function unusedComponentName(message: string): string | undefined {
  const unprefixed = message.replace(/^\[[^\]]*\] /, "");
  return (
    /^Component '(.*?)' is registered but never used/.exec(unprefixed)?.[1] ??
    /^The "(.*?)" component has been registered but not used\./.exec(unprefixed)?.[1]
  );
}
export function pairRuleLocationDivergences(
  positives: LintRecord[],
  negatives: LintRecord[],
): Json[] {
  const pairs: Json[] = [];
  let i = 0;
  while (i < positives.length) {
    const positive = positives[i];
    let match: { index: number; subject: string; reason: string } | undefined;
    for (const [index, negative] of negatives.entries()) {
      if (
        positive.file !== negative.file ||
        positive.ruleId !== negative.ruleId ||
        positive.severity !== negative.severity
      )
        continue;
      if (positive.ruleId === "vue/no-unused-components") {
        const component = unusedComponentName(positive.message);
        if (component === undefined) break;
        if (component === unusedComponentName(negative.message)) {
          match = {
            index,
            subject: `component:${component}`,
            reason:
              "patina reports the SFC/script anchor while eslint-plugin-vue reports the component registration property.",
          };
          break;
        }
      }
      if (
        positive.ruleId === "vue/require-v-for-key" &&
        (negative.line < positive.line ||
          (negative.line === positive.line && negative.column <= positive.column)) &&
        (negative.endLine > positive.endLine ||
          (negative.endLine === positive.endLine && negative.endColumn >= positive.endColumn))
      ) {
        match = {
          index,
          subject: "missing-v-for-key",
          reason:
            "patina reports the v-for directive range while eslint-plugin-vue reports the owning element range.",
        };
        break;
      }
    }
    if (!match) {
      i++;
      continue;
    }
    const negative = negatives.splice(match.index, 1)[0];
    positives.splice(i, 1);
    const side = (record: LintRecord): Json => ({
      line: record.line,
      column: record.column,
      endLine: record.endLine,
      endColumn: record.endColumn,
      message: record.message,
    });
    pairs.push({
      file: positive.file,
      ruleId: positive.ruleId,
      upstreamRuleId: negative.upstreamRuleId ?? null,
      severity: positive.severity,
      subject: match.subject,
      reason: match.reason,
      patina: side(positive),
      baseline: side(negative),
    });
  }
  return pairs;
}
