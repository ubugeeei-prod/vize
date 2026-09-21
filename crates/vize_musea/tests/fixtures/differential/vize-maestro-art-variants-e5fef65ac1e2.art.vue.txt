<script setup lang="ts" isolate="false">
function sharedFormat(value: boolean, count: number): string {
  return String(value).repeat(count)
}
</script>
<art title="Shared">
  <variant name="Shared">
    <p>{{ sharedFormat(true, 2) }}</p>
  </variant>
</art>
