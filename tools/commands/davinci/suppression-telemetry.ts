import fs from "node:fs";
import path from "node:path";

import { isEntrypoint, runEntrypoint } from "../../support/command-runtime.ts";
import {
  CORPUS_SHARD,
  flattenLintJson,
  listVueFiles,
  repoRoot,
  resolveVizeCli,
  runVizeLintJson,
  shardProjectDir,
  writeJson,
} from "../../support/compat/davinci/lib/fpfn-shared.ts";
import {
  defuseSuppressions,
  intersectSuppressions,
  loadRuleMap,
  scanSuppressions,
} from "../../support/compat/davinci/lib/fpfn-suppress-scan.ts";
import type { SuppressionScan } from "../../support/compat/davinci/lib/fpfn-suppress-scan.ts";

const usage =
  "Usage: node tools/commands/davinci/suppression-telemetry.ts (--fixtures <dir> | --corpus-shard) --out <dir> [--report <path>]\n\nCollects eslint-disable pragmas, maps rule names to vize analogs, and\nreports vize diagnostics on suppressed lines as FP candidates.";

interface Arguments {
  fixtures?: string;
  corpusShard: boolean;
  out?: string;
  report?: string;
  help: boolean;
}

interface SourceRoot {
  root: string;
  prefix: string;
}

interface ResolvedSources {
  kind: "fixtures" | "corpus-shard";
  label: string;
  roots: SourceRoot[];
}

class UsageError extends Error {
  readonly exitCode = 2;
}

function value(argv: readonly string[], index: number, name: string): string {
  const result = argv[index];
  if (result === undefined) throw new UsageError(`${name} requires a value`);
  return result;
}

export function parseArguments(argv: readonly string[]): Arguments {
  const args: Arguments = { corpusShard: false, help: false };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === "--fixtures") args.fixtures = value(argv, ++index, argument);
    else if (argument === "--corpus-shard") args.corpusShard = true;
    else if (argument === "--out") args.out = value(argv, ++index, argument);
    else if (argument === "--report") args.report = value(argv, ++index, argument);
    else if (argument === "--help" || argument === "-h") args.help = true;
    else throw new UsageError(`unknown argument ${argument}\n\n${usage}`);
  }
  return args;
}

function resolveSources(args: Arguments): ResolvedSources {
  if (Number(args.fixtures !== undefined) + Number(args.corpusShard) !== 1) {
    throw new UsageError(`exactly one of --fixtures/--corpus-shard is required\n\n${usage}`);
  }
  if (args.fixtures !== undefined) {
    const fixtures = path.resolve(args.fixtures);
    if (!fs.existsSync(fixtures)) {
      throw new UsageError(`--fixtures directory not found: ${fixtures}`);
    }
    return {
      kind: "fixtures",
      label: path.relative(repoRoot, fixtures).split(path.sep).join("/"),
      roots: [{ root: fixtures, prefix: "" }],
    };
  }

  const roots = CORPUS_SHARD.map((project) => {
    const root = shardProjectDir(project);
    if (!fs.existsSync(root) || listVueFiles(root).length === 0) {
      throw new UsageError(
        `corpus shard project ${project} is not hydrated. Run:\n  git submodule update --init --depth 1 -- tests/_fixtures/_git/${project}`,
      );
    }
    return { root, prefix: `${project}/` };
  });
  return { kind: "corpus-shard", label: CORPUS_SHARD.join("+"), roots };
}

