<script setup lang="ts">
import { computed, ref } from "vue";
defineArt("./Counter.vue", { title: "Counter" });
const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>

<art>
  <variant name="First">
    <Counter :count="doubled" />
  </variant>
  <variant name="Second">
    <Counter :count="count" />
  </variant>
</art>
