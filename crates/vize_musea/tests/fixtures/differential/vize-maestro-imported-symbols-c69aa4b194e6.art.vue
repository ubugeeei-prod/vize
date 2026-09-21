<script setup lang="ts">
import { format } from './format'
function decorate(value: string): string { return `[${value}]` }
</script>

<art title="Definition">
  <variant name="Secondary">
    <p>{{ decorate('art') }} {{ format('world') }}</p>
  </variant>
</art>
