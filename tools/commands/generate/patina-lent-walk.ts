/**
 * Generate Patina's lifetime-carrying walk from checksum-pinned OXC source.
 * --source defaults to the pinned checkout in CARGO_HOME. --check generates
 * in a temporary directory and compares every output byte after formatting.
 * Rust tests compare the generated traversal's observable hooks with OXC.
 */
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { homedir, tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const revision = "fc702c1fa9f0412d06ec6908b58cd395b826cf7f";
const sourcePath = "crates/oxc_ast_visit/src/generated/visit.rs";
const sourceSha256 = "5440cd4c4a2e10effd6b9eafc8d7a283258884b91eae1feef76988ca6760490e";
const entryPoints = ["walk_program", "walk_expression", "walk_jsx_expression_container"];
const expectedOmissions = ["walk_argument", "walk_spans"];

type Token = { text: string; start: number; end: number };
type Edit = { start: number; end: number; replacement: string };
type Walk = { name: string; body: string };
type Options = {
  source?: string;
  outputDir: string;
  rustfmt: string;
  oxfmt: string;
  check: boolean;
};

/** Skip literals/comments so their delimiters never participate in balancing. */
function* tokens(source: string, from = 0): Generator<Token> {
  let at = from;
  while (at < source.length) {
    if (/\s/u.test(source[at]!)) {
      at += 1;
      continue;
    }
    if (source.startsWith("//", at)) {
      const end = source.indexOf("\n", at);
      at = end < 0 ? source.length : end + 1;
      continue;
    }
    if (source.startsWith("/*", at)) {
      let depth = 1;
      at += 2;
      while (depth !== 0) {
        if (at >= source.length) throw new Error("unterminated Rust block comment");
        if (source.startsWith("/*", at)) {
          depth += 1;
          at += 2;
        } else if (source.startsWith("*/", at)) {
          depth -= 1;
          at += 2;
        } else {
          at += 1;
        }
      }
      continue;
    }
    const raw = /^(?:br|cr|r)(#*)"/u.exec(source.slice(at));
    if (raw) {
      const closing = '"' + raw[1]!;
      const end = source.indexOf(closing, at + raw[0].length);
      if (end < 0) throw new Error("unterminated Rust raw string");
      at = end + closing.length;
      continue;
    }
    if (source[at] === '"') {
      at += 1;
      while (at < source.length && source[at] !== '"') {
        at += source[at] === "\\" ? 2 : 1;
      }
      if (at >= source.length) throw new Error("unterminated Rust string");
      at += 1;
      continue;
    }
    if (source[at] === "'") {
      const character = /^'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'/u.exec(
        source.slice(at),
      );
      if (character) {
        at += character[0].length;
        continue;
      }
    }
    // A lifetime apostrophe is deliberately a separate punctuation token.
    const word = /^[A-Za-z_][A-Za-z_0-9]*/u.exec(source.slice(at));
    const end = at + (word?.[0].length ?? 1);
    yield { text: source.slice(at, end), start: at, end };
    at = end;
  }
}

/** Return the byte-independent JS string offset just after the closing token. */
function matching(source: string, opening: number): number {
  const pairs = new Map([
    ["(", ")"],
    ["[", "]"],
    ["{", "}"],
  ]);
  const closings = new Set(pairs.values());
  const stack: string[] = [];
  for (const token of tokens(source, opening)) {
    const closing = pairs.get(token.text);
    if (closing) stack.push(closing);
    else if (closings.has(token.text)) {
      if (stack.pop() !== token.text) throw new Error("mismatched Rust delimiters");
      if (stack.length === 0) return token.end;
    }
  }
  throw new Error("unclosed Rust delimiter");
}

function applyEdits(source: string, edits: Edit[]): string {
  let previous = source.length + 1;
  edits.sort((a, b) => b.start - a.start || b.end - a.end);
  for (const edit of edits) {
    if (edit.end > previous) throw new Error("overlapping source rewrites");
    source = source.slice(0, edit.start) + edit.replacement + source.slice(edit.end);
    previous = edit.start;
  }
  return source;
}

function removeNoopHooks(source: string): string {
  source = source.replace(/^\s*let kind = AstKind::\w+\(visitor\.alloc\(it\)\);\n/gmu, "\n");
  const parsed = [...tokens(source)];
  const edits: Edit[] = [];
  const noops = new Set(["enter_node", "leave_node", "enter_scope", "leave_scope"]);
  for (let at = 0; at < parsed.length - 3; at += 1) {
    const [receiver, dot, method, paren] = parsed.slice(at, at + 4) as [Token, Token, Token, Token];
    if (
      receiver.text !== "visitor" ||
      dot.text !== "." ||
      !noops.has(method.text) ||
      paren.text !== "("
    )
      continue;
    let end = matching(source, paren.start);
    if (source[end] !== ";") throw new Error("hook invocation is no longer a standalone statement");
    end += 1;
    const start = source.lastIndexOf("\n", receiver.start - 1) + 1;
    if (source.slice(start, receiver.start).trim())
      throw new Error("hook invocation is no longer on its own line");
    if (source[end] === "\n") end += 1;
    edits.push({ start, end, replacement: "" });
  }
  return applyEdits(source, edits);
}

function directWalkCalls(source: string): string {
  const parsed = [...tokens(source)];
  const edits: Edit[] = [];
  for (let at = 0; at < parsed.length - 3; at += 1) {
    const [receiver, dot, method, paren] = parsed.slice(at, at + 4) as [Token, Token, Token, Token];
    if (
      receiver.text !== "visitor" ||
      dot.text !== "." ||
      !method.text.startsWith("visit_") ||
      paren.text !== "("
    )
      continue;
    matching(source, paren.start);
    edits.push({
      start: receiver.start,
      end: method.end,
      replacement: "walk_" + method.text.slice(6),
    });
    edits.push({ start: paren.end, end: paren.end, replacement: "visitor, " });
  }
  return applyEdits(source, edits);
}

/** Match Python textwrap.dedent for this pinned source's space-indented functions. */
function dedent(source: string): string {
  const margins = source
    .split("\n")
    .filter((line) => line.trim())
    .map((line) => /^ */u.exec(line)![0].length);
  const margin = Math.min(...margins);
  return (
    source
      .split("\n")
      .map((line) => (line.trim() ? line.slice(margin) : ""))
      .join("\n")
      .trim() + "\n"
  );
}

function makeFunction(original: string, name: string): Walk {
  let body = dedent(original).replace("pub fn ", "pub(crate) fn ");
  const bounds = [...body.matchAll(/V: Visit<'a>/gu)].length;
  const borrows = [...body.matchAll(/\bit: &/gu)].length;
  if (bounds !== 1 || borrows !== 1) throw new Error("unexpected signature: " + name);
  body = body.replace(/V: Visit<'a>/gu, "V: LentNodes<'a>").replace(/\bit: &/gu, "it: &'a ");
  body = directWalkCalls(removeNoopHooks(body));
  const opening = [...tokens(body)].find((token) => token.text === "{")?.start;
  if (opening === undefined) throw new Error("missing function body: " + name);
  const closing = matching(body, opening) - 1;
  let before = "";
  let after = "";
  if (name === "walk_jsx_element" || name === "walk_jsx_fragment") {
    const kind = name.slice(5);
    before = `\n    if !visitor.enter_${kind}(it) {\n        return;\n    }`;
    after = `    visitor.leave_${kind}(it);\n`;
  } else if (name === "walk_call_expression") {
    before = "\n    visitor.call_expression(it);";
  }
  body = body.slice(0, closing) + after + body.slice(closing);
  body = body.slice(0, opening + 1) + before + body.slice(opening + 1);
  const words = [...tokens(body)].map((token) => token.text);
  for (const argument of ["visitor", "it", "flags"]) {
    if (words.filter((word) => word === argument).length === 1) {
      body = body.replace(new RegExp(`\\b${argument}\\b`, "gu"), "_" + argument);
    }
  }
  return { name, body };
}

function walkCalls(body: string): Set<string> {
  const parsed = [...tokens(body)];
  return new Set(
    parsed
      .filter((token, at) => /^walk_\w+$/u.test(token.text) && parsed[at + 1]?.text === "(")
      .map((token) => token.text),
  );
}

/** Static reachability is computed before chunk emission, without rewriting paths. */
function reachableFunctions(functions: Walk[]): Set<string> {
  const byName = new Map(functions.map((walk) => [walk.name, walk]));
  const reachable = new Set<string>();
  const queue = [...entryPoints];
  for (let at = 0; at < queue.length; at += 1) {
    const name = queue[at]!;
    if (reachable.has(name)) continue;
    const walk = byName.get(name);
    if (!walk) throw new Error("unknown walk dependency: " + name);
    reachable.add(name);
    queue.push(...walkCalls(walk.body));
  }
  return reachable;
}

function chunkSource(chunk: Walk[]): { contents: string; external: Set<string> } {
  const body = chunk.map((walk) => walk.body).join("\n");
  const parsed = [...tokens(body)].map((token) => token.text);
  const identifiers = new Set(
    parsed.filter(
      (word, at) => /^[A-Z]/u.test(word) && !(parsed[at - 2] === ":" && parsed[at - 1] === ":"),
    ),
  );
  const excluded = new Set(["ArenaVec", "LentNodes", "ScopeFlags", "V", "Some", "None", "Option"]);
  const ast = new Set([...identifiers].filter((word) => !excluded.has(word)));
  for (const match of body.matchAll(/\b(match_\w+)!/gu)) ast.add(match[1]!);
  const defined = new Set(chunk.map((walk) => walk.name));
  const external = new Set([...walkCalls(body)].filter((name) => !defined.has(name)));
  const imports = ["use super::super::LentNodes;"];
  if (external.size) imports.push("use super::{" + [...external].sort().join(", ") + "};");
  if (identifiers.has("ArenaVec")) imports.push("use oxc_allocator::ArenaVec;");
  if (ast.size) imports.push("use oxc_ast::ast::{" + [...ast].sort().join(", ") + "};");
  if (identifiers.has("ScopeFlags")) imports.push("use oxc_syntax::scope::ScopeFlags;");
  return {
    contents:
      "// Generated from OXC's pinned immutable walk; see mod.rs for provenance.\n" +
      imports.join("\n") +
      "\n\n" +
      body,
    external,
  };
}

function parseOptions(): Options {
  const root = fileURLToPath(new URL("../../../", import.meta.url));
  const options: Options = {
    outputDir: path.join(root, "crates/vize_patina/src/lent_walk/generated"),
    rustfmt: "rustfmt",
    oxfmt: path.join(root, "node_modules/.bin/oxfmt"),
    check: false,
  };
  const args = process.argv.slice(2);
  for (let at = 0; at < args.length; at += 1) {
    const argument = args[at]!;
    if (argument === "--check") options.check = true;
    else if (argument === "--help" || argument === "-h") {
      process.stdout.write(
        "Usage: node tools/commands/generate/patina-lent-walk.ts [--source PATH] [--output-dir PATH] [--rustfmt COMMAND] [--oxfmt COMMAND] [--check]\n",
      );
      process.exit(0);
    } else {
      const equal = argument.indexOf("=");
      const key = equal < 0 ? argument : argument.slice(0, equal);
      if (!["--source", "--output-dir", "--rustfmt", "--oxfmt"].includes(key))
        throw new Error("unknown argument: " + argument);
      const value = equal < 0 ? args[++at] : argument.slice(equal + 1);
      if (!value || value.startsWith("--")) throw new Error("missing value for " + key);
      if (key === "--source") options.source = path.resolve(value);
      else if (key === "--output-dir") options.outputDir = path.resolve(value);
      else if (key === "--rustfmt") options.rustfmt = value;
      else options.oxfmt = value;
    }
  }
  return options;
}

function findSource(): string {
  const cargoHome = process.env.CARGO_HOME ?? path.join(homedir(), ".cargo");
  const checkouts = path.join(cargoHome, "git/checkouts");
  const candidates: string[] = [];
  if (existsSync(checkouts)) {
    for (const checkout of readdirSync(checkouts, { withFileTypes: true })) {
      if (
        !checkout.name.startsWith("oxc-") ||
        !(checkout.isDirectory() || checkout.isSymbolicLink())
      )
        continue;
      const directory = path.join(checkouts, checkout.name);
      for (const commit of readdirSync(directory, { withFileTypes: true })) {
        if (
          !commit.name.startsWith(revision.slice(0, 7)) ||
          !(commit.isDirectory() || commit.isSymbolicLink())
        )
          continue;
        const candidate = path.join(directory, commit.name, sourcePath);
        if (existsSync(candidate)) candidates.push(candidate);
      }
    }
  }
  if (candidates.length !== 1)
    throw new Error("pass --source or fetch the pinned OXC dependency with cargo fetch --locked");
  return candidates[0]!;
}

function sha256(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function lines(source: string): number {
  if (source.length === 0) return 0;
  return source.split("\n").length - (source.endsWith("\n") ? 1 : 0);
}

function generate(
  options: Options,
  outputDir: string,
): { functions: number; chunks: number; sourceFunctionCount: number; omitted: string[] } {
  const sourceFile = options.source ?? findSource();
  const raw = readFileSync(sourceFile);
  if (sha256(raw) !== sourceSha256)
    throw new Error("source hash changed; review OXC traversal before regenerating");
  const source = raw.toString("utf8");
  const start = source.indexOf("pub mod walk {");
  if (start < 0) throw new Error("missing OXC walk module");
  const opening = source.indexOf("{", start);
  const walked = source.slice(opening + 1, matching(source, opening) - 1);
  const functions: Walk[] = [];
  for (const found of walked.matchAll(/^    (?:#\[inline\]\n    )?pub fn (walk_\w+)/gmu)) {
    const from = found.index + found[0].length;
    const body = [...tokens(walked, from)].find((token) => token.text === "{")?.start;
    if (body === undefined) throw new Error("missing OXC walk body: " + found[1]!);
    functions.push(makeFunction(walked.slice(found.index, matching(walked, body)), found[1]!));
  }
  if (functions.length !== 260)
    throw new Error(`expected 260 OXC walk functions, found ${functions.length}`);
  const reachable = reachableFunctions(functions);
  const omitted = functions.filter((walk) => !reachable.has(walk.name)).map((walk) => walk.name);
  if (JSON.stringify(omitted) !== JSON.stringify(expectedOmissions) || reachable.size !== 258)
    throw new Error("unexpected unreachable walk functions: " + omitted.join(", "));
  const retained = functions.filter((walk) => reachable.has(walk.name));
  const combined = retained.map((walk) => walk.body).join("\n");
  const words = new Set([...tokens(combined)].map((token) => token.text));
  if (["unsafe", "alloc", "AstKind"].some((word) => words.has(word)))
    throw new Error("unexpected unchecked lifetime operation remains");
  const hooks = new Set([...combined.matchAll(/visitor\.(\w+)\(/gu)].map((match) => match[1]!));
  const expectedHooks = [
    "enter_jsx_element",
    "leave_jsx_element",
    "enter_jsx_fragment",
    "leave_jsx_fragment",
    "call_expression",
  ];
  if (hooks.size !== expectedHooks.length || expectedHooks.some((hook) => !hooks.has(hook)))
    throw new Error("unexpected residual visitor hooks: " + [...hooks].join(", "));

  // Keep the source-derived boundaries used by the original 260-function
  // generator. Prune within those chunks so every retained private path stays
  // stable, without inserting padding or suppressing dead-code diagnostics.
  const header = "// Generated from OXC's pinned immutable walk; see mod.rs for provenance.\n";
  const sourceChunks: Walk[][] = [];
  let chunk: Walk[] = [];
  for (const walk of functions) {
    if (
      chunk.length &&
      lines(header + chunk.map((item) => item.body).join("\n") + walk.body) > 450
    ) {
      sourceChunks.push(chunk);
      chunk = [];
    }
    chunk.push(walk);
  }
  if (chunk.length) sourceChunks.push(chunk);
  const chunks = sourceChunks.map((items) => items.filter((walk) => reachable.has(walk.name)));
  if (chunks.some((items) => items.length === 0))
    throw new Error("reachability pruning emptied a source chunk; review module paths");
  mkdirSync(outputDir, { recursive: true });
  const called = new Set(entryPoints);
  for (const [index, items] of chunks.entries()) {
    const rendered = chunkSource(items);
    for (const name of rendered.external) called.add(name);
    writeFileSync(
      path.join(outputDir, `part_${String(index + 1).padStart(2, "0")}.rs`),
      rendered.contents,
    );
  }
  const declarations = chunks
    .map((items, index) => {
      const name = `part_${String(index + 1).padStart(2, "0")}`;
      return (
        `mod ${name};\npub(super) use ${name}::{` +
        items
          .filter((walk) => called.has(walk.name))
          .map((walk) => walk.name)
          .join(", ") +
        "};"
      );
    })
    .join("\n");
  const root = `//! Private lifetime-carrying immutable AST traversal.
//!
//! Adapted from OXC ${revision}, \`${sourcePath}\`.
//! Original source SHA-256: \`${sourceSha256}\`.
//! The 258 walk bodies reachable from the three entry points retain the
//! order and AST matches of the pinned source's 260 walk bodies. Only the
//! five LentNodes hooks are observable; OXC's other hooks were no-ops for
//! the former Lending visitor. Every node input now borrows for \`'a\`, so
//! rustc proves that the hooks may retain it for the root's borrow.
//!
//! OXC copyright (c) 2024-present VoidZero Inc. & Contributors; (c) 2023
//! Boshen. The complete MIT license is in the adjacent LICENSE file.

${declarations}
`;
  writeFileSync(path.join(outputDir, "mod.rs"), root);
  const license = path.resolve(path.dirname(realpathSync(sourceFile)), "../../../../LICENSE");
  writeFileSync(path.join(outputDir, "LICENSE"), readFileSync(license));
  const formatted = spawnSync(
    options.rustfmt,
    ["--edition", "2024", path.join(outputDir, "mod.rs")],
    { stdio: "inherit" },
  );
  if (formatted.error) throw formatted.error;
  if (formatted.status !== 0)
    throw new Error(`rustfmt failed: ${formatted.signal ?? formatted.status}`);
  const outputs: Record<string, string> = {};
  for (const name of readdirSync(outputDir).sort()) {
    if (name !== "provenance.json")
      outputs[name] = sha256(readFileSync(path.join(outputDir, name)));
  }
  writeFileSync(
    path.join(outputDir, "provenance.json"),
    JSON.stringify(
      {
        revision,
        source: sourcePath,
        sha256: sourceSha256,
        source_function_count: functions.length,
        functions: retained.length,
        omitted_unreachable: omitted,
        chunks: chunks.length,
        function_names: retained.map((walk) => walk.name),
        outputs,
      },
      null,
      2,
    ) + "\n",
  );
  const metadata = spawnSync(options.oxfmt, [path.join(outputDir, "provenance.json")], {
    stdio: "inherit",
  });
  if (metadata.error) throw metadata.error;
  if (metadata.status !== 0) throw new Error(`oxfmt failed: ${metadata.signal ?? metadata.status}`);
  return {
    functions: retained.length,
    chunks: chunks.length,
    sourceFunctionCount: functions.length,
    omitted,
  };
}

function compareOutputs(expectedDir: string, actualDir: string): void {
  const expected = readdirSync(expectedDir).sort();
  const actual = readdirSync(actualDir).sort();
  if (
    JSON.stringify(expected) !== JSON.stringify(actual) ||
    expected.some(
      (name) =>
        !readFileSync(path.join(expectedDir, name)).equals(
          readFileSync(path.join(actualDir, name)),
        ),
    )
  ) {
    throw new Error("generated Patina walk differs; regenerate from the pinned OXC source");
  }
}

function main(): void {
  const options = parseOptions();
  const temporary = options.check ? mkdtempSync(path.join(tmpdir(), "vize-lent-walk-")) : undefined;
  try {
    const result = generate(options, temporary ?? options.outputDir);
    if (temporary) compareOutputs(temporary, options.outputDir);
    process.stdout.write(
      JSON.stringify({
        output_dir: options.outputDir,
        functions: result.functions,
        source_function_count: result.sourceFunctionCount,
        omitted_unreachable: result.omitted,
        chunks: result.chunks,
        checked: options.check,
      }) + "\n",
    );
  } finally {
    if (temporary) rmSync(temporary, { recursive: true, force: true });
  }
}

try {
  main();
} catch (error) {
  process.stderr.write((error instanceof Error ? error.message : String(error)) + "\n");
  process.exitCode = 1;
}
