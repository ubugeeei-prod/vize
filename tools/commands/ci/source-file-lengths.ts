import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import fs from "node:fs";

interface SourceFile {
  path: string;
  lines: number;
}

type FailureReason = "new file exceeds limit" | "crossed limit" | "over-limit file grew";

interface CheckFailure extends SourceFile {
  baseLines?: number;
  reason: FailureReason;
}

const sourceSuffixes = [
  ".rs",
  ".ts",
  ".tsx",
  ".js",
  ".jsx",
  ".mjs",
  ".cjs",
  ".mts",
  ".cts",
  ".vue",
  ".css",
  ".scss",
  ".html",
  ".pkl",
  ".mbt",
  ".scm",
  ".lua",
  ".json",
  ".toml",
  ".yaml",
  ".yml",
  ".md",
];

function lineCount(content: string): number {
  if (content.length === 0) return 0;
  const withoutFinalNewline = content.endsWith("\n") ? content.slice(0, -1) : content;
  return withoutFinalNewline.split("\n").length;
}

function hasSegment(file: string, segment: string): boolean {
  return file.startsWith(`${segment}/`) || file.includes(`/${segment}/`);
}

function isExcluded(file: string): boolean {
  return (
    file.endsWith("Cargo.lock") ||
    file.endsWith("pnpm-lock.yaml") ||
    file.endsWith("package-lock.json") ||
    file.endsWith(".lock") ||
    file.startsWith("tools/benchmarks/results/") ||
    file.startsWith("tools/commands/") ||
    file.startsWith("tools/support/") ||
    file.startsWith("upstream/forks/") ||
    file === "npm/cli/schemas/vize.config.schema.json" ||
    file === "npm/cli/src/types/generated.ts" ||
    file === "docs/davinci/plan/croquis-consumption.md" ||
    file === "docs/davinci/plan/corpus-coverage.md" ||
    file === "package.json" ||
    file.endsWith("/package.json") ||
    file === ".github/workflows/release.yml" ||
    ["ja", "zh-CN", "pt-BR", "fr"].some((locale) => file.startsWith(`docs/content/${locale}/`)) ||
    file.includes(".min.") ||
    [
      "_fixtures",
      "__fixtures__",
      "fixtures",
      "__snapshots__",
      "snapshots",
      "dist",
      "target",
      "node_modules",
      "vendor",
      "generated",
      "__generated__",
      "i18n",
      "playwright-report",
      "coverage",
    ].some((segment) => hasSegment(file, segment)) ||
    [
      "npm/native/index.js",
      "npm/native/index.d.ts",
      "npm/fresco-native/index.js",
      "npm/fresco-native/index.d.ts",
    ].includes(file)
  );
}

function git(args: readonly string[]): SpawnSyncReturns<string> {
  return spawnSync("git", [...args], { encoding: "utf8", maxBuffer: Number.POSITIVE_INFINITY });
}

function trackedPaths(): string[] {
  const result = git(["ls-files"]);
  if (result.error !== undefined || result.status !== 0) {
    throw new Error("Failed to read tracked files with git ls-files");
  }
  return result.stdout.trimEnd().split("\n").filter(Boolean);
}

