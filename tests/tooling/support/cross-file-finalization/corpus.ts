/** A physical 241-SFC application: root, 40 feature components, 200 leaves. */
export function files(): Record<string, string> {
  const inputs: Record<string, string> = {};
  const features: string[] = [];
  const renders: string[] = [];
  for (let feature = 0; feature < 40; feature++) {
    const name = `Feature${String(feature).padStart(2, "0")}`;
    features.push(`import ${name} from './${name}.vue'`);
    renders.push(`<${name} />`);
    const imports: string[] = [];
    const leaves: string[] = [];
    for (let leaf = 0; leaf < 5; leaf++) {
      const child = `Leaf${String(feature).padStart(2, "0")}_${leaf}`;
      imports.push(`import ${child} from './${child}.vue'`);
      leaves.push(`<${child} value="日本語🙂" />`);
      const source = `<script setup lang="ts">\nconst props = defineProps<{ value?: string }>()\n</script>\n<template><p>{{ props.value }}</p></template>\n`;
      inputs[`${child}.vue`] = leaf % 2 === 0 ? source : source.replaceAll("\n", "\r\n");
    }
    inputs[`${name}.vue`] =
      `<script setup lang="ts">\n${imports.join("\n")}\n</script>\n<template><section>${leaves.join("")}</section></template>\n`;
  }
  inputs["App.vue"] =
    `<script setup lang="ts">\n${features.join("\n")}\n</script>\n<template><main>${renders.join("")}</main></template>\n`;
  return inputs;
}
