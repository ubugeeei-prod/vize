<!-- @inferTemplateDollarAttrs true -->
<script setup lang="ts">
import { useAttrs } from "vue";

declare module "vue" {
  interface ComponentCustomProperties {
    $attrs: {
      /** **Invoice total** in minor currency units.
       * @example
       * invoiceTotal: 1200
       */
      invoiceTotal: number;
    };
  }
}

const ownAttrs = useAttrs();
ownAttrs.invoiceTotal;
</script>

<template>{{ $attrs.invoiceTotal }}</template>