/** Replace comments and literals with spaces while retaining line offsets. */
function rustCode(source: string): string {
  const output = source.split("");
  const blank = (start: number, end: number): void => {
    for (let index = start; index < end; index += 1) {
      if (output[index] !== "\n") output[index] = " ";
    }
  };
  let index = 0;
  while (index < source.length) {
    if (source.startsWith("//", index)) {
      const end = source.indexOf("\n", index + 2);
      const stop = end === -1 ? source.length : end;
      blank(index, stop);
      index = stop;
      continue;
    }
    if (source.startsWith("/*", index)) {
      const start = index;
      let depth = 1;
      index += 2;
      while (index < source.length && depth > 0) {
        if (source.startsWith("/*", index)) {
          depth += 1;
          index += 2;
        } else if (source.startsWith("*/", index)) {
          depth -= 1;
          index += 2;
        } else index += 1;
      }
      blank(start, index);
      continue;
    }
    const raw = /^(?:br|r)(#+)?"/u.exec(source.slice(index));
    if (raw !== null) {
      const start = index;
      const hashes = raw[1] ?? "";
      index += raw[0].length;
      const closing = `"${hashes}`;
      const end = source.indexOf(closing, index);
      index = end === -1 ? source.length : end + closing.length;
      blank(start, index);
      continue;
    }
    if (source[index] === '"') {
      const start = index++;
      while (index < source.length) {
        if (source[index] === "\\") index += 2;
        else if (source[index++] === '"') break;
      }
      blank(start, index);
      continue;
    }
    index += 1;
  }
  return output.join("");
}

function moduleLayoutFailures(paths: readonly string[]): string[] {
  const failures: string[] = [];
  for (const file of paths.filter((candidate) => candidate.endsWith(".rs"))) {
    if (
      !file.startsWith("tools/commands/") &&
      ["fixtures", "_fixtures", "__fixtures__"].some((segment) => hasSegment(file, segment))
    ) {
      continue;
    }
    let source: string;
    try {
      source = fs.readFileSync(file, "utf8");
    } catch (error) {
      failures.push(`${file}: ${error instanceof Error ? error.message : String(error)}`);
      continue;
    }
    if (source.startsWith("#!") && !source.startsWith("#![")) {
      source = source.slice(source.indexOf("\n") + 1);
    }
    const code = rustCode(source);
    for (const match of code.matchAll(/#\s*!?\s*\[([\s\S]*?)\]/gu)) {
      const body = match[1]?.trimStart() ?? "";
      if (/^path\b/u.test(body) || (/^cfg_attr\b/u.test(body) && /\bpath\s*=/u.test(body))) {
        const line = source.slice(0, match.index).split("\n").length;
        failures.push(`${file}:${line}: path attribute`);
      }
    }
  }
  return failures;
}

function currentFiles(paths: readonly string[]): SourceFile[] {
  const files: SourceFile[] = [];
  for (const file of paths) {
    if (!sourceSuffixes.some((suffix) => file.endsWith(suffix)) || isExcluded(file)) continue;
    try {
      if (!fs.lstatSync(file).isFile()) continue;
      files.push({ path: file, lines: lineCount(fs.readFileSync(file, "utf8")) });
    } catch {
      // Preserve the former inventory behavior: unreadable entries are omitted.
    }
  }
  return files.sort(
    (left, right) => right.lines - left.lines || left.path.localeCompare(right.path),
  );
}

function refExists(reference: string): boolean {
  return reference !== "" && git(["rev-parse", "--verify", `${reference}^{commit}`]).status === 0;
}

function renamePaths(reference: string): Map<string, string> {
  const result = git(["diff", "--name-status", "--find-renames", reference, "--"]);
  const paths = new Map<string, string>();
  if (result.status !== 0) return paths;
  for (const line of result.stdout.trimEnd().split("\n")) {
    const [status, before, after] = line.split("\t");
    if (status?.startsWith("R") && before !== undefined && after !== undefined) {
      paths.set(after, before);
    }
  }
  return paths;
}

function baseLines(
  reference: string,
  file: string,
  renames: ReadonlyMap<string, string>,
): number | undefined {
  const result = git(["show", `${reference}:${renames.get(file) ?? file}`]);
  return result.status === 0 ? lineCount(result.stdout) : undefined;
}

function failures(
  files: readonly SourceFile[],
  maximum: number,
  reference: string,
  renames: ReadonlyMap<string, string>,
): CheckFailure[] {
  const result: CheckFailure[] = [];
  for (const file of files) {
    if (file.lines <= maximum) continue;
    const previous = baseLines(reference, file.path, renames);
    let reason: FailureReason | undefined;
    if (previous === undefined) reason = "new file exceeds limit";
    else if (previous <= maximum) reason = "crossed limit";
    else if (file.lines > previous) reason = "over-limit file grew";
    if (reason !== undefined) result.push({ ...file, baseLines: previous, reason });
  }
  return result.sort(
    (left, right) => right.lines - left.lines || left.path.localeCompare(right.path),
  );
}

function printInventory(files: readonly SourceFile[], maximum: number, limit: number): void {
  const over = files.filter((file) => file.lines > maximum);
  console.log(`Source files scanned: ${files.length}`);
  console.log(`Files over ${maximum} lines: ${over.length}`);
  console.log("\n| Lines | Path |\n| ---: | --- |");
  for (const file of over.slice(0, Math.max(0, limit)))
    console.log(`| ${file.lines} | \`${file.path}\` |`);
}

function readInteger(args: readonly string[], index: number, name: string): number {
  const source = args[index + 1];
  if (source === undefined) throw new Error(`Missing value for ${name}`);
  if (!/^-?\d+$/u.test(source)) throw new Error(`Invalid ${name} value: ${source}`);
  return Number(source);
}

function main(): number {
  const args = process.argv.slice(2);
  let maximum = 500;
  let limit = 50;
  let check = false;
  let reference = "";
  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === "--check") check = true;
    else if (argument === "--max-lines") maximum = readInteger(args, index++, argument);
    else if (argument === "--limit") limit = readInteger(args, index++, argument);
    else if (argument === "--base-ref") {
      const value = args[++index];
      if (value === undefined) throw new Error("Missing value for --base-ref");
      reference = value;
    } else if (argument === "--help") {
      console.log(
        "Usage: node tools/commands/ci/source-file-lengths.ts [--check --base-ref <ref>] [--max-lines 500] [--limit 50]",
      );
      return 0;
    } else throw new Error(`Unknown argument: ${argument}`);
  }

  const paths = trackedPaths();
  const moduleFailures = moduleLayoutFailures(paths);
  const files = currentFiles(paths);
  printInventory(files, maximum, limit);
  if (!check) return 0;
  if (moduleFailures.length > 0) {
    console.log("Rust modules must use ordinary module discovery:");
    for (const failure of moduleFailures) console.log(`- ${failure}`);
    return 1;
  }
  if (!refExists(reference)) {
    console.log("\nNo comparable base ref found; inventory completed without enforcing growth.");
    return 0;
  }
  const rejected = failures(files, maximum, reference, renamePaths(reference));
  console.log(`\nCompared with ${reference}.`);
  if (rejected.length === 0) {
    console.log(`No new or grown files exceed ${maximum} lines.`);
    return 0;
  }
  console.log("Files requiring action:");
  console.log("| Lines | Base | Reason | Path |");
  console.log("| ---: | ---: | --- | --- |");
  for (const failure of rejected) {
    console.log(
      `| ${failure.lines} | ${failure.baseLines ?? "-"} | ${failure.reason} | \`${failure.path}\` |`,
    );
  }
  return 1;
}

try {
  process.exitCode = main();
} catch (error) {
  console.log(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
