// IR identity for move- and rename-only changes (maintainer rule, 2026-11-19):
// such a change passes when its optimized LLVM IR equals the base's after
// symbol names and paths are normalized, its output is byte-identical and its
// release binary does not grow.
//
//   node tools/commands/levels/ir-identity.ts --base <rev> [--head <rev>]
//     [--crate <package>]... [--step <level-layout-move step>]...
//     [--rename-path <base::path>=<head::path>]... [--rename <ident>=<ident>]...
//     [--target-dir <dir>] [--work <dir>] [--keep] [--json]
//     [--diff <lines>] [--save <dir>]
//   node tools/commands/levels/ir-identity.ts --ir <base.ll> <head.ll> [rewrites]
//
// `--save` keeps each package's base and head IR (`<pkg>.base.ll`,
// `<pkg>.head.ll`) so `--ir` can recheck them without building; `--diff`
// prints the first differing normalized lines of each changed function.
//
// Each revision is built with the release profile (opt-level 3, one codegen
// unit) and `--emit=llvm-ir` for every listed package (default: the level and
// product packages whose `src/` the change touches). The base is checked out in
// a detached worktree; the head is the current checkout unless `--head` names a
// revision. Base paths are relocated to the head's spelling through the moves
// and identifier renames of the named `level-layout-move.ts` steps plus explicit
// `--rename-path`/`--rename` pairs, then every function is compared by its
// normalized body. Exit status 1 means a function was added, removed or changed.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { relocate, steps, type Step } from "../davinci/level-layout-move.ts";
import {
  bodyDiff,
  compareModules,
  isIdentical,
  normalizeModule,
  type ModuleDifference,
  type Relocate,
} from "../../support/levels/ir-normalize.ts";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

export interface PathRewrites {
  readonly steps: readonly Step[];
  readonly paths: readonly (readonly [string, string])[];
  readonly identifiers: ReadonlyMap<string, string>;
}

/** Rewrite every `crate::a::b` path of a demangled name from base to head. */
export function relocator(rewrites: PathRewrites): Relocate {
  const byCrate = new Map<string, Step[]>();
  for (const step of rewrites.steps) {
    const list = byCrate.get(step.name) ?? [];
    list.push(step);
    byCrate.set(step.name, list);
  }
  const identifiers = new Map(rewrites.identifiers);
  for (const step of rewrites.steps)
    for (const [from, to] of Object.entries(step.identifiers ?? {})) identifiers.set(from, to);
  const ident = "[A-Za-z_][A-Za-z0-9_]*";
  return (name) => {
    let out = name;
    for (const [crate, crateSteps] of byCrate)
      out = out.replace(
        new RegExp(`\\b${crate}((?:::${ident})+)`, "gu"),
        (_whole, tail: string) => {
          let segments = tail.slice(2).split("::");
          for (const step of crateSteps) segments = relocate(step.moves, segments);
          return [crate, ...segments].join("::");
        },
      );
    for (const [from, to] of rewrites.paths)
      out = out.replace(
        new RegExp(`${from.replaceAll(/[.*+?^${}()|[\]\\]/gu, "\\$&")}(?![A-Za-z0-9_])`, "gu"),
        to,
      );
    if (identifiers.size)
      out = out.replace(new RegExp(ident, "gu"), (word) => identifiers.get(word) ?? word);
    return out;
  };
}

const git = (cwd: string, ...args: string[]) =>
  execFileSync("git", args, { cwd, encoding: "utf8", maxBuffer: 1 << 28 }).trim();

/** Packages under `crates/` or `davinci/` whose sources the range changes. */
export function touchedPackages(root: string, base: string, head: string): string[] {
  const files = git(root, "diff", "--name-only", base, head).split("\n").filter(Boolean);
  const packages = new Set<string>();
  for (const file of files) {
    const match = /^((?:crates|davinci|examples)\/[^/]+)\/src\//u.exec(file);
    if (!match) continue;
    const manifest = path.join(root, match[1], "Cargo.toml");
    if (!fs.existsSync(manifest)) continue;
    const name = /^name\s*=\s*"([^"]+)"/mu.exec(fs.readFileSync(manifest, "utf8"))?.[1];
    if (name) packages.add(name);
  }
  return [...packages].sort();
}

