<script setup lang="ts">
const schema = { fields: [] as string[] }
function handleSubmit() {}
</script>

<art>
  <variant name="Default" default>
    <AfsForm :schema="schema" @submit="handleSubmit" />
  </variant>
</art>
