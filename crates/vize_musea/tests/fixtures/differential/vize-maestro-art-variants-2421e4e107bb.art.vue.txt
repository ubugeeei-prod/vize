<script setup lang="ts">
const props = defineProps<{
  formatter: (value: number) => string
}>()
</script>
<art title="Macro">
  <variant name="Macro">
    <p>{{ props.formatter(1) }}</p>
  </variant>
</art>
