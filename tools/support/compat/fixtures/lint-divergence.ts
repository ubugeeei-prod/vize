import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { isAbsolute, join } from "node:path";

import {
  byteOrder,
  isRecord,
  collectBaselineFindings,
  collectPatinaFindings,
  compareRecords,
  indexRuleMap,
  invalid,
  normalizePath,
  type LintRange,
  type LintRecord,
  type Severity,
} from "./lint-divergence-records.ts";

type Compared = LintRecord & { readonly upstreamRuleId?: string };

interface SharedPair extends LintRange {
  readonly file: string;
  readonly ruleId: string;
  readonly upstreamRuleId: string | undefined;
  readonly severity: Severity;
  readonly patinaMessage: string;
  readonly baselineMessage: string;
}

interface DocumentedSide extends LintRange {
  readonly severity: Severity;
}

interface DocumentedDivergence {
  readonly file: string;
  readonly ruleId: string;
  readonly patina: DocumentedSide;
  readonly baseline: DocumentedSide;
  readonly issue: number;
  readonly reason: string;
}

interface LocationSide extends LintRange {
  readonly message: string;
}

interface RuleLocationDivergence {
  readonly file: string;
  readonly ruleId: string;
  readonly upstreamRuleId: string | undefined;
  readonly severity: Severity;
  readonly subject: string;
  readonly reason: string;
  readonly patina: LocationSide;
  readonly baseline: LocationSide;
}

interface RuleLocationMatch {
  readonly negativeIndex: number;
  readonly reason: string;
  readonly subject: string;
}

interface RuleLocationRule {
  readonly reason: string;
  subject(positive: Compared, negative: Compared): string | null;
}

export interface CompareLintInput {
  readonly projectId: unknown;
  readonly cwd: unknown;
  readonly ruleMap: unknown;
  readonly patinaFindings: unknown;
  readonly eslintResults: unknown;
  readonly documentedDivergences?: unknown;
}

function upstreamOf(ruleMap: unknown): { package: unknown; version: unknown } {
  const upstream = isRecord(ruleMap) ? ruleMap.upstream : undefined;
  if (!isRecord(upstream)) return invalid("rule map must name its upstream package");
  return { package: upstream.package, version: upstream.version };
}

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
}: CompareLintInput) {
  if (typeof projectId !== "string" || projectId.length === 0)
    return invalid("project id is required");
  if (typeof cwd !== "string" || !isAbsolute(cwd)) return invalid("cwd must be absolute");
  const { byUpstream, patinaTargets } = indexRuleMap(ruleMap);
  const patina = collectPatinaFindings(patinaFindings, cwd);
  const baselineInput = collectBaselineFindings(eslintResults, cwd);

  const unimplemented: (LintRecord & { issue: unknown })[] = [];
  const intentionalDivergences: (LintRecord & { reason: unknown })[] = [];
  const comparableBaseline: Compared[] = [];
  for (const finding of baselineInput.findings) {
    const entry = byUpstream.get(finding.ruleId);
    if (entry == null) {
      return invalid(`baseline rule ${finding.ruleId} is absent from the pinned rule map`);
    }
    if (entry.status === "unimplemented") {
      unimplemented.push({ ...finding, issue: entry.issue });
      continue;
    }
    if (entry.status === "intentional-divergence") {
      intentionalDivergences.push({ ...finding, reason: entry.reason });
      continue;
    }
    const intentional = baselineIntentionalReason(finding, cwd);
    if (intentional !== null) {
      intentionalDivergences.push({ ...finding, reason: intentional });
      continue;
    }
    comparableBaseline.push({
      ...finding,
      ruleId: String(entry.patinaRule),
      upstreamRuleId: finding.ruleId,
    });
  }

  const patinaOnlyRuleFindings: LintRecord[] = [];
  const comparablePatina: Compared[] = [];
  for (const finding of patina) {
    (patinaTargets.has(finding.ruleId) ? comparablePatina : patinaOnlyRuleFindings).push(finding);
  }

  const patinaGroups = groupByIdentity(comparablePatina);
  const baselineGroups = groupByIdentity(comparableBaseline);
  const identities = [...new Set([...patinaGroups.keys(), ...baselineGroups.keys()])].sort(
    byteOrder,
  );
  const shared: SharedPair[] = [];
  const messageDifferences: SharedPair[] = [];
  const falsePositives: Compared[] = [];
  const falseNegatives: Compared[] = [];

  for (const identity of identities) {
    const candidates = patinaGroups.get(identity) ?? [];
    const expected = baselineGroups.get(identity) ?? [];
    const commonCount = Math.min(candidates.length, expected.length);
    for (let index = 0; index < commonCount; index += 1) {
      const candidate = candidates[index];
      const baseline = expected[index];
      if (candidate === undefined || baseline === undefined) continue;
      const pair: SharedPair = {
        file: candidate.file,
        ruleId: candidate.ruleId,
        upstreamRuleId: baseline.upstreamRuleId,
        severity: candidate.severity,
        line: candidate.line,
        column: candidate.column,
        endLine: candidate.endLine,
        endColumn: candidate.endColumn,
        patinaMessage: candidate.message,
        baselineMessage: baseline.message,
      };
      shared.push(pair);
      if (pair.patinaMessage !== pair.baselineMessage) messageDifferences.push(pair);
    }
    falsePositives.push(...candidates.slice(commonCount));
    falseNegatives.push(...expected.slice(commonCount));
  }

  for (let index = falsePositives.length - 1; index >= 0; index -= 1) {
    const finding = falsePositives[index];
    if (finding === undefined) continue;
    const reason = patinaIntentionalReason(finding, cwd);
    if (reason === null) continue;
    falsePositives.splice(index, 1);
    intentionalDivergences.push({ ...finding, reason });
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
    upstream: upstreamOf(ruleMap),
    summary,
    ...classified,
    sha256: createHash("sha256")
      .update(JSON.stringify({ summary, ...classified }))
      .digest("hex"),
  };
}

