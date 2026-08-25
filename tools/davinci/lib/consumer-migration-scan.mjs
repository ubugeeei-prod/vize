// Scanner for davinci-road/plan/consumer-migration-surfaces.md.
//
// It deliberately counts lexical crate/surface mentions, not semantic Rust
// resolution. Rust comments and string literals are stripped before matching;
// Cargo comments are stripped while dependency keys remain visible.

import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";

import { repoRoot } from "./paths.mjs";
import { stripRust } from "./rust-source.mjs";

export const SURFACES = [
  { id: "davinci", label: "Davinci", group: "stage", names: ["vize_davinci"] },
  { id: "s0", label: "S0/carton", group: "stage", names: ["vize_s0", "vize_carton"] },
  { id: "s1", label: "S1/sinopia", group: "stage", names: ["vize_s1", "vize_sinopia"] },
  { id: "s2", label: "S2/disegno", group: "stage", names: ["vize_s2", "vize_disegno"] },
  {
    id: "s1_to_s2",
    label: "S1->S2/ricalco",
    group: "stage",
    names: ["vize_s1_to_s2", "vize_ricalco"],
  },
  {
    id: "old_ast",
    label: "old AST/parser",
    group: "old",
    names: ["vize_relief", "vize_armature"],
  },
  {
    id: "croquis",
    label: "Croquis analysis",
    group: "old",
    names: ["vize_croquis", "vize_croquis_cf"],
  },
  {
    id: "raw_oxc",
    label: "raw OXC",
    group: "raw",
    names: [
      "oxc_allocator",
      "oxc_ast",
      "oxc_ast_visit",
      "oxc_codegen",
      "oxc_formatter",
      "oxc_formatter_core",
      "oxc_parser",
      "oxc_semantic",
      "oxc_syntax",
    ],
  },
];

export const CONSUMERS = [
  {
    id: "compiler",
    label: "Compiler",
    scope: "build command plus atelier compiler crates",
    entries: [
      { path: "crates/vize/src/commands/build.rs" },
      { path: "crates/vize/src/commands/build" },
      { crate: "vize_atelier_core" },
      { crate: "vize_atelier_dom" },
      { crate: "vize_atelier_sfc" },
      { crate: "vize_atelier_ssr" },
      { crate: "vize_atelier_vapor" },
      { crate: "vize_atelier_jsx" },
    ],
  },
  {
    id: "linter",
    label: "Linter",
    scope: "lint command plus Patina rule engine",
    entries: [
      { path: "crates/vize/src/commands/lint.rs" },
      { path: "crates/vize/src/commands/lint" },
      { crate: "vize_patina" },
    ],
  },
  {
    id: "typechecker",
    label: "Typechecker",
    scope: "check command plus Canon, excluding dedicated content-mapper files",
    entries: [
      { path: "crates/vize/src/commands/check.rs" },
      { path: "crates/vize/src/commands/check" },
      { crate: "vize_canon", exclude: (relPath) => isContentMapperPath(relPath) },
    ],
  },
  {
    id: "typechecker-content-mapper",
    label: "Typechecker content-mapper",
    scope: "content-mapper command plus Canon content-mapper protocol files",
    entries: [
      { path: "crates/vize/src/commands/content_mapper.rs" },
      { path: "crates/vize/src/commands/content_mapper" },
      {
        path: "crates/vize_canon/src/batch/virtual_project",
        include: (relPath) => isContentMapperPath(relPath),
      },
    ],
  },
  {
    id: "formatter",
    label: "Formatter",
    scope: "fmt command, Glyph formatter crate, and LSP format handler",
    entries: [
      { path: "crates/vize/src/commands/fmt.rs" },
      { path: "crates/vize/src/commands/fmt" },
      { crate: "vize_glyph" },
      { path: "crates/vize_maestro/src/server/format.rs" },
    ],
  },
  {
    id: "lsp",
    label: "LSP",
    scope: "lsp/ide commands plus Maestro editor/server crate",
    entries: [
      { path: "crates/vize/src/commands/lsp.rs" },
      { path: "crates/vize/src/commands/ide.rs" },
      { path: "crates/vize/src/commands/ide" },
      { crate: "vize_maestro" },
    ],
  },
];

const SURFACE_BY_ID = new Map(SURFACES.map((surface) => [surface.id, surface]));

function isContentMapperPath(relPath) {
  return relPath.includes("crates/vize_canon/src/batch/virtual_project/content_mapper");
}

