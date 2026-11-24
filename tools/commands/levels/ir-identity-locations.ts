// IR identity with panic locations normalized, for comment-, whitespace- and
// split-only changes (maintainer decision, 2026-11-24).
//
//   node tools/commands/levels/ir-identity-locations.ts --base <rev> [--head <rev>]
//     [--crate <package>]... [--ignore-location-files]
//     [--rename-path <base::path>=<head::path>]... [--rename <ident>=<ident>]...
//     [--target-dir <dir>] [--work <dir>] [--keep] [--json] [--diff <lines>] [--save <dir>]
//   node tools/commands/levels/ir-identity-locations.ts --ir <base.ll> <head.ll> [options]
//
// rustc materializes a `core::panic::Location { file, line, col }` constant for
// every `#[track_caller]` call and compiler-inserted panic. Deleting a comment
// line shifts the line of every later site, so the IR differs in those
// constants although no instruction changes. This wrapper rewrites each
// Location's line and column to zero in both modules, then runs the
// `ir-identity.ts` comparison (release profile, names and paths normalized).
//
// `--ignore-location-files` also blanks each Location's file, for splits and
// moves: code that moves to a new file points at a new path string. The report
// then gives the path-string bytes on each side, which is the only release-size
// growth a split may cause (about 30-60 bytes per new file with panic sites).
//
// Release-profile builds are heavy: hold the fleet release lock and check free
// disk first, as the fleet brief describes.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";

import { isEntrypoint } from "../../support/command-runtime.ts";
import { isIdentical } from "../../support/levels/ir-normalize.ts";
import {
  comparePackage,
  emitIr,
  relocator,
  report,
  touchedPackages,
  type PackageResult,
} from "./ir-identity.ts";

export interface LocationSummary {
  /** Location constants rewritten. */
  readonly sites: number;
  /** Distinct `.rs` path strings the Locations point at, with their byte sizes. */
  readonly files: ReadonlyMap<string, number>;
}

const pathString =
  /^@("?)([^"\s]+)\1 = [^=]*?constant \[(\d+) x i8\] c"((?:[^"\\]|\\[0-9A-Fa-f]{2}|\\\\)*)"/u;
