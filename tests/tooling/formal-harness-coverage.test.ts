import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse as parseToml } from "@iarna/toml";
import { parse as parseYaml } from "yaml";

// docs/davinci/decisions/2026-11-17-creusot-inventory.md: a law counts only
// when CI runs its verifier. Every `#[kani::proof]` harness must be reached by
// a Kani step of a generated workflow, either from the standalone root Kani
// compiles or through the module tree of a `--manifest-path` crate whose
// `--harness` filters select it. Every Creusot proof package must be a member
// of the proof workspace that `07-proofs-creusot.yml` replays.

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const creusotDir = path.join(repoRoot, "tests", "formal", "creusot");

type Step = { uses?: string; run?: string; with?: Record<string, unknown> };
type Workflow = { jobs?: Record<string, { steps?: Step[] }> };

/** One Kani invocation: a standalone root or a crate with harness filters. */
type Invocation = {
  where: string;
  root?: string;
  manifest?: string;
  tests: boolean;
  features: Set<string>;
  filters: string[];
};

function repoPath(file: string): string {
  return path.relative(repoRoot, file).split(path.sep).join("/");
}

function trackedFiles(pattern: string): string[] {
  return execFileSync("git", ["ls-files", "-z", "--", pattern], { cwd: repoRoot, encoding: "utf8" })
    .split("\0")
    .filter(Boolean);
}

function parseInvocation(where: string, words: string[]): Invocation {
  const invocation: Invocation = { where, tests: false, features: new Set(), filters: [] };
  for (let at = 0; at < words.length; at += 1) {
    const word = words[at]!;
    const value = (): string => {
      const next = words[at + 1];
      assert.ok(next, `${where}: ${word} needs a value`);
      at += 1;
      return next;
    };
    if (word === "--manifest-path") {
      invocation.manifest = value();
    } else if (word === "--harness") {
      invocation.filters.push(value());
    } else if (word === "--features") {
      for (const feature of value().split(",")) invocation.features.add(feature);
    } else if (word === "--tests") {
      invocation.tests = true;
    } else if (word.endsWith(".rs") && !word.startsWith("-")) {
      invocation.root = word;
    }
  }
  assert.ok(
    Boolean(invocation.root) !== Boolean(invocation.manifest),
    `${where}: a Kani step names one standalone root or one manifest`,
  );
  return invocation;
}

/** Kani steps of every generated workflow and composite action. */
function kaniInvocations(): Invocation[] {
  const invocations: Invocation[] = [];
  const files = [
    ...trackedFiles(".github/workflows/*.yml"),
    ...trackedFiles(".github/actions/**/action.yml"),
  ];
  for (const file of files) {
    const document = parseYaml(fs.readFileSync(path.join(repoRoot, file), "utf8")) as Workflow & {
      runs?: { steps?: Step[] };
    };
    const jobs = Object.entries(document.jobs ?? {}).map(
      ([name, job]) => [name, job.steps] as const,
    );
    const steps = [...jobs, ["action", document.runs?.steps] as const];
    for (const [job, jobSteps] of steps) {
      for (const step of jobSteps ?? []) {
        const where = `${file} ${job}`;
        if (step.uses?.startsWith("model-checking/kani-github-action@")) {
          const args = typeof step.with?.args === "string" ? step.with.args : "";
          invocations.push(parseInvocation(where, args.split(/\s+/u).filter(Boolean)));
        }
        for (const line of (step.run ?? "").split("\n")) {
          const words = line.trim().split(/\s+/u);
          if (words[0] === "kani") invocations.push(parseInvocation(where, words.slice(1)));
          if (words[0] === "cargo" && words[1] === "kani") {
            invocations.push(parseInvocation(where, words.slice(2)));
          }
        }
      }
    }
  }
  for (const script of trackedFiles(".github/scripts/**/*.sh")) {
    assert.doesNotMatch(
      fs.readFileSync(path.join(repoRoot, script), "utf8"),
      /^\s*(?:cargo )?kani /mu,
      `${script}: run Kani from a workflow step so this check sees it`,
    );
  }
  return invocations;
}

/** A build: whether tests are compiled, and the enabled features (`all` for every one). */
type Gate = { tests: boolean; features: Set<string> | "all" };

/**
 * Evaluate a `cfg` predicate under a Kani build: `kani` holds, `creusot` does
 * not, `test` follows `--tests`, features follow `--features`, and
 * `all`/`any`/`not` combine them. Platform predicates (`target_*`, `unix`)
 * are taken as holding, since a module they gate is compiled on some runner.
 */