function surfaceRegex(surface) {
  const alternatives = surface.names
    .slice()
    .sort((a, b) => b.length - a.length)
    .map((name) => name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  return new RegExp(`\\b(?:${alternatives.join("|")})\\b`, "g");
}

const SURFACE_REGEXES = SURFACES.map((surface) => [surface, surfaceRegex(surface)]);

function stripTomlComments(text) {
  return text
    .split("\n")
    .map((line) => {
      let quoted = false;
      for (let i = 0; i < line.length; i++) {
        if (line[i] === '"' && line[i - 1] !== "\\") quoted = !quoted;
        if (line[i] === "#" && !quoted) return line.slice(0, i);
      }
      return line;
    })
    .join("\n");
}

function lineOfIndex(text, index) {
  let line = 1;
  for (let i = 0; i < index; i++) if (text[i] === "\n") line++;
  return line;
}

function walkFiles(root) {
  const files = [];
  if (!existsSync(root)) return files;
  const stat = statSync(root);
  if (stat.isFile()) return [root];
  const visit = (dir) => {
    for (const dirent of readdirSync(dir, { withFileTypes: true }).sort((a, b) =>
      a.name < b.name ? -1 : a.name > b.name ? 1 : 0,
    )) {
      const full = path.join(dir, dirent.name);
      if (dirent.isDirectory()) visit(full);
      else if (dirent.isFile()) files.push(full);
    }
  };
  visit(root);
  return files;
}

function crateFiles(crateDir) {
  const root = path.join(repoRoot, "crates", crateDir);
  return [
    path.join(root, "Cargo.toml"),
    ...walkFiles(path.join(root, "src")),
    ...walkFiles(path.join(root, "tests")),
    ...walkFiles(path.join(root, "benches")),
  ];
}

function isScannable(file) {
  return file.endsWith(".rs") || file.endsWith("Cargo.toml");
}

function fileMode(relPath) {
  if (relPath.endsWith("/Cargo.toml")) return "manifest";
  const base = path.basename(relPath);
  if (relPath.includes("/tests/") || relPath.includes("/benches/")) return "test";
  if (base === "tests.rs" || base.endsWith("_tests.rs")) return "test";
  return "source";
}

function siteMode(relPath, stripped, index) {
  const mode = fileMode(relPath);
  if (mode !== "source") return mode;
  const cfgTest = stripped.indexOf("#[cfg(test)]");
  return cfgTest !== -1 && index >= cfgTest ? "test" : "source";
}

function filesForEntry(entry) {
  const files = entry.crate ? crateFiles(entry.crate) : walkFiles(path.join(repoRoot, entry.path));
  return files
    .filter(isScannable)
    .map((file) => path.relative(repoRoot, file).split(path.sep).join("/"))
    .filter((relPath) => (entry.include ? entry.include(relPath) : true))
    .filter((relPath) => (entry.exclude ? !entry.exclude(relPath) : true));
}

function collectFiles(consumer) {
  const files = new Set();
  for (const entry of consumer.entries) {
    for (const file of filesForEntry(entry)) files.add(file);
  }
  return [...files].sort((a, b) => a.localeCompare(b));
}

function scanFile(relPath) {
  const file = path.join(repoRoot, relPath);
  const text = readFileSync(file, "utf8");
  const stripped = relPath.endsWith("Cargo.toml") ? stripTomlComments(text) : stripRust(text);
  const sites = [];
  for (const [surface, regex] of SURFACE_REGEXES) {
    regex.lastIndex = 0;
    let match;
    while ((match = regex.exec(stripped)) !== null) {
      sites.push({
        relPath,
        line: lineOfIndex(stripped, match.index),
        mode: siteMode(relPath, stripped, match.index),
        surfaceId: surface.id,
        group: surface.group,
      });
    }
  }
  return sites;
}

function emptySurfaceCounts() {
  return Object.fromEntries(SURFACES.map((surface) => [surface.id, 0]));
}

function summarizeConsumer(consumer) {
  const files = collectFiles(consumer);
  const sites = files.flatMap(scanFile);
  const groupCounts = { stage: 0, old: 0, raw: 0 };
  const modeCounts = { source: 0, manifest: 0, test: 0 };
  const surfaceCounts = emptySurfaceCounts();
  const fileRows = new Map();

  for (const site of sites) {
    groupCounts[site.group] += 1;
    modeCounts[site.mode] += 1;
    surfaceCounts[site.surfaceId] += 1;
    const key = `${site.relPath}\0${site.mode}`;
    const row = fileRows.get(key) ?? {
      relPath: site.relPath,
      mode: site.mode,
      firstLine: site.line,
      total: 0,
      surfaceCounts: emptySurfaceCounts(),
    };
    row.firstLine = Math.min(row.firstLine, site.line);
    row.total += 1;
    row.surfaceCounts[site.surfaceId] += 1;
    fileRows.set(key, row);
  }

  return {
    ...consumer,
    fileCount: files.length,
    surfaceFileCount: new Set(sites.map((site) => site.relPath)).size,
    sites,
    groupCounts,
    modeCounts,
    surfaceCounts,
    fileRows: [...fileRows.values()].sort((a, b) =>
      a.relPath === b.relPath ? a.mode.localeCompare(b.mode) : a.relPath.localeCompare(b.relPath),
    ),
  };
}

export function surfaceLabel(id) {
  return SURFACE_BY_ID.get(id)?.label ?? id;
}

export function scanConsumerMigrationSurfaces() {
  return {
    surfaces: SURFACES,
    consumers: CONSUMERS.map(summarizeConsumer),
  };
}
