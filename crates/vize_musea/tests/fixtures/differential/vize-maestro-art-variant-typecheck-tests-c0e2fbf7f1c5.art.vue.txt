<script setup lang="ts">
const label: string = "plain";
</script>

<art title="Plain">
  <variant name="Primary" default>
    <span>{{ label }}</span>
  </variant>
  <variant name="Secondary">
    <span>{{ label.toUpperCase() }}</span>
  </variant>
  <variant name="Tertiary">
    <span>{{ label.trim() }}</span>
  </variant>
</art>
