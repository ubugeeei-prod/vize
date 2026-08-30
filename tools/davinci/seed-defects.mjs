// Seeded-defect generator + identity-based recall assertion (Davinci P0-13).
//
// Doctrine: davinci-road/assurance.md, "Seeded-defect recall — the FN
// oracle". Two pilot defect classes are injected into COPIES of real
// sources (originals are never touched):
//
//   (a) undefined-template-ref — rename one `<script setup>` binding that
//       the template references (first eligible, deterministic), so the
//       template reference dangles. Expected: vue/no-undefined-refs.
//   (b) unused-binding — inject `const __davinci_seeded_unused = 0;` into
//       `<script setup>`. Expected today: nothing (documented FN gap).
//
// Modes:
//   --fixtures <dir>  seed every .vue under <dir> (the committed CI set
//                     lives at tests/_fixtures/davinci-fpfn)
//   --matrix          generate matrix stubs via matrix-gen.mjs --write into
//                     <out>/matrix-src, then seed those
//   --corpus-shard    seed the hydrated P0-13 shard submodules
//                     (splitpanes, layoutit-grid, cssgridgenerator)
//   --out <dir>       working area; writes <out>/original/, <out>/seeded/,
//                     <out>/manifest.json
//   --assert          run `vize lint --no-config --format json` over both
//                     trees and compare the EXACT diagnostic set against
//                     the manifest (identity, never counts); exit 1 on any
//                     miss/drift, listing each one
//   --report <path>   write the assertion report JSON
//   --baseline-lint-json / --seeded-lint-json <path>
//                     self-test hooks: use a pre-recorded lint JSON instead
//                     of spawning vize (tests/tooling/davinci-fpfn-pilots)
//
// Exit codes: 0 = seeded / assertion passed, 1 = assertion failed,
// 2 = usage or environment error.

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import {
  CLASS_A,
  CLASS_A_RULE,
  CLASS_B,
  applySeed,
  describeSeededSpan,
  planClassA,
  planClassB,
} from "./lib/fpfn-seed-apply.mjs";
import { assertSeededTree } from "./lib/fpfn-seed-assert.mjs";
import {
  CORPUS_SHARD,
  lineStartsOf,
  listVueFiles,
  readJson,
  repoRoot,
  resolveVizeCli,
  shardProjectDir,
  writeJson,
} from "./lib/fpfn-shared.mjs";

const toolDir = path.dirname(fileURLToPath(import.meta.url));

const USAGE = `Usage: rust-script tools/commands/davinci/seed-defects.rs (--fixtures <dir> | --matrix | --corpus-shard) --out <dir> [--assert] [--report <path>]

Seeds the P0-13 defect classes into copies of .vue sources and (with
--assert) verifies recall by diagnostic identity against the manifest.`;

function fail(message) {
  console.error(message);
  process.exit(2);
}

function parseArgs(argv) {
  const args = {
    fixtures: null,
    matrix: false,
    corpusShard: false,
    out: null,
    assert: false,
    report: null,
    baselineLintJson: null,
    seededLintJson: null,
  };
  const takeValue = (index, name) => {
    const value = argv[index + 1];
    if (value == null) fail(`${name} requires a value`);
    return value;
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--fixtures") args.fixtures = takeValue(i++, arg);
    else if (arg === "--matrix") args.matrix = true;
    else if (arg === "--corpus-shard") args.corpusShard = true;
    else if (arg === "--out") args.out = takeValue(i++, arg);
    else if (arg === "--assert") args.assert = true;
    else if (arg === "--report") args.report = takeValue(i++, arg);
    else if (arg === "--baseline-lint-json") args.baselineLintJson = takeValue(i++, arg);
    else if (arg === "--seeded-lint-json") args.seededLintJson = takeValue(i++, arg);
    else if (arg === "--help" || arg === "-h") args.help = true;
    else fail(`unknown argument ${arg}\n\n${USAGE}`);
  }
  return args;
}

