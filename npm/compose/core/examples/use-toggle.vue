<script setup lang="ts">
import { useId } from "vue";
import { useToggle } from "@vizejs/composable/use-toggle";

const { state: expanded, toggle } = useToggle();
const detailsId = useId();
</script>

<template>
  <div class="composable-example">
    <button type="button" :aria-expanded="expanded" :aria-controls="detailsId" @click="toggle()">
      {{ expanded ? "Hide delivery details" : "Show delivery details" }}
    </button>
    <p v-if="expanded" :id="detailsId">Free delivery in 2–3 business days.</p>
    <output aria-live="polite">Details are {{ expanded ? "visible" : "hidden" }}.</output>
  </div>
</template>
