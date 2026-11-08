import fs from "node:fs";
import path from "node:path";
import { runEntrypoint } from "../../support/command-runtime.ts";

const LEGACY_COMMAND_MARKER = "legacy_command";
const TOOL_HOST_MARKER = "tool_host";
const RUST_SCRIPT_BUDGET = 59;

interface Token {
  value: string;
}

function repositoryRoot(start = process.cwd()): string {
  let current = path.resolve(start);
  while (true) {
    if (
      fs.statSync(path.join(current, "Cargo.toml"), { throwIfNoEntry: false })?.isFile() &&
      fs.statSync(path.join(current, "pnpm-workspace.yaml"), { throwIfNoEntry: false })?.isFile()
    ) {
      return current;
    }
    const parent = path.dirname(current);
    if (parent === current) throw new Error(`cannot find repository root from ${start}`);
    current = parent;
  }
}

function collectFiles(directory: string): string[] {
  if (!fs.existsSync(directory)) return [];
  const files: string[] = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (entry.name === "node_modules") continue;
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) files.push(...collectFiles(filename));
    else if (entry.isFile()) files.push(filename);
  }
  return files;
}

function skipQuoted(source: string, start: number, quote: string): number {
  let index = start + 1;
  while (index < source.length) {
    if (source[index] === "\\") index += 2;
    else if (source[index++] === quote) break;
  }
  return index;
}

