import assert from "node:assert/strict";
import path from "node:path";

import { test } from "vite-plus/test";
import type { Component } from "vue";

import { parseStoryArt, storyVariantModuleId } from "../../scripts/story-variants.ts";
import { renderAndHydrate } from "../testing/ssr-hydration.ts";
import { uiStoryTestbedInventory } from "./story-testbed-inventory.ts";

const storySources = import.meta.glob<string>("../families/**/*.art.vue", {
  query: "?raw",
  import: "default",
  eager: true,
});

const readyStories = uiStoryTestbedInventory.filter((entry) =>
  entry.artifacts.some(
    (artifact) => artifact.surface === "musea-story" && artifact.status === "ready",
  ),
);

test("every Musea story file is a ready story of its catalogued family", () => {
  assert.ok(readyStories.length > 0, "at least one family ships a ready story");
  assert.deepEqual(
    Object.keys(storySources)
      .map((key) => `src/${key.replace(/^\.\.\//, "")}`)
      .sort(),
    readyStories.map((entry) => entry.storyFile).sort(),
  );
});

for (const entry of readyStories) {
  test(`${entry.canonicalName} story variants render through SSR and hydrate cleanly`, async () => {
    const source = storySources[`../${entry.storyFile.replace(/^src\//, "")}`];
    assert.ok(source, `${entry.storyFile} must exist`);
    const story = parseStoryArt(source, entry.storyFile);
    assert.equal(story.title, entry.title, "the story title follows the catalog");
    assert.equal(story.status, "ready");
    assert.ok(story.variants.length >= 2, "a story covers more than one state");
    assert.equal(story.variants.filter((variant) => variant.isDefault).length, 1);
    const names = story.variants.map((variant) => variant.name);
    assert.equal(new Set(names).size, names.length, "variant names are unique");
    assert.ok(
      names.every((name) => name.length > 0),
      "every variant is named",
    );

    for (const [index, variant] of story.variants.entries()) {
      const id = storyVariantModuleId(path.resolve(entry.storyFile), index);
      const module = (await import(/* @vite-ignore */ id)) as { default: Component };
      const { html, dispose } = await renderAndHydrate(module.default);
      try {
        assert.ok(html.length > 0, `${variant.name} renders markup`);
      } finally {
        dispose();
      }
    }
  });
}
