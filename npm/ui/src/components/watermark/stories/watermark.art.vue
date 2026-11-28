<script setup lang="ts">
import { Watermark } from "../index.ts";
</script>

<art title="Watermark" status="ready">
  <variant name="Default" default>
    <Watermark text="DRAFT">
      <p>Quarterly plan</p>
    </Watermark>
  </variant>
  <variant name="Confidential">
    <Watermark text="CONFIDENTIAL" :rotate="-32" :gap="220">
      <p>Board notes</p>
    </Watermark>
  </variant>
</art>
