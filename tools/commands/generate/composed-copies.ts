// Generates the composed copies of the template generators.
//
// The SFC assembly composes a template's render map into the component's, and
// that map needs links a plain map does not carry. A plain compile must not
// pay for them, not even through the inlining a second caller of a shared
// callee changes, so each crate that emits such links compiles its source
// twice: as itself, and as a generated copy in its own `composed` module. The
// copy is the crate's source with three changes: the crate's copy macro keeps
// its `composed` tokens instead of its `plain` ones, `crate::` paths name the
// copy, and the copies of the crates it depends on stand in for those crates.
// The copy lives inside the crate it copies, so publishing it publishes no new
// crate. `--check` fails when a committed copy is stale.

import fs from "node:fs";
import path from "node:path";

import {
  commandArguments,
  isEntrypoint,
  printLine,
  repositoryRoot,
  runEntrypoint,
} from "../../support/command-runtime.ts";

const command = "tools/commands/generate/composed-copies.ts";

/** The module of each crate that holds its copy. */
export const copyModule = "composed";

export interface ComposedCopy {
  /** The crate whose source is copied. */
  readonly crate: string;
  /** The `macro_rules!` in `lib.rs` that selects a copy's tokens. */
  readonly macro: string;
  /** Modules the copy names from the crate instead of compiling, so a host
   * hands both compiles the same options, context and results. */
  readonly sharedModules: readonly SharedModule[];
  /** Modules of `lib.rs` the copy leaves out: they serve only tests. */
  readonly droppedModules: readonly string[];
  /** Crates whose copies the copy uses in place of the crates. */
  readonly composedDependencies: readonly string[];
}

export interface SharedModule {
  /** The source file that declares the module, from `src/`. */
  readonly file: string;
  readonly module: string;
  /** The crate path the copy names the module by. */
  readonly path: string;
}

export const composedCopies: readonly ComposedCopy[] = [
  {
    crate: "vize_atelier_core",
    macro: "render_map_copy",
    sharedModules: [
      { file: "codegen/context.rs", module: "result", path: "crate::codegen" },
      { file: "lane.rs", module: "script_tables", path: "crate::lane" },
    ],
    droppedModules: ["test_macros"],
    composedDependencies: [],
  },
  {
    crate: "vize_atelier_dom",
    macro: "dom_map_copy",
    sharedModules: [
      { file: "lib.rs", module: "options", path: "crate::options" },
      { file: "compile/selection.rs", module: "differential", path: "crate::differential" },
    ],
    droppedModules: [],
    composedDependencies: ["vize_atelier_core"],
  },
  {
    crate: "vize_atelier_ssr",
    macro: "sfc_map_copy",
    sharedModules: ["differential", "errors", "options", "shared"].map((module) => ({
      file: "lib.rs",
      module,
      path: `crate::${module}`,
    })),
    droppedModules: [],
    composedDependencies: ["vize_atelier_core"],
  },
];

const header = (source: string) =>
  `// DO NOT EDIT. Generated from ${source} by \`node ${command}\`.\n` +
  `// Edit the source and run the command again.\n`;

const moduleDeclaration = (name: string) =>
  new RegExp(`^\\s*(?:pub(?:\\([^)]*\\))?\\s+)?mod\\s+${name};$`, "m");

/** The declaration of the copy module in `lib.rs`, which the copy drops. */
const copyDeclaration = new RegExp(
  `^#\\[cfg\\(not\\(any\\(test, doctest\\)\\)\\)\\]\\n#\\[doc\\(hidden\\)\\]\\n#\\[rustfmt::skip\\]\\npub mod ${copyModule};\\n`,
  "m",
);

/** Every module `source` declares only as `#[cfg(test)] mod name;`. A module
 * the crate compiles only for its tests can be one the copy compiles
 * outright, so a declaration without the attribute keeps it. */
export function testOnlyModules(source: string): string[] {
  const declared = [...source.matchAll(/^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+);/gm)].map((match) => match[1]);
  const tested = testDeclarations(source);
  return [...new Set(tested)].filter(
    (name) => declared.filter((other) => other === name).length === tested.filter((other) => other === name).length,
  );
}

function testDeclarations(source: string): string[] {
  const lines = source.split("\n");
  const names: string[] = [];
  for (let index = 0; index < lines.length; index += 1) {
    if (lines[index]?.trim() !== "#[cfg(test)]") continue;
    let next = index + 1;
    // Skip further attributes, which may span lines.
    while (next < lines.length && lines[next]?.trim().startsWith("#[")) {
      let depth = 0;
      do {
        const line = lines[next] ?? "";
        depth += (line.match(/\[/g) ?? []).length - (line.match(/\]/g) ?? []).length;
        next += 1;
      } while (depth > 0 && next < lines.length);
    }
    const declaration = /^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+);/.exec(lines[next] ?? "");
    if (declaration?.[1] !== undefined) names.push(declaration[1]);
  }
  return names;
}

/** The directory holding the child modules of the module in `file`. */
function childDirectory(file: string): string {
  const base = path.basename(file);
  if (base === "lib.rs" || base === "mod.rs") return path.dirname(file);
  return file.slice(0, -".rs".length);
}

/** The source files of `srcDir` the copy compiles: every module but test
 * modules, their children, the modules the copy names from the crate, and
 * the copy itself. */
