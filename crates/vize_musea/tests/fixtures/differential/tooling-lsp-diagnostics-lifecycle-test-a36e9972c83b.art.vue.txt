<script setup lang="ts">
import { ref } from 'vue'

const primaryLabel = ref('primary')
</script>

<art title="Button" component="./Child.vue">
  <variant name="Primary" default>
    <Child>{{ primaryLabel }}</Child>
  </variant>
</art>
