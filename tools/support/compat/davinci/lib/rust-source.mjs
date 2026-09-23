// Rust source pre-processing and `use`-declaration parsing.
//
// Everything downstream counts references over *stripped* text (comments,
// strings and char literals blanked out at stable offsets) and resolves
// symbols through the flattened use-trees produced here.

/**
 * Replace comments (line, nested block, doc), string literals (plain, raw,
 * byte), and char literals with spaces, preserving offsets and newlines.
 * Lifetimes (`'a`) are kept.
 */
export function stripRust(source) {
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

export function maskKeepNewlines(text) {
  return text.replace(/[^\n]/g, " ");
}

/**
 * Find `use …;` statements in stripped source.
 * Returns [{ start, end, isPub, body }] where body excludes `use` and `;`.
 */
export function findUseDecls(stripped) {
  const decls = [];
  const re =
    /(^|[^A-Za-z0-9_])((?:pub(?:\s*\(\s*(?:crate|super|self|in\s+[A-Za-z0-9_:]+)\s*\))?\s+)?use\s+[^;]+;)/g;
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
export function expandUseTree(body, prefix = []) {
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

export function splitTopLevel(text) {
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
