import { computed, ref, watch, type Ref } from "vue";

export type ExperimentalScope = "compiler" | "typechecker" | "croquis";
export type ExperimentalKey =
  | "experimentalInTagComments"
  | "experimentalPatternedTemplate"
  | "experimentalSelfComponent"
  | "experimentalStrictSlotChildren";
export type ExperimentalOptions = Partial<Record<ExperimentalKey, boolean>>;

export const EXPERIMENTAL_FEATURES: ReadonlyArray<{
  key: ExperimentalKey;
  label: string;
  scopes: readonly ExperimentalScope[];
  source: string;
}> = [
  {
    key: "experimentalInTagComments",
    label: "In-tag comments",
    scopes: ["compiler", "typechecker", "croquis"],
    source: `<script setup lang="ts">
const label = 'Hello'
</script>

<template>
  <button
    // Button caption
    :aria-label="label"
  >{{ label }}</button>
</template>`,
  },
  {
    key: "experimentalPatternedTemplate",
    label: "Patterned templates",
    scopes: ["compiler", "croquis"],
    source: `<script setup lang="ts">
const status = 'ready'
</script>

<template v-match="status">
  <p v-when="'ready'">Ready</p>
  <p v-when="_">Waiting</p>
</template>`,
  },
  {
    key: "experimentalSelfComponent",
    label: "Self component",
    scopes: ["compiler"],
    source: `<script setup lang="ts">
defineProps<{ depth: number }>()
</script>

<template>
  <Self v-if="depth > 0" :depth="depth - 1" />
  <span v-else>Leaf</span>
</template>`,
  },
  {
    key: "experimentalStrictSlotChildren",
    label: "Strict slot children",
    scopes: ["typechecker"],
    source: `<script setup lang="ts">
const ButtonSlot = {} as {
  readonly __vizeSlots?: { default: () => HTMLButtonElement | [HTMLButtonElement] }
}
</script>

<template>
  <ButtonSlot><input /></ButtonSlot>
</template>`,
  },
];

export function featuresFor(scope: ExperimentalScope) {
  return EXPERIMENTAL_FEATURES.filter((feature) => feature.scopes.includes(scope));
}

export function parseExperimentalOptions(
  value: unknown,
  scope: ExperimentalScope,
): ExperimentalOptions {
  if (!value || typeof value !== "object" || Array.isArray(value)) return {};
  return Object.fromEntries(
    featuresFor(scope).map(({ key }) => [
      key,
      Object.hasOwn(value, key) && Reflect.get(value, key) === true,
    ]),
  );
}

export function useExperimentalFeatures(scope: ExperimentalScope, source: Ref<string>) {
  const options = ref<ExperimentalOptions>({});
  const storageKey = "vize-experimentals-" + scope;
  try {
    options.value = parseExperimentalOptions(
      JSON.parse(localStorage.getItem(storageKey) ?? "{}"),
      scope,
    );
  } catch {
    // Storage is optional in private browsing and server rendering.
  }
  watch(
    options,
    (value) => {
      try {
        localStorage.setItem(storageKey, JSON.stringify(value));
      } catch {
        /* Keep in-memory controls usable when storage is unavailable. */
      }
    },
    { deep: true },
  );

  function loadExample(key: string) {
    const feature = featuresFor(scope).find((item) => item.key === key);
    if (!feature) return;
    options.value = { ...options.value, [feature.key]: true };
    source.value = feature.source;
  }
  return {
    options,
    loadExample,
    enabled: computed(() => Object.values(options.value).some(Boolean)),
  };
}
