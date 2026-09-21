<script setup lang="ts">
function format(value: string, precision: number): string { return value.repeat(precision) }
format('script', )
</script>

<art title="Button" component="./Button.vue">
  <variant name="Empty">
    <p>Nothing to call</p>
  </variant>
  <variant name="Primary">
    <p>{{ format('art', ) }}</p>
  </variant>
</art>
