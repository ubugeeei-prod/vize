#!/usr/bin/env node
// Corpus construct-coverage report (Davinci P0-6).
//
// Scans the HYDRATED corpus projects registered in
// tests/_fixtures/vue-ecosystem-fixtures.json for the construct-taxonomy
// dimensions in davinci-road/plan/taxonomy.toml and emits
// davinci-road/plan/corpus-coverage.md: per-construct x per-project counts
// plus a scope-proof footer (hydrated-project count vs manifest total).
//
// Inputs per project:
//   - `vueGlobs`        — *.vue scanned as SFCs (template block html or pug),
//                         *.tsx / *.jsx scanned as JSX sources
//   - `petiteVueGlobs`  — *.html / *.js scanned with the HTML scanner
//                         (petite-vue corpus entries; see the manifest notes)
//
// Hydration mirrors tools/fixtures/glyph-corpus.mjs: a project counts as
// hydrated when its fixture directory exists and is non-empty. Absent
// fixtures are excluded from every table, and the scope-proof footer says
// so loudly — the assurance rule is empty-means-proven-empty, never
// silently partial.
//
// The scan is a lexical pass, not a compile: what is mechanically derived
// and what is skipped is spelled out in the report's "Skipped" section
// (binding sources in particular are presence signals, not per-expression
// attribution).
//
// Modes:
//   (default)  dry run — scan and print a summary; nothing is written
//   --write    write davinci-road/plan/corpus-coverage.md
//   --check    verify the committed report byte-matches a fresh scan
//              (exit 1 on drift)
//
// Determinism contract: output depends only on taxonomy.toml, the fixtures
// manifest, and the hydrated fixture trees. Projects iterate in manifest
// order, files in byte order; no timestamps, no absolute paths.
//
// Exit codes: 0 = success / up to date, 1 = --check found drift,
// 2 = usage or input validation error.

import { Buffer } from "node:buffer";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { parseTomlLite, TomlLiteError } from "./toml-lite.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const taxonomyPath = path.join(repoRoot, "davinci-road", "plan", "taxonomy.toml");
const manifestPath = path.join(repoRoot, "tests", "_fixtures", "vue-ecosystem-fixtures.json");
const reportPath = path.join(repoRoot, "davinci-road", "plan", "corpus-coverage.md");

function fail(message) {
  console.error(message);
  process.exit(2);
}

// --------------------------------------------------------------------------
// Inputs

function loadTaxonomy() {
  try {
    return parseTomlLite(fs.readFileSync(taxonomyPath, "utf8"));
  } catch (error) {
    if (error instanceof TomlLiteError)
      fail(`malformed taxonomy ${taxonomyPath}: ${error.message}`);
    throw error;
  }
}

function loadProjects() {
  const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
  return manifest.projects.map((project) => {
    const fixtureDir = path.resolve(repoRoot, project.fixturePath);
    const hydrated = fs.existsSync(fixtureDir) && fs.readdirSync(fixtureDir).length > 0;
    return { ...project, fixtureDir, hydrated };
  });
}

function collectFiles(cwd, patterns) {
  return [
    ...new Set(
      patterns.flatMap((pattern) =>
        fs
          .globSync(pattern, { cwd, exclude: [".yarn/**", "**/node_modules/**"] })
          .filter((entry) => fs.statSync(path.resolve(cwd, entry)).isFile())
          .map((entry) => entry.replaceAll("\\", "/")),
      ),
    ),
  ]
    .map((file) => ({ file, bytes: Buffer.from(file) }))
    .sort((left, right) => Buffer.compare(left.bytes, right.bytes))
    .map(({ file }) => file);
}

// --------------------------------------------------------------------------
// Tag / attribute classification against the taxonomy

