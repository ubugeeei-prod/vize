/**
 * Replay the codename removal (#6832; maintainer decisions 2026-11-03 and 2026-11-09).
 *
 * - Program name: every path and line outside docs/davinci/ loses it. A whole word becomes
 *   `levels` (the level crates' home), an embedded one becomes `level`.
 * - Level codenames: each becomes its level name (see `levelNames`) in paths and lines. Markdown
 *   under docs/ and lines that say "codename" (the alias form) keep them; other data under docs/
 *   names current code and follows.
 *   A level codename counts only where a word part starts, so `historicalComplete` and
 *   `portfolio` are not codenames.
 *
 * Case is preserved. On a conflict, take main and re-run.
 *   node tools/support/levels/rename-codenames.ts --check|--move|--rewrite
 * Replay: `--move` and commit (move-only), then `--rewrite` (reformats what it touched) and
 * commit; `--check` is then empty.
 */
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = fileURLToPath(new URL("../../../", import.meta.url));
const NAME = "davinci";
const kept = "docs/" + NAME;
const self = "tools/support/levels/rename-codenames.ts";
// The completion gate must spell the names it bans; it moves but keeps its bytes.
const gate = /^tools\/commands\/[^/]+\/completion-gates\.ts$/u;
const capture =
  /\.(?:log|jsonl|stderr|stdout)(?:\.txt)?$|(?:^|\/)(?:stderr|stdout)\.txt$|(?:^|\/)(?:fix-history|capture-[^/]+)\/|(?:^|\/)(?:original-[^/]*|observed-[^/]*\.json|[^/]*[.-]observation\.json)$/u;
/**
 * Captured output that a live parser or comparison reads in tests is a test input and follows the
 * rename: observer logs, the generated L3 reference graphs and the dump CLI's expected stderr.
 */
const parsedCapture =
  /^(?:tests\/tooling\/fixtures\/canonical-observer-logs|tests\/fixtures\/l3-reference|crates\/vize\/tests\/fixtures\/dump_cli)\//u;
/**
 * Records that pin git trees at past revisions name the paths those trees had, and so do the
 * verifiers that read those revisions (the warm type-backed path and timing host moves and the
 * raw git-reader harness, whose body digests are of past bytes), and the L2 dump tests that pin
 * the historical v1 wire headers byte for byte; and the literal listing sources (empty-vue-star,
 * misskey registry, javascript-sfc source context and its callers) that list trees of past revisions,
 * the instance-globals source-packet provenance captured from past runs, and the product fix-history
 * ledgers, which list the paths each past commit touched.
 * The lint preset audit keeps the nextest.toml it reviewed and its baseline scope as written.
 * Archived instruction budgets and reports are evidence of past toolchain re-baselines.
 * The retained template-emit projection manifest is regenerated only from immutable past artifacts.
 * The 47c6 deprecated-template research packets are bound to their retained source revision.
 */
const pinnedTrees =
  /^(?:editors\/vscode\/test\/css-hover-source-custody\.json|tests\/performance\/support\/warm-type-backed-(?:(?:path|timing)-host(?:-manifest\.json|\.ts)|git-reader-harness\.ts)|tests\/tooling\/warm-type-backed-(?:path|timing)-host\.test\.ts|levels\/vize_l2\/tests\/dump_(?:protocol|owner_names)\.rs|tools\/support\/compat\/github\/(?:empty-vue-star-listing-(?:source|part-\d+)|misskey-registry-part-\d+|javascript-sfc-source-context(?:-\d+)?)\.json|tests\/_fixtures\/ci\/javascript-sfc-source-callers\.json|tests\/fixtures\/typechecker\/undeclared-instance-globals-10617\/source-packets-[^/]+-provenance\.json|docs\/davinci\/plan\/[a-z-]+-fix-history[a-z-]*\.tsv|tests\/_fixtures\/lint-preset-default-warnings\/(?:cases\.json|reviewed-nextest\.toml)|tests\/_fixtures\/tooling\/instruction-methodology-transition\/[^/]+|tests\/_fixtures\/differential\/typechecker\/template-dollar-emit\/projection-retained-[^/]+\.manifest\.json|tests\/_fixtures\/differential\/lint\/deprecated-template-research-47c6\/[^/]+)$/u;
/**
 * Frozen copies of past files are captures too: a source or config file saved as `.txt` or
 * `.fixture`, and the frozen-source and source-custody trees that pin such copies.
 */
