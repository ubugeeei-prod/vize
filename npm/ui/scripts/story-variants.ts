import { readFile } from "node:fs/promises";
import path from "node:path";

import { parse } from "@vue/compiler-sfc";
import type { Plugin } from "vite";

/** One `<variant>` of a Musea story. */
export interface StoryVariant {
  readonly name: string;
  readonly isDefault: boolean;
  readonly template: string;
}

/** A parsed `*.art.vue` Musea story. */
export interface StoryArt {
  readonly title: string;
  readonly status: string | undefined;
  readonly scriptSetup: string;
  readonly variants: readonly StoryVariant[];
}

/**
 * Parse a Musea story with the Vue SFC parser: the `<art>` block is a custom
 * block, and its `<variant>` children are custom blocks of its content.
 */
export function parseStoryArt(source: string, filename: string): StoryArt {
  const { descriptor, errors } = parse(source, { filename });
  if (errors.length > 0) throw new Error(`${filename}: ${String(errors[0])}`);
  const art = descriptor.customBlocks.find((block) => block.type === "art");
  if (art === undefined) throw new Error(`${filename}: missing <art> block`);
  const { title, status } = art.attrs;
  if (typeof title !== "string") throw new Error(`${filename}: <art> needs a title`);
  const body = parse(art.content, { filename }).descriptor;
  return {
    title,
    status: typeof status === "string" ? status : undefined,
    scriptSetup: descriptor.scriptSetup?.content ?? "",
    variants: body.customBlocks
      .filter((block) => block.type === "variant")
      .map((block) => ({
        name: String(block.attrs.name ?? ""),
        isDefault: block.attrs.default === true,
        template: block.content,
      })),
  };
}

const variantModule = /\.art\.story-(\d+)\.vue$/;

/** Virtual SFC module id that renders one variant of the story at `artFile`. */
export function storyVariantModuleId(artFile: string, index: number): string {
  return artFile.replace(/\.art\.vue$/, `.art.story-${index}.vue`);
}

/**
 * Serve each story variant as its own SFC next to the story, so relative
 * imports resolve exactly as they do in the story's `<script setup>`.
 */
export function uiStoryVariants(): Plugin {
  return {
    name: "vize-ui-story-variants",
    enforce: "pre",
    resolveId(id, importer) {
      if (!variantModule.test(id)) return null;
      return path.isAbsolute(id) ? id : path.resolve(path.dirname(importer ?? "."), id);
    },
    async load(id) {
      const match = variantModule.exec(id);
      if (match === null) return null;
      const artFile = id.replace(variantModule, ".art.vue");
      const story = parseStoryArt(await readFile(artFile, "utf8"), artFile);
      const variant = story.variants[Number(match[1])];
      if (variant === undefined) throw new Error(`${artFile}: no variant ${match[1]}`);
      return `<script setup lang="ts">\n${story.scriptSetup}\n</script>\n\n<template>\n${variant.template}\n</template>\n`;
    },
  };
}
