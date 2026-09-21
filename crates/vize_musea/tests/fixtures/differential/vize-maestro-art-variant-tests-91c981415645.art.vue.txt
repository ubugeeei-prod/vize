<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });

const broken: number = "not a number";
</script>

<art>
  <variant name="Primary" default>
    <Button :label="broken" />
  </variant>
</art>
