<script setup lang="ts">
import { useColorMode } from "@vueuse/core";
import { computed } from "vue";

interface Props {
  text?: string;
  shadowColor?: string;
}

const props = withDefaults(defineProps<Props>(), {
  text: "Fast",
});

const isDark = computed(() => useColorMode().value === "dark");
const shadowColor = computed(() => (isDark.value ? "white" : "black"));
</script>

<template>
  <div class="flex h-56 w-full flex-col items-center justify-center gap-2">
    <h1 class="text-8xl leading-none font-extrabold tracking-tighter text-balance">
      Ship
      <LineShadowText
        class="italic"
        :shadow-color="props.shadowColor || shadowColor"
      >
        {{ props.text }}
      </LineShadowText>
    </h1>
  </div>
</template>
