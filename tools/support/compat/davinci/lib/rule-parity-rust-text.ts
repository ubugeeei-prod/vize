// Extra Rust source pre-processing for the rule-parity generator: a stripping
// pass that keeps string literals (rule names and `"v-…"` literals are signals
// here, not noise), literal collection, and the impl/hook shape of a rule file.
//
// Complements ./rust-source.mjs, whose stripRust blanks strings as well.

import { RULE_TRAITS } from "./rule-parity-paths.mjs";

/**
 * Like stripRust, but keeps string/char literals: only comments are blanked.
 * Used where string contents are the signal (rule names, "v-…" literals).
 */
export function stripRustComments(source) {
  const out = source.split("");
  const n = source.length;
  let i = 0;
  const blank = (from, to) => {
    for (let k = from; k < to; k++) if (out[k] !== "\n") out[k] = " ";
  };
  const skipString = (from) => {
    // Assumes source[from] is the opening quote of a plain/byte string.
    let j = from + 1;
    while (j < n && source[j] !== '"') j += source[j] === "\\" ? 2 : 1;
    return j + 1;
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
          continue;
        }
        j++;
        const closer = '"' + "#".repeat(hashes);
        const end = source.indexOf(closer, j);
        i = end === -1 ? n : end + closer.length;
      } else if (source[j] === '"') {
        i = skipString(j);
      } else {
        i++;
      }
    } else if (c === '"') {
      i = skipString(i);
    } else if (c === "'") {
      if (c2 === "\\") {
        let j = i + 2;
        while (j < n && source[j] !== "'") j++;
        i = j + 1;
      } else if (source[i + 2] === "'") {
        i += 3;
      } else {
        i++;
      }
    } else if (/[A-Za-z0-9_]/.test(c)) {
      let j = i;
      while (j < n && /[A-Za-z0-9_]/.test(source[j])) j++;
      i = j;
    } else {
      i++;
    }
  }
  return out.join("");
}

/** Collect the contents of "…" / r#"…"# / b"…" string literals in code text. */
export function collectStringLiterals(code) {
  const literals = [];
  const n = code.length;
  let i = 0;
  while (i < n) {
    const c = code[i];
    const c2 = code[i + 1];
    if (c === "r" || ((c === "b" || c === "c") && (c2 === "r" || c2 === '"'))) {
      let j = i;
      if (c === "b" || c === "c") j++;
      if (code[j] === "r") {
        j++;
        let hashes = 0;
        while (code[j] === "#") {
          hashes++;
          j++;
        }
        if (code[j] !== '"') {
          i++;
          continue;
        }
        j++;
        const closer = '"' + "#".repeat(hashes);
        const end = code.indexOf(closer, j);
        literals.push(code.slice(j, end === -1 ? n : end));
        i = end === -1 ? n : end + closer.length;
      } else if (code[j] === '"') {
        let k = j + 1;
        while (k < n && code[k] !== '"') k += code[k] === "\\" ? 2 : 1;
        literals.push(code.slice(j + 1, k));
        i = k + 1;
      } else {
        i++;
      }
    } else if (c === '"') {
      let k = i + 1;
      while (k < n && code[k] !== '"') k += code[k] === "\\" ? 2 : 1;
      literals.push(code.slice(i + 1, k));
      i = k + 1;
    } else if (/[A-Za-z0-9_]/.test(c)) {
      let j = i;
      while (j < n && /[A-Za-z0-9_]/.test(code[j])) j++;
      i = j;
    } else {
      i++;
    }
  }
  return literals;
}

export function matchBraceBlock(text, openIdx) {
  let depth = 0;
  for (let i = openIdx; i < text.length; i++) {
    if (text[i] === "{") depth++;
    else if (text[i] === "}") {
      depth--;
      if (depth === 0) return i;
    }
  }
  return -1;
}

/**
 * Find `impl <Trait> for <Type> { … }` blocks for the rule traits and list the
 * fns defined at the impl's top level, with each fn's body text.
 */
export function findImplBlocks(stripped) {
  const blocks = [];
  const re = new RegExp(
    `\\bimpl\\s+(${RULE_TRAITS.join("|")})\\s+for\\s+([A-Za-z_][A-Za-z0-9_]*)`,
    "g",
  );
  let m;
  while ((m = re.exec(stripped)) !== null) {
    const open = stripped.indexOf("{", m.index + m[0].length);
    if (open === -1) continue;
    const close = matchBraceBlock(stripped, open);
    if (close === -1) continue;
    const body = stripped.slice(open + 1, close);
    const fns = new Map(); // fnName -> body text
    const fnRe = /\bfn\s+([a-z_][a-z0-9_]*)/g;
    let fm;
    while ((fm = fnRe.exec(body)) !== null) {
      // Only fns at the impl's own level: no unclosed brace before this fn.
      const before = body.slice(0, fm.index);
      let depth = 0;
      for (const ch of before) {
        if (ch === "{") depth++;
        else if (ch === "}") depth--;
      }
      if (depth !== 0) continue;
      const fnOpen = body.indexOf("{", fnRe.lastIndex);
      if (fnOpen === -1) continue;
      const fnClose = matchBraceBlock(body, fnOpen);
      if (fnClose === -1) continue;
      fns.set(fm[1], body.slice(fnOpen + 1, fnClose));
    }
    blocks.push({ trait: m[1], type: m[2], fns });
    re.lastIndex = close;
  }
  return blocks;
}
