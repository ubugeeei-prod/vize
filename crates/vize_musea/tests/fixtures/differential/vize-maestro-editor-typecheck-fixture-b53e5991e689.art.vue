<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="Primary" default>
    <Button :label="123" />
  </variant>
</art>
