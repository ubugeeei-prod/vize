import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import path from "node:path";

import { compileSfc, type SfcCompileOptionsNapi } from "@vizejs/native";

import { rendererFixtures } from "./renderer-fixtures.ts";

interface RendererLane {
  /** Stable lane name emitted in CI diagnostics. */
  readonly name: "dom" | "ssr" | "vapor";
  /** Native SFC compiler options that distinguish this lane. */
  readonly options: Readonly<Pick<SfcCompileOptionsNapi, "ssr" | "vapor">>;
}

const rendererLanes: readonly RendererLane[] = [
  { name: "dom", options: { ssr: false, vapor: false } },
  { name: "ssr", options: { ssr: true, vapor: false } },
  { name: "vapor", options: { ssr: false, vapor: true } },
];

const inlineFixtures = [
  ...rendererFixtures,
  {
    filename: "SpatialNavigationConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { createCollectionRegistry } from "./collection.ts";
import { useSpatialNavigation } from "./spatial-navigation.ts";

const registry = createCollectionRegistry<string, string>();
registry.register({ key: "alpha", value: "Alpha", textValue: "Alpha", order: 0 });
registry.register({ key: "bravo", value: "Bravo", textValue: "Bravo", order: 1 });
const navigation = useSpatialNavigation({
  registry,
  focusBehavior: "logical",
  getRect: ({ key }) => ({
    bottom: 100,
    height: 100,
    left: key === "alpha" ? 0 : 120,
    right: key === "alpha" ? 100 : 220,
    top: 0,
    width: 100,
  }),
});
</script>

<template>
  <div
    v-bind="navigation.spatialNavigationProps"
    role="grid"
    tabindex="0"
    :aria-activedescendant="registry.activeKey.value ? 'cell-' + registry.activeKey.value : undefined"
  >
    <div v-for="key in ['alpha', 'bravo']" :key="key" :id="'cell-' + key" role="gridcell">
      {{ key }}
    </div>
  </div>
</template>
`,
  },
  {
    filename: "CompositeNavigationConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { createCollectionRegistry } from "./collection.ts";
import { useCompositeNavigation } from "./composite-navigation.ts";

const registry = createCollectionRegistry<string, string>();
registry.register({ key: "alpha", value: "Alpha", textValue: "Alpha", order: 0 });
registry.register({ key: "bravo", value: "Bravo", textValue: "Bravo", order: 1 });
const navigation = useCompositeNavigation({
  registry,
  focusStrategy: "active-descendant",
  getItemId: ({ key }) => "option-" + key,
});
</script>

<template>
  <div v-bind="navigation.getContainerProps()" role="listbox">
    <div
      v-for="key in ['alpha', 'bravo']"
      :key="key"
      v-bind="navigation.getItemProps(key)"
      role="option"
      :aria-selected="navigation.activeKey.value === key"
    >
      {{ key }}
    </div>
  </div>
</template>
`,
  },
  {
    filename: "TypeaheadConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { createCollectionRegistry } from "./collection.ts";
import { useTypeahead } from "./typeahead.ts";

const registry = createCollectionRegistry<string, string>();
registry.register({ key: "alpha", value: "Alpha", textValue: "Alpha" });
const typeahead = useTypeahead({ registry });
</script>

<template>
  <div
    v-bind="typeahead.typeaheadProps"
    :data-active="registry.activeKey.value || undefined"
    :data-query="typeahead.query.value || undefined"
    tabindex="0"
  >
    Typeahead target
  </div>
</template>
`,
  },
  {
    filename: "FocusConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { useFocusRing } from "./focus.ts";

const focus = useFocusRing({
  onFocus(event) {
    void event.isFocusVisible;
  },
});
</script>

<template>
  <button
    v-bind="focus.focusProps"
    type="button"
    :data-focus-visible="focus.isFocusVisible.value || undefined"
    :data-focused="focus.isFocused.value || undefined"
  >
    Focus target
  </button>
</template>
`,
  },
  {
    filename: "MoveConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { useMove } from "./move.ts";

const move = useMove({
  onMove(event) {
    void event.deltaX;
  },
});
</script>

<template>
  <div v-bind="move.moveProps" :data-moving="move.isMoving.value || undefined" tabindex="0">
    Move target
  </div>
</template>
`,
  },
  {
    filename: "HoverConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { useHover } from "./hover.ts";

const hover = useHover({
  onHoverStart(event) {
    void event.pointerType;
  },
});
</script>

<template>
  <div v-bind="hover.hoverProps" :data-hovered="hover.isHovered.value || undefined">
    Hover target
  </div>
</template>
`,
  },
  {
    filename: "LongPressConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { useLongPress } from "./long-press.ts";

const longPress = useLongPress({
  accessibilityDescription: "Hold for actions",
  onLongPress(event) {
    void event.pointerType;
  },
});
</script>

<template>
  <button
    v-bind="longPress.longPressProps"
    type="button"
    :data-long-pressed="longPress.isLongPressed.value || undefined"
  >
    Actions
  </button>
</template>
`,
  },
  {
    filename: "PressConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { usePress } from "./press.ts";

const press = usePress({
  onPress(event) {
    void event.pointerType;
  },
});
</script>

<template>
  <button
    v-bind="press.pressProps"
    type="button"
    :data-pressed="press.isPressed.value || undefined"
  >
    Activate
  </button>
</template>
`,
  },
] as const;

