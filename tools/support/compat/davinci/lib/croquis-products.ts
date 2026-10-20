// Product enumeration from the vize_croquis sources: the `pub` fields of the
// `Croquis` struct, the tracker/product types those fields reference, the
// types croquis.rs re-exports, and the crate-root `pub use` groups.

import { readFileSync } from "node:fs";

import { byKey } from "./ordering.mjs";
import { CROQUIS_RS, LIB_RS } from "./paths.mjs";
import { expandUseTree, findUseDecls, stripRust } from "./rust-source.mjs";

export function enumerateProducts() {
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

  passthroughs.sort(byKey);
  return { typeProducts, fieldProducts, passthroughs };
}