/** Build one package's library with the release profile and return its `.ll`. */
export function emitIr(checkout: string, pkg: string, targetDir: string): string {
  const crate = pkg.replaceAll("-", "_");
  const messages = execFileSync(
    "cargo",
    [
      "rustc",
      "--locked",
      "--release",
      "-p",
      pkg,
      "--lib",
      "--message-format=json",
      "--",
      "--emit=llvm-ir,link",
    ],
    {
      cwd: checkout,
      encoding: "utf8",
      maxBuffer: 1 << 28,
      stdio: ["ignore", "pipe", "inherit"],
      env: { ...process.env, CARGO_TARGET_DIR: targetDir, CARGO_INCREMENTAL: "0" },
    },
  );
  // The `.ll` sits beside the hashed rlib/rmeta, in `release/deps/` or, with
  // cargo's build-dir layout, in `release/build/<pkg>/<hash>/out/`; a cached
  // build keeps the one its last compilation wrote.
  const checkoutUrl = `path+file://${fs.realpathSync(checkout)}/`;
  for (const line of messages.split("\n")) {
    if (!line.startsWith("{")) continue;
    const message = JSON.parse(line) as {
      reason?: string;
      package_id?: string;
      target?: { name?: string };
      filenames?: string[];
    };
    if (message.reason !== "compiler-artifact" || message.target?.name !== crate) continue;
    if (!message.package_id?.startsWith(checkoutUrl)) continue;
    for (const file of message.filenames ?? []) {
      const hashed = new RegExp(`^lib${crate}-([0-9a-f]+)\\.(?:rlib|rmeta)$`, "u").exec(
        path.basename(file),
      );
      const ir = hashed && path.join(path.dirname(file), `${crate}-${hashed[1]}.ll`);
      if (ir && fs.existsSync(ir)) return fs.readFileSync(ir, "utf8");
    }
  }
  throw new Error(`cargo reported no LLVM IR for ${pkg} in ${checkout}`);
}

export interface PackageResult extends ModuleDifference {
  readonly package: string;
  /** First differing normalized lines per changed function (`--diff`). */
  readonly diffs?: Readonly<Record<string, readonly string[]>>;
}

/** Compare one package's base and head IR. */
export function comparePackage(
  pkg: string,
  baseIr: string,
  headIr: string,
  move: Relocate,
  diffLimit = 0,
): PackageResult {
  const base = normalizeModule(baseIr, move);
  const head = normalizeModule(headIr);
  const difference = compareModules(base, head);
  if (!diffLimit) return { package: pkg, ...difference };
  const diffs: Record<string, readonly string[]> = {};
  for (const name of difference.changed.slice(0, 20))
    diffs[name] = bodyDiff(base, head, name, diffLimit);
  return { package: pkg, ...difference, diffs };
}

export function report(results: readonly PackageResult[]): string {
  const lines: string[] = [];
  for (const result of results) {
    const ok = isIdentical(result);
    lines.push(
      `${ok ? "identical" : "DIFFERENT"} ${result.package}: ${result.identical} functions identical` +
        (ok
          ? ""
          : `, ${result.changed.length} changed, ${result.onlyBase.length} only in base, ${result.onlyHead.length} only in head`),
    );
    for (const [label, names] of [
      ["changed", result.changed],
      ["only base", result.onlyBase],
      ["only head", result.onlyHead],
    ] as const)
      for (const name of names.slice(0, 20)) {
        lines.push(`  ${label}: ${name}`);
        for (const line of result.diffs?.[name] ?? []) lines.push(`      ${line.slice(0, 400)}`);
      }
  }
  return lines.join("\n");
}

