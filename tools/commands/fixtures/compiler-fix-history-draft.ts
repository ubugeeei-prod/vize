// Drafts compiler fix-history specs (#6880) from the markup literals a fix
// commit added to its tests. The drafts are review input for
// compiler-fix-history-case.ts, never registered unread: each fix still needs
// its entry, target and options checked against what the commit fixed.
//
//   node tools/commands/fixtures/compiler-fix-history-draft.ts SHA... > drafts.json
//
// Every Rust string literal on an added test line whose text looks like a
// template (`<tag`, `{{`) or a whole SFC becomes one candidate. A literal that
// holds `<template>`, `<script` or `<style` is an SFC candidate; any other one
// is a bare DOM template candidate.
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { HISTORY_CRATES } from "./compiler-fix-history.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const scopes = HISTORY_CRATES.map((name) => `crates/${name}`);
const TEST_PATH = /(?:\/tests?\/|_tests?\.rs$|\/tests\.rs$|\.snap$)/;

export type Candidate = {
  file: string;
  /** The battery name written just before the literal, when there is one. */
  name: string | null;
  entry: "sfc" | "template" | "jsx";
  source: string;
};

function unescapeRust(body: string): string {
  return body.replace(
    /\\(?:u\{([0-9a-fA-F]+)\}|x([0-9a-fA-F]{2})|([nrt0\\"'])|\n\s*)/g,
    (match, unicode, hex, simple) => {
      if (unicode) return String.fromCodePoint(Number.parseInt(unicode, 16));
      if (hex) return String.fromCharCode(Number.parseInt(hex, 16));
      if (simple)
        return { n: "\n", r: "\r", t: "\t", "0": "\0", "\\": "\\", '"': '"', "'": "'" }[
          simple as "n"
        ];
      return ""; // line continuation
    },
  );
}

// Scan one added test hunk body for raw and ordinary string literals.
export type Literal = { text: string; start: number; end: number };

export function literals(text: string): Literal[] {
  const found: Literal[] = [];
  let index = 0;
  while (index < text.length) {
    const raw = /r(#*)"/y;
    raw.lastIndex = index;
    const rawMatch = raw.exec(text);
    if (rawMatch && (index === 0 || !/[A-Za-z0-9_]/.test(text[index - 1]))) {
      const close = `"${rawMatch[1]}`;
      const start = index + rawMatch[0].length;
      const end = text.indexOf(close, start);
      if (end < 0) break;
      found.push({ text: text.slice(start, end), start: index, end: end + close.length });
      index = end + close.length;
      continue;
    }
    if (text[index] === "'" && /^'(?:\\.|[^\\'])'/.test(text.slice(index, index + 4))) {
      index += text[index + 1] === "\\" ? 4 : 3;
      continue;
    }
    if (text[index] === "/" && text[index + 1] === "/") {
      const end = text.indexOf("\n", index);
      index = end < 0 ? text.length : end;
      continue;
    }
    if (text[index] === '"') {
      let end = index + 1;
      while (end < text.length && text[end] !== '"') end += text[end] === "\\" ? 2 : 1;
      found.push({ text: unescapeRust(text.slice(index + 1, end)), start: index, end: end + 1 });
      index = end + 1;
      continue;
    }
    index += 1;
  }
  return found;
}

// A whole SFC has a script or style block, or is one bare `<template>` block;
// a `<template #slot>` or `<template v-if>` fragment is template markup.
const SFC_BLOCK = /<(?:script|style)[\s>]/;
const SFC_TEMPLATE =
  /^\s*<template(?:\s+(?:lang|functional|src)(?:="[^"]*")?)*\s*>[\s\S]*<\/template>\s*$/;
const isSfc = (source: string) => SFC_BLOCK.test(source) || SFC_TEMPLATE.test(source);
// `format!` templates (`{name}` holes, `{{{{` escapes) and expected outputs
// (`import`/`function`/`export` code) are not authored inputs.
const FORMAT_HOLE = /(?<!\{)\{[A-Za-z_][A-Za-z0-9_]*\}(?!\})|\{\{\{\{/;
const OUTPUT = /^\s*(?:import\s|export\s|function\s|const\s|let\s|var\s|return\s)/;
const isInput = (source: string) =>
  !FORMAT_HOLE.test(source) && !OUTPUT.test(source) && /^\s*(?:<|\{\{)/.test(source);
const MARKUP = /<[A-Za-z][\w:.-]*[\s/>]|\{\{/;

export function candidates(sha: string): Candidate[] {
  const patch = execFileSync(
    "git",
    ["show", "--format=", "--no-ext-diff", "-U0", sha, "--", ...scopes],
    { cwd: root, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  );
  const added = new Map<string, string[]>();
  let current = "";
  for (const line of patch.split("\n")) {
    if (line.startsWith("+++ ")) {
      current = line.slice(6);
      continue;
    }
    if (line.startsWith("+") && TEST_PATH.test(current) && current.endsWith(".rs"))
      added.set(current, [...(added.get(current) ?? []), line.slice(1)]);
  }
  const out: Candidate[] = [];
  const seen = new Set<string>();
  for (const [file, lines] of added) {
    const text = lines.join("\n");
    let previous: Literal | null = null;
    for (const literal of literals(text)) {
      const source = literal.text;
      // Only a `("name", source)` tuple or a `name: "..", src: ..` field pair
      // names its source; any other neighbouring literal is unrelated.
      const name =
        previous &&
        /^[a-z][a-z0-9_]*$/.test(previous.text) &&
        /^\s*,\s*(?:(?:src|source|input):\s*)?$/.test(text.slice(previous.end, literal.start))
          ? previous.text
          : null;
      previous = literal;
      const jsx = file.startsWith("crates/vize_atelier_jsx/");
      if (!MARKUP.test(source) || seen.has(source) || (!jsx && !isInput(source))) continue;
      seen.add(source);
      const entry = jsx ? "jsx" : isSfc(source) ? "sfc" : "template";
      out.push({ file, name, entry, source });
    }
  }
  return out;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const drafts = process.argv.slice(2).map((sha) => ({ commit: sha, candidates: candidates(sha) }));
  process.stdout.write(`${JSON.stringify(drafts, null, 2)}\n`);
}
