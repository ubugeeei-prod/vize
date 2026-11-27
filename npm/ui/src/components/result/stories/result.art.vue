<script setup lang="ts">
import { Result } from "../index.ts";
</script>

<art title="Result" status="ready">
  <variant name="Default" default>
    <Result status="info" title="Check your inbox" description="We sent a confirmation link." />
  </variant>
  <variant name="Error">
    <Result status="error" title="Payment failed" description="The card was declined." />
  </variant>
  <variant name="Not found">
    <Result status="not-found" title="Page not found" />
  </variant>
  <variant name="Empty">
    <Result status="empty" title="No results" description="Try a different query." />
  </variant>
</art>
