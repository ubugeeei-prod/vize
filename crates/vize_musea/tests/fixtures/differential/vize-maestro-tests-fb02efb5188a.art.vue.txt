<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="Primary" default>
    <Button :variant="123" />
  </variant>
</art>