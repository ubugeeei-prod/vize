import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { byteOrder, get, list, projectStrings, relativePath, type Json } from "./value.ts";
import { parseBudgetMode } from "./reporting.ts";

export interface Args {
  measureCoverageGap: boolean;
  budgetMode: string;
  outputDir: string;
  preset: string | null;
  projects: string[];
  registry: string;
  shardCount: bigint;
  shardIndex: bigint;
  timeoutMs: bigint;
  vizeBin: string | null;
}
export const help = [
  "Usage: rust-script tools/commands/fixtures/lint-divergence-report.rs [options]",
  "",
  "Classify vize lint against eslint-plugin-vue over the pinned real projects.",
  "",
  "  --project <id[,id]>     Limit registry projects",
  "  --preset <name>         Patina preset under test (default: ecosystem)",
  "  --all-mapped-rules      Compare every mapped rule, ignoring preset membership",
  "  --measure-coverage-gap  Also enable unimplemented upstream rules",
  "  --shard-index <n>       Zero-based project shard index",
  "  --shard-count <n>       Total balanced project shards",
  "  --output-dir <dir>      Report directory",
  "  --budget-mode <mode>    enforce or record-only (default: enforce)",
  "  --vize-bin <path>       Vize executable",
  "  --timeout-ms <n>        Per-project vize lint timeout",
  "",
].join("\n");
function integer(value: string, name: string, positive: boolean): bigint {
  const valid = /^\+?\d+$/.test(value),
    parsed = valid ? BigInt(value) : -1n;
  if (!valid || parsed > 18446744073709551615n || parsed < (positive ? 1n : 0n))
    throw new Error(`${name} must be a ${positive ? "positive" : "non-negative"} integer`);
  return parsed;
}
export function parseArgs(argv: string[], root: string): Args | null {
  const args: Args = {
    measureCoverageGap: false,
    budgetMode: "enforce",
    outputDir: path.join(root, ".vize/lint-divergence"),
    preset: "ecosystem",
    projects: [],
    registry: path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"),
    shardCount: 1n,
    shardIndex: 0n,
    timeoutMs: 600000n,
    vizeBin: null,
  };
  const absolute = (value: string) => (path.isAbsolute(value) ? value : path.join(root, value));
  for (let index = 0; index < argv.length; index++) {
    const arg = argv[index],
      value = () => {
        const next = argv[++index];
        if (next === undefined) throw new Error(`${arg} requires a value`);
        return next;
      };
    if (arg === "--output-dir") args.outputDir = absolute(value());
    else if (arg === "--budget-mode") args.budgetMode = parseBudgetMode(value());
    else if (arg === "--preset") args.preset = value();
    else if (arg === "--all-mapped-rules") args.preset = null;
    else if (arg === "--measure-coverage-gap") args.measureCoverageGap = true;
    else if (arg === "--project")
      args.projects.push(
        ...value()
          .split(",")
          .map((part) => part.trim())
          .filter(Boolean),
      );
    else if (arg === "--registry") args.registry = absolute(value());
    else if (arg === "--shard-count") args.shardCount = integer(value(), arg, true);
    else if (arg === "--shard-index") args.shardIndex = integer(value(), arg, false);
    else if (arg === "--timeout-ms") args.timeoutMs = integer(value(), arg, true);
    else if (arg === "--vize-bin") args.vizeBin = value();
    else if (arg === "--help" || arg === "-h") {
      process.stdout.write(help);
      return null;
    } else throw new Error(`Unknown argument: ${arg}`);
  }
  if (args.shardIndex >= args.shardCount)
    throw new Error("--shard-index must be less than --shard-count");
  return args;
}
export function selectedProjects(registry: Json, args: Args): Json[] {
  const projects = get(registry, "projects");
  if (!Array.isArray(projects)) throw new Error("registry must list projects");
  return projects
    .filter((project) => list(get(project, "coverage")).includes("linter"))
    .filter(
      (project) => !args.projects.length || args.projects.some((id) => get(project, "id") === id),
    )
    .filter((_, index) => BigInt(index) % args.shardCount === args.shardIndex);
}
function normalized(value: string): string {
  return value.replaceAll("\\", "/").replace(/^(?:\.\/)+/, "");
}

