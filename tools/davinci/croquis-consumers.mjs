#!/usr/bin/env node
// Croquis consumption matrix generator (Davinci P0-7).
//
// Enumerates the public analysis products of crates/vize_croquis
// (the `pub` fields of the `Croquis` struct in src/croquis.rs, the
// tracker/product types those fields reference, the types re-exported by
// croquis.rs, and the crate-root `pub use` groups in src/lib.rs), then
// resolves consumers across every other workspace crate symbol-aware:
// per-file alias tables built from parsed Rust `use` declarations
// (braces, `as` aliases, `pub use` re-export chains) plus typed-receiver
// field-access counting. A naive text-grep lane runs as a cross-check and
// disagreements are reported in the artifact, never reconciled.
//
// Usage:
//   node tools/davinci/croquis-consumers.mjs --write   # regenerate artifact
//   node tools/davinci/croquis-consumers.mjs --check   # diff against committed
//
// Node builtins only. Output is deterministic (stable sort everywhere,
// no timestamps, no absolute paths).

import { readFileSync, readdirSync, writeFileSync, existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const CRATES_DIR = path.join(repoRoot, "crates");
const CROQUIS_CRATE_DIR = "vize_croquis";
const CROQUIS_RS = path.join(CRATES_DIR, CROQUIS_CRATE_DIR, "src", "croquis.rs");
const LIB_RS = path.join(CRATES_DIR, CROQUIS_CRATE_DIR, "src", "lib.rs");
const ARTIFACT_REL = "davinci-road/plan/croquis-consumption.md";
const ARTIFACT = path.join(repoRoot, ARTIFACT_REL);
const REGEN_COMMAND = "node tools/davinci/croquis-consumers.mjs --write";

// ---------------------------------------------------------------------------
// Rust source pre-processing
// ---------------------------------------------------------------------------

/**
 * Replace comments (line, nested block, doc), string literals (plain, raw,
 * byte), and char literals with spaces, preserving offsets and newlines.
 * Lifetimes (`'a`) are kept.
 */
function stripRust(source) {
  const out = source.split("");
  const n = source.length;
  let i = 0;
  const blank = (from, to) => {
    for (let k = from; k < to; k++) if (out[k] !== "\n") out[k] = " ";
  };
  while (i < n) {
    const c = source[i];
    const c2 = source[i + 1];
    if (c === "/" && c2 === "/") {
      let j = i;
      while (j < n && source[j] !== "\n") j++;
      blank(i, j);
      i = j;
    } else if (c === "/" && c2 === "*") {
      let depth = 1;
      let j = i + 2;
      while (j < n && depth > 0) {
        if (source[j] === "/" && source[j + 1] === "*") {
          depth++;
          j += 2;
        } else if (source[j] === "*" && source[j + 1] === "/") {
          depth--;
          j += 2;
        } else {
          j++;
        }
      }
      blank(i, j);
      i = j;
    } else if (c === "r" || ((c === "b" || c === "c") && (c2 === "r" || c2 === '"'))) {
      // Raw strings r"…", r#"…"#, br"…", and byte strings b"…".
      let j = i;
      if (c === "b" || c === "c") j++;
      if (source[j] === "r") {
        j++;
        let hashes = 0;
        while (source[j] === "#") {
          hashes++;
          j++;
        }
        if (source[j] !== '"') {
          i++;
          continue; // identifier starting with r/br, not a raw string
        }
        j++;
        const closer = '"' + "#".repeat(hashes);
        const end = source.indexOf(closer, j);
        j = end === -1 ? n : end + closer.length;
        blank(i, j);
        i = j;
      } else if (source[j] === '"') {
        j++;
        while (j < n && source[j] !== '"') j += source[j] === "\\" ? 2 : 1;
        j++;
        blank(i, j);
        i = j;
      } else {
        i++;
      }
    } else if (c === '"') {
      let j = i + 1;
      while (j < n && source[j] !== '"') j += source[j] === "\\" ? 2 : 1;
      j++;
      blank(i, j);
      i = j;
    } else if (c === "'") {
      // Char literal vs lifetime: 'x' or '\n' closes with a quote; 'a (no
      // closing quote within two chars) is a lifetime and is kept.
      if (c2 === "\\") {
        let j = i + 2;
        while (j < n && source[j] !== "'") j++;
        blank(i, j + 1);
        i = j + 1;
      } else if (source[i + 2] === "'") {
        blank(i, i + 3);
        i += 3;
      } else {
        i++;
      }
    } else if (/[A-Za-z0-9_]/.test(c)) {
      // Skip whole identifier so `br` / `r` prefixes inside idents don't trip.
      let j = i;
      while (j < n && /[A-Za-z0-9_]/.test(source[j])) j++;
      i = j;
    } else {
      i++;
    }
  }
  return out.join("");
}

// ---------------------------------------------------------------------------
// `use` declaration parsing
// ---------------------------------------------------------------------------

/**
 * Find `use …;` statements in stripped source.
 * Returns [{ start, end, isPub, body }] where body excludes `use` and `;`.
 */
function findUseDecls(stripped) {
  const decls = [];
  const re = /(^|[^A-Za-z0-9_])((?:pub(?:\s*\(\s*(?:crate|super|self|in\s+[A-Za-z0-9_:]+)\s*\))?\s+)?use\s+[^;]+;)/g;
  let m;
  while ((m = re.exec(stripped)) !== null) {
    const stmt = m[2];
    const body = /use\s+([^;]+);$/s.exec(stmt)[1].trim();
    decls.push({
      start: m.index + m[1].length,
      end: m.index + m[0].length,
      isPub: stmt.trimStart().startsWith("pub"),
      body,
    });
  }
  return decls;
}

/**
 * Expand a use-tree body into flat entries.
 * "a::b::{C, d::E as F, self, *}" ->
 *   [{segments:["a","b","C"], alias:null, glob:false},
 *    {segments:["a","b","d","E"], alias:"F", glob:false},
 *    {segments:["a","b"], alias:null, glob:false, self:true},
 *    {segments:["a","b"], alias:null, glob:true}]
 */
function expandUseTree(body, prefix = []) {
  const text = body.trim();
  if (text === "") return [];
  const braceIdx = text.indexOf("{");
  if (braceIdx === -1) {
    // plain path, may end with `as Alias` or `*`
    const asMatch = /^(.*?)\s+as\s+([A-Za-z_][A-Za-z0-9_]*)$/s.exec(text);
    const rawPath = (asMatch ? asMatch[1] : text).trim();
    const alias = asMatch ? asMatch[2] : null;
    const segments = rawPath
      .split("::")
      .map((s) => s.trim())
      .filter((s) => s.length > 0);
    if (segments[segments.length - 1] === "*") {
      return [{ segments: [...prefix, ...segments.slice(0, -1)], alias: null, glob: true }];
    }
    if (segments[segments.length - 1] === "self") {
      return [{ segments: [...prefix, ...segments.slice(0, -1)], alias, glob: false, self: true }];
    }
    return [{ segments: [...prefix, ...segments], alias, glob: false }];
  }
  // path prefix before the brace group
  const before = text.slice(0, braceIdx).trim().replace(/::$/, "");
  const beforeSegments = before === "" ? [] : before.split("::").map((s) => s.trim());
  // find matching close brace
  let depth = 0;
  let close = -1;
  for (let i = braceIdx; i < text.length; i++) {
    if (text[i] === "{") depth++;
    else if (text[i] === "}") {
      depth--;
      if (depth === 0) {
        close = i;
        break;
      }
    }
  }
  if (close === -1) return [];
  const inner = text.slice(braceIdx + 1, close);
  const parts = splitTopLevel(inner);
  const entries = [];
  for (const part of parts) {
    entries.push(...expandUseTree(part, [...prefix, ...beforeSegments]));
  }
  return entries;
}

function splitTopLevel(text) {
  const parts = [];
  let depth = 0;
  let current = "";
  for (const ch of text) {
    if (ch === "{") depth++;
    if (ch === "}") depth--;
    if (ch === "," && depth === 0) {
      if (current.trim()) parts.push(current);
      current = "";
    } else {
      current += ch;
    }
  }
  if (current.trim()) parts.push(current);
  return parts;
}

// ---------------------------------------------------------------------------
// Product enumeration from vize_croquis sources
// ---------------------------------------------------------------------------

function enumerateProducts() {
  const croquisSource = readFileSync(CROQUIS_RS, "utf8");
  const croquisStripped = stripRust(croquisSource);
  const libSource = readFileSync(LIB_RS, "utf8");
  const libStripped = stripRust(libSource);

  /** name -> { name, kind: "type"|"fn-or-const", module } */
  const typeProducts = new Map();
  /** fieldName -> { name, module, typeText } */
  const fieldProducts = new Map();
  const passthroughs = [];

  const addType = (name, module) => {
    if (!typeProducts.has(name)) {
      typeProducts.set(name, { name, module });
    }
  };

  // --- lib.rs: local modules + crate-root re-export groups ------------------
  const localModules = new Set();
  for (const m of libStripped.matchAll(/(?:^|\s)(?:pub\s+)?mod\s+([a-z_][a-z0-9_]*)\s*;/g)) {
    localModules.add(m[1]);
  }
  for (const decl of findUseDecls(libStripped)) {
    if (!decl.isPub) continue;
    for (const entry of expandUseTree(decl.body)) {
      if (entry.glob || entry.segments.length === 0) continue;
      const root = entry.segments[0];
      const item = entry.alias ?? entry.segments[entry.segments.length - 1];
      if (localModules.has(root)) {
        addType(item, entry.segments.slice(0, -1).join("::") || root);
      } else {
        passthroughs.push(entry.segments.join("::"));
      }
    }
  }

  // --- croquis.rs: use-map, pub-use re-exports, Croquis struct fields -------
  /** localName -> module path inside the crate ("race", "provide", ...) */
  const croquisUseMap = new Map();
  for (const decl of findUseDecls(croquisStripped)) {
    for (const entry of expandUseTree(decl.body)) {
      if (entry.glob || entry.segments.length === 0) continue;
      const root = entry.segments[0];
      const item = entry.alias ?? entry.segments[entry.segments.length - 1];
      if (decl.isPub) {
        // pub use bindings::{...}; — croquis.rs submodule product re-exports
        addType(item, `croquis::${entry.segments.slice(0, -1).join("::") || root}`);
      } else if (root === "crate") {
        const module = entry.segments.slice(1, -1).join("::");
        croquisUseMap.set(item, module === "" ? "(crate root)" : module);
      }
      // `use vize_carton::…` in croquis.rs: external, not a product source.
    }
  }

  // The Croquis struct itself is defined in croquis.rs.
  addType("Croquis", "croquis");

  // Struct fields.
  const structMatch = /pub\s+struct\s+Croquis\s*\{/.exec(croquisStripped);
  if (!structMatch) throw new Error("Croquis struct not found in croquis.rs");
  let depth = 0;
  let bodyStart = -1;
  let bodyEnd = -1;
  for (let i = structMatch.index; i < croquisStripped.length; i++) {
    if (croquisStripped[i] === "{") {
      if (depth === 0) bodyStart = i + 1;
      depth++;
    } else if (croquisStripped[i] === "}") {
      depth--;
      if (depth === 0) {
        bodyEnd = i;
        break;
      }
    }
  }
  const body = croquisStripped.slice(bodyStart, bodyEnd);
  // Fields: `pub name:` then type text up to a top-level comma.
  const fieldRe = /pub\s+([a-z_][a-z0-9_]*)\s*:/g;
  let fm;
  while ((fm = fieldRe.exec(body)) !== null) {
    const name = fm[1];
    let i = fieldRe.lastIndex;
    let d = 0;
    let typeText = "";
    while (i < body.length) {
      const ch = body[i];
      if (ch === "<" || ch === "(" || ch === "[") d++;
      else if (ch === ">" || ch === ")" || ch === "]") d--;
      else if (ch === "," && d === 0) break;
      typeText += ch;
      i++;
    }
    typeText = typeText.replace(/\s+/g, " ").trim();
    fieldProducts.set(name, { name, module: "croquis", typeText });
    // Referenced tracker/product types: capitalized idents in the field type
    // that resolve through croquis.rs's own use declarations or are already
    // known products (croquis.rs pub-use re-exports / lib.rs crate root).
    for (const ident of typeText.matchAll(/\b([A-Z][A-Za-z0-9_]*)\b/g)) {
      const typeName = ident[1];
      if (croquisUseMap.has(typeName)) {
        const module = croquisUseMap.get(typeName);
        if (module === "(crate root)") {
          if (!typeProducts.has(typeName)) addType(typeName, "(crate root)");
        } else {
          addType(typeName, module);
        }
      }
      // Idents like Vec/Option/CompactString/FxHashMap don't resolve to
      // crate modules and are skipped; croquis.rs pub-use types (e.g.
      // BindingMetadata, TemplateInfo) were already added above.
    }
  }

  passthroughs.sort();
  return { typeProducts, fieldProducts, passthroughs };
}

// ---------------------------------------------------------------------------
// Workspace crate discovery
// ---------------------------------------------------------------------------

function discoverCrates() {
  const crates = [];
  for (const dirent of readdirSync(CRATES_DIR, { withFileTypes: true }).sort((a, b) =>
    a.name < b.name ? -1 : a.name > b.name ? 1 : 0,
  )) {
    if (!dirent.isDirectory()) continue;
    const cargoToml = path.join(CRATES_DIR, dirent.name, "Cargo.toml");
    const srcDir = path.join(CRATES_DIR, dirent.name, "src");
    if (!existsSync(cargoToml) || !existsSync(srcDir)) continue;
    const nameMatch = /^\s*name\s*=\s*"([^"]+)"/m.exec(readFileSync(cargoToml, "utf8"));
    crates.push({
      dir: dirent.name,
      name: nameMatch ? nameMatch[1] : dirent.name,
      srcDir,
    });
  }
  return crates;
}

