// Natural modified-v-on corpus scan shared by the v-on storage gate
// (tests/tooling/davinci-v-on-storage.test.ts) and the committed inventory
// generator (tools/support/compat/davinci/v-on-corpus.mjs).
//
// Reviewed scope: Git-tracked template files, documentation, and source/test
// formats that carry inline template fixtures. Binary/generated/untracked
// files are outside the inventory. JS-family carriers are explicit so a
// fixture moved from TS to JS cannot silently disappear from the evidence.
// Baseline: 9e18d171c3ef3a16021dff4debeab21195f99017, immediately before the
// SmallVec change. The current Git-tracked corpus keeps being scanned so
// future natural spellings force an intentional capacity/evidence update,
// while the marked synthetic boundary cases never justify their own chosen
// capacity.
//
// The committed evidence is sharded one TSV per source area
// (docs/davinci/plan/v-on-corpus/<area>.tsv, one spelling per line, no
// totals), so two PRs adding fixtures in different crates never edit the
// same committed line.

import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";

import { repoRoot } from "./paths.mjs";

// The gate itself carries lookalike and boundary fixtures for its unit tests.
export const V_ON_GATE = "tests/tooling/davinci-v-on-storage.test.ts";
export const V_ON_CORPUS_DIR = "docs/davinci/plan/v-on-corpus";
export const V_ON_CORPUS_REGEN = "rust-script tools/commands/davinci/v-on-corpus.rs --write";

export const templateCarrierExtensions = new Set([
  ".astro",
  ".cjs",
  ".cts",
  ".html",
  ".js",
  ".jsx",
  ".md",
  ".mjs",
  ".mts",
  ".rs",
  ".svelte",
  ".ts",
  ".tsx",
  ".vue",
]);

export const syntheticBoundary =
  /\/\/ v-on-storage-synthetic:start[\s\S]*?\/\/ v-on-storage-synthetic:end/gu;
const modifiedOnName = /^(?:@|v-on:)(?!\[)[^\s=./>]+(?:\.[^\s=./>]+)+$/u;

/** Git-tracked carrier files with marked synthetic boundaries removed, in `git ls-files` order. */
export function trackedNaturalSources() {
  const tracked = spawnSync("git", ["ls-files", "-z"], { cwd: repoRoot, encoding: "utf8" });
  if (tracked.status !== 0) throw new Error(`git ls-files failed: ${tracked.stderr}`);
  return tracked.stdout
    .split("\0")
    .filter(
      (file) =>
        file !== "" && file !== V_ON_GATE && templateCarrierExtensions.has(path.extname(file)),
    )
    .map((file) => ({
      file,
      source: readFileSync(path.join(repoRoot, file), "utf8").replace(syntheticBoundary, ""),
    }));
}

export function startTags(source) {
  const tags = [];
  for (let start = 0; start < source.length - 1; start += 1) {
    if (source[start] !== "<" || !/[A-Za-z]/u.test(source[start + 1])) continue;

    let quote;
    for (let end = start + 2; end < source.length; end += 1) {
      const character = source[end];
      if (quote !== undefined) {
        if (character === quote) quote = undefined;
      } else if (character === '"' || character === "'") {
        quote = character;
      } else if (character === ">") {
        tags.push(source.slice(start, end + 1));
        start = end;
        break;
      } else if (character === "<") {
        break;
      }
    }
  }
  return tags;
}

function attributeNames(tag) {
  const names = [];
  let cursor = 1;
  while (cursor < tag.length && !/[\s/>]/u.test(tag[cursor])) cursor += 1;

  while (cursor < tag.length) {
    while (/\s/u.test(tag[cursor] ?? "")) cursor += 1;
    if (tag[cursor] === "/" || tag[cursor] === ">" || tag[cursor] === undefined) break;

    const nameStart = cursor;
    while (!/[\s=/>]/u.test(tag[cursor] ?? ">")) cursor += 1;
    names.push(tag.slice(nameStart, cursor));
    while (/\s/u.test(tag[cursor] ?? "")) cursor += 1;
    if (tag[cursor] !== "=") continue;

    cursor += 1;
    while (/\s/u.test(tag[cursor] ?? "")) cursor += 1;
    const escapedQuote =
      tag[cursor] === "\\" && (tag[cursor + 1] === '"' || tag[cursor + 1] === "'");
    const quote = escapedQuote ? tag[cursor + 1] : tag[cursor];
    if (quote === '"' || quote === "'") {
      cursor += escapedQuote ? 2 : 1;
      while (
        cursor < tag.length &&
        (escapedQuote ? tag[cursor] !== "\\" || tag[cursor + 1] !== quote : tag[cursor] !== quote)
      ) {
        cursor += 1;
      }
      cursor += escapedQuote ? 2 : 1;
    } else {
      while (!/[\s>]/u.test(tag[cursor] ?? ">")) cursor += 1;
    }
  }
  return names;
}

export function modifiedOnSpellings(source) {
  return startTags(source).flatMap((tag) =>
    attributeNames(tag).filter((name) => modifiedOnName.test(name)),
  );
}

export function classify(spelling) {
  const normalized = spelling.startsWith("@") ? spelling.slice(1) : spelling.slice("v-on:".length);
  const [name, ...modifiers] = normalized.split(".");
  const keyboard = name === "keydown" || name === "keyup" || name === "keypress";
  const buckets = { options: 0, event: 0, keys: 0 };

  for (const modifier of modifiers) {
    if (modifier === "native") continue;
    if (modifier === "capture" || modifier === "once" || modifier === "passive") {
      buckets.options += 1;
    } else if ((modifier === "left" || modifier === "right") && keyboard) {
      buckets.keys += 1;
    } else if (
      [
        "stop",
        "prevent",
        "self",
        "ctrl",
        "shift",
        "alt",
        "meta",
        "middle",
        "exact",
        "left",
        "right",
      ].includes(modifier)
    ) {
      buckets.event += 1;
    } else {
      buckets.keys += 1;
    }
  }
  return buckets;
}

// Shard key: the first two directories of the path (`crates/vize_s2`,
// `playground/src`), or the directory itself for shallower files.
function areaOf(file) {
  const dirs = file.split("/").slice(0, -1);
  return dirs.length === 0 ? "root" : dirs.slice(0, 2).join("--");
}

/** The committed inventory: one `file<TAB>spelling` TSV per area, stable order, no totals. */
export function renderVOnCorpus(sources) {
  const byArea = new Map();
  for (const { file, source } of sources) {
    for (const spelling of modifiedOnSpellings(source)) {
      const area = areaOf(file);
      if (!byArea.has(area)) byArea.set(area, []);
      byArea.get(area).push(`${file}\t${spelling}`);
    }
  }
  return [...byArea.keys()]
    .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))
    .map((area) => ({
      relPath: `${V_ON_CORPUS_DIR}/${area}.tsv`,
      text: `file\tspelling\n${byArea.get(area).join("\n")}\n`,
    }));
}
