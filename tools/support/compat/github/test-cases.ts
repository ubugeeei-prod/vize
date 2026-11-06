/**
 * Test case reports: one JSON packet per test job, read by the Check test report.
 * A packet names the job that ran the cases and its run attempt, and lists every
 * case with the repository path that owns it. Producers know only their runner.
 */
import { readFileSync, readdirSync, realpathSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  asArray,
  asNumber,
  asRecord,
  asString,
  asStringArray,
  parseJson,
} from "../../json-value.ts";

export type Outcome = "passed" | "failed" | "skipped";
const OUTCOMES: readonly Outcome[] = ["passed", "failed", "skipped"];
export interface TestCase {
  /** Crate directory or test file, relative to the repository root. */
  readonly path: string;
  readonly name: string;
  readonly outcome: Outcome;
}
export interface TestCasePacket {
  readonly version: 1;
  /** Display name of the producing job, as the Actions jobs API reports its tail. */
  readonly job: string;
  readonly attempt: number;
  readonly cases: readonly TestCase[];
}

function positive(value: string | undefined, name: string): number {
  if (!value || !/^[1-9]\d*$/.test(value)) throw new Error(`${name} must be a positive integer`);
  return Number(value);
}

export function packet(
  env: Readonly<Record<string, string | undefined>>,
  cases: readonly TestCase[],
): TestCasePacket {
  const job = env.TEST_CASES_JOB;
  if (!job) throw new Error("TEST_CASES_JOB is required");
  return {
    version: 1,
    job,
    attempt: positive(env.GITHUB_RUN_ATTEMPT, "GITHUB_RUN_ATTEMPT"),
    cases,
  };
}

export function parsePacket(value: unknown, label: string): TestCasePacket {
  const record = asRecord(value, label);
  if (record.version !== 1) throw new Error(`${label} has an unknown version`);
  return {
    version: 1,
    job: asString(record.job, `${label}.job`),
    attempt: asNumber(record.attempt, `${label}.attempt`),
    cases: asArray(record.cases, `${label}.cases`).map((entry, index) => {
      const item = asRecord(entry, `${label}.cases[${index}]`);
      const outcome = asString(item.outcome, `${label}.cases[${index}].outcome`);
      if (!OUTCOMES.some((known) => known === outcome))
        throw new Error(`${label}.cases[${index}] has outcome ${outcome}`);
      return {
        path: asString(item.path, `${label}.cases[${index}].path`),
        name: asString(item.name, `${label}.cases[${index}].name`),
        outcome: outcome as Outcome,
      };
    }),
  };
}

/** Every packet under `directory`, keeping each job's latest attempt up to `attempt`. */
export function readPackets(directory: string, attempt: number): TestCasePacket[] {
  const files: string[] = [];
  const walk = (dir: string) => {
    for (const entry of readdirSync(dir)) {
      const file = path.join(dir, entry);
      if (statSync(file).isDirectory()) walk(file);
      else if (entry.endsWith(".json")) files.push(file);
    }
  };
  walk(directory);
  const latest = new Map<string, TestCasePacket>();
  for (const file of files.sort()) {
    const read = parsePacket(parseJson(readFileSync(file, "utf8")), file);
    if (read.attempt > attempt) throw new Error(`${file} is from a later attempt ${read.attempt}`);
    const kept = latest.get(read.job);
    if (kept?.attempt === read.attempt)
      throw new Error(`Two packets for ${read.job} ${read.attempt}`);
    if (!kept || kept.attempt < read.attempt) latest.set(read.job, read);
  }
  return [...latest.values()];
}

/** Crate name to its workspace directory, from the root manifest's members. */
export function workspaceCrates(root: string): Map<string, string> {
  const manifest = readFileSync(path.join(root, "Cargo.toml"), "utf8");
  const members = /^members\s*=\s*\[([^\]]*)\]/m.exec(manifest)?.[1];
  if (!members) throw new Error("Cargo.toml has no workspace members");
  const crates = new Map<string, string>();
  for (const [, member] of members.matchAll(/"([^"]+)"/g)) {
    const own = readFileSync(path.join(root, member, "Cargo.toml"), "utf8");
    const name = /^\[package\][^[]*?^name\s*=\s*"([^"]+)"/m.exec(own)?.[1];
    if (!name) throw new Error(`${member}/Cargo.toml has no package name`);
    crates.set(name, member);
  }
  return crates;
}

function decode(text: string): string {
  return text.replace(/&(lt|gt|quot|apos|amp|#x[0-9a-f]+|#\d+);/gi, (entity, body: string) => {
    const named: Record<string, string> = { lt: "<", gt: ">", quot: '"', apos: "'", amp: "&" };
    if (named[body] !== undefined) return named[body];
    const code = body[1] === "x" ? Number.parseInt(body.slice(2), 16) : Number(body.slice(1));
    return Number.isFinite(code) ? String.fromCodePoint(code) : entity;
  });
}

export interface NodeTestEvent {
  readonly type: string;
  readonly data?: {
    readonly name?: string;
    readonly file?: string;
    readonly skip?: boolean | string;
    readonly todo?: boolean | string;
    readonly details?: { readonly type?: string };
  };
}

