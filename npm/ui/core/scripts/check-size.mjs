import { readFile } from "node:fs/promises";
import { posix } from "node:path";
import { gzipSync } from "node:zlib";

const distributionDirectory = new URL("../dist/", import.meta.url);
const staticImportPattern = /\b(?:import|export)\s+(?:[^"']*?\s+from\s+)?["'](\.\.?\/[^"']+)["']/g;
const budgets = new Map([
  ["index.mjs", 124_300],
  ["alert.mjs", 1_050],
  ["announcer.mjs", 4_100],
  ["aspect-ratio.mjs", 1_500],
  ["button.mjs", 1_600],
  ["link.mjs", 3_150],
  ["toggle.mjs", 2_050],
  ["input.mjs", 3_900],
  ["textarea.mjs", 4_000],
  ["search-field.mjs", 4_400],
  ["separator.mjs", 900],
  ["skeleton.mjs", 1_250],
  ["meter.mjs", 3_700],
  ["switch.mjs", 3_750],
  ["checkbox.mjs", 1_900],
  ["collection.mjs", 5_700],
  ["composite-navigation.mjs", 5_355],
  ["catalog.mjs", 10_000],
  ["command.mjs", 2_200],
  ["context.mjs", 700],
  ["controllable-state.mjs", 600],
  ["dialog.mjs", 21_500],
  ["dismissable-layer.mjs", 4_250],
  ["drag-and-drop.mjs", 12_650],
  ["error-summary.mjs", 4_500],
  ["field-wiring.mjs", 2_700],
  ["form.mjs", 2_300],
  ["id.mjs", 2_400],
  ["inert-outside.mjs", 3_350],
  ["interaction-modality.mjs", 3_300],
  ["focus.mjs", 5_850],
  ["focus-scope.mjs", 6_300],
  ["focus-guards.mjs", 5_800],
  ["history.mjs", 3_400],
  ["hover.mjs", 2_200],
  ["live-region.mjs", 2_400],
  ["locale.mjs", 2_100],
  ["long-press.mjs", 8_175],
  ["measure.mjs", 2_400],
  // Styled entries statically import the shared dist/style.css, so their
  // budgets cover the packaged stylesheet alongside their JavaScript.
  ["motion.mjs", 5_600],
  ["move.mjs", 5_050],
  ["pointer-grace.mjs", 1_800],
  ["portal.mjs", 1_700],
  ["positioner.mjs", 7_100],
  ["presence.mjs", 2_800],
  ["progress.mjs", 3_100],
  ["press.mjs", 5_950],
  ["scroll-lock.mjs", 3_150],
  ["shortcut.mjs", 7_400],
  ["sortable.mjs", 17_000],
  ["spatial-navigation.mjs", 3_725],
  ["theme.mjs", 5_200],
  ["theme-scope.mjs", 2_600],
  ["transition.mjs", 3_000],
  ["typeahead.mjs", 2_000],
  ["virtualizer.mjs", 9_500],
  ["primitive.mjs", 800],
  ["visually-hidden.mjs", 3_700],
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
