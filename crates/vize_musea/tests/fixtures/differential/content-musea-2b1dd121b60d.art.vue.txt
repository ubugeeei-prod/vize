<script setup lang="ts">
import { computed, ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>

<art>
  <variant name="Base" default>
    <Counter :count="count" />
  </variant>
  <variant name="Doubled">
    <Counter :count="doubled" />
  </variant>
</art>
