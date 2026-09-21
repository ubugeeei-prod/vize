<script setup lang="ts">
const primaryLabel = ref('primary')
const secondaryLabel = ref('secondary')
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
<Button>{{ primaryLabel }}</Button>
  </variant>
  <variant name="Secondary">
<Button>{{ secondaryLabel }}</Button>
  </variant>
</art>
