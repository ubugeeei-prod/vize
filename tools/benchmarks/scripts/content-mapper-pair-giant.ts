/** A giant authored component with a large script and distinct typed prop uses. */
import { COMPILER_OPTIONS } from "./scale-project-generator.ts";

export function giantComponentProps(blocks: number): Map<string, string> {
  if (!Number.isSafeInteger(blocks) || blocks < 1)
    throw new Error("giant component prop blocks must be a positive integer");
  const script = ['<script setup lang="ts">', 'import Child from "./Child.vue";'];
  const template = ["<template>"];
  for (let index = 0; index < blocks; index++) {
    script.push(`const value${index} = ${index};`);
    template.push(`  <Child :count="value${index}" />`);
  }
  script.push("</script>");
  template.push("</template>");
  return new Map([
    ["src/App.vue", `${script.join("\n")}\n${template.join("\n")}\n`],
    [
      "src/Child.vue",
      '<script setup lang="ts">\ndefineProps<{ count: number }>();\n</script>\n<template><div /></template>\n',
    ],
    [
      "tsconfig.json",
      JSON.stringify({ compilerOptions: COMPILER_OPTIONS, include: ["src/**/*.vue"] }),
    ],
  ]);
}

/** Existing non-giant project entries belong to the generated type-check lane. */
export function giantBlockSizes(projects: string): number[] {
  return projects
    .split(",")
    .filter((entry) => entry.startsWith("giant:"))
    .map((entry) => {
      const blocks = Number(entry.slice("giant:".length));
      if (!Number.isSafeInteger(blocks) || blocks < 1)
        throw new Error("giant component prop blocks must be a positive integer");
      return blocks;
    });
}
