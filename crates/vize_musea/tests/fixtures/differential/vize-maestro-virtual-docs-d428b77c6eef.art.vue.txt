<script setup lang="ts">
const primaryLabel = ref('primary')
const secondaryLabel = ref('secondary')
</script>

<art title="Button" component="./Button.vue">
  <variant name="Primary" default>
    <Button :label="primaryLabel" />
  </variant>
  <variant name="Secondary">
    <Button :label="secondaryLabel" />
  </variant>
</art>