function cfgHolds(predicate: string, gate: Gate): boolean {
  let at = 0;
  const skipSpace = (): void => {
    while (predicate[at] === " ") at += 1;
  };
  const term = (): boolean => {
    skipSpace();
    const name = /^\w+/u.exec(predicate.slice(at))?.[0];
    assert.ok(name, `unsupported cfg predicate: ${predicate}`);
    at += name.length;
    skipSpace();
    if (predicate[at] === "(") {
      at += 1;
      const values: boolean[] = [];
      skipSpace();
      while (predicate[at] !== ")") {
        assert.ok(at < predicate.length, `unbalanced cfg predicate: ${predicate}`);
        values.push(term());
        skipSpace();
        if (predicate[at] === ",") at += 1;
        skipSpace();
      }
      at += 1;
      if (name === "all") return values.every(Boolean);
      if (name === "any") return values.some(Boolean);
      assert.equal(name, "not", `unsupported cfg combinator: ${predicate}`);
      assert.equal(values.length, 1, `not() takes one predicate: ${predicate}`);
      return !values[0];
    }
    if (predicate[at] === "=") {
      at += 1;
      skipSpace();
      const value = /^"([^"]*)"/u.exec(predicate.slice(at));
      assert.ok(value, `unsupported cfg value: ${predicate}`);
      at += value[0].length;
      if (name === "feature") return gate.features === "all" || gate.features.has(value[1]!);
      return true;
    }
    if (name === "test") return gate.tests;
    if (name === "creusot") return false;
    return true;
  };
  const value = term();
  skipSpace();
  assert.equal(at, predicate.length, `unsupported cfg predicate: ${predicate}`);
  return value;
}

/** Whether the attributes in front of an item keep it under this build. */
function enabled(attributes: string[], gate: Gate): boolean {
  return attributes.every((attribute) => {
    const predicate = /^#\[cfg\((.*)\)\]$/u.exec(attribute)?.[1];
    return predicate === undefined || cfgHolds(predicate, gate);
  });
}

type Scan = {
  /** Files of the non-inline modules this file declares, under this build. */
  modules: string[];
  /** Harness names, qualified by the file's inline modules. */
  harnesses: string[];
};

/**
 * Read a Rust file's module declarations and harnesses. `dir` is where its
 * child module files live: the file's own directory for a crate root or
 * `mod.rs`, otherwise the directory named after the file. Without `dir`,
 * only the harnesses are read.
 */
function scanFile(file: string, dir: string | undefined, gate: Gate): Scan {
  const scan: Scan = { modules: [], harnesses: [] };
  const inline: Array<{ name: string; depth: number; on: boolean }> = [];
  let depth = 0;
  let attributes: string[] = [];
  let proof = false;
  for (const raw of fs.readFileSync(file, "utf8").split("\n")) {
    const line = raw.replace(/\/\/.*$/u, "").trim();
    const prefix = inline.map((module) => module.name);
    if (line.startsWith("#[")) {
      attributes.push(line);
      if (line === "#[kani::proof]") proof = true;
      continue;
    }
    const declared = line.match(/^(?:pub(?:\([^)]*\))? )?mod (\w+)\s*(;|\{)/u);
    const live = inline.every((module) => module.on);
    if (declared?.[2] === "{") {
      inline.push({ name: declared[1]!, depth, on: enabled(attributes, gate) });
    } else if (declared && dir !== undefined && live && enabled(attributes, gate)) {
      const base = path.join(dir, ...prefix, declared[1]!);
      const target = [`${base}.rs`, path.join(base, "mod.rs")].find((candidate) =>
        fs.existsSync(candidate),
      );
      assert.ok(target, `${repoPath(file)} declares missing module ${declared[1]}`);
      scan.modules.push(target);
    }
    const harness = line.match(/^(?:pub(?:\([^)]*\))? )?fn (\$?\w+)\s*\(/u);
    if (proof && harness) {
      if (live && enabled(attributes, gate)) {
        scan.harnesses.push([...prefix, harness[1]!].join("::"));
      }
      proof = false;
    }
    if (!line.startsWith("#[") && line.length > 0) attributes = [];
    for (const char of line) {
      if (char === "{") depth += 1;
      if (char === "}") {
        depth -= 1;
        if (inline.at(-1)?.depth === depth) inline.pop();
      }
    }
  }
  return scan;
}

/** One harness: its name within its file (inline modules included) and its full path. */
type Harness = { local: string; qualified: string };

