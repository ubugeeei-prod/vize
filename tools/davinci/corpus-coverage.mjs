// Corpus construct-coverage report (Davinci P0-6).
//
// Scans the HYDRATED corpus projects registered in
// tests/_fixtures/vue-ecosystem-fixtures.json for the construct-taxonomy
// dimensions in davinci-road/plan/taxonomy.toml and emits
// davinci-road/plan/corpus-coverage.md: per-construct x per-project counts
// plus a scope-proof footer (hydrated-project count vs manifest total).
//
// Inputs per project:
//   - `vueGlobs`        — *.vue scanned as SFCs (template block html or pug),
//                         *.tsx / *.jsx scanned as JSX sources
//   - `petiteVueGlobs`  — *.html / *.js scanned with the HTML scanner
//                         (petite-vue corpus entries; see the manifest notes)
//
// Hydration mirrors tools/fixtures/glyph-corpus.mjs: a project counts as
// hydrated when its fixture directory exists and is non-empty. Absent
// fixtures are excluded from every table, and the scope-proof footer says
// so loudly — the assurance rule is empty-means-proven-empty, never
// silently partial.
//
// The scan is a lexical pass, not a compile: what is mechanically derived
// and what is skipped is spelled out in the report's "Skipped" section
// (binding sources in particular are presence signals, not per-expression
// attribution). The scanners live in lib/corpus-coverage-*.mjs.
//
// Modes:
//   (default)  dry run — scan and print a summary; nothing is written
//   --write    write davinci-road/plan/corpus-coverage.md
//   --check    verify the committed report byte-matches a fresh scan
//              (exit 1 on drift)
//
// Determinism contract: output depends only on taxonomy.toml, the fixtures
// manifest, and the hydrated fixture trees. Projects iterate in manifest
// order, files in byte order; no timestamps, no absolute paths.
//
// Exit codes: 0 = success / up to date, 1 = --check found drift,
// 2 = usage or input validation error.

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { classifyAttribute, classifyTag } from "./lib/corpus-coverage-classify.mjs";
import { scanProject, sfcBlocks } from "./lib/corpus-coverage-project.mjs";
import { buildReport } from "./lib/corpus-coverage-render.mjs";
import { parseTomlLite, TomlLiteError } from "./toml-lite.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const taxonomyPath = path.join(repoRoot, "davinci-road", "plan", "taxonomy.toml");
const manifestPath = path.join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const reportPath = path.join(repoRoot, "davinci-road", "plan", "corpus-coverage.md");

function fail(message) {
  console.error(message);
  process.exit(2);
}

function loadTaxonomy() {
  try {
    return parseTomlLite(fs.readFileSync(taxonomyPath, "utf8"));
  } catch (error) {
    if (error instanceof TomlLiteError)
      fail(`malformed taxonomy ${taxonomyPath}: ${error.message}`);
    throw error;
  }
}

function loadProjects() {
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  return manifest.projects.map((project) => {
    const fixtureDir = path.resolve(repoRoot, project.fixturePath);
    const hydrated = fs.existsSync(fixtureDir) && fs.readdirSync(fixtureDir).length > 0;
    return { ...project, fixtureDir, hydrated };
  });
}

function parseArgs(argv) {
  const args = { write: false, check: false };
  for (const arg of argv) {
    if (arg === "--write") args.write = true;
    else if (arg === "--check") args.check = true;
    else fail(`unknown argument: ${arg} (expected --write or --check)`);
  }
  if (args.write && args.check) fail("--write and --check are mutually exclusive");
  return args;
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const taxonomy = loadTaxonomy();
  const projects = loadProjects();
  for (const project of projects) {
    if (project.hydrated) project.counts = scanProject(project, taxonomy);
  }
  const report = buildReport(taxonomy, projects);
  const hydratedCount = projects.filter((project) => project.hydrated).length;

  if (args.check) {
    const committed = fs.existsSync(reportPath) ? fs.readFileSync(reportPath, "utf8") : null;
    if (committed !== report) {
      console.error(
        `${path.relative(repoRoot, reportPath)} is stale (or the hydrated fixture set changed). ` +
          "Regenerate: rust-script tools/commands/davinci/corpus-coverage.rs --write",
      );
      process.exit(1);
    }
    console.log(
      `corpus-coverage: up to date (${hydratedCount}/${projects.length} projects hydrated)`,
    );
    return;
  }

  if (args.write) {
    fs.writeFileSync(reportPath, report);
    console.log(
      `wrote ${path.relative(repoRoot, reportPath)} (${hydratedCount}/${projects.length} projects hydrated)`,
    );
    return;
  }

  console.log(
    `dry run: would write ${path.relative(repoRoot, reportPath)} (${hydratedCount}/${projects.length} projects hydrated; pass --write)`,
  );
}

const invokedDirectly =
  process.argv[1] != null && fileURLToPath(import.meta.url) === path.resolve(process.argv[1]);
if (invokedDirectly) {
  main();
}

export { classifyAttribute, classifyTag, scanProject, sfcBlocks };
