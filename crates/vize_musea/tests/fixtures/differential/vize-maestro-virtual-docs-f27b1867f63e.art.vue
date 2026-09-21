<script setup lang="ts" isolate="false">
const count = ref(0)
</script>

<art title="Counter">
  <variant name="First">
    <Counter :count="count" />
  </variant>
  <variant name="Second">
    <Counter :count="count" />
  </variant>
</art>