const frozenCopy =
  /\.(?:rs|ts|mts|mjs|js|json|toml|lock|ya?ml|tsv|vue|md)\.(?:txt|fixture)$|(?:^|\/)(?:frozen-source|source-custody)\//u;
/** Captured output of a past run is history and keeps its bytes. */
export const keepsBytes = (file: string): boolean =>
  pinnedTrees.test(file) ||
  (frozenCopy.test(file) && !parsedCapture.test(file)) ||
  (capture.test(file) && !parsedCapture.test(file));

interface LevelName {
  /** Lowercase spelling for snake, kebab and plain words. */
  readonly lower: string;
  /** Kebab spelling, when it differs from `lower`. */
  readonly kebab?: string;
  /** Spelling as a capitalized word part. */
  readonly pascal: string;
  /** Spelling as a whole lowercase word, when it differs from `lower`. */
  readonly word?: string;
}

/** Each level codename and the name that replaces it. */
export const levelNames: Readonly<Record<string, LevelName>> = {
  sinopia: { lower: "l1", pascal: "L1" },
  ricalco: { lower: "l1_to_l2", kebab: "l1-to-l2", pascal: "L1ToL2" },
  disegno: { lower: "l2", pascal: "L2" },
  impeto: { lower: "l3", pascal: "L3" },
  stesura: { lower: "l4", pascal: "L4" },
  folio: { lower: "dump", pascal: "Dump" },
  spolvero: { lower: "stage", pascal: "Stage", word: "stages" },
};
const levelPattern = new RegExp(Object.keys(levelNames).join("|"), "giu");
const anyPattern = new RegExp([NAME, ...Object.keys(levelNames)].join("|"), "giu");

const wordChar = /[A-Za-z0-9_-]/u;
// The file kinds a codename-bearing path can end in.
const fileExtension =
  /^\.(?:md|rs|ts|tsx|mjs|js|json|toml|tsv|vue|snap|txt|ya?ml|sh|folio|dump)(?![A-Za-z0-9_-])/u;
const lowerChar = /[a-z]/u;
const letter = /[A-Za-z]/u;
const protectedSpans = new RegExp(
  [
    `${kept}(?![A-Za-z0-9_-])`,
    `docs%2F${NAME}(?![A-Za-z0-9_-])`,
    String.raw`docs\\\/${NAME}(?![A-Za-z0-9_-])`,
    // Links pinned to a commit or branch other than main show that tree as it was.
    String.raw`https?://github\.com/ubugeeei-prod/vize/(?:blob|tree|raw|commit|compare)/(?!main/)[^\s)"'\x60\]>]+`,
    // A temporary-directory path is a past run's machine path, never a tracked file.
    String.raw`(?<![\w.-])(?:/private)?/tmp/[^\s"'\x60)\]]+`,
    // A recorded Git commit subject is history: audits pin the subjects as they were written.
    String.raw`"subject": ?"(?:[^"\\\n]|\\.)*"`,
  ].join("|"),
  "gu",
);

const cased = (sample: string, lower: string) =>
  sample === sample.toUpperCase()
    ? lower.toUpperCase()
    : sample[0] === sample[0].toUpperCase()
      ? lower[0].toUpperCase() + lower.slice(1)
      : lower;

/** A level codename at `at` starts a word part: not inside `historicalComplete`. */
function startsPart(text: string, at: number): boolean {
  const before = text[at - 1];
  if (before === undefined || !letter.test(before)) return true;
  return lowerChar.test(before) && !lowerChar.test(text[at]);
}

/** Level codenames found in `text`, at word-part starts. */
export function levelHits(text: string): string[] {
  const hits: string[] = [];
  for (const match of text.matchAll(levelPattern)) {
    if (startsPart(text, match.index)) hits.push(match[0].toLowerCase());
  }
  return hits;
}

function levelReplacement(word: string, at: number, match: string): string {
  const name = levelNames[match.toLowerCase()];
  const whole = match.length === word.length;
  if (match === match.toUpperCase())
    return (name.kebab && word.includes("-") ? name.kebab : name.lower).toUpperCase();
  if (match[0] === match[0].toUpperCase()) return name.pascal;
  if (whole && name.word) return name.word;
  const kebab = word[at - 1] === "-" || word[at + match.length] === "-";
  return kebab && name.kebab ? name.kebab : name.lower;
}

export interface Scope {
  readonly program: boolean;
  readonly levels: boolean;
}
const everything: Scope = { program: true, levels: true };