/** Every file of a crate's module tree with its harnesses. */
function moduleTree(root: string, gate: Gate): Map<string, Harness[]> {
  const harnesses = new Map<string, Harness[]>();
  const visit = (file: string, modulePath: string[], isRoot: boolean): void => {
    const ownsDir = isRoot || path.basename(file) === "mod.rs";
    const dir = ownsDir ? path.dirname(file) : file.slice(0, -".rs".length);
    const scan = scanFile(file, dir, gate);
    harnesses.set(
      repoPath(file),
      scan.harnesses.map((local) => ({ local, qualified: [...modulePath, local].join("::") })),
    );
    for (const child of scan.modules) {
      const name =
        path.basename(child) === "mod.rs"
          ? path.basename(path.dirname(child))
          : path.basename(child, ".rs");
      const inner = path.relative(dir, path.dirname(child)).split(path.sep).filter(Boolean);
      const prefix = path.basename(child) === "mod.rs" ? inner.slice(0, -1) : inner;
      visit(child, [...modulePath, ...prefix, name], false);
    }
  };
  visit(root, [], true);
  return harnesses;
}

/** Repository file → in-file harness names some workflow verifies. */
function verifiedHarnesses(): Map<string, Set<string>> {
  const verified = new Map<string, Set<string>>();
  const note = (file: string, name: string): void => {
    const names = verified.get(file) ?? new Set<string>();
    names.add(name);
    verified.set(file, names);
  };
  for (const invocation of kaniInvocations()) {
    const gate = { tests: invocation.tests, features: invocation.features };
    const root = invocation.root
      ? path.join(repoRoot, invocation.root)
      : path.join(repoRoot, path.dirname(invocation.manifest!), "src", "lib.rs");
    assert.ok(fs.existsSync(root), `${invocation.where}: missing ${repoPath(root)}`);
    let selected = 0;
    for (const [file, harnesses] of moduleTree(root, gate)) {
      for (const { local, qualified } of harnesses) {
        const chosen =
          invocation.filters.length === 0 ||
          invocation.filters.some((filter) => qualified.includes(filter));
        if (chosen) {
          note(file, local);
          selected += 1;
        }
      }
    }
    assert.ok(selected > 0, `${invocation.where}: the Kani step selects no harness`);
  }
  return verified;
}

test("a Kani step of a generated workflow verifies every Kani harness", () => {
  const verified = verifiedHarnesses();
  const orphans: string[] = [];
  let total = 0;
  const harnessFiles = execFileSync("git", ["grep", "-lz", "kani::proof", "--", "*.rs"], {
    cwd: repoRoot,
    encoding: "utf8",
  })
    .split("\0")
    .filter(Boolean);
  for (const file of harnessFiles) {
    const source = fs.readFileSync(path.join(repoRoot, file), "utf8");
    // Every harness in the file, under a build that enables every gate.
    const { harnesses } = scanFile(path.join(repoRoot, file), undefined, {
      tests: true,
      features: "all",
    });
    for (const local of harnesses) {
      total += 1;
      if (!verified.get(file)?.has(local)) orphans.push(`${file} ${local}`);
    }
    assert.equal(
      [...source.matchAll(/^ *#\[kani::proof\]$/gmu)].length,
      [
        ...source.matchAll(
          /^ *#\[kani::proof\]\n(?: *#\[[^\n]*\]\n)* *(?:pub(?:\([^)]*\))? )?fn \$?\w+\(/gmu,
        ),
      ].length,
      `${file}: every #[kani::proof] is followed by its harness function`,
    );
  }
  assert.ok(total > 0, "the repository has Kani harnesses");
  assert.deepEqual(orphans, [], "Kani harnesses no workflow verifies");
});

test("every Creusot proof package is a member of the replayed proof workspace", () => {
  const workspace = parseToml(fs.readFileSync(path.join(creusotDir, "Cargo.toml"), "utf8")) as {
    workspace?: { members?: string[] };
  };
  const members = workspace.workspace?.members ?? [];
  assert.ok(members.length > 0, "proof workspace members");
  const packages = fs
    .readdirSync(creusotDir, { withFileTypes: true })
    .filter(
      (entry) =>
        entry.isDirectory() && fs.existsSync(path.join(creusotDir, entry.name, "Cargo.toml")),
    )
    .map((entry) => entry.name);
  assert.deepEqual([...members].sort(), [...packages].sort());

  const libraries = members.map((member) => {
    const manifest = parseToml(
      fs.readFileSync(path.join(creusotDir, member, "Cargo.toml"), "utf8"),
    ) as { lib?: { path?: string } };
    const lib = manifest.lib?.path;
    assert.ok(lib, `${member} names its library root`);
    return repoPath(path.resolve(creusotDir, member, lib));
  });
  // A `creusot.rs` root outside the workspace would be a proof nobody replays.
  const roots = trackedFiles("*.rs").filter((file) => path.basename(file) === "creusot.rs");
  assert.deepEqual([...roots].sort(), [...libraries].sort());
});
