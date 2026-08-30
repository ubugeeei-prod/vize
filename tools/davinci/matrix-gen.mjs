// Construct-matrix fixture generator — SKELETON (Davinci P0-12).
//
// Reads davinci-road/plan/taxonomy.toml (DRAFT, pending maintainer
// sign-off) and emits deterministic .vue fixture stubs under
// tests/fixtures/davinci-matrix/. This skeleton generates the
// element_kind x directive plane only; the full cross-product (modifier
// classes, binding sources, block combinations) arrives with the pipeline
// stages that consume the fixtures, which also bring the expected
// outputs. Stubs are source text only — no expected outputs, nothing is
// compiled here.
//
// Modes:
//   (default)        dry run — validate the taxonomy and print the
//                    would-be fixture count; nothing is written
//   --write          write the stubs to the target directory
//   --check          verify the on-disk fixture set byte-matches a fresh
//                    generation (exit 1 listing missing/stale/extra files)
//   --out-dir <dir>  override the target directory (self-tests write to a
//                    temp dir; the committed home is tests/fixtures/davinci-matrix)
//
// Determinism contract (assurance doctrine, "generated fixtures, not
// hand-picked examples"): output depends only on taxonomy.toml. Iteration
// follows taxonomy file order, stub content carries no timestamps,
// absolute paths, or environment data — two runs are byte-identical.
//
// Exit codes: 0 = success / up to date, 1 = --check found drift,
// 2 = usage or taxonomy validation error.

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { parseTomlLite, TomlLiteError } from "./toml-lite.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const taxonomyPath = path.join(repoRoot, "davinci-road", "plan", "taxonomy.toml");
const defaultOutDir = path.join(repoRoot, "tests", "fixtures", "davinci-matrix");

const ID_RE = /^[a-z][a-z0-9-]*$/;
const DIMENSIONS = [
  "element_kind",
  "directive",
  "modifier_class",
  "binding_source",
  "block_combination",
];

function fail(message) {
  console.error(message);
  process.exit(2);
}

function loadTaxonomy() {
  let parsed;
  try {
    parsed = parseTomlLite(fs.readFileSync(taxonomyPath, "utf8"));
  } catch (error) {
    if (error instanceof TomlLiteError)
      fail(`malformed taxonomy ${taxonomyPath}: ${error.message}`);
    throw error;
  }
  for (const dimension of DIMENSIONS) {
    const entries = parsed[dimension];
    if (!Array.isArray(entries) || entries.length === 0) {
      fail(`taxonomy ${taxonomyPath}: missing [[${dimension}]] entries`);
    }
    const ids = new Set();
    for (const entry of entries) {
      if (typeof entry.id !== "string" || !ID_RE.test(entry.id)) {
        fail(`taxonomy [[${dimension}]]: entry with missing or non-kebab-case id`);
      }
      if (ids.has(entry.id)) fail(`taxonomy [[${dimension}]]: duplicate id ${entry.id}`);
      ids.add(entry.id);
    }
  }
  for (const kind of parsed.element_kind) {
    if (typeof kind.representative !== "string" || kind.representative.length === 0) {
      fail(`taxonomy [[element_kind]] ${kind.id}: representative tag is required`);
    }
  }
  for (const directive of parsed.directive) {
    if (typeof directive.usage !== "string" || directive.usage.length === 0) {
      fail(`taxonomy [[directive]] ${directive.id}: usage attribute text is required`);
    }
  }
  return parsed;
}

// One stub per element_kind x directive pair, keyed by file name, in
// taxonomy order. Content is pure text derived from the two entries.
export function generateStubs(taxonomy) {
  const files = new Map();
  for (const kind of taxonomy.element_kind) {
    for (const directive of taxonomy.directive) {
      const name = `${kind.id}--${directive.id}.vue`;
      const tag = kind.representative;
      const lines = [
        "<!--",
        "  generated-by: tools/davinci/matrix-gen.mjs (P0-12 skeleton) — do not edit",
        `  construct: element_kind=${kind.id} directive=${directive.id}`,
        "-->",
        "<template>",
      ];
      if (directive.needs_prior_branch === true) {
        lines.push('  <div v-if="first"></div>');
      }
      lines.push(`  <${tag} ${directive.usage}></${tag}>`);
      lines.push("</template>");
      files.set(name, `${lines.join("\n")}\n`);
    }
  }
  return files;
}

