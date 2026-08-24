import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const thisGate = "tests/tooling/davinci-v-on-storage.test.ts";
const sourceExtensions = new Set([".html", ".md", ".rs", ".ts", ".tsx", ".vue"]);

// Baseline: 9e18d171c3ef3a16021dff4debeab21195f99017, immediately before the
// SmallVec change. Continue scanning the current Git-tracked corpus so future
// natural spellings force an intentional capacity/evidence update, while the
// marked synthetic boundary cases never justify their own chosen capacity.
const modifierSpelling = /@(?:keydown|keyup|keypress|click|[A-Za-z][\w:-]*)(?:\.[A-Za-z0-9_-]+)+/gu;
const syntheticBoundary =
  /\/\/ v-on-storage-synthetic:start[\s\S]*?\/\/ v-on-storage-synthetic:end/gu;

type Buckets = { options: number; event: number; keys: number };

function trackedNaturalSources(): Array<{ file: string; source: string }> {
  const tracked = spawnSync("git", ["ls-files", "-z"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(tracked.status, 0, tracked.stderr);

  return tracked.stdout
    .split("\0")
    .filter((file) => file !== "" && file !== thisGate && sourceExtensions.has(path.extname(file)))
    .map((file) => ({
      file,
      source: fs.readFileSync(path.join(repoRoot, file), "utf8").replace(syntheticBoundary, ""),
    }));
}

function classify(spelling: string): Buckets {
  const [name, ...modifiers] = spelling.slice(1).split(".");
  const keyboard = name === "keydown" || name === "keyup" || name === "keypress";
  const buckets: Buckets = { options: 0, event: 0, keys: 0 };

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

test("the natural committed v-on corpus fits the two-entry inline buckets", () => {
  const spellings = trackedNaturalSources().flatMap(({ source }) =>
    Array.from(source.matchAll(modifierSpelling), (match) => match[0]),
  );
  const maxima = spellings.map(classify).reduce<Buckets>(
    (max, buckets) => ({
      options: Math.max(max.options, buckets.options),
      event: Math.max(max.event, buckets.event),
      keys: Math.max(max.keys, buckets.keys),
    }),
    { options: 0, event: 0, keys: 0 },
  );

  assert.equal(spellings.length, 242, "update the measured corpus evidence intentionally");
  assert.deepEqual(maxima, { options: 2, event: 2, keys: 2 });
});

test("the corpus inventory excludes only marked synthetic storage boundaries", () => {
  const source = `natural @click.stop\n// v-on-storage-synthetic:start\nsynthetic @click.stop.prevent.self\n// v-on-storage-synthetic:end`;
  assert.deepEqual(
    Array.from(
      source.replace(syntheticBoundary, "").matchAll(modifierSpelling),
      (match) => match[0],
    ),
    ["@click.stop"],
  );
});
