import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

/** Rule identifiers enforced by the component authoring gate. */
export type AuthoringRule =
  | "behavior-table"
  | "interaction-test"
  | "source-regex-behavior"
  | "explicit-sfc";

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
  const report = (file: string, rule: AuthoringRule, message: string) =>
    violations.push({ file: path.relative(root, file), rule, message });

  for (const sfc of sfcFiles) {
    const basename = path.basename(sfc);
    const source = await read(sfc);

    for (const message of explicitSfcProblems(source)) report(sfc, "explicit-sfc", message);

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

function explicitSfcProblems(source: string): string[] {
  const problems: string[] = [];
  if (!source.includes('<script setup lang="ts">')) {
    problems.push('Missing <script setup lang="ts"> block');
  }
  if (!source.includes("<template>")) problems.push("Missing <template> block");
  if (!source.includes("<style scoped>")) problems.push("Missing <style scoped> block");
  if (/\bh\s*\(/.test(source)) problems.push("Render-function escape hatch h() is not allowed");
  if (/defineOptions|withDefaults|interface (?:Props|Emits)/.test(source)) {
    problems.push("Use literal defineProps/defineEmits types without helper indirection");
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
