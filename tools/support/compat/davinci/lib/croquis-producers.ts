// Workspace-wide table of croquis *producers*: the names that hand a
// `Croquis` value to a caller. Field-access counting needs these so a
// receiver bound from a cross-file call (`drawer.finish()`, `ctx.analysis()`,
// `result.croquis`) still resolves to a `Croquis`.

import { readFileSync } from "node:fs";

import { walkRustFiles } from "./crates.mjs";
import { CROQUIS_CRATE_NAME } from "./paths.mjs";
import { expandUseTree, findUseDecls, stripRust } from "./rust-source.mjs";

/**
 * Scan the whole workspace (including vize_croquis) for croquis-value
 * producers, so receivers bound from cross-file calls resolve:
 * - names of `pub fn`s whose return type mentions `Croquis`
 *   (e.g. `Drawer::finish`, `LintContext::analysis`), and
 * - names of `pub` struct fields typed `Croquis` (e.g. `analysis.croquis`).
 * Name-level: owner types are not tracked (limitation stated in artifact).
 */
export function collectCroquisProducers(allCrates) {
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
          if (
            root === CROQUIS_CRATE_NAME ||
            (inCroquis && (root === "crate" || root === "super" || root === "self"))
          ) {
            tokens.push(entry.alias ?? "Croquis");
          }
        }
      }
      const tokenAlt = [...new Set(tokens)]
        .map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
        .join("|");
      const returnsCroquis = (retText) =>
        new RegExp(`(?<![A-Za-z0-9_])(?:${tokenAlt})(?![A-Za-z0-9_])`).test(retText);
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
