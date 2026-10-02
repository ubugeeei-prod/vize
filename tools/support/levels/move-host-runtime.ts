#!/usr/bin/env node
// Replay the #6834 host Corsa owner move on current main.
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const FILES = [
  "corsa_api_mode.rs",
  "corsa_resolver.rs",
  "corsa_resolver/normalize_tests.rs",
  "corsa_resolver/tests.rs",
  "corsa_resolver/tests/package_runtime.rs",
] as const;
const CONSUMERS = ["vize", "vize_canon", "vize_maestro", "vize_patina"] as const;
type Phase = "moves" | "integrate" | "check";

function update(root: string, relative: string, change: (source: string) => string) {
  const filename = path.join(root, relative);
  const before = readFileSync(filename, "utf8");
  const after = change(before);
  if (before !== after) writeFileSync(filename, after);
}

function* rustSources(directory: string): Generator<string> {
  if (!existsSync(directory)) return;
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) yield* rustSources(filename);
    else if (entry.isFile() && entry.name.endsWith(".rs")) yield filename;
  }
}

const HOST = /^(?:r#)?corsa_(?:api_mode|resolver)(?![\p{ID_Continue}])/u;
const GROUP = /(?<![\p{ID_Continue}])([ \t]*)((?:pub(?:\([^)]*\))?\s+)?use)\s+vize_l0\s*::\s*\{/gu;
const RAW_STRING = /^(?:br|cr|r)(#*)"/u;
const CHAR = /^'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F_]+\}|[^\n])|[^\\'\n])'/u;
const LEX_START = /\/\/|\/\*|(?<![\p{ID_Continue}])(?:br|cr|r)#*"|["']/gu;
const DIRECT =
  /(?<![\p{ID_Continue}])vize_l0(?=\s*::\s*(?:r#)?corsa_(?:api_mode|resolver)(?![\p{ID_Continue}]))/gu;

/** Mask Rust comments/literals without changing offsets or line boundaries. */
export function lexicalView(text: string): string {
  const out = text.split("");
  const starts = new RegExp(LEX_START);
  for (let i = 0; i < text.length;) {
    starts.lastIndex = i;
    const token = starts.exec(text);
    if (!token) break;
    i = token.index;
    const raw = RAW_STRING.exec(text.slice(i));
    const char = CHAR.exec(text.slice(i));
    let end: number;
    if (text.startsWith("//", i)) {
      end = text.indexOf("\n", i);
      if (end < 0) end = text.length;
    } else if (text.startsWith("/*", i)) {
      end = i + 2;
      let depth = 1;
      while (end < text.length && depth) {
        if (text.startsWith("/*", end)) {
          depth++;
          end += 2;
        } else if (text.startsWith("*/", end)) {
          depth--;
          end += 2;
        } else end++;
      }
      if (depth) throw new Error("Unterminated Rust comment");
    } else if (raw) {
      const closing = `"${raw[1]}`;
      end = text.indexOf(closing, i + raw[0].length);
      if (end < 0) throw new Error("Unterminated raw string");
      end += closing.length;
    } else if (text[i] === '"') {
      end = i + 1;
      while (end < text.length && text[end] !== '"') end += text[end] === "\\" ? 2 : 1;
      if (end >= text.length) throw new Error("Unterminated string");
      end++;
    } else if (char) end = i + char[0].length;
    else {
      i++;
      continue;
    }
    for (let offset = i; offset < end; offset++) out[offset] = text[offset] === "\n" ? "\n" : " ";
    i = end;
  }
  return out.join("");
}

function importAttributes(text: string, view: string, start: number): [number, string] {
  let first = start;
  let cursor = start;
  for (;;) {
    cursor--;
    while (cursor >= 0 && /\s/u.test(view[cursor])) cursor--;
    if (cursor < 0 || view[cursor] !== "]") break;
    let depth = 1;
    let opening = cursor - 1;
    while (opening >= 0 && depth) {
      depth += Number(view[opening] === "]") - Number(view[opening] === "[");
      opening--;
    }
    while (opening >= 0 && /\s/u.test(view[opening])) opening--;
    if (depth || opening < 0 || view[opening] !== "#") break;
    first = opening;
    while (first > 0 && /[ \t]/u.test(text[first - 1])) first--;
    cursor = first;
  }
  return [first, text.slice(first, start)];
}

function* groupedImports(text: string) {
  const view = lexicalView(text);
  for (const match of view.matchAll(GROUP)) {
    const start = match.index + match[0].length;
    let depth = 1;
    let end = start;
    for (; end < view.length; end++) {
      depth += Number(view[end] === "{") - Number(view[end] === "}");
      if (depth !== 0) continue;
      const tail = /^\s*;/u.exec(view.slice(end + 1));
      if (!tail) throw new Error("Malformed grouped import");
      const [first, attributes] = importAttributes(text, view, match.index);
      yield {
        match,
        body: text.slice(start, end),
        end: end + 1 + tail[0].length,
        first,
        attributes,
      };
      break;
    }
    if (depth) throw new Error("Unterminated grouped import");
  }
}

function* importItems(body: string) {
  const view = lexicalView(body);
  let depth = 0;
  let start = 0;
  function item(end: number) {
    const value = body.slice(start, end).trim();
    return lexicalView(value).trim() ? value + (value.includes("//") ? "\n" : "") : null;
  }
  for (let index = 0; index < view.length; index++) {
    depth += Number(view[index] === "{") - Number(view[index] === "}");
    if (view[index] !== "," || depth) continue;
    const value = item(index);
    if (value) yield value;
    start = index + 1;
  }
  const value = item(body.length);
  if (value) yield value;
}

const isHost = (item: string) => HOST.test(lexicalView(item).trim());

export function rewriteGroupedImports(text: string): string {
  for (const { match, body, end, first, attributes } of [...groupedImports(text)].reverse()) {
    const items = [...importItems(body)];
    const host = items.filter(isHost);
    if (!host.length) continue;
    const storage = items.filter((item) => !isHost(item));
    const prefixStart = match.index + match[1].length;
    const prefix = match[1] + text.slice(prefixStart, prefixStart + match[2].length);
    const lines = storage.length ? [`${prefix} vize_l0::{${storage.join(", ")}};`] : [];
    lines.push(...host.map((item) => `${prefix} vize_carton::${item};`));
    text =
      text.slice(0, first) + lines.map((line) => attributes + line).join("\n") + text.slice(end);
  }
  return text;
}

/** Rewrite real qualified host paths and grouped uses, never literal data. */
export function rewriteHostImports(text: string): string {
  for (const match of [...lexicalView(text).matchAll(DIRECT)].reverse()) {
    text = text.slice(0, match.index) + "vize_carton" + text.slice(match.index + match[0].length);
  }
  return rewriteGroupedImports(text);
}

export function hasHostImport(text: string): boolean {
  if ([...lexicalView(text).matchAll(DIRECT)].length) return true;
  return [...groupedImports(text)].some(({ body }) => [...importItems(body)].some(isHost));
}

function integrate(root: string) {
  update(root, "crates/vize_carton/src/corsa_resolver/tests.rs", (s) =>
    s.replaceAll("error.to_string()", 'crate::cstr!("{error}")'),
  );
  update(root, "davinci/vize_l0/src/lib.rs", (s) =>
    s.replace(
      /#\[cfg\(not\(target_arch = "wasm32"\)\)\]\npub mod corsa_(?:api_mode|resolver);\n/gu,
      "",
    ),
  );
  update(root, "crates/vize_carton/src/lib.rs", (s) =>
    s.includes("pub mod corsa_resolver;")
      ? s
      : s.replace(
          "//! Legacy compatibility facade for the shared foundation.",
          "//! Legacy storage compatibility and host-runtime integration.",
        ) +
        '\n#[cfg(not(target_arch = "wasm32"))]\npub mod corsa_api_mode;\n' +
        '#[cfg(not(target_arch = "wasm32"))]\n' +
        "#[expect(\n    clippy::disallowed_types,\n" +
        '    reason = "legacy host discovery retains its environment and path collections"\n)]\n' +
        "pub mod corsa_resolver;\n",
  );
  update(root, "davinci/vize_l0/Cargo.toml", (s) =>
    s.replace(
      "[target.'cfg(not(target_arch = \"wasm32\"))'.dependencies]\nwhich = { workspace = true }\n\n",
      "",
    ),
  );
  update(root, "crates/vize_carton/Cargo.toml", (s) =>
    s.includes("which = { workspace = true }")
      ? s
      : s.replace(
          "[features]\n",
          "[target.'cfg(not(target_arch = \"wasm32\"))'.dependencies]\n" +
            "which = { workspace = true }\nserde_json.workspace = true\n\n[features]\n",
        ),
  );
  for (const name of CONSUMERS) {
    update(root, `crates/${name}/Cargo.toml`, (s) =>
      /^vize_carton\s*[.=]/mu.test(s)
        ? s
        : s.replace("[dependencies]\n", "[dependencies]\nvize_carton.workspace = true\n"),
    );
    for (const source of rustSources(path.join(root, "crates", name))) {
      const before = readFileSync(source, "utf8");
      let text = before;
      // Canon's private storage alias otherwise shadows the real host owner.
      for (const alias of [
        ...lexicalView(text).matchAll(/^extern crate vize_l0 as vize_carton;\n/gmu),
      ].reverse()) {
        text = text.slice(0, alias.index) + text.slice(alias.index + alias[0].length);
      }
      text = rewriteHostImports(text);
      if (name === "vize" && path.basename(source) === "check_cli.rs") {
        text = text.replace(
          "};\n\nuse vize_l0::{cstr, path::canonicalize_non_verbatim};\n\nuse vize_carton::corsa_resolver::platform_suffix;",
          "};\nuse vize_l0::{cstr, path::canonicalize_non_verbatim};\nuse vize_carton::corsa_resolver::platform_suffix;",
        );
      }
      if (text !== before) writeFileSync(source, text);
    }
  }
}

export function replay(phase: Phase, root = ROOT) {
  const locations = FILES.map((relative) => [
    path.join(root, "davinci/vize_l0/src", relative),
    path.join(root, "crates/vize_carton/src", relative),
  ]);
  for (const [old, destination] of locations) {
    if (existsSync(old) === existsSync(destination)) {
      throw new Error(`Missing source or move collision: ${path.relative(root, old)}`);
    }
    if (phase !== "moves" && existsSync(old)) throw new Error("Run the move-only phase first");
  }
  if (phase === "moves") {
    for (const [old, destination] of locations) {
      if (!existsSync(old)) continue;
      mkdirSync(path.dirname(destination), { recursive: true });
      execFileSync("git", ["mv", old, destination], { cwd: root });
    }
  } else if (phase === "integrate") integrate(root);
  else {
    for (const name of CONSUMERS) {
      for (const source of rustSources(path.join(root, "crates", name))) {
        if (hasHostImport(readFileSync(source, "utf8"))) {
          throw new Error(`Unmigrated host import: ${path.relative(root, source)}`);
        }
      }
    }
    console.log("Host runtime ownership is integrated");
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [phase, flag, root, ...extra] = process.argv.slice(2);
  if (
    !(phase === "moves" || phase === "integrate" || phase === "check") ||
    (flag !== undefined && (flag !== "--root" || !root)) ||
    extra.length
  ) {
    throw new Error("Usage: move-host-runtime.ts moves|integrate|check [--root PATH]");
  }
  replay(phase, root ? path.resolve(root) : ROOT);
}
