/**
 * Test case reports: one JSON packet per test job, read by the Check test report.
 * A packet names the job that ran the cases and its run attempt, and lists every
 * case with the repository path that owns it. Producers know only their runner.
 */
import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { asArray, asNumber, asRecord, asString, parseJson } from "../../json-value.ts";

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

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  const [mode, junit, out, ...rest] = process.argv.slice(2);
  if (mode !== "nextest" || !junit || !out || rest.length)
    throw new Error("Usage: test-cases.ts nextest <junit.xml> <packet.json>");
  const root = fileURLToPath(new URL("../../../../", import.meta.url));
  const cases = nextestCases(readFileSync(junit, "utf8"), workspaceCrates(root));
  writeFileSync(out, `${JSON.stringify(packet(process.env, cases))}\n`);
  console.log(`Recorded ${cases.length} test cases in ${out}`);
}
