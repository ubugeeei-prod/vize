<script setup lang="ts">
import { ref } from 'vue'

const label = ref('label')
</script>

<art title="Button" component="./Child.vue">
  <variant name="Primary" default>
    <Child>{{ label }}</Child>
  </variant>
</art>
