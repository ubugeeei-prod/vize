<script setup lang="ts">
import { Statistic } from "../index.ts";
</script>

<art title="Statistic" status="ready">
  <variant name="Default" default>
    <Statistic title="Revenue" :value="1280" prefix="$" suffix="/mo" />
  </variant>
  <variant name="Negative">
    <Statistic title="Refunds" :value="-240" prefix="$" />
  </variant>
  <variant name="Compact">
    <Statistic title="Requests" :value="1200" notation="compact" :maximum-fraction-digits="1" />
  </variant>
</art>
