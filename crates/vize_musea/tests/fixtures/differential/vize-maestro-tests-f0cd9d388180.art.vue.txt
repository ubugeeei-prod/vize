<script setup>
const count = ref(0)
</script>

<art title="Counter" component="./Counter.vue">
  <variant name="Interactive">
    <Counter :count="count" />
  </variant>
</art>