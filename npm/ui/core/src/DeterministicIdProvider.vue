<script setup lang="ts">
import { useId as useVueId } from "vue";

import {
  createDeterministicIdScope,
  provideDeterministicIdScope,
  useOptionalDeterministicIdScope,
} from "./deterministic-id.ts";

const { prefix = undefined, seed = undefined } = defineProps<{
  /**
   * Namespace prefix for this subtree.
   *
   * @default The parent prefix when nested; otherwise "vize"
   */
  readonly prefix?: string;

  /**
   * Stable request, island, or subtree seed.
   *
   * @default Vue's SSR- and hydration-stable useId() value
   */
  readonly seed?: string | number;
}>();

defineSlots<{
  default(props: { readonly namespace: string; readonly prefix: string }): unknown;
}>();

const vueSeed = useVueId();
const parent = useOptionalDeterministicIdScope();
const resolvedSeed = seed ?? vueSeed;
const scope =
  parent === undefined
    ? createDeterministicIdScope({
        prefix: prefix ?? "vize",
        seed: resolvedSeed,
      })
    : prefix === undefined
      ? parent.createChild({ seed: resolvedSeed })
      : parent.createChild({ prefix, seed: resolvedSeed });

provideDeterministicIdScope(scope);

defineExpose({ namespace: scope.namespace, prefix: scope.prefix });
</script>

<template>
  <slot :namespace="scope.namespace" :prefix="scope.prefix" />
</template>

<style scoped>
/* This provider renders no element and therefore owns no visual styles. */
</style>
