<script setup lang="ts" isolate="false">
defineArt("./Button.vue", { title: "Button" });

const sharedLabel: string = "shared";
</script>

<art>
  <variant name="Primary" default>
    <Button :label="sharedLabel" />
  </variant>
  <variant name="Secondary">
    <Button :label="sharedLabel.toUpperCase()" />
  </variant>
</art>
