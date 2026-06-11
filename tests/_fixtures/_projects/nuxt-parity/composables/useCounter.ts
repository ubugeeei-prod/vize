import { type Ref, ref } from "vue";

export interface Counter {
  count: Ref<number>;
  increment: (by: number) => void;
}

// Auto-imported via `.nuxt/imports.d.ts`. The `step` argument is strongly
// typed so a wrong-typed call is a real TS2345, not silently `any`.
export function useCounter(step: number): Counter {
  const count = ref(0);
  const increment = (by: number): void => {
    count.value += by * step;
  };
  return { count, increment };
}
