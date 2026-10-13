// Accept only the known cross-crate tokenizer re-export false positives.
//
// cargo-semver-checks 0.47.0 cannot resolve items relocated to a dependency and
// re-exported from their old paths (upstream issue #355). The paired downstream
// consumer fixture must pass against both 0.429.1 and the candidate before
// this allowlist is used. Every other SemVer diagnostic remains fatal.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

type Counter = ReadonlyMap<string, number>;
type Entries = ReadonlyMap<string, Counter>;

const summary = "Summary semver requires new major version: 6 major and 0 minor checks failed";

function count(values: readonly string[]): Map<string, number> {
  const counter = new Map<string, number>();
  for (const value of values) counter.set(value, (counter.get(value) ?? 0) + 1);
  return counter;
}

export function expectedEntries(): Entries {
  const enums = ["State", "QuoteType"];
  const funcs = ["is_whitespace", "is_tag_start_char", "is_end_of_tag_section"];
  const codes = [
    "TAB",
    "NEWLINE",
    "FORM_FEED",
    "CARRIAGE_RETURN",
    "SPACE",
    "EXCLAMATION_MARK",
    "DOUBLE_QUOTE",
    "NUMBER",
    "AMP",
    "SINGLE_QUOTE",
    "DASH",
    "DOT",
    "SLASH",
    "ZERO",
    "NINE",
    "COLON",
    "SEMI",
    "LT",
    "EQ",
    "GT",
    "QUESTION_MARK",
    "AT",
    "UPPER_A",
    "UPPER_F",
    "UPPER_Z",
    "LEFT_SQUARE",
    "RIGHT_SQUARE",
    "GRAVE_ACCENT",
    "LOWER_A",
    "LOWER_F",
    "LOWER_V",
    "LOWER_X",
    "LOWER_Z",
    "LEFT_BRACE",
    "RIGHT_BRACE",
  ];
  // Every item is reported once at its old module path and once at the root.
  const both = (kind: string, names: readonly string[]): Counter =>
    count(
      names.flatMap((name) =>
        ["tokenizer::", ""].map((prefix) => `${kind} vize_armature::${prefix}${name}`),
      ),
    );
  return new Map([
    ["enum_missing", both("enum", enums)],
    ["function_missing", both("function", funcs)],
    ["module_missing", both("mod", ["char_codes"])],
    ["pub_module_level_const_missing", new Map(codes.map((name) => [name, 2]))],
    ["struct_missing", both("struct", ["Tokenizer"])],
    ["trait_missing", both("trait", ["Callbacks"])],
  ]);
}

function splitOnce(text: string, separator: string): readonly [string, string | undefined] {
  const index = text.indexOf(separator);
  if (index < 0) return [text, undefined];
  return [text.slice(0, index), text.slice(index + separator.length)];
}

function lines(text: string): readonly string[] {
  const split = text.split("\n");
  if (split.at(-1) === "") split.pop();
  return split;
}

/** The whole diagnostic line must match, as the reviewed allowlist requires. */
function capture(pattern: RegExp, line: string): string | undefined {
  return pattern.exec(line)?.[1];
}

const constDiagnostic = /^([A-Z_]+) in file .*\/src\/tokenizer\/char_codes\.rs:\d+$/u;
const moduleDiagnostic = /^(.+), previously in file .*\/src\/tokenizer\/char_codes\.rs:\d+$/u;
const itemDiagnostic = /^(.+), previously in file .*\/src\/tokenizer(?:\/types)?\.rs:\d+$/u;

export function reportedEntries(log: string): Entries {
  // A failure section runs to the next section, the summary or the end of the log.
  const failures = log.matchAll(
    /^--- failure ([a-z_]+):[^\n]*\n([\s\S]*?)(?=^--- failure |^\s+Summary |(?![\s\S]))/gmu,
  );
  const actual = new Map<string, Counter>();
  for (const [, lint, section] of failures) {
    const [, afterFailedIn] = splitOnce(section, "Failed in:\n");
    if (afterFailedIn === undefined || actual.has(lint)) {
      throw new Error(`missing evidence or duplicate lint: ${lint}`);
    }
    const [entries] = splitOnce(afterFailedIn, "\n\n");
    const parsed: string[] = [];
    for (const raw of lines(entries)) {
      if (!raw.startsWith("  ")) throw new Error(`unexpected diagnostic line: ${raw}`);
      const line = raw.trim();
      if (lint === "pub_module_level_const_missing") {
        const name = capture(constDiagnostic, line);
        if (name === undefined) throw new Error(`unexpected const diagnostic: ${line}`);
        parsed.push(name);
      } else {
        const pattern = lint === "module_missing" ? moduleDiagnostic : itemDiagnostic;
        const item = capture(pattern, line);
        if (item === undefined) throw new Error(`unexpected item diagnostic: ${line}`);
        parsed.push(item);
      }
    }
    actual.set(lint, count(parsed));
  }
  return actual;
}

/** Entries of `left` that exceed their count in `right`. */
function excess(left: Counter | undefined, right: Counter | undefined): Map<string, number> {
  const result = new Map<string, number>();
  for (const [name, total] of left ?? []) {
    const remaining = total - (right?.get(name) ?? 0);
    if (remaining > 0) result.set(name, remaining);
  }
  return result;
}

function describe(counter: ReadonlyMap<string, number>): string {
  return JSON.stringify(Object.fromEntries(counter));
}

export interface Verdict {
  readonly status: 0 | 1;
  readonly stdout: readonly string[];
  readonly stderr: readonly string[];
}

export function verdict(rawLog: string): Verdict {
  const refuse = (...stderr: string[]): Verdict => ({ status: 1, stdout: [], stderr });
  // eslint-disable-next-line no-control-regex -- strips the terminal colors cargo emits.
  const colorless = rawLog.replace(/\x1b\[[0-9;]*m/gu, "");
  // The log is read as text: a Windows or classic Mac line ending is a newline.
  const log = colorless.replace(/\r\n?/gu, "\n");
  if (!log.includes(summary)) return refuse("unexpected SemVer summary");
  if (log.includes("\nerror:") || log.includes("\npanic")) {
    return refuse("SemVer command also reported a tool/build error");
  }
  let actual: Entries;
  try {
    actual = reportedEntries(log);
  } catch (error) {
    return refuse(error instanceof Error ? error.message : String(error));
  }
  const expected = expectedEntries();
  const lints = [...new Set([...actual.keys(), ...expected.keys()])].sort();
  const differences: string[] = [];
  for (const lint of lints) {
    const unexpected = excess(actual.get(lint), expected.get(lint));
    const missing = excess(expected.get(lint), actual.get(lint));
    if (unexpected.size > 0 || missing.size > 0) {
      differences.push(`${lint}: unexpected=${describe(unexpected)}, missing=${describe(missing)}`);
    }
  }
  // A lint reported with no entries still differs from the reviewed set.
  const sameLints = actual.size === expected.size && lints.length === expected.size;
  if (differences.length > 0 || !sameLints) return refuse(...differences);
  return {
    status: 0,
    stdout: [
      "Only the documented vize_armature tokenizer cross-crate re-export false positives remain.",
    ],
    stderr: [],
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const logPath = process.argv[2];
  if (logPath === undefined) throw new Error("usage: allow-armature-tokenizer-reexports.ts LOG");
  const result = verdict(readFileSync(logPath, "utf8"));
  for (const line of result.stdout) console.log(line);
  for (const line of result.stderr) console.error(line);
  process.exit(result.status);
}
