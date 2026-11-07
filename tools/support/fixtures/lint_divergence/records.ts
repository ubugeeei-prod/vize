import path from "node:path";
import {
  byteOrder,
  get,
  requiredString,
  unsigned,
  type Json,
  type JsonObject,
  type Integer,
} from "./value.ts";

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
export function recordValue(record: LintRecord): JsonObject {
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
export function baselineIntentionalReason(
  finding: LintRecord,
  source?: string | null,
): string | null {
  const name = quotedName(finding.message)?.replace(/^(?:v-bind:|:)/, "");
  if (
    finding.ruleId === "vue/require-explicit-slots" &&
    source !== undefined &&
    source !== null &&
    importedUppercaseSlotRange(finding, source)
  )
    return "eslint-plugin-vue treats an imported uppercase <Slot> component as a slot outlet case-insensitively; Patina reports only the nested lowercase <slot> outlet that Vue actually renders as a slot.";
  if (
    finding.ruleId === "vue/no-useless-mustaches" &&
    source !== undefined &&
    source !== null &&
    whitespaceOnlyMustache(finding, source)
  )
    return "Patina keeps whitespace-only string interpolations because replacing them with static whitespace can be removed by Vue's whitespace condensing and change the rendered output.";
  if (
    finding.ruleId === "vue/html-quotes" &&
    source !== undefined &&
    source !== null &&
    directiveValueHoldsDoubleQuote(finding, source)
  )
    return "Patina applies avoidEscape to directive expressions: changing this single-quoted binding to double quotes would require escaping the JavaScript string delimiters that the formatter emits.";
  if (
    finding.ruleId === "vue/attribute-hyphenation" &&
    name !== undefined &&
    /^(?:aria|data)[A-Z]/.test(name)
  )
    return "Vize keeps camelCase aria*/data* names on components: type checkers read hyphenated aria-*/data-* names as fallthrough attributes, so hyphenating them would drop the prop type check.";
  if (
    finding.ruleId === "vue/attribute-hyphenation" &&
    source !== undefined &&
    source !== null &&
    authoredForeignNativeOpening(finding, source) !== null
  )
    return "Patina recognizes the authored SVG or MathML tag as native even when it is a fragment root; eslint-plugin-vue loses the foreign-element namespace there and applies this component-only attribute rule.";
  if (
    finding.ruleId === "vue/no-v-text-v-html-on-component" &&
    source !== undefined &&
    source !== null &&
    authoredForeignNativeOpening(finding, source) !== null
  )
    return "Patina recognizes the authored SVG or MathML tag as native even when it is a fragment root; eslint-plugin-vue loses the foreign-element namespace there and treats v-text or v-html as component content.";
  if (
    finding.ruleId === "vue/valid-v-for" &&
    source !== undefined &&
    source !== null &&
    authoredForeignNativeOpening(finding, source) !== null
  )
    return "Patina recognizes the authored SVG or MathML tag as native even when it is a fragment root; eslint-plugin-vue loses the foreign-element namespace and incorrectly applies its custom-component v-for exemption.";
  if (
    finding.ruleId === "vue/no-v-text-v-html-on-component" &&
    source !== undefined &&
    source !== null &&
    dynamicComponentIsNativeOnly(finding, source)
  )
    return "Patina proves that this dynamic <component :is> expression can only select native elements, so v-text/v-html cannot overwrite a Vue component render tree.";
  return null;
}

function directiveValueHoldsDoubleQuote(finding: LintRecord, source: string): boolean {
  if (finding.line !== finding.endLine) return false;
  const line = source.split("\n")[Number(finding.line) - 1];
  if (line === undefined) return false;
  const start = Number(finding.column) - 1,
    end = Number(finding.endColumn) - 1,
    value = line.slice(start, end);
  if (!value.startsWith("'") || !value.endsWith("'") || !value.slice(1, -1).includes('"'))
    return false;
  return /(?:^|\s)(?:v-[^\s=]+|[:@#][^\s=]+)\s*=\s*$/u.test(line.slice(0, start));
}

function importedUppercaseSlotRange(finding: LintRecord, source: string): boolean {
  const lines = source.split("\n");
  const startLine = lines[Number(finding.line) - 1];
  if (startLine === undefined || !startLine.slice(Number(finding.column) - 1).startsWith("<Slot"))
    return false;
  return lines
    .slice(Number(finding.line) - 1, Number(finding.endLine))
    .join("\n")
    .includes("<slot");
}

function whitespaceOnlyMustache(finding: LintRecord, source: string): boolean {
  if (finding.line !== finding.endLine) return false;
  const line = source.split("\n")[Number(finding.line) - 1];
  if (line === undefined) return false;
  const authored = line.slice(Number(finding.column) - 1, Number(finding.endColumn) - 1);
  const match = /^\{\{\s*(['"`])([\s\S]*?)\1\s*\}\}$/u.exec(authored);
  return match !== null && match[2].length > 0 && /^(?:\s|\\[tnrfv ])+$/u.test(match[2]);
}

function dynamicComponentIsNativeOnly(finding: LintRecord, source: string): boolean {
  const lines = source.split("\n");
  if (finding.line > lines.length) return false;
  let offset = 0;
  for (let line = 1; line < finding.line; line++) offset += (lines[line - 1]?.length ?? 0) + 1;
  offset += Number(finding.column) - 1;
  const start = source.lastIndexOf("<component", offset);
  if (start < 0) return false;
  const end = source.indexOf(">", offset);
  if (end < 0) return false;
  const opening = source.slice(start, end);
  const binding = /(?:^|\s)(?::is|v-bind:is)\s*=\s*(["'])(.*?)\1/s.exec(opening)?.[2];
  if (binding === undefined) return false;
  const values = [...binding.matchAll(/(["'])(.*?)\1/g)].map((match) => match[2]);
  if (values.length === 0) return false;
  const native = new Set([
    "a",
    "article",
    "aside",
    "button",
    "div",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "img",
    "input",
    "label",
    "li",
    "main",
    "nav",
    "ol",
    "option",
    "p",
    "section",
    "select",
    "small",
    "span",
    "strong",
    "svg",
    "table",
    "tbody",
    "td",
    "textarea",
    "th",
    "thead",
    "tr",
    "ul",
  ]);
  return values.every((value) => native.has(value));
}
export function patinaIntentionalReason(finding: LintRecord, source: string): string | null {
  if (
    finding.ruleId === "script/require-default-prop" &&
    findingInsideReferencedPropsObject(finding, source)
  )
    return "Patina resolves a local runtime props object referenced by the component's props option; eslint-plugin-vue does not follow that identifier, so this is additional sound default-prop coverage.";
  if (
    finding.ruleId === "script/custom-event-name-casing" &&
    findingPrecedesDefineEmits(finding, source)
  )
    return "Patina resolves the defineEmits binding before visiting calls; eslint-plugin-vue misses calls authored before the macro declaration because its visitor records the binding later.";
  if (
    finding.ruleId === "script/no-deprecated-events-api" &&
    !isDirectThisEventsApiCall(finding, source)
  )
    return "Patina follows proven Vue-instance aliases and wrapped or optional callees; eslint-plugin-vue only recognizes an unwrapped, non-optional call directly on this.";
  if (
    finding.ruleId === "vue/no-static-inline-styles" &&
    findingInsideTypeCastStyleBinding(finding, source)
  )
    return "Patina unwraps TypeScript casts around style bindings and preserves the upstream rule's static-property checks; eslint-plugin-vue skips the entire binding when its root is a TS cast.";
  if (
    finding.ruleId === "script/no-ref-as-operand" &&
    findingPrecedesLocalRefDeclaration(finding, source)
  )
    return "Patina resolves the lexical ref binding before visiting its references, so it catches an operand authored before the local ref declaration; eslint-plugin-vue misses that earlier reference because its visitor has not recorded the declaration yet.";
  if (
    finding.ruleId === "vue/require-v-for-key" &&
    authoredForeignNativeOpening(finding, source)?.includes("v-for")
  )
    return "Patina recognizes this fragment-root SVG or MathML tag as native and requires its v-for key; eslint-plugin-vue loses the foreign-element namespace and treats the tag as a custom component that is exempt from this rule.";
  if (finding.ruleId !== "script/define-props-destructuring") return null;
  const lines = source.split("\n");
  if (finding.line > lines.length) return null;
  let offset = 0;
  for (let line = 1; line < finding.line; line++) offset += (lines[line - 1]?.length ?? 0) + 1;
  offset += Number(finding.column) - 1;
  if (!source.startsWith("defineProps", offset)) return null;
  return "Patina's default deliberately requires a direct bare defineProps call to bind an explicit destructuring pattern; eslint-plugin-vue does not report that stronger policy.";
}

function findingOffset(finding: LintRecord, source: string): number | null {
  const lines = source.split("\n");
  if (finding.line > lines.length) return null;
  let offset = 0;
  for (let line = 1; line < finding.line; line++) offset += (lines[line - 1]?.length ?? 0) + 1;
  return offset + Number(finding.column) - 1;
}

function findingEndOffset(finding: LintRecord, source: string): number | null {
  const lines = source.split("\n");
  if (finding.endLine > lines.length) return null;
  let offset = 0;
  for (let line = 1; line < finding.endLine; line++) offset += (lines[line - 1]?.length ?? 0) + 1;
  return offset + Number(finding.endColumn) - 1;
}

function findingPrecedesDefineEmits(finding: LintRecord, source: string): boolean {
  const offset = findingOffset(finding, source);
  return offset !== null && source.indexOf("defineEmits", offset) > offset;
}

function isDirectThisEventsApiCall(finding: LintRecord, source: string): boolean {
  const start = findingOffset(finding, source),
    end = findingEndOffset(finding, source);
  if (start === null || end === null || end < start) return false;
  const authored = source.slice(Math.max(0, start - 32), Math.min(source.length, end + 8));
  return /this\s*(?:\.\s*\$(?:on|off|once)|\[\s*['"]\$(?:on|off|once)['"]\s*\])\s*\(/u.test(
    authored,
  );
}

function findingInsideTypeCastStyleBinding(finding: LintRecord, source: string): boolean {
  const offset = findingOffset(finding, source);
  if (offset === null) return false;
  const start = Math.max(
    source.lastIndexOf(":style", offset),
    source.lastIndexOf("v-bind:style", offset),
  );
  if (start < 0 || offset - start > 20_000) return false;
  const end = source.indexOf(">", offset);
  if (end < 0) return false;
  return /\}\s+as\s+[A-Za-z_$][\w$]*/u.test(source.slice(offset, end));
}

const FOREIGN_NATIVE_TAGS = new Set([
  "animate",
  "circle",
  "clipPath",
  "defs",
  "ellipse",
  "feBlend",
  "feColorMatrix",
  "feComponentTransfer",
  "feComposite",
  "feConvolveMatrix",
  "feDiffuseLighting",
  "feDisplacementMap",
  "feDistantLight",
  "feDropShadow",
  "feFlood",
  "feFuncA",
  "feFuncB",
  "feFuncG",
  "feFuncR",
  "feGaussianBlur",
  "feImage",
  "feMerge",
  "feMergeNode",
  "feMorphology",
  "feOffset",
  "fePointLight",
  "feSpecularLighting",
  "feSpotLight",
  "feTile",
  "feTurbulence",
  "filter",
  "foreignObject",
  "g",
  "image",
  "line",
  "linearGradient",
  "marker",
  "mask",
  "math",
  "mfrac",
  "mi",
  "mn",
  "mo",
  "mrow",
  "msqrt",
  "mstyle",
  "mtext",
  "path",
  "pattern",
  "polygon",
  "polyline",
  "radialGradient",
  "rect",
  "stop",
  "svg",
  "symbol",
  "text",
  "textPath",
  "tspan",
  "use",
]);

function authoredForeignNativeOpening(finding: LintRecord, source: string): string | null {
  const offset = findingOffset(finding, source);
  if (offset === null) return null;
  const start = source.lastIndexOf("<", offset);
  if (start < 0) return null;
  const end = source.indexOf(">", start + 1);
  if (end < offset || end < 0) return null;
  const opening = source.slice(start, end + 1);
  const tag = /^<\s*([A-Za-z][\w:.-]*)/u.exec(opening)?.[1];
  return tag !== undefined && FOREIGN_NATIVE_TAGS.has(tag) ? opening : null;
}

function findingPrecedesLocalRefDeclaration(finding: LintRecord, source: string): boolean {
  const offset = findingOffset(finding, source);
  if (offset === null) return false;
  const name = quotedName(finding.message) ?? /^[A-Za-z_$][\w$]*/u.exec(source.slice(offset))?.[0];
  if (name === undefined || !source.startsWith(name, offset)) return false;
  const scriptEnd = source.indexOf("</script>", offset);
  const tail = source.slice(offset + name.length, scriptEnd < 0 ? source.length : scriptEnd);
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  return new RegExp(
    `\\b(?:const|let|var)\\s+${escaped}\\s*=\\s*(?:computed|customRef|ref|shallowRef|toRef|toRefs)\\s*\\(`,
    "u",
  ).test(tail);
}

function findingInsideReferencedPropsObject(finding: LintRecord, source: string): boolean {
  const lines = source.split("\n");
  if (finding.line > lines.length) return false;
  let offset = 0;
  for (let line = 1; line < finding.line; line++) offset += (lines[line - 1]?.length ?? 0) + 1;
  offset += Number(finding.column) - 1;
  const candidates = [
    ...source.slice(0, offset).matchAll(/\bconst\s+([A-Za-z_$][\w$]*)\s*=\s*\{/gu),
  ].reverse();
  return candidates.some((match) => {
    if (match.index === undefined || offset - match.index > 20_000) return false;
    const name = match[1];
    return new RegExp(`\\bprops\\s*:\\s*${name}\\b`, "u").test(source.slice(offset));
  });
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
      if (
        positive.ruleId === "script/require-explicit-slots" &&
        (negative.line < positive.line ||
          (negative.line === positive.line && negative.column <= positive.column)) &&
        (negative.endLine > positive.endLine ||
          (negative.endLine === positive.endLine && negative.endColumn >= positive.endColumn))
      ) {
        const slot =
          /^\[?vize:[^\]]+\]?\s*Slot '([^']+)'/u.exec(positive.message)?.[1] ??
          /^Slot '([^']+)'/u.exec(positive.message)?.[1];
        if (slot !== undefined) {
          match = {
            index,
            subject: `slot:${slot}`,
            reason:
              "patina reports the actual lowercase <slot> outlet while eslint-plugin-vue can anchor the enclosing imported <Slot> component case-insensitively.",
          };
          break;
        }
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
