// Minimal TOML-subset parser shared by the Davinci tooling
// (tools/davinci/assertion-lint.mjs, tools/davinci/matrix-gen.mjs,
// tools/davinci/bench-compare.mjs).
//
// Node builtins only is a hard constraint for these tools, so this file
// implements exactly the subset the committed Davinci TOML artifacts use
// (davinci-road/plan/assertion-allowlist.toml, davinci-road/plan/taxonomy.toml,
// davinci-road/plan/budgets.toml) and rejects everything else loudly instead
// of guessing:
//
//   - `#` comments and blank lines
//   - `[table]` and `[[array-of-tables]]` headers with bare dotted keys
//   - `key = value` with bare keys ([A-Za-z0-9_-]+)
//   - values: basic strings "…" (escapes: \" \\ \n \r \t \uXXXX),
//     literal strings '…' (no escapes), integers, plain decimal floats
//     (`-?digits.digits` — no exponents, no `inf`/`nan`), booleans,
//     (possibly multi-line) arrays of those scalars, and single-line inline
//     tables `{ key = value, … }` (the shape budgets.toml uses for its
//     uniform per-bench entries)
//
// Dates are written as quoted strings by convention in these files
// (e.g. expires = "2026-12-31"); native TOML date values are rejected.
// Duplicate keys and duplicate non-array table headers are errors.

class TomlLiteError extends Error {
  constructor(message, line) {
    super(line == null ? message : `line ${line}: ${message}`);
    this.name = "TomlLiteError";
    this.line = line;
  }
}

const BARE_KEY = /^[A-Za-z0-9_-]+$/;

function parseDottedKey(raw, lineNo) {
  const parts = raw.split(".").map((part) => part.trim());
  if (parts.length === 0 || parts.some((part) => !BARE_KEY.test(part))) {
    throw new TomlLiteError(`invalid key ${JSON.stringify(raw)} (bare dotted keys only)`, lineNo);
  }
  return parts;
}

function unescapeBasicString(body, lineNo) {
  let result = "";
  for (let i = 0; i < body.length; i += 1) {
    const ch = body[i];
    if (ch !== "\\") {
      result += ch;
      continue;
    }
    const escape = body[i + 1];
    i += 1;
    if (escape === '"') result += '"';
    else if (escape === "\\") result += "\\";
    else if (escape === "n") result += "\n";
    else if (escape === "r") result += "\r";
    else if (escape === "t") result += "\t";
    else if (escape === "u") {
      const hex = body.slice(i + 1, i + 5);
      if (!/^[0-9a-fA-F]{4}$/.test(hex)) {
        throw new TomlLiteError(`invalid \\u escape in string`, lineNo);
      }
      result += String.fromCharCode(Number.parseInt(hex, 16));
      i += 4;
    } else {
      throw new TomlLiteError(`unsupported escape \\${escape ?? "<eof>"} in string`, lineNo);
    }
  }
  return result;
}

// Tokenizes a value expression. Returns { value, rest } where rest is the
// remaining text after the value (used for arrays and trailing comments).
function readValue(text, lineNo) {
  const trimmed = text.trimStart();
  if (trimmed.startsWith('"')) {
    for (let i = 1; i < trimmed.length; i += 1) {
      if (trimmed[i] === "\\") {
        i += 1;
      } else if (trimmed[i] === '"') {
        return {
          value: unescapeBasicString(trimmed.slice(1, i), lineNo),
          rest: trimmed.slice(i + 1),
        };
      }
    }
    throw new TomlLiteError("unterminated basic string", lineNo);
  }
  if (trimmed.startsWith("'")) {
    const end = trimmed.indexOf("'", 1);
    if (end === -1) throw new TomlLiteError("unterminated literal string", lineNo);
    return { value: trimmed.slice(1, end), rest: trimmed.slice(end + 1) };
  }
  if (trimmed.startsWith("[")) {
    const items = [];
    let rest = trimmed.slice(1);
    for (;;) {
      rest = rest.trimStart();
      if (rest.startsWith("#")) {
        // comment inside a multi-line array: discard to end of line
        const newline = rest.indexOf("\n");
        if (newline === -1) throw new TomlLiteError("unterminated array", lineNo);
        rest = rest.slice(newline + 1);
        continue;
      }
      if (rest.startsWith("]")) {
        return { value: items, rest: rest.slice(1) };
      }
      if (rest.length === 0) throw new TomlLiteError("unterminated array", lineNo);
      const item = readValue(rest, lineNo);
      items.push(item.value);
      rest = item.rest.trimStart();
      if (rest.startsWith(",")) {
        rest = rest.slice(1);
      } else if (!rest.startsWith("]") && !rest.startsWith("#") && rest.length > 0) {
        throw new TomlLiteError("expected `,` or `]` in array", lineNo);
      }
    }
  }
  if (trimmed.startsWith("{")) {
    const table = {};
    let rest = trimmed.slice(1);
    for (;;) {
      // Inline tables are single-line per the TOML spec, so a newline before
      // the closing brace is an error rather than a continuation.
      rest = rest.replace(/^[ \t]+/, "");
      if (rest.startsWith("}")) return { value: table, rest: rest.slice(1) };
      if (rest.length === 0 || rest.startsWith("\n")) {
        throw new TomlLiteError("unterminated inline table", lineNo);
      }
      const equals = rest.indexOf("=");
      if (equals === -1) throw new TomlLiteError("expected key = value in inline table", lineNo);
      const keyParts = parseDottedKey(rest.slice(0, equals), lineNo);
      if (keyParts.length !== 1) {
        throw new TomlLiteError("dotted keys in inline tables are not supported", lineNo);
      }
      const key = keyParts[0];
      if (key in table) throw new TomlLiteError(`duplicate key ${key} in inline table`, lineNo);
      const item = readValue(rest.slice(equals + 1), lineNo);
      table[key] = item.value;
      rest = item.rest.replace(/^[ \t]+/, "");
      if (rest.startsWith(",")) {
        rest = rest.slice(1);
      } else if (!rest.startsWith("}")) {
        throw new TomlLiteError("expected `,` or `}` in inline table", lineNo);
      }
    }
  }
  const scalar = /^[^\s,\]}#]+/.exec(trimmed);
  if (scalar == null) throw new TomlLiteError("missing value", lineNo);
  const raw = scalar[0];
  const rest = trimmed.slice(raw.length);
  if (raw === "true") return { value: true, rest };
  if (raw === "false") return { value: false, rest };
  if (/^-?[0-9]+$/.test(raw)) return { value: Number.parseInt(raw, 10), rest };
  if (/^-?[0-9]+\.[0-9]+$/.test(raw)) return { value: Number.parseFloat(raw), rest };
  throw new TomlLiteError(
    `unsupported value ${JSON.stringify(raw)} (quote dates and other strings)`,
    lineNo,
  );
}