const location =
  /(<?\{ ptr )@("?)([^"\s,]+)\2(, \[16 x i8\] c")((?:[^"\\]|\\[0-9A-Fa-f]{2}|\\\\)*)(" \}>?)/gu;

/** LLVM spells non-printable bytes of a `c"…"` string as `\XX` escapes. */
export function decodeLlvmString(body: string): number[] {
  const bytes: number[] = [];
  for (let at = 0; at < body.length;) {
    if (body[at] === "\\") {
      if (body[at + 1] === "\\") {
        bytes.push(92);
        at += 2;
      } else {
        bytes.push(Number.parseInt(body.slice(at + 1, at + 3), 16));
        at += 3;
      }
    } else bytes.push(body.charCodeAt(at++));
  }
  return bytes;
}

function encodeBytes(bytes: readonly number[]): string {
  return bytes.map((byte) => `\\${byte.toString(16).toUpperCase().padStart(2, "0")}`).join("");
}

/**
 * Rewrite every panic Location in one `.ll` module: line and column become 0,
 * and with `files` the path pointer and length become placeholders too.
 */
export function normalizeLocations(
  ir: string,
  options: { readonly files?: boolean } = {},
): { readonly ir: string; readonly summary: LocationSummary } {
  const paths = new Map<string, { text: string; size: number }>();
  for (const line of ir.split("\n")) {
    const match = pathString.exec(line);
    if (!match) continue;
    const bytes = decodeLlvmString(match[4]);
    const text = String.fromCharCode(...bytes.filter((byte) => byte !== 0));
    if (text.endsWith(".rs")) paths.set(match[2], { text, size: Number(match[3]) });
  }
  let sites = 0;
  const files = new Map<string, number>();
  const rewritten = ir.replace(
    location,
    (
      whole,
      open: string,
      _quote: string,
      target: string,
      middle: string,
      body: string,
      close: string,
    ) => {
      const file = paths.get(target);
      const bytes = decodeLlvmString(body);
      if (!file || bytes.length !== 16) return whole;
      // Bytes 0-7 are the path length (little endian), 8-11 the line, 12-15 the column.
      const length = bytes
        .slice(0, 8)
        .reduce((sum, byte, index) => sum + byte * 2 ** (8 * index), 0);
      if (length !== file.text.length && length !== file.text.length + 1) return whole;
      sites++;
      files.set(file.text, file.size);
      const kept = options.files ? Array(8).fill(0) : bytes.slice(0, 8);
      const pointer = options.files ? "@{location-file}" : `@${target}`;
      return `${open}${pointer}${middle}${encodeBytes([...kept, ...Array(8).fill(0)])}${close}`;
    },
  );
  return { ir: rewritten, summary: { sites, files } };
}

export interface LocationResult extends PackageResult {
  readonly locations: {
    readonly base: { sites: number; files: number; pathBytes: number };
    readonly head: { sites: number; files: number; pathBytes: number };
  };
}

const sizeOf = (summary: LocationSummary) => ({
  sites: summary.sites,
  files: summary.files.size,
  pathBytes: [...summary.files.values()].reduce((total, size) => total + size, 0),
});

export function compareWithLocations(
  pkg: string,
  baseIr: string,
  headIr: string,
  options: {
    readonly files?: boolean;
    readonly move?: Parameters<typeof comparePackage>[3];
    readonly diff?: number;
  } = {},
): LocationResult {
  const base = normalizeLocations(baseIr, options);
  const head = normalizeLocations(headIr, options);
  const result = comparePackage(
    pkg,
    base.ir,
    head.ir,
    options.move ?? ((name) => name),
    options.diff ?? 0,
  );
  return {
    ...result,
    locations: { base: sizeOf(base.summary), head: sizeOf(head.summary) },
  };
}

export function locationReport(results: readonly LocationResult[]): string {
  const lines = [report(results)];
  for (const { package: pkg, locations } of results) {
    const delta = locations.head.pathBytes - locations.base.pathBytes;
    lines.push(
      `  ${pkg}: ${locations.base.sites}/${locations.head.sites} panic locations normalized (base/head); ` +
        `${locations.base.files}/${locations.head.files} location files, ` +
        `${locations.base.pathBytes}/${locations.head.pathBytes} path-string bytes (${delta >= 0 ? "+" : ""}${delta})`,
    );
  }
  return lines.join("\n");
}

interface Options {
  base: string;
  head: string;
  crates: string[];
  paths: [string, string][];
  identifiers: Map<string, string>;
  targetDir: string;
  work: string;
  save: string;
  ir: string[];
  diff: number;
  keep: boolean;
  json: boolean;
  files: boolean;
}

function parseOptions(argv: readonly string[]): Options {
  const options: Options = {
    base: "",
    head: "",
    crates: [],
    paths: [],
    identifiers: new Map(),
    targetDir: "",
    work: "",
    save: "",
    ir: [],
    diff: 0,
    keep: false,
    json: false,
    files: false,
  };
  for (let index = 0; index < argv.length; index++) {
    const flag = argv[index];
    const value = () => {
      const next = argv[++index];
      if (next === undefined) throw new Error(`${flag} needs a value`);
      return next;
    };
    const pair = (text: string): [string, string] => {
      const at = text.indexOf("=");
      if (at <= 0) throw new Error(`${flag} expects <from>=<to>`);
      return [text.slice(0, at), text.slice(at + 1)];
    };
    if (flag === "--base") options.base = value();
    else if (flag === "--head") options.head = value();
    else if (flag === "--crate") options.crates.push(value());
    else if (flag === "--rename-path") options.paths.push(pair(value()));
    else if (flag === "--rename") options.identifiers.set(...pair(value()));
    else if (flag === "--target-dir") options.targetDir = value();
    else if (flag === "--work") options.work = value();
    else if (flag === "--save") options.save = value();
    else if (flag === "--ir") options.ir.push(value(), value());
    else if (flag === "--diff") options.diff = Number(value());
    else if (flag === "--keep") options.keep = true;
    else if (flag === "--json") options.json = true;
    else if (flag === "--ignore-location-files") options.files = true;
    else throw new Error(`unknown flag ${flag}`);
  }
  if (!options.base && !options.ir.length)
    throw new Error("--base <rev> or --ir <base.ll> <head.ll> is required");
  return options;
}

const git = (cwd: string, ...args: string[]) =>
  execFileSync("git", args, { cwd, encoding: "utf8", maxBuffer: 1 << 28 }).trim();

function main(argv: readonly string[]): number {
  const options = parseOptions(argv);
  const move = relocator({ steps: [], paths: options.paths, identifiers: options.identifiers });
  const compare = (pkg: string, baseIr: string, headIr: string) =>
    compareWithLocations(pkg, baseIr, headIr, { files: options.files, move, diff: options.diff });
  let results: LocationResult[];
  if (options.ir.length) {
    const [baseIr, headIr] = options.ir.map((file) => fs.readFileSync(file, "utf8"));
    results = [compare(options.crates[0] ?? path.basename(options.ir[1], ".ll"), baseIr, headIr)];
  } else {
    const root = process.cwd();
    const work = options.work || fs.mkdtempSync(path.join(os.tmpdir(), "vize-ir-locations-"));
    const targetDir = options.targetDir || path.join(work, "target");
    const checkouts: string[] = [];
    const checkout = (rev: string, label: string) => {
      const dir = path.join(work, label);
      if (!fs.existsSync(dir)) git(root, "worktree", "add", "--detach", dir, rev);
      else git(dir, "checkout", "--quiet", "--detach", rev);
      checkouts.push(dir);
      return dir;
    };
    try {
      const baseDir = checkout(options.base, "base");
      const headDir = options.head ? checkout(options.head, "head") : root;
      const packages = options.crates.length
        ? options.crates
        : touchedPackages(root, options.base, options.head || "HEAD");
      results = packages.map((pkg) => {
        const baseIr = emitIr(baseDir, pkg, targetDir);
        const headIr = emitIr(headDir, pkg, targetDir);
        if (options.save) {
          fs.mkdirSync(options.save, { recursive: true });
          fs.writeFileSync(path.join(options.save, `${pkg}.base.ll`), baseIr);
          fs.writeFileSync(path.join(options.save, `${pkg}.head.ll`), headIr);
        }
        return compare(pkg, baseIr, headIr);
      });
    } finally {
      if (!options.keep) {
        for (const dir of checkouts) git(root, "worktree", "remove", "--force", dir);
        if (!options.work) fs.rmSync(work, { recursive: true, force: true });
      }
    }
  }
  console.log(options.json ? JSON.stringify(results, null, 2) : locationReport(results));
  return results.every(isIdentical) ? 0 : 1;
}

if (isEntrypoint(import.meta.url)) {
  try {
    process.exitCode = main(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
