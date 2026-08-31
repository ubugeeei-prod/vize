// Per-project scanning for the corpus construct-coverage report (Davinci
// P0-6): SFC block splitting plus the file enumeration that routes each
// hydrated source to its scanner. Files iterate in byte order so the report
// stays byte-identical across platforms.

import { Buffer } from "node:buffer";
import fs from "node:fs";
import path from "node:path";

import { ATTR_RE, emptyCounts, scanHtml, scanJsx, scanPug } from "./corpus-coverage-scan.mjs";
import { byKey } from "./ordering.mjs";

const BLOCK_OPEN_RE = /^<(template|script|style)\b([^>]*)>/;
const BLOCK_COMBINATION_VOCAB = ["template", "script", "script-setup", "style-scoped"];

export function collectFiles(cwd, patterns) {
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

/** Top-level SFC blocks via the column-0 heuristic used across vue tooling. */
export function sfcBlocks(source) {
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

function scanScriptBlock(content, setup, counts) {
  if (setup) counts.bindingSignal.setup += 1;
  if (/\bdefineProps\s*[<(]/.test(content) || /\bprops\s*:/.test(content)) {
    counts.bindingSignal.props += 1;
  }
  if (/\bdata\s*\(\s*\)\s*\{/.test(content) || /\bdata\s*:\s*\(\s*\)\s*=>/.test(content)) {
    counts.bindingSignal.data += 1;
  }
  if (/\binject\s*[:(]/.test(content)) counts.bindingSignal.inject += 1;
}

export function scanSfc(source, counts, taxonomy) {
  const present = new Set();
  let pug = false;
  for (const block of sfcBlocks(source)) {
    const content = block.content.join("\n");
    if (block.tag === "template") {
      present.add("template");
      if (block.attrs.lang === "pug") {
        pug = true;
        scanPug(content, counts);
      } else {
        scanHtml(content, counts);
      }
    } else if (block.tag === "script") {
      const setup = block.attrs.setup !== undefined;
      present.add(setup ? "script-setup" : "script");
      scanScriptBlock(content, setup, counts);
    } else if (block.tag === "style") {
      present.add(block.attrs.scoped !== undefined ? "style-scoped" : "style");
    }
  }
  counts.files[pug ? "sfcPug" : "sfc"] += 1;
  const presentKey = BLOCK_COMBINATION_VOCAB.filter((block) => present.has(block))
    .sort(byKey)
    .join("+");
  if ([...present].every((block) => BLOCK_COMBINATION_VOCAB.includes(block))) {
    for (const combination of taxonomy.block_combination) {
      const comboKey = [...combination.blocks].sort(byKey).join("+");
      if (comboKey === presentKey) counts.blockCombination[combination.id] += 1;
    }
  }
}

export function scanProject(project, taxonomy) {
  const counts = emptyCounts(taxonomy);
  const vueGlobs = project.vueGlobs ?? [];
  const read = (file) => fs.readFileSync(path.resolve(project.fixtureDir, file), "utf8");
  const sfcFiles = collectFiles(
    project.fixtureDir,
    vueGlobs.filter((glob) => glob.endsWith(".vue")),
  );
  const jsxFiles = collectFiles(
    project.fixtureDir,
    vueGlobs.filter((glob) => glob.endsWith(".tsx") || glob.endsWith(".jsx")),
  );
  for (const file of sfcFiles) {
    scanSfc(read(file), counts, taxonomy);
  }
  for (const file of jsxFiles) {
    counts.files.jsx += 1;
    scanJsx(read(file), counts);
  }
  for (const file of collectFiles(project.fixtureDir, project.petiteVueGlobs ?? [])) {
    counts.files[file.endsWith(".html") ? "html" : "js"] += 1;
    scanHtml(read(file), counts);
  }
  return counts;
}
