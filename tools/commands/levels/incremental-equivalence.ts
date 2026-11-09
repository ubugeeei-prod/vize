import { readdirSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const CORPUS_SHARD = ["splitpanes", "layoutit-grid", "cssgridgenerator"] as const;
const SCRIPTS = "tools/commands/levels/incremental-equivalence/scripts";
const USAGE = `usage: node tools/commands/levels/incremental-equivalence.ts (--corpus-shard | --fixtures <dir>)... [--seeded] [--report <path>]`;

interface FixtureRoot {
  label: string;
  directory: string;
}

interface Arguments {
  roots: FixtureRoot[];
  seeded: boolean;
  reportPath?: string;
  help: boolean;
}

function main(argv: readonly string[]): number {
  const root = process.cwd();
  if (!isDirectory(path.join(root, SCRIPTS))) {
    throw new Error(`run from the repository root (${SCRIPTS} not found)\n${USAGE}`);
  }
  const args = parseArguments(root, argv);
  if (args.help) {
    process.stdout.write(`${USAGE}\n`);
    return 0;
  }
  if (args.roots.length === 0) throw new Error(USAGE);

  const cargoArguments = [
    "run",
    "--release",
    "--locked",
    "-p",
    "vize_incremental",
    "--example",
    "incremental_equivalence",
  ];
  if (args.seeded) cargoArguments.push("--features", "seeded-stale-cache");
  cargoArguments.push("--", "--scripts", SCRIPTS);
  for (const fixture of args.roots) {
    cargoArguments.push("--root", `${fixture.label}=${fixture.directory}`);
  }

  const result = spawnSync("cargo", cargoArguments, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: Number.POSITIVE_INFINITY,
  });
  if (result.error !== undefined) throw new Error(`cannot run cargo: ${result.error.message}`);
  const stdout = result.stdout;
  const stderr = result.stderr;
  if (args.reportPath !== undefined) {
    writeFileSync(path.resolve(root, args.reportPath), stdout);
  }
  process.stdout.write(stdout);

  const counts = parseCounts(stdout);
  const count = (key: string): number => counts.get(key) ?? 0;
  const hasScope = ["files", "steps_applied", "comparisons", "blocks_compared"].every(
    (key) => count(key) > 0,
  );
  if (!hasScope) {
    process.stderr.write(stderr);
    process.stderr.write("TS-42 FAIL: the run compared nothing (scope proof missing)\n");
    return 1;
  }

  const mismatches = count("mismatches");
  if (args.seeded) {
    const caught = result.status === 1 && mismatches > 0;
    process.stdout.write(
      `TS-42 seeded stale cache: ${caught ? "caught" : "MISSED"} (${mismatches} stale states)\n`,
    );
    return caught ? 0 : 1;
  }

  const passed = result.status === 0 && mismatches === 0;
  if (!passed) process.stderr.write(stderr);
  process.stdout.write(
    `TS-42 ${passed ? "PASS" : "FAIL"}: ${count("files")} files, ${count("steps_applied")} steps, ${count("comparisons")} comparisons, ${count("blocks_compared")} blocks, ${mismatches} mismatches\n`,
  );
  return passed ? 0 : 1;
}

function parseArguments(root: string, argv: readonly string[]): Arguments {
  const args: Arguments = { roots: [], seeded: false, help: false };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    const value = (): string => {
      index += 1;
      const next = argv[index];
      if (next === undefined) throw new Error(USAGE);
      return next;
    };
    if (argument === "--corpus-shard") {
      for (const id of CORPUS_SHARD) {
        const directory = path.join(root, "tests", "_fixtures", "_git", id);
        if (!hasVueFile(directory)) {
          throw new Error(
            `corpus shard project ${id} is not hydrated. Run:\n  git submodule update --init --depth 1 -- tests/_fixtures/_git/${id}`,
          );
        }
        args.roots.push({ label: id, directory });
      }
    } else if (argument === "--fixtures") {
      const fixturePath = value();
      args.roots.push({
        label: fixturePath.replaceAll("/", "_"),
        directory: path.resolve(root, fixturePath),
      });
    } else if (argument === "--seeded") {
      args.seeded = true;
    } else if (argument === "--report") {
      args.reportPath = value();
    } else if (argument === "--help" || argument === "-h") {
      args.help = true;
    } else {
      throw new Error(`unknown argument ${argument}\n${USAGE}`);
    }
  }
  return args;
}

function parseCounts(stdout: string): Map<string, number> {
  const counts = new Map<string, number>();
  for (const line of stdout.split("\n")) {
    const separator = line.indexOf("=");
    if (separator === -1) continue;
    const value = line.slice(separator + 1);
    if (!/^\d+$/u.test(value)) continue;
    counts.set(line.slice(0, separator), Number(value));
  }
  return counts;
}

function hasVueFile(directory: string): boolean {
  let entries;
  try {
    entries = readdirSync(directory, { withFileTypes: true });
  } catch {
    return false;
  }
  return entries.some((entry) => {
    const entryPath = path.join(directory, entry.name);
    if (entry.isDirectory()) return hasVueFile(entryPath);
    if (entry.isSymbolicLink()) {
      try {
        return statSync(entryPath).isDirectory()
          ? hasVueFile(entryPath)
          : entryPath.endsWith(".vue");
      } catch {
        return false;
      }
    }
    return entry.isFile() && entry.name.endsWith(".vue");
  });
}

function isDirectory(directory: string): boolean {
  try {
    return statSync(directory).isDirectory();
  } catch {
    return false;
  }
}

try {
  process.exitCode = main(process.argv.slice(2));
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 2;
}
