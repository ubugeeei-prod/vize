<!-- Error, success, and not-found results. Only the error is an alert. -->
<script setup lang="ts">
import { Result } from "../index.ts";
</script>

<template>
  <div>
    <Result status="success" title="Payment received" description="A receipt is on its way." />
    <Result status="error" title="Payment failed" description="The card was declined.">
      <button type="button">Try another card</button>
    </Result>
    <Result status="not-found" title="Page not found" />
  </div>
</template>
