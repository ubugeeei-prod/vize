// Generates the composed copies of the template generators.
//
// The SFC assembly composes a template's render map into the component's, and
// that map needs links a plain map does not carry. A plain compile must not
// pay for them, not even through the inlining a second caller of a shared
// callee changes, so each crate that emits such links is compiled twice: as
// itself, and as a generated copy in its own crate. The copy is this crate's
// source with one change: the crate's copy macro keeps its `composed` tokens
// instead of its `plain` ones. `--check` fails when a committed copy is stale.

import fs from "node:fs";
import path from "node:path";

import {
  commandArguments,
  isEntrypoint,
  printLine,
  repositoryRoot,
  runEntrypoint,
} from "../../support/command-runtime.ts";

const command = "tools/commands/generate/composed-crates.ts";

export interface ComposedCrate {
  /** The crate whose source is copied. */
  readonly plain: string;
  /** The generated crate. */
  readonly composed: string;
  /** The `macro_rules!` in the plain `lib.rs` that selects a copy's tokens. */
  readonly macro: string;
  readonly description: string;
  /** Modules the copy names from the plain crate instead of compiling, so a
   * host hands both compiles the same options, context and results. */
  readonly sharedModules: readonly SharedModule[];
  /** Crates the copy depends on in place of the plain crate's. */
  readonly composedDependencies: readonly string[];
  /** Further dependencies of the copy. */
  readonly extraDependencies: readonly string[];
}

export interface SharedModule {
  /** The source file that declares the module, from `src/`. */
  readonly file: string;
  readonly module: string;
  /** The plain crate's path the copy names the module by. */
  readonly path: string;
}

export const composedCrates: readonly ComposedCrate[] = [
  {
    plain: "vize_atelier_core",
    composed: "vize_atelier_core_composed",
    macro: "render_map_copy",
    description:
      "Generated copy of vize_atelier_core whose code generator records the render maps the SFC assembly composes",
    sharedModules: [
      { file: "codegen/context.rs", module: "result", path: "vize_atelier_core::codegen" },
      { file: "lane.rs", module: "script_tables", path: "vize_atelier_core::lane" },
    ],
    composedDependencies: [],
    extraDependencies: ["vize_atelier_core"],
  },
  {
    plain: "vize_atelier_dom",
    composed: "vize_atelier_dom_composed",
    macro: "dom_map_copy",
    description:
      "Generated copy of vize_atelier_dom whose SFC template compile records the render maps the SFC assembly composes",
    sharedModules: [
      { file: "lib.rs", module: "options", path: "vize_atelier_dom::options" },
      { file: "compile/selection.rs", module: "differential", path: "vize_atelier_dom::differential" },
    ],
    composedDependencies: ["vize_atelier_core"],
    extraDependencies: ["vize_atelier_dom"],
  },
  {
    plain: "vize_atelier_ssr",
    composed: "vize_atelier_ssr_composed",
    macro: "sfc_map_copy",
    description:
      "Generated copy of vize_atelier_ssr whose SSR compile records the render maps the SFC assembly composes",
    sharedModules: ["differential", "errors", "options", "shared"].map((module) => ({
      file: "lib.rs",
      module,
      path: `vize_atelier_ssr::${module}`,
    })),
    composedDependencies: ["vize_atelier_core"],
    extraDependencies: ["vize_atelier_ssr"],
  },
];

const rustHeader = (source: string) =>
  `// DO NOT EDIT. Generated from ${source} by \`node ${command}\`.\n` +
  `// Edit the source and run the command again.\n`;

const tomlHeader = (source: string) =>
  `# DO NOT EDIT. Generated from ${source} by \`node ${command}\`.\n` +
  `# Edit the source and run the command again.\n`;

/** Every module `source` declares only as `#[cfg(test)] mod name;`. A module
 * the plain crate compiles only for its tests can be one the copy compiles
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
 * modules, their children, and the modules the copy re-exports. */