function walkRustFiles(dir) {
  const files = [];
  const visit = (d) => {
    for (const dirent of readdirSync(d, { withFileTypes: true }).sort((a, b) =>
      a.name < b.name ? -1 : a.name > b.name ? 1 : 0,
    )) {
      const full = path.join(d, dirent.name);
      if (dirent.isDirectory()) visit(full);
      else if (dirent.isFile() && dirent.name.endsWith(".rs")) files.push(full);
    }
  };
  visit(dir);
  return files;
}

// ---------------------------------------------------------------------------
// Consumer resolution
// ---------------------------------------------------------------------------

const CROQUIS_CRATE_NAME = "vize_croquis";

/**
 * Scan the whole workspace (including vize_croquis) for croquis-value
 * producers, so receivers bound from cross-file calls resolve:
 * - names of `pub fn`s whose return type mentions `Croquis`
 *   (e.g. `Drawer::finish`, `LintContext::analysis`), and
 * - names of `pub` struct fields typed `Croquis` (e.g. `analysis.croquis`).
 * Name-level: owner types are not tracked (limitation stated in artifact).
 */
function collectCroquisProducers(allCrates) {
  const fns = new Set();
  const fields = new Set();
  for (const crate of allCrates) {
    const inCroquis = crate.name === CROQUIS_CRATE_NAME;
    for (const abs of walkRustFiles(crate.srcDir)) {
      const stripped = stripRust(readFileSync(abs, "utf8"));
      // Croquis type tokens valid in this file.
      const tokens = [];
      if (inCroquis) tokens.push("Croquis", "crate::croquis::Croquis", "croquis::Croquis");
      else tokens.push("vize_croquis::Croquis", "vize_croquis::croquis::Croquis");
      for (const decl of findUseDecls(stripped)) {
        for (const entry of expandUseTree(decl.body)) {
          if (entry.glob || entry.segments.length === 0) continue;
          const last = entry.segments[entry.segments.length - 1];
          if (last !== "Croquis") continue;
          const root = entry.segments[0];
          if (root === CROQUIS_CRATE_NAME || (inCroquis && (root === "crate" || root === "super" || root === "self"))) {
            tokens.push(entry.alias ?? "Croquis");
          }
        }
      }
      const tokenAlt = [...new Set(tokens)]
        .map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
        .join("|");
      const returnsCroquis = (retText) => new RegExp(`(?<![A-Za-z0-9_])(?:${tokenAlt})(?![A-Za-z0-9_])`).test(retText);
      // pub fns returning Croquis (multi-line signatures included).
      const fnRe = /pub(?:\s*\([^)]*\))?\s+(?:const\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)/g;
      let fm;
      while ((fm = fnRe.exec(stripped)) !== null) {
        // Scan the signature: from the fn name to the body `{` or `;`.
        let i = fnRe.lastIndex;
        let depth = 0;
        let sig = "";
        while (i < stripped.length && sig.length < 600) {
          const ch = stripped[i];
          if (ch === "(" || ch === "<" || ch === "[") depth++;
          else if (ch === ")" || ch === ">" || ch === "]") depth--;
          else if ((ch === "{" || ch === ";") && depth <= 0) break;
          sig += ch;
          i++;
        }
        const arrow = sig.indexOf("->");
        if (arrow !== -1 && returnsCroquis(sig.slice(arrow))) fns.add(fm[1]);
      }
      // pub struct fields typed Croquis.
      const fieldRe = new RegExp(
        `pub(?:\\s*\\([^)]*\\))?\\s+([a-z_][a-z0-9_]*)\\s*:\\s*(?:&(?:'[a-z_][A-Za-z0-9_]*)?\\s*(?:mut\\s+)?)?` +
          `(?:(?:Option|Box|Rc|Arc)\\s*<\\s*&?(?:'[a-z_][A-Za-z0-9_]*)?\\s*)*(?:${tokenAlt})(?![A-Za-z0-9_])`,
        "g",
      );
      let dm;
      while ((dm = fieldRe.exec(stripped)) !== null) fields.add(dm[1]);
    }
  }
  return { fns, fields };
}