function canonical(file: string): string {
  const resolved = path.resolve(file);
  try {
    return realpathSync(resolved);
  } catch {
    return resolved;
  }
}

/** Repository path of a test file. A file outside `root` is refused. */
export function ownedPath(file: string, root: string): string {
  const direct = path.relative(root, file).split("\\").join("/");
  const rel =
    direct && !direct.startsWith("../") && !path.isAbsolute(direct)
      ? direct
      : path.relative(canonical(root), canonical(file)).split("\\").join("/");
  if (!rel || rel.startsWith("../") || path.isAbsolute(rel))
    throw new Error(`test file ${file} is outside ${root}`);
  return rel;
}

// Node's own summary counts tests, not suites. A file that fails to load is one
// failed test whose name is that file.
export function nodeTestCases(events: readonly NodeTestEvent[], root = process.cwd()): TestCase[] {
  const cases: TestCase[] = [];
  for (const { type, data } of events) {
    if ((type !== "test:pass" && type !== "test:fail") || data?.details?.type !== "test") continue;
    if (!data.file || !data.name) throw new Error(`A ${type} event has no file or name`);
    const file = ownedPath(data.file, root);
    const reported = data.name.split("\\").join("/");
    const outcome: Outcome =
      data.skip || data.todo ? "skipped" : type === "test:fail" ? "failed" : "passed";
    // A load failure is named after the file, absolutely or relative to the root.
    const name =
      reported === file || reported === data.file.split("\\").join("/")
        ? file
        : `${file} › ${data.name}`;
    cases.push({ path: file, name, outcome });
  }
  return cases;
}

const VITEST_OUTCOMES: Readonly<Record<string, Outcome>> = {
  passed: "passed",
  failed: "failed",
  pending: "skipped",
  skipped: "skipped",
  todo: "skipped",
};

function vitestName(file: string, ancestors: readonly string[], title: string): string {
  const base = path.posix.basename(file);
  const parts = [...ancestors, title]
    .map((part) => part.trim())
    .filter((part) => part && part !== file && part !== base);
  return parts.length === 0 ? file : `${file} › ${parts.join(" › ")}`;
}

/** Cases of a Vitest JSON report. Pending and todo tests are skipped. */
export function vitestCases(report: unknown, root: string): TestCase[] {
  const record = asRecord(report, "vitest report");
  const files = asArray(record.testResults, "vitest report.testResults");
  const cases: TestCase[] = [];
  for (const [index, entry] of files.entries()) {
    const label = `vitest report.testResults[${index}]`;
    const fileRecord = asRecord(entry, label);
    const file = ownedPath(asString(fileRecord.name, `${label}.name`), root);
    const assertions = asArray(fileRecord.assertionResults, `${label}.assertionResults`);
    if (assertions.length === 0) {
      if (fileRecord.status === "failed") cases.push({ path: file, name: file, outcome: "failed" });
      continue;
    }
    for (const [caseIndex, assertion] of assertions.entries()) {
      const caseLabel = `${label}.assertionResults[${caseIndex}]`;
      const item = asRecord(assertion, caseLabel);
      const status = asString(item.status, `${caseLabel}.status`);
      const outcome = VITEST_OUTCOMES[status];
      if (!outcome) throw new Error(`${caseLabel} has status ${status}`);
      cases.push({
        path: file,
        name: vitestName(
          file,
          asStringArray(item.ancestorTitles, `${caseLabel}.ancestorTitles`),
          asString(item.title, `${caseLabel}.title`),
        ),
        outcome,
      });
    }
  }
  return cases;
}

/** Cases of a nextest JUnit report; the binary id's crate gives each case's path. */
export function nextestCases(junit: string, crates: ReadonlyMap<string, string>): TestCase[] {
  // Stored output can hold XML-looking text in CDATA.
  const structure = junit.replace(/<!\[CDATA\[[\s\S]*?\]\]>/g, "").replace(/<!--[\s\S]*?-->/g, "");
  const cases: TestCase[] = [];
  for (const match of structure.matchAll(/<testcase\b([^>]*?)(?:\/>|>([\s\S]*?)<\/testcase>)/g)) {
    const attributes = Object.fromEntries(
      [...match[1].matchAll(/([\w-]+)="([^"]*)"/g)].map(([, key, value]) => [key, decode(value)]),
    );
    const { classname, name } = attributes;
    if (!classname || !name) throw new Error("A JUnit test case has no classname or name");
    const crate = classname.split("::")[0];
    const owner = crates.get(crate);
    if (!owner) throw new Error(`JUnit crate ${crate} is not a workspace member`);
    const body = match[2] ?? "";
    // Retried passes keep <flakyFailure>; only a final <failure> or <error> fails.
    const outcome: Outcome = /<(?:failure|error)\b/.test(body)
      ? "failed"
      : /<skipped\b/.test(body)
        ? "skipped"
        : "passed";
    cases.push({ path: owner, name: `${classname} ${name}`, outcome });
  }
  return cases;
}