export function compiledSources(srcDir: string, spec: ComposedCopy): string[] {
  const kept: string[] = [];
  const root = path.join(srcDir, "lib.rs");
  const visit = (file: string) => {
    kept.push(file);
    const source = fs.readFileSync(file, "utf8");
    const skipped = new Set(testOnlyModules(source));
    if (file === root) for (const name of [copyModule, ...spec.droppedModules]) skipped.add(name);
    for (const module of spec.sharedModules) {
      if (file === path.join(srcDir, ...module.file.split("/"))) skipped.add(module.module);
    }
    const declared = new Set(
      [...source.matchAll(/^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+);/gm)].map((match) => match[1] ?? ""),
    );
    for (const name of [...declared].sort()) {
      if (skipped.has(name)) continue;
      const dir = childDirectory(file);
      const flat = path.join(dir, `${name}.rs`);
      const nested = path.join(dir, name, "mod.rs");
      const child = fs.existsSync(flat) ? flat : nested;
      if (!fs.existsSync(child)) throw new Error(`${file} declares missing module ${name}`);
      visit(child);
    }
  };
  visit(root);
  return kept.sort();
}

/** Removes the declaration of `name` and the attributes directly above it. */
function dropDeclaration(text: string, name: string, file: string): string {
  const lines = text.split("\n");
  const at = lines.findIndex((line) => moduleDeclaration(name).test(line));
  if (at === -1) throw new Error(`${file} declares no \`mod ${name};\``);
  let start = at;
  while (start > 0 && lines[start - 1]?.trimStart().startsWith("#[")) start -= 1;
  lines.splice(start, at - start + 1);
  return lines.join("\n");
}

/** The copy of one source file, `relative` to the crate's `src/`. */
export function composedSource(spec: ComposedCopy, relative: string, source: string): string {
  let text = source;
  const file = `crates/${spec.crate}/src/${relative}`;
  if (relative === "lib.rs") {
    const pattern = new RegExp(
      `(macro_rules! ${spec.macro} \\{\\n    \\(plain \\{ \\$\\(\\$plain:tt\\)\\* \\} composed \\{ \\$\\(\\$composed:tt\\)\\* \\}\\) => \\{\\n        )\\$\\(\\$plain\\)\\*`,
    );
    if (!pattern.test(text)) throw new Error(`${file} has no \`${spec.macro}!\` to flip`);
    text = text.replace(pattern, "$1$($composed)*");
    if (!copyDeclaration.test(text)) throw new Error(`${file} does not declare \`pub mod ${copyModule};\` as expected`);
    text = text.replace(copyDeclaration, "");
    for (const name of spec.droppedModules) text = dropDeclaration(text, name, file);
  }
  // `$crate::` too: the copy's macros name the copy.
  text = text.replace(/\bcrate::/g, `crate::${copyModule}::`);
  for (const dependency of spec.composedDependencies) {
    text = text.replace(new RegExp(`\\b${dependency}::`, "g"), `${dependency}::${copyModule}::`);
  }
  for (const shared of spec.sharedModules) {
    if (shared.file !== relative) continue;
    const found = new RegExp(`^(pub(?:\\([^)]*\\))? )?mod ${shared.module};$`, "m").exec(text);
    if (found === null) throw new Error(`${file} declares no \`mod ${shared.module};\``);
    const alias = shared.path.endsWith(`::${shared.module}`) ? "" : ` as ${shared.module}`;
    text = text.replace(found[0], `${found[1] ?? ""}use ${shared.path}${alias};`);
  }
  return header(file) + text;
}

/** Every file of every copy, keyed by its path from the repository root. */
export function generatedFiles(root: string): Map<string, string> {
  const files = new Map<string, string>();
  for (const spec of composedCopies) {
    const srcDir = path.join(root, "crates", spec.crate, "src");
    for (const file of compiledSources(srcDir, spec)) {
      const relative = path.relative(srcDir, file).split(path.sep).join("/");
      const source = fs.readFileSync(file, "utf8").replaceAll("\r\n", "\n");
      const target = relative === "lib.rs" ? `${copyModule}.rs` : `${copyModule}/${relative}`;
      files.set(`crates/${spec.crate}/src/${target}`, composedSource(spec, relative, source));
    }
  }
  return files;
}

function existingFiles(root: string): Set<string> {
  const found = new Set<string>();
  const walk = (dir: string) => {
    if (!fs.existsSync(dir)) return;
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else found.add(path.relative(root, full).split(path.sep).join("/"));
    }
  };
  for (const spec of composedCopies) {
    const src = path.join(root, "crates", spec.crate, "src");
    if (fs.existsSync(path.join(src, `${copyModule}.rs`))) found.add(`crates/${spec.crate}/src/${copyModule}.rs`);
    walk(path.join(src, copyModule));
  }
  return found;
}

/** Whether `file`, from the repository root, is a generated copy. */
export function isComposedCopy(file: string): boolean {
  return composedCopies.some(
    (spec) =>
      file === `crates/${spec.crate}/src/${copyModule}.rs` ||
      file.startsWith(`crates/${spec.crate}/src/${copyModule}/`),
  );
}

export function generateComposedCopies(args: readonly string[] = commandArguments()): number {
  const root = repositoryRoot(import.meta.url);
  const check = args.includes("--check");
  const files = generatedFiles(root);
  const stale: string[] = [];
  for (const [relative, text] of files) {
    const target = path.join(root, relative);
    const current = fs.existsSync(target) ? fs.readFileSync(target, "utf8") : undefined;
    if (current === text) continue;
    stale.push(relative);
    if (!check) {
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, text);
    }
  }
  for (const relative of existingFiles(root)) {
    if (files.has(relative)) continue;
    stale.push(relative);
    if (!check) fs.rmSync(path.join(root, relative));
  }
  if (check && stale.length > 0) {
    printLine(`Composed copies are stale; run \`node ${command}\`:`);
    for (const relative of stale.sort()) printLine(`  ${relative}`);
    return 1;
  }
  if (!check) printLine(`Wrote ${stale.length} composed copy file(s).`);
  return 0;
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => generateComposedCopies());