/**
 * Whole words with a fixed replacement, checked before the generic rules: the deprecated feature
 * aliases name the features they alias, and the dissolved `vize_*` substrate crates name the level
 * crate that took their contents.
 */
const fixedWords: Readonly<Record<string, string>> = {
  [`${NAME}-differential`]: "legacy-differential",
  [`${NAME}-dom-differential`]: "legacy-dom-differential",
  [`vize_${NAME}`]: "vize_l0",
  [`vize_${NAME}_derive`]: "vize_l0_derive",
};

export function renameWord(value: string, scope: Scope = everything): string {
  if (scope.program && Object.hasOwn(fixedWords, value)) return fixedWords[value];
  let out = value;
  if (scope.levels) {
    out = out.replace(levelPattern, (match, at: number) =>
      startsPart(out, at) ? levelReplacement(out, at, match) : match,
    );
  }
  if (!scope.program) return out;
  if (out.toLowerCase() === NAME) return cased(out, "levels");
  const replaced = out.replace(new RegExp(NAME, "giu"), (match) => cased(match, "level"));
  // `davinci-level-x` must not become `level-level-x`.
  return replaced.replace(/(level)[-_]?level/giu, "$1");
}

/** Each maximal word around an occurrence; scanning by index keeps long lines linear. */
function words(text: string, from: number, to: number): { start: number; end: number }[] {
  const found: { start: number; end: number }[] = [];
  anyPattern.lastIndex = from;
  let match: RegExpExecArray | null;
  while ((match = anyPattern.exec(text)) !== null && match.index < to) {
    const at = match.index;
    let start = at,
      end = at + match[0].length;
    while (start > from && wordChar.test(text[start - 1])) start--;
    while (end < to && wordChar.test(text[end])) end++;
    // An ANSI colour (`\x1B[32m`) ends at its `m`; the word starts after it.
    const sgr = /^\d+(?:;\d+)*m/u.exec(text.slice(start, at));
    if (sgr && text[start - 1] === "[") start += sgr[0].length;
    found.push({ start, end });
    anyPattern.lastIndex = end;
  }
  return found;
}