function parseArgs(argv) {
  const args = { write: false, check: false, outDir: null };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--write") args.write = true;
    else if (arg === "--check") args.check = true;
    else if (arg === "--out-dir") {
      args.outDir = argv[i + 1];
      i += 1;
      if (args.outDir == null) fail("--out-dir requires a directory argument");
    } else if (arg === "--help" || arg === "-h") args.help = true;
    else fail(`unknown argument ${arg}`);
  }
  if (args.write && args.check) fail("--write and --check are mutually exclusive");
  return args;
}

const USAGE = `Usage: rust-script tools/commands/davinci/matrix-gen.rs [--write | --check] [--out-dir <dir>]

Generates construct-matrix fixture stubs from davinci-road/plan/taxonomy.toml.
Default is a dry run that prints the would-be fixture count.`;

function describe(taxonomy) {
  const counts = DIMENSIONS.map((dimension) => `${dimension}=${taxonomy[dimension].length}`);
  console.log(`taxonomy: ${counts.join(" ")} (${taxonomy.taxonomy?.status ?? "unknown"})`);
}

function relative(target) {
  const rel = path.relative(repoRoot, target);
  return rel.startsWith("..") ? target : rel.split(path.sep).join("/");
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    console.log(USAGE);
    return 0;
  }
  const taxonomy = loadTaxonomy();
  const stubs = generateStubs(taxonomy);
  const outDir = path.resolve(args.outDir ?? defaultOutDir);
  describe(taxonomy);
  const plane = `${taxonomy.element_kind.length} element kinds x ${taxonomy.directive.length} directives`;

  if (args.check) {
    const missing = [];
    const stale = [];
    for (const [name, content] of stubs) {
      const target = path.join(outDir, name);
      if (!fs.existsSync(target)) missing.push(name);
      else if (fs.readFileSync(target, "utf8") !== content) stale.push(name);
    }
    const extra = (fs.existsSync(outDir) ? fs.readdirSync(outDir) : [])
      .filter((name) => name.endsWith(".vue") && !stubs.has(name))
      .sort();
    if (missing.length + stale.length + extra.length > 0) {
      for (const name of missing) console.log(`missing: ${name}`);
      for (const name of stale) console.log(`stale: ${name}`);
      for (const name of extra) console.log(`extra: ${name}`);
      console.log(`matrix-gen: ${relative(outDir)} is out of date — rerun with --write and commit`);
      return 1;
    }
    console.log(`matrix-gen: ${stubs.size} fixture stubs up to date in ${relative(outDir)}`);
    return 0;
  }

  if (args.write) {
    fs.mkdirSync(outDir, { recursive: true });
    for (const [name, content] of stubs) {
      fs.writeFileSync(path.join(outDir, name), content);
    }
    const extra = fs
      .readdirSync(outDir)
      .filter((name) => name.endsWith(".vue") && !stubs.has(name))
      .sort();
    console.log(`matrix-gen: wrote ${stubs.size} fixture stubs (${plane}) to ${relative(outDir)}`);
    for (const name of extra) {
      console.warn(`warning: ${name} is not part of the generated set (would fail --check)`);
    }
    return 0;
  }

  console.log(`matrix-gen (skeleton): ${plane} -> ${stubs.size} fixture stubs`);
  console.log(
    `dry run — nothing written; target ${relative(outDir)} (--write to emit, --check to verify)`,
  );
  return 0;
}

const invokedDirectly =
  process.argv[1] != null && fileURLToPath(import.meta.url) === path.resolve(process.argv[1]);
if (invokedDirectly) {
  process.exitCode = main();
}
