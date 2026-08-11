import { readFile } from "node:fs/promises";
import { posix } from "node:path";
import { gzipSync } from "node:zlib";

const distributionDirectory = new URL("../dist/", import.meta.url);
const staticImportPattern = /\b(?:import|export)\s+(?:[^"']*?\s+from\s+)?["'](\.\.?\/[^"']+)["']/g;
const budgets = new Map([
  ["index.mjs", 13_150],
  ["button.mjs", 1_600],
  ["checkbox.mjs", 1_900],
  ["collection.mjs", 5_700],
  ["context.mjs", 700],
  ["controllable-state.mjs", 600],
  ["id.mjs", 2_300],
  ["interaction-modality.mjs", 3_300],
  ["primitive.mjs", 800],
  ["visually-hidden.mjs", 800],
  ["media.mjs", 2_400],
  ["media-pdf.mjs", 2_048],
  ["media-source.mjs", 1_800],
]);

async function collectStaticDependencies(file, collected = new Map()) {
  if (collected.has(file)) return collected;

  const source = await readFile(new URL(file, distributionDirectory));
  collected.set(file, source);

  for (const match of source.toString().matchAll(staticImportPattern)) {
    const dependency = posix.normalize(posix.join(posix.dirname(file), match[1]));
    if (dependency === ".." || dependency.startsWith("../")) {
      throw new Error(`Output dependency escapes dist: ${dependency}`);
    }
    await collectStaticDependencies(dependency, collected);
  }

  return collected;
}

for (const [entry, maximumGzipBytes] of budgets) {
  const files = await collectStaticDependencies(entry);
  const gzipBytes = gzipSync(
    [...files.entries()]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([file, source]) => `/* ${file} */\n${source}`)
      .join("\n"),
  ).byteLength;

  console.log(
    JSON.stringify({
      entry: `@vizejs/ui/${entry.replace(/\.mjs$/, "")}`,
      files: files.size,
      gzipBytes,
      maximumGzipBytes,
    }),
  );

  if (gzipBytes > maximumGzipBytes) process.exitCode = 1;
}