const programTitle = NAME[0].toUpperCase() + NAME.slice(1);
const phraseStart = /(?:^|[\n.!?:;"'`/#*|>-])[ \t]*$/u;

/**
 * The capitalized program name in prose is a modifier ("the X lane", "X ICE machinery"): it
 * becomes `level`, capitalized only where a phrase starts. A plan item ("X P1-7") names the plan.
 */
function renameInProse(text: string, start: number, end: number, scope: Scope): string {
  const word = text.slice(start, end);
  if (!scope.program || word !== programTitle) return renameWord(word, scope);
  const opening = phraseStart.test(text.slice(Math.max(0, start - 80), start));
  const noun = /^ P\d/u.test(text.slice(end, end + 3)) ? "plan" : "level";
  return opening ? noun[0].toUpperCase() + noun.slice(1) : noun;
}

function replaceWords(
  text: string,
  from: number,
  to: number,
  scope: (start: number, end: number) => Scope,
): string {
  let out = "",
    cursor = from;
  for (const { start, end } of words(text, from, to)) {
    out += text.slice(cursor, start);
    out += renameInProse(text, start, end, scope(start, end));
    cursor = end;
  }
  return out + text.slice(cursor, to);
}

/** Captured output moves with its directory; only its bytes are history. */
export function renamePath(relative: string, limit: Scope = everything): string {
  const program = limit.program && !(relative === kept || relative.startsWith(kept + "/"));
  return replaceWords(relative, 0, relative.length, () => ({ program, levels: limit.levels }));
}

function resolvesToKept(file: string, text: string, start: number): boolean {
  const prefix = /(?:\.\.?\/)+$/u.exec(text.slice(Math.max(0, start - 64), start))?.[0];
  if (!prefix) return false;
  return path.posix.normalize(path.posix.join(path.posix.dirname(file), prefix + NAME)) === kept;
}

/** Fenced blocks and inline code spans of a Markdown text: there names mean current code. */
function codeRanges(text: string): [number, number][] {
  const ranges: [number, number][] = [];
  for (const match of text.matchAll(/^(`{3,}|~{3,})[^\n]*\n[\s\S]*?^\1[^\n]*$|`[^`\n]+`/gmu)) {
    ranges.push([match.index, match.index + match[0].length]);
  }
  return ranges;
}

/** Start offsets of the lines that say "codename": the alias form keeps level codenames. */
function aliasLines(text: string): [number, number][] {
  const ranges: [number, number][] = [];
  const lower = text.toLowerCase();
  for (let at = lower.indexOf("codename"); at !== -1; at = lower.indexOf("codename", at + 1)) {
    const start = lower.lastIndexOf("\n", at) + 1;
    const newline = lower.indexOf("\n", at);
    const end = newline === -1 ? lower.length : newline;
    ranges.push([start, end]);
    at = end;
  }
  return ranges;
}

/**
 * Rewrites one file's text. Markdown prose keeps the program name inside docs/davinci/; paths and
 * code spans name current code and follow. In Markdown under docs/ the level codenames stay as
 * aliases and historical names (published packages, old wire headers); only file paths follow
 * their moves. Other data under docs/ names current code and follows fully.
 */
export function rewriteText(
  file: string,
  text: string,
  limit: Scope = everything,
  known?: ReadonlySet<string>,
): string {
  anyPattern.lastIndex = 0;
  if (!anyPattern.test(text)) return text;
  if (keptHistory(file)) {
    return known ? rewriteKeptPaths(file, text, limit, known) : text;
  }
  const markdown = file.endsWith(".md");
  const keptProse = markdown && file.startsWith(kept + "/");
  const docs = markdown && file.startsWith("docs/");
  const code = keptProse ? codeRanges(text) : [];
  const alias = aliasLines(text);
  const within = (ranges: [number, number][], at: number) =>
    ranges.some(([start, end]) => at >= start && at < end);
  // A word next to a path separator, before a file extension or as one (`x-baseline.folio`)
  // names a file.
  const pathAt = (start: number, stop: number) =>
    text[start - 1] === "/" ||
    text[stop] === "/" ||
    fileExtension.test(text.slice(stop, stop + 8)) ||
    (text[start - 1] === "." &&
      wordChar.test(text[start - 2] ?? "") &&
      Object.hasOwn(levelNames, text.slice(start, stop).toLowerCase()));
  const proseAt = (start: number, stop: number) => !pathAt(start, stop) && !within(code, start);
  let out = "",
    cursor = 0;
  const free = (end: number) => {
    out += replaceWords(text, cursor, end, (start, stop) => ({
      program:
        limit.program && !resolvesToKept(file, text, start) && !(keptProse && proseAt(start, stop)),
      levels: limit.levels && !within(alias, start) && (!docs || pathAt(start, stop)),
    }));
  };
  for (const span of text.matchAll(protectedSpans)) {
    free(span.index);
    out += span[0];
    cursor = span.index + span[0].length;
  }
  free(text.length);
  return out;
}

/**
 * Plan data under the kept docs that tools read against current code (inventories, budgets,
 * allowlists, schemas) follows the rename fully. Fix histories and records pin past revisions.
 */
const liveKeptData = (file: string): boolean =>
  file.startsWith(kept + "/plan/") &&
  !file.endsWith(".md") &&
  !/fix-history|-records?\//u.test(file);

/** A kept-docs file whose names stay as written, apart from references to moved paths. */
export const keptHistory = (file: string): boolean =>
  file.startsWith(kept + "/") && !liveKeptData(file);

/** Tracked files and every directory that holds one, for path references. */
export function knownPaths(files: readonly string[]): Set<string> {
  const known = new Set<string>();
  for (const file of files) {
    known.add(file);
    for (let at = file.indexOf("/"); at !== -1; at = file.indexOf("/", at + 1)) {
      known.add(file.slice(0, at));
    }
  }
  return known;
}

const pathToken = /(?<![A-Za-z0-9_.\/-])(?:\.{1,2}\/)*[A-Za-z0-9_.-]+(?:\/[A-Za-z0-9_.-]+)*\/?/gu;

/**
 * The kept docs are history: their prose and names stay. Only a reference to a tracked path that
 * moves follows it, so links keep resolving; a path that no longer exists stays as written.
 */
function rewriteKeptPaths(
  file: string,
  text: string,
  limit: Scope,
  known: ReadonlySet<string>,
): string {
  const dir = path.posix.dirname(file);
  return text.replace(pathToken, (token: string) => {
    anyPattern.lastIndex = 0;
    if (!anyPattern.test(token)) return token;
    const body = token.replace(/[.]+$/u, "");
    const trail = token.slice(body.length);
    const bare = body.replace(/\/$/u, "");
    // A file name names its sibling in the same directory (`[x.md](./x.md)`).
    const sibling = !bare.includes("/") && bare.includes(".");
    if (!bare.includes("/") && !sibling) return token;
    const relative = sibling || /^\.{1,2}\//u.test(bare);
    const target = relative ? path.posix.normalize(path.posix.join(dir, bare)) : bare;
    // A bare top-level directory names the program's place in history, not a file to reach.
    if (target.startsWith("../") || !target.includes("/")) return token;
    const moved = renamePath(target, limit);
    if (moved === target || !(known.has(target) || known.has(moved))) return token;
    if (!relative) return renamePath(body, limit) + trail;
    // A relative link keeps its spelling; only its words that the move renames change.
    const into = target === kept || target.startsWith(kept + "/");
    const scope = { program: limit.program && !into, levels: limit.levels };
    return replaceWords(body, 0, body.length, () => scope) + trail;
  });
}

interface Tracked {
  readonly path: string;
  readonly mode: string;
}

function tracked(cwd: string): Tracked[] {
  return execFileSync("git", ["ls-files", "-s", "-z"], {
    cwd,
    encoding: "utf8",
    maxBuffer: 1 << 28,
  })
    .split("\0")
    .filter(Boolean)
    .map((line) => {
      const [meta, file] = line.split("\t");
      return { path: file, mode: meta.split(" ")[0] };
    });
}

export function plannedMoves(paths: readonly string[]): Map<string, string> {
  const moves = new Map<string, string>();
  const targets = new Map<string, string>();
  const existing = new Set(paths);
  for (const file of paths) {
    const target = renamePath(file);
    if (target === file) continue;
    const clash = targets.get(target) ?? (existing.has(target) ? target : undefined);
    if (clash) throw new Error(`${file} and ${clash} both map to ${target}`);
    targets.set(target, file);
    moves.set(file, target);
  }
  return moves;
}

function removeEmptyParents(cwd: string, file: string): void {
  for (let dir = path.dirname(file); dir !== "."; dir = path.dirname(dir)) {
    const full = path.join(cwd, dir);
    if (!fs.existsSync(full) || fs.readdirSync(full).length > 0) return;
    fs.rmdirSync(full);
  }
}

const textFile = (bytes: Buffer) => !bytes.subarray(0, 8000).includes(0);

/** A ledger generator may replace a tracked file under a new name; the old one is gone. */
function contentTargets(cwd: string): string[] {
  return tracked(cwd)
    .filter(({ mode }) => mode === "100644" || mode === "100755")
    .map(({ path: file }) => file)
    .filter((file) => file !== self && !gate.test(file) && !keepsBytes(file))
    .filter((file) => fs.existsSync(path.join(cwd, file)));
}

export function run(mode: string, cwd = root): string[] {
  if (mode === "--move") {
    const moves = plannedMoves(tracked(cwd).map(({ path: file }) => file));
    for (const [from, to] of moves) {
      fs.mkdirSync(path.join(cwd, path.dirname(to)), { recursive: true });
      fs.renameSync(path.join(cwd, from), path.join(cwd, to));
      removeEmptyParents(cwd, from);
    }
    // One index update for every move: `git mv` per file rewrites the whole index each time.
    const pathspecs = [...moves].flat();
    if (pathspecs.length > 0) {
      execFileSync("git", ["add", "-A", "--pathspec-from-file=-", "--pathspec-file-nul"], {
        cwd,
        input: pathspecs.map((file) => `:(literal)${file}`).join("\0"),
      });
    }
    return [...moves.keys()];
  }
  if (mode !== "--check" && mode !== "--rewrite") {
    throw new Error("usage: rename-codenames.ts --check|--move|--rewrite");
  }
  const stale: string[] = [];
  const files = tracked(cwd).map(({ path: file }) => file);
  const moves = plannedMoves(files);
  const known = knownPaths(files);
  stale.push(...moves.keys());
  for (const file of contentTargets(cwd)) {
    const full = path.join(cwd, file);
    const bytes = fs.readFileSync(full);
    if (!textFile(bytes)) continue;
    const text = bytes.toString("utf8");
    const next = rewriteText(file, text, everything, known);
    if (next === text) continue;
    stale.push(file);
    if (mode === "--rewrite") fs.writeFileSync(full, next);
  }
  if (mode === "--rewrite") {
    format(
      cwd,
      stale.filter((file) => !moves.has(file)),
    );
    // Ledgers whose rows or file names derive from moved paths are regenerated, not edited.
    for (const generator of generators) {
      execFileSync(process.execPath, [generator, "--write"], { cwd, stdio: "ignore" });
    }
    // Renamed packages move in the lockfile's name order; let cargo re-sort it.
    execFileSync("cargo", ["metadata", "--format-version", "1", "--offline"], {
      cwd,
      stdio: "ignore",
    });
    stale.push(...repin(cwd));
  }
  return [...new Set(stale)];
}

const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const gitBlob = (bytes: Buffer) =>
  createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");

/**
 * Holders whose digests pin the current bytes of a file the rewrite changes. Most digests in the
 * tree pin bytes of a past revision that only happen to equal main's (qualification records,
 * body custody, frozen predecessors); re-pinning those breaks them, so a holder is listed here
 * only when its test reads the current file.
 */
const currentPins =
  /^(?:docs\/davinci\/plan\/compiler-fix-history-index\.json|tools\/benchmarks\/scripts\/typechecker-native-source-recipe-fixture\.mjs|tests\/_fixtures\/tooling\/native-source-recipe-436\.md|tests\/_fixtures\/differential\/compiler\/manifest\.json|tools\/support\/compat\/github\/vapor-style-api-fixtures\.ts)$/u;

/**
 * Re-records the whole-file digests (SHA-256 and git blob ids) that name a file the rewrite
 * changed, in the holders listed in `currentPins`. A record whose pins change changes its own
 * digest, so this repeats until nothing moves.
 */
function repin(cwd: string): string[] {
  const pinned = new Set<string>();
  const holders = contentTargets(cwd).filter((file) => currentPins.test(file));
  const last = new Map<string, readonly string[]>();
  for (let round = 0; round < 8; round++) {
    const replacements = new Map<string, string>();
    const changed = execFileSync("git", ["diff", "--name-only", "-z"], {
      cwd,
      encoding: "utf8",
      maxBuffer: 1 << 28,
    })
      .split("\0")
      .filter(Boolean);
    for (const file of changed) {
      const full = path.join(cwd, file);
      if (!fs.existsSync(full)) continue;
      const recorded =
        last.get(file) ??
        ((bytes) => [sha256(bytes), gitBlob(bytes)])(
          execFileSync("git", ["show", `:${file}`], { cwd, maxBuffer: 1 << 30 }),
        );
      const bytes = fs.readFileSync(full);
      const current = [sha256(bytes), gitBlob(bytes)];
      current.forEach((to, index) => {
        if (recorded[index] !== to) replacements.set(recorded[index], to);
      });
      last.set(file, current);
    }
    if (replacements.size === 0) break;
    const digest = new RegExp([...replacements.keys()].join("|"), "gu");
    for (const file of holders) {
      const full = path.join(cwd, file);
      const bytes = fs.readFileSync(full);
      if (!textFile(bytes)) continue;
      const text = bytes.toString("utf8");
      digest.lastIndex = 0;
      if (!digest.test(text)) continue;
      fs.writeFileSync(
        full,
        text.replace(digest, (from) => replacements.get(from) ?? from),
      );
      pinned.add(file);
    }
  }
  return [...pinned];
}

const generators = [
  "tools/support/compat/levels/v-on-corpus.ts",
  "tools/support/compat/levels/croquis-consumers.ts",
  "tools/support/compat/levels/sourcelocation-inventory.ts",
  "tools/support/compat/levels/consumer-migration-surfaces.ts",
];

/** Renamed spellings change line widths; reflow only the rewritten files. */
function format(cwd: string, files: string[]): void {
  const chunks = (list: string[]) =>
    Array.from({ length: Math.ceil(list.length / 200) }, (_, i) =>
      list.slice(i * 200, i * 200 + 200),
    );
  const rust = files.filter((file) => file.endsWith(".rs"));
  for (const chunk of chunks(rust)) {
    execFileSync(process.env.RUSTFMT ?? "rustfmt", ["--edition", "2024", ...chunk], { cwd });
  }
  const web = files.filter((file) => /\.(?:css|json|md|[cm]?[jt]sx?|toml|vue|ya?ml)$/u.test(file));
  for (const chunk of chunks(web)) {
    execFileSync(path.join(cwd, "node_modules/.bin/vp"), ["fmt", ...chunk], {
      cwd,
      stdio: "ignore",
    });
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const mode = process.argv[2] ?? "";
  const stale = run(mode);
  process.stdout.write(`${stale.length} paths ${mode === "--check" ? "still named" : "changed"}\n`);
  if (mode === "--check" && stale.length > 0) process.exitCode = 1;
}