// SVG/MathML-namespace child tags that do not collide with an HTML element
// name; ambiguous names (a, title, style, script, set, ...) stay `native`.
const SVG_ONLY_TAGS = new Set([
  "animate",
  "animateMotion",
  "animateTransform",
  "circle",
  "clipPath",
  "defs",
  "desc",
  "ellipse",
  "feBlend",
  "feColorMatrix",
  "feComponentTransfer",
  "feComposite",
  "feConvolveMatrix",
  "feDiffuseLighting",
  "feDisplacementMap",
  "feDistantLight",
  "feDropShadow",
  "feFlood",
  "feFuncA",
  "feFuncB",
  "feFuncG",
  "feFuncR",
  "feGaussianBlur",
  "feImage",
  "feMerge",
  "feMergeNode",
  "feMorphology",
  "feOffset",
  "fePointLight",
  "feSpecularLighting",
  "feSpotLight",
  "feTile",
  "feTurbulence",
  "foreignObject",
  "g",
  "linearGradient",
  "marker",
  "mask",
  "metadata",
  "mpath",
  "path",
  "pattern",
  "polygon",
  "polyline",
  "radialGradient",
  "rect",
  "stop",
  "symbol",
  "text",
  "textPath",
  "tspan",
  "use",
  "view",
]);
const MATHML_ONLY_TAGS = new Set([
  "annotation",
  "annotation-xml",
  "maction",
  "merror",
  "mfrac",
  "mi",
  "mmultiscripts",
  "mn",
  "mo",
  "mover",
  "mpadded",
  "mphantom",
  "mprescripts",
  "mroot",
  "mrow",
  "ms",
  "mspace",
  "msqrt",
  "mstyle",
  "msub",
  "msubsup",
  "msup",
  "mtable",
  "mtd",
  "mtext",
  "mtr",
  "munder",
  "munderover",
  "semantics",
]);

function classifyTag(tag) {
  if (tag === "slot") return "slot";
  if (tag === "template") return "template";
  if (tag === "svg" || SVG_ONLY_TAGS.has(tag)) return "svg";
  if (tag === "math" || MATHML_ONLY_TAGS.has(tag)) return "mathml";
  if (/^[A-Z]/.test(tag) || tag.includes("-")) return "component";
  return "native";
}

const BUILTIN_DIRECTIVES = new Set([
  "v-if",
  "v-else-if",
  "v-else",
  "v-for",
  "v-on",
  "v-bind",
  "v-model",
  "v-show",
  "v-html",
  "v-text",
  "v-once",
  "v-memo",
  "v-cloak",
  "v-pre",
]);
const EVENT_MODIFIERS = new Set(["stop", "prevent", "capture", "self", "once", "passive"]);
const KEY_MODIFIERS = new Set([
  "enter",
  "tab",
  "delete",
  "esc",
  "space",
  "up",
  "down",
  "left",
  "right",
  "ctrl",
  "alt",
  "shift",
  "meta",
  "exact",
]);
// Mouse-button modifiers are `left`/`right` (shared with the key class,
// disambiguated by event name below) plus `middle` (unambiguous).
const MOUSE_EVENTS = new Set(["click", "dblclick", "mousedown", "mouseup", "contextmenu"]);
const BIND_MODIFIERS = new Set(["prop", "camel", "attr"]);
const MODEL_MODIFIERS = new Set(["lazy", "number", "trim"]);

/**
 * Classify one attribute name into `{ directive, modifierClasses, vSlot }`.
 * `directive` is a taxonomy [[directive]] id or null; `vSlot` marks v-slot /
 * `#` shorthand occurrences, which have no taxonomy row.
 */
