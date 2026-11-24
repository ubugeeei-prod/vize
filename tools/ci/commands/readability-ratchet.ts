// The readability ratchet: shrink-only counts of the house-style debts that a
// formatter cannot fix, plus the byte-custody registry check.
//
//   node tools/commands/ci/readability-ratchet.ts --check       (part of `vp run check`)
//   node tools/commands/ci/readability-ratchet.ts --write       lower the baseline
//   node tools/commands/ci/readability-ratchet.ts --summary     totals per area
//   node tools/commands/ci/readability-ratchet.ts --files <path>...
//
// Options: `--root <dir>` scans another checkout, `--baseline <file>` reads or
// writes another baseline, `--no-custody` skips the registry, and `--reseed`
// (with `--write`) records the current counts even where they rose. Reseeding
// is only for a new counter or a maintainer-approved change of a definition.
//
// The counters and the area rules are described in `readability/counters.ts`
// and `readability/baseline.ts`. Only `enforcedCounters` fail `check`; the
// others are recorded and printed as advisories until the house style is
// adopted. A sweep lowers its area in the same commit with `--write`;
// afterwards any regrowth of an enforced counter fails `check`.

import fs from "node:fs";
import path from "node:path";

import { isEntrypoint } from "../../support/command-runtime.ts";
import { gitTrackedFiles } from "../../support/git-tracked-files.ts";
import { checkCustody, parseRegistry, registryPath } from "./byte-custody.ts";
import { compare, parseBaseline, renderBaseline, type Comparison } from "./readability/baseline.ts";
import {
  counterNames,
  countFile,
  enforcedCounters,
  fileLineLimit,
  languageOf,
  rustTestModules,
  type Counts,
} from "./readability/counters.ts";

export const baselinePath = "tools/config/readability-baseline.tsv";

