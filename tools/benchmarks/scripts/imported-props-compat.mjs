import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";

/** Preserve optional fields and unsuccessful rows through both public APIs. */
export function compatibility(native, root) {
  const sources = [
    '<script setup lang="ts">definePageMeta({ name: "compatibility" }); const props = withDefaults(defineProps<{ value?: string }>(), { value: "default" })</script><template><section>{{ props.value }}</section></template><style scoped>.value { color: red }</style><docs lang="json">{"value":1}</docs>',
    '<script>export default { name: "CompatibilityProbe" }</script><template><div /></template>',
    "<template><div></template>",
    '<script setup lang="ts">import type { Missing } from "./missing-compatibility-type"; defineProps<Missing>()</script><template><div /></template>',
  ];
  const files = sources.map((source, index) => ({
    path: path.join(root, `Compatibility${index}.vue`),
    source,
  }));
  const options = {
    includeStyles: true,
    includeCustomBlocks: true,
    includeMacroArtifacts: true,
    includeHashes: true,
    includeSourceMap: true,
    inlineTemplate: true,
    nuxtPageMeta: true,
  };
  const packets = [
    native.compileSfcBatchWithResults(files, options),
    native.compileSfcBatchForTargets(files, options, [{ ssr: false }, { ssr: true }]),
  ];
  assert.equal(packets[0].results.length, files.length);
  assert(packets[0].results.some((row) => row.errors.length > 0));
  assert(packets[0].results[0].styles.length > 0);
  assert(packets[0].results[0].customBlocks.length > 0);
  assert(packets[0].results[0].macroArtifacts.length > 0);
  for (const packet of packets) {
    delete packet.timeMs;
    for (const target of packet.targets ?? []) delete target.timeMs;
  }
  return { sha256: createHash("sha256").update(JSON.stringify(packets)).digest("hex"), packets };
}