function patinaIntentionalReason(finding: LintRecord, cwd: string): string | null {
  let source: string;
  try {
    source = readFileSync(join(cwd, finding.file), "utf8");
  } catch {
    return null;
  }
  if (finding.ruleId === "script/require-default-prop") {
    const lines = source.split("\n");
    if (finding.line > lines.length) return null;
    let offset = 0;
    for (let line = 1; line < finding.line; line += 1) offset += (lines[line - 1]?.length ?? 0) + 1;
    offset += finding.column - 1;
    const candidates = [
      ...source.slice(0, offset).matchAll(/\bconst\s+([A-Za-z_$][\w$]*)\s*=\s*\{/gu),
    ].reverse();
    if (
      candidates.some(
        (match) =>
          match.index !== undefined &&
          offset - match.index <= 20_000 &&
          new RegExp(`\\bprops\\s*:\\s*${match[1]}\\b`, "u").test(source.slice(offset)),
      )
    )
      return "Patina resolves a local runtime props object referenced by the component's props option; eslint-plugin-vue does not follow that identifier, so this is additional sound default-prop coverage.";
    return null;
  }
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
  for (let line = 1; line < finding.line; line += 1) {
    offset += (lines[line - 1]?.length ?? 0) + 1;
  }
  offset += finding.column - 1;
  if (!source.startsWith("defineProps", offset)) return null;
  return "Patina's default deliberately requires a direct bare defineProps call to bind an explicit destructuring pattern; eslint-plugin-vue does not report that stronger policy.";
}

function findingOffset(finding: LintRecord, source: string): number | null {
  const lines = source.split("\n");
  if (finding.line > lines.length) return null;
  let offset = 0;
  for (let line = 1; line < finding.line; line += 1) offset += (lines[line - 1]?.length ?? 0) + 1;
  return offset + finding.column - 1;
}

function findingEndOffset(finding: LintRecord, source: string): number | null {
  const lines = source.split("\n");
  if (finding.endLine > lines.length) return null;
  let offset = 0;
  for (let line = 1; line < finding.endLine; line += 1)
    offset += (lines[line - 1]?.length ?? 0) + 1;
  return offset + finding.endColumn - 1;
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
  const start = source.lastIndexOf("<", offset),
    end = source.indexOf(">", start + 1);
  if (start < 0 || end < offset) return null;
  const opening = source.slice(start, end + 1),
    tag = /^<\s*([A-Za-z][\w:.-]*)/u.exec(opening)?.[1];
  return tag !== undefined && FOREIGN_NATIVE_TAGS.has(tag) ? opening : null;
}

function findingPrecedesLocalRefDeclaration(finding: LintRecord, source: string): boolean {
  const offset = findingOffset(finding, source);
  if (offset === null) return false;
  const name =
    /'([^']*)'/.exec(finding.message)?.[1] ?? /^[A-Za-z_$][\w$]*/u.exec(source.slice(offset))?.[0];
  if (name === undefined || !source.startsWith(name, offset)) return false;
  const scriptEnd = source.indexOf("</script>", offset),
    tail = source.slice(offset + name.length, scriptEnd < 0 ? source.length : scriptEnd),
    escaped = name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  return new RegExp(
    `\\b(?:const|let|var)\\s+${escaped}\\s*=\\s*(?:computed|customRef|ref|shallowRef|toRef|toRefs)\\s*\\(`,
    "u",
  ).test(tail);
}

