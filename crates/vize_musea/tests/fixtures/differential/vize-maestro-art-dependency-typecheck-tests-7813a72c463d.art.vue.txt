<script setup lang="ts">
defineArt("@ui/Button.vue", { title: "Button" });
</script>
<art>
  <variant name="First" default>
    <Button :label="123" />
  </variant>
  <variant name="Second">
    <Button :label="456" />
  </variant>
</art>