<script setup lang="ts">
import { FullscreenButton } from "./fullscreen-button.ts";
</script>

<art title="Fullscreen Button" status="ready">
  <variant name="Default" default>
    <FullscreenButton />
  </variant>
  <variant name="Custom labels">
    <FullscreenButton enter-label="View report fullscreen" exit-label="Exit fullscreen" />
  </variant>
  <variant name="Disabled">
    <FullscreenButton disabled />
  </variant>
</art>