const excludedSegments = new Set([
  "node_modules",
  "target",
  "dist",
  "fixtures",
  "_fixtures",
  "__fixtures__",
  "snapshots",
  "__snapshots__",
  "__snapshot__",
  "expected",
  "generated",
  "__generated__",
  "test-fixtures",
  "testdata",
  "vendor",
  "i18n",
  "corpora",
  "typecheck-corpora",
  "playwright-report",
  "coverage",
  ".vscode-test",
]);
const excludedPrefixes = [
  "upstream/",
  ".github/",
  "tools/benchmarks/results/",
  "tests/expected/",
  "tests/fuzz/regressions/",
  "docs/content/ja/",
  "docs/content/zh-CN/",
  "docs/content/pt-BR/",
  "docs/content/fr/",
];
const excludedFiles = new Set([
  "npm/cli/src/types/generated.ts",
  "npm/native/index.js",
  "npm/native/index.d.ts",
  "npm/fresco-native/index.js",
  "npm/fresco-native/index.d.ts",
]);
// `source-file-lengths.ts` exempts these from the length limit until the tools
// move lands; the `lines` counter follows it so the two gates agree.
const lengthExemptPrefixes = ["tools/commands/", "tools/support/"];
// A generated file says so in its first lines.
const headerLength = 400;
const leadingComments =
  /^(?:\uFEFF|#![^\n]*(?:\n|$))?(?:\s*(?:\/\/[^\n]*|\/\*[\s\S]*?(?:\*\/|$)|<!--[\s\S]*?(?:-->|$)))*/u;
const generatedHeader =
  /@generated|do not edit|auto-?generated|generated (?:by|from)|this file is generated/iu;

/** Whether a tracked path is first-party code the ratchet counts. */
export function inScope(file: string): boolean {
  if (!languageOf(file) || excludedFiles.has(file) || file.includes(".min.")) return false;
  if (excludedPrefixes.some((prefix) => file.startsWith(prefix))) return false;
  return !file
    .split("/")
    .slice(0, -1)
    .some((segment) => excludedSegments.has(segment));
}

export function countTree(root: string, files?: readonly string[]): Map<string, Counts> {
  const sources = new Map<string, string>();
  const testFiles = new Set<string>();
  const testDirectories: string[] = [];
  for (const file of files ?? gitTrackedFiles(root)) {
    if (!inScope(file)) continue;
    const absolute = path.join(root, file);
    if (!fs.lstatSync(absolute, { throwIfNoEntry: false })?.isFile()) continue;
    const source = fs.readFileSync(absolute, "utf8");
    if (generatedHeader.test(leadingComments.exec(source.slice(0, headerLength))?.[0] ?? ""))
      continue;
    sources.set(file, source);
    if (file.endsWith(".rs"))
      for (const module of rustTestModules(file, source))
        if (module.endsWith("/")) testDirectories.push(module);
        else testFiles.add(module);
  }
  const counts = new Map<string, Counts>();
  for (const [file, source] of sources) {
    const lines = !lengthExemptPrefixes.some((prefix) => file.startsWith(prefix));
    const test = testFiles.has(file) || testDirectories.some((prefix) => file.startsWith(prefix));
    counts.set(file, countFile(file, source, { lines, test }));
  }
  return counts;
}

export function describe(findings: Comparison["findings"]): string[] {
  const out: string[] = [];
  for (const finding of findings) {
    const what =
      finding.counter === "lines"
        ? `${finding.files[0].path} has ${finding.current} lines (limit ${Math.max(finding.baseline, fileLineLimit)}); split it by responsibility`
        : `${finding.area}: ${finding.counter} rose from ${finding.baseline} to ${finding.current}`;
    out.push(what);
    if (finding.counter !== "lines")
      for (const file of finding.files.slice(0, 10))
        out.push(`    ${file.path}: ${file.baseline} -> ${file.current}`);
  }
  return out;
}

const advice = [
  "The readability ratchet only lets these counts fall (tools/commands/ci/readability/counters.ts).",
  "Drop the issue number from the comment or name: the issue lives in the fixture manifest or",
  "ledger row, or in `TODO(#n)` for open work. Split a file that would pass 500 lines.",
];

function option(argv: readonly string[], name: string): string | undefined {
  const at = argv.indexOf(name);
  return at >= 0 ? argv[at + 1] : undefined;
}

export function main(argv: readonly string[]): number {
  if (argv.includes("--reseed") && !argv.includes("--write")) {
    console.error("--reseed requires --write");
    return 1;
  }
  const root = path.resolve(option(argv, "--root") ?? process.cwd());
  const baselineFile = path.resolve(root, option(argv, "--baseline") ?? baselinePath);
  const filesAt = argv.indexOf("--files");
  if (filesAt >= 0) {
    const files = argv.slice(filesAt + 1).filter((file) => !file.startsWith("--"));
    for (const [file, counts] of countTree(root, files))
      console.log([file, ...counterNames.map((name) => `${name}=${counts[name]}`)].join(" "));
    return 0;
  }
  const started = Date.now();
  const current = countTree(root);
  const baseline = fs.existsSync(baselineFile)
    ? parseBaseline(fs.readFileSync(baselineFile, "utf8"))
    : new Map<string, Counts>();
  const comparison = compare(baseline, current);
  const totals = counterNames
    .map((name) => `${name} ${comparison.totals[name].current}/${comparison.totals[name].baseline}`)
    .join(", ");
  console.log(
    `readability ratchet: ${current.size} files in ${Date.now() - started} ms; current/baseline: ${totals}`,
  );
  if (argv.includes("--summary")) printSummary(current);
  let status = 0;
  const failures = comparison.findings.filter((finding) => enforcedCounters.has(finding.counter));
  const advisories = comparison.findings.filter(
    (finding) => !enforcedCounters.has(finding.counter),
  );
  if (advisories.length)
    console.log(
      ["  advisory (not enforced until the house style is adopted):", ...describe(advisories)].join(
        "\n  ",
      ),
    );
  if (argv.includes("--write")) {
    if (failures.length && !argv.includes("--reseed")) {
      console.error(["Refusing to raise the baseline:", ...describe(failures)].join("\n"));
      return 1;
    }
  } else if (failures.length) {
    console.error([...describe(failures), "", ...advice].join("\n"));
    status = 1;
  } else if (comparison.lowerable)
    console.log(`  ${comparison.lowerable} baseline row(s) can be lowered with --write`);
  if (!argv.includes("--no-custody")) {
    const registryFile = path.join(root, registryPath);
    const registry = fs.existsSync(registryFile)
      ? parseRegistry(fs.readFileSync(registryFile, "utf8"))
      : new Map();
    const report = checkCustody(root, registry);
    console.log(
      `byte custody: ${registry.size} registered, ${report.problems.length} problem(s), ${report.stale.length} stale row(s)`,
    );
    for (const problem of report.problems) console.error(`  ${problem}`);
    if (report.problems.length) status = 1;
  }
  if (argv.includes("--write") && status === 0) {
    fs.writeFileSync(baselineFile, renderBaseline(current));
    console.log(`  wrote ${path.relative(root, baselineFile)}`);
  }
  return status;
}

function printSummary(current: ReadonlyMap<string, Counts>): void {
  const areas = new Map<string, Counts>();
  for (const [file, counts] of current) {
    const parts = file.split("/");
    const area = parts.length > 2 ? `${parts[0]}/${parts[1]}` : parts[0];
    const total =
      areas.get(area) ?? (Object.fromEntries(counterNames.map((n) => [n, 0])) as Counts);
    for (const name of counterNames)
      total[name] += name === "lines" ? +(counts.lines > 0) : counts[name];
    areas.set(area, total);
  }
  console.log(["area", "files>500", ...counterNames.slice(1)].join("\t"));
  for (const [area, total] of [...areas].sort(([left], [right]) => (left < right ? -1 : 1)))
    console.log([area, ...counterNames.map((name) => total[name])].join("\t"));
}

if (isEntrypoint(import.meta.url)) {
  try {
    process.exitCode = main(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