function classifyAttribute(rawName) {
  let name = rawName;
  let directive = null;
  let arg = "";
  let vSlot = false;

  if (name.startsWith("@")) {
    directive = "v-on";
    name = name.slice(1);
  } else if (name.startsWith(":")) {
    directive = "v-bind";
    name = name.slice(1);
  } else if (name.startsWith("#")) {
    vSlot = true;
    name = name.slice(1);
  } else if (name.startsWith(".")) {
    // `.prop`-shorthand binding (`.innerHTML="x"`).
    return { directive: "v-bind", modifierClasses: ["v-bind"], vSlot: false };
  } else if (name.startsWith("v-")) {
    const base = name.split(":", 1)[0].split(".", 1)[0];
    if (base === "v-slot") {
      vSlot = true;
    } else if (BUILTIN_DIRECTIVES.has(base)) {
      directive = base;
    } else {
      directive = "custom";
    }
    name = name.slice(base.length);
    if (name.startsWith(":")) name = name.slice(1);
  } else {
    return null;
  }

  const segments = name.split(".");
  arg = segments[0] ?? "";
  const modifiers = segments.slice(1).filter((segment) => segment.length > 0);
  const modifierClasses = [];
  for (const modifier of modifiers) {
    if (directive === "v-on") {
      if (EVENT_MODIFIERS.has(modifier)) {
        modifierClasses.push("event");
      } else if (modifier === "middle") {
        modifierClasses.push("mouse-button");
      } else if (
        (modifier === "left" || modifier === "right") &&
        MOUSE_EVENTS.has(arg.toLowerCase())
      ) {
        modifierClasses.push("mouse-button");
      } else if (KEY_MODIFIERS.has(modifier)) {
        modifierClasses.push("key");
      }
      // Custom key aliases and unknown tokens are ignored (see "Skipped").
    } else if (directive === "v-bind" && BIND_MODIFIERS.has(modifier)) {
      modifierClasses.push("v-bind");
    } else if (directive === "v-model" && MODEL_MODIFIERS.has(modifier)) {
      modifierClasses.push("v-model");
    }
  }
  return { directive, modifierClasses, vSlot };
}

// --------------------------------------------------------------------------
// Scanners. Each returns partial counts merged into the project row.

