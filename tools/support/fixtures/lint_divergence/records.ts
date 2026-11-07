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
  if (finding.ruleId === "vue/no-shared-component-data")
    return "Patina intentionally merges vue/no-shared-component-data and vue/no-deprecated-data-object-declaration into one diagnostic for the same invalid data option.";
  if (
    source !== undefined &&
    source !== null &&
    hasUtsScript(source) &&
    ["script/define-props-destructuring", "vue/define-props-destructuring"].includes(finding.ruleId)
  )
    return "Patina does not apply JavaScript or TypeScript semantic script rules to a UTS block; UTS is a distinct native runtime language.";
  if (
    source !== undefined &&
    source !== null &&
    hasUtsScript(source) &&
    finding.ruleId === "vue/html-self-closing"
  )
    return "Patina preserves explicit closing tags in UTS-native component templates, where web-SFC self-closing rewrites are not a safe compatibility assumption.";
  if (
    ["vue/attribute-order", "vue/attributes-order"].includes(finding.ruleId) &&
    finding.message.includes('Attribute "v-bind" should go before')
  )
    return "Patina preserves a trailing object v-bind after authored listeners because moving it earlier changes Vue's merge precedence and can change which listener wins.";
  if (
    finding.ruleId === "vue/one-component-per-file" &&
    source !== undefined &&
    source !== null &&
    /\bcreateApp\s*\([^,]+,\s*\{/su.test(source)
  )
    return "eslint-plugin-vue counts createApp's root-props object as another component definition; Patina excludes that ordinary props object.";
  if (
    finding.ruleId === "vue/html-self-closing" &&
    source !== undefined &&
    source !== null &&
    mismatchedCaseClosingTag(finding, source)
  )
    return "Patina refuses to propose a self-closing rewrite for a component whose authored closing tag has mismatched casing; the malformed tag must be repaired first.";
  if (
    finding.ruleId === "vue/no-bare-strings-in-template" &&
    source !== undefined &&
    source !== null &&
    findingInsideElement(finding, source, "noscript")
  )
    return "Patina treats noscript contents as raw fallback markup and does not report its embedded template-like text as a user-facing bare string.";
  if (
    finding.ruleId === "vue/no-unused-components" &&
    source !== undefined &&
    source !== null &&
    /<p\b[^>]*>[\s\S]{0,4000}<div\b/iu.test(source)
  )
    return "Patina refuses an unused-component conclusion after invalid paragraph content forces HTML tree recovery; repair the malformed template before relying on component-use analysis.";
  if (
    finding.ruleId === "vue/no-unused-emit-declarations" &&
    source !== undefined &&
    source !== null &&
    templateEmitsDeclaredEvent(finding, source)
  )
    return "Patina reads the authored template expression and sees this declared event being emitted; eslint-plugin-vue does not expose that template call to this rule.";
  if (
    ["vue/require-explicit-slots", "vue/no-unused-refs"].includes(finding.ruleId) &&
    source !== undefined &&
    source !== null &&
    hasTableTransitionRecoveryBoundary(source)
  )
    return "Patina refuses a cross-template unused or explicit-slot conclusion after HTML table tree construction reparents a transition component; the recovered partial tree is not proof of the authored Vue scope.";
  if (
    finding.ruleId === "vue/no-deprecated-events-api" &&
    source !== undefined &&
    source !== null &&
    instanceEventsApiMayBeOverridden(finding, source)
  )
    return "Patina does not assume this.$on/$off/$once still names Vue's legacy Events API after an unknown object is copied onto this; eslint-plugin-vue reports the member spelling without accounting for that overwrite.";
  if (
    finding.ruleId === "vue/no-ref-object-reactivity-loss" &&
    source !== undefined &&
    source !== null &&
    usesLegacyReactivityTransform(source)
  )
    return "Patina does not model the removed experimental $ref/$() reactivity transform; eslint-plugin-vue applies its ref-loss analysis to that legacy transformed surface.";
  if (
    finding.ruleId === "vue/no-deprecated-filter" &&
    source !== undefined &&
    source !== null &&
    /\bas\s+['"][^'"]+['"]\s*\|\s*['"][^'"]+['"]/u.test(authoredFindingText(finding, source) ?? "")
  )
    return "vue-eslint-parser mistakes a TypeScript cast union in this directive expression for Vue 2 filter syntax; Patina preserves the authored TypeScript expression.";
  if (
    [
      "vue/prefer-use-template-ref",
      "vue/define-props-destructuring",
      "vue/no-setup-props-reactivity-loss",
    ].includes(finding.ruleId) &&
    source !== undefined &&
    source !== null &&
    hasModuleAndSetupScripts(source)
  )
    return "eslint-plugin-vue shifts this second-script diagnostic by the preceding module-script length; Patina preserves the authored SFC coordinates.";
  const name = quotedName(finding.message)?.replace(/^(?:v-bind:|:)/, "");
  if (
    finding.ruleId === "vue/require-default-prop" &&
    source !== undefined &&
    source !== null &&
    name !== undefined &&
    sameFileSpreadProvidesPropDefault(source, name)
  )
    return "Patina resolves this withDefaults key from a same-file constant spread; eslint-plugin-vue reports the optional prop because its defaults analysis does not cross the sibling script boundary.";
  if (
    ["vue/require-explicit-slots", "vue/no-multiple-template-root"].includes(finding.ruleId) &&
    source !== undefined &&
    source !== null &&
    importedUppercaseSlotRange(finding, source)
  )
    return finding.ruleId === "vue/require-explicit-slots"
      ? "eslint-plugin-vue treats an imported uppercase <Slot> component as a slot outlet case-insensitively; Patina reports only the nested lowercase <slot> outlet that Vue actually renders as a slot."
      : "eslint-plugin-vue treats an imported uppercase <Slot> component as a native slot outlet case-insensitively and rejects its nested lowercase <slot>; Patina preserves Vue's case-sensitive component semantics.";
  if (
    finding.ruleId === "vue/no-useless-mustaches" &&
    source !== undefined &&
    source !== null &&
    whitespaceOnlyMustache(finding, source)
  )
    return "Patina keeps whitespace-only string interpolations because replacing them with static whitespace can be removed by Vue's whitespace condensing and change the rendered output.";
  if (
    finding.ruleId === "vue/v-on-handler-style" &&
    source !== undefined &&
    source !== null &&
    stableVOnHandlerReference(finding, source)
  )
    return "Patina preserves a stable method reference across formatting whitespace and computed member syntax; eslint-plugin-vue misclassifies the authored reference as an inline handler.";
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
  const start = findingOffset(finding, source);
  if (start === null) return false;
  const tail = source.slice(start),
    close = tail.indexOf("</Slot>");
  return close >= 0 && tail.slice(0, close).includes("<slot");
}

function whitespaceOnlyMustache(finding: LintRecord, source: string): boolean {
  const authored = authoredFindingText(finding, source);
  if (authored === null) return false;
  const match = /^\{\{\s*(['"`])([\s\S]*?)\1\s*\}\}$/u.exec(authored);
  return match !== null && match[2].length > 0 && /^(?:\s|\\[tnrfv ])+$/u.test(match[2]);
}

function stableVOnHandlerReference(finding: LintRecord, source: string): boolean {
  const authored = authoredFindingText(finding, source);
  if (authored === null) return false;
  const compact = authored.trim().replaceAll(/\p{White_Space}+/gu, "");
  return /^[A-Za-z_$][\w$]*(?:(?:\??\.[A-Za-z_$][\w$]*)|(?:\[['"][^'"\\]+['"]\]))*$/u.test(compact);
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
    "style",
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
  if (hasUtsScript(source))
    return "Patina preserves linting for Vue files with UTS scripts; vue-eslint-parser rejects the UTS block before eslint-plugin-vue can run mapped rules.";
  if (finding.ruleId === "vue/no-setup-props-reactivity-loss")
    return "Patina's complete contract follows root-scope prop values through nested containers, call arguments, and assignments that eslint-plugin-vue does not inspect.";
  if (
    ["script/define-props-declaration", "script/define-emits-declaration"].includes(
      finding.ruleId,
    ) &&
    /<script\b[^>]*\b(?:setup\b[^>]*\blang\s*=\s*["']tsx["']|lang\s*=\s*["']tsx["'][^>]*\bsetup\b)/u.test(
      source,
    )
  )
    return 'Patina extends the declaration-style rule to <script setup lang="tsx">; eslint-plugin-vue only activates it for an exact lang="ts" block.';
  if (
    finding.ruleId === "vue/no-bare-strings-in-template" &&
    findingLine(finding, source)?.includes("v-text")
  )
    return "Patina checks each authored text segment surrounding a v-text code example; the upstream visitor drops those adjacent segments.";
  if (
    ["vue/multi-word-component-names", "vue/no-bare-strings-in-template"].includes(
      finding.ruleId,
    ) &&
    !/<script\b/iu.test(source)
  )
    return "Patina applies component and template-content policy to template-only SFCs; eslint-plugin-vue does not establish a component definition for this file and skips the equivalent check.";
  if (finding.ruleId === "script/no-ref-as-operand")
    return "Patina resolves the complete lexical ref set before checking every operand position; eslint-plugin-vue's source-order visitor misses this proven ref operand.";
  if (finding.ruleId === "script/no-side-effects-in-computed-properties")
    return "Patina checks external-state assignments and legacy Vue mutation helpers inside computed getters; eslint-plugin-vue's visitor misses this proven side effect.";
  if (finding.ruleId === "vue/no-unused-vars")
    return "Patina checks every authored v-for binding, including recovered and nested template scopes; eslint-plugin-vue misses this provably unused alias.";
  if (
    finding.ruleId === "vue/no-reserved-component-names" &&
    quotedName(finding.message) === "Component"
  )
    return "Patina treats the canonical dynamic-component name Component as reserved regardless of casing; eslint-plugin-vue misses this locally declared reserved name.";
  if (finding.ruleId === "vue/no-mutating-props" && mutationUsesTypeAssertion(finding, source))
    return "Patina unwraps TypeScript assertions on a proven prop mutation target; eslint-plugin-vue skips the mutation when the assignment target is wrapped in a cast.";
  if (
    finding.ruleId === "script/require-default-prop" &&
    typedDefaultComesFromComposedContract(finding, source)
  )
    return "Patina resolves optional props inherited through local interface heritage or intersections; eslint-plugin-vue does not expand that composed prop contract.";
  if (
    finding.ruleId === "script/no-boolean-default" &&
    findingInsideSameFileDefaultsSpread(finding, source)
  )
    return "Patina resolves Boolean defaults from a same-file constant spread; eslint-plugin-vue does not follow the spread into the sibling script block.";
  if (
    finding.ruleId === "script/require-valid-default-prop" &&
    genericRuntimeConstructorDefault(finding, source)
  )
    return "Patina recognizes a TypeScript-instantiated Array or Object expression as the corresponding runtime prop constructor; eslint-plugin-vue skips that generic constructor spelling.";
  if (
    finding.ruleId === "script/no-deprecated-delete-set" &&
    /@Component\b/u.test(source) &&
    /\bclass\s+[A-Za-z_$][\w$]*\s+extends\s+Vue\b/u.test(source)
  )
    return "Patina recognizes this decorated Vue class instance and reports its removed this.$set/this.$delete API; eslint-plugin-vue does not model the class-component receiver.";
  if (
    finding.ruleId === "script/return-in-computed-property" &&
    (usesAutoImportedComputed(source) || usesWrappedOptionsApi(source))
  )
    return "Patina checks computed getters reached through framework auto-imports and component-option wrappers; eslint-plugin-vue does not recognize that component surface.";
  if (finding.ruleId === "script/prefer-use-template-ref" && hasInlinePugTemplate(source))
    return "Patina correlates ref attributes in authored Pug templates with script refs; eslint-plugin-vue does not expose those Pug ref attributes to this rule.";
  if (
    [
      "script/no-deprecated-dollar-listeners-api",
      "script/no-deprecated-dollar-scopedslots-api",
    ].includes(finding.ruleId) &&
    findingInsideScriptBlock(finding, source)
  )
    return "Patina preserves script-side legacy instance API coverage when the fixture's eslint-plugin-vue path only reports equivalent template references.";
  if (
    ["script/require-default-prop", "script/require-prop-types"].includes(finding.ruleId) &&
    findingInsideReferencedPropsObject(finding, source)
  )
    return "Patina resolves a local runtime props object referenced by the component's props option; eslint-plugin-vue does not follow that identifier, so this is additional sound prop-contract coverage.";
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
  if (finding.ruleId === "script/prefer-use-template-ref" && hasModuleAndSetupScripts(source))
    return "Patina reports the authored template-ref declaration; eslint-plugin-vue shifts the same second-script diagnostic by the preceding module-script length.";
  if (
    finding.ruleId === "script/prefer-use-template-ref" &&
    templateRefIsOtherwiseUnread(finding, source)
  )
    return "Patina applies the Vue 3.5 template-ref migration to every statically paired ref declaration; eslint-plugin-vue misses the declaration when the script never reads the binding.";
  if (
    finding.ruleId === "script/one-component-per-file" &&
    /<script\b[^>]*\bsetup\b[^>]*>/iu.test(source) &&
    /\bdefineOptions\s*\(/u.test(source) &&
    /\bdefineComponent\s*\(/u.test(source)
  )
    return "Patina counts both the script-setup SFC described by defineOptions and its local defineComponent declaration; eslint-plugin-vue misses these script-setup component definitions.";
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
  if (
    finding.ruleId === "vue/v-slot-style" &&
    authoredFindingText(finding, source)?.trimStart().startsWith("#=")
  )
    return "Patina parses Vue's bare default-slot shorthand and enforces the configured component-slot style; vue-eslint-parser exposes it as a plain attribute, so eslint-plugin-vue cannot visit it.";
  if (finding.ruleId === "vue/no-mutating-props" && mutatesClassDecoratorProp(finding, source))
    return "Patina recognizes vue-property-decorator @Prop class members as component props and catches their template mutation; eslint-plugin-vue does not recover this class-decorator prop binding.";
  if (
    finding.ruleId === "vue/no-unused-components" &&
    registeredComponentIsProvablyAbsent(finding, source)
  )
    return "Patina reports a statically registered component that is absent from the authored executable template; eslint-plugin-vue misses this unused Options API registration.";
  if (finding.ruleId !== "script/define-props-destructuring") return null;
  if (hasModuleAndSetupScripts(source))
    return "Patina analyzes defineProps in a setup block after a normal script block; eslint-plugin-vue does not consistently expose that second-script finding.";
  const lines = source.split("\n");
  const lineNumber = Number(finding.line);
  const columnNumber = Number(finding.column);
  if (
    !Number.isSafeInteger(lineNumber) ||
    !Number.isSafeInteger(columnNumber) ||
    lineNumber < 1 ||
    lineNumber > lines.length ||
    columnNumber < 1
  )
    return null;
  let offset = 0;
  for (let line = 1; line < lineNumber; line++) offset += (lines[line - 1]?.length ?? 0) + 1;
  offset += columnNumber - 1;
  const authoredLine = lines[lineNumber - 1] ?? "";
  if (
    !source.startsWith("defineProps", offset) &&
    !/\b(?:const|let)\s+[A-Za-z_$][\w$]*\s*=\s*(?:withDefaults\s*\(\s*)?defineProps\s*[<(]/u.test(
      authoredLine,
    )
  )
    return null;
  return "Patina's default deliberately requires a direct bare defineProps call to bind an explicit destructuring pattern; eslint-plugin-vue does not report that stronger policy.";
}

function hasUtsScript(source: string): boolean {
  return /<script\b[^>]*\blang\s*=\s*(?:["']uts["']|uts)(?:\s|>)/u.test(source);
}

function registeredComponentIsProvablyAbsent(finding: LintRecord, source: string): boolean {
  const name = /^Component '([^']+)' is registered but never used in template$/u.exec(
    finding.message,
  )?.[1];
  if (name === undefined) return false;
  const template = /<template\b[^>]*>([\s\S]*?)<\/template>/iu.exec(source)?.[1];
  if (template === undefined || /<component\b/iu.test(template)) return false;
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  if (!new RegExp(`\\bcomponents\\s*:\\s*\\{[\\s\\S]{0,20000}\\b${escaped}\\b`, "u").test(source))
    return false;
  const kebab = name
    .replace(/([a-z\d])([A-Z])/gu, "$1-$2")
    .replace(/_/gu, "-")
    .toLowerCase();
  const escapedKebab = kebab.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  const executable = template.replace(/<!--[\s\S]*?-->/gu, "");
  return !new RegExp(`<\\s*(?:${escaped}|${escapedKebab})(?=[\\s/>])`, "iu").test(executable);
}

function hasModuleAndSetupScripts(source: string): boolean {
  return (
    /<script\b(?=[^>]*>)(?![^>]*\bsetup\b)[^>]*>/iu.test(source) &&
    /<script\b[^>]*\bsetup\b[^>]*>/iu.test(source)
  );
}

function sameFileSpreadProvidesPropDefault(source: string, prop: string): boolean {
  const spreadNames = [
    ...source.matchAll(/\bwithDefaults\s*\([\s\S]{0,20000}?\.\.\.([A-Za-z_$][\w$]*)/gu),
  ].map((match) => match[1]);
  const escapedProp = prop.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  for (const name of spreadNames) {
    const escapedName = name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    const declaration = new RegExp(
      `\\b(?:export\\s+)?const\\s+${escapedName}\\s*=\\s*\\{`,
      "u",
    ).exec(source);
    if (declaration?.index === undefined) continue;
    const start = declaration.index + declaration[0].length;
    const end = source.indexOf("}", start);
    if (end < 0) continue;
    if (new RegExp(`(?:^|[\\n,{])\\s*${escapedProp}\\s*:`, "u").test(source.slice(start, end)))
      return true;
  }
  return false;
}

function findingLine(finding: LintRecord, source: string): string | null {
  return source.split("\n")[Number(finding.line) - 1] ?? null;
}

function findingInsideElement(finding: LintRecord, source: string, tag: string): boolean {
  const offset = findingOffset(finding, source);
  if (offset === null) return false;
  const open = source.lastIndexOf(`<${tag}`, offset);
  const closeBefore = source.lastIndexOf(`</${tag}>`, offset);
  if (open < 0 || closeBefore > open) return false;
  const closeAfter = source.indexOf(`</${tag}>`, open);
  return closeAfter < 0 || offset <= closeAfter + tag.length + 3;
}

function usesAutoImportedComputed(source: string): boolean {
  return (
    /\bcomputed\s*\(/u.test(source) &&
    !/\bimport\s*\{[^}]*\bcomputed\b[^}]*\}\s*from\s*["'](?:vue|#imports)["']/su.test(source)
  );
}

function usesWrappedOptionsApi(source: string): boolean {
  return /\bexport\s+default\s+new\s+[A-Z_$][\w$]*\s*\(\s*\{[\s\S]*?\bcomputed\s*:/u.test(source);
}

function mismatchedCaseClosingTag(finding: LintRecord, source: string): boolean {
  const start = findingOffset(finding, source);
  if (start === null) return false;
  const authored = authoredFindingText(finding, source) ?? "";
  const closing = /<\/([A-Za-z][\w.-]*)\s*>/u.exec(authored);
  if (closing === null) return false;
  const closingOffset = start + (closing.index ?? 0);
  const prefix = source.slice(Math.max(0, closingOffset - 1000), closingOffset);
  const opening = /<([A-Z][\w.-]*)\b[^<>]*>\s*$/u.exec(prefix)?.[1];
  return (
    opening !== undefined &&
    opening !== closing[1] &&
    opening.toLowerCase() === closing[1]?.toLowerCase()
  );
}

function hasInlinePugTemplate(source: string): boolean {
  return /<template\b[^>]*\blang\s*=\s*(?:["']pug["']|pug)[^>]*>/u.test(source);
}

function templateEmitsDeclaredEvent(finding: LintRecord, source: string): boolean {
  if (!/<template\b/iu.test(source)) return false;
  const event = /["'`]([^"'`]+)["'`]/u.exec(finding.message)?.[1];
  if (event === undefined) return false;
  const escaped = event.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  return new RegExp(`\\$emit\\s*\\(\\s*["']${escaped}["']`, "u").test(source);
}

function hasTableTransitionRecoveryBoundary(source: string): boolean {
  return /<tbody\b[\s\S]*?<transition-group\b/iu.test(source);
}

function instanceEventsApiMayBeOverridden(finding: LintRecord, source: string): boolean {
  const offset = findingOffset(finding, source);
  if (offset === null) return false;
  return /\b(?:extend|Object\.assign)\s*\(\s*this\s*,[\s\S]*$/u.test(
    source.slice(Math.max(0, offset - 4_000), offset),
  );
}

function usesLegacyReactivityTransform(source: string): boolean {
  return /(?:\$ref|\$computed|\$shallowRef|\$customRef|\$toRef|\$)\s*\(/u.test(source);
}

function mutationUsesTypeAssertion(finding: LintRecord, source: string): boolean {
  const authored = authoredFindingText(finding, source) ?? findingLine(finding, source) ?? "";
  return /\bas\s+[A-Za-z_$][\w$]*(?:\s*<[^>]+>)?/u.test(authored);
}

function typedDefaultComesFromComposedContract(finding: LintRecord, source: string): boolean {
  const offset = findingOffset(finding, source);
  if (offset === null) return false;
  const before = source.slice(Math.max(0, offset - 20_000), offset);
  const interfaceStart = before.lastIndexOf("interface ");
  if (interfaceStart >= 0) {
    const headerEnd = source.indexOf("{", interfaceStart);
    const bodyEnd = source.indexOf("}", offset);
    if (
      headerEnd >= interfaceStart &&
      headerEnd < offset &&
      bodyEnd >= offset &&
      /\bextends\b/u.test(source.slice(interfaceStart, headerEnd))
    )
      return true;
    if (headerEnd >= interfaceStart && headerEnd < offset && bodyEnd >= offset) {
      const name = /^interface\s+([A-Za-z_$][\w$]*)/u.exec(source.slice(interfaceStart))?.[1];
      const root = /\bdefineProps\s*<\s*([A-Za-z_$][\w$]*)/u.exec(source)?.[1];
      if (name !== undefined && root !== undefined && localTypeDependsOn(source, root, name)) {
        return true;
      }
    }
  }
  const typeStart = before.lastIndexOf("type ");
  if (typeStart < 0) return false;
  const bodyStart = source.indexOf("{", typeStart);
  const bodyEnd = source.indexOf("}", offset);
  return (
    bodyStart >= typeStart &&
    bodyStart < offset &&
    bodyEnd >= offset &&
    /^\s*&/u.test(source.slice(bodyEnd + 1))
  );
}

function localTypeDependsOn(source: string, root: string, target: string): boolean {
  const pending = [root];
  const seen = new Set<string>();
  while (pending.length > 0 && seen.size < 32) {
    const name = pending.pop();
    if (name === undefined || !seen.add(name)) continue;
    if (name === target) return true;
    const escaped = name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    const interfaceMatch = new RegExp(
      `\\binterface\\s+${escaped}(?:\\s*<[^>{}]*>)?\\s*([^{}]*)\\{`,
      "u",
    ).exec(source);
    const aliasMatch = new RegExp(
      `\\btype\\s+${escaped}(?:\\s*<[^;={}]*>)?\\s*=\\s*([^;]+)`,
      "u",
    ).exec(source);
    const dependencyText = interfaceMatch?.[1] ?? aliasMatch?.[1] ?? "";
    for (const match of dependencyText.matchAll(/\b[A-Za-z_$][\w$]*\b/gu)) {
      pending.push(match[0]);
    }
  }
  return false;
}

function findingInsideSameFileDefaultsSpread(finding: LintRecord, source: string): boolean {
  const offset = findingOffset(finding, source);
  if (offset === null) return false;
  const spreadNames = [
    ...source.matchAll(/\bwithDefaults\s*\([\s\S]{0,20000}?\.\.\.([A-Za-z_$][\w$]*)/gu),
  ].map((match) => match[1]);
  return spreadNames.some((name) => {
    const escaped = name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
    const declaration = new RegExp(`\\b(?:export\\s+)?const\\s+${escaped}\\s*=\\s*\\{`, "u").exec(
      source,
    );
    if (declaration?.index === undefined) return false;
    const start = declaration.index + declaration[0].length;
    const end = source.indexOf("\n}", start);
    return start <= offset && offset < end;
  });
}

function genericRuntimeConstructorDefault(finding: LintRecord, source: string): boolean {
  const offset = findingOffset(finding, source);
  if (offset === null) return false;
  const before = source.slice(Math.max(0, offset - 500), offset);
  return /\btype\s*:\s*(?:Array|Object)\s*<[^>]+>\s*,[\s\S]*?\bdefault\s*:\s*$/u.test(before);
}

function templateRefIsOtherwiseUnread(finding: LintRecord, source: string): boolean {
  const name = quotedName(finding.message);
  if (name === undefined) return false;
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  const script = /<script\b[^>]*\bsetup\b[^>]*>([\s\S]*?)<\/script>/iu.exec(source)?.[1];
  if (script === undefined) return false;
  const references = script.match(new RegExp(`\\b${escaped}\\b`, "gu"))?.length ?? 0;
  return references === 1;
}

function findingInsideScriptBlock(finding: LintRecord, source: string): boolean {
  const offset = findingOffset(finding, source);
  if (offset === null) return false;
  const open = source.lastIndexOf("<script", offset);
  if (open < 0) return false;
  const content = source.indexOf(">", open) + 1;
  const close = source.indexOf("</script>", content);
  return content > 0 && content <= offset && (close < 0 || offset < close);
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

function authoredFindingText(finding: LintRecord, source: string): string | null {
  const start = findingOffset(finding, source),
    end = findingEndOffset(finding, source);
  return start === null || end === null || end < start ? null : source.slice(start, end);
}

function mutatesClassDecoratorProp(finding: LintRecord, source: string): boolean {
  const name = quotedName(finding.message)?.split(/[.[]/u)[0];
  if (name === undefined || name === "") return false;
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  return new RegExp(
    `@Prop\\b[\\s\\S]{0,500}?\\b(?:public\\s+|private\\s+|protected\\s+|readonly\\s+)*${escaped}\\b`,
    "u",
  ).test(source);
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
  "view",
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
    return (
      new RegExp(`\\bprops\\s*:\\s*${name}\\b`, "u").test(source.slice(offset)) ||
      (name === "props" && /\bprops\s*[,}]/u.test(source.slice(offset)))
    );
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
      if (positive.ruleId === "script/no-unused-emit-declarations") {
        const event = /(?:The ['`]|^['`])([^'`]+)(?:['`] event|['`] is)/u.exec(
          positive.message.replace(/^\[[^\]]*\]\s*/u, ""),
        )?.[1];
        const baselineEvent = /(?:The ['`]|^['`])([^'`]+)(?:['`] event|['`] is)/u.exec(
          negative.message.replace(/^\[[^\]]*\]\s*/u, ""),
        )?.[1];
        if (event !== undefined && event === baselineEvent) {
          match = {
            index,
            subject: `event:${event}`,
            reason:
              "patina reports the declared event name while eslint-plugin-vue reports the complete emit declaration entry.",
          };
          break;
        }
      }
      if (
        ["script/require-default-prop", "script/require-valid-default-prop"].includes(
          positive.ruleId,
        )
      ) {
        const prop = quotedName(positive.message);
        const baselineProp = quotedName(negative.message);
        if (prop !== undefined && prop === baselineProp) {
          match = {
            index,
            subject: `prop:${prop}`,
            reason:
              positive.ruleId === "script/require-default-prop"
                ? "both tools report the same inherited optional prop; Patina anchors the resolved base declaration while eslint-plugin-vue anchors the repeated derived declaration."
                : "both tools reject the same prop default; Patina reports the complete default expression while eslint-plugin-vue reports the returned value.",
          };
          break;
        }
      }
      if (positive.ruleId === "script/define-props-destructuring") {
        match = {
          index,
          subject: `define-props:${positive.line}`,
          reason:
            "patina reports the props binding identifier while eslint-plugin-vue reports the defineProps call through the containing declaration.",
        };
        break;
      }
      if (positive.ruleId === "vue/require-component-is") {
        match = {
          index,
          subject: "component-without-is",
          reason:
            "both tools report the same dynamic component without is; Patina reports the element while eslint-plugin-vue extends the range through recovered following content.",
        };
        break;
      }
      if (
        positive.ruleId === "vue/html-button-has-type" &&
        positive.line === negative.line &&
        (negative.line > positive.line ||
          (negative.column >= positive.column && negative.endColumn <= positive.endColumn))
      ) {
        match = {
          index,
          subject: `button:${positive.line}:${negative.column}`,
          reason:
            "patina reports the complete button opening element while eslint-plugin-vue reports only the invalid type attribute value.",
        };
        break;
      }
      if (
        positive.ruleId === "vue/no-multiple-template-root" &&
        positive.line === negative.line &&
        positive.column === negative.column
      ) {
        match = {
          index,
          subject: `root:${positive.line}:${positive.column}`,
          reason:
            "patina reports the extra root element itself while eslint-plugin-vue extends the diagnostic through the following root content.",
        };
        break;
      }
      if (
        positive.ruleId === "script/no-computed-properties-in-data" &&
        positive.line === negative.line &&
        positive.endLine === negative.endLine &&
        negative.column <= positive.column &&
        negative.endColumn === positive.endColumn
      ) {
        match = {
          index,
          subject: `computed-read:${positive.line}:${positive.column}`,
          reason:
            "patina reports the computed property identifier while eslint-plugin-vue includes the this receiver.",
        };
        break;
      }
      if (
        positive.ruleId === "vue/no-use-v-else-with-v-for" &&
        positive.line === negative.line &&
        positive.column === negative.column &&
        (positive.endLine > negative.endLine ||
          (positive.endLine === negative.endLine && positive.endColumn >= negative.endColumn))
      ) {
        match = {
          index,
          subject: `element:${positive.line}:${positive.column}`,
          reason:
            "both tools report the same authored element; their recovered end differs after nested template branches.",
        };
        break;
      }
      if (
        positive.ruleId === "script/no-deprecated-data-object-declaration" &&
        positive.line === negative.line &&
        positive.column === negative.column &&
        (negative.endLine > positive.endLine ||
          (negative.endLine === positive.endLine && negative.endColumn >= positive.endColumn))
      ) {
        match = {
          index,
          subject: `data:${positive.line}:${positive.column}`,
          reason:
            "patina reports the data property key while eslint-plugin-vue reports the complete object-valued property.",
        };
        break;
      }
      if (
        positive.ruleId === "vue/no-setup-props-reactivity-loss" &&
        (positive.line < negative.line ||
          (positive.line === negative.line && positive.column <= negative.column)) &&
        (positive.endLine > negative.endLine ||
          (positive.endLine === negative.endLine && positive.endColumn >= negative.endColumn))
      ) {
        match = {
          index,
          subject: `props-read:${negative.line}:${negative.column}`,
          reason:
            "patina reports the complete root-scope expression while eslint-plugin-vue reports the inner props access.",
        };
        break;
      }
      if (
        positive.ruleId === "vue/v-on-handler-style" &&
        (positive.line < negative.line ||
          (positive.line === negative.line && positive.column <= negative.column)) &&
        (positive.endLine > negative.endLine ||
          (positive.endLine === negative.endLine && positive.endColumn >= negative.endColumn))
      ) {
        match = {
          index,
          subject: `handler:${negative.line}:${negative.column}`,
          reason:
            "patina includes leading comments and formatting whitespace in the recovered handler span while eslint-plugin-vue starts at the executable expression.",
        };
        break;
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
        ["script/prefer-import-from-vue", "vue/no-dupe-v-else-if"].includes(positive.ruleId) &&
        (positive.line < negative.line ||
          (positive.line === negative.line && positive.column <= negative.column)) &&
        (positive.endLine > negative.endLine ||
          (positive.endLine === negative.endLine && positive.endColumn >= negative.endColumn))
      ) {
        match = {
          index,
          subject: `expression:${negative.line}:${negative.column}`,
          reason:
            positive.ruleId === "script/prefer-import-from-vue"
              ? "patina reports the complete import source literal while eslint-plugin-vue excludes the opening quote."
              : "patina reports the complete v-else-if condition while eslint-plugin-vue reports only the duplicated subexpression.",
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
      if (positive.ruleId === "vue/html-self-closing") {
        match = {
          index,
          subject: `element:${positive.line}:${negative.line}`,
          reason:
            "patina reports the complete authored start tag or empty element while eslint-plugin-vue reports the self-closing slash or closing tag.",
        };
        break;
      }
      if (
        [
          "script/require-default-prop",
          "script/no-boolean-default",
          "script/prefer-use-template-ref",
          "script/no-required-prop-with-default",
          "vue/no-setup-props-reactivity-loss",
        ].includes(positive.ruleId)
      ) {
        const startShift = Number(positive.line) - Number(negative.line),
          endShift = Number(positive.endLine) - Number(negative.endLine);
        if (
          startShift !== 0 &&
          startShift === endShift &&
          positive.column === negative.column &&
          positive.endColumn === negative.endColumn
        ) {
          const quoted = (message: string) => /['"`]([^'"`]+)['"`]/u.exec(message)?.[1],
            left = quoted(positive.message),
            right = quoted(negative.message);
          if (left !== undefined && right !== undefined && left !== right) continue;
          match = {
            index,
            subject: left === undefined ? `line-shift:${positive.column}` : `binding:${left}`,
            reason:
              "eslint-plugin-vue offsets this script-setup finding by the preceding normal script block; Patina reports the authored SFC line.",
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
