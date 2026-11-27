<script setup lang="ts">
import { ColorWheel } from "../index.ts";
</script>

<art title="Color Wheel" status="ready">
  <variant name="Default" default>
    <ColorWheel label="Hue" />
  </variant>
  <variant name="Blue">
    <ColorWheel label="Hue" :model-value="220" />
  </variant>
  <variant name="Disabled">
    <ColorWheel label="Hue" :model-value="40" disabled />
  </variant>
</art>
