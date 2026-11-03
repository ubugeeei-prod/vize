<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();
// The generated routes and `addRoute` may declare these names: no claim.
router.push({ name: "reports", params: { year: 2026 } });
router.push({ name: "invoices" });
</script>

<template>
  <!-- `order` is static, so its params are still checked. -->
  <RouterLink :to="{ name: 'order', params: { orderId: 1, lineId: 2 } }">order</RouterLink>
</template>
