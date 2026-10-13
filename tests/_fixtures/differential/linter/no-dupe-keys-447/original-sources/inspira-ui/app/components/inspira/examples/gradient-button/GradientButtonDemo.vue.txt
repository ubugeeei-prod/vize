<script lang="ts" setup>
import { useColorMode } from "@vueuse/core";
import { computed } from "vue";

interface Props {
  duration?: number;
  borderWidth?: number;
  borderRadius?: number;
  blur?: number;
  bgColor?: string;
}

const props = defineProps<Props>();

const isDark = computed(() => useColorMode().value === "dark");
const bgColor = computed(() => (isDark.value ? "#000" : "#fff"));
</script>

<template>
  <div class="z-10 flex h-56 w-full flex-col items-center justify-center">
    <GradientButton
      :bg-color="props.bgColor || bgColor"
      :duration="props.duration"
      :border-width="props.borderWidth"
      :border-radius="props.borderRadius"
      :blur="props.blur"
    >
      Zooooooooooom 🚀
    </GradientButton>
  </div>
</template>
