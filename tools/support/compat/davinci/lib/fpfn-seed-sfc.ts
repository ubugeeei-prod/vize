// Lexical SFC analysis for the seeded-defect generator (Davinci P0-13).
//
// This is deliberately NOT a Vue parser. The seeder only needs to pick, per
// file, one deterministic injection site per defect class, and it must never
// pick a site it does not fully understand — so every helper here errs on
// the side of declaring a file ineligible. The eligibility contract for
// class (a) (undefined template ref) is:
//
//   - the file has exactly one `<script setup>` block and one `<template>`
//     block;
//   - the candidate binding is a top-level `const`/`let`/`var`/`function`
//     declaration with a simple identifier name (destructuring skipped);
//   - the binding name occurs exactly once in the template's expression
//     positions ({{ ... }} interpolations and quoted values of directive
//     attributes: v-*, :*, @*, #*) as a standalone identifier token, and
//     nowhere else in the template text;
//   - the name is not shadowed: it does not appear inside any v-for /
//     v-slot / #slot attribute value in the template;
//   - the seeded replacement name does not already occur in the file.
//
// "First eligible" (the deterministic choice mandated by the plan) means:
// bindings in declaration order, first one meeting every clause above.

// Top-level SFC blocks only: both the opening and the closing tag must sit
// at the start of a line (nested <template v-slot> elements are indented in
// any conventionally formatted source; unconventional files simply come out
// ineligible via the count guards below).
const BLOCK_RE = /^<(script|template)\b([^>]*)>([\s\S]*?)^<\/\1>/gm;
const DECL_RE = /^(?:export\s+)?(?:const|let|var|function)\s+([A-Za-z_$][A-Za-z0-9_$]*)/;
const INTERPOLATION_RE = /\{\{([\s\S]*?)\}\}/g;
const DIRECTIVE_ATTR_RE =
  /(^|\s)([v:@#][^\s"'<>/=]*)\s*=\s*"([^"]*)"|(^|\s)([v:@#][^\s"'<>/=]*)\s*=\s*'([^']*)'/g;

/** Extract the `<script setup>` and `<template>` blocks with spans. */
export function extractBlocks(source) {
  const blocks = { scriptSetup: null, template: null, scriptSetupCount: 0, templateCount: 0 };
  for (const match of source.matchAll(BLOCK_RE)) {
    const [full, tag, attrs, content] = match;
    const contentStart = match.index + full.length - content.length - `</${tag}>`.length;
    const record = {
      attrs,
      content,
      contentStart,
      contentEnd: contentStart + content.length,
    };
    if (tag === "script" && /(^|\s)setup(\s|=|$)/.test(attrs)) {
      blocks.scriptSetupCount += 1;
      blocks.scriptSetup = record;
    } else if (tag === "template" && attrs.trim() === "") {
      blocks.templateCount += 1;
      blocks.template = record;
    }
  }
  if (blocks.scriptSetupCount !== 1) blocks.scriptSetup = null;
  if (blocks.templateCount !== 1) blocks.template = null;
  return blocks;
}

/**
 * Top-level bindings of a `<script setup>` body, in declaration order.
 * A line-wise scan with brace/paren/bracket depth tracking; strings,
 * template literals, and comments are blanked before depth counting.
 */
export function topLevelBindings(scriptContent) {
  const blanked = blankScriptNoise(scriptContent);
  const bindings = [];
  let depth = 0;
  let lineStart = 0;
  for (let i = 0; i <= blanked.length; i += 1) {
    if (i === blanked.length || blanked[i] === "\n") {
      const line = blanked.slice(lineStart, i);
      if (depth === 0) {
        const declaration = DECL_RE.exec(line.trimStart());
        if (declaration) bindings.push(declaration[1]);
      }
      for (const char of line) {
        if (char === "{" || char === "(" || char === "[") depth += 1;
        else if (char === "}" || char === ")" || char === "]") depth -= 1;
      }
      lineStart = i + 1;
    }
  }
  return bindings;
}

/** Blank string/template-literal/comment contents, preserving offsets. */
export function blankScriptNoise(text) {
  let out = "";
  let mode = null; // '"' | "'" | "`" | "//" | "/*"
  for (let i = 0; i < text.length; i += 1) {
    const char = text[i];
    const next = text[i + 1];
    if (mode === null) {
      if (char === '"' || char === "'" || char === "`") {
        mode = char;
        out += char;
      } else if (char === "/" && next === "/") {
        mode = "//";
        out += "  ";
        i += 1;
      } else if (char === "/" && next === "*") {
        mode = "/*";
        out += "  ";
        i += 1;
      } else {
        out += char;
      }
    } else if (mode === "//") {
      if (char === "\n") {
        mode = null;
        out += "\n";
      } else out += " ";
    } else if (mode === "/*") {
      if (char === "*" && next === "/") {
        mode = null;
        out += "  ";
        i += 1;
      } else out += char === "\n" ? "\n" : " ";
    } else {
      // Inside a string or template literal.
      if (char === "\\") {
        out += "  ";
        i += 1;
      } else if (char === mode) {
        mode = null;
        out += char;
      } else out += char === "\n" ? "\n" : " ";
    }
  }
  return out;
}

/** Expression-position segments of a template body: [{start, text}] . */
export function templateExpressionSegments(templateContent) {
  const segments = [];
  for (const match of templateContent.matchAll(INTERPOLATION_RE)) {
    segments.push({ start: match.index + 2, text: match[1] });
  }
  for (const match of templateContent.matchAll(DIRECTIVE_ATTR_RE)) {
    const doubleQuoted = match[3] != null;
    const value = doubleQuoted ? match[3] : match[6];
    const valueStart = match.index + match[0].length - value.length - 1;
    segments.push({ start: valueStart, text: value });
  }
  return segments.sort((a, b) => a.start - b.start);
}

/**
 * Standalone identifier-token occurrences of `name` inside a segment text,
 * as offsets relative to the segment start. Occurrences inside string
 * literals of the expression, member accesses (`.name`), and object keys
 * (`name:`) are excluded.
 */
export function identifierOccurrences(segmentText, name) {
  const blanked = blankScriptNoise(segmentText);
  const occurrences = [];
  let from = 0;
  while (true) {
    const at = blanked.indexOf(name, from);
    if (at === -1) break;
    from = at + 1;
    const before = at === 0 ? "" : blanked[at - 1];
    const after = at + name.length >= blanked.length ? "" : blanked[at + name.length];
    if (/[A-Za-z0-9_$]/.test(before) || before === ".") continue;
    if (/[A-Za-z0-9_$]/.test(after)) continue;
    // Conservative: drop occurrences directly followed by `:` — object keys
    // (`{ name: 1 }`) are not references, and ternary middles are excluded
    // with them rather than risking a misclassified site.
    if (/^\s*:(?!:)/.test(blanked.slice(at + name.length))) continue;
    occurrences.push(at);
  }
  return occurrences;
}

/** Whether `name` is introduced by a v-for / v-slot / #slot value anywhere. */
export function isShadowedInTemplate(templateContent, name) {
  for (const match of templateContent.matchAll(DIRECTIVE_ATTR_RE)) {
    const attrName = match[2] ?? match[5];
    const value = match[3] ?? match[6];
    const isScopeDirective =
      attrName === "v-for" ||
      attrName === "v-slot" ||
      attrName.startsWith("v-slot:") ||
      attrName.startsWith("#");
    if (isScopeDirective && new RegExp(`\\b${escapeRegExp(name)}\\b`).test(value)) return true;
  }
  return false;
}

/** Count word-boundary occurrences of `name` in the whole template body. */
export function totalTemplateOccurrences(templateContent, name) {
  const matches = templateContent.match(
    new RegExp(`(?<![A-Za-z0-9_$])${escapeRegExp(name)}(?![A-Za-z0-9_$])`, "g"),
  );
  return matches == null ? 0 : matches.length;
}

export function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