const PLAYWRIGHT_OUTCOMES: Readonly<Record<string, Outcome>> = {
  passed: "passed",
  failed: "failed",
  timedOut: "failed",
  interrupted: "failed",
  skipped: "skipped",
};

/** Cases from Playwright's JSON reporter, using the final retry as the outcome. */
export function playwrightCases(report: unknown, root: string): TestCase[] {
  const cases: TestCase[] = [];
  const reportRecord = asRecord(report, "playwright report");
  const config = asRecord(reportRecord.config, "playwright report.config");
  const configuredRoot = asString(config.rootDir, "playwright report.config.rootDir");
  const reporterRoot = path.isAbsolute(configuredRoot)
    ? configuredRoot
    : path.resolve(root, configuredRoot);
  const visit = (suite: unknown, ancestors: readonly string[], label: string) => {
    const record = asRecord(suite, label);
    const title = typeof record.title === "string" ? record.title.trim() : "";
    const titles = title ? [...ancestors, title] : ancestors;
    for (const [index, entry] of asArray(record.specs ?? [], `${label}.specs`).entries()) {
      const specLabel = `${label}.specs[${index}]`;
      const spec = asRecord(entry, specLabel);
      const reported = asString(spec.file, `${specLabel}.file`).split("\\").join("/");
      const file = ownedPath(
        path.isAbsolute(reported) ? reported : path.resolve(reporterRoot, reported),
        root,
      );
      const specTitle = asString(spec.title, `${specLabel}.title`).trim();
      for (const [testIndex, testEntry] of asArray(spec.tests, `${specLabel}.tests`).entries()) {
        const testLabel = `${specLabel}.tests[${testIndex}]`;
        const testRecord = asRecord(testEntry, testLabel);
        const results = asArray(testRecord.results, `${testLabel}.results`);
        if (results.length === 0 && testRecord.expectedStatus !== "skipped") continue;
        const final = results.length
          ? asRecord(results.at(-1), `${testLabel}.results[${results.length - 1}]`)
          : null;
        const status = final?.status ?? testRecord.expectedStatus;
        const outcome = typeof status === "string" ? PLAYWRIGHT_OUTCOMES[status] : undefined;
        if (!outcome) throw new Error(`${testLabel} has no final Playwright outcome`);
        const project = asString(testRecord.projectName, `${testLabel}.projectName`).trim();
        const parts = [...titles, specTitle, project].filter(
          (part) => part && part !== reported && part !== path.posix.basename(reported),
        );
        cases.push({ path: file, name: `${file} › ${parts.join(" › ")}`, outcome });
      }
    }
    for (const [index, child] of asArray(record.suites ?? [], `${label}.suites`).entries())
      visit(child, titles, `${label}.suites[${index}]`);
  };
  for (const [index, suite] of asArray(reportRecord.suites, "playwright report.suites").entries())
    visit(suite, [], `playwright report.suites[${index}]`);
  return cases;
}

/** Combine partial packets produced by multiple runners in one Actions job. */
export function mergePackets(packets: readonly TestCasePacket[]): TestCasePacket {
  const [first, ...rest] = packets;
  if (!first) throw new Error("At least one test case packet is required");
  for (const current of rest) {
    if (current.job !== first.job || current.attempt !== first.attempt)
      throw new Error("Merged test case packets must have the same job and attempt");
  }
  return { ...first, cases: packets.flatMap((current) => current.cases) };
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  const [mode, ...args] = process.argv.slice(2);
  const usage =
    "Usage: test-cases.ts <nextest|vitest|node|playwright> <report> <packet.json> | merge <packet.json> <partial...>";
  if (mode === "merge") {
    const [out, ...reports] = args;
    if (!out || reports.length === 0) throw new Error(usage);
    const merged = mergePackets(
      reports.map((file) => parsePacket(parseJson(readFileSync(file, "utf8")), file)),
    );
    writeFileSync(out, `${JSON.stringify(merged)}\n`);
    console.log(`Merged ${merged.cases.length} test cases into ${out}`);
    process.exit(0);
  }
  const [report, out, ...rest] = args;
  if (!report || !out || rest.length) throw new Error(usage);
  const root = fileURLToPath(new URL("../../../../", import.meta.url));
  const cases =
    mode === "nextest"
      ? nextestCases(readFileSync(report, "utf8"), workspaceCrates(root))
      : mode === "vitest"
        ? vitestCases(parseJson(readFileSync(report, "utf8")), root)
        : mode === "node"
          ? nodeTestCases(
              readFileSync(report, "utf8")
                .split("\n")
                .filter((line) => line.trim())
                .map((line) => parseJson(line) as NodeTestEvent),
              root,
            )
          : mode === "playwright"
            ? playwrightCases(parseJson(readFileSync(report, "utf8")), root)
            : null;
  if (!cases) throw new Error(usage);
  writeFileSync(out, `${JSON.stringify(packet(process.env, cases))}\n`);
  console.log(`Recorded ${cases.length} test cases in ${out}`);
}
