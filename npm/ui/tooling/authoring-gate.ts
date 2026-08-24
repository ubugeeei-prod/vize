import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

import { parse } from "@vue/compiler-sfc";

import { VIZE_UI_SFC_AUTHORING_RULES, type SfcAuthoringRuleId } from "./authoring-contract.ts";

/** Rule identifiers enforced by the component authoring gate. */
export type AuthoringRule = SfcAuthoringRuleId;

/** One violation of the component authoring standard. */
export interface AuthoringViolation {
  /** Path relative to the audited directory. */
  readonly file: string;

  /** Rule the file violates. */
  readonly rule: AuthoringRule;

  /** Human-readable explanation with the expected remediation. */
  readonly message: string;
}

const SOURCE_REGEX_ASSERTION = /\.(?:match|doesNotMatch)\(\s*source\b/;
const SFC_SOURCE_READ = /readFile\([^)]*\.vue/;
const SOURCE_CONTRACT_PRAGMA = "source-contract:";
const DEFINE_PROPS_TYPE = /defineProps\s*<\s*\{([\s\S]*?)\}\s*>\s*\(/g;
const DEFINE_EMITS_TYPE = /defineEmits\s*<\s*\{([\s\S]*?)\}\s*>\s*\(/g;
const PROP_DECLARATION =
  /(?:(\/\*\*[\s\S]*?\*\/)\s*)?readonly\s+(?:"([^"]+)"|'([^']+)'|([A-Za-z_$][\w$]*))\??\s*:/g;
const EVENT_DECLARATION =
  /(?:(\/\*\*[\s\S]*?\*\/)\s*)?(?:"([^"]+)"|'([^']+)'|([A-Za-z_$][\w$]*))\s*:\s*\[/g;
const AUTHORING_RULE_IDS = new Set<AuthoringRule>(
  VIZE_UI_SFC_AUTHORING_RULES.map((rule) => rule.id),
);

/**
 * Audit a source directory against the Vize UI component authoring standard.
 *
 * Every SFC must ship a normative behavior table (a `*.behavior.md` that
 * references the SFC filename) and a mounted-DOM interaction test (a
 * `*.test.ts` that imports the SFC). Regex-on-source is not behavior
 * evidence: `assert.match(source, ...)`-style assertions and `.vue` source
 * reads inside tests are violations unless the same or one of the two
 * preceding lines carries a `source-contract:` pragma naming why mounted-DOM
 * behavior cannot observe the contract.
 * Public props must also document their default value with an `@default` tag
 * in the prop JSDoc, so hover and generated docs preserve first-render behavior.
 *
 * @param sourceDirectory Directory containing SFC sources, tests, and behavior tables.
 * @returns Violations in stable path order; an empty array means full compliance.
 */
export async function auditComponentAuthoring(
  sourceDirectory: string,
): Promise<readonly AuthoringViolation[]> {
  const root = path.resolve(sourceDirectory);
  const files = await collectFiles(root);
  const read = (filename: string) => readFile(filename, "utf8");

  const sfcFiles = files.filter((filename) => filename.endsWith(".vue"));
  const testFiles = files.filter((filename) => filename.endsWith(".test.ts"));
  const behaviorFiles = files.filter((filename) => filename.endsWith(".behavior.md"));

  const testSources = await Promise.all(testFiles.map(read));
  const behaviorSources = await Promise.all(behaviorFiles.map(read));

  const violations: AuthoringViolation[] = [];
  const report = (file: string, rule: AuthoringRule, message: string) => {
    assertPublishedAuthoringRule(rule);
    violations.push({ file: path.relative(root, file), rule, message });
  };

  for (const sfc of sfcFiles) {
    const basename = path.basename(sfc);
    const source = await read(sfc);

    for (const message of explicitSfcProblems(source, sfc)) report(sfc, "explicit-sfc", message);
    for (const message of propDefaultDocProblems(source, sfc)) {
      report(sfc, "prop-default-doc", message);
    }
    for (const message of eventDocProblems(source, sfc)) report(sfc, "event-doc", message);

    if (!behaviorSources.some((table) => table.includes(basename))) {
      report(
        sfc,
        "behavior-table",
        `No *.behavior.md references ${basename}; add a normative state x input -> outcome table`,
      );
    }

    const importPattern = new RegExp(`from\\s+["'][^"']*/${basename.replaceAll(".", "\\.")}["']`);
    if (!testSources.some((test) => importPattern.test(test))) {
      report(
        sfc,
        "interaction-test",
        `No *.test.ts imports ${basename}; add mounted-DOM interaction tests`,
      );
    }
  }

  testFiles.forEach((test, index) => {
    const lines = (testSources[index] ?? "").split("\n");
    lines.forEach((line, lineIndex) => {
      if (!SOURCE_REGEX_ASSERTION.test(line) && !SFC_SOURCE_READ.test(line)) return;
      if (hasSourceContractPragma(lines, lineIndex)) return;
      report(
        test,
        "source-regex-behavior",
        `Line ${lineIndex + 1} asserts on component source text; ` +
          "prove the behavior on mounted DOM or annotate with a source-contract: pragma",
      );
    });
  });

  return violations.sort(
    (left, right) => left.file.localeCompare(right.file) || left.rule.localeCompare(right.rule),
  );
}

function assertPublishedAuthoringRule(rule: AuthoringRule): void {
  if (!AUTHORING_RULE_IDS.has(rule)) {
    throw new Error(`Authoring rule ${rule} is not published in the SFC authoring contract`);
  }
}

/**
 * Render authoring violations for terminals and continuous integration.
 *
 * @param violations Violations returned by {@link auditComponentAuthoring}.
 * @returns A newline-delimited report; an empty string means a clean audit.
 */
export function formatAuthoringViolations(violations: readonly AuthoringViolation[]): string {
  return violations
    .map((violation) => `${violation.file} [${violation.rule}] ${violation.message}`)
    .join("\n");
}

function explicitSfcProblems(source: string, filename: string): string[] {
  const { descriptor, errors } = parse(source, { filename });
  const problems: string[] = [];
  if (errors.length > 0) problems.push(`SFC source has ${errors.length} parse error(s)`);
  if (descriptor.scriptSetup?.lang !== "ts") {
    problems.push('Missing <script setup lang="ts"> block');
  }
  if (descriptor.template === null) problems.push("Missing <template> block");
  if (!descriptor.styles.some((style) => style.scoped)) {
    problems.push("Missing <style scoped> block");
  }
  const scripts = [descriptor.script?.content, descriptor.scriptSetup?.content]
    .filter((script): script is string => script !== undefined)
    .join("\n");
  if (/\bh\s*\(/.test(scripts)) problems.push("Render-function escape hatch h() is not allowed");
  if (/defineOptions|withDefaults|interface (?:Props|Emits)/.test(scripts)) {
    problems.push("Use literal defineProps/defineEmits types without helper indirection");
  }
  return problems;
}

function propDefaultDocProblems(source: string, filename: string): string[] {
  const { descriptor, errors } = parse(source, { filename });
  if (errors.length > 0 || descriptor.scriptSetup === null) return [];

  const problems: string[] = [];
  for (const propsType of descriptor.scriptSetup.content.matchAll(DEFINE_PROPS_TYPE)) {
    const body = propsType[1] ?? "";
    for (const prop of body.matchAll(PROP_DECLARATION)) {
      const jsdoc = prop[1] ?? "";
      if (/@default\b/u.test(jsdoc)) continue;
      const propName = prop[2] ?? prop[3] ?? prop[4] ?? "<unknown>";
      problems.push(
        `Prop ${propName} is missing @default documentation; document the public default value`,
      );
    }
  }
  return problems;
}

function eventDocProblems(source: string, filename: string): string[] {
  const { descriptor, errors } = parse(source, { filename });
  if (errors.length > 0 || descriptor.scriptSetup === null) return [];

  const problems: string[] = [];
  for (const emitsType of descriptor.scriptSetup.content.matchAll(DEFINE_EMITS_TYPE)) {
    const body = emitsType[1] ?? "";
    for (const event of body.matchAll(EVENT_DECLARATION)) {
      if ((event[1] ?? "").trim().length > 0) continue;
      const eventName = event[2] ?? event[3] ?? event[4] ?? "<unknown>";
      problems.push(
        `Event ${eventName} is missing documentation; document dispatch timing and payload intent`,
      );
    }
  }
  return problems;
}

function hasSourceContractPragma(lines: readonly string[], lineIndex: number): boolean {
  return lines
    .slice(Math.max(0, lineIndex - 2), lineIndex + 1)
    .some((line) => line.includes(SOURCE_CONTRACT_PRAGMA));
}

async function collectFiles(directory: string): Promise<string[]> {
  const entries = await readdir(directory, { withFileTypes: true });
  const files: string[] = [];
  for (const entry of entries) {
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) files.push(...(await collectFiles(filename)));
    else if (entry.isFile()) files.push(filename);
  }
  return files.sort();
}