function run(args: Arguments): void {
  if (args.help) {
    process.stdout.write(`${usage}\n`);
    return;
  }
  if (args.out === undefined) throw new UsageError(`--out <dir> is required\n\n${usage}`);

  const output = path.resolve(args.out);
  const source = resolveSources(args);
  const files: string[] = [];
  const suppressionsByPath = new Map<string, SuppressionScan>();
  const nameOccurrences = new Map<string, number>();
  let suppressionComments = 0;
  let namedSuppressions = 0;
  let bareSuppressions = 0;

  for (const sourceRoot of source.roots) {
    for (const relativePath of listVueFiles(sourceRoot.root)) {
      const filePath = `${sourceRoot.prefix}${relativePath}`;
      files.push(filePath);
      const text = fs.readFileSync(path.join(sourceRoot.root, relativePath), "utf8");
      const scanned = scanSuppressions(text);
      suppressionComments += scanned.comments.length;
      for (const comment of scanned.comments) {
        if (comment.kind === "enable") continue;
        if (comment.rules.length === 0) bareSuppressions += 1;
        else {
          namedSuppressions += comment.rules.length;
          for (const rule of comment.rules) {
            nameOccurrences.set(rule, (nameOccurrences.get(rule) ?? 0) + 1);
          }
        }
      }
      if (scanned.ranges.length > 0) suppressionsByPath.set(filePath, scanned);
      const { defused } = defuseSuppressions(text);
      const destination = path.join(output, "defused", filePath);
      fs.mkdirSync(path.dirname(destination), { recursive: true });
      fs.writeFileSync(destination, defused);
    }
  }

  const lintRows = flattenLintJson(
    runVizeLintJson(resolveVizeCli(), path.join(output, "defused"), files),
  );
  const diagnosticsByPath = new Map<string, typeof lintRows>();
  for (const row of lintRows) {
    const rows = diagnosticsByPath.get(row.path) ?? [];
    rows.push(row);
    diagnosticsByPath.set(row.path, rows);
  }

  const ruleMap = loadRuleMap();
  const { candidates, onBareLines } = intersectSuppressions(
    diagnosticsByPath,
    suppressionsByPath,
    ruleMap,
  );
  const names = [...nameOccurrences].sort(([left], [right]) =>
    left < right ? -1 : left > right ? 1 : 0,
  );
  const unmapped = names
    .filter(([rule]) => !ruleMap.mapped.has(rule))
    .map(([rule, occurrences]) => ({ rule, occurrences }));
  const mappedNamesSeen = names.filter(([rule]) => ruleMap.mapped.has(rule)).length;
  const report = {
    schemaVersion: 1,
    tool: "tools/commands/davinci/suppression-telemetry.ts",
    source: { kind: source.kind, label: source.label },
    ruleMap: {
      fixture: ruleMap.fixturePath,
      mappedRules: ruleMap.fixtureMappedCount,
      coreSidecarRules: ruleMap.coreSidecarCount,
    },
    scope: {
      filesScanned: files.length,
      suppressionComments,
      namedSuppressions,
      bareSuppressions,
      ruleNamesSeen: nameOccurrences.size,
      mappedNamesSeen,
      unmappedNamesSeen: unmapped.length,
      defusedRunDiagnostics: lintRows.length,
      diagnosticsOnBareSuppressedLines: onBareLines,
    },
    unmapped,
    candidates,
  };
  if (args.report !== undefined) writeJson(path.resolve(args.report), report);

  process.stdout.write(
    `suppression-telemetry: source=${report.source.label} candidates=${candidates.length}\n` +
      `scope-proof: files-scanned=${report.scope.filesScanned} suppression-comments=${suppressionComments} named=${namedSuppressions} bare=${bareSuppressions} rules-mapped=${ruleMap.fixtureMappedCount + ruleMap.coreSidecarCount} mapped-seen=${mappedNamesSeen} unmapped-seen=${unmapped.length} fp-candidates=${candidates.length}\n`,
  );
  for (const entry of unmapped) {
    process.stdout.write(`unmapped: ${entry.rule} x${entry.occurrences}\n`);
  }
  for (const candidate of candidates) {
    process.stdout.write(
      `fp-candidate: ${candidate.path}:${candidate.line}:${candidate.column} ${candidate.vizeRule} (suppressed as ${candidate.eslintRule} at line ${candidate.commentLine})\n`,
    );
  }
}

function main(): number {
  try {
    run(parseArguments(process.argv.slice(2)));
    return 0;
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    return error instanceof UsageError ? error.exitCode : 2;
  }
}

if (isEntrypoint(import.meta.url)) runEntrypoint(main);
