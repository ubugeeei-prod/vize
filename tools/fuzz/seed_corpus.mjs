#!/usr/bin/env node
// Seeds fuzz/corpus/<target>/ from repository fixtures so libFuzzer starts
// with a coverage map that reflects realistic SFC and template shapes. The
// seed file content is reproducible from the repo (the .vue and template
// fixtures are all in git), so the grown corpus does not need to be
// checked in — CI caches it instead.
//
// Targets currently seeded:
//   - sfc_parse: whole .vue files
//   - template_compile: contents of <template>...</template> blocks
//
// Usage: node tools/fuzz/seed_corpus.mjs

import { createHash } from "node:crypto";
import { globSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(here, "..", "..");
const corpusRoot = join(repoRoot, "fuzz", "corpus");

const VUE_GLOBS = [
  "tests/fixtures/**/*.vue",
  "tests/_fixtures/_projects/**/*.vue",
  "playground/src/**/*.vue",
  "playground/e2e/**/*.vue",
];

function findVueFiles() {
  return VUE_GLOBS.flatMap((pattern) =>
    globSync(pattern, { cwd: repoRoot }).map((relativePath) => join(repoRoot, relativePath)),
  );
}

function hash(content) {
  return createHash("sha1").update(content).digest("hex").slice(0, 16);
}

function resetCorpus(target) {
  const dir = join(corpusRoot, target);
  rmSync(dir, { force: true, recursive: true });
  mkdirSync(dir, { recursive: true });
  return dir;
}

function writeSeed(dir, content) {
  if (content.length === 0) return;
  writeFileSync(join(dir, hash(content)), content);
}

function extractTemplateBlock(source) {
  // The fuzz target consumes raw template source (the content between
  // <template> and </template>, with all attributes stripped). This pulls
  // the *first* template block from each SFC. Skipped if the file has no
  // template (e.g. script-only fixtures).
  const open = source.match(/<template\b[^>]*>/);
  if (!open) return null;
  const start = open.index + open[0].length;
  const closeIdx = source.indexOf("</template>", start);
  if (closeIdx === -1) return null;
  return source.slice(start, closeIdx);
}

function main() {
  const sfcDir = resetCorpus("sfc_parse");
  const templateDir = resetCorpus("template_compile");

  const files = findVueFiles();
  let sfcCount = 0;
  let templateCount = 0;
  for (const path of files) {
    const content = readFileSync(path, "utf8");
    writeSeed(sfcDir, content);
    sfcCount += 1;

    const template = extractTemplateBlock(content);
    if (template != null) {
      writeSeed(templateDir, template);
      templateCount += 1;
    }
  }

  process.stdout.write(
    `Seeded ${sfcCount} sfc_parse entries and ${templateCount} template_compile entries from ${files.length} fixtures.\n`,
  );
}

main();