function emptyCounts(taxonomy) {
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

const START_TAG_RE = /<([A-Za-z][A-Za-z0-9-]*)((?:"[^"]*"|'[^']*'|[^"'>])*?)\/?>/g;
const ATTR_RE = /([^\s"'<>/=]+)(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s"'=<>`]+))?/g;

function recordAttrName(rawName, counts) {
  const classified = classifyAttribute(rawName);
  if (!classified) return;
  if (classified.vSlot) counts.vSlot += 1;
  if (classified.directive) counts.directive[classified.directive] += 1;
  for (const modifierClass of classified.modifierClasses) {
    counts.modifierClass[modifierClass] += 1;
  }
}

/** HTML-ish markup: SFC templates, petite-vue pages. Comments stripped. */
function scanHtml(source, counts) {
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
function scanPug(source, counts) {
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
function scanJsx(source, counts) {
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
          if (EVENT_MODIFIERS.has(modifier)) {
            counts.modifierClass.event += 1;
          } else if (modifier === "middle") {
            counts.modifierClass["mouse-button"] += 1;
          } else if (
            (modifier === "left" || modifier === "right") &&
            MOUSE_EVENTS.has(eventName.toLowerCase())
          ) {
            counts.modifierClass["mouse-button"] += 1;
          } else if (KEY_MODIFIERS.has(modifier)) {
            counts.modifierClass.key += 1;
          }
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

const BLOCK_OPEN_RE = /^<(template|script|style)\b([^>]*)>/;

/** Top-level SFC blocks via the column-0 heuristic used across vue tooling. */
function sfcBlocks(source) {
  const blocks = [];
  const lines = source.split("\n");
  let current = null;
  for (const line of lines) {
    if (current) {
      if (line.startsWith(`</${current.tag}`)) {
        blocks.push(current);
        current = null;
      } else {
        current.content.push(line);
      }
      continue;
    }
    const open = BLOCK_OPEN_RE.exec(line);
    if (!open) continue;
    const [, tag, attrsText] = open;
    const attrs = {};
    for (const attrMatch of attrsText.matchAll(ATTR_RE)) {
      const value = /=\s*(?:"([^"]*)"|'([^']*)')/.exec(attrMatch[0]);
      attrs[attrMatch[1]] = value ? (value[1] ?? value[2]) : true;
    }
    const selfClosed = /\/>\s*$/.test(line);
    const inlineClose = line.includes(`</${tag}>`);
    current = { tag, attrs, content: [] };
    if (selfClosed || inlineClose) {
      blocks.push(current);
      current = null;
    }
  }
  if (current) blocks.push(current);
  return blocks;
}

const BLOCK_COMBINATION_VOCAB = ["template", "script", "script-setup", "style-scoped"];

function scanSfc(source, counts, taxonomy) {
  const blocks = sfcBlocks(source);
  const present = new Set();
  let pug = false;
  for (const block of blocks) {
    if (block.tag === "template") {
      present.add("template");
      const content = block.content.join("\n");
      if (block.attrs.lang === "pug") {
        pug = true;
        scanPug(content, counts);
      } else {
        scanHtml(content, counts);
      }
    } else if (block.tag === "script") {
      present.add(block.attrs.setup !== undefined ? "script-setup" : "script");
      const content = block.content.join("\n");
      if (block.attrs.setup !== undefined) counts.bindingSignal.setup += 1;
      if (/\bdefineProps\s*[<(]/.test(content) || /\bprops\s*:/.test(content)) {
        counts.bindingSignal.props += 1;
      }
      if (/\bdata\s*\(\s*\)\s*\{/.test(content) || /\bdata\s*:\s*\(\s*\)\s*=>/.test(content)) {
        counts.bindingSignal.data += 1;
      }
      if (/\binject\s*[:(]/.test(content)) counts.bindingSignal.inject += 1;
    } else if (block.tag === "style") {
      present.add(block.attrs.scoped !== undefined ? "style-scoped" : "style");
    }
  }
  counts.files[pug ? "sfcPug" : "sfc"] += 1;
  const byName = (left, right) => (left < right ? -1 : left > right ? 1 : 0);
  const presentKey = BLOCK_COMBINATION_VOCAB.filter((block) => present.has(block))
    .sort(byName)
    .join("+");
  if ([...present].every((block) => BLOCK_COMBINATION_VOCAB.includes(block))) {
    for (const combination of taxonomy.block_combination) {
      const comboKey = [...combination.blocks].sort(byName).join("+");
      if (comboKey === presentKey) counts.blockCombination[combination.id] += 1;
    }
  }
}

function scanProject(project, taxonomy) {
  const counts = emptyCounts(taxonomy);
  const vueGlobs = project.vueGlobs ?? [];
  const sfcFiles = collectFiles(
    project.fixtureDir,
    vueGlobs.filter((glob) => glob.endsWith(".vue")),
  );
  const jsxFiles = collectFiles(
    project.fixtureDir,
    vueGlobs.filter((glob) => glob.endsWith(".tsx") || glob.endsWith(".jsx")),
  );
  for (const file of sfcFiles) {
    scanSfc(fs.readFileSync(path.resolve(project.fixtureDir, file), "utf8"), counts, taxonomy);
  }
  for (const file of jsxFiles) {
    counts.files.jsx += 1;
    scanJsx(fs.readFileSync(path.resolve(project.fixtureDir, file), "utf8"), counts);
  }
  for (const file of collectFiles(project.fixtureDir, project.petiteVueGlobs ?? [])) {
    counts.files[file.endsWith(".html") ? "html" : "js"] += 1;
    scanHtml(fs.readFileSync(path.resolve(project.fixtureDir, file), "utf8"), counts);
  }
  return counts;
}

// --------------------------------------------------------------------------
// Report rendering (vp-canonical markdown tables: padded columns, numeric
// columns right-aligned)

function renderTable(header, alignRight, rows) {
  const all = [header, ...rows.map((row) => row.map(String))];
  const widths = header.map((_, column) => Math.max(3, ...all.map((row) => row[column].length)));
  const line = (row) =>
    `| ${row.map((cell, column) => cell[alignRight[column] ? "padStart" : "padEnd"](widths[column])).join(" | ")} |`;
  const separator = `| ${widths
    .map((width, column) => (alignRight[column] ? `${"-".repeat(width - 1)}:` : "-".repeat(width)))
    .join(" | ")} |`;
  return [line(header), separator, ...rows.map((row) => line(row.map(String)))].join("\n");
}

function dimensionTable(title, ids, hydrated, pick) {
  const header = ["project", ...ids];
  const alignRight = [false, ...ids.map(() => true)];
  const rows = hydrated.map((project) => [
    `\`${project.id}\``,
    ...ids.map((id) => pick(project.counts, id)),
  ]);
  const totals = ids.map((id) =>
    hydrated.reduce((sum, project) => sum + pick(project.counts, id), 0),
  );
  const seen = ids.map((id) => hydrated.filter((project) => pick(project.counts, id) > 0).length);
  rows.push(["**total sites**", ...totals]);
  rows.push(["**projects seen**", ...seen]);
  return `### ${title}\n\n${renderTable(header, alignRight, rows)}`;
}

function buildReport(taxonomy, projects) {
  const hydrated = projects.filter((project) => project.hydrated);
  const total = projects.length;
  const lines = [];
  lines.push(`<!-- GENERATED FILE — do not edit by hand.
     Regenerate: node tools/davinci/corpus-coverage.mjs --write
     Verify:     node tools/davinci/corpus-coverage.mjs --check
     Generator:  tools/davinci/corpus-coverage.mjs -->

# Corpus construct coverage

Counts of the [taxonomy.toml](./taxonomy.toml) construct dimensions observed in the **hydrated** corpus projects registered in \`tests/_fixtures/vue-ecosystem-fixtures.json\` (Davinci P0-6). This file is generated; it goes stale whenever the taxonomy, the fixtures manifest, or the set of hydrated fixture submodules changes — regenerate with \`--write\`, verify with \`--check\` (byte-compare). The \`--check\` staleness gate can only join \`tests/tooling/davinci-matrices.test.ts\` once CI hydrates the full corpus; until then the scope-proof footer below is the honesty mechanism.

## Scan scope

Sources scanned per hydrated project (from the manifest's \`vueGlobs\`, plus \`petiteVueGlobs\` for the petite-vue entries):`);
  lines.push("");
  lines.push(
    renderTable(
      ["project", "sfc (html)", "sfc (pug)", "jsx/tsx", "html", "js"],
      [false, true, true, true, true, true],
      hydrated.map((project) => [
        `\`${project.id}\``,
        project.counts.files.sfc,
        project.counts.files.sfcPug,
        project.counts.files.jsx,
        project.counts.files.html,
        project.counts.files.js,
      ]),
    ),
  );
  lines.push("");
  lines.push("## Per-construct counts (hydrated projects only)");
  lines.push("");
  lines.push(
    dimensionTable(
      "Dimension 1: element_kind (start-tag classes)",
      taxonomy.element_kind.map((entry) => entry.id),
      hydrated,
      (counts, id) => counts.elementKind[id],
    ),
  );
  lines.push("");
  lines.push(
    dimensionTable(
      "Dimension 2: directive (attribute names, incl. `:` / `@` shorthand)",
      taxonomy.directive.map((entry) => entry.id),
      hydrated,
      (counts, id) => counts.directive[id],
    ),
  );
  lines.push("");
  lines.push(
    dimensionTable(
      "Dimension 3: modifier_class (modifier tokens on the applicable directive)",
      taxonomy.modifier_class.map((entry) => entry.id),
      hydrated,
      (counts, id) => counts.modifierClass[id],
    ),
  );
  lines.push("");
  const bindingIds = ["setup", "props", "data", "inject"];
  lines.push(
    dimensionTable(
      "Dimension 4: binding_source — declaration-site presence signals (SFC file counts, NOT per-expression attribution)",
      bindingIds,
      hydrated,
      (counts, id) => counts.bindingSignal[id],
    ),
  );
  lines.push("");
  lines.push(
    dimensionTable(
      "Dimension 5: block_combination (SFCs whose top-level blocks match the combination exactly)",
      taxonomy.block_combination.map((entry) => entry.id),
      hydrated,
      (counts, id) => counts.blockCombination[id],
    ),
  );
  lines.push("");
  lines.push(`## Skipped (not mechanically derived by this scan)

- **binding_source per-expression attribution** — mapping each template identifier to its declaration site needs scope analysis (the croquis engine's job). The table above reports file-level declaration-site signals only (\`<script setup>\` present / \`defineProps\`-or-\`props:\` / \`data()\` / \`inject\`); the \`global\` source has no mechanical signal and is not measured at all.
- **\`v-slot\` / \`#\` shorthand** — scanned (${projects
    .filter((project) => project.hydrated)
    .reduce(
      (sum, project) => sum + project.counts.vSlot,
      0,
    )} occurrences across hydrated projects) but reported nowhere above: the taxonomy has no \`v-slot\` directive row today.
- **JSX plain props** — every JSX prop is an expression binding; counting them all as \`v-bind\` would be noise, so only \`v-*\` props and \`on[A-Z]*\` event props (counted as \`v-on\`, with \`_modifier\` suffixes matched to modifier classes) are classified.
- **petite-vue built-ins** — \`v-scope\` / \`v-effect\` have no taxonomy row and land in \`custom\` (the not-in-builtin-set escape hatch).
- **Lexical limits** — pug templates are scanned line-heuristically (no pug parse); wakapi's HTML interleaves Go \`{{ }}\` template actions that the scanner skims over; TSX start tags reuse an HTML regex (single-uppercase-letter names are dropped as probable type parameters, other generics can leak); SVG/MathML descendants count via a fixed unambiguous-name set, so namespace children whose names collide with HTML tags count as \`native\`; unknown \`v-on\` modifier tokens (custom key aliases) are ignored.
- **Element kinds in scripts** — render functions and template strings inside \`.js\`/\`.ts\` sources are not scanned; only the file classes in the scan-scope table are.

## Scope proof (assurance rule: empty means proven-empty, never silently partial)

- **Hydrated: ${hydrated.length} of ${total} manifest projects.**`);
  if (hydrated.length < total) {
    lines.push(`
> **PARTIAL CORPUS — this report measures ${hydrated.length}/${total} projects.** Every count above, including every zero, is a statement about the ${hydrated.length} hydrated projects only. The remaining ${total - hydrated.length} manifest projects are **unmeasured**, not empty. Do not read dimension coverage off this report until the full corpus is hydrated (P0-6 leaves the full-coverage step open pending corpus hydration in CI).`);
  } else {
    lines.push(`
All manifest projects were hydrated for this run: zeros above are proven-empty over the whole registered corpus.`);
  }
  lines.push("");
  return lines.join("\n");
}

// --------------------------------------------------------------------------
// Modes

function parseArgs(argv) {
  const args = { write: false, check: false };
  for (const arg of argv) {
    if (arg === "--write") args.write = true;
    else if (arg === "--check") args.check = true;
    else fail(`unknown argument: ${arg} (expected --write or --check)`);
  }
  if (args.write && args.check) fail("--write and --check are mutually exclusive");
  return args;
}

function main() {
  const args = parseArgs(process.argv.slice(2));
  const taxonomy = loadTaxonomy();
  const projects = loadProjects();
  for (const project of projects) {
    if (project.hydrated) project.counts = scanProject(project, taxonomy);
  }
  const report = buildReport(taxonomy, projects);
  const hydratedCount = projects.filter((project) => project.hydrated).length;

  if (args.check) {
    const committed = fs.existsSync(reportPath) ? fs.readFileSync(reportPath, "utf8") : null;
    if (committed !== report) {
      console.error(
        `${path.relative(repoRoot, reportPath)} is stale (or the hydrated fixture set changed). ` +
          "Regenerate: node tools/davinci/corpus-coverage.mjs --write",
      );
      process.exit(1);
    }
    console.log(
      `corpus-coverage: up to date (${hydratedCount}/${projects.length} projects hydrated)`,
    );
    return;
  }

  if (args.write) {
    fs.writeFileSync(reportPath, report);
    console.log(
      `wrote ${path.relative(repoRoot, reportPath)} (${hydratedCount}/${projects.length} projects hydrated)`,
    );
    return;
  }

  console.log(
    `dry run: would write ${path.relative(repoRoot, reportPath)} (${hydratedCount}/${projects.length} projects hydrated; pass --write)`,
  );
}

const invokedDirectly =
  process.argv[1] != null && fileURLToPath(import.meta.url) === path.resolve(process.argv[1]);
if (invokedDirectly) {
  main();
}

export { classifyAttribute, classifyTag, scanProject, sfcBlocks };