function parseArgs(argv: readonly string[]) {
  const options = {
    base: "",
    head: "",
    crates: [] as string[],
    steps: [] as string[],
    paths: [] as [string, string][],
    identifiers: new Map<string, string>(),
    targetDir: "",
    work: "",
    save: "",
    ir: [] as string[],
    diff: 0,
    keep: false,
    json: false,
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
    else if (flag === "--step") options.steps.push(value());
    else if (flag === "--rename-path") options.paths.push(pair(value()));
    else if (flag === "--rename") options.identifiers.set(...pair(value()));
    else if (flag === "--target-dir") options.targetDir = value();
    else if (flag === "--work") options.work = value();
    else if (flag === "--save") options.save = value();
    else if (flag === "--ir") options.ir.push(value(), value());
    else if (flag === "--diff") options.diff = Number(value());
    else if (flag === "--keep") options.keep = true;
    else if (flag === "--json") options.json = true;
    else throw new Error(`unknown flag ${flag}`);
  }
  if (!options.base && !options.ir.length)
    throw new Error("--base <rev> or --ir <base.ll> <head.ll> is required");
  return options;
}

function rewritesOf(options: ReturnType<typeof parseArgs>): Relocate {
  return relocator({
    steps: options.steps.map((name) => {
      const step = steps[name];
      if (!step) throw new Error(`unknown step ${name}; known: ${Object.keys(steps).join(", ")}`);
      return step;
    }),
    paths: options.paths,
    identifiers: options.identifiers,
  });
}

function finish(options: ReturnType<typeof parseArgs>, results: readonly PackageResult[]): number {
  console.log(options.json ? JSON.stringify(results, null, 2) : report(results));
  return results.every(isIdentical) ? 0 : 1;
}

function main(argv: readonly string[]): number {
  const options = parseArgs(argv);
  const move = rewritesOf(options);
  if (options.ir.length) {
    const [baseIr, headIr] = options.ir.map((file) => fs.readFileSync(file, "utf8"));
    const label = options.crates[0] ?? path.basename(options.ir[1], ".ll");
    return finish(options, [comparePackage(label, baseIr, headIr, move, options.diff)]);
  }
  const work = options.work || fs.mkdtempSync(path.join(os.tmpdir(), "vize-ir-identity-"));
  const targetDir = options.targetDir || path.join(work, "target");
  const checkouts: string[] = [];
  const checkout = (rev: string, label: string) => {
    const dir = path.join(work, label);
    if (!fs.existsSync(dir)) git(repoRoot, "worktree", "add", "--detach", dir, rev);
    else git(dir, "checkout", "--quiet", "--detach", rev);
    checkouts.push(dir);
    return dir;
  };
  try {
    const baseDir = checkout(options.base, "base");
    const headDir = options.head ? checkout(options.head, "head") : repoRoot;
    const packages = options.crates.length
      ? options.crates
      : touchedPackages(repoRoot, options.base, options.head || "HEAD");
    const results = packages.map((pkg) => {
      const baseIr = emitIr(baseDir, pkg, targetDir);
      const headIr = emitIr(headDir, pkg, targetDir);
      if (options.save) {
        fs.mkdirSync(options.save, { recursive: true });
        fs.writeFileSync(path.join(options.save, `${pkg}.base.ll`), baseIr);
        fs.writeFileSync(path.join(options.save, `${pkg}.head.ll`), headIr);
      }
      return comparePackage(pkg, baseIr, headIr, move, options.diff);
    });
    return finish(options, results);
  } finally {
    if (!options.keep) {
      for (const dir of checkouts) git(repoRoot, "worktree", "remove", "--force", dir);
      if (!options.work) fs.rmSync(work, { recursive: true, force: true });
    }
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1]))
  process.exitCode = main(process.argv.slice(2));