function analyzeConsumers(products) {
  const allCrates = discoverCrates();
  const crates = allCrates.filter((c) => c.name !== CROQUIS_CRATE_NAME);
  const crateNames = new Set(crates.map((c) => c.name));
  const producers = collectCroquisProducers(allCrates);

  // Parse every file once.
  /** crate -> [{ rel, raw, stripped, masked, decls: [entry…] }] */
  const parsed = new Map();
  for (const crate of crates) {
    const files = [];
    for (const abs of walkRustFiles(crate.srcDir)) {
      const raw = readFileSync(abs, "utf8");
      const stripped = stripRust(raw);
      const decls = findUseDecls(stripped);
      // Mask use decls out of the reference-counting text.
      let masked = stripped;
      for (const d of decls) {
        masked = masked.slice(0, d.start) + maskKeepNewlines(stripped.slice(d.start, d.end)) + masked.slice(d.end);
      }
      const entries = [];
      for (const d of decls) {
        for (const entry of expandUseTree(d.body)) {
          entries.push({ ...entry, isPub: d.isPub });
        }
      }
      files.push({
        rel: path.relative(repoRoot, abs).split(path.sep).join("/"),
        raw,
        masked,
        entries,
      });
    }
    parsed.set(crate.name, files);
  }

  // Fixpoint resolution of use entries to vize_croquis items across
  // `pub use` re-export chains (name-level approximation).
  /** crate -> Map(exportedName -> itemName) */
  const crateExports = new Map(crates.map((c) => [c.name, new Map()]));
  /** per file: resolved item aliases / module aliases / counted pub-use sites */
  for (const crate of crates) {
    for (const file of parsed.get(crate.name)) {
      file.itemAliases = new Map(); // localName -> { item, via }
      file.moduleAliases = new Map(); // localName -> module path under vize_croquis
      file.pubUseSites = []; // itemName[]
      file.globImports = [];
    }
  }
  let changed = true;
  let rounds = 0;
  while (changed && rounds < 12) {
    changed = false;
    rounds++;
    for (const crate of crates) {
      for (const file of parsed.get(crate.name)) {
        for (const entry of file.entries) {
          if (entry.resolvedDone) continue;
          const root = entry.segments[0];
          let itemName = null;
          let via = null;
          let modulePath = null;
          if (root === CROQUIS_CRATE_NAME) {
            if (entry.glob) {
              file.globImports.push(entry.segments.join("::") + "::*");
              entry.resolvedDone = true;
              changed = true;
              continue;
            }
            const last = entry.segments[entry.segments.length - 1];
            if (entry.self) {
              // `use vize_croquis::foo::{self}` binds module `foo`.
              modulePath = entry.segments.slice(1).join("::");
            } else if (entry.segments.length === 1) {
              // `use vize_croquis;` / `use vize_croquis as vc;`
              modulePath = "";
            } else {
              itemName = last;
              via = entry.segments.join("::");
              // A lowercase final segment may be a module (see moduleAliases
              // handling below) or a fn/const item; both are tracked.
              modulePath = entry.segments.slice(1).join("::");
            }
          } else if (crateNames.has(root) || root === "crate") {
            const exports = root === "crate" ? crateExports.get(crate.name) : crateExports.get(root);
            const last = entry.alias ?? entry.segments[entry.segments.length - 1];
            const target = exports?.get(entry.segments[entry.segments.length - 1]);
            if (target) {
              itemName = target;
              via = entry.segments.join("::") + " -> " + target;
            } else {
              continue; // may resolve in a later round
            }
          } else {
            entry.resolvedDone = true;
            changed = true;
            continue;
          }
          const local = entry.alias ?? entry.segments[entry.segments.length - 1];
          if (itemName !== null) {
            file.itemAliases.set(local, { item: itemName, via });
            if (/^[a-z_]/.test(itemName) && modulePath !== null) {
              // Could also be a module path; allow `local::Member` refs.
              file.moduleAliases.set(local, modulePath);
            }
            if (entry.isPub) {
              crateExports.get(crate.name).set(local, itemName);
              file.pubUseSites.push(itemName);
            }
          } else if (modulePath !== null) {
            file.moduleAliases.set(local, modulePath);
          }
          entry.resolvedDone = true;
          changed = true;
        }
      }
    }
  }

  // Reference + field-access counting.
  const rows = new Map(); // key `${product} ${crate}` -> { sites, files:Set }
  const nonProduct = new Map(); // same keying for non-product croquis items
  const grepRows = new Map(); // naive lane
  const globFiles = [];

  const bump = (map, product, crate, file, count) => {
    if (count <= 0) return;
    const key = product + " " + crate;
    if (!map.has(key)) map.set(key, { product, crate, sites: 0, files: new Set() });
    const row = map.get(key);
    row.sites += count;
    row.files.add(file);
  };

  const isProduct = (name) => products.typeProducts.has(name);
  const fieldNames = [...products.fieldProducts.keys()].sort();
  const fieldAlt = fieldNames.join("|");

  for (const crate of crates) {
    for (const file of parsed.get(crate.name)) {
      const attribute = (name, count) => {
        if (isProduct(name)) bump(rows, name, crate.name, file.rel, count);
        else bump(nonProduct, name, crate.name, file.rel, count);
      };

      for (const g of file.globImports) globFiles.push(`${file.rel}: use ${g}`);
      for (const item of file.pubUseSites) attribute(item, 1);

      // (1) Fully qualified paths in code (masked text has use decls removed).
      let text = file.masked;
      const fqRe = /\bvize_croquis(?:::[a-z_][a-z0-9_]*)*::([A-Za-z_][A-Za-z0-9_]*)/g;
      const fqCounts = new Map();
      text = text.replace(fqRe, (whole, item) => {
        fqCounts.set(item, (fqCounts.get(item) ?? 0) + 1);
        return " ".repeat(whole.length);
      });
      for (const [item, count] of fqCounts) attribute(item, count);

      // (2) Module-alias qualified references: `alias::Member`.
      const memberHits = new Map();
      for (const [local] of file.moduleAliases) {
        const re = new RegExp(`(?<![A-Za-z0-9_:.])${local}::([A-Za-z_][A-Za-z0-9_]*)`, "g");
        text = text.replace(re, (whole, member) => {
          memberHits.set(member, (memberHits.get(member) ?? 0) + 1);
          return " ".repeat(whole.length);
        });
      }
      for (const [member, count] of memberHits) attribute(member, count);

      // (3) Item-alias bare references.
      for (const [local, { item }] of file.itemAliases) {
        const re = new RegExp(`(?<![A-Za-z0-9_:.'])${local}(?![A-Za-z0-9_])`, "g");
        const count = (text.match(re) ?? []).length;
        attribute(item, count);
      }

      // (4) Field accesses on values provably typed `Croquis` in this file.
      if (fieldAlt) {
        const croquisTypeNames = [];
        for (const [local, { item }] of file.itemAliases) {
          if (item === "Croquis") croquisTypeNames.push(local);
        }
        croquisTypeNames.push("vize_croquis::Croquis", "vize_croquis::croquis::Croquis");
        const receivers = new Set();
        const croquisMethods = new Set();
        for (const typeName of croquisTypeNames) {
          const t = typeName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
          // parameter / struct field / let annotations, incl. &, &'a, &mut,
          // Option<&…>, Box/Rc/Arc wrappers
          const annRe = new RegExp(
            `\\b([a-z_][a-z0-9_]*)\\s*:\\s*(?:&(?:'[a-z_][A-Za-z0-9_]*)?\\s*(?:mut\\s+)?)?` +
              `(?:(?:Option|Box|Rc|Arc)\\s*<\\s*&?(?:'[a-z_][A-Za-z0-9_]*)?\\s*)*${t}\\b`,
            "g",
          );
          for (const m of file.masked.matchAll(annRe)) receivers.add(m[1]);
          // methods/functions returning Croquis (same file)
          const fnRe = new RegExp(
            `\\bfn\\s+([a-z_][a-z0-9_]*)\\s*(?:<[^>]*>)?\\s*\\([^)]*\\)\\s*->\\s*[^;{]*\\b${t}\\b`,
            "g",
          );
          for (const m of file.masked.matchAll(fnRe)) croquisMethods.add(m[1]);
        }
        // Receivers bound from workspace-wide croquis producers:
        //   let croquis = drawer.finish();      (pub fn … -> Croquis)
        //   let Some(a) = ctx.analysis() else…  (pub fn … -> Option<&Croquis>)
        //   let summary = result.croquis;       (pub field …: Croquis)
        if (producers.fns.size > 0 || producers.fields.size > 0) {
          const fnAlt = [...producers.fns].sort().join("|");
          const fieldAltP = [...producers.fields].sort().join("|");
          const letRe = new RegExp(
            `\\blet\\s+(?:mut\\s+)?(?:Some\\s*\\(\\s*)?(?:mut\\s+)?(?:&\\s*)?([a-z_][a-z0-9_]*)\\s*\\)?\\s*=\\s*([^;]{0,200})`,
            "g",
          );
          const ctorAlt = croquisTypeNames
            .map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
            .join("|");
          for (const m of file.masked.matchAll(letRe)) {
            const rhs = m[2];
            let bound = false;
            if (fnAlt && new RegExp(`[.:](?:${fnAlt})\\s*\\(`).test(rhs)) bound = true;
            if (!bound && fieldAltP && new RegExp(`\\.(?:${fieldAltP})\\b(?!\\s*\\()`).test(rhs)) bound = true;
            // Constructor / associated calls: `let c = Croquis::default();`
            if (!bound && new RegExp(`(?<![A-Za-z0-9_])(?:${ctorAlt})\\s*::\\s*[a-z_][a-z0-9_]*\\s*\\(`).test(rhs)) {
              bound = true;
            }
            if (bound) receivers.add(m[1]);
          }
        }
        receivers.delete("self");
        const fieldHits = new Map();
        const seenPositions = new Set();
        const collect = (re) => {
          for (const m of file.masked.matchAll(re)) {
            const field = m[1];
            const pos = m.index + m[0].length;
            if (seenPositions.has(pos)) continue;
            seenPositions.add(pos);
            fieldHits.set(field, (fieldHits.get(field) ?? 0) + 1);
          }
        };
        if (receivers.size > 0) {
          const rAlt = [...receivers].sort().join("|");
          collect(
            new RegExp(
              `(?<![A-Za-z0-9_])(?:${rAlt})\\s*(?:\\.\\s*[a-z_][a-z0-9_]*\\(\\)\\s*)*\\.\\s*(${fieldAlt})(?![A-Za-z0-9_(])`,
              "g",
            ),
          );
        }
        const chainMethods = new Set([...croquisMethods, ...producers.fns]);
        if (chainMethods.size > 0) {
          const mAlt = [...chainMethods].sort().join("|");
          collect(
            new RegExp(`\\.\\s*(?:${mAlt})\\(\\)\\s*\\.\\s*(${fieldAlt})(?![A-Za-z0-9_(])`, "g"),
          );
        }
        if (producers.fields.size > 0) {
          // Inline chains through a croquis-typed field: `entry.analysis.race_conditions`.
          const pfAlt = [...producers.fields].sort().join("|");
          collect(
            new RegExp(`\\.\\s*(?:${pfAlt})\\s*\\.\\s*(${fieldAlt})(?![A-Za-z0-9_(])`, "g"),
          );
        }
        for (const [field, count] of fieldHits) {
          bump(rows, "Croquis." + field, crate.name, file.rel, count);
        }
      }

      // (5) Naive grep lane over the RAW text (comments/strings included).
      for (const name of products.typeProducts.keys()) {
        const re = new RegExp(`(?<![A-Za-z0-9_])${name}(?![A-Za-z0-9_])`, "g");
        bump(grepRows, name, crate.name, file.rel, (file.raw.match(re) ?? []).length);
      }
      for (const field of fieldNames) {
        const re = new RegExp(`\\.${field}(?![A-Za-z0-9_])`, "g");
        bump(grepRows, "Croquis." + field, crate.name, file.rel, (file.raw.match(re) ?? []).length);
      }
    }
  }

  globFiles.sort();
  return { crates, rows, nonProduct, grepRows, globFiles };
}

