import assert from "node:assert/strict";
import fs from "node:fs";
import type { Context } from "@oxlint/plugins";
import { clearFileStateCache, getFileStateCacheStats } from "./file-state.ts";
import type { PatinaSettings } from "./model.ts";

export function qualifyCollectedCacheControls<T>(
  file: string,
  source: string,
  paths: string[],
  rules: Record<string, unknown>,
  settings: PatinaSettings,
  context: (filename: string, settings: PatinaSettings) => Context,
  observe: (
    name: string,
    ctx: Context,
    selected: Record<string, unknown>,
    expected: number,
    collected?: boolean,
  ) => T,
  fallback: T,
  withoutHtml: T,
): void {
  clearFileStateCache();
  const collectedSettings: PatinaSettings = { ...settings };
  const selected = structuredClone(rules);
  const collectedFirst = observe(
    "collector cold without hints",
    context(file, collectedSettings),
    selected,
    1,
    true,
  );
  assert.deepEqual(
    observe("collector resident warm", context(file, collectedSettings), selected, 0, true),
    collectedFirst,
  );
  (selected["vize/vue/attribute-hyphenation"] as unknown[])[1] = "never";
  assert.notDeepEqual(
    observe(
      "collector in-place option change",
      context(file, collectedSettings),
      selected,
      1,
      true,
    ),
    collectedFirst,
  );
  selected["vize/vue/attribute-hyphenation"] = "warn";
  const scalar = observe(
    "collector scalar override option reset",
    context(file, collectedSettings),
    selected,
    1,
    true,
  );
  assert.deepEqual(scalar, fallback);
  selected["vize/vue/no-v-html"] = "off";
  assert.deepEqual(
    observe(
      "collector file activation change",
      context(file, collectedSettings),
      selected,
      1,
      true,
    ),
    withoutHtml,
  );
  fs.writeFileSync(file, "\n\n" + source);
  assert.notDeepEqual(
    observe(
      "collector physical revision change",
      context(file, collectedSettings),
      selected,
      1,
      true,
    ),
    withoutHtml,
  );
  fs.writeFileSync(file, source);
  assert.deepEqual(
    observe(
      "collector physical revision reversion",
      context(file, collectedSettings),
      selected,
      1,
      true,
    ),
    withoutHtml,
  );
  collectedSettings.helpLevel = "full";
  observe(
    "collector settings revision change",
    context(file, collectedSettings),
    selected,
    1,
    true,
  );
  clearFileStateCache();
  for (const [index, filename] of paths.entries())
    observe(`collector LRU cold ${index}`, context(filename, collectedSettings), selected, 1, true);
  assert.deepEqual(getFileStateCacheStats(), { capacity: 128, entries: 128 });
  observe("collector LRU resident warm", context(paths[128], collectedSettings), selected, 0, true);
  observe("collector LRU evicted cold", context(paths[0], collectedSettings), selected, 1, true);
}
