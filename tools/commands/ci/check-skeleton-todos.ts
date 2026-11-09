import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, extname, join } from "node:path";
import { isEntrypoint } from "../../support/command-runtime.ts";

const baselinePath = "tools/config/skeleton-todos.toml";
const marker = "expect(clippy::todo";

export function parseBaseline(source: string): Map<string, number> {
  const entries = new Map<string, number>();
  for (const [index, original] of source.split(/\r?\n/u).entries()) {
    const line = original.split("#", 1)[0]?.trim() ?? "";
    if (line === "") continue;
    const separator = line.indexOf("=");
    const key = separator < 0 ? "" : line.slice(0, separator).trim().replace(/^"|"$/gu, "");
    const literal = separator < 0 ? "" : line.slice(separator + 1).trim();
    const value = Number(literal);
    if (key === "" || !/^\d+$/u.test(literal) || !Number.isSafeInteger(value)) {
      throw new Error(`${baselinePath}:${index + 1}: expected \`crate = count\``);
    }
    if (entries.has(key)) throw new Error(`${baselinePath}:${index + 1}: duplicate entry`);
    entries.set(key, value);
  }
  return entries;
}

export function countMarkers(source: string): number {
  const code = source
    .split(/\r?\n/u)
    .filter((line) => !line.trimStart().startsWith("//"))
    .join("")
    .replaceAll(/\s/gu, "");
  return code.split(marker).length - 1;
}

function rustFiles(directory: string): string[] {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const target = join(directory, entry.name);
    if (entry.isDirectory()) {
      return entry.name === "target" || entry.name === "node_modules" ? [] : rustFiles(target);
    }
    return entry.isFile() && extname(entry.name) === ".rs" ? [target] : [];
  });
}

function countCrates(directory: string): Map<string, number> {
  const counts = new Map<string, number>();
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const target = join(directory, entry.name);
    if (!entry.isDirectory()) continue;
    const count = rustFiles(target).reduce(
      (total, file) => total + countMarkers(readFileSync(file, "utf8")),
      0,
    );
    if (count > 0) counts.set(basename(target), count);
  }
  return counts;
}

export function renderBaseline(previous: string, counts: ReadonlyMap<string, number>): string {
  const header: string[] = [];
  for (const line of previous.split(/\r?\n/u)) {
    if (line.trim() !== "" && !line.trimStart().startsWith("#")) break;
    header.push(line);
  }
  const lines = [...counts].sort(([left], [right]) => left.localeCompare(right));
  return [...header, ...lines.map(([name, count]) => `${name} = ${count}`), ""].join("\n");
}

export function checkSkeletonTodos(arguments_: readonly string[]): number {
  const check = arguments_.includes("--check");
  const write = arguments_.includes("--write");
  const baselineText = readFileSync(baselinePath, "utf8");
  const baseline = parseBaseline(baselineText);
  const counts = new Map([...countCrates("crates"), ...countCrates("davinci")]);
  const problems: string[] = [];
  let raised = false;
  for (const [crate, count] of counts) {
    const limit = baseline.get(crate);
    if (limit === undefined) {
      raised = true;
      problems.push(`${crate}: ${count} skeleton todo module(s) but no entry in ${baselinePath}`);
    } else if (count > limit) {
      raised = true;
      problems.push(
        `${crate}: ${count} skeleton todo module(s), baseline ${limit}; counts only go down`,
      );
    } else if (count < limit) {
      problems.push(
        `${crate}: ${count} skeleton todo module(s), baseline ${limit}; lower the baseline (--write)`,
      );
    }
  }
  for (const crate of baseline.keys()) {
    if (!counts.has(crate)) {
      problems.push(`${crate}: no skeleton todo modules left; remove its entry (--write)`);
    }
  }

  const total = [...counts.values()].reduce((sum, count) => sum + count, 0);
  console.log(`skeleton todos: ${total} module(s) in ${counts.size} crate(s)`);
  if (write) {
    if (raised) {
      console.error(problems.join("\n"));
      return 1;
    }
    writeFileSync(baselinePath, renderBaseline(baselineText, counts));
    console.log(`  wrote ${baselinePath}`);
    return 0;
  }
  if (problems.length === 0 || !check) {
    for (const problem of problems) console.log(`  ${problem}`);
    return 0;
  }
  console.error(problems.join("\n"));
  return 1;
}

if (isEntrypoint(import.meta.url)) {
  try {
    process.exitCode = checkSkeletonTodos(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
