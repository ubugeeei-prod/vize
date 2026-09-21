<script setup lang="ts">
const simpleLabel: string = "typed"
const props = defineProps<{ title: string }>()
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <p>{{ simpleLabel }} {{ props.title }}</p>
  </variant>
</art>
