#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { format } from "oxfmt";
import { SNAPSHOT_LOCALES } from "./published-snapshot-locales.mjs";
import { RESULT_PATH, snapshotPage } from "./published-snapshot-render.mjs";
import { updatePerformance, updateReadme } from "./published-snapshot-sections.mjs";
import {
  assertFreshSnapshot,
  validatePublishedSnapshot,
} from "./published-snapshot-validation.mjs";

export const REPO_ROOT = fileURLToPath(new URL("../../../", import.meta.url));
export const performancePath = (locale) =>
  `docs/content/${locale === "en" ? "" : `${locale}/`}architecture/performance.md`;

/** Prepare and validate every output before touching the working tree. */
export async function preparePublication(input, read) {
  const data = validatePublishedSnapshot(input);
  const sources = new Map([
    [RESULT_PATH, `${JSON.stringify(data, null, 2)}\n`],
    ["README.md", updateReadme(read("README.md"), data)],
  ]);
  for (const locale of Object.keys(SNAPSHOT_LOCALES)) {
    const page = performancePath(locale);
    sources.set(page, updatePerformance(read(page), data, locale));
    sources.set(
      page.replace(/performance\.md$/u, "performance-blacksmith.md"),
      snapshotPage(data, locale),
    );
  }
  const outputs = new Map();
  for (const [file, source] of sources) {
    const result = await format(file, source);
    if (result.errors.length)
      throw new Error(`benchmark publication: ${file}: ${JSON.stringify(result.errors)}`);
    outputs.set(file, result.code);
  }
  return outputs;
}

export async function publishSnapshot({
  root = REPO_ROOT,
  json,
  check = false,
  now = Date.now(),
} = {}) {
  const read = (file) => readFileSync(resolve(root, file), "utf8");
  const previous = JSON.parse(read(RESULT_PATH));
  const data = validatePublishedSnapshot(
    json ? JSON.parse(readFileSync(resolve(json), "utf8")) : previous,
  );
  if (json) assertFreshSnapshot(data, previous, now);
  const outputs = await preparePublication(data, read);
  const changed = [...outputs].filter(([file, source]) => read(file) !== source);
  if (check && changed.length)
    throw new Error(
      `benchmark publication: stale generated files: ${changed.map(([file]) => file).join(", ")}`,
    );
  if (!check) for (const [file, source] of changed) writeFileSync(resolve(root, file), source);
  return changed.map(([file]) => file);
}

async function main() {
  const { values } = parseArgs({
    options: { json: { type: "string" }, check: { type: "boolean" } },
  });
  const changed = await publishSnapshot(values);
  console.log(
    changed.length
      ? `Updated ${changed.length} benchmark files.`
      : "Published benchmark files are up to date.",
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