/** Rust glob::Pattern syntax: braces are literal, stars require separators. */
function globPattern(pattern: string, grammar: "vue" | "ignore" = "vue"): RegExp | null {
  let expression = "";
  for (let i = 0; i < pattern.length; i++) {
    const char = pattern[i];
    if (char === "*") {
      if (pattern[i + 1] === "*") {
        if ((i > 0 && pattern[i - 1] !== "/") || (i + 2 < pattern.length && pattern[i + 2] !== "/"))
          return null;
        i++;
        if (pattern[i + 1] === "/") {
          expression += "(?:[^/]+/)*";
          i++;
        } else expression += ".*";
      } else expression += "[^/]*";
    } else if (char === "?") expression += "[^/]";
    else if (char === "[") {
      let end = i + 1;
      if (pattern[end] === "!" || (grammar === "ignore" && pattern[end] === "^")) end++;
      if (pattern[end] === "]") end++;
      end = pattern.indexOf("]", end);
      if (end < 0) return null;
      let content = pattern.slice(i + 1, end);
      // glob::Pattern accepts only ! as class negation; globset also accepts ^.
      // A first ] is a literal member in both grammars, not an empty JS class.
      const negated = content.startsWith("!") || (grammar === "ignore" && content.startsWith("^"));
      if (negated) content = content.slice(1);
      const members = Array.from(content, (member) =>
        "\\[]^".includes(member) ? `\\${member}` : member,
      ).join("");
      expression += `${grammar === "vue" ? "(?!/)" : ""}[${negated ? "^" : ""}${members}]`;
      i = end;
    } else expression += char.replace(/[\\^$.*+?()[\]{}|]/g, "\\$&");
  }
  try {
    return new RegExp(`^${expression}$`, grammar === "vue" ? "u" : "");
  } catch {
    return null;
  }
}
interface IgnoreRule {
  base: string;
  pattern: string;
  negate: boolean;
  directory: boolean;
  git: boolean;
}
function gitRoot(directory: string): string | null {
  for (;;) {
    if (fs.existsSync(path.join(directory, ".git"))) return directory;
    const parent = path.dirname(directory);
    if (parent === directory) return null;
    directory = parent;
  }
}
function parseIgnoreRule(source: string, base: string, git: boolean): IgnoreRule[] {
  let pattern = source.trimEnd();
  if (!pattern || pattern.startsWith("#")) return [];
  const escapedMarker = pattern.startsWith("\\!") || pattern.startsWith("\\#");
  const negate = !escapedMarker && pattern.startsWith("!");
  if (escapedMarker || negate) pattern = pattern.slice(1);
  const directory = pattern.endsWith("/");
  if (directory) pattern = pattern.slice(0, -1);
  return [{ base, pattern, negate, directory, git }];
}
function ignoreRules(directory: string, git = false): IgnoreRule[] {
  if (git && gitRoot(directory) === null) return [];
  const filename = path.join(directory, git ? ".gitignore" : ".ignore");
  try {
    return fs
      .readFileSync(filename, "utf8")
      .split(/\r?\n/)
      .flatMap((source) => parseIgnoreRule(source, directory, git));
  } catch {
    return [];
  }
}
function ignoredByRules(
  filename: string,
  directory: boolean,
  rules: IgnoreRule[],
): boolean | undefined {
  let ignored: boolean | undefined;
  for (const rule of rules) {
    if (rule.directory && !directory) continue;
    const relative = relativePath(rule.base, filename),
      pattern = rule.pattern.replace(/^\//, "");
    const matcher = globPattern(pattern, "ignore");
    if (!matcher) continue;
    if (
      pattern.includes("/") || rule.pattern.startsWith("/")
        ? matcher.test(relative)
        : matcher.test(path.basename(relative))
    )
      ignored = !rule.negate;
  }
  return ignored;
}
function inheritedIgnoreRules(directory: string): IgnoreRule[] {
  const ancestors: string[] = [];
  let current = path.dirname(directory);
  for (;;) {
    ancestors.unshift(current);
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  const repository = gitRoot(directory);
  return ancestors.flatMap((ancestor) => [
    ...ignoreRules(ancestor),
    ...(repository !== null &&
    (ancestor === repository || ancestor.startsWith(repository + path.sep))
      ? ignoreRules(ancestor, true)
      : []),
  ]);
}
function gitDecisions(directory: string, filenames: string[]): Map<string, boolean> {
  if (!filenames.length) return new Map();
  const result = spawnSync("git", ["check-ignore", "--no-index", "--verbose", "-z", "--stdin"], {
    cwd: directory,
    input: filenames.join("\0") + "\0",
    encoding: "utf8",
    maxBuffer: Infinity,
  });
  const fields = (result.stdout ?? "").split("\0"),
    decisions = new Map<string, boolean>(),
    root = gitRoot(directory) ?? directory;
  for (let index = 0; index + 3 < fields.length; index += 4) {
    const origin = fields[index],
      pattern = fields[index + 2],
      filename = fields[index + 3];
    // WalkBuilder calls Gitignore::matched for each visited entry. Git's
    // check-ignore also propagates an ignored ancestor directory, which would
    // incorrectly suppress an explicitly selected root under e.g. target/.
    const base = path.dirname(path.isAbsolute(origin) ? origin : path.join(root, origin));
    let isDirectory = false;
    try {
      isDirectory = fs.statSync(filename).isDirectory();
    } catch {
      continue;
    }
    const decision = ignoredByRules(filename, isDirectory, parseIgnoreRule(pattern, base, true));
    if (decision !== undefined) decisions.set(filename, decision);
  }
  return decisions;
}
function walk(
  cwd: string,
  directory: string,
  matcher: RegExp | null,
  absolute: boolean,
  files: Set<string>,
  inherited: IgnoreRule[],
): void {
  let entries: fs.Dirent[];
  try {
    entries = fs.readdirSync(directory, { withFileTypes: true });
  } catch {
    return;
  }
  // A nested Git root stops parent Git rules, while ordinary .ignore rules
  // continue across that boundary (ignore::matched_ignore's saw_git contract).
  const parents = fs.existsSync(path.join(directory, ".git"))
    ? inherited.filter((rule) => !rule.git)
    : inherited;
  const rules = [...parents, ...ignoreRules(directory), ...ignoreRules(directory, true)];
  const paths = entries.map((entry) => path.join(directory, entry.name));
  const git = gitDecisions(directory, paths);
  for (const entry of entries) {
    const filename = path.join(directory, entry.name);
    const localIgnore =
      ignoredByRules(
        filename,
        entry.isDirectory(),
        rules.filter((rule) => !rule.git),
      ) ??
      ignoredByRules(
        filename,
        entry.isDirectory(),
        rules.filter((rule) => rule.git),
      ) ??
      git.get(filename);
    if (
      entry.name === ".git" ||
      (matcher === null && localIgnore === undefined && entry.name.startsWith(".")) ||
      localIgnore === true
    )
      continue;
    if (entry.isDirectory()) walk(cwd, filename, matcher, absolute, files, rules);
    else {
      try {
        if (!fs.statSync(filename).isFile()) continue;
      } catch {
        continue;
      }
      if (
        path.extname(filename) === ".vue" &&
        (matcher === null ||
          matcher.test(normalized(absolute ? filename : relativePath(cwd, filename))))
      )
        files.add(relativePath(cwd, filename));
    }
  }
}
export function collectVueInputPaths(cwd: string, project: Json): string[] {
  const files = new Set<string>();
  for (const pattern of projectStrings(project, "vueGlobs")) {
    const candidate = path.resolve(cwd, pattern);
    try {
      const stat = fs.statSync(candidate);
      if (stat.isFile()) {
        if (path.extname(candidate) === ".vue") files.add(relativePath(cwd, candidate));
        continue;
      }
      if (stat.isDirectory()) {
        walk(cwd, candidate, null, false, files, inheritedIgnoreRules(candidate));
        continue;
      }
    } catch {
      /* A glob's literal spelling normally does not exist. */
    }
    const start = pattern.search(/[*?[{]/),
      prefix = pattern.slice(0, start < 0 ? pattern.length : start);
    const base = !prefix
      ? "."
      : prefix.includes("/")
        ? prefix.slice(0, prefix.lastIndexOf("/")) || "."
        : prefix;
    const glob = normalized(pattern),
      matcher = globPattern(glob);
    const baseDirectory = path.resolve(cwd, base);
    walk(
      cwd,
      baseDirectory,
      matcher,
      path.isAbsolute(glob),
      files,
      inheritedIgnoreRules(baseDirectory),
    );
  }
  return [...files].sort(byteOrder);
}
