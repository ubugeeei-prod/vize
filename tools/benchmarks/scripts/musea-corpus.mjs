/**
 * Pinned `.art.vue` corpus for the Musea benchmark lane (#3464).
 *
 * `@vizejs/vite-plugin-musea` does its build-time work per art file, and every
 * branch it takes is decided by that file's shape: whether metadata arrives via
 * the `defineArt` macro or legacy `<art>` attributes, how many variants there
 * are, what the `<script setup>` block imports and destructures, and how many
 * `<style>` blocks survive the language filter. A corpus of one repeated file
 * would measure one path through the plugin and let the JIT specialise on it,
 * so the generator below varies those axes on a fixed schedule keyed to the
 * file index. Same index, same bytes, on every machine and every run — the
 * numbers are only reproducible if the input is.
 *
 * The mix is modelled on `examples/vite-musea`, the only gallery in the repo:
 * component-per-file, a default variant plus a couple of states, a `<script
 * setup>` that pulls tokens in from a sibling module, and a scoped style block.
 *
 * Nothing here imports the plugin, Vite, or the native binding, so the shape of
 * the corpus can be asserted in a clean checkout with no build artifacts.
 */

import { mkdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Files in the pinned corpus. 240 art files carry 720 variants, which is the
 * size of a mid-sized real gallery and keeps every measured stage well clear of
 * the timer noise floor while a full lane still runs in seconds. Pinned rather
 * than derived so two runs of the lane are comparable by construction; override
 * with `--files` when profiling, and the run reports the count it used.
 */
export const MUSEA_CORPUS_FILE_COUNT = 240;

/** Sibling modules the corpus imports from; written alongside the art files. */
export const MUSEA_CORPUS_SUPPORT_FILES = ["bench-tokens.ts", "styles/bench-tokens.css"];

/** Variants per file, cycled by index: 2, 3, then 4. */
function variantCount(index) {
  return 2 + (index % 3);
}

/**
 * Every third file declares `<art>` interaction attributes, which is the only
 * input to `extractCustomArtMetadata`'s parse-and-dedupe path in
 * `src/plugin/art-processing.ts`.
 */
function artTagAttributes(index) {
  if (index % 3 !== 0) {
    return "";
  }
  return ' action-events="click,focus,click" capture-mousemove';
}

/**
 * One file in five keeps the legacy `<art title="..." component="...">` shape
 * instead of the `defineArt` macro. Both are supported and they reach the
 * metadata through different code, so both belong in the corpus.
 */
function usesLegacyArtAttributes(index) {
  return index % 5 === 4;
}

function componentName(index) {
  return `BenchComponent${index}`;
}

function scriptSetupBlock(index) {
  const component = componentName(index);
  const legacy = usesLegacyArtAttributes(index);
  const defineArtCall = legacy
    ? ""
    : `defineArt("./${component}.vue", {
  title: "${component}",
  description: "Generated art file ${index} for the Musea benchmark lane.",
  category: "Bench",
  status: "${["ready", "draft", "deprecated"][index % 3]}",
  tags: ["bench", "musea", "variant-${index % 7}"],
  order: ${index},
});

`;

  // A side-effect import with a relative specifier is the input to
  // `rewriteRelativeImportStatement`; the multi-line named import is the input
  // to `splitTopLevelCommaList` and `collectImportedNames`; the two
  // destructuring forms below drive `collectObjectDestructuredNames` and
  // `collectArrayDestructuredNames`. All four live in `src/art-module.ts` and
  // run on every `load` of every art file.
  return `<script setup lang="ts">
import { computed, ref } from "vue";
import ${component} from "./${component}.vue";
import {
  benchPrimaryTokens,
  benchSurfaceTokens,
  benchToneFor,
} from "./bench-tokens";
import "./styles/bench-tokens.css";

${defineArtCall}const { label, tone } = benchPrimaryTokens;
const [firstSurface, secondSurface] = benchSurfaceTokens;
const pressed = ref(false);
const caption = computed(() => \`\${label} / \${tone} / ${index}\`);

function toggle(): void {
  pressed.value = !pressed.value;
}
</script>`;
}

function variantBlock(index, variantIndex) {
  const component = componentName(index);
  const name = ["Default", "Pressed", "Muted", "Compact"][variantIndex];
  const attributes = [
    ` name="${name}"`,
    variantIndex === 0 ? " default" : "",
    variantIndex === 2 ? " skip-vrt" : "",
  ].join("");
  const surface = variantIndex % 2 === 0 ? "firstSurface" : "secondSurface";
  return `  <variant${attributes}>
    <${component} :pressed="pressed" :tone="benchToneFor(${variantIndex})" :surface="${surface}" @click="toggle">
      {{ caption }}
    </${component}>
  </variant>`;
}

function artBlock(index) {
  const legacy = usesLegacyArtAttributes(index);
  const legacyAttributes = legacy
    ? ` title="${componentName(index)}" component="./${componentName(index)}.vue" category="Bench"`
    : "";
  const variants = Array.from({ length: variantCount(index) }, (_, variantIndex) =>
    variantBlock(index, variantIndex),
  );
  return `<art${legacyAttributes}${artTagAttributes(index)}>
${variants.join("\n\n")}
</art>`;
}

/**
 * Two style blocks, one of which `extractStyleBlocks` must discard for its
 * `lang`. The filter runs on every file, so the discarded block is part of the
 * measured work rather than padding.
 */
function styleBlocks(index) {
  return `<style scoped>
.bench-component-${index} {
  color: var(--bench-primary);
  padding: ${index % 8}px ${(index % 5) + 1}px;
  border-radius: ${index % 4}px;
}

.bench-component-${index} .label {
  font-weight: ${500 + (index % 3) * 100};
}
</style>

<style lang="scss" scoped>
.bench-component-${index} {
  &:hover {
    color: var(--bench-accent);
  }
}
</style>`;
}

/**
 * The exact bytes of one corpus art file. Pure function of the index: no
 * timestamps, no randomness, no absolute paths.
 */
export function createArtFileSource(index) {
  return `${scriptSetupBlock(index)}

${artBlock(index)}

${styleBlocks(index)}
`;
}

/** The `.vue` component each art file targets, so the imports resolve. */
export function createComponentSource(index) {
  return `<script setup lang="ts">
defineProps<{ pressed: boolean; tone: string; surface: string }>();
defineEmits<{ click: [] }>();
</script>

<template>
  <button class="bench-component-${index}" :data-tone="tone" :data-surface="surface">
    <span class="label"><slot /></span>
  </button>
</template>
`;
}

/** The token module every art file imports from. */
export function createTokenModuleSource() {
  return `export const benchPrimaryTokens = { label: "Bench", tone: "primary" };
export const benchSurfaceTokens = ["surface-raised", "surface-sunken"];
export function benchToneFor(index: number): string {
  return benchSurfaceTokens[index % benchSurfaceTokens.length];
}
`;
}

/** The stylesheet every art file side-effect imports. */
export function createTokenStyleSource() {
  return `:root {
  --bench-primary: #2f6feb;
  --bench-accent: #eb2f6f;
}
`;
}

/**
 * Write the corpus into `directory`, replacing whatever was there.
 *
 * The directory must be a caller-chosen fixed path. The plugin derives scope
 * ids and virtual module ids from absolute file names, so a per-run temporary
 * root would change the generated modules between runs and make output
 * comparison impossible (#3464); `os.tmpdir()` is not stable under
 * `nix develop`, which sets a fresh `TMPDIR` per invocation.
 *
 * Returns the absolute art-file paths in index order plus the corpus byte size.
 */
export function writeMuseaCorpus(directory, fileCount = MUSEA_CORPUS_FILE_COUNT) {
  rmSync(directory, { recursive: true, force: true });
  mkdirSync(join(directory, "styles"), { recursive: true });

  writeFileSync(join(directory, "bench-tokens.ts"), createTokenModuleSource());
  writeFileSync(join(directory, "styles", "bench-tokens.css"), createTokenStyleSource());

  const files = [];
  let bytes = 0;
  for (let index = 0; index < fileCount; index += 1) {
    const artPath = join(directory, `${componentName(index)}.art.vue`);
    writeFileSync(artPath, createArtFileSource(index));
    writeFileSync(join(directory, `${componentName(index)}.vue`), createComponentSource(index));
    files.push(artPath);
    bytes += statSync(artPath).size;
  }

  return { files, bytes };
}

/**
 * How many variants the corpus declares in total, without writing it. The lane
 * reports this next to the file count because per-variant codegen is the part
 * of `load` that scales with it.
 */
export function museaCorpusVariantCount(fileCount = MUSEA_CORPUS_FILE_COUNT) {
  let total = 0;
  for (let index = 0; index < fileCount; index += 1) {
    total += variantCount(index);
  }
  return total;
}
