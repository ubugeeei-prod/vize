<script setup lang="ts">
import { defineStateModel } from "../src/index.ts";

type CartState = {
  readonly items: readonly string[];
};

type CartAction = { readonly type: "add"; readonly sku: string } | { readonly type: "clear" };

export const cartModel = defineStateModel({
  key: "cart.session",
  source: "./CartPanel.vue",
  version: 1,
  initialState: { items: [] } satisfies CartState,
  reducer(state, action: CartAction): CartState {
    switch (action.type) {
      case "add":
        return { items: [...state.items, action.sku] };
      case "clear":
        return { items: [] };
    }
  },
});
</script>

<template>
  <section>{{ cartModel.key }}</section>
</template>
