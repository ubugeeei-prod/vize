<script setup lang="ts">
import { ref } from 'vue'

const primaryLabel = ref('primary')
const secondaryLabel = ref('secondary')
</script>

<art title="Button" component="./Child.vue">
  <variant name="Primary" default>
    <Child>{{ primaryLabel }}</Child>
  </variant>
  <variant name="Secondary">
    <Child>{{ secondaryLabel }}</Child>
  </variant>
</art>
