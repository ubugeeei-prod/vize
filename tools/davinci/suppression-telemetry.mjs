// Suppression-telemetry FP oracle (Davinci P0-13).
//
// Doctrine: davinci-road/assurance.md, "Suppression telemetry — the FP
// oracle". Real projects carry `eslint-disable` pragmas; every vize
// diagnostic that fires on a line a user suppressed for the ANALOGOUS
// upstream rule is a false-positive candidate that must be triaged in
// davinci-road/plan/ledger-fp.md (`fixed` / `justified-with-witness` /
// `deferred-with-issue`), never left ambient.
//
// Mechanics (see lib/fpfn-suppress-scan.mjs for details): vize honors
// eslint-disable pragmas natively, so the tool lints byte-length-preserving
// DEFUSED copies of the sources — otherwise the suppressed diagnostics
// under measurement never surface. Rule names are mapped to vize analogs
// via the committed parity fixture (eslint-plugin-vue) plus a verified-core
// sidecar; unmapped names are reported, not errors.
//
// Modes:
//   --fixtures <dir>   scan every .vue under <dir>
//   --corpus-shard     scan the hydrated P0-13 shard submodules
//   --out <dir>        working area for the defused copies
//   --report <path>    write the telemetry report JSON
//
// Exit codes: 0 = scan completed (candidates belong in the ledger, they are
// not a tool failure), 2 = usage or environment error.

import fs from "node:fs";
import path from "node:path";
import process from "node:process";

import {
  defuseSuppressions,
  intersectSuppressions,
  loadRuleMap,
  scanSuppressions,
} from "./lib/fpfn-suppress-scan.mjs";
import {
  CORPUS_SHARD,
  flattenLintJson,
  listVueFiles,
  repoRoot,
  resolveVizeCli,
  runVizeLintJson,
  shardProjectDir,
  writeJson,
} from "./lib/fpfn-shared.mjs";

const USAGE = `Usage: rust-script tools/commands/davinci/suppression-telemetry.rs (--fixtures <dir> | --corpus-shard) --out <dir> [--report <path>]

Collects eslint-disable pragmas, maps rule names to vize analogs, and
reports vize diagnostics on suppressed lines as FP candidates.`;

function fail(message) {
  console.error(message);
  process.exit(2);
}

function parseArgs(argv) {
  const args = { fixtures: null, corpusShard: false, out: null, report: null };
  const takeValue = (index, name) => {
    const value = argv[index + 1];
    if (value == null) fail(`${name} requires a value`);
    return value;
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--fixtures") args.fixtures = takeValue(i++, arg);
    else if (arg === "--corpus-shard") args.corpusShard = true;
    else if (arg === "--out") args.out = takeValue(i++, arg);
    else if (arg === "--report") args.report = takeValue(i++, arg);
    else if (arg === "--help" || arg === "-h") args.help = true;
    else fail(`unknown argument ${arg}\n\n${USAGE}`);
  }
  return args;
}

function resolveSources(args) {
  const picked = [args.fixtures != null, args.corpusShard].filter(Boolean).length;
  if (picked !== 1) fail(`exactly one of --fixtures/--corpus-shard is required\n\n${USAGE}`);
  if (args.fixtures != null) {
    const root = path.resolve(args.fixtures);
    if (!fs.existsSync(root)) fail(`--fixtures directory not found: ${root}`);
    const label = path.relative(repoRoot, root).split(path.sep).join("/") || root;
    return { kind: "fixtures", label, roots: [{ root, prefix: "" }] };
  }
  const roots = [];
  for (const id of CORPUS_SHARD) {
    const root = shardProjectDir(id);
    if (!fs.existsSync(root) || listVueFiles(root).length === 0) {
      fail(
        `corpus shard project ${id} is not hydrated. Run:\n` +
          `  git submodule update --init --depth 1 -- tests/_fixtures/_git/${id}`,
      );
    }
    roots.push({ root, prefix: `${id}/` });
  }
  return { kind: "corpus-shard", label: CORPUS_SHARD.join("+"), roots };
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    console.log(USAGE);
    return 0;
  }
  if (args.out == null) fail(`--out <dir> is required\n\n${USAGE}`);
  const source = resolveSources(args);
  const outDir = path.resolve(args.out);

  const files = [];
  const suppressionsByPath = new Map();
  let suppressionComments = 0;
  let namedSuppressions = 0;
  let bareSuppressions = 0;
  const nameOccurrences = new Map();
  for (const { root, prefix } of source.roots) {
    for (const relPath of listVueFiles(root)) {
      const filePath = `${prefix}${relPath}`;
      files.push(filePath);
      const text = fs.readFileSync(path.join(root, relPath), "utf8");
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
      const target = path.join(outDir, "defused", filePath);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, defused);
    }
  }

  const cli = resolveVizeCli();
  const lintRows = flattenLintJson(runVizeLintJson(cli, path.join(outDir, "defused"), files));
  const diagnosticsByPath = new Map();
  for (const row of lintRows) {
    if (!diagnosticsByPath.has(row.path)) diagnosticsByPath.set(row.path, []);
    diagnosticsByPath.get(row.path).push(row);
  }

  const ruleMap = loadRuleMap();
  const { candidates, onBareLines } = intersectSuppressions(
    diagnosticsByPath,
    suppressionsByPath,
    ruleMap,
  );

  const unmapped = [...nameOccurrences.entries()]
    .filter(([rule]) => !ruleMap.mapped.has(rule))
    .map(([rule, occurrences]) => ({ rule, occurrences }))
    .sort((a, b) => (a.rule < b.rule ? -1 : a.rule > b.rule ? 1 : 0));
  const mappedSeen = [...nameOccurrences.keys()].filter((rule) => ruleMap.mapped.has(rule)).length;

  const report = {
    schemaVersion: 1,
    tool: "tools/davinci/suppression-telemetry.mjs",
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
      mappedNamesSeen: mappedSeen,
      unmappedNamesSeen: unmapped.length,
      defusedRunDiagnostics: lintRows.length,
      diagnosticsOnBareSuppressedLines: onBareLines,
    },
    unmapped,
    candidates,
  };
  if (args.report != null) writeJson(path.resolve(args.report), report);

  console.log(`suppression-telemetry: source=${source.label} candidates=${candidates.length}`);
  console.log(
    `scope-proof: files-scanned=${report.scope.filesScanned} ` +
      `suppression-comments=${report.scope.suppressionComments} ` +
      `named=${report.scope.namedSuppressions} bare=${report.scope.bareSuppressions} ` +
      `rules-mapped=${report.ruleMap.mappedRules + report.ruleMap.coreSidecarRules} ` +
      `mapped-seen=${report.scope.mappedNamesSeen} unmapped-seen=${report.scope.unmappedNamesSeen} ` +
      `fp-candidates=${candidates.length}`,
  );
  for (const entry of unmapped) {
    console.log(`unmapped: ${entry.rule} x${entry.occurrences}`);
  }
  for (const candidate of candidates) {
    console.log(
      `fp-candidate: ${candidate.path}:${candidate.line}:${candidate.column} ` +
        `${candidate.vizeRule} (suppressed as ${candidate.eslintRule} at line ${candidate.commentLine})`,
    );
  }
  return 0;
}

process.exitCode = main();
