<script setup lang="ts">
import { imported, imported as importedAlias } from './format'
const simpleLabel: string = 'typed'
function format(value: string, precision: number): string {
  return value.repeat(precision)
}
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary">
    <p>ラベル → {{ simpleLabel }} ・ 桁 ✅ {{ format('art', 2) }} {{ imported(10, 16) }} {{ importedAlias(10, 16) }}</p>
  </variant>
</art>
