// Per-source lexical scanners for the corpus construct-coverage report
// (Davinci P0-6). Each scanner walks one source text and accumulates into the
// counts record created by `emptyCounts`; what these passes deliberately do
// not derive is spelled out in the report's "Skipped" section.

import { classifyAttribute, classifyModifier, classifyTag } from "./corpus-coverage-classify.mjs";

export const START_TAG_RE = /<([A-Za-z][A-Za-z0-9-]*)((?:"[^"]*"|'[^']*'|[^"'>])*?)\/?>/g;
export const ATTR_RE = /([^\s"'<>/=]+)(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s"'=<>`]+))?/g;

/** A zeroed counts record covering every taxonomy row, so zeros are explicit. */
export function emptyCounts(taxonomy) {
  const counts = {
    elementKind: {},
    directive: {},
    modifierClass: {},
    blockCombination: {},
    bindingSignal: { setup: 0, props: 0, data: 0, inject: 0 },
    vSlot: 0,
    files: { sfc: 0, sfcPug: 0, jsx: 0, html: 0, js: 0 },
  };
  for (const kind of taxonomy.element_kind) counts.elementKind[kind.id] = 0;
  for (const directive of taxonomy.directive) counts.directive[directive.id] = 0;
  for (const modifierClass of taxonomy.modifier_class) counts.modifierClass[modifierClass.id] = 0;
  for (const combination of taxonomy.block_combination) counts.blockCombination[combination.id] = 0;
  return counts;
}

export function recordAttrName(rawName, counts) {
  const classified = classifyAttribute(rawName);
  if (!classified) return;
  if (classified.vSlot) counts.vSlot += 1;
  if (classified.directive) counts.directive[classified.directive] += 1;
  for (const modifierClass of classified.modifierClasses) {
    counts.modifierClass[modifierClass] += 1;
  }
}

/** HTML-ish markup: SFC templates, petite-vue pages. Comments stripped. */
export function scanHtml(source, counts) {
  const stripped = source.replace(/<!--[\s\S]*?-->/g, "");
  for (const tagMatch of stripped.matchAll(START_TAG_RE)) {
    const [, tag, attrsText] = tagMatch;
    counts.elementKind[classifyTag(tag)] += 1;
    for (const attrMatch of attrsText.matchAll(ATTR_RE)) {
      recordAttrName(attrMatch[1], counts);
    }
  }
}

/**
 * Pug templates, line-heuristic: a leading identifier is a tag, leading
 * `.foo` / `#bar` is pug's implied div, attributes live in the parenthesized
 * group directly after the tag token — consumed across lines (formatted pug
 * routinely breaks one attribute per line). No pug parse — see "Skipped".
 */
export function scanPug(source, counts) {
  const lines = source.split("\n");
  let group = null; // { depth, quote, text, indent } inside a (...) attr group
  let skipIndent = null; // literal-block indent (`tag.` block text, `//` comments)
  const flushGroup = () => {
    // Blank out quoted attribute values first so value text never scans as
    // attribute names.
    const attrsText = group.text.replace(/"[^"]*"|'[^']*'|`[^`]*`/g, '""');
    for (const attrMatch of attrsText.matchAll(/(^|[\s,])([@:#.]?[A-Za-z][^\s=,()]*)/g)) {
      recordAttrName(attrMatch[2], counts);
    }
    group = null;
  };
  const consume = (text) => {
    for (let i = 0; i < text.length; i += 1) {
      const char = text[i];
      if (group.quote) {
        group.text += char;
        if (char === group.quote) group.quote = null;
        continue;
      }
      if (char === '"' || char === "'" || char === "`") {
        group.quote = char;
        group.text += char;
        continue;
      }
      if (char === "(") group.depth += 1;
      if (char === ")") {
        group.depth -= 1;
        if (group.depth === 0) {
          const indent = group.indent;
          flushGroup();
          // `tag(attrs).` opens a literal text block.
          if (text.slice(i + 1).trim() === ".") skipIndent = indent;
          return;
        }
      }
      group.text += char;
    }
    group.text += "\n";
  };
  for (const rawLine of lines) {
    if (group) {
      consume(rawLine);
      continue;
    }
    if (rawLine.trim() === "") continue;
    const indent = rawLine.length - rawLine.trimStart().length;
    if (skipIndent !== null) {
      if (indent > skipIndent) continue; // literal block content
      skipIndent = null;
    }
    const line = rawLine.trimStart();
    if (line.startsWith("//")) {
      skipIndent = indent; // pug block comment swallows deeper lines
      continue;
    }
    if (line.startsWith("|") || line.startsWith("-")) continue;
    let head = null;
    const tagMatch = /^([A-Za-z][A-Za-z0-9-]*)/.exec(line);
    if (tagMatch) {
      counts.elementKind[classifyTag(tagMatch[1])] += 1;
      head = tagMatch[1];
    } else if (/^[.#][A-Za-z_-]/.test(line)) {
      counts.elementKind.native += 1; // pug implied <div>
      head = "";
    } else {
      continue;
    }
    // The attr group must open directly after the tag token and its
    // optional `.class`/`#id` chain (pug syntax: `tag.cls#id(attrs)`).
    const rest = line.slice(head.length);
    const opener = /^[A-Za-z0-9_.#-]*\(/.exec(rest);
    if (opener) {
      group = { depth: 1, quote: null, text: "", indent };
      consume(line.slice(head.length + opener[0].length));
      continue;
    }
    // `tag.` / `.cls.` (bare dot, no inline text) opens a literal text block.
    if (/^[A-Za-z0-9_.#-]*\.$/.test(rest)) skipIndent = indent;
  }
  if (group) flushGroup();
}

/**
 * JSX/TSX sources. Start tags reuse the HTML tag regex; single-uppercase-
 * letter names are dropped as probable type parameters. `on[A-Z]*` props
 * count as v-on (the JSX event spelling); `_modifier` suffixes on them are
 * matched against the v-on modifier classes; `v-*` props classify like
 * template directives; plain props are NOT counted as v-bind (see
 * "Skipped").
 */
export function scanJsx(source, counts) {
  const stripped = source.replace(/\/\*[\s\S]*?\*\//g, "").replace(/(^|[^:])\/\/[^\n]*/g, "$1");
  for (const tagMatch of stripped.matchAll(START_TAG_RE)) {
    const [, tag, attrsText] = tagMatch;
    if (/^[A-Z]$/.test(tag)) continue;
    counts.elementKind[classifyTag(tag)] += 1;
    for (const attrMatch of attrsText.matchAll(ATTR_RE)) {
      const name = attrMatch[1];
      const eventProp = /^on[A-Z][A-Za-z0-9]*(?:_[a-z]+)*$/.exec(name);
      if (eventProp) {
        counts.directive["v-on"] += 1;
        const [eventName, ...modifiers] = name.slice(2).split("_");
        for (const modifier of modifiers) {
          const modifierClass = classifyModifier("v-on", eventName, modifier);
          if (modifierClass) counts.modifierClass[modifierClass] += 1;
        }
        continue;
      }
      if (name === "v-slot" || name === "v-slots" || name.startsWith("v-slot:")) {
        counts.vSlot += 1;
        continue;
      }
      if (name.startsWith("v-")) recordAttrName(name, counts);
    }
  }
}