function rawStringEnd(source: string, start: number): number | undefined {
  const match = /^(?:br|rb|r)(#*)"/u.exec(source.slice(start));
  if (!match) return undefined;
  const terminator = `"${match[1]}`;
  const end = source.indexOf(terminator, start + match[0].length);
  return end < 0 ? source.length : end + terminator.length;
}

function rustTokens(source: string): Token[] {
  const tokens: Token[] = [];
  let index = 0;
  while (index < source.length) {
    const rawEnd = rawStringEnd(source, index);
    if (rawEnd !== undefined) {
      index = rawEnd;
      continue;
    }
    if (source.startsWith("//", index)) {
      index = source.indexOf("\n", index + 2);
      if (index < 0) break;
      continue;
    }
    if (source.startsWith("/*", index)) {
      let depth = 1;
      index += 2;
      while (index < source.length && depth > 0) {
        if (source.startsWith("/*", index)) {
          depth++;
          index += 2;
        } else if (source.startsWith("*/", index)) {
          depth--;
          index += 2;
        } else index++;
      }
      continue;
    }
    const character = source[index];
    if (character === '"') {
      index = skipQuoted(source, index, character);
      continue;
    }
    if (character === "'" && source[index + 2] === "'") {
      index += 3;
      continue;
    }
    if (character === "'" && source[index + 1] === "\\" && source[index + 3] === "'") {
      index += 4;
      continue;
    }
    const identifier = /^(?:r#)?[A-Za-z_][A-Za-z0-9_]*/u.exec(source.slice(index));
    if (identifier) {
      tokens.push({ value: identifier[0] });
      index += identifier[0].length;
      continue;
    }
    if ("{};#[]=".includes(character)) tokens.push({ value: character });
    index++;
  }
  return tokens;
}

function matchingBrace(tokens: readonly Token[], opening: number): number {
  let depth = 0;
  for (let index = opening; index < tokens.length; index++) {
    if (tokens[index].value === "{") depth++;
    if (tokens[index].value === "}" && --depth === 0) return index;
  }
  return tokens.length;
}

function visitItems(
  tokens: readonly Token[],
  base: string,
  modules: Set<string>,
  errors: string[],
): void {
  let pathAttribute = false;
  for (let index = 0; index < tokens.length; index++) {
    const token = tokens[index].value;
    if (token === "#" && tokens[index + 1]?.value === "[") {
      let end = index + 2;
      while (end < tokens.length && tokens[end].value !== "]") end++;
      pathAttribute ||= tokens.slice(index + 2, end).some(({ value }) => value === "path");
      index = end;
      continue;
    }
    if (token === "{") {
      index = matchingBrace(tokens, index);
      pathAttribute = false;
      continue;
    }
    if (token !== "mod") {
      if (token === ";") pathAttribute = false;
      continue;
    }
    const rawName = tokens[index + 1]?.value;
    const separator = tokens[index + 2]?.value;
    if (!rawName || !/^(?:r#)?[A-Za-z_][A-Za-z0-9_]*$/u.test(rawName)) continue;
    const name = rawName.replace(/^r#/u, "");
    if (pathAttribute) {
      errors.push(`${base}: ${name} must use ordinary module discovery`);
      pathAttribute = false;
      continue;
    }
    if (separator === "{") {
      const end = matchingBrace(tokens, index + 2);
      visitItems(tokens.slice(index + 3, end), path.join(base, name), modules, errors);
      index = end;
      continue;
    }
    if (separator !== ";") continue;
    const flat = path.join(base, `${name}.rs`);
    const directory = path.join(base, name, "mod.rs");
    const flatExists = fs.statSync(flat, { throwIfNoEntry: false })?.isFile() ?? false;
    const directoryExists = fs.statSync(directory, { throwIfNoEntry: false })?.isFile() ?? false;
    if (flatExists && !directoryExists) visitModule(flat, false, modules, errors);
    else if (!flatExists && directoryExists) visitModule(directory, false, modules, errors);
    else if (flatExists && directoryExists)
      errors.push(`ambiguous ordinary module: ${flat} and ${directory}`);
    else errors.push(`missing ordinary module: ${flat} or ${directory}`);
    index += 2;
  }
}

function visitModule(
  filename: string,
  root: boolean,
  modules: Set<string>,
  errors: string[],
): void {
  if (modules.has(filename)) return;
  modules.add(filename);
  let source: string;
  try {
    source = fs.readFileSync(filename, "utf8");
  } catch (error) {
    errors.push(`cannot read ${filename}: ${String(error)}`);
    return;
  }
  const base =
    root || path.basename(filename) === "mod.rs"
      ? path.dirname(filename)
      : filename.slice(0, -path.extname(filename).length);
  visitItems(rustTokens(source), base, modules, errors);
}

function relative(root: string, filename: string): string {
  return path.relative(root, filename).split(path.sep).join("/");
}

function verifyLayout(): { nodeCommands: number; rustScripts: number } {
  const root = repositoryRoot();
  const commandFiles = collectFiles(path.join(root, "tools/commands"));
  const rustFiles = commandFiles.filter((file) => file.endsWith(".rs"));
  const nodeFiles = commandFiles.filter((file) => /\.(?:[cm]?js|ts)$/u.test(file));
  const errors: string[] = [];
  for (const retired of [
    "tools/rust",
    "tools/support/legacy_command.rs",
    "tools/support/tool_host.rs",
  ])
    if (fs.existsSync(path.join(root, retired))) errors.push(`${retired} must not come back`);

  const modules = new Set<string>();
  let rustScripts = 0;
  for (const filename of rustFiles) {
    const source = fs.readFileSync(filename, "utf8");
    if (!source.startsWith("#!/usr/bin/env rust-script\n")) continue;
    rustScripts++;
    visitModule(filename, true, modules, errors);
  }
  if (rustScripts > RUST_SCRIPT_BUDGET)
    errors.push(`Rust Script command budget grew from ${RUST_SCRIPT_BUDGET} to ${rustScripts}`);

  for (const filename of rustFiles) {
    const command = relative(root, filename);
    const source = fs.readFileSync(filename, "utf8");
    if (!source.startsWith("#!/usr/bin/env rust-script\n") && !modules.has(filename))
      errors.push(
        `${command} must start with a rust-script shebang or be a reachable ordinary module`,
      );
    if (source.includes(LEGACY_COMMAND_MARKER))
      errors.push(`${command} must not reference the legacy command runner`);
    if (source.includes(`${TOOL_HOST_MARKER}::run(`))
      errors.push(`${command} must not proxy to Node tooling`);
    if (command.includes("-vize/"))
      errors.push(`${command} must use product-neutral editor buckets`);
  }
  if (nodeFiles.length === 0) errors.push("stable Node command surface must not be empty");
  if (errors.length > 0) throw new Error(errors.join("\n"));
  return { nodeCommands: nodeFiles.length, rustScripts };
}

function printModuleClosure(paths: readonly string[]): void {
  const root = repositoryRoot();
  const modules = new Set<string>();
  const errors: string[] = [];
  for (const requested of paths) {
    if (path.isAbsolute(requested) || requested.split(/[\\/]/u).includes("..")) {
      errors.push(`expected repository-relative module root: ${requested}`);
      continue;
    }
    visitModule(path.join(root, requested), true, modules, errors);
  }
  if (errors.length > 0) throw new Error(errors.join("\n"));
  console.log(JSON.stringify([...modules].map((file) => relative(root, file)).sort()));
}

runEntrypoint(() => {
  const args = process.argv.slice(2);
  if (args[0] === "--module-closure") {
    printModuleClosure(args.slice(1));
    return;
  }
  const { nodeCommands, rustScripts } = verifyLayout();
  console.log(
    `tool commands: verified ${nodeCommands} stable Node commands; ${rustScripts} Rust Script commands remain`,
  );
});