function maskKeepNewlines(text) {
  return text.replace(/[^\n]/g, " ");
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

function byKey(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

function renderArtifact(products, analysis) {
  const { typeProducts, fieldProducts, passthroughs } = products;
  const { rows, nonProduct, grepRows, globFiles } = analysis;

  const productIds = [];
  for (const name of [...typeProducts.keys()].sort(byKey)) {
    productIds.push({ id: name, kind: "type", module: typeProducts.get(name).module });
  }
  for (const name of [...fieldProducts.keys()].sort(byKey)) {
    productIds.push({
      id: "Croquis." + name,
      kind: "field",
      module: "croquis",
      typeText: fieldProducts.get(name).typeText,
    });
  }

  const rowsByProduct = new Map();
  for (const row of rows.values()) {
    if (!rowsByProduct.has(row.product)) rowsByProduct.set(row.product, []);
    rowsByProduct.get(row.product).push(row);
  }
  for (const list of rowsByProduct.values()) list.sort((a, b) => byKey(a.crate, b.crate));

  const consumed = productIds.filter((p) => rowsByProduct.has(p.id));
  const unconsumed = productIds.filter((p) => !rowsByProduct.has(p.id));

  const lines = [];
  lines.push("<!-- GENERATED FILE — do not edit by hand.");
  lines.push(`     Regenerate: ${REGEN_COMMAND}`);
  lines.push("     Verify:     node tools/davinci/croquis-consumers.mjs --check");
  lines.push("     Generator:  tools/davinci/croquis-consumers.mjs -->");
  lines.push("");
  lines.push("# Croquis consumption matrix");
  lines.push("");
  lines.push(
    "Which workspace crates consume the public analysis products of" +
      " `crates/vize_croquis`. Mechanizes the 2026-08-13 hand audit in" +
      " [semantic-engine.md](../semantic-engine.md#the-problem-measured) (Davinci P0-7).",
  );
  lines.push("");
  lines.push("## Resolution method (and its limits)");
  lines.push("");
  lines.push("**Product enumeration** — parsed from source, not hardcoded:");
  lines.push("");
  lines.push(
    "- `pub` fields of the `Croquis` struct in `crates/vize_croquis/src/croquis.rs`" +
      " (rows named `Croquis.<field>`), plus the tracker/product types those fields" +
      " reference, resolved through croquis.rs's own `use crate::…` declarations.",
  );
  lines.push(
    "- Types re-exported by croquis.rs (`pub use bindings::…`, `snapshot::…`, …) and" +
      " the crate-root `pub use` groups in `crates/vize_croquis/src/lib.rs` whose" +
      " source is a local module (this is what brings in the `effect_graph`," +
      " `scope`, `symbol`, `analyzer`, `drawer`, and `reactivity_overlay` families).",
  );
  lines.push(
    "- Crate-root passthrough re-exports of foreign items are **excluded** from the" +
      " product set: " +
      passthroughs.map((p) => "`" + p + "`").join(", ") +
      ".",
  );
  lines.push("");
  lines.push("**Consumer resolution** — symbol-aware, per `crates/*/src/**/*.rs`:");
  lines.push("");
  lines.push(
    "- Rust `use` declarations are parsed (brace groups, `as` aliases, `pub use`)" +
      " into per-file alias tables mapping local names to `vize_croquis` items;" +
      " `pub use` re-export chains across crates are followed to a fixpoint" +
      " (name-level, see limits). Comments and string literals are stripped before" +
      " any counting, and the `use` declarations themselves are not counted as" +
      " reference sites (a `pub use` re-export counts as one site).",
  );
  lines.push(
    "- **type rows** — sites are references to a resolved local alias, a module-" +
      "qualified member (`reactivity::ReactiveKind`), or a fully qualified" +
      " `vize_croquis::…` path.",
  );
  lines.push(
    "- **`Croquis.<field>` rows** — sites are field accesses (`summary.bindings`)" +
      " counted only on receivers resolved to `Croquis` values: idents with a" +
      " `Croquis` type annotation (params, struct fields, `let`," +
      " `&`/`&'a`/`&mut`/`Option<&…>`/`Box`/`Rc`/`Arc` wrappers), calls to" +
      " same-file functions returning `Croquis`, and `let`-bindings whose" +
      " right-hand side calls a workspace `pub fn` returning `Croquis`" +
      " (`drawer.finish()`, `ctx.analysis()`), reads a workspace `pub` field" +
      " typed `Croquis` (`result.croquis`), or calls an associated function on" +
      " the `Croquis` type itself (`Croquis::default()`) — producer tables" +
      " parsed from `crates/*/src`, matched by name. Inline chains through" +
      " those producers" +
      " (`entry.analysis.race_conditions`, `ctx.croquis().bindings`) are counted" +
      " too.",
  );
  lines.push("");
  lines.push("**Known limits** (undercounts are possible; the grep lane below bounds them):");
  lines.push("");
  lines.push(
    "- Re-export chains resolve by item **name**, not full module path; same-named" +
      " items reached through different facade modules would be conflated.",
  );
  lines.push(
    "- No type inference: field accesses through closure params, iterator" +
      " chains, destructuring patterns, or re-borrowed locals (`let b = &a;`)" +
      " are not counted; the producer tables match croquis-returning method" +
      " **names** without owner types, so a same-named method on an unrelated" +
      " type can mark a false receiver (only matters if that value also has a" +
      " product-named field).",
  );
  lines.push("- Macro-generated code is invisible to source parsing.");
  lines.push(
    "- `#[cfg(test)]` code inside `src/` is included; `tests/`, `benches/`," +
      " `examples/` directories are not scanned. `vize_croquis` itself is excluded" +
      " (internal use is not consumption). Note that `vize_croquis_cf` is a" +
      " separate crate and therefore counted as an external consumer, even though" +
      " it is part of the same semantic layer.",
  );
  if (globFiles.length > 0) {
    lines.push("- Glob imports of `vize_croquis` (cannot be alias-resolved):");
    for (const g of globFiles) lines.push(`  - ${g}`);
  } else {
    lines.push("- No glob imports (`use vize_croquis::…::*`) exist in the workspace today.");
  }
  lines.push("");

  lines.push("## Products with external consumers");
  lines.push("");
  lines.push("| product | kind | module | consuming crate | files | sites |");
  lines.push("| --- | --- | --- | --- | ---: | ---: |");
  for (const p of consumed) {
    for (const row of rowsByProduct.get(p.id)) {
      lines.push(
        `| \`${p.id}\` | ${p.kind} | \`${p.module}\` | \`${row.crate}\` | ${row.files.size} | ${row.sites} |`,
      );
    }
  }
  lines.push("");

  lines.push("## Products with no external consumers");
  lines.push("");
  lines.push(
    "Computed (or exported) by `vize_croquis`, referenced by no other workspace" +
      " crate under the resolution above.",
  );
  lines.push("");
  const unconsumedByModule = new Map();
  for (const p of unconsumed) {
    if (!unconsumedByModule.has(p.module)) unconsumedByModule.set(p.module, []);
    unconsumedByModule.get(p.module).push(p);
  }
  for (const module of [...unconsumedByModule.keys()].sort(byKey)) {
    const items = unconsumedByModule
      .get(module)
      .map((p) => "`" + p.id + "`")
      .join(", ");
    lines.push(`- \`${module}\`: ${items}`);
  }
  lines.push("");

  lines.push("## Non-product `vize_croquis` imports observed");
  lines.push("");
  lines.push(
    "Items consumers import from `vize_croquis` that are outside the product set" +
      " above (module-path items never re-exported at the crate root nor referenced" +
      " by `croquis.rs`). Kept visible so nothing resolved is silently dropped.",
  );
  lines.push("");
  lines.push("| item | consuming crate | files | sites |");
  lines.push("| --- | --- | ---: | ---: |");
  const nonProductRows = [...nonProduct.values()].sort(
    (a, b) => byKey(a.product, b.product) || byKey(a.crate, b.crate),
  );
  for (const row of nonProductRows) {
    lines.push(`| \`${row.product}\` | \`${row.crate}\` | ${row.files.size} | ${row.sites} |`);
  }
  lines.push("");

  lines.push("## Cross-check: symbol-resolved vs naive grep");
  lines.push("");
  lines.push(
    "The naive lane counts raw word-boundary text matches per product name" +
      " (`\\.field` matches for field rows) over the same files — comments," +
      " strings, doc text, and same-named unrelated symbols included, imports" +
      " included. Disagreements are listed, **not** reconciled: `grep > resolved`" +
      " usually means comments/unrelated same-named symbols (for field rows: field" +
      " accesses on non-`Croquis` receivers); `grep < resolved` would indicate a" +
      " resolver bug and must be investigated.",
  );
  lines.push("");
  lines.push("| product | resolved | grep | disagreeing crates (resolved/grep) |");
  lines.push("| --- | ---: | ---: | --- |");
  const perProduct = new Map();
  const addTo = (map, key, crate, sites) => {
    if (!map.has(key)) map.set(key, new Map());
    map.get(key).set(crate, sites);
  };
  const resolvedPer = new Map();
  const grepPer = new Map();
  for (const row of rows.values()) addTo(resolvedPer, row.product, row.crate, row.sites);
  for (const row of grepRows.values()) addTo(grepPer, row.product, row.crate, row.sites);
  const allProducts = productIds.map((p) => p.id);
  for (const id of allProducts) {
    const r = resolvedPer.get(id) ?? new Map();
    const g = grepPer.get(id) ?? new Map();
    const crates = [...new Set([...r.keys(), ...g.keys()])].sort(byKey);
    const diffs = crates.filter((c) => (r.get(c) ?? 0) !== (g.get(c) ?? 0));
    if (diffs.length === 0) continue;
    const rTotal = [...r.values()].reduce((a, b) => a + b, 0);
    const gTotal = [...g.values()].reduce((a, b) => a + b, 0);
    const detail = diffs.map((c) => `\`${c}\` (${r.get(c) ?? 0}/${g.get(c) ?? 0})`).join(", ");
    lines.push(`| \`${id}\` | ${rTotal} | ${gTotal} | ${detail} |`);
  }
  lines.push("");
  return lines.join("\n");
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

function generate() {
  const products = enumerateProducts();
  const analysis = analyzeConsumers(products);
  return renderArtifact(products, analysis);
}

function main() {
  const mode = process.argv[2];
  if (mode !== "--write" && mode !== "--check") {
    console.error("usage: node tools/davinci/croquis-consumers.mjs --write | --check");
    process.exit(2);
  }
  const generated = generate();
  if (mode === "--write") {
    writeFileSync(ARTIFACT, generated);
    console.log(`wrote ${ARTIFACT_REL}`);
    return;
  }
  // --check
  if (!existsSync(ARTIFACT)) {
    console.error(`stale: ${ARTIFACT_REL} does not exist. Regenerate with: ${REGEN_COMMAND}`);
    process.exit(1);
  }
  const committed = readFileSync(ARTIFACT, "utf8");
  if (committed === generated) {
    console.log(`${ARTIFACT_REL} is up to date`);
    return;
  }
  const committedLines = committed.split("\n");
  const generatedLines = generated.split("\n");
  let firstDiff = -1;
  const max = Math.max(committedLines.length, generatedLines.length);
  for (let i = 0; i < max; i++) {
    if (committedLines[i] !== generatedLines[i]) {
      firstDiff = i;
      break;
    }
  }
  const committedSet = new Set(committedLines);
  const generatedSet = new Set(generatedLines);
  const removed = committedLines.filter((l) => !generatedSet.has(l)).length;
  const added = generatedLines.filter((l) => !committedSet.has(l)).length;
  console.error(`stale: ${ARTIFACT_REL} drifted from the current sources.`);
  console.error(
    `  first differing line: ${firstDiff + 1} (committed ${committedLines.length} lines, regenerated ${generatedLines.length})`,
  );
  if (firstDiff >= 0) {
    console.error(`  - ${(committedLines[firstDiff] ?? "<eof>").slice(0, 160)}`);
    console.error(`  + ${(generatedLines[firstDiff] ?? "<eof>").slice(0, 160)}`);
  }
  console.error(`  lines only in committed: ${removed}, only in regenerated: ${added}`);
  console.error(`  Regenerate with: ${REGEN_COMMAND}`);
  process.exit(1);
}

main();
