// Shared filesystem, environment and string helpers for the release and
// publishing commands that were ported from the MoonBit `tools/moon/lib`
// helper. Each helper keeps the observable semantics the MoonBit runtime had
// (path normalization, raw directory order, integer parsing, trimming) so the
// TypeScript commands spawn the same processes and write the same bytes.
import fs from "node:fs";
import path from "node:path";

import { ioErrorMessage, writeStderr } from "./command_io.ts";

/** Returns command-line arguments after the script path. */
export function cliArgs(): string[] {
  return process.argv.slice(2);
}

/** Returns a non-empty environment value, mirroring `Some(value) if value != ""`. */
export function nonEmptyEnv(name: string): string | undefined {
  const value = process.env[name];
  return value === undefined || value === "" ? undefined : value;
}

/** Resolves a binary name from the environment, falling back to a default. */
export function toolBin(envName: string, fallback: string): string {
  return nonEmptyEnv(envName) ?? fallback;
}

/** Resolves a path against the current directory and normalizes it. */
export function resolvePath(value: string): string {
  return path.resolve(value);
}

/** Joins two path fragments and normalizes the result. */
export function joinPath(base: string, child: string): string {
  return path.join(base, child);
}

/** Returns the parent directory using POSIX dirname rules. */
export function dirnamePath(value: string): string {
  return path.dirname(value);
}

/** Returns whether `stat` succeeds for a path (symlinks are followed). */
export function pathExists(target: string): boolean {
  try {
    fs.statSync(target);
    return true;
  } catch {
    return false;
  }
}

/** Returns whether a path is a directory, treating IO errors as false. */
export function isDir(target: string): boolean {
  try {
    return fs.statSync(target).isDirectory();
  } catch {
    return false;
  }
}

/**
 * Reads directory entry names in raw `readdir` order, without sorting, and
 * returns an empty list when enumeration fails. Like the original C helper, a
 * failed `opendir` is reported on standard error as `perror("opendir")` did.
 */
export function readDir(target: string): string[] {
  let directory: fs.Dir;
  try {
    directory = fs.opendirSync(target);
  } catch (error) {
    writeStderr(`opendir: ${ioErrorMessage(error)}\n`);
    return [];
  }
  const names: string[] = [];
  try {
    for (let entry = directory.readSync(); entry !== null; entry = directory.readSync()) {
      names.push(entry.name);
    }
  } catch {
    return [];
  } finally {
    directory.closeSync();
  }
  return names;
}

/** Reads a UTF-8 file and collapses IO failures into `null`. */
export function readFileToString(target: string): string | null {
  try {
    return fs.readFileSync(target, "utf8");
  } catch {
    return null;
  }
}

/** Writes a UTF-8 file, throwing on failure. */
export function writeStringToFile(target: string, content: string): void {
  fs.writeFileSync(target, content);
}

/** Writes a UTF-8 file and converts write failures into a boolean result. */
export function tryWriteStringToFile(target: string, content: string): boolean {
  try {
    writeStringToFile(target, content);
    return true;
  } catch {
    return false;
  }
}

const integerDigit = (character: string): number => {
  const code = character.charCodeAt(0);
  if (code >= 48 && code <= 57) return code - 48;
  if (code >= 97 && code <= 122) return code - 97 + 10;
  if (code >= 65 && code <= 90) return code - 65 + 10;
  return -1;
};

const INT32_MIN = -(2n ** 31n);
const INT32_MAX = 2n ** 31n - 1n;

/**
 * Parses an integer with the MoonBit `@string.parse_int` grammar: an optional
 * sign, an optional `0x`/`0o`/`0b` prefix, digits with single underscores
 * between digits, and a 32-bit result. Returns `null` for invalid input.
 */
export function parseMoonInt(value: string): number | null {
  if (value === "") return null;
  let rest = value;
  let negative = false;
  if (rest.startsWith("+")) {
    rest = rest.slice(1);
  } else if (rest.startsWith("-")) {
    negative = true;
    rest = rest.slice(1);
  }
  let base = 10;
  let allowUnderscore = false;
  if (/^0[xX]/.test(rest)) {
    base = 16;
    rest = rest.slice(2);
    allowUnderscore = true;
  } else if (/^0[oO]/.test(rest)) {
    base = 8;
    rest = rest.slice(2);
    allowUnderscore = true;
  } else if (/^0[bB]/.test(rest)) {
    base = 2;
    rest = rest.slice(2);
    allowUnderscore = true;
  }
  if (!/^(?:[0-9a-zA-Z]|_[0-9a-zA-Z])/.test(rest)) return null;
  let accumulator = 0n;
  const characters = Array.from(rest);
  for (let index = 0; index < characters.length; index++) {
    const character = characters[index];
    if (character === "_") {
      if (index === characters.length - 1 || !allowUnderscore) return null;
      allowUnderscore = false;
      continue;
    }
    const digit = integerDigit(character);
    if (digit < 0 || digit >= base) return null;
    // Saturate past the 32-bit range: MoonBit rejects both 64-bit overflow and
    // 32-bit range errors, so the exact magnitude no longer matters.
    if (accumulator <= INT32_MAX + 1n) {
      accumulator = accumulator * BigInt(base) + BigInt(digit);
    }
    allowUnderscore = true;
  }
  const signed = negative ? -accumulator : accumulator;
  if (signed < INT32_MIN || signed > INT32_MAX) return null;
  return Number(signed);
}

const TRIM_CHARACTERS = new Set(["\t", "\n", "\r", " "]);

/** Trims tab, newline, carriage return and space from both ends. */
export function moonTrim(value: string): string {
  let start = 0;
  let end = value.length;
  while (start < end && TRIM_CHARACTERS.has(value[start])) start++;
  while (end > start && TRIM_CHARACTERS.has(value[end - 1])) end--;
  return value.slice(start, end);
}

/** Replaces the first occurrence of `search` without replacement patterns. */
export function replaceFirst(text: string, search: string, replacement: string): string {
  const index = text.indexOf(search);
  if (index === -1) return text;
  return text.slice(0, index) + replacement + text.slice(index + search.length);
}

/** Replaces every occurrence of `search` without replacement patterns. */
export function replaceEvery(text: string, search: string, replacement: string): string {
  if (search === "") {
    return replacement + Array.from(text, (character) => character + replacement).join("");
  }
  return text.split(search).join(replacement);
}

/** Lowercases ASCII letters only, leaving every other character unchanged. */
export function asciiLowercase(text: string): string {
  return text.replace(/[A-Z]/g, (character) => character.toLowerCase());
}

/** Compares strings by UTF-16 code units. */
export function lexicalCompare(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
