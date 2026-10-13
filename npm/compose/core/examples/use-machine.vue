<script setup lang="ts">
import { useMachine } from "@vizejs/composable/use-machine";

type Step = "cart" | "shipping" | "review" | "placed";
type Action = "add" | "checkout" | "back" | "ship" | "place" | "restart";

const { state, context, nextEvents, send, can } = useMachine<Step, Action, { items: number }>({
  initial: "cart",
  context: { items: 0 },
  states: {
    cart: {
      on: {
        add: { target: "cart", action: (order) => ({ items: order.items + 1 }) },
        checkout: { target: "shipping", guard: (order) => order.items > 0 },
      },
    },
    shipping: { on: { back: "cart", ship: "review" } },
    review: { on: { back: "shipping", place: "placed" } },
    placed: { on: { restart: { target: "cart", action: () => ({ items: 0 }) } } },
  },
});

const labels: Record<Action, string> = {
  add: "Add item",
  checkout: "Go to shipping",
  back: "Back",
  ship: "Review order",
  place: "Place order",
  restart: "Start a new order",
};
</script>

<template>
  <div class="composable-example">
    <p>Walk through a checkout that only moves forward when each step is ready.</p>
    <output aria-live="polite">Step: {{ state }} · Items: {{ context.items }}</output>
    <div class="example-actions">
      <button v-for="event in nextEvents" :key="event" type="button" @click="() => send(event)">
        {{ labels[event] }}
      </button>
    </div>
    <p v-if="state === 'cart' && !can('checkout')">Add an item before checking out.</p>
  </div>
</template>
