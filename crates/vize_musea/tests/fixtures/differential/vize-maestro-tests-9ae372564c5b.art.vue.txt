<script setup lang="ts">
const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>

<art title="Counter" component="./Counter.vue">
  <variant name="Default">
<Counter :count="" />
  </variant>
</art>
