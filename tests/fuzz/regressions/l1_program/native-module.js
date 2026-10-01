// A complete ordinary Program with real statement children.
import { ref } from 'vue';
export const count = ref(0);
export function advance(values) {
  for (const value of values) {
    if (value > 0) count.value += value;
  }
  return count.value;
}
