<script setup lang="ts">
import { PrintButton } from "./print-button.ts";
</script>

<art title="Print Button" status="ready">
  <variant name="Default" default>
    <PrintButton />
  </variant>
  <variant name="Custom labels">
    <PrintButton idle-label="Print invoice" printing-label="Opening print dialog" />
  </variant>
  <variant name="Disabled">
    <PrintButton idle-label="Print invoice" disabled />
  </variant>
</art>
