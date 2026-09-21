<script setup lang="ts">
import { ref } from "vue";

defineArt("./Counter.vue", { title: "Counter" });

const count = ref(0);
</script>

<art>
  <variant name="Initial" default>
    <Counter :count="count" />
  </variant>
  <variant name="Interactive">
    <Counter :count="count" />
  </variant>
</art>