export function compiledSources(srcDir: string, shared: readonly SharedModule[]): string[] {
  const kept: string[] = [];
  const visit = (file: string) => {
    kept.push(file);
    const source = fs.readFileSync(file, "utf8");
    const skipped = new Set(testOnlyModules(source));
    for (const module of shared) {
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
  visit(path.join(srcDir, "lib.rs"));
  return kept.sort();
}

/** The copy of one source file. */
export function composedSource(spec: ComposedCrate, relative: string, source: string): string {
  let text = source;
  if (relative === "lib.rs") {
    const pattern = new RegExp(
      `(macro_rules! ${spec.macro} \\{\\n    \\(plain \\{ \\$\\(\\$plain:tt\\)\\* \\} composed \\{ \\$\\(\\$composed:tt\\)\\* \\}\\) => \\{\\n        )\\$\\(\\$plain\\)\\*`,
    );
    if (!pattern.test(text)) throw new Error(`${spec.plain}/src/lib.rs has no \`${spec.macro}!\` to flip`);
    text = text.replace(pattern, "$1$($composed)*");
  }
  for (const shared of spec.sharedModules) {
    if (shared.file !== relative) continue;
    const declaration = new RegExp(`^(pub(?:\\([^)]*\\))? )?mod ${shared.module};$`, "m");
    const found = declaration.exec(text);
    if (found === null) throw new Error(`${spec.plain}/src/${relative} declares no \`mod ${shared.module};\``);
    const alias = shared.path.endsWith(`::${shared.module}`) ? "" : ` as ${shared.module}`;
    text = text.replace(declaration, `${found[1] ?? ""}use ${shared.path}${alias};`);
  }
  for (const dependency of spec.composedDependencies) {
    text = text.replace(new RegExp(`\\b${dependency}(?=::)`, "g"), `${dependency}_composed`);
  }
  return rustHeader(`crates/${spec.plain}/src/${relative}`) + text;
}

interface TomlSection {
  readonly header: string;
  readonly lines: string[];
}

function tomlSections(text: string): TomlSection[] {
  const sections: TomlSection[] = [{ header: "", lines: [] }];
  for (const line of text.replaceAll("\r\n", "\n").split("\n")) {
    if (/^\[{1,2}[^\]]+\]{1,2}\s*$/.test(line)) sections.push({ header: line.trim(), lines: [] });
    else sections.at(-1)?.lines.push(line);
  }
  return sections;
}

/** The copy's manifest: the plain package, features and dependencies, as a
 * library with no tests, benches or examples of its own. */
export function composedManifest(spec: ComposedCrate, manifest: string): string {
  const renamed = (line: string) =>
    spec.composedDependencies.reduce(
      (text, dependency) => text.replace(new RegExp(`\\b${dependency}\\b(?=[ /=])`, "g"), `${dependency}_composed`),
      line,
    );
  const out: string[] = [];
  for (const section of tomlSections(manifest)) {
    const body = section.lines.filter((line) => !line.trimStart().startsWith("#"));
    if (section.header === "[package]") {
      out.push("[package]");
      for (const line of body) {
        if (line.startsWith("name = ")) out.push(`name = "${spec.composed}"`);
        else if (line.startsWith("description = ")) out.push(`description = "${spec.description}"`);
        else out.push(line);
      }
    } else if (section.header === "[features]") {
      // A copy that re-exports modules of its plain crate arms the plain
      // crate's features with its own.
      const forwards = spec.extraDependencies.includes(spec.plain);
      // A copy has no published history, so it keeps no deprecated feature
      // spellings.
      const current = body.filter((line) => !/^davinci-[\w-]+ = /.test(line));
      out.push(
        "[features]",
        ...current.map((line) => {
          const feature = /^([\w-]+) = \[(.*)\]$/.exec(renamed(line));
          if (feature === null || !forwards || feature[1] === "default") return renamed(line);
          const enabled = [feature[2], `"${spec.plain}/${feature[1]}"`].filter((part) => part !== "");
          return `${feature[1]} = [${enabled.join(", ")}]`;
        }),
      );
    } else if (section.header === "[dependencies]") {
      out.push("[dependencies]", ...body.map(renamed).filter((line, index, all) => line !== "" || index < all.length - 1));
      for (const dependency of spec.extraDependencies) out.push(`${dependency} = { workspace = true }`);
      out.push("");
    } else if (section.header === "[lints]") {
      out.push("[lints]", ...body);
    }
  }
  out.push("[lib]", "test = false", "doctest = false", "bench = false", "");
  return (
    tomlHeader(`crates/${spec.plain}/Cargo.toml`) +
    out
      .join("\n")
      .replace(/\n{3,}/g, "\n\n")
      .replace(/\n*$/, "\n")
  );
}

function composedReadme(spec: ComposedCrate): string {
  return `<!-- DO NOT EDIT. Generated by \`node ${command}\`. -->

# ${spec.composed}

\`${spec.composed}\` is the source of [\`${spec.plain}\`](../${spec.plain}) compiled
again, with \`${spec.macro}!\` keeping its \`composed\` tokens. ${spec.description.replace(/^Generated copy of \w+ whose /, "Its ")}.
Hosts reach it only when they compose a render map, so a compile without one
runs \`${spec.plain}\` unchanged.

Do not edit this crate. Edit \`${spec.plain}\` and run \`node ${command}\`.

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).
`;
}

/** Every file of every copy, keyed by its path from the repository root. */
export function generatedFiles(root: string): Map<string, string> {
  const files = new Map<string, string>();
  for (const spec of composedCrates) {
    const plainDir = path.join(root, "crates", spec.plain);
    const srcDir = path.join(plainDir, "src");
    const outDir = path.join("crates", spec.composed);
    for (const file of compiledSources(srcDir, spec.sharedModules)) {
      const relative = path.relative(srcDir, file).split(path.sep).join("/");
      const source = fs.readFileSync(file, "utf8").replaceAll("\r\n", "\n");
      files.set(`${outDir}/src/${relative}`, composedSource(spec, relative, source));
    }
    const manifest = fs.readFileSync(path.join(plainDir, "Cargo.toml"), "utf8");
    files.set(`${outDir}/Cargo.toml`, composedManifest(spec, manifest));
    files.set(`${outDir}/README.md`, composedReadme(spec));
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
  for (const spec of composedCrates) walk(path.join(root, "crates", spec.composed));
  return found;
}

export function generateComposedCrates(args: readonly string[] = commandArguments()): number {
  const root = repositoryRoot(import.meta.url);
  const check = args.includes("--check");
  const files = generatedFiles(root);
  const existing = existingFiles(root);
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
  for (const relative of existing) {
    if (files.has(relative)) continue;
    stale.push(relative);
    if (!check) fs.rmSync(path.join(root, relative));
  }
  if (check && stale.length > 0) {
    printLine(`Composed crates are stale; run \`node ${command}\`:`);
    for (const relative of stale.sort()) printLine(`  ${relative}`);
    return 1;
  }
  if (!check) printLine(`Wrote ${stale.length} composed crate file(s).`);
  return 0;
}

if (isEntrypoint(import.meta.url)) runEntrypoint(() => generateComposedCrates());