/**
 * Recursively collect authored Vue SFCs in deterministic path order.
 *
 * Generated output is deliberately excluded: this gate verifies the canonical,
 * inspectable source that consumers install and edit.
 */
async function collectSfcFiles(sourceRoot: string): Promise<readonly string[]> {
  const entries = await readdir(sourceRoot, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry): Promise<readonly string[]> => {
      const entryPath = path.join(sourceRoot, entry.name);
      if (entry.isDirectory()) return collectSfcFiles(entryPath);
      return entry.isFile() && entry.name.endsWith(".vue") ? [entryPath] : [];
    }),
  );

  return files.flat().sort((left, right) => left.localeCompare(right));
}

/** Format native diagnostics without losing the file and renderer lane. */
function formatDiagnostics(
  file: string,
  lane: RendererLane,
  diagnostics: readonly string[],
): string {
  return `${file} failed ${lane.name} compilation:\n${diagnostics
    .map((diagnostic) => `  - ${diagnostic}`)
    .join("\n")}`;
}

/**
 * Compile one component through a production renderer lane.
 *
 * The Vapor+SSR cross-product is intentionally absent. Vize currently falls
 * back to standard SSR for that combination, and issue #3134 tracks native
 * Vapor SSR as a release-blocking capability rather than treating fallback as
 * conformance.
 */
function verifyRendererLane(file: string, source: string, lane: RendererLane): void {
  const result = compileSfc(source, {
    filename: file,
    isTs: true,
    mode: "module",
    sourceMap: true,
    ...lane.options,
  });

  assert.equal(result.errors.length, 0, formatDiagnostics(file, lane, result.errors));
  assert.equal(result.warnings.length, 0, formatDiagnostics(file, lane, result.warnings));
  assert.ok(result.code.trim().length > 0, `${file} emitted empty ${lane.name} JavaScript`);

  if (lane.name === "vapor") {
    assert.match(
      result.code,
      /defineVaporComponent|__vaporRender|__vapor\s*:\s*true/,
      `${file} did not emit a Vapor component`,
    );
  } else {
    assert.doesNotMatch(
      result.code,
      /defineVaporComponent|__vaporRender|__vapor\s*:\s*true/,
      `${file} leaked Vapor output into the ${lane.name} lane`,
    );
  }

  if (lane.name === "ssr") {
    assert.match(
      result.code,
      /function ssrRender|@vue\/server-renderer/,
      `${file} did not emit an SSR render function`,
    );
  } else {
    assert.doesNotMatch(
      result.code,
      /function ssrRender|@vue\/server-renderer/,
      `${file} leaked SSR output into the ${lane.name} lane`,
    );
  }
}

const sourceRoots = process.argv.slice(2);
const resolvedRoots = sourceRoots.length > 0 ? sourceRoots : ["src"];
const sourceFiles = (
  await Promise.all(resolvedRoots.map((sourceRoot) => collectSfcFiles(path.resolve(sourceRoot))))
).flat();

assert.ok(sourceFiles.length > 0, `No Vue SFCs found below: ${resolvedRoots.join(", ")}`);

for (const file of sourceFiles) {
  const source = await readFile(file, "utf8");
  for (const lane of rendererLanes) verifyRendererLane(file, source, lane);
}
for (const fixture of inlineFixtures) {
  for (const lane of rendererLanes) {
    verifyRendererLane(fixture.filename, fixture.source, lane);
  }
}

console.log(
  JSON.stringify({
    check: "@vizejs/ui renderer conformance",
    sourceFiles: sourceFiles.length,
    inlineFixtures: inlineFixtures.length,
    compilations: (sourceFiles.length + inlineFixtures.length) * rendererLanes.length,
    lanes: rendererLanes.map((lane) => lane.name),
  }),
);