function assertOnlyComment(rest, lineNo) {
  const trailing = rest.trim();
  if (trailing.length > 0 && !trailing.startsWith("#")) {
    throw new TomlLiteError(`unexpected trailing content ${JSON.stringify(trailing)}`, lineNo);
  }
}

function navigate(root, parts, lineNo, { arrayTable }) {
  let node = root;
  for (let i = 0; i < parts.length; i += 1) {
    const key = parts[i];
    const last = i === parts.length - 1;
    if (last && arrayTable) {
      if (!(key in node)) node[key] = [];
      if (!Array.isArray(node[key])) {
        throw new TomlLiteError(`key ${key} is not an array of tables`, lineNo);
      }
      const table = {};
      node[key].push(table);
      return table;
    }
    if (!(key in node)) {
      node[key] = {};
    } else if (last && !arrayTable) {
      throw new TomlLiteError(`duplicate table [${parts.join(".")}]`, lineNo);
    }
    const next = node[key];
    node = Array.isArray(next) ? next[next.length - 1] : next;
    if (node == null || typeof node !== "object") {
      throw new TomlLiteError(`key ${key} is not a table`, lineNo);
    }
  }
  return node;
}

export function parseTomlLite(text) {
  const root = {};
  let current = root;
  const lines = text.split("\n");
  for (let index = 0; index < lines.length; index += 1) {
    const lineNo = index + 1;
    const line = lines[index].trim();
    if (line.length === 0 || line.startsWith("#")) continue;
    if (line.startsWith("[[")) {
      const end = line.indexOf("]]");
      if (end === -1) throw new TomlLiteError("malformed [[table]] header", lineNo);
      assertOnlyComment(line.slice(end + 2), lineNo);
      current = navigate(root, parseDottedKey(line.slice(2, end), lineNo), lineNo, {
        arrayTable: true,
      });
      continue;
    }
    if (line.startsWith("[")) {
      const end = line.indexOf("]");
      if (end === -1) throw new TomlLiteError("malformed [table] header", lineNo);
      assertOnlyComment(line.slice(end + 1), lineNo);
      current = navigate(root, parseDottedKey(line.slice(1, end), lineNo), lineNo, {
        arrayTable: false,
      });
      continue;
    }
    const equals = line.indexOf("=");
    if (equals === -1)
      throw new TomlLiteError(`expected key = value, got ${JSON.stringify(line)}`, lineNo);
    const keyParts = parseDottedKey(line.slice(0, equals), lineNo);
    if (keyParts.length !== 1) {
      throw new TomlLiteError("dotted keys in assignments are not supported", lineNo);
    }
    const key = keyParts[0];
    if (key in current) throw new TomlLiteError(`duplicate key ${key}`, lineNo);
    // Multi-line arrays: keep appending source lines until the value closes.
    let valueText = line.slice(equals + 1);
    let parsed;
    for (;;) {
      try {
        parsed = readValue(valueText, lineNo);
        break;
      } catch (error) {
        const unterminated =
          error instanceof TomlLiteError && /^line \d+: unterminated array/.test(error.message);
        if (unterminated && index + 1 < lines.length) {
          index += 1;
          valueText += `\n${lines[index]}`;
          continue;
        }
        throw error;
      }
    }
    assertOnlyComment(parsed.rest, lineNo);
    current[key] = parsed.value;
  }
  return root;
}

export { TomlLiteError };