function baselineIntentionalReason(finding: LintRecord, cwd: string): string | null {
  const quoted = /'([^']*)'/.exec(finding.message)?.[1]?.replace(/^(?:v-bind:|:)/, "");
  if (
    finding.ruleId === "vue/attribute-hyphenation" &&
    quoted !== undefined &&
    /^(?:aria|data)[A-Z]/.test(quoted)
  )
    return "Vize keeps camelCase aria*/data* names on components because hyphenating them would drop the prop type check.";
  let source: string;
  try {
    source = readFileSync(join(cwd, finding.file), "utf8");
  } catch {
    return null;
  }
  if (
    finding.ruleId === "vue/attribute-hyphenation" &&
    authoredForeignNativeOpening(finding, source) !== null
  )
    return "Patina recognizes the authored SVG or MathML tag as native even when it is a fragment root; eslint-plugin-vue loses the foreign-element namespace there and applies this component-only attribute rule.";
  if (
    finding.ruleId === "vue/no-v-text-v-html-on-component" &&
    authoredForeignNativeOpening(finding, source) !== null
  )
    return "Patina recognizes the authored SVG or MathML tag as native even when it is a fragment root; eslint-plugin-vue loses the foreign-element namespace there and treats v-text or v-html as component content.";
  if (
    finding.ruleId === "vue/valid-v-for" &&
    authoredForeignNativeOpening(finding, source) !== null
  )
    return "Patina recognizes the authored SVG or MathML tag as native even when it is a fragment root; eslint-plugin-vue loses the foreign-element namespace and incorrectly applies its custom-component v-for exemption.";
  if (finding.ruleId === "vue/require-explicit-slots") {
    const lines = source.split("\n"),
      startLine = lines[finding.line - 1],
      selected = lines.slice(finding.line - 1, finding.endLine).join("\n");
    return startLine?.slice(finding.column - 1).startsWith("<Slot") && selected.includes("<slot")
      ? "eslint-plugin-vue treats an imported uppercase <Slot> component as a slot outlet case-insensitively; Patina reports only the nested lowercase <slot> outlet that Vue actually renders as a slot."
      : null;
  }
  if (finding.ruleId === "vue/no-useless-mustaches") {
    if (finding.line !== finding.endLine) return null;
    const line = source.split("\n")[finding.line - 1];
    if (line === undefined) return null;
    const authored = line.slice(finding.column - 1, finding.endColumn - 1);
    const match = /^\{\{\s*(['"`])([\s\S]*?)\1\s*\}\}$/u.exec(authored);
    return match !== null && match[2].length > 0 && /^(?:\s|\\[tnrfv ])+$/u.test(match[2])
      ? "Patina keeps whitespace-only string interpolations because replacing them with static whitespace can be removed by Vue's whitespace condensing and change the rendered output."
      : null;
  }
  if (finding.ruleId === "vue/html-quotes") {
    if (finding.line !== finding.endLine) return null;
    const line = source.split("\n")[finding.line - 1];
    if (line === undefined) return null;
    const start = finding.column - 1,
      value = line.slice(start, finding.endColumn - 1);
    return value.startsWith("'") &&
      value.endsWith("'") &&
      value.slice(1, -1).includes('"') &&
      /(?:^|\s)(?:v-[^\s=]+|[:@#][^\s=]+)\s*=\s*$/u.test(line.slice(0, start))
      ? "Patina applies avoidEscape to directive expressions: changing this single-quoted binding to double quotes would require escaping the JavaScript string delimiters that the formatter emits."
      : null;
  }
  if (finding.ruleId !== "vue/no-v-text-v-html-on-component") return null;
  const lines = source.split("\n");
  if (finding.line > lines.length) return null;
  let offset = 0;
  for (let line = 1; line < finding.line; line += 1) offset += (lines[line - 1]?.length ?? 0) + 1;
  offset += finding.column - 1;
  const start = source.lastIndexOf("<component", offset),
    end = source.indexOf(">", offset);
  if (start < 0 || end < 0) return null;
  const opening = source.slice(start, end),
    binding = /(?:^|\s)(?::is|v-bind:is)\s*=\s*(["'])(.*?)\1/s.exec(opening)?.[2];
  if (binding === undefined) return null;
  const values = [...binding.matchAll(/(["'])(.*?)\1/g)].map((match) => match[2]),
    native = new Set([
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
  return values.length > 0 && values.every((value) => native.has(value))
    ? "Patina proves that this dynamic component expression can only select native elements, so v-text or v-html cannot overwrite a Vue component render tree."
    : null;
}

function selectDocumentedDivergences(
  values: unknown,
  projectId: string,
  cwd: string,
): DocumentedDivergence[] {
  if (!Array.isArray(values)) return invalid("documented divergences must be an array");
  const selected: DocumentedDivergence[] = [];
  const identities = new Set<string>();
  for (const [index, value] of (values as readonly unknown[]).entries()) {
    const label = `documented divergence ${index}`;
    if (!isRecord(value)) return invalid(`${label} must be an object`);
    if (typeof value.project !== "string" || value.project.length === 0) {
      invalid(`${label} must name a project`);
    }
    const file = normalizePath(value.file, cwd, `${label}.file`);
    if (!file.endsWith(".vue")) invalid(`${label} must reference a .vue file`);
    const { ruleId, issue } = value;
    if (typeof ruleId !== "string" || ruleId.length === 0) {
      return invalid(`${label}.ruleId must name the patina rule`);
    }
    const patina = documentedSide(value.patina, `${label}.patina`);
    const baseline = documentedSide(value.baseline, `${label}.baseline`);
    if (sameSpan(patina, baseline) && patina.severity === baseline.severity) {
      invalid(`${label} must record a difference between the two linters`);
    }
    if (typeof issue !== "number" || !Number.isSafeInteger(issue) || issue < 1) {
      return invalid(`${label}.issue must be the tracking issue number`);
    }
    // A ledger entry suppresses a real divergence, so it has to carry a written
    // rationale a reviewer can check rather than a placeholder.
    const reason = typeof value.reason === "string" ? value.reason.replace(/\s+/g, " ").trim() : "";
    if (reason.length < 40) invalid(`${label}.reason must explain why the divergence is expected`);
    const identity = [String(value.project), file, ruleId, patina.line, patina.column].join("\0");
    if (identities.has(identity)) invalid(`${label} duplicates an earlier documented divergence`);
    identities.add(identity);
    if (value.project !== projectId) continue;
    selected.push({ file, ruleId, patina, baseline, issue, reason });
  }
  return selected.sort(
    (left, right) =>
      byteOrder(left.file, right.file) ||
      byteOrder(left.ruleId, right.ruleId) ||
      left.patina.line - right.patina.line ||
      left.patina.column - right.patina.column,
  );
}

function pairDocumentedDivergences(
  expected: readonly DocumentedDivergence[],
  falsePositives: Compared[],
  falseNegatives: Compared[],
): DocumentedDivergence[] {
  const paired: DocumentedDivergence[] = [];
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

function findDocumented(
  records: readonly Compared[],
  divergence: DocumentedDivergence,
  side: DocumentedSide,
): number {
  return records.findIndex(
    (candidate) =>
      candidate.file === divergence.file &&
      candidate.ruleId === divergence.ruleId &&
      candidate.severity === side.severity &&
      sameSpan(candidate, side),
  );
}

function documentedSide(value: unknown, label: string): DocumentedSide {
  if (!isRecord(value)) return invalid(`${label} must be an object`);
  const { severity } = value;
  if (severity !== "error" && severity !== "warning") {
    return invalid(`${label}.severity must be error or warning`);
  }
  const position = (field: keyof LintRange): number => {
    const number = value[field];
    if (typeof number !== "number" || !Number.isSafeInteger(number) || number < 1) {
      return invalid(`${label}.${field} must be a positive safe integer`);
    }
    return number;
  };
  return {
    severity,
    line: position("line"),
    column: position("column"),
    endLine: position("endLine"),
    endColumn: position("endColumn"),
  };
}

function pairRuleLocationDivergences(
  falsePositives: Compared[],
  falseNegatives: Compared[],
): RuleLocationDivergence[] {
  const paired: RuleLocationDivergence[] = [];
  for (let positiveIndex = 0; positiveIndex < falsePositives.length;) {
    const positive = falsePositives[positiveIndex];
    if (positive === undefined) break;
    const match = findRuleLocationDivergence(positive, falseNegatives);
    if (match == null) {
      positiveIndex += 1;
      continue;
    }
    const [negative] = falseNegatives.splice(match.negativeIndex, 1);
    if (negative === undefined) break;
    falsePositives.splice(positiveIndex, 1);
    paired.push(ruleLocationDivergence(positive, negative, match));
  }
  return paired.sort(compareRuleLocationDivergences);
}

function findRuleLocationDivergence(
  positive: Compared,
  falseNegatives: readonly Compared[],
): RuleLocationMatch | null {
  const rule = ruleLocationDivergenceRules[positive.ruleId];
  if (rule == null) return null;
  for (let negativeIndex = 0; negativeIndex < falseNegatives.length; negativeIndex += 1) {
    const negative = falseNegatives[negativeIndex];
    if (
      negative === undefined ||
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

const ruleLocationDivergenceRules: Readonly<Record<string, RuleLocationRule>> = {
  "script/require-explicit-slots": {
    reason:
      "patina reports the actual lowercase <slot> outlet while eslint-plugin-vue can anchor the enclosing imported <Slot> component case-insensitively.",
    subject(positive, negative) {
      const slot = positive.message.match(/Slot '([^']+)'/u)?.[1];
      return slot !== undefined && containsSpan(negative, positive) ? `slot:${slot}` : null;
    },
  },
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

function ruleLocationDivergence(
  positive: Compared,
  negative: Compared,
  match: RuleLocationMatch,
): RuleLocationDivergence {
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

function side(value: LintRecord): LocationSide {
  return {
    line: value.line,
    column: value.column,
    endLine: value.endLine,
    endColumn: value.endColumn,
    message: value.message,
  };
}

function unusedComponentName(message: string): string | null {
  return (
    message.match(/Component '([^']+)' is registered but never used(?: in template)?/u)?.[1] ??
    message.match(/The "([^"]+)" component has been registered but not used\./u)?.[1] ??
    null
  );
}

function sameSpan(left: LintRange, right: LintRange): boolean {
  return (
    left.line === right.line &&
    left.column === right.column &&
    left.endLine === right.endLine &&
    left.endColumn === right.endColumn
  );
}

function containsSpan(outer: LintRange, inner: LintRange): boolean {
  return compareSpanStart(outer, inner) <= 0 && compareSpanEnd(outer, inner) >= 0;
}

function compareSpanStart(left: LintRange, right: LintRange): number {
  return left.line - right.line || left.column - right.column;
}

function compareSpanEnd(left: LintRange, right: LintRange): number {
  return left.endLine - right.endLine || left.endColumn - right.endColumn;
}

function groupByIdentity(records: readonly Compared[]): Map<string, Compared[]> {
  const groups = new Map<string, Compared[]>();
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

function compareShared(left: SharedPair, right: SharedPair): number {
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

function compareRuleLocationDivergences(
  left: RuleLocationDivergence,
  right: RuleLocationDivergence,
): number {
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

function ratio(count: number, total: number): number {
  return total === 0 ? 0 : count / total;
}