/** Resolve the source roots: [{root, prefix, label}] . */
function resolveSources(args, outDir) {
  const picked = [args.fixtures != null, args.matrix, args.corpusShard].filter(Boolean).length;
  if (picked !== 1)
    fail(`exactly one of --fixtures/--matrix/--corpus-shard is required\n\n${USAGE}`);
  if (args.fixtures != null) {
    const root = path.resolve(args.fixtures);
    if (!fs.existsSync(root)) fail(`--fixtures directory not found: ${root}`);
    const label = path.relative(repoRoot, root).split(path.sep).join("/") || root;
    return { kind: "fixtures", label, roots: [{ root, prefix: "" }] };
  }
  if (args.matrix) {
    const matrixDir = path.join(outDir, "matrix-src");
    const generated = spawnSync(
      process.execPath,
      [path.join(toolDir, "matrix-gen.mjs"), "--write", "--out-dir", matrixDir],
      { cwd: repoRoot, encoding: "utf8" },
    );
    if (generated.status !== 0) {
      fail(`matrix-gen.mjs --write failed:\n${generated.stdout}${generated.stderr}`);
    }
    return { kind: "matrix", label: "matrix-gen", roots: [{ root: matrixDir, prefix: "" }] };
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

function seed(args, outDir) {
  const source = resolveSources(args, outDir);
  const files = [];
  const injections = [];
  const edits = {};
  let classAEligible = 0;
  for (const { root, prefix } of source.roots) {
    for (const relPath of listVueFiles(root)) {
      const seedPath = `${prefix}${relPath}`;
      const original = fs.readFileSync(path.join(root, relPath), "utf8");
      const classA = planClassA(original);
      const classB = planClassB(original);
      const { seeded, edits: fileEdits } = applySeed(original, classA.plan, classB.plan);
      const seededStarts = lineStartsOf(seeded);
      const fileRecord = {
        path: seedPath,
        classA: classA.plan != null,
        classB: classB.plan != null,
      };
      if (classA.plan == null) fileRecord.classAReason = classA.reason;
      files.push(fileRecord);
      if (classA.plan != null) {
        classAEligible += 1;
        const refStart = mapTemplateRef(seeded, classA.plan, fileEdits);
        injections.push({
          class: CLASS_A,
          path: seedPath,
          expectedRule: CLASS_A_RULE,
          identifier: { original: classA.plan.name, seeded: classA.plan.seededName },
          scriptRenameCount: classA.plan.renameSpans.length,
          expected: describeSeededSpan(
            seeded,
            seededStarts,
            refStart,
            refStart + classA.plan.name.length,
          ),
        });
      }
      if (classB.plan != null) {
        const idStart = seeded.indexOf("__davinci_seeded_unused");
        injections.push({
          class: CLASS_B,
          path: seedPath,
          expectedRule: null,
          identifier: { original: null, seeded: "__davinci_seeded_unused" },
          createdScriptSetupBlock: classB.plan.createdBlock,
          expected: describeSeededSpan(
            seeded,
            seededStarts,
            idStart,
            idStart + "__davinci_seeded_unused".length,
          ),
          note: "vize_croquis unused_bindings has no lint consumer (documented FN, ledger-fn.md)",
        });
      }
      if (fileEdits.length > 0) edits[seedPath] = fileEdits;
      fs.mkdirSync(path.dirname(path.join(outDir, "original", seedPath)), { recursive: true });
      fs.writeFileSync(path.join(outDir, "original", seedPath), original);
      fs.mkdirSync(path.dirname(path.join(outDir, "seeded", seedPath)), { recursive: true });
      fs.writeFileSync(path.join(outDir, "seeded", seedPath), seeded);
    }
  }
  injections.sort(
    (a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0) || (a.class < b.class ? -1 : 1),
  );
  const manifest = {
    schemaVersion: 1,
    tool: "tools/davinci/seed-defects.mjs",
    source: { kind: source.kind, label: source.label },
    scope: {
      filesCopied: files.length,
      classAEligible,
      classAInjections: injections.filter((entry) => entry.class === CLASS_A).length,
      classBInjections: injections.filter((entry) => entry.class === CLASS_B).length,
    },
    files,
    injections,
    edits,
  };
  writeJson(path.join(outDir, "manifest.json"), manifest);
  console.log(
    `seed-defects: source=${source.label} -> ${path.relative(process.cwd(), outDir) || outDir}`,
  );
  console.log(
    `scope-proof: files-scanned=${manifest.scope.filesCopied} ` +
      `class-a-eligible=${manifest.scope.classAEligible} ` +
      `class-a-injections=${manifest.scope.classAInjections} ` +
      `class-b-injections=${manifest.scope.classBInjections}`,
  );
  return manifest;
}

/** Seeded-file offset of the (unique) dangling template reference. */
function mapTemplateRef(seeded, plan, fileEdits) {
  let refStart = plan.templateRef[0];
  for (const { span, delta } of fileEdits) {
    if (span[1] <= plan.templateRef[0]) refStart += delta;
  }
  const found = seeded.slice(refStart, refStart + plan.name.length);
  if (found !== plan.name) {
    throw new Error(`seed-defects internal error: template ref relocation failed (${found})`);
  }
  return refStart;
}

function printAssertReport(report) {
  const { classA, classB, baselineShift, unexpected } = report;
  console.log(
    `assert: class-a detected=${classA.detected}/${classA.expected} ` +
      `class-b detected=${classB.detected}/${classB.expected} ` +
      `baseline mapped=${baselineShift.mapped} verdict=${report.verdict}`,
  );
  const describe = (row) =>
    `${row.path}:${row.line}:${row.column}-${row.endLine}:${row.endColumn} ${row.ruleId}`;
  for (const miss of classA.misses) {
    console.log(`MISS class-a ${describe(miss)} identifier=${miss.identifier}`);
  }
  for (const miss of baselineShift.misses) console.log(`MISS baseline ${describe(miss)}`);
  for (const row of baselineShift.unmappable) console.log(`UNMAPPABLE baseline ${describe(row)}`);
  for (const row of unexpected) console.log(`UNEXPECTED ${describe(row)}`);
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    console.log(USAGE);
    return 0;
  }
  if (args.out == null) fail(`--out <dir> is required\n\n${USAGE}`);
  const outDir = path.resolve(args.out);
  fs.mkdirSync(outDir, { recursive: true });

  const manifestPath = path.join(outDir, "manifest.json");
  const hasSource = args.fixtures != null || args.matrix || args.corpusShard;
  let manifest;
  if (hasSource) manifest = seed(args, outDir);
  else if (args.assert && fs.existsSync(manifestPath)) manifest = readJson(manifestPath);
  else fail(`nothing to do: pass a source mode, or --assert with an existing ${manifestPath}`);

  if (!args.assert) return 0;
  const hooks = { baselineLintJson: args.baselineLintJson, seededLintJson: args.seededLintJson };
  const cli =
    args.baselineLintJson != null && args.seededLintJson != null ? null : resolveVizeCli();
  const report = assertSeededTree({ manifest, outDir, cli, hooks });
  if (args.report != null) writeJson(path.resolve(args.report), report);
  printAssertReport(report);
  return report.verdict === "pass" ? 0 : 1;
}

process.exitCode = main();
