<script setup lang="ts" generic="T extends string">
const selected = await Promise.resolve('ready' as T)
function genericFormat(value: T): T { return value }
</script>
<art title="Generic">
  <variant name="Generic">
    <p>{{ genericFormat(selected) }}</p>
  </variant>
</art>
