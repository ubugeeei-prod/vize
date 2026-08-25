<script>
export default {
  name: "LegacyFilters",
  data() {
    return {
      amount: 12,
      order: {
        id: "a-1",
        status: "paid",
      },
    };
  },
};
</script>

<template>
  <section class="legacy-filters">
    <p>{{ order.id | normalizeId }}</p>
    <p>{{ amount | currency("$") | suffix(" USD") }}</p>
    <StatusBadge :type="order.status | statusFilter" />
    <slot :value="amount | currency"></slot>
  </section>
</template>
